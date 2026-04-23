//! Projects configuration hot-reload
//!
//! Watches ~/.hoop/projects.yaml for changes and emits events when the
//! configuration is updated. Handles validation and error reporting.

use anyhow::{Context, Result};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde_yaml::Error as YamlError;
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
    /// Path to the projects.yaml file
    pub path: PathBuf,
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
            // Return empty registry if file doesn't exist
            return Ok(Self {
                registry: hoop_schema::ProjectsRegistry::default(),
                path: path.to_path_buf(),
            });
        }

        let contents = fs::read_to_string(path)
            .context("Failed to read projects.yaml")?;

        let registry: hoop_schema::ProjectsRegistry = serde_yaml::from_str(&contents)
            .context("Failed to parse projects.yaml")?;

        Ok(Self {
            registry,
            path: path.to_path_buf(),
        })
    }

    /// Get all workspace paths from all projects
    pub fn all_workspace_paths(&self) -> Vec<PathBuf> {
        self.registry
            .projects
            .iter()
            .flat_map(|p| p.all_paths())
            .collect()
    }

    /// Validate all workspaces exist and have .beads directories
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        for project in &self.registry.projects {
            for workspace in project.workspace_views() {
                if !workspace.path.exists() {
                    errors.push(format!(
                        "Project '{}': workspace path does not exist: {}",
                        project.name(),
                        workspace.path.display()
                    ));
                    continue;
                }

                let beads_path = workspace.path.join(".beads");
                if !beads_path.exists() || !beads_path.is_dir() {
                    errors.push(format!(
                        "Project '{}': .beads directory not found at: {}",
                        project.name(),
                        workspace.path.display()
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
}

impl From<YamlError> for ConfigError {
    fn from(err: YamlError) -> Self {
        Self {
            message: err.to_string(),
            line: err.location().map(|l| line(&l)).unwrap_or(0),
            col: err.location().map(|l| column(&l)).unwrap_or(0),
        }
    }
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
    ConfigReloaded { config: ProjectsConfig },
    /// Configuration failed to parse
    ConfigError { error: ConfigError },
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

        let result = Self::do_reload(path, config.clone()).await;

        match result {
            Ok(new_config) => {
                // Validate the new configuration
                let errors = new_config.validate();
                for error in &errors {
                    warn!("Projects configuration validation error: {}", error);
                }

                let _ = event_tx.send(ProjectsEvent::ConfigReloaded {
                    config: new_config,
                });
                info!("Projects configuration reloaded successfully");
            }
            Err(error) => {
                let msg = error.message.clone();
                let _ = event_tx.send(ProjectsEvent::ConfigError { error });
                warn!("Projects configuration failed to load: {}", msg);
            }
        }
    }

    async fn do_reload(
        path: &Path,
        config: Arc<Mutex<ProjectsConfig>>,
    ) -> Result<ProjectsConfig, ConfigError> {
        if !path.exists() {
            // File was deleted, return empty config
            let new_config = ProjectsConfig {
                registry: hoop_schema::ProjectsRegistry::default(),
                path: path.to_path_buf(),
            };
            *config.lock().await = new_config.clone();
            return Ok(new_config);
        }

        let contents = fs::read_to_string(path)
            .map_err(|e| ConfigError {
                message: format!("Failed to read file: {}", e),
                line: 0,
                col: 0,
            })?;

        let registry: hoop_schema::ProjectsRegistry = serde_yaml::from_str(&contents)
            .map_err(|e| ConfigError::from(e))?;

        let new_config = ProjectsConfig {
            registry,
            path: path.to_path_buf(),
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
            path: PathBuf::from("/nonexistent/projects.yaml"),
        };

        assert!(cfg.all_workspace_paths().is_empty());
        assert!(cfg.validate().is_empty());
    }
}
