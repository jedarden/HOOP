//! Session tailer for CLI sessions
//!
//! Discovers and parses `.jsonl` session files from CLI providers (Claude Code, Codex, OpenCode, Gemini, Aider).
//! Two-phase discovery: stat everything + sort by mtime, then parse in parallel.
//! 5-second background poll detects external edits.
//! Bootstrap interceptor aliases newly-found files back to existing session IDs.
//! Filter-by-cwd to scope sessions to the registered project.
//!
//! Per-project runtime (plan §4.3): each project gets its own session tailer
//! scoped to sessions whose cwd is under the project path.
//!
//! Multi-adapter support: each adapter implements SessionAdapter trait for
//! discovery and parsing of its session file format.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use hoop_schema::{
    ParsedSession, ParsedSessionKind,
    ParsedSessionMessagesItem, ParsedSessionMessagesItemUsage, ParsedSessionTotalUsage,
};
use crate::tag_join;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Check whether `cwd` falls under `project_path` using canonical comparison.
///
/// Both sides are resolved via `fs::canonicalize` to handle symlinks and alternate
/// mounts. If canonicalization fails (e.g. path doesn't exist on this host), the
/// raw path is used as a fallback so sessions from remote machines aren't silently
/// dropped.
fn cwd_matches_project(cwd: &str, project_path: &Path) -> bool {
    let resolved_cwd = std::fs::canonicalize(cwd).unwrap_or_else(|_| PathBuf::from(cwd));
    let resolved_project = std::fs::canonicalize(project_path).unwrap_or_else(|_| project_path.to_path_buf());
    let cwd_str = resolved_cwd.to_string_lossy();
    let project_str = resolved_project.to_string_lossy();
    if cwd_str == *project_str {
        return true;
    }
    cwd_str.starts_with(&*project_str)
        && cwd_str.as_ref().get(project_str.len()..=project_str.len()) == Some("/")
}

/// Events emitted by the session tailer
#[derive(Debug, Clone)]
pub enum SessionEvent {
    /// One or more sessions were discovered/updated
    ConversationsUpdated { sessions: Vec<ParsedSession> },
    /// A new session file was bound to an existing session ID
    SessionBound { id: String, file_path: String },
    /// A session was bound to a bead via the needle tag-join resolver (dual-identity invariant §B1).
    /// Emitted exactly once per (bead_id, provider_session_id) pair — subsequent appearances are no-ops.
    TagJoinBound {
        /// HOOP internal stable session ID
        session_id: String,
        /// Provider-native session ID (e.g. Claude Code's UUID)
        provider_session_id: String,
        bead_id: String,
        worker: String,
        strand: Option<String>,
        /// Timestamp of the first join
        ts: DateTime<Utc>,
    },
    /// An error occurred
    Error(String),
}

/// Message metadata from Claude Code JSONL
#[derive(Debug, Deserialize)]
struct ClaudeMessage {
    /// Message role
    role: String,
    /// Message content
    content: Option<serde_json::Value>,
    /// Model used (for assistant messages)
    #[allow(dead_code)]
    model: Option<String>,
    /// Token usage
    usage: Option<ClaudeUsage>,
    /// Timestamp (ISO 8601)
    timestamp: Option<String>,
}

/// Token usage from Claude Code
#[derive(Debug, Deserialize, Clone)]
struct ClaudeUsage {
    /// Input tokens
    pub input_tokens: Option<u64>,
    /// Output tokens
    pub output_tokens: Option<u64>,
    /// Cache read tokens
    pub cache_read_tokens: Option<u64>,
    /// Cache write tokens
    pub cache_creation_tokens: Option<u64>,
}

impl From<ClaudeUsage> for ParsedSessionMessagesItemUsage {
    fn from(u: ClaudeUsage) -> Self {
        Self {
            input_tokens: u.input_tokens.unwrap_or(0) as i64,
            output_tokens: u.output_tokens.unwrap_or(0) as i64,
            cache_read_tokens: u.cache_read_tokens.unwrap_or(0) as i64,
            cache_write_tokens: u.cache_creation_tokens.unwrap_or(0) as i64,
        }
    }
}

impl From<ClaudeUsage> for ParsedSessionTotalUsage {
    fn from(u: ClaudeUsage) -> Self {
        Self {
            input_tokens: u.input_tokens.unwrap_or(0) as i64,
            output_tokens: u.output_tokens.unwrap_or(0) as i64,
            cache_read_tokens: u.cache_read_tokens.unwrap_or(0) as i64,
            cache_write_tokens: u.cache_creation_tokens.unwrap_or(0) as i64,
        }
    }
}

/// Session metadata from Claude Code
#[derive(Debug, Deserialize)]
struct ClaudeMetadata {
    /// Session ID (UUID)
    session_id: String,
    /// Current working directory
    cwd: Option<String>,
    /// Title (may be derived from first prompt)
    title: Option<String>,
    /// Start time
    start_time: Option<String>,
    /// End time
    end_time: Option<String>,
}

/// Raw Claude Code JSONL entry
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClaudeEntry {
    /// A message in the conversation
    Message(ClaudeMessage),
    /// Session metadata
    Metadata(ClaudeMetadata),
    /// Unknown entry type
    #[serde(other)]
    Unknown,
}

/// File metadata for discovery phase
#[derive(Debug, Clone)]
struct DiscoveredFile {
    path: PathBuf,
    mtime: std::time::SystemTime,
    #[allow(dead_code)]
    size: u64,
}

/// Adapter name for session discovery
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdapterName {
    Claude,
    Codex,
    OpenCode,
    Gemini,
    Aider,
}

impl AdapterName {
    /// Get the adapter name as a static string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::OpenCode => "opencode",
            Self::Gemini => "gemini",
            Self::Aider => "aider",
        }
    }

    /// Parse from string
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "claude" => Some(Self::Claude),
            "codex" => Some(Self::Codex),
            "opencode" => Some(Self::OpenCode),
            "gemini" => Some(Self::Gemini),
            "aider" => Some(Self::Aider),
            _ => None,
        }
    }
}

/// Result of parsing a session file
#[derive(Debug)]
#[allow(dead_code)]
struct ParsedSessionFile {
    /// Path to the session file
    path: PathBuf,
    /// Parsed session data (if successful)
    session: Option<ParsedSession>,
    /// Error message (if parsing failed)
    error: Option<String>,
}

/// Trait for adapter-specific session discovery and parsing
trait SessionAdapter: Send + Sync {
    /// Get the adapter name
    fn name(&self) -> AdapterName;

    /// Get the default session directory for this adapter
    fn default_session_dir(&self) -> PathBuf;

    /// Discover session files for this adapter
    fn discover_sessions(&self, project_path: Option<&Path>) -> Vec<DiscoveredFile>;

    /// Parse a single session file
    fn parse_session_file(&self, path: &Path, project_path: Option<&Path>) -> Result<Option<ParsedSession>>;
}

/// Claude Code adapter - parses ~/.claude/projects/**/*.jsonl
struct ClaudeAdapter;

impl SessionAdapter for ClaudeAdapter {
    fn name(&self) -> AdapterName {
        AdapterName::Claude
    }

    fn default_session_dir(&self) -> PathBuf {
        let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.push(".claude");
        home.push("projects");
        home
    }

    fn discover_sessions(&self, _project_path: Option<&Path>) -> Vec<DiscoveredFile> {
        let mut discovered = Vec::new();
        let dir = self.default_session_dir();
        let _ = SessionTailer::scan_directory_recursive(&dir, &mut discovered);
        discovered
    }

    fn parse_session_file(&self, path: &Path, project_path: Option<&Path>) -> Result<Option<ParsedSession>> {
        SessionTailer::parse_claude_session_file(path, project_path)
    }
}

/// Codex adapter - parses OpenAI Codex sessions with token_count events
struct CodexAdapter;

impl SessionAdapter for CodexAdapter {
    fn name(&self) -> AdapterName {
        AdapterName::Codex
    }

    fn default_session_dir(&self) -> PathBuf {
        let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.push(".codex");
        home.push("sessions");
        home
    }

    fn discover_sessions(&self, _project_path: Option<&Path>) -> Vec<DiscoveredFile> {
        let mut discovered = Vec::new();
        let dir = self.default_session_dir();
        let _ = SessionTailer::scan_directory_recursive(&dir, &mut discovered);
        discovered
    }

    fn parse_session_file(&self, path: &Path, project_path: Option<&Path>) -> Result<Option<ParsedSession>> {
        SessionTailer::parse_codex_session_file(path, project_path)
    }
}

/// OpenCode adapter - parses OpenCode sessions with per-message tokens and cost
struct OpenCodeAdapter;

impl SessionAdapter for OpenCodeAdapter {
    fn name(&self) -> AdapterName {
        AdapterName::OpenCode
    }

    fn default_session_dir(&self) -> PathBuf {
        let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.push(".opencode");
        home.push("sessions");
        home
    }

    fn discover_sessions(&self, _project_path: Option<&Path>) -> Vec<DiscoveredFile> {
        let mut discovered = Vec::new();
        let dir = self.default_session_dir();
        let _ = SessionTailer::scan_directory_recursive(&dir, &mut discovered);
        discovered
    }

    fn parse_session_file(&self, path: &Path, project_path: Option<&Path>) -> Result<Option<ParsedSession>> {
        SessionTailer::parse_opencode_session_file(path, project_path)
    }
}

/// Gemini adapter - parses Google Gemini CLI sessions with native usage fields
struct GeminiAdapter;

impl SessionAdapter for GeminiAdapter {
    fn name(&self) -> AdapterName {
        AdapterName::Gemini
    }

    fn default_session_dir(&self) -> PathBuf {
        let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.push(".gemini");
        home.push("sessions");
        home
    }

    fn discover_sessions(&self, _project_path: Option<&Path>) -> Vec<DiscoveredFile> {
        let mut discovered = Vec::new();
        let dir = self.default_session_dir();
        let _ = SessionTailer::scan_directory_recursive(&dir, &mut discovered);
        discovered
    }

    fn parse_session_file(&self, path: &Path, project_path: Option<&Path>) -> Result<Option<ParsedSession>> {
        SessionTailer::parse_gemini_session_file(path, project_path)
    }
}

/// Aider adapter - parses Aider sessions (similar format to Claude)
struct AiderAdapter;

impl SessionAdapter for AiderAdapter {
    fn name(&self) -> AdapterName {
        AdapterName::Aider
    }

    fn default_session_dir(&self) -> PathBuf {
        let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.push(".aider");
        home.push("sessions");
        home
    }

    fn discover_sessions(&self, _project_path: Option<&Path>) -> Vec<DiscoveredFile> {
        let mut discovered = Vec::new();
        let dir = self.default_session_dir();
        let _ = SessionTailer::scan_directory_recursive(&dir, &mut discovered);
        discovered
    }

    fn parse_session_file(&self, path: &Path, project_path: Option<&Path>) -> Result<Option<ParsedSession>> {
        // Aider uses similar format to Claude, can use Claude parser
        SessionTailer::parse_aider_session_file(path, project_path)
    }
}

/// Session tailer configuration
#[derive(Debug, Clone)]
pub struct SessionTailerConfig {
    /// Claude Code sessions directory
    pub claude_projects_dir: PathBuf,
    /// Project path to filter sessions by (cwd must be under this path)
    pub project_path: Option<PathBuf>,
    /// Discovery concurrency limit
    pub discovery_concurrency: usize,
    /// Background poll interval (seconds)
    pub poll_interval_secs: u64,
    /// Enabled adapters (if empty, all adapters are enabled)
    pub enabled_adapters: Vec<AdapterName>,
}

impl Default for SessionTailerConfig {
    fn default() -> Self {
        let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.push(".claude");
        home.push("projects");
        Self {
            claude_projects_dir: home,
            project_path: None,
            discovery_concurrency: 16,
            poll_interval_secs: 5,
            enabled_adapters: vec![
                AdapterName::Claude,
                AdapterName::Codex,
                AdapterName::OpenCode,
                AdapterName::Gemini,
                AdapterName::Aider,
            ],
        }
    }
}

/// Session tailer state
struct SessionTailerState {
    /// Map of session IDs to their file paths
    id_to_path: HashMap<String, PathBuf>,
    /// Map of file paths to session IDs (for bootstrap interceptor)
    path_to_id: HashMap<PathBuf, String>,
    /// Bootstrap interceptor: (first_prompt_hash, cwd) -> session_id
    bootstrap_matches: HashMap<(String, String), String>,
    /// Last discovery timestamp
    last_discovery: Option<DateTime<Utc>>,
    /// Available session adapters
    adapters: Vec<Box<dyn SessionAdapter>>,
    /// (bead_id, provider_session_id) pairs already emitted — enforces at-most-once emission (§B1)
    session_bound_seen: HashSet<(String, String)>,
    /// Forward index: bead_id → provider_session_id
    bead_to_provider_session: HashMap<String, String>,
    /// Reverse index: provider_session_id → bead_id
    provider_session_to_bead: HashMap<String, String>,
}

impl Default for SessionTailerState {
    fn default() -> Self {
        Self {
            id_to_path: HashMap::new(),
            path_to_id: HashMap::new(),
            bootstrap_matches: HashMap::new(),
            last_discovery: None,
            adapters: vec![
                Box::new(ClaudeAdapter),
                Box::new(CodexAdapter),
                Box::new(OpenCodeAdapter),
                Box::new(GeminiAdapter),
                Box::new(AiderAdapter),
            ],
            session_bound_seen: HashSet::new(),
            bead_to_provider_session: HashMap::new(),
            provider_session_to_bead: HashMap::new(),
        }
    }
}

/// Session tailer for CLI sessions
pub struct SessionTailer {
    config: SessionTailerConfig,
    event_tx: broadcast::Sender<SessionEvent>,
    watcher: Option<RecommendedWatcher>,
    _shutdown_tx: mpsc::Sender<()>,
    state: Arc<Mutex<SessionTailerState>>,
}

impl SessionTailer {
    /// Create a new session tailer
    pub fn new(config: SessionTailerConfig) -> Result<Self> {
        let (event_tx, _) = broadcast::channel(256);
        let (shutdown_tx, _) = mpsc::channel(1);

        // Build adapter list based on enabled_adapters config
        let all_adapters: Vec<Box<dyn SessionAdapter>> = vec![
            Box::new(ClaudeAdapter),
            Box::new(CodexAdapter),
            Box::new(OpenCodeAdapter),
            Box::new(GeminiAdapter),
            Box::new(AiderAdapter),
        ];

        let adapters = if config.enabled_adapters.is_empty() {
            all_adapters
        } else {
            let enabled_set: std::collections::HashSet<_> =
                config.enabled_adapters.iter().collect();
            all_adapters
                .into_iter()
                .filter(|a| enabled_set.contains(&a.name()))
                .collect()
        };

        let state = SessionTailerState { adapters, ..Default::default() };

        Ok(Self {
            config,
            event_tx,
            watcher: None,
            _shutdown_tx: shutdown_tx,
            state: Arc::new(Mutex::new(state)),
        })
    }

    /// Subscribe to events from this tailer
    pub fn subscribe(&self) -> broadcast::Receiver<SessionEvent> {
        self.event_tx.subscribe()
    }

    /// Start watching for session changes
    pub fn start(&mut self) -> Result<()> {
        let claude_dir = self.config.claude_projects_dir.clone();
        let claude_dir_for_watch = claude_dir.clone();
        let event_tx = self.event_tx.clone();
        let state = self.state.clone();
        let project_path = self.config.project_path.clone();

        // Create the watcher
        let mut watcher = notify::recommended_watcher(move |res| {
            if let Err(e) = Self::handle_watch_event(
                res,
                &claude_dir_for_watch,
                &event_tx,
                state.clone(),
                project_path.clone(),
            ) {
                warn!("Error handling session watch event: {}", e);
            }
        })
        .context("Failed to create file watcher")?;

        // Watch the Claude projects directory
        let watch_path = if claude_dir.exists() {
            claude_dir.clone()
        } else {
            // Create the directory if it doesn't exist
            fs::create_dir_all(&claude_dir)
                .context("Failed to create Claude projects directory")?;
            claude_dir
        };

        watcher
            .watch(&watch_path, RecursiveMode::Recursive)
            .context("Failed to watch Claude projects directory")?;

        self.watcher = Some(watcher);

        // Initial discovery
        info!("Running initial session discovery...");
        if let Err(e) = self.discover_sessions() {
            warn!("Error during initial session discovery: {}", e);
        }

        // Start background poller
        self.start_background_poller();

        info!(
            "Session tailer watching {}",
            self.config.claude_projects_dir.display()
        );

        Ok(())
    }

    /// Start the background poller
    fn start_background_poller(&self) {
        let interval_secs = self.config.poll_interval_secs;
        let event_tx = self.event_tx.clone();
        let state = self.state.clone();
        let project_path = self.config.project_path.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval_secs));
            loop {
                interval.tick().await;

                // Check for external edits by re-running discovery
                // Bootstrap interceptor will handle aliasing
                if let Err(e) = Self::do_discovery(
                    &state,
                    &event_tx,
                    project_path.as_deref(),
                    false,
                ) {
                    warn!("Error during background discovery: {}", e);
                }
            }
        });
    }

    /// Handle a watch event from notify
    fn handle_watch_event(
        res: Result<notify::Event, notify::Error>,
        claude_dir: &Path,
        event_tx: &broadcast::Sender<SessionEvent>,
        state: Arc<Mutex<SessionTailerState>>,
        project_path: Option<PathBuf>,
    ) -> Result<()> {
        let event = res?;

        // Check if the event is for a JSONL file
        let relevant = event.paths.iter().any(|p| {
            p.extension().map(|e| e == "jsonl").unwrap_or(false)
                && p.starts_with(claude_dir)
        });

        if !relevant {
            return Ok(());
        }

        use notify::EventKind::*;

        match event.kind {
            Create(_) | Modify(_) => {
                // Trigger discovery for the modified file
                if let Err(e) = Self::do_discovery(
                    &state,
                    event_tx,
                    project_path.as_deref(),
                    false,
                ) {
                    warn!("Error handling session file change: {}", e);
                }
            }
            Remove(_) => {
                debug!("Session file removed");
                // Session will be aged out on next discovery
            }
            _ => {}
        }

        Ok(())
    }

    /// Discover sessions using two-phase approach
    fn discover_sessions(&self) -> Result<()> {
        Self::do_discovery(
            &self.state,
            &self.event_tx,
            self.config.project_path.as_deref(),
            true,
        )
    }

    /// Internal discovery implementation
    fn do_discovery(
        state: &Arc<Mutex<SessionTailerState>>,
        event_tx: &broadcast::Sender<SessionEvent>,
        project_path: Option<&Path>,
        is_initial: bool,
    ) -> Result<()> {
        // Phase 1: Discover all session files from all adapters
        let state_guard = state.lock().unwrap();
        let mut discovered = Vec::new();

        for adapter in &state_guard.adapters {
            let mut adapter_files = adapter.discover_sessions(project_path);
            debug!(
                "Discovered {} files from {} adapter",
                adapter_files.len(),
                adapter.name().as_str()
            );
            discovered.append(&mut adapter_files);
        }

        // Sort by mtime (newest first)
        discovered.sort_by_key(|b| std::cmp::Reverse(b.mtime));

        // Phase 2: Parse in parallel with bounded concurrency
        let concurrency = 16;
        let sessions = Self::parse_discovered_files_multi_adapter(
            discovered,
            concurrency,
            project_path,
            &state_guard.adapters,
        )?;

        // Apply bootstrap interceptor (alias new files to existing IDs)
        let sessions = Self::apply_bootstrap_interceptor(
            sessions,
            &mut state.lock().unwrap(),
            event_tx,
        );

        // Send sessions in batches of 100 for progressive streaming
        const BATCH_SIZE: usize = 100;
        for chunk in sessions.chunks(BATCH_SIZE) {
            let _ = event_tx.send(SessionEvent::ConversationsUpdated {
                sessions: chunk.to_vec(),
            });
        }

        // Update last discovery timestamp
        state.lock().unwrap().last_discovery = Some(Utc::now());

        if is_initial {
            info!("Initial session discovery complete");
        }

        Ok(())
    }

    /// Recursively scan directory for JSONL files
    fn scan_directory_recursive(dir: &Path, discovered: &mut Vec<DiscoveredFile>) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory {}", dir.display()))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry.metadata()?;

            if metadata.is_dir() {
                // Recurse into subdirectories
                Self::scan_directory_recursive(&path, discovered)?;
            } else if path.extension().map(|e| e == "jsonl").unwrap_or(false) {
                let mtime = metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                discovered.push(DiscoveredFile {
                    path,
                    mtime,
                    size: metadata.len(),
                });
            }
        }

        Ok(())
    }

    /// Parse discovered files from multiple adapters
    fn parse_discovered_files_multi_adapter(
        files: Vec<DiscoveredFile>,
        _concurrency: usize,
        project_path: Option<&Path>,
        adapters: &[Box<dyn SessionAdapter>],
    ) -> Result<Vec<ParsedSession>> {
        // Use rayon for parallel processing
        let project_path = project_path.map(|p| p.to_path_buf());

        let sessions: Vec<_> = files
            .par_iter()
            .filter_map(|file| {
                // Try each adapter until one succeeds
                for adapter in adapters {
                    match adapter.parse_session_file(&file.path, project_path.as_deref()) {
                        Ok(Some(session)) => return Some(session),
                        Ok(None) => continue, // This adapter doesn't recognize the file
                        Err(e) => {
                            debug!(
                                "Adapter {} failed to parse {}: {}",
                                adapter.name().as_str(),
                                file.path.display(),
                                e
                            );
                            continue;
                        }
                    }
                }
                None
            })
            .collect();

        Ok(sessions)
    }

    /// Parse a Claude Code session file
    fn parse_claude_session_file(
        path: &Path,
        project_path: Option<&Path>,
    ) -> Result<Option<ParsedSession>> {
        let file = File::open(path)
            .with_context(|| format!("Failed to open session file {}", path.display()))?;

        let reader = BufReader::new(file);
        let mut parser = NdjsonParser::new();

        let mut messages = Vec::new();
        let mut session_id = String::new();
        let mut cwd = String::new();
        let mut title = String::new();
        let mut start_time: Option<DateTime<Utc>> = None;
        let mut end_time: Option<DateTime<Utc>> = None;
        let mut total_usage = ParsedSessionTotalUsage {
            input_tokens: 0,
            output_tokens: 0,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
        };
        let mut first_prompt_hash = String::new();
        let mut first_user_content: Option<String> = None;

        let mut line_number: usize = 0;
        for line in reader.lines() {
            let line = line?;
            line_number += 1;
            let source = crate::parse_jsonl_safe::LineSource {
                tag: "sessions/claude",
                file_path: path.to_path_buf(),
                line_number,
            };
            if let Some(entry) = parser.parse_line(&line, &source)? {
                match entry {
                    ClaudeEntry::Message(msg) => {
                        // Track usage
                        if let Some(usage) = &msg.usage {
                            let usage: ParsedSessionMessagesItemUsage = usage.clone().into();
                            total_usage.input_tokens += usage.input_tokens;
                            total_usage.output_tokens += usage.output_tokens;
                            total_usage.cache_read_tokens += usage.cache_read_tokens;
                            total_usage.cache_write_tokens += usage.cache_write_tokens;
                        }

                        // Capture first prompt for bootstrap matching + tag-join
                        if msg.role == "user" && first_prompt_hash.is_empty() {
                            if let Some(content) = &msg.content {
                                first_prompt_hash = Self::hash_content(content);
                                first_user_content = extract_text_content(content);
                            }
                        }

                        let timestamp = msg.timestamp.and_then(|s| s.parse().ok());
                        messages.push(ParsedSessionMessagesItem {
                            role: msg.role,
                            content: msg.content.unwrap_or(serde_json::Value::Null),
                            usage: msg.usage.map(|u| u.into()),
                            timestamp,
                        });
                    }
                    ClaudeEntry::Metadata(meta) => {
                        session_id = meta.session_id;
                        cwd = meta.cwd.unwrap_or_default();
                        title = meta.title.unwrap_or_else(|| {
                            // Derive title from first user message
                            messages
                                .iter()
                                .find(|m| m.role == "user")
                                .and_then(|m| {
                                    m.content.as_str()
                                        .and_then(|s| s.chars().take(50).collect::<String>().into())
                                })
                                .unwrap_or_else(|| String::from("(Untitled)"))
                        });
                        start_time = meta.start_time.and_then(|s| s.parse().ok());
                        end_time = meta.end_time.and_then(|s| s.parse().ok());
                    }
                    ClaudeEntry::Unknown => {}
                }
            }
        }

        // Handle remaining partial line
        if let Some(partial) = parser.finish() {
            warn!("Incomplete final line in session file: {}", partial.trim());
        }

        // Filter by cwd if project_path is specified (canonical comparison)
        if let Some(project_path) = project_path {
            if !cwd_matches_project(&cwd, project_path) {
                return Ok(None); // Filter out sessions not under this project
            }
        }

        // Determine session kind via tag-join resolver
        let tag_result = tag_join::resolve(&title, first_user_content.as_deref());
        let kind = tag_result.kind;

        // Generate stable ID if we don't have one
        let id = if session_id.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            session_id.clone()
        };

        // Derive timestamps
        let created_at = start_time.unwrap_or_else(Utc::now);
        let updated_at = end_time.unwrap_or(created_at);
        let complete = end_time.is_some();

        // Default title if still empty
        let title = if title.is_empty() {
            messages
                .iter()
                .find(|m| m.role == "user")
                .and_then(|m| {
                    m.content.as_str()
                        .and_then(|s| s.chars().take(50).collect::<String>().into())
                })
                .unwrap_or_else(|| String::from("(Untitled)"))
        } else {
            title
        };

        Ok(Some(ParsedSession {
            id,
            session_id,
            provider: "claude".to_string(),
            kind,
            cwd,
            title,
            messages,
            total_usage,
            created_at,
            updated_at,
            complete,
            file_path: path.display().to_string(),
        }))
    }

    /// Hash content for bootstrap matching
    fn hash_content(content: &serde_json::Value) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        content.to_string().hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Apply bootstrap interceptor to prevent duplicates
    fn apply_bootstrap_interceptor(
        sessions: Vec<ParsedSession>,
        state: &mut SessionTailerState,
        event_tx: &broadcast::Sender<SessionEvent>,
    ) -> Vec<ParsedSession> {
        let mut result = Vec::new();

        for session in sessions {
            let file_path = PathBuf::from(&session.file_path);

            // Check if we've seen this file before
            if let Some(_existing_id) = state.path_to_id.get(&file_path) {
                // File already known - skip or update
                continue;
            }

            // Check for bootstrap match by (first_prompt_hash, cwd)
            let first_prompt = session
                .messages
                .iter()
                .find(|m| m.role == "user")
                .map(|m| Self::hash_content(&m.content))
                .unwrap_or_default();

            let key = (first_prompt, session.cwd.clone());

            if let Some(existing_id) = state.bootstrap_matches.get(&key).cloned() {
                // Found a match - alias this file to the existing session ID
                state.path_to_id.insert(file_path.clone(), existing_id.clone());
                state.id_to_path.insert(existing_id.clone(), file_path);
                // Emit session_bound event
                let _ = event_tx.send(SessionEvent::SessionBound {
                    id: existing_id.clone(),
                    file_path: session.file_path.clone(),
                });
                maybe_emit_session_bound(event_tx, &existing_id, &session.session_id, &session.kind, state);
            } else {
                // New session - register it
                state.id_to_path.insert(session.id.clone(), file_path.clone());
                state.path_to_id.insert(file_path, session.id.clone());
                state.bootstrap_matches.insert(key, session.id.clone());
                maybe_emit_session_bound(event_tx, &session.id, &session.session_id, &session.kind, state);
                result.push(session);
            }
        }

        result
    }

    /// Stop the session tailer gracefully
    ///
    /// Flushes any pending discoveries and stops the file watcher.
    /// This should be called during shutdown to ensure clean state.
    pub async fn stop(&mut self) -> Result<()> {
        debug!("Stopping session tailer");

        // Drop the watcher to stop file watching
        drop(self.watcher.take());

        // Give the file watcher a moment to clean up
        tokio::time::sleep(Duration::from_millis(50)).await;

        debug!("Session tailer stopped");
        Ok(())
    }

    /// Parse a Codex session file (OpenAI Codex format)
    fn parse_codex_session_file(
        path: &Path,
        project_path: Option<&Path>,
    ) -> Result<Option<ParsedSession>> {
        let file = File::open(path)
            .with_context(|| format!("Failed to open Codex session file {}", path.display()))?;

        let reader = BufReader::new(file);
        let mut messages = Vec::new();
        let mut session_id = String::new();
        let mut cwd = String::new();
        let mut title = String::new();
        let mut start_time: Option<DateTime<Utc>> = None;
        let mut end_time: Option<DateTime<Utc>> = None;
        let mut total_usage = ParsedSessionTotalUsage {
            input_tokens: 0,
            output_tokens: 0,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
        };
        let mut first_prompt_hash = String::new();
        let mut first_user_content: Option<String> = None;

        let mut line_number: usize = 0;
        for line in reader.lines() {
            let line = line?;
            line_number += 1;
            let source = crate::parse_jsonl_safe::LineSource {
                tag: "sessions/codex",
                file_path: path.to_path_buf(),
                line_number,
            };
            let value = match crate::parse_jsonl_safe::parse_line::<serde_json::Value>(&line, &source) {
                crate::parse_jsonl_safe::ParseResult::Ok(v) => v,
                _ => continue,
            };

            let event_type = value.get("type").and_then(|v| v.as_str());

            match event_type {
                Some("message") | Some("text") => {
                    let role = value.get("role")
                        .and_then(|v| v.as_str())
                        .unwrap_or("user")
                        .to_string();

                    let content = value.get("content").cloned()
                        .unwrap_or(serde_json::Value::Null);

                    let usage = value.get("token_count").and_then(|tc| {
                        tc.as_u64().map(|tokens| ParsedSessionMessagesItemUsage {
                            input_tokens: if role == "user" { tokens as i64 } else { 0 },
                            output_tokens: if role == "assistant" { tokens as i64 } else { 0 },
                            cache_read_tokens: 0,
                            cache_write_tokens: 0,
                        })
                    });

                    if let Some(u) = &usage {
                        total_usage.input_tokens += u.input_tokens;
                        total_usage.output_tokens += u.output_tokens;
                    }

                    if role == "user" && first_prompt_hash.is_empty() {
                        first_prompt_hash = Self::hash_content(&content);
                        first_user_content = extract_text_content(&content);
                    }

                    let timestamp = value.get("timestamp")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse().ok());

                    messages.push(ParsedSessionMessagesItem {
                        role,
                        content,
                        usage,
                        timestamp,
                    });
                }
                Some("session_start") | Some("metadata") => {
                    session_id = value.get("session_id")
                        .and_then(|v| v.as_str())
                        .or_else(|| value.get("id").and_then(|v| v.as_str()))
                        .unwrap_or(&uuid::Uuid::new_v4().to_string())
                        .to_string();

                    cwd = value.get("cwd")
                        .and_then(|v| v.as_str())
                        .or_else(|| value.get("working_directory").and_then(|v| v.as_str()))
                        .unwrap_or(&cwd)
                        .to_string();

                    title = value.get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&title)
                        .to_string();

                    start_time = value.get("start_time")
                        .or_else(|| value.get("created_at"))
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse().ok());
                }
                Some("session_end") | Some("completed") => {
                    end_time = value.get("end_time")
                        .or_else(|| value.get("completed_at"))
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse().ok());
                }
                _ => {
                    debug!("Unknown Codex event type: {:?}", event_type);
                }
            }
        }

        if let Some(project_path) = project_path {
            if !cwd_matches_project(&cwd, project_path) {
                return Ok(None);
            }
        }

        let tag_result = tag_join::resolve(&title, first_user_content.as_deref());
        let kind = tag_result.kind;

        let id = if session_id.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            session_id.clone()
        };

        let created_at = start_time.unwrap_or_else(Utc::now);
        let updated_at = end_time.unwrap_or(created_at);
        let complete = end_time.is_some();

        let title = if title.is_empty() {
            messages
                .iter()
                .find(|m| m.role == "user")
                .and_then(|m| {
                    m.content.as_str()
                        .and_then(|s| s.chars().take(50).collect::<String>().into())
                })
                .unwrap_or_else(|| String::from("(Untitled)"))
        } else {
            title
        };

        Ok(Some(ParsedSession {
            id,
            session_id,
            provider: "codex".to_string(),
            kind,
            cwd,
            title,
            messages,
            total_usage,
            created_at,
            updated_at,
            complete,
            file_path: path.display().to_string(),
        }))
    }

    /// Parse an OpenCode session file (OpenCode format with tokens and cost)
    fn parse_opencode_session_file(
        path: &Path,
        project_path: Option<&Path>,
    ) -> Result<Option<ParsedSession>> {
        let file = File::open(path)
            .with_context(|| format!("Failed to open OpenCode session file {}", path.display()))?;

        let reader = BufReader::new(file);
        let mut messages = Vec::new();
        let mut session_id = String::new();
        let mut cwd = String::new();
        let mut title = String::new();
        let mut start_time: Option<DateTime<Utc>> = None;
        let mut end_time: Option<DateTime<Utc>> = None;
        let mut total_usage = ParsedSessionTotalUsage {
            input_tokens: 0,
            output_tokens: 0,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
        };
        let mut first_prompt_hash = String::new();
        let mut first_user_content: Option<String> = None;

        let mut line_number: usize = 0;
        for line in reader.lines() {
            let line = line?;
            line_number += 1;
            let source = crate::parse_jsonl_safe::LineSource {
                tag: "sessions/opencode",
                file_path: path.to_path_buf(),
                line_number,
            };
            let value = match crate::parse_jsonl_safe::parse_line::<serde_json::Value>(&line, &source) {
                crate::parse_jsonl_safe::ParseResult::Ok(v) => v,
                _ => continue,
            };

            let event_type = value.get("type").and_then(|v| v.as_str());

            match event_type {
                Some("message") => {
                    let role = value.get("role")
                        .and_then(|v| v.as_str())
                        .unwrap_or("user")
                        .to_string();

                    let content = value.get("content").cloned()
                        .unwrap_or(serde_json::Value::Null);

                    let usage = if let Some(tokens_obj) = value.get("tokens") {
                        let input = tokens_obj.get("input")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as i64;
                        let output = tokens_obj.get("output")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as i64;
                        let cache_read = tokens_obj.get("cache_read")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as i64;
                        let cache_write = tokens_obj.get("cache_write")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as i64;

                        Some(ParsedSessionMessagesItemUsage {
                            input_tokens: input,
                            output_tokens: output,
                            cache_read_tokens: cache_read,
                            cache_write_tokens: cache_write,
                        })
                    } else {
                        value.get("token_count").and_then(|v| v.as_u64()).map(|token_count| ParsedSessionMessagesItemUsage {
                            input_tokens: if role == "user" { token_count as i64 } else { 0 },
                            output_tokens: if role == "assistant" { token_count as i64 } else { 0 },
                            cache_read_tokens: 0,
                            cache_write_tokens: 0,
                        })
                    };

                    if let Some(u) = &usage {
                        total_usage.input_tokens += u.input_tokens;
                        total_usage.output_tokens += u.output_tokens;
                        total_usage.cache_read_tokens += u.cache_read_tokens;
                        total_usage.cache_write_tokens += u.cache_write_tokens;
                    }

                    if role == "user" && first_prompt_hash.is_empty() {
                        first_prompt_hash = Self::hash_content(&content);
                        first_user_content = extract_text_content(&content);
                    }

                    let timestamp = value.get("timestamp")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse().ok());

                    messages.push(ParsedSessionMessagesItem {
                        role,
                        content,
                        usage,
                        timestamp,
                    });
                }
                Some("metadata") | Some("session") => {
                    session_id = value.get("session_id")
                        .or_else(|| value.get("id"))
                        .and_then(|v| v.as_str())
                        .unwrap_or(&uuid::Uuid::new_v4().to_string())
                        .to_string();

                    cwd = value.get("cwd")
                        .or_else(|| value.get("working_directory"))
                        .and_then(|v| v.as_str())
                        .unwrap_or(&cwd)
                        .to_string();

                    title = value.get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&title)
                        .to_string();

                    start_time = value.get("start_time")
                        .or_else(|| value.get("created_at"))
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse().ok());
                }
                Some("end") | Some("complete") => {
                    end_time = value.get("end_time")
                        .or_else(|| value.get("completed_at"))
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse().ok());
                }
                _ => {}
            }
        }

        if let Some(project_path) = project_path {
            if !cwd_matches_project(&cwd, project_path) {
                return Ok(None);
            }
        }

        let tag_result = tag_join::resolve(&title, first_user_content.as_deref());
        let kind = tag_result.kind;

        let id = if session_id.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            session_id.clone()
        };

        let created_at = start_time.unwrap_or_else(Utc::now);
        let updated_at = end_time.unwrap_or(created_at);
        let complete = end_time.is_some();

        let title = if title.is_empty() {
            messages
                .iter()
                .find(|m| m.role == "user")
                .and_then(|m| {
                    m.content.as_str()
                        .and_then(|s| s.chars().take(50).collect::<String>().into())
                })
                .unwrap_or_else(|| String::from("(Untitled)"))
        } else {
            title
        };

        Ok(Some(ParsedSession {
            id,
            session_id,
            provider: "opencode".to_string(),
            kind,
            cwd,
            title,
            messages,
            total_usage,
            created_at,
            updated_at,
            complete,
            file_path: path.display().to_string(),
        }))
    }

    /// Parse a Gemini session file (Google Gemini CLI format)
    fn parse_gemini_session_file(
        path: &Path,
        project_path: Option<&Path>,
    ) -> Result<Option<ParsedSession>> {
        let file = File::open(path)
            .with_context(|| format!("Failed to open Gemini session file {}", path.display()))?;

        let reader = BufReader::new(file);
        let mut messages = Vec::new();
        let mut session_id = String::new();
        let mut cwd = String::new();
        let mut title = String::new();
        let mut start_time: Option<DateTime<Utc>> = None;
        let mut end_time: Option<DateTime<Utc>> = None;
        let mut total_usage = ParsedSessionTotalUsage {
            input_tokens: 0,
            output_tokens: 0,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
        };
        let mut first_prompt_hash = String::new();
        let mut first_user_content: Option<String> = None;

        let mut line_number: usize = 0;
        for line in reader.lines() {
            let line = line?;
            line_number += 1;
            let source = crate::parse_jsonl_safe::LineSource {
                tag: "sessions/gemini",
                file_path: path.to_path_buf(),
                line_number,
            };
            let value = match crate::parse_jsonl_safe::parse_line::<serde_json::Value>(&line, &source) {
                crate::parse_jsonl_safe::ParseResult::Ok(v) => v,
                _ => continue,
            };

            let event_type = value.get("type").and_then(|v| v.as_str());

            match event_type {
                Some("message") | Some("turn") => {
                    let role = value.get("role")
                        .and_then(|v| v.as_str())
                        .unwrap_or("user")
                        .to_string();

                    let content = value.get("content").cloned()
                        .unwrap_or(serde_json::Value::Null);

                    let usage = if let Some(usage_obj) = value.get("usage") {
                        let input = usage_obj.get("promptTokenCount")
                            .or_else(|| usage_obj.get("input_tokens"))
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as i64;
                        let output = usage_obj.get("candidatesTokenCount")
                            .or_else(|| usage_obj.get("output_tokens"))
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as i64;
                        let cache_read = usage_obj.get("cachedContentTokenCount")
                            .or_else(|| usage_obj.get("cache_read_tokens"))
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as i64;
                        let cache_write = usage_obj.get("cache_write_tokens")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as i64;

                        Some(ParsedSessionMessagesItemUsage {
                            input_tokens: input,
                            output_tokens: output,
                            cache_read_tokens: cache_read,
                            cache_write_tokens: cache_write,
                        })
                    } else {
                        None
                    };

                    if let Some(u) = &usage {
                        total_usage.input_tokens += u.input_tokens;
                        total_usage.output_tokens += u.output_tokens;
                        total_usage.cache_read_tokens += u.cache_read_tokens;
                        total_usage.cache_write_tokens += u.cache_write_tokens;
                    }

                    if role == "user" && first_prompt_hash.is_empty() {
                        first_prompt_hash = Self::hash_content(&content);
                        first_user_content = extract_text_content(&content);
                    }

                    let timestamp = value.get("timestamp")
                        .or_else(|| value.get("time"))
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse().ok());

                    messages.push(ParsedSessionMessagesItem {
                        role,
                        content,
                        usage,
                        timestamp,
                    });
                }
                Some("metadata") | Some("session_info") => {
                    session_id = value.get("session_id")
                        .or_else(|| value.get("id"))
                        .and_then(|v| v.as_str())
                        .unwrap_or(&uuid::Uuid::new_v4().to_string())
                        .to_string();

                    cwd = value.get("cwd")
                        .or_else(|| value.get("working_directory"))
                        .and_then(|v| v.as_str())
                        .unwrap_or(&cwd)
                        .to_string();

                    title = value.get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&title)
                        .to_string();

                    start_time = value.get("start_time")
                        .or_else(|| value.get("created_at"))
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse().ok());
                }
                Some("end") => {
                    end_time = value.get("end_time")
                        .or_else(|| value.get("completed_at"))
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse().ok());
                }
                _ => {}
            }
        }

        if let Some(project_path) = project_path {
            if !cwd_matches_project(&cwd, project_path) {
                return Ok(None);
            }
        }

        let tag_result = tag_join::resolve(&title, first_user_content.as_deref());
        let kind = tag_result.kind;

        let id = if session_id.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            session_id.clone()
        };

        let created_at = start_time.unwrap_or_else(Utc::now);
        let updated_at = end_time.unwrap_or(created_at);
        let complete = end_time.is_some();

        let title = if title.is_empty() {
            messages
                .iter()
                .find(|m| m.role == "user")
                .and_then(|m| {
                    m.content.as_str()
                        .and_then(|s| s.chars().take(50).collect::<String>().into())
                })
                .unwrap_or_else(|| String::from("(Untitled)"))
        } else {
            title
        };

        Ok(Some(ParsedSession {
            id,
            session_id,
            provider: "gemini".to_string(),
            kind,
            cwd,
            title,
            messages,
            total_usage,
            created_at,
            updated_at,
            complete,
            file_path: path.display().to_string(),
        }))
    }

    /// Parse an Aider session file (similar to Claude format)
    fn parse_aider_session_file(
        path: &Path,
        project_path: Option<&Path>,
    ) -> Result<Option<ParsedSession>> {
        let file = File::open(path)
            .with_context(|| format!("Failed to open Aider session file {}", path.display()))?;

        let reader = BufReader::new(file);
        let mut messages = Vec::new();
        let mut session_id = String::new();
        let mut cwd = String::new();
        let mut title = String::new();
        let mut start_time: Option<DateTime<Utc>> = None;
        let mut end_time: Option<DateTime<Utc>> = None;
        let mut total_usage = ParsedSessionTotalUsage {
            input_tokens: 0,
            output_tokens: 0,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
        };
        let mut first_prompt_hash = String::new();
        let mut first_user_content: Option<String> = None;

        let mut line_number: usize = 0;
        for line in reader.lines() {
            let line = line?;
            line_number += 1;
            let source = crate::parse_jsonl_safe::LineSource {
                tag: "sessions/aider",
                file_path: path.to_path_buf(),
                line_number,
            };
            let entry = match crate::parse_jsonl_safe::parse_line::<ClaudeEntry>(&line, &source) {
                crate::parse_jsonl_safe::ParseResult::Ok(v) => v,
                _ => continue,
            };
            match entry {
                ClaudeEntry::Message(msg) => {
                    if let Some(usage) = &msg.usage {
                        let usage: ParsedSessionMessagesItemUsage = usage.clone().into();
                        total_usage.input_tokens += usage.input_tokens;
                        total_usage.output_tokens += usage.output_tokens;
                        total_usage.cache_read_tokens += usage.cache_read_tokens;
                        total_usage.cache_write_tokens += usage.cache_write_tokens;
                    }

                    if msg.role == "user" && first_prompt_hash.is_empty() {
                        if let Some(content) = &msg.content {
                            first_prompt_hash = Self::hash_content(content);
                            first_user_content = extract_text_content(content);
                        }
                    }

                    let timestamp = msg.timestamp.and_then(|s| s.parse().ok());
                    messages.push(ParsedSessionMessagesItem {
                        role: msg.role,
                        content: msg.content.unwrap_or(serde_json::Value::Null),
                        usage: msg.usage.map(|u| u.into()),
                        timestamp,
                    });
                }
                ClaudeEntry::Metadata(meta) => {
                    session_id = meta.session_id;
                    cwd = meta.cwd.unwrap_or_default();
                    title = meta.title.unwrap_or_default();
                    start_time = meta.start_time.and_then(|s| s.parse().ok());
                    end_time = meta.end_time.and_then(|s| s.parse().ok());
                }
                ClaudeEntry::Unknown => {}
            }
        }

        if let Some(project_path) = project_path {
            if !cwd_matches_project(&cwd, project_path) {
                return Ok(None);
            }
        }

        let tag_result = tag_join::resolve(&title, first_user_content.as_deref());
        let kind = tag_result.kind;

        let id = if session_id.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            session_id.clone()
        };

        let created_at = start_time.unwrap_or_else(Utc::now);
        let updated_at = end_time.unwrap_or(created_at);
        let complete = end_time.is_some();

        let title = if title.is_empty() {
            messages
                .iter()
                .find(|m| m.role == "user")
                .and_then(|m| {
                    m.content.as_str()
                        .and_then(|s| s.chars().take(50).collect::<String>().into())
                })
                .unwrap_or_else(|| String::from("(Untitled)"))
        } else {
            title
        };

        Ok(Some(ParsedSession {
            id,
            session_id,
            provider: "aider".to_string(),
            kind,
            cwd,
            title,
            messages,
            total_usage,
            created_at,
            updated_at,
            complete,
            file_path: path.display().to_string(),
        }))
    }
}

/// Extract text content from a message content field.
///
/// CLI adapters store content in different shapes:
/// - Plain string: `"text here"`
/// - Array of blocks: `[{"type": "text", "text": "..."}]`
fn extract_text_content(content: &serde_json::Value) -> Option<String> {
    match content {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Array(blocks) => {
            for block in blocks {
                if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                    if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                        return Some(text.to_string());
                    }
                }
            }
            None
        }
        _ => None,
    }
}

/// Emit a TagJoinBound event for worker sessions (dual-identity invariant §B1).
///
/// Idempotent: emits exactly once per (bead_id, provider_session_id) pair within this
/// state lifetime.  Subsequent calls with the same pair are silent no-ops.
fn maybe_emit_session_bound(
    event_tx: &broadcast::Sender<SessionEvent>,
    session_id: &str,
    provider_session_id: &str,
    kind: &ParsedSessionKind,
    state: &mut SessionTailerState,
) {
    if let ParsedSessionKind::Variant0 { worker, bead, strand } = kind {
        let key = (bead.clone(), provider_session_id.to_string());
        if state.session_bound_seen.insert(key) {
            // First meeting of this (bead_id, provider_session_id) pair — record and emit.
            state.bead_to_provider_session.insert(bead.clone(), provider_session_id.to_string());
            state.provider_session_to_bead.insert(provider_session_id.to_string(), bead.clone());
            let _ = event_tx.send(SessionEvent::TagJoinBound {
                session_id: session_id.to_string(),
                provider_session_id: provider_session_id.to_string(),
                bead_id: bead.clone(),
                worker: worker.clone(),
                strand: strand.clone(),
                ts: Utc::now(),
            });
        }
    }
}

/// Line-buffered NDJSON parser for Claude Code sessions
struct NdjsonParser {
    partial: String,
}

impl NdjsonParser {
    fn new() -> Self {
        Self {
            partial: String::new(),
        }
    }

    fn parse_line(&mut self, line: &str, source: &crate::parse_jsonl_safe::LineSource) -> Result<Option<ClaudeEntry>> {
        let mut input = line;

        if !self.partial.is_empty() {
            self.partial.push_str(line);
            input = self.partial.as_str();
        }

        match serde_json::from_str::<ClaudeEntry>(input) {
            Ok(entry) => {
                self.partial.clear();
                Ok(Some(entry))
            }
            Err(e) => {
                // Treat as partial if short AND doesn't end with closing bracket
                if input.len() < 256 && !input.ends_with('}') && !input.ends_with(']') {
                    if self.partial.is_empty() {
                        self.partial = input.to_string();
                    }
                    Ok(None)
                } else {
                    crate::parse_jsonl_safe::quarantine_raw(input, &e.to_string(), source);
                    self.partial.clear();
                    Ok(Some(ClaudeEntry::Unknown))
                }
            }
        }
    }

    fn finish(&mut self) -> Option<String> {
        if self.partial.is_empty() {
            None
        } else {
            let partial = self.partial.clone();
            self.partial.clear();
            Some(partial)
        }
    }
}

impl Default for NdjsonParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hoop_schema::{ParsedSessionKindVariant1, ParsedSessionKindVariant2};

    #[test]
    fn test_tag_join_worker_via_resolve() {
        let result = tag_join::resolve("[needle:alpha:bd-abc123:pluck] Implement feature X", None);
        match result.kind {
            ParsedSessionKind::Variant0 { worker, bead, strand } => {
                assert_eq!(worker, "alpha");
                assert_eq!(bead, "bd-abc123");
                assert_eq!(strand.as_deref(), Some("pluck"));
            }
            _ => panic!("Expected Worker kind"),
        }
        assert!(result.binding.is_some());
    }

    #[test]
    fn test_tag_join_worker_no_strand_via_resolve() {
        let result = tag_join::resolve("[needle:bravo:bd-def456:] Some task", None);
        match result.kind {
            ParsedSessionKind::Variant0 { worker, bead, strand } => {
                assert_eq!(worker, "bravo");
                assert_eq!(bead, "bd-def456");
                assert!(strand.is_none());
            }
            _ => panic!("Expected Worker kind"),
        }
    }

    #[test]
    fn test_tag_join_ad_hoc_via_resolve() {
        let result = tag_join::resolve("Fix the login bug", None);
        assert_eq!(result.kind, ParsedSessionKind::Variant2(ParsedSessionKindVariant2::AdHoc));
        assert!(result.binding.is_none());
    }

    #[test]
    fn test_tag_join_dictated_via_resolve() {
        let result = tag_join::resolve("[dictated] Voice note transcript", None);
        assert_eq!(result.kind, ParsedSessionKind::Variant1(ParsedSessionKindVariant1::Dictated));
        assert!(result.binding.is_none());
    }

    #[test]
    fn test_usage_from_claude() {
        let claude = ClaudeUsage {
            input_tokens: Some(100),
            output_tokens: Some(50),
            cache_read_tokens: Some(10),
            cache_creation_tokens: Some(5),
        };
        let usage: ParsedSessionMessagesItemUsage = claude.into();
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);
        assert_eq!(usage.cache_read_tokens, 10);
        assert_eq!(usage.cache_write_tokens, 5);
    }

    // --- session_bound idempotency tests (§B1, §3.4) ---

    fn make_worker_kind(worker: &str, bead: &str, strand: Option<&str>) -> ParsedSessionKind {
        ParsedSessionKind::Variant0 {
            worker: worker.to_string(),
            bead: bead.to_string(),
            strand: strand.map(|s| s.to_string()),
        }
    }

    #[test]
    fn session_bound_emitted_on_first_join() {
        let (event_tx, mut event_rx) = broadcast::channel(16);
        let mut state = SessionTailerState {
            adapters: vec![],
            ..SessionTailerState::default()
        };

        let kind = make_worker_kind("alpha", "bd-abc123", Some("pluck"));
        maybe_emit_session_bound(&event_tx, "hoop-session-1", "provider-sess-xyz", &kind, &mut state);

        match event_rx.try_recv().expect("expected exactly one TagJoinBound event") {
            SessionEvent::TagJoinBound { bead_id, provider_session_id, worker, .. } => {
                assert_eq!(bead_id, "bd-abc123");
                assert_eq!(provider_session_id, "provider-sess-xyz");
                assert_eq!(worker, "alpha");
            }
            other => panic!("unexpected event: {:?}", other),
        }
        assert!(event_rx.try_recv().is_err(), "no second event expected");
    }

    #[test]
    fn session_bound_replay_fixture_twice_emits_exactly_once() {
        let (event_tx, mut event_rx) = broadcast::channel(16);
        let mut state = SessionTailerState {
            adapters: vec![],
            ..SessionTailerState::default()
        };

        let kind = make_worker_kind("bravo", "bd-def456", None);

        // First pass — should emit
        maybe_emit_session_bound(&event_tx, "hoop-s1", "prov-id-001", &kind, &mut state);
        // Second pass with identical (bead_id, provider_session_id) — must be a no-op
        maybe_emit_session_bound(&event_tx, "hoop-s1", "prov-id-001", &kind, &mut state);

        // Exactly one event in the channel
        event_rx.try_recv().expect("expected one event");
        assert!(event_rx.try_recv().is_err(), "second identical pair must not emit");
    }

    #[test]
    fn session_bound_different_pairs_each_emit_once() {
        let (event_tx, mut event_rx) = broadcast::channel(16);
        let mut state = SessionTailerState {
            adapters: vec![],
            ..SessionTailerState::default()
        };

        let kind_a = make_worker_kind("alpha", "bd-aaa", Some("pluck"));
        let kind_b = make_worker_kind("bravo", "bd-bbb", Some("mend"));

        maybe_emit_session_bound(&event_tx, "s1", "prov-a", &kind_a, &mut state);
        maybe_emit_session_bound(&event_tx, "s2", "prov-b", &kind_b, &mut state);

        // Both pairs should emit independently
        event_rx.try_recv().expect("first pair should emit");
        event_rx.try_recv().expect("second pair should emit");
        assert!(event_rx.try_recv().is_err());
    }

    #[test]
    fn session_bound_indexes_both_ids() {
        let (event_tx, _) = broadcast::channel(16);
        let mut state = SessionTailerState {
            adapters: vec![],
            ..SessionTailerState::default()
        };

        let kind = make_worker_kind("gamma", "bd-ccc999", Some("weave"));
        maybe_emit_session_bound(&event_tx, "s1", "prov-session-gamma", &kind, &mut state);

        // Forward index: bead → provider session
        assert_eq!(
            state.bead_to_provider_session.get("bd-ccc999").map(|s| s.as_str()),
            Some("prov-session-gamma"),
        );
        // Reverse index: provider session → bead
        assert_eq!(
            state.provider_session_to_bead.get("prov-session-gamma").map(|s| s.as_str()),
            Some("bd-ccc999"),
        );
        // Dedup set contains the pair
        assert!(state.session_bound_seen.contains(&(
            "bd-ccc999".to_string(),
            "prov-session-gamma".to_string(),
        )));
    }

    #[test]
    fn session_bound_ad_hoc_kind_emits_nothing() {
        use hoop_schema::ParsedSessionKindVariant2;
        let (event_tx, mut event_rx) = broadcast::channel(16);
        let mut state = SessionTailerState {
            adapters: vec![],
            ..SessionTailerState::default()
        };

        let kind = ParsedSessionKind::Variant2(ParsedSessionKindVariant2::AdHoc);
        maybe_emit_session_bound(&event_tx, "s1", "prov-id", &kind, &mut state);

        assert!(event_rx.try_recv().is_err(), "ad-hoc sessions must not emit session_bound");
        assert!(state.session_bound_seen.is_empty());
    }

    // ── Canonicalization symlink fixtures ──────────────────────────────────

    #[test]
    fn cwd_matches_project_via_symlink() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let real = tmp.path().join("real-project");
        fs::create_dir_all(&real).expect("mkdir");

        let link = tmp.path().join("link-project");
        std::os::unix::fs::symlink(&real, &link).expect("symlink");

        let canonical = fs::canonicalize(&real).expect("canonicalize");

        // Session CWD is under the symlink, project path is canonical
        assert!(
            cwd_matches_project(&link.to_string_lossy(), &canonical),
            "symlink CWD must match canonical project path"
        );

        // Session CWD is canonical, project path is the symlink
        assert!(
            cwd_matches_project(&canonical.to_string_lossy(), &link.as_path()),
            "canonical CWD must match symlink project path"
        );

        // Both are the same (no symlink)
        assert!(
            cwd_matches_project(&canonical.to_string_lossy(), &canonical.as_path()),
            "identical canonical paths must match"
        );
    }

    #[test]
    fn cwd_matches_project_subdir_via_symlink() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let real = tmp.path().join("real-project");
        let subdir = real.join("src");
        fs::create_dir_all(&subdir).expect("mkdir");

        let link = tmp.path().join("link-project");
        std::os::unix::fs::symlink(&real, &link).expect("symlink");

        let canonical = fs::canonicalize(&real).expect("canonicalize");
        let cwd = link.join("src").to_string_lossy().to_string();

        assert!(
            cwd_matches_project(&cwd, &canonical),
            "subdirectory under symlink CWD must match canonical project path"
        );
    }

    #[test]
    fn cwd_does_not_match_unrelated_path() {
        assert!(
            !cwd_matches_project("/home/coding/project-a", Path::new("/home/coding/project-b")),
            "unrelated paths must not match"
        );
    }

    #[test]
    fn cwd_does_not_match_prefix_path_without_separator() {
        // /home/coding/FOOBAR must NOT match project /home/coding/FOO
        assert!(
            !cwd_matches_project("/home/coding/FOOBAR", Path::new("/home/coding/FOO")),
            "path that extends the project name without / separator must not match"
        );
        // But /home/coding/FOO/src should match
        assert!(
            cwd_matches_project("/home/coding/FOO/src", Path::new("/home/coding/FOO")),
            "subdirectory under project must match"
        );
        // And /home/coding/FOO itself should match
        assert!(
            cwd_matches_project("/home/coding/FOO", Path::new("/home/coding/FOO")),
            "exact match must work"
        );
    }

    #[test]
    fn cwd_matches_symlink_with_subdir() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let real = tmp.path().join("real");
        let subdir = real.join("subdir");
        fs::create_dir_all(&subdir).expect("mkdir");

        let link = tmp.path().join("link");
        std::os::unix::fs::symlink(&real, &link).expect("symlink");

        let canonical = fs::canonicalize(&real).expect("canonicalize");
        // CWD via symlink subdir must match canonical project
        let cwd_via_link = link.join("subdir").to_string_lossy().to_string();
        assert!(
            cwd_matches_project(&cwd_via_link, &canonical),
            "CWD via symlink subdir must match canonical project"
        );
    }
}
