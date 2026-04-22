//! Session tailer for CLI sessions
//!
//! Discovers and parses `.jsonl` session files from CLI providers (Claude Code, Codex, etc.).
//! Two-phase discovery: stat everything + sort by mtime, then parse in parallel.
//! 5-second background poll detects external edits.
//! Bootstrap interceptor aliases newly-found files back to existing session IDs.
//! Filter-by-cwd to scope sessions to the registered project.
//!
//! Per-project runtime (plan §4.3): each project gets its own session tailer
//! scoped to sessions whose cwd is under the project path.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use hoop_schema::{MessageUsage, ParsedSession, SessionKind, SessionMessage};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Events emitted by the session tailer
#[derive(Debug, Clone)]
pub enum SessionEvent {
    /// One or more sessions were discovered/updated
    ConversationsUpdated { sessions: Vec<ParsedSession> },
    /// A new session file was bound to an existing session ID
    SessionBound { id: String, file_path: String },
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

impl From<ClaudeUsage> for MessageUsage {
    fn from(u: ClaudeUsage) -> Self {
        Self {
            input_tokens: u.input_tokens.unwrap_or(0),
            output_tokens: u.output_tokens.unwrap_or(0),
            cache_read_tokens: u.cache_read_tokens.unwrap_or(0),
            cache_write_tokens: u.cache_creation_tokens.unwrap_or(0),
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
    size: u64,
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
        }
    }
}

/// Session tailer state
#[derive(Debug)]
struct SessionTailerState {
    /// Map of session IDs to their file paths
    id_to_path: HashMap<String, PathBuf>,
    /// Map of file paths to session IDs (for bootstrap interceptor)
    path_to_id: HashMap<PathBuf, String>,
    /// Bootstrap interceptor: (first_prompt_hash, cwd) -> session_id
    bootstrap_matches: HashMap<(String, String), String>,
    /// Last discovery timestamp
    last_discovery: Option<DateTime<Utc>>,
}

impl Default for SessionTailerState {
    fn default() -> Self {
        Self {
            id_to_path: HashMap::new(),
            path_to_id: HashMap::new(),
            bootstrap_matches: HashMap::new(),
            last_discovery: None,
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

        Ok(Self {
            config,
            event_tx,
            watcher: None,
            _shutdown_tx: shutdown_tx,
            state: Arc::new(Mutex::new(SessionTailerState::default())),
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
                    &event_tx,
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
        // Phase 1: Discover all JSONL files
        let claude_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".claude")
            .join("projects");

        let mut discovered = Vec::new();
        Self::scan_directory_recursive(&claude_dir, &mut discovered)?;

        // Sort by mtime (newest first)
        discovered.sort_by(|a, b| b.mtime.cmp(&a.mtime));

        // Phase 2: Parse in parallel with bounded concurrency
        let concurrency = 16;
        let sessions = Self::parse_discovered_files(discovered, concurrency, project_path)?;

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

    /// Parse discovered files in parallel with bounded concurrency
    fn parse_discovered_files(
        files: Vec<DiscoveredFile>,
        _concurrency: usize,
        project_path: Option<&Path>,
    ) -> Result<Vec<ParsedSession>> {
        // Use rayon for parallel processing
        let project_path = project_path.map(|p| p.to_path_buf());

        let sessions: Vec<_> = files
            .par_iter()
            .filter_map(|file| {
                match Self::parse_session_file(&file.path, project_path.as_deref()) {
                    Ok(Some(session)) => Some(session),
                    Ok(None) => None, // Filtered out
                    Err(e) => {
                        warn!("Error parsing session file {}: {}", file.path.display(), e);
                        None
                    }
                }
            })
            .collect();

        Ok(sessions)
    }

    /// Parse a single session file
    fn parse_session_file(
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
        let mut total_usage = MessageUsage {
            input_tokens: 0,
            output_tokens: 0,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
        };
        let mut first_prompt_hash = String::new();

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = parser.parse_line(&line)? {
                match entry {
                    ClaudeEntry::Message(msg) => {
                        // Track usage
                        if let Some(usage) = &msg.usage {
                            let usage: MessageUsage = usage.clone().into();
                            total_usage.input_tokens += usage.input_tokens;
                            total_usage.output_tokens += usage.output_tokens;
                            total_usage.cache_read_tokens += usage.cache_read_tokens;
                            total_usage.cache_write_tokens += usage.cache_write_tokens;
                        }

                        // Capture first prompt for bootstrap matching
                        if msg.role == "user" && first_prompt_hash.is_empty() {
                            if let Some(content) = &msg.content {
                                first_prompt_hash = Self::hash_content(content);
                            }
                        }

                        messages.push(SessionMessage {
                            role: msg.role,
                            content: msg.content.unwrap_or(serde_json::Value::Null),
                            usage: msg.usage.map(|u| u.into()),
                            timestamp: msg.timestamp,
                        });
                    }
                    ClaudeEntry::Metadata(meta) => {
                        session_id = meta.session_id;
                        cwd = meta.cwd.unwrap_or_else(|| String::new());
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

        // Filter by cwd if project_path is specified
        if let Some(project_path) = project_path {
            let project_str = project_path.to_string_lossy();
            if !cwd.starts_with(&*project_str) {
                return Ok(None); // Filter out sessions not under this project
            }
        }

        // Determine session kind from prefix tag
        let kind = Self::classify_session(&title, &cwd);

        // Generate stable ID if we don't have one
        let id = if session_id.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            session_id.clone()
        };

        // Derive timestamps
        let created_at = start_time.unwrap_or_else(|| Utc::now());
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

    /// Classify a session based on title prefix tag
    fn classify_session(title: &str, _cwd: &str) -> SessionKind {
        // Check for NEEDLE worker tag: [needle:<worker>:<bead>:<strand>]
        if let Some(captures) = regex::Regex::new(r"^\[needle:([^:]+):([^:]+):([^:]*)\]")
            .ok()
            .and_then(|re| re.captures(title))
        {
            return SessionKind::Worker {
                worker: captures.get(1).map(|m| m.as_str().to_string()).unwrap_or_default(),
                bead: captures.get(2).map(|m| m.as_str().to_string()).unwrap_or_default(),
                strand: captures.get(3).map(|m| m.as_str().to_string()).filter(|s| !s.is_empty()),
            };
        }

        // Check for dictated prefix
        if title.starts_with("[dictated]") {
            return SessionKind::Dictated;
        }

        // Default to operator (normal conversation)
        SessionKind::Operator
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

            if let Some(existing_id) = state.bootstrap_matches.get(&key) {
                // Found a match - alias this file to the existing session ID
                state.path_to_id.insert(file_path.clone(), existing_id.clone());
                state.id_to_path.insert(existing_id.clone(), file_path);
                // Emit session_bound event
                let _ = event_tx.send(SessionEvent::SessionBound {
                    id: existing_id.clone(),
                    file_path: session.file_path.clone(),
                });
            } else {
                // New session - register it
                state.id_to_path.insert(session.id.clone(), file_path.clone());
                state.path_to_id.insert(file_path, session.id.clone());
                state.bootstrap_matches.insert(key, session.id.clone());
                result.push(session);
            }
        }

        result
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

    fn parse_line(&mut self, line: &str) -> Result<Option<ClaudeEntry>> {
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
            Err(_) => {
                // Treat as partial if short AND doesn't end with closing bracket
                if input.len() < 256 && !input.ends_with('}') && !input.ends_with(']') {
                    if self.partial.is_empty() {
                        self.partial = input.to_string();
                    }
                    Ok(None)
                } else {
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_classify_session_worker() {
        let title = "[needle:alpha:bd-abc123:main] Implement feature X";
        let kind = SessionTailer::classify_session(title, "/home/coding/project");
        match kind {
            SessionKind::Worker { worker, bead, strand } => {
                assert_eq!(worker, "alpha");
                assert_eq!(bead, "bd-abc123");
                assert_eq!(strand.as_deref(), Some("main"));
            }
            _ => panic!("Expected Worker kind"),
        }
    }

    #[test]
    fn test_classify_session_operator() {
        let title = "Fix the login bug";
        let kind = SessionTailer::classify_session(title, "/home/coding/project");
        assert_eq!(kind, SessionKind::Operator);
    }

    #[test]
    fn test_classify_session_dictated() {
        let title = "[dictated] Voice note transcript";
        let kind = SessionTailer::classify_session(title, "/home/coding/project");
        assert_eq!(kind, SessionKind::Dictated);
    }

    #[test]
    fn test_usage_from_claude() {
        let claude = ClaudeUsage {
            input_tokens: Some(100),
            output_tokens: Some(50),
            cache_read_tokens: Some(10),
            cache_creation_tokens: Some(5),
        };
        let usage: MessageUsage = claude.into();
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);
        assert_eq!(usage.cache_read_tokens, 10);
        assert_eq!(usage.cache_write_tokens, 5);
    }
}
