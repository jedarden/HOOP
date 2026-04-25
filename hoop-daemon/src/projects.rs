//! Projects configuration hot-reload
//!
//! Watches ~/.hoop/projects.yaml for changes and emits events when the
//! configuration is updated. Handles validation and error reporting.

use anyhow::{Context, Result};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use serde_yaml::Error as YamlError;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// Projects configuration loaded from projects.yaml
#[derive(Debug, Clone)]
pub struct ProjectsConfig {
    /// The registry loaded from disk
    pub registry: hoop_schema::ProjectsRegistry,
    /// Pre-resolved canonical paths keyed by (project_name, raw_path).
    /// Populated on load so consumers don't need to re-canonicalize.
    pub canonical_cache: std::collections::HashMap<(String, PathBuf), PathBuf>,
    /// Path to the projects.yaml file
    pub path: PathBuf,
    /// SHA-256 hex digest of the raw file contents at load time
    pub content_hash: String,
}

impl ProjectsConfig {
    /// Load the projects registry from the default path
    pub fn load() -> Result<Self> {
        let path = registry_path()?;
        Self::load_from(&path)
    }

    /// Load from a specific path
    pub fn load_from(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self {
                registry: hoop_schema::ProjectsRegistry::default(),
                canonical_cache: std::collections::HashMap::new(),
                path: path.to_path_buf(),
                content_hash: String::new(),
            });
        }

        let contents = fs::read_to_string(path)
            .context("Failed to read projects.yaml")?;

        let content_hash = hex::encode(Sha256::digest(contents.as_bytes()));

        let mut registry: hoop_schema::ProjectsRegistry = serde_yaml::from_str(&contents)
            .context("Failed to parse projects.yaml")?;

        // Auto-populate canonical_path on any entry missing it, and persist.
        let backfilled = Self::backfill_canonical_paths(&mut registry);
        if backfilled {
            Self::write_back(path, &registry)?;
        }

        let canonical_cache = Self::build_canonical_cache(&registry);

        Ok(Self {
            registry,
            canonical_cache,
            path: path.to_path_buf(),
            content_hash,
        })
    }

    /// Auto-populate `canonical_path` on every workspace entry that lacks it.
    ///
    /// Returns `true` if any entry was updated (caller should persist).
    /// Failures to resolve are silently skipped — remote-host paths may not exist locally.
    fn backfill_canonical_paths(registry: &mut hoop_schema::ProjectsRegistry) -> bool {
        let mut changed = false;
        for project in &mut registry.projects {
            match project {
                hoop_schema::ProjectsRegistryProjectsItem::Variant0 {
                    path, canonical_path, ..
                } => {
                    if canonical_path.is_none() {
                        if let Ok(resolved) = fs::canonicalize(path) {
                            *canonical_path = Some(resolved.to_string_lossy().to_string());
                            changed = true;
                        }
                    }
                }
                hoop_schema::ProjectsRegistryProjectsItem::Variant1 { workspaces, .. } => {
                    for ws in workspaces {
                        if ws.canonical_path.is_none() {
                            if let Ok(resolved) = fs::canonicalize(&ws.path) {
                                ws.canonical_path = Some(resolved.to_string_lossy().to_string());
                                changed = true;
                            }
                        }
                    }
                }
            }
        }
        changed
    }

    /// Serialize the registry back to the YAML file.
    fn write_back(path: &Path, registry: &hoop_schema::ProjectsRegistry) -> Result<()> {
        let yaml = serde_yaml::to_string(registry)
            .context("Failed to serialize projects.yaml")?;
        fs::write(path, yaml)
            .context("Failed to write projects.yaml")?;
        info!("Backfilled canonical_path entries in {}", path.display());
        Ok(())
    }

    /// Resolve canonical paths for all workspace entries.
    ///
    /// Uses the stored `canonical_path` when present; otherwise resolves via
    /// `fs::canonicalize`. Failures are logged and the raw path is kept as-is
    /// so that sessions from remote hosts aren't silently dropped.
    fn build_canonical_cache(
        registry: &hoop_schema::ProjectsRegistry,
    ) -> std::collections::HashMap<(String, PathBuf), PathBuf> {
        let mut cache = std::collections::HashMap::new();
        for project in &registry.projects {
            let name = project.name().to_string();
            for view in project.workspace_views() {
                let resolved = view
                    .canonical_path
                    .as_ref()
                    .filter(|cp| cp.exists())
                    .cloned()
                    .or_else(|| fs::canonicalize(&view.path).ok())
                    .unwrap_or_else(|| view.path.clone());
                cache.insert((name.clone(), view.path.clone()), resolved);
            }
        }
        cache
    }

    /// Look up the pre-resolved canonical path for a project workspace.
    /// Falls back to the raw path if not in cache.
    pub fn canonical_for(&self, project_name: &str, raw_path: &Path) -> PathBuf {
        self.canonical_cache
            .get(&(project_name.to_string(), raw_path.to_path_buf()))
            .cloned()
            .unwrap_or_else(|| raw_path.to_path_buf())
    }

    /// Get all workspace paths from all projects
    pub fn all_workspace_paths(&self) -> Vec<PathBuf> {
        self.registry
            .projects
            .iter()
            .flat_map(|p| p.all_paths())
            .collect()
    }

    /// Validate all workspaces exist and have .beads directories.
    ///
    /// Also detects duplicate canonical paths across projects — same workspace
    /// appearing via different raw paths (symlinks, alt mounts) produces a
    /// warning so the operator can merge or remove the duplicate.
    pub fn validate(&self) -> Vec<ConfigError> {
        let mut errors = Vec::new();

        // Track (canonical_path -> Vec<(project_name, raw_path)>) for dedup
        let mut canonical_map: std::collections::HashMap<PathBuf, Vec<(String, PathBuf)>> =
            std::collections::HashMap::new();

        for project in &self.registry.projects {
            for workspace in project.workspace_views() {
                if !workspace.path.exists() {
                    errors.push(ConfigError::validation(
                        format!(
                            "Project '{}': workspace path does not exist: {}",
                            project.name(),
                            workspace.path.display()
                        ),
                        Some(format!("projects[{}].path", project.name())),
                        Some("existing directory".to_string()),
                        Some(workspace.path.display().to_string()),
                    ));
                    continue;
                }

                let beads_path = workspace.path.join(".beads");
                if !beads_path.exists() || !beads_path.is_dir() {
                    errors.push(ConfigError::validation(
                        format!(
                            "Project '{}': .beads directory not found at: {}",
                            project.name(),
                            workspace.path.display()
                        ),
                        Some(format!("projects[{}].path", project.name())),
                        Some("directory containing .beads/".to_string()),
                        Some(workspace.path.display().to_string()),
                    ));
                }

                // Resolve canonical path for dedup detection
                let resolved = workspace
                    .canonical_path
                    .as_ref()
                    .and_then(|cp| {
                        if cp.exists() {
                            Some(cp.clone())
                        } else {
                            None
                        }
                    })
                    .or_else(|| fs::canonicalize(&workspace.path).ok())
                    .unwrap_or_else(|| workspace.path.clone());

                canonical_map
                    .entry(resolved)
                    .or_default()
                    .push((project.name().to_string(), workspace.path.clone()));
            }
        }

        // Warn on duplicate canonical paths across different projects
        for (canonical, entries) in &canonical_map {
            if entries.len() > 1 {
                let project_names: Vec<&str> =
                    entries.iter().map(|(name, _)| name.as_str()).collect();
                let unique_projects: std::collections::HashSet<&str> =
                    project_names.into_iter().collect();
                if unique_projects.len() > 1 {
                    let raw_paths: Vec<String> = entries
                        .iter()
                        .map(|(name, raw)| format!("{} ({})", raw.display(), name))
                        .collect();
                    errors.push(ConfigError::validation(
                        format!(
                            "Duplicate canonical path: {} maps to projects: {}",
                            canonical.display(),
                            raw_paths.join(", ")
                        ),
                        None,
                        None,
                        Some(canonical.display().to_string()),
                    ));
                }
            }
        }

        errors
    }
}

/// Configuration parse error details
#[derive(Debug, Clone)]
pub struct ConfigError {
    /// Human-readable error message
    pub message: String,
    /// Line number where the error occurred (1-indexed)
    pub line: usize,
    /// Column number where the error occurred (1-indexed)
    pub col: usize,
    /// Dotted path to the offending field (e.g. "projects[].name")
    pub field: Option<String>,
    /// What was expected (e.g. "string", "field present")
    pub expected: Option<String>,
    /// What was actually found (e.g. "integer", "missing")
    pub got: Option<String>,
}

impl ConfigError {
    /// Create a semantic validation error with structured details.
    pub fn validation(message: String, field: Option<String>, expected: Option<String>, got: Option<String>) -> Self {
        Self {
            message,
            line: 0,
            col: 0,
            field,
            expected,
            got,
        }
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<YamlError> for ConfigError {
    fn from(err: YamlError) -> Self {
        let msg = err.to_string();
        let (field, expected, got) = parse_serde_details(&msg);
        Self {
            message: msg,
            line: err.location().map(|l| line(&l)).unwrap_or(0),
            col: err.location().map(|l| column(&l)).unwrap_or(0),
            field,
            expected,
            got,
        }
    }
}

/// Extract structured details from serde error messages.
fn parse_serde_details(msg: &str) -> (Option<String>, Option<String>, Option<String>) {
    // Pattern: missing field `name` at line X column Y
    if let Some(rest) = msg.strip_prefix("missing field `") {
        let field_end = rest.find('`').unwrap_or(rest.len());
        let field_name = &rest[..field_end];
        return (
            Some(field_name.to_string()),
            Some("field present".to_string()),
            Some("missing".to_string()),
        );
    }

    // Pattern: unknown field `extra`, expected ...
    if let Some(rest) = msg.strip_prefix("unknown field `") {
        let field_end = rest.find('`').unwrap_or(rest.len());
        let field_name = &rest[..field_end];
        return (
            Some(field_name.to_string()),
            Some("known field".to_string()),
            Some("unknown field".to_string()),
        );
    }

    // Pattern: invalid type: integer `42`, expected a string
    if let Some(rest) = msg.strip_prefix("invalid type: ") {
        // rest = "integer `42`, expected a string" or similar
        if let Some(comma_pos) = rest.find(", expected ") {
            let got_part = &rest[..comma_pos];
            // Extract the type word (first word)
            let got_type = got_part.split_whitespace().next().unwrap_or(got_part);
            let expected_part = &rest[comma_pos + ", expected ".len()..];
            // Clean up "a string" -> "string"
            let expected_clean = expected_part
                .strip_prefix("a ")
                .unwrap_or(expected_part)
                .strip_prefix("an ")
                .unwrap_or(expected_part)
                .trim();
            return (
                None,
                Some(expected_clean.to_string()),
                Some(got_type.to_string()),
            );
        }
    }

    // Pattern: invalid value: ..., expected ...
    if let Some(rest) = msg.strip_prefix("invalid value: ") {
        if let Some(comma_pos) = rest.find(", expected ") {
            let got_part = &rest[..comma_pos];
            let got_summary = got_part.split_whitespace().next().unwrap_or(got_part);
            let expected_part = &rest[comma_pos + ", expected ".len()..];
            let expected_clean = expected_part
                .strip_prefix("a ")
                .unwrap_or(expected_part)
                .trim();
            return (
                None,
                Some(expected_clean.to_string()),
                Some(got_summary.to_string()),
            );
        }
    }

    // Pattern: <path>: data did not match any variant of untagged enum <type>
    // This occurs when typify-generated untagged enums can't match any variant.
    if let Some(pos) = msg.find("data did not match any variant of untagged enum ") {
        let field_path = msg[..pos].trim_end_matches(": ").trim();
        let enum_name_start = pos + "data did not match any variant of untagged enum ".len();
        let enum_name = msg[enum_name_start..]
            .split_whitespace()
            .next()
            .unwrap_or("enum");
        return (
            if field_path.is_empty() { None } else { Some(field_path.to_string()) },
            Some(format!("valid variant of {}", enum_name)),
            Some("no matching variant".to_string()),
        );
    }

    (None, None, None)
}

fn line(loc: &serde_yaml::Location) -> usize {
    loc.line()
}

fn column(loc: &serde_yaml::Location) -> usize {
    loc.column()
}

/// Events emitted by the projects watcher
#[derive(Debug, Clone)]
pub enum ProjectsEvent {
    /// Configuration was reloaded successfully
    ConfigReloaded {
        config: ProjectsConfig,
        /// Hash of the previous config file contents
        prev_hash: String,
        /// List of keys that changed between old and new config
        delta_keys: Vec<String>,
    },
    /// Configuration failed to parse
    ConfigError {
        error: ConfigError,
        /// Hash of the previous (last-good) config file contents
        prev_hash: String,
    },
}

/// Audit payload for config reload events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigReloadAudit {
    pub file: String,
    pub prev_hash: String,
    pub new_hash: String,
    pub delta_keys: Vec<String>,
    pub actor: String,
}

/// Audit payload for rejected config reload events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigReloadRejectedAudit {
    pub file: String,
    pub prev_hash: String,
    pub error: String,
    pub actor: String,
}

/// Compute the delta between two project registries.
///
/// Returns a sorted list of human-readable delta keys describing what changed.
pub fn compute_delta(old: &hoop_schema::ProjectsRegistry, new: &hoop_schema::ProjectsRegistry) -> Vec<String> {
    let mut deltas = Vec::new();

    let old_map: BTreeMap<&str, &hoop_schema::ProjectsRegistryProjectsItem> = old
        .projects
        .iter()
        .map(|p| (p.name(), p))
        .collect();
    let new_map: BTreeMap<&str, &hoop_schema::ProjectsRegistryProjectsItem> = new
        .projects
        .iter()
        .map(|p| (p.name(), p))
        .collect();

    // Projects removed
    for name in old_map.keys() {
        if !new_map.contains_key(name) {
            deltas.push(format!("-project:{}", name));
        }
    }

    // Projects added
    for name in new_map.keys() {
        if !old_map.contains_key(name) {
            deltas.push(format!("+project:{}", name));
        }
    }

    // Projects that exist in both — compare fields
    for (name, new_proj) in &new_map {
        if let Some(old_proj) = old_map.get(name) {
            let old_views = old_proj.workspace_views();
            let new_views = new_proj.workspace_views();

            let old_paths: Vec<_> = old_views.iter().map(|v| v.path.display().to_string()).collect();
            let new_paths: Vec<_> = new_views.iter().map(|v| v.path.display().to_string()).collect();
            if old_paths != new_paths {
                deltas.push(format!("~project:{}.paths", name));
            }

            let old_roles: Vec<_> = old_views.iter().map(|v| v.role.clone()).collect();
            let new_roles: Vec<_> = new_views.iter().map(|v| v.role.clone()).collect();
            if old_roles != new_roles {
                deltas.push(format!("~project:{}.roles", name));
            }

            if old_proj.label() != new_proj.label() {
                deltas.push(format!("~project:{}.label", name));
            }
            if old_proj.color() != new_proj.color() {
                deltas.push(format!("~project:{}.color", name));
            }
        }
    }

    deltas.sort();
    deltas
}

/// Projects configuration watcher
///
/// Watches ~/.hoop/projects.yaml for changes and emits events when
/// the file is modified. Implements debouncing to avoid emitting
/// multiple events for rapid successive edits.
pub struct ProjectsWatcher {
    config: Arc<Mutex<ProjectsConfig>>,
    event_tx: tokio::sync::broadcast::Sender<ProjectsEvent>,
    watcher: Option<RecommendedWatcher>,
    _shutdown_tx: mpsc::Sender<()>,
    debouncer: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl ProjectsWatcher {
    /// Create a new projects watcher
    pub fn new() -> Result<Self> {
        let (event_tx, _) = tokio::sync::broadcast::channel(64);
        let (shutdown_tx, _) = mpsc::channel(1);

        let config = ProjectsConfig::load()
            .context("Failed to load initial projects configuration")?;

        Ok(Self {
            config: Arc::new(Mutex::new(config)),
            event_tx,
            watcher: None,
            _shutdown_tx: shutdown_tx,
            debouncer: Arc::new(Mutex::new(None)),
        })
    }

    /// Subscribe to projects events
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<ProjectsEvent> {
        self.event_tx.subscribe()
    }

    /// Get the current configuration
    pub async fn config(&self) -> ProjectsConfig {
        self.config.lock().await.clone()
    }

    /// Start watching the projects.yaml file for changes
    pub fn start(&mut self) -> Result<()> {
        let watch_path = watch_path()?;

        // Ensure the .hoop directory exists
        if let Some(parent) = watch_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .context("Failed to create .hoop directory")?;
            }
        }

        let watch_path_for_event = watch_path.clone();
        let event_tx = self.event_tx.clone();
        let config = self.config.clone();
        let debouncer = self.debouncer.clone();

        let mut watcher = notify::recommended_watcher(move |res| {
            if let Err(e) = Self::handle_watch_event(
                res,
                &watch_path_for_event,
                &event_tx,
                config.clone(),
                debouncer.clone(),
            ) {
                warn!("Error handling projects watch event: {}", e);
            }
        })
        .context("Failed to create file watcher")?;

        // Watch the parent directory (NonRecursive mode)
        let watch_dir = if let Some(parent) = watch_path.parent() {
            if parent.exists() {
                parent.to_path_buf()
            } else {
                PathBuf::from(".")
            }
        } else {
            PathBuf::from(".")
        };

        watcher.watch(&watch_dir, RecursiveMode::NonRecursive)
            .context("Failed to watch projects directory")?;

        self.watcher = Some(watcher);

        info!("Projects watcher watching {}", watch_path.display());

        // Validate initial configuration
        let cfg = self.config.try_lock().unwrap();
        let errors = cfg.validate();
        drop(cfg);

        for error in errors {
            warn!("Projects configuration validation error: {}", error);
        }

        Ok(())
    }

    fn handle_watch_event(
        res: Result<notify::Event, notify::Error>,
        watch_path: &Path,
        event_tx: &tokio::sync::broadcast::Sender<ProjectsEvent>,
        config: Arc<Mutex<ProjectsConfig>>,
        debouncer: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    ) -> Result<()> {
        let event = res?;

        // Only care about modify events on the specific file
        let relevant = event.paths.iter().any(|p| p == watch_path);
        if !relevant {
            return Ok(());
        }

        use notify::EventKind::*;

        if matches!(event.kind, Create(_) | Modify(_)) {
            // Cancel any pending debounced task and start a new one
            let mut debouncer_guard = debouncer.blocking_lock();
            if let Some(handle) = debouncer_guard.take() {
                handle.abort();
            }

            let event_tx = event_tx.clone();
            let config_clone = config.clone();
            let watch_path = watch_path.to_path_buf();

            let handle = tokio::spawn(async move {
                // Wait 5 seconds before reloading (debounce)
                tokio::time::sleep(Duration::from_secs(5)).await;
                Self::reload_config(&watch_path, event_tx, config_clone).await;
            });

            *debouncer_guard = Some(handle);
        }

        Ok(())
    }

    /// Hot-reload with validate-before-apply + rollback (§17.5).
    ///
    /// Pipeline: schema-validate → semantic-validate → apply.
    /// On any failure the previous valid config stays in place and a
    /// `ConfigError` event is emitted with structured details.
    async fn reload_config(
        path: &Path,
        event_tx: tokio::sync::broadcast::Sender<ProjectsEvent>,
        config: Arc<Mutex<ProjectsConfig>>,
    ) {
        debug!("Reloading projects configuration from {}", path.display());

        // Capture prev_hash and old registry before reload
        let (prev_hash, old_registry) = {
            let cfg = config.lock().await;
            (cfg.content_hash.clone(), cfg.registry.clone())
        };

        // ── Phase 1: schema-validate (parse YAML into typed structure) ───────
        let parse_result = Self::parse_config(path).await;

        let mut new_config = match parse_result {
            Ok(cfg) => cfg,
            Err(error) => {
                let msg = error.message.clone();
                let _ = event_tx.send(ProjectsEvent::ConfigError {
                    error,
                    prev_hash,
                });
                warn!("Projects configuration rejected (schema): {}", msg);
                return;
            }
        };

        // ── Phase 2: semantic validation (paths, .beads, dedup) ─────────────
        let validation_errors = new_config.validate();
        if !validation_errors.is_empty() {
            // Collect all errors into a single structured ConfigError
            let first_error = validation_errors.first().unwrap().clone();
            let all_messages: Vec<String> = validation_errors.iter().map(|e| e.message.clone()).collect();
            let combined = ConfigError {
                message: all_messages.join("; "),
                line: first_error.line,
                col: first_error.col,
                field: first_error.field,
                expected: first_error.expected,
                got: first_error.got,
            };
            let msg = combined.message.clone();
            let _ = event_tx.send(ProjectsEvent::ConfigError {
                error: combined,
                prev_hash,
            });
            warn!("Projects configuration rejected (semantic): {}", msg);
            return;
        }

        // ── Phase 3: apply (store new config) ───────────────────────────────
        // Backfill canonical paths — only after validation passes
        if ProjectsConfig::backfill_canonical_paths(&mut new_config.registry) {
            if let Err(e) = ProjectsConfig::write_back(path, &new_config.registry) {
                warn!("Failed to backfill canonical paths on reload: {}", e);
            }
            new_config.canonical_cache = ProjectsConfig::build_canonical_cache(&new_config.registry);
        }

        *config.lock().await = new_config.clone();

        let delta_keys = compute_delta(&old_registry, &new_config.registry);
        let new_hash = new_config.content_hash.clone();

        let _ = event_tx.send(ProjectsEvent::ConfigReloaded {
            config: new_config,
            prev_hash: prev_hash.clone(),
            delta_keys,
        });
        info!(
            "Projects configuration reloaded successfully ({} → {})",
            &prev_hash[..8.min(prev_hash.len())],
            &new_hash[..8.min(new_hash.len())],
        );
    }

    /// Parse the config file without mutating shared state.
    /// Returns the parsed config on success or a structured error on failure.
    async fn parse_config(path: &Path) -> Result<ProjectsConfig, ConfigError> {
        if !path.exists() {
            return Ok(ProjectsConfig {
                registry: hoop_schema::ProjectsRegistry::default(),
                canonical_cache: std::collections::HashMap::new(),
                path: path.to_path_buf(),
                content_hash: String::new(),
            });
        }

        let contents = fs::read_to_string(path)
            .map_err(|e| ConfigError {
                message: format!("Failed to read file: {}", e),
                line: 0,
                col: 0,
                field: None,
                expected: None,
                got: None,
            })?;

        let content_hash = hex::encode(Sha256::digest(contents.as_bytes()));

        let registry: hoop_schema::ProjectsRegistry = serde_yaml::from_str(&contents)
            .map_err(ConfigError::from)?;

        let canonical_cache = ProjectsConfig::build_canonical_cache(&registry);

        Ok(ProjectsConfig {
            registry,
            canonical_cache,
            path: path.to_path_buf(),
            content_hash,
        })
    }
}

/// Get the path to the projects.yaml file
fn registry_path() -> Result<PathBuf> {
    let mut home = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."));
    home.push(".hoop");
    home.push("projects.yaml");
    Ok(home)
}

/// Get the path to watch (the directory containing projects.yaml)
fn watch_path() -> Result<PathBuf> {
    registry_path()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_from_yaml_error() {
        let yaml = r#"
projects:
  - name: test
    workspaces:
      - path: /tmp/test
        role: primary
  invalid_key: value
"#;
        let result: std::result::Result<hoop_schema::ProjectsRegistry, _> = serde_yaml::from_str(yaml);
        assert!(result.is_err());

        let yaml_err = result.unwrap_err();
        let config_err = ConfigError::from(yaml_err);

        assert!(!config_err.message.is_empty());
    }

    #[test]
    fn test_projects_config_empty() {
        let cfg = ProjectsConfig {
            registry: hoop_schema::ProjectsRegistry::default(),
            canonical_cache: std::collections::HashMap::new(),
            path: PathBuf::from("/nonexistent/projects.yaml"),
            content_hash: String::new(),
        };

        assert!(cfg.all_workspace_paths().is_empty());
        assert!(cfg.validate().is_empty());
    }

    #[test]
    fn test_validate_detects_duplicate_canonical_paths() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("real-repo");
        fs::create_dir_all(repo.join(".beads")).expect("mkdir");

        let link = tmp.path().join("link-repo");
        std::os::unix::fs::symlink(&repo, &link).expect("symlink");

        let canonical = fs::canonicalize(&repo).expect("canonicalize");

        // Two projects pointing at the same canonical location via symlink
        let yaml = format!(
            r#"
projects:
  - name: proj-a
    workspaces:
      - path: {}
        canonical_path: {}
        role: primary
  - name: proj-b
    workspaces:
      - path: {}
        canonical_path: {}
        role: primary
"#,
            repo.display(),
            canonical.display(),
            link.display(),
            canonical.display(),
        );

        let registry: hoop_schema::ProjectsRegistry =
            serde_yaml::from_str(&yaml).expect("parse");
        let cfg = ProjectsConfig {
            registry,
            canonical_cache: std::collections::HashMap::new(),
            path: PathBuf::from("/tmp/test-projects.yaml"),
            content_hash: String::new(),
        };

        let errors = cfg.validate();
        let dup_errors: Vec<_> = errors.iter().filter(|e| e.message.contains("Duplicate canonical path")).collect();
        assert_eq!(dup_errors.len(), 1, "should detect exactly one duplicate canonical path, got: {:?}", errors);
        assert!(dup_errors[0].message.contains("proj-a"), "should mention proj-a");
        assert!(dup_errors[0].message.contains("proj-b"), "should mention proj-b");
    }

    #[test]
    fn test_canonical_cache_resolves_paths() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("real-repo");
        fs::create_dir_all(repo.join(".beads")).expect("mkdir");

        let link = tmp.path().join("link-repo");
        std::os::unix::fs::symlink(&repo, &link).expect("symlink");

        let expected_canonical = fs::canonicalize(&repo).expect("canonicalize");

        // Project with symlink raw path but no canonical_path stored
        let yaml = format!(
            r#"
projects:
  - name: my-project
    workspaces:
      - path: {}
        role: primary
"#,
            link.display(),
        );

        let registry: hoop_schema::ProjectsRegistry =
            serde_yaml::from_str(&yaml).expect("parse");
        let cache = ProjectsConfig::build_canonical_cache(&registry);

        let resolved = cache
            .get(&(String::from("my-project"), link.clone()))
            .expect("should have cache entry");
        assert_eq!(*resolved, expected_canonical, "cache should resolve symlink to real path");
    }

    #[test]
    fn test_canonical_for_lookup() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        fs::create_dir_all(repo.join(".beads")).expect("mkdir");
        let canonical = fs::canonicalize(&repo).expect("canonicalize");

        let yaml = format!(
            r#"
projects:
  - name: test-proj
    path: {}
"#,
            repo.display(),
        );

        let registry: hoop_schema::ProjectsRegistry =
            serde_yaml::from_str(&yaml).expect("parse");
        let canonical_cache = ProjectsConfig::build_canonical_cache(&registry);
        let cfg = ProjectsConfig {
            registry,
            canonical_cache,
            path: PathBuf::from("/tmp/test.yaml"),
            content_hash: String::new(),
        };

        let resolved = cfg.canonical_for("test-proj", &repo);
        assert_eq!(resolved, canonical);
    }

    #[test]
    fn test_canonical_for_missing_returns_raw() {
        let cfg = ProjectsConfig {
            registry: hoop_schema::ProjectsRegistry::default(),
            canonical_cache: std::collections::HashMap::new(),
            path: PathBuf::from("/tmp/test.yaml"),
            content_hash: String::new(),
        };

        let raw = PathBuf::from("/nonexistent/path");
        assert_eq!(cfg.canonical_for("no-proj", &raw), raw);
    }

    // ── §17.5 Integration Tests: hot-reload validate-before-apply + rollback ──

    /// Helper: create a temp directory with a valid projects.yaml and .beads dirs.
    fn setup_valid_project(tmp: &tempfile::TempDir, name: &str) -> PathBuf {
        let repo = tmp.path().join(name);
        fs::create_dir_all(repo.join(".beads")).expect("mkdir .beads");
        repo
    }

    /// Write a valid projects.yaml with one project pointing at `repo`.
    fn write_valid_config(path: &Path, name: &str, repo: &Path) {
        let yaml = format!(
            "projects:\n  - name: {}\n    path: {}\n",
            name,
            repo.display()
        );
        fs::write(path, yaml).expect("write config");
    }

    /// Write an invalid YAML file (broken syntax).
    fn write_invalid_yaml(path: &Path) {
        fs::write(path, "projects:\n  - name: good\n    path: /exists\n  {\n")
            .expect("write invalid yaml");
    }

    /// Write a YAML with a schema-level error (unknown field type).
    fn write_schema_invalid(path: &Path) {
        fs::write(path, "projects:\n  - name: test\n    path: 42\n")
            .expect("write schema-invalid yaml");
    }

    /// Write a YAML that parses but fails semantic validation (nonexistent path).
    fn write_semantic_invalid(path: &Path) {
        let yaml = "projects:\n  - name: ghost\n    path: /nonexistent/path/xyz\n";
        fs::write(path, yaml).expect("write semantic-invalid yaml");
    }

    /// Integration test: edit-to-invalid-then-fix cycle preserves state (§17.5 acceptance).
    ///
    /// Cycle:
    /// 1. Start with valid config (project "alpha")
    /// 2. Write invalid YAML → verify old config kept + error event emitted
    /// 3. Write schema-invalid YAML → verify old config still kept + error event
    /// 4. Write valid config (project "beta") → verify new config applied + success event
    #[tokio::test]
    async fn test_edit_invalid_then_fix_cycle() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config_path = tmp.path().join("projects.yaml");

        // Phase 0: seed a valid config with project "alpha"
        let alpha = setup_valid_project(&tmp, "alpha-repo");
        write_valid_config(&config_path, "alpha", &alpha);

        let initial = ProjectsConfig::load_from(&config_path).expect("initial load");
        assert_eq!(initial.registry.projects.len(), 1);
        assert_eq!(initial.registry.projects[0].name(), "alpha");
        let initial_hash = initial.content_hash.clone();

        // Build the in-memory state that reload_config expects
        let shared_config = Arc::new(Mutex::new(initial));
        let (event_tx, _) = tokio::sync::broadcast::channel::<ProjectsEvent>(64);

        // Subscribe BEFORE reload so we don't miss the broadcast
        let mut rx = event_tx.subscribe();

        // ── Phase 1: write broken YAML → reject, keep "alpha" ────────────────
        write_invalid_yaml(&config_path);
        ProjectsWatcher::reload_config(&config_path, event_tx.clone(), shared_config.clone()).await;

        match rx.try_recv() {
            Ok(ProjectsEvent::ConfigError { error, prev_hash }) => {
                assert!(!error.message.is_empty(), "error should have a message");
                assert_eq!(prev_hash, initial_hash, "prev_hash should be the initial hash");
                assert!(error.line > 0, "parse error should report a line number");
            }
            other => panic!("expected ConfigError, got {:?}", other),
        }

        // Verify old config is still in place
        {
            let cfg = shared_config.lock().await;
            assert_eq!(cfg.registry.projects.len(), 1);
            assert_eq!(cfg.registry.projects[0].name(), "alpha");
            assert_eq!(cfg.content_hash, initial_hash, "hash should not change on rejection");
        }

        // ── Phase 2: write schema-invalid YAML → reject again ────────────────
        write_schema_invalid(&config_path);
        ProjectsWatcher::reload_config(&config_path, event_tx.clone(), shared_config.clone()).await;

        match rx.try_recv() {
            Ok(ProjectsEvent::ConfigError { error, .. }) => {
                assert!(!error.message.is_empty());
                assert!(
                    error.field.is_some() || error.expected.is_some() || error.got.is_some(),
                    "schema error should have structured details"
                );
            }
            other => panic!("expected ConfigError, got {:?}", other),
        }

        // Config should still be "alpha"
        {
            let cfg = shared_config.lock().await;
            assert_eq!(cfg.registry.projects[0].name(), "alpha");
        }

        // ── Phase 3: write valid config with project "beta" → accept ─────────
        let beta = setup_valid_project(&tmp, "beta-repo");
        write_valid_config(&config_path, "beta", &beta);
        ProjectsWatcher::reload_config(&config_path, event_tx.clone(), shared_config.clone()).await;

        match rx.try_recv() {
            Ok(ProjectsEvent::ConfigReloaded { config, prev_hash, delta_keys }) => {
                assert_eq!(config.registry.projects.len(), 1);
                assert_eq!(config.registry.projects[0].name(), "beta");
                assert_eq!(prev_hash, initial_hash);
                assert!(delta_keys.iter().any(|d| d.contains("-project:alpha")), "should show alpha removed");
                assert!(delta_keys.iter().any(|d| d.contains("+project:beta")), "should show beta added");
            }
            other => panic!("expected ConfigReloaded, got {:?}", other),
        }

        // Config should now be "beta"
        {
            let cfg = shared_config.lock().await;
            assert_eq!(cfg.registry.projects[0].name(), "beta");
        }
    }

    /// Integration test: semantic validation rejection (nonexistent workspace path).
    #[tokio::test]
    async fn test_semantic_validation_rejects_nonexistent_path() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config_path = tmp.path().join("projects.yaml");

        // Seed valid config
        let real = setup_valid_project(&tmp, "real");
        write_valid_config(&config_path, "real-project", &real);
        let initial = ProjectsConfig::load_from(&config_path).expect("load");
        let shared_config = Arc::new(Mutex::new(initial));
        let (event_tx, _) = tokio::sync::broadcast::channel::<ProjectsEvent>(64);

        // Subscribe before reload
        let mut rx = event_tx.subscribe();

        // Write config pointing at nonexistent path
        write_semantic_invalid(&config_path);
        ProjectsWatcher::reload_config(&config_path, event_tx.clone(), shared_config.clone()).await;

        match rx.try_recv() {
            Ok(ProjectsEvent::ConfigError { error, .. }) => {
                assert!(
                    error.message.contains("does not exist") || error.message.contains("not found"),
                    "semantic error should mention missing path: {}",
                    error.message
                );
                assert!(
                    error.field.is_some(),
                    "should identify the problematic field"
                );
            }
            other => panic!("expected ConfigError, got {:?}", other),
        }

        // Old config preserved
        let cfg = shared_config.lock().await;
        assert_eq!(cfg.registry.projects[0].name(), "real-project");
    }

    /// Schema violation details: field + expected-vs-got surface correctly.
    #[test]
    fn test_schema_violation_surfaces_field_line_expected_got() {
        // Missing required field — typify generates untagged enums, so the
        // error may report at the "projects" level rather than individual field.
        let yaml = "projects:\n  - path: /tmp/test\n";
        let result: std::result::Result<hoop_schema::ProjectsRegistry, _> = serde_yaml::from_str(yaml);
        assert!(result.is_err());
        let err = ConfigError::from(result.unwrap_err());

        // The error message should be non-empty and include line info
        assert!(!err.message.is_empty(), "should have an error message");
        // Structured details should be populated (exact field name varies by typify output)
        assert!(
            err.field.is_some() || err.expected.is_some() || err.got.is_some(),
            "schema error should have at least one structured detail: field={:?} expected={:?} got={:?}",
            err.field, err.expected, err.got
        );

        // Wrong type for a workspace path
        let yaml2 = "projects:\n  - name: 42\n    path: /tmp/test\n";
        let _result2: std::result::Result<hoop_schema::ProjectsRegistry, _> = serde_yaml::from_str(yaml2);
        // name=42 may parse as untagged enum variant (typify) — verify ConfigError surfaces for genuinely broken YAML
        let yaml3 = "projects:\n  - name: test\n    workspaces:\n      - path: 123\n";
        let result3: std::result::Result<hoop_schema::ProjectsRegistry, _> = serde_yaml::from_str(yaml3);
        assert!(result3.is_err(), "path=123 should fail parse");
        let err3 = ConfigError::from(result3.unwrap_err());
        // Typify untagged enums produce "data did not match any variant" errors
        assert!(
            err3.message.contains("invalid")
                || err3.message.contains("mismatch")
                || err3.message.contains("did not match"),
            "type error should mention invalid/mismatch/no match: {}",
            err3.message
        );
        // Should have line info
        assert!(err3.line > 0 || err3.col > 0, "should report location");
    }

    /// Integration test: config.yml (HoopConfig) hot-reload schema rejection.
    #[test]
    fn test_hoop_config_bad_schema_rejected() {
        // Valid minimal HoopConfig
        let valid = r#"{"schema_version": "1.0.0"}"#;
        let parsed: hoop_schema::HoopConfig = serde_json::from_str(valid).expect("minimal config should parse");
        let serialized = serde_json::to_string(&parsed).expect("serialize");
        assert!(serialized.contains("1.0.0"), "schema_version should round-trip");

        // Invalid schema_version format
        let bad_version = r#"{"schema_version": "not-a-version"}"#;
        let result: std::result::Result<hoop_schema::HoopConfig, _> = serde_json::from_str(bad_version);
        assert!(result.is_err(), "bad schema_version should be rejected");

        // Unknown field in nested section
        let unknown_field = r#"{"schema_version": "1.0.0", "agent": {"adapter": "unknown_adapter"}}"#;
        let result2: std::result::Result<hoop_schema::HoopConfig, _> = serde_json::from_str(unknown_field);
        assert!(result2.is_err(), "unknown adapter enum value should be rejected");
    }

    /// Integration test: write same content → no delta keys (no-op reload).
    #[tokio::test]
    async fn test_reload_identical_content_no_delta() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config_path = tmp.path().join("projects.yaml");
        let repo = setup_valid_project(&tmp, "repo");

        write_valid_config(&config_path, "same-project", &repo);
        let initial = ProjectsConfig::load_from(&config_path).expect("load");
        let initial_hash = initial.content_hash.clone();
        let shared_config = Arc::new(Mutex::new(initial));
        let (event_tx, _) = tokio::sync::broadcast::channel::<ProjectsEvent>(64);

        // Subscribe before reload
        let mut rx = event_tx.subscribe();

        // Re-write identical content (same hash, should still fire event but with empty deltas)
        write_valid_config(&config_path, "same-project", &repo);
        ProjectsWatcher::reload_config(&config_path, event_tx.clone(), shared_config.clone()).await;

        match rx.try_recv() {
            Ok(ProjectsEvent::ConfigReloaded { delta_keys, prev_hash, .. }) => {
                assert!(delta_keys.is_empty(), "no keys should change on identical reload");
                assert_eq!(prev_hash, initial_hash);
            }
            other => panic!("expected ConfigReloaded, got {:?}", other),
        }
    }
}
