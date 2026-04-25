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

        let registry: hoop_schema::ProjectsRegistry = serde_yaml::from_str(&contents)
            .context("Failed to parse projects.yaml")?;

        let canonical_cache = Self::build_canonical_cache(&registry);

        Ok(Self {
            registry,
            canonical_cache,
            path: path.to_path_buf(),
            content_hash,
        })
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

        let result = Self::do_reload(path, config.clone()).await;

        match result {
            Ok(new_config) => {
                // Validate the new configuration
                let errors = new_config.validate();
                for error in &errors {
                    warn!("Projects configuration validation error: {}", error);
                }

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
            Err(error) => {
                let msg = error.message.clone();
                let _ = event_tx.send(ProjectsEvent::ConfigError {
                    error,
                    prev_hash,
                });
                warn!("Projects configuration failed to load: {}", msg);
            }
        }
    }

    async fn do_reload(
        path: &Path,
        config: Arc<Mutex<ProjectsConfig>>,
    ) -> Result<ProjectsConfig, ConfigError> {
        if !path.exists() {
            let new_config = ProjectsConfig {
                registry: hoop_schema::ProjectsRegistry::default(),
                canonical_cache: std::collections::HashMap::new(),
                path: path.to_path_buf(),
                content_hash: String::new(),
            };
            *config.lock().await = new_config.clone();
            return Ok(new_config);
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

        let new_config = ProjectsConfig {
            registry,
            canonical_cache,
            path: path.to_path_buf(),
            content_hash,
        };

        *config.lock().await = new_config.clone();
        Ok(new_config)
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
}
