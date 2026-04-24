//! Per-project runtime supervisor
//!
//! Each project gets its own supervised tokio task that manages:
//! - Bead reader for the project's workspaces
//! - Session tailer scoped to the project
//! - Panic recovery via JoinError detection
//! - Exponential backoff restart limiting
//!
//! A panic in one project's runtime is caught, logged, and restarted.
//! Other projects are unaffected. Missing .beads/ directories result in
//! error state rather than crash.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{debug, error, info, warn};

use crate::beads::{BeadEvent, BeadReader, BeadReaderConfig};
use crate::cost::CostAggregator;
use crate::events::{BeadEventData, EventTailer, EventTailerConfig, TailerEvent};
use crate::projects::ProjectsConfig;
use crate::sessions::{SessionEvent, SessionTailer, SessionTailerConfig};
use crate::shutdown::ShutdownPhase;
use crate::Bead;

/// Maximum consecutive failures before giving up
pub const MAX_CONSECUTIVE_FAILURES: usize = 5;

/// Base restart delay in seconds
pub const BASE_RESTART_DELAY_SECS: u64 = 1;

/// Maximum restart delay in seconds
pub const MAX_RESTART_DELAY_SECS: u64 = 300;

/// Project runtime state
#[derive(Debug, Clone)]
pub enum ProjectRuntimeState {
    /// Runtime is starting
    Starting,
    /// Runtime is healthy and running
    Healthy,
    /// Runtime has failed but will restart
    Failed {
        error: String,
        failed_at: DateTime<Utc>,
        consecutive_failures: usize,
        next_restart_at: DateTime<Utc>,
    },
    /// Runtime has a permanent error (will not auto-restart)
    Error {
        error: String,
        errored_at: DateTime<Utc>,
    },
    /// Runtime has been abandoned (too many failures)
    Abandoned {
        error: String,
        abandoned_at: DateTime<Utc>,
    },
}

impl ProjectRuntimeState {
    /// Returns true if the project is currently running (or starting)
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Starting | Self::Healthy)
    }

    /// Returns the error message if in a failed or error state
    pub fn error(&self) -> Option<&str> {
        match self {
            Self::Failed { error, .. } | Self::Abandoned { error, .. } | Self::Error { error, .. } => Some(error),
            _ => None,
        }
    }

    /// Returns a clean lowercase state name for frontend display
    pub fn to_display_string(&self) -> &'static str {
        match self {
            Self::Starting => "starting",
            Self::Healthy => "healthy",
            Self::Failed { .. } => "failed",
            Self::Error { .. } => "error",
            Self::Abandoned { .. } => "abandoned",
        }
    }
}

/// Project runtime status for UI display
#[derive(Debug, Clone)]
pub struct ProjectRuntimeStatus {
    /// Project name
    pub project_name: String,
    /// Project path
    pub project_path: PathBuf,
    /// Current runtime state
    pub state: ProjectRuntimeState,
    /// Number of workspaces being watched
    pub workspace_count: usize,
    /// Number of active beads
    pub bead_count: usize,
}

/// Per-project runtime
struct ProjectRuntime {
    /// Project name
    name: String,
    /// Workspace paths for this project
    workspaces: Vec<PathBuf>,
    /// Current state
    state: ProjectRuntimeState,
    /// Consecutive failure count
    consecutive_failures: usize,
    /// Task handle
    task_handle: Option<tokio::task::JoinHandle<()>>,
    /// Shutdown sender
    shutdown_tx: Option<mpsc::Sender<()>>,
    /// Shared reference to session tailer (for graceful shutdown)
    /// Stored in Arc<Mutex<>> so both the runtime task and supervisor can access it
    session_tailer: Arc<std::sync::Mutex<Option<SessionTailer>>>,
    /// Shared reference to bead readers (for graceful shutdown and error monitoring)
    bead_readers: Arc<std::sync::Mutex<Vec<BeadReader>>>,
    /// Bead count for this project (open beads)
    bead_count: usize,
}

/// Supervisor for all project runtimes
#[derive(Clone)]
pub struct ProjectSupervisor {
    /// All managed project runtimes
    runtimes: Arc<RwLock<HashMap<String, ProjectRuntime>>>,
    /// Bead event broadcast (for all projects)
    bead_tx: broadcast::Sender<BeadEvent>,
    /// Session event broadcast (for all projects)
    session_tx: broadcast::Sender<SessionEvent>,
    /// Worker registry for conversation updates
    worker_registry: Arc<crate::ws::WorkerRegistry>,
    /// Shared beads store
    beads: Arc<std::sync::RwLock<Vec<Bead>>>,
    /// Status broadcast for UI updates
    status_tx: broadcast::Sender<ProjectRuntimeStatus>,
    /// Shutdown coordinator for graceful shutdown
    shutdown: Arc<crate::shutdown::ShutdownCoordinator>,
    /// Event tailer for global events.jsonl (bead claim/close/release/update events)
    event_tailer: Arc<std::sync::Mutex<Option<EventTailer>>>,
    /// Cost aggregator for session usage
    cost_aggregator: Arc<std::sync::RwLock<CostAggregator>>,
}

impl std::fmt::Debug for ProjectSupervisor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProjectSupervisor")
            .field("runtimes", &"<RwLock>")
            .finish()
    }
}

impl ProjectSupervisor {
    /// Create a new project supervisor
    pub fn new(
        bead_tx: broadcast::Sender<BeadEvent>,
        session_tx: broadcast::Sender<SessionEvent>,
        worker_registry: Arc<crate::ws::WorkerRegistry>,
        beads: Arc<std::sync::RwLock<Vec<Bead>>>,
        shutdown: Arc<crate::shutdown::ShutdownCoordinator>,
        cost_aggregator: Arc<std::sync::RwLock<CostAggregator>>,
    ) -> Self {
        let (status_tx, _) = broadcast::channel(64);

        Self {
            runtimes: Arc::new(RwLock::new(HashMap::new())),
            bead_tx,
            session_tx,
            worker_registry,
            beads,
            status_tx,
            shutdown,
            event_tailer: Arc::new(std::sync::Mutex::new(None)),
            cost_aggregator,
        }
    }

    /// Start the global event tailer (events.jsonl)
    pub async fn start_event_tailer(&self) -> Result<()> {
        let mut event_tailer = EventTailer::new(EventTailerConfig {
            replay_on_startup: true,
            ..Default::default()
        }).context("Failed to create event tailer")?;

        event_tailer.start().context("Failed to start event tailer")?;

        // Subscribe to event tailer events and forward to worker registry
        let mut event_rx = event_tailer.subscribe();
        let worker_registry = self.worker_registry.clone();

        tokio::spawn(async move {
            while let Ok(event) = event_rx.recv().await {
                match event {
                    TailerEvent::Event(parsed) => {
                        // Convert to BeadEventData and add to registry
                        if let Some(bead_event) = BeadEventData::from_event(&parsed.event) {
                            let ws_event = crate::ws::BeadEventData {
                                timestamp: bead_event.timestamp.clone(),
                                event_type: bead_event.event_type.clone(),
                                bead_id: bead_event.bead_id.clone(),
                                worker: bead_event.worker.clone(),
                                line_number: Some(parsed.line_number),
                                raw: parsed.raw.clone(),
                            };
                            worker_registry.add_bead_event(ws_event).await;

                            // Also broadcast to WebSocket clients via a dedicated channel
                            // For now, we'll rely on periodic snapshots
                        }
                    }
                    TailerEvent::Rotated => {
                        debug!("Event log rotated");
                    }
                    TailerEvent::Error(e) => {
                        warn!("Event tailer error: {}", e);
                    }
                }
            }
        });

        *self.event_tailer.lock().unwrap() = Some(event_tailer);
        info!("Global event tailer started");
        Ok(())
    }

    /// Stop the event tailer gracefully
    pub async fn stop_event_tailer(&self) {
        if let Some(_tailer) = self.event_tailer.lock().unwrap().take() {
            debug!("Stopping event tailer");
            // The tailer will be dropped when replaced with None
        }
    }

    /// Subscribe to runtime status updates
    pub fn subscribe_status(&self) -> broadcast::Receiver<ProjectRuntimeStatus> {
        self.status_tx.subscribe()
    }

    /// Get current status of all runtimes
    pub async fn snapshot(&self) -> Vec<ProjectRuntimeStatus> {
        let runtimes = self.runtimes.read().await;
        let beads = self.beads.read().unwrap();
        runtimes
            .values()
            .map(|r| {
                let bead_count = count_open_beads_for_workspaces(&beads, &r.workspaces);
                ProjectRuntimeStatus {
                    project_name: r.name.clone(),
                    project_path: r.workspaces.first().cloned().unwrap_or_default(),
                    state: r.state.clone(),
                    workspace_count: r.workspaces.len(),
                    bead_count,
                }
            })
            .collect()
    }

    /// Reconcile runtimes with the given projects configuration
    pub async fn reconcile(&self, config: &ProjectsConfig) -> Result<()> {
        let mut runtimes = self.runtimes.write().await;

        // Build map of existing projects
        let existing: std::collections::HashSet<String> = runtimes.keys().cloned().collect();

        // Build map of desired projects
        let mut desired: HashMap<String, Vec<PathBuf>> = HashMap::new();
        for project in &config.registry.projects {
            let paths: Vec<PathBuf> = project.all_paths().map(|p| p.to_path_buf()).collect();
            desired.insert(project.name().to_string(), paths);
        }

        // Remove runtimes that are no longer in config
        for name in existing.difference(&desired.keys().cloned().collect()) {
            info!("Removing runtime for project: {}", name);
            if let Some(mut runtime) = runtimes.remove(name) {
                self.stop_runtime(&mut runtime).await;
            }
        }

        // Add or update runtimes
        for (name, paths) in desired {
            if paths.is_empty() {
                warn!("Project {} has no workspace paths, skipping", name);
                continue;
            }

            // Check if runtime already exists
            if let Some(runtime) = runtimes.get_mut(&name) {
                // Update workspaces if changed
                if runtime.workspaces != paths {
                    info!("Restarting runtime for project {} (workspaces changed)", name);
                    self.stop_runtime(runtime).await;
                    runtime.workspaces = paths.clone();
                    self.start_runtime(runtime)?;
                }
            } else {
                // Create new runtime
                info!("Starting runtime for project: {}", name);
                let mut runtime = ProjectRuntime {
                    name: name.clone(),
                    workspaces: paths.clone(),
                    state: ProjectRuntimeState::Starting,
                    consecutive_failures: 0,
                    task_handle: None,
                    shutdown_tx: None,
                    session_tailer: Arc::new(std::sync::Mutex::new(None)),
                    bead_readers: Arc::new(std::sync::Mutex::new(Vec::new())),
                    bead_count: 0,
                };
                self.start_runtime(&mut runtime)?;
                runtimes.insert(name, runtime);
            }
        }

        Ok(())
    }

    /// Stop a project runtime
    async fn stop_runtime(&self, runtime: &mut ProjectRuntime) {
        // Send shutdown signal first
        if let Some(tx) = &runtime.shutdown_tx {
            let _ = tx.send(()).await;
        }

        // Stop all bead readers
        let bead_readers = runtime.bead_readers.lock().unwrap().drain(..).collect::<Vec<_>>();
        for mut bead_reader in bead_readers {
            if let Err(e) = bead_reader.stop().await {
                warn!("Error stopping bead reader for {}: {}", runtime.name, e);
            }
        }

        // Flush session tailer state via the shared reference
        let tailer_opt = runtime.session_tailer.lock().unwrap().take();
        if let Some(mut session_tailer) = tailer_opt {
            if let Err(e) = session_tailer.stop().await {
                warn!("Error stopping session tailer for {}: {}", runtime.name, e);
            }
        }

        // Give the task time to shut down gracefully (max 2s), then abort
        if let Some(handle) = runtime.task_handle.take() {
            let abort_handle = handle.abort_handle();
            if tokio::time::timeout(Duration::from_secs(2), handle)
                .await
                .is_err()
            {
                abort_handle.abort();
            }
        }

        runtime.shutdown_tx = None;
        runtime.state = ProjectRuntimeState::Starting;
        runtime.bead_count = 0;
    }

    /// Start a project runtime with supervision
    fn start_runtime(&self, runtime: &mut ProjectRuntime) -> Result<()> {
        let project_name = runtime.name.clone();
        let workspaces = runtime.workspaces.clone();
        let bead_tx = self.bead_tx.clone();
        let session_tx = self.session_tx.clone();
        let worker_registry = self.worker_registry.clone();
        let beads = self.beads.clone();
        let _runtimes = self.runtimes.clone();
        let _status_tx = self.status_tx.clone();
        let shutdown = self.shutdown.clone();
        let session_tailer = runtime.session_tailer.clone();
        let bead_readers = runtime.bead_readers.clone();
        let cost_aggregator = self.cost_aggregator.clone();
        let supervisor = self.clone();

        // Create shutdown channel for this runtime
        let (shutdown_tx, _shutdown_rx) = mpsc::channel::<()>(1);
        runtime.shutdown_tx = Some(shutdown_tx);

        // Create error channel for propagating errors from spawned tasks
        let (error_tx, _error_rx) = mpsc::channel::<anyhow::Error>(1);

        // Spawn the supervised task
        // tokio::spawn catches panics and returns JoinError on .await
        let project_name_clone = project_name.clone();
        let supervisor_clone = supervisor.clone();

        let task_handle = tokio::spawn(async move {
            info!("Project runtime started: {}", project_name_clone);

            // Run the project runtime
            let result = Self::run_project_runtime(
                project_name_clone.clone(),
                workspaces.clone(),
                bead_tx,
                session_tx,
                worker_registry,
                beads.clone(),
                shutdown,
                session_tailer,
                bead_readers,
                error_tx,
                cost_aggregator,
            ).await;

            match result {
                Ok(()) => {
                    info!("Project runtime shut down gracefully: {}", project_name_clone);
                }
                Err(e) => {
                    error!("Project runtime failed: {} - error: {}", project_name_clone, e);
                    supervisor_clone.handle_failure(&project_name_clone, &e.to_string()).await;
                }
            }
        });

        // Store the task handle for later access
        runtime.task_handle = Some(task_handle);
        Ok(())
    }

    /// Restart a specific project runtime
    async fn restart_runtime(&self, project_name: &str) -> Result<()> {
        let mut runtimes = self.runtimes.write().await;
        if let Some(runtime) = runtimes.get_mut(project_name) {
            // First stop the old runtime
            self.stop_runtime(runtime).await;
            // Then start it again
            self.start_runtime(runtime)?;
            Ok(())
        } else {
            warn!("Cannot restart runtime for {}: not found", project_name);
            Err(anyhow::anyhow!("Runtime not found: {}", project_name))
        }
    }

    /// Check if an error is permanent (should not trigger auto-restart)
    pub fn is_permanent_error(error: &str) -> bool {
        let error_lower = error.to_lowercase();
        error_lower.contains("workspace path does not exist") ||
        error_lower.contains(".beads directory not found") ||
        error_lower.contains("does not exist")
    }

    /// Handle runtime failure with exponential backoff and auto-restart
    async fn handle_failure(&self, project_name: &str, error: &str) {
        // Check if this is a permanent error (should not auto-restart)
        if Self::is_permanent_error(error) {
            let mut runtimes = self.runtimes.write().await;
            if let Some(runtime) = runtimes.get_mut(project_name) {
                runtime.state = ProjectRuntimeState::Error {
                    error: error.to_string(),
                    errored_at: Utc::now(),
                };
                error!(
                    "Project runtime {}: permanent error - {}",
                    project_name, error
                );

                // Send status update
                let _ = self.status_tx.send(ProjectRuntimeStatus {
                    project_name: project_name.to_string(),
                    project_path: runtime.workspaces.first().cloned().unwrap_or_default(),
                    state: runtime.state.clone(),
                    workspace_count: runtime.workspaces.len(),
                    bead_count: 0,
                });
            }
            return;
        }

        // Handle transient errors with backoff and restart
        let (should_restart, delay_secs) = {
            let mut runtimes = self.runtimes.write().await;
            if let Some(runtime) = runtimes.get_mut(project_name) {
                runtime.consecutive_failures += 1;

                // Check if we should abandon this runtime
                if runtime.consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                    runtime.state = ProjectRuntimeState::Abandoned {
                        error: error.to_string(),
                        abandoned_at: Utc::now(),
                    };
                    error!(
                        "Project runtime abandoned after {} consecutive failures: {}",
                        runtime.consecutive_failures, project_name
                    );

                    // Send status update
                    let _ = self.status_tx.send(ProjectRuntimeStatus {
                        project_name: project_name.to_string(),
                        project_path: runtime.workspaces.first().cloned().unwrap_or_default(),
                        state: runtime.state.clone(),
                        workspace_count: runtime.workspaces.len(),
                        bead_count: 0,
                    });

                    return;
                }

                // Calculate exponential backoff delay
                let delay_secs = (BASE_RESTART_DELAY_SECS * 2_u64.pow(runtime.consecutive_failures as u32 - 1))
                    .min(MAX_RESTART_DELAY_SECS);
                let next_restart = Utc::now() + chrono::Duration::seconds(delay_secs as i64);

                runtime.state = ProjectRuntimeState::Failed {
                    error: error.to_string(),
                    failed_at: Utc::now(),
                    consecutive_failures: runtime.consecutive_failures,
                    next_restart_at: next_restart,
                };

                warn!(
                    "Project runtime failed (attempt {}/{}): {} - restarting in {}s",
                    runtime.consecutive_failures, MAX_CONSECUTIVE_FAILURES, project_name, delay_secs
                );

                // Send status update
                let _ = self.status_tx.send(ProjectRuntimeStatus {
                    project_name: project_name.to_string(),
                    project_path: runtime.workspaces.first().cloned().unwrap_or_default(),
                    state: runtime.state.clone(),
                    workspace_count: runtime.workspaces.len(),
                    bead_count: 0,
                });

                (true, delay_secs)
            } else {
                return;
            }
        };

        // Schedule restart
        if should_restart {
            let supervisor_clone = self.clone();
            let project_name = project_name.to_string();

            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(delay_secs)).await;
                info!("Restarting project runtime after backoff: {}", project_name);
                if let Err(e) = supervisor_clone.restart_runtime(&project_name).await {
                    error!("Failed to restart runtime {}: {}", project_name, e);
                }
            });
        }
    }

    /// Run the project runtime (bead reader + session tailer)
    #[allow(clippy::too_many_arguments)]
    async fn run_project_runtime(
        project_name: String,
        workspaces: Vec<PathBuf>,
        bead_tx: broadcast::Sender<BeadEvent>,
        _session_tx: broadcast::Sender<SessionEvent>,
        worker_registry: Arc<crate::ws::WorkerRegistry>,
        beads: Arc<std::sync::RwLock<Vec<Bead>>>,
        shutdown: Arc<crate::shutdown::ShutdownCoordinator>,
        session_tailer_clone: Arc<std::sync::Mutex<Option<SessionTailer>>>,
        bead_readers_clone: Arc<std::sync::Mutex<Vec<BeadReader>>>,
        error_tx: mpsc::Sender<anyhow::Error>,
        _cost_aggregator: Arc<std::sync::RwLock<CostAggregator>>,
    ) -> Result<()> {
        // Subscribe to shutdown phases
        let mut shutdown_rx = shutdown.subscribe();

        // Validate workspaces exist and have .beads directories
        for workspace in &workspaces {
            if !workspace.exists() {
                return Err(anyhow::anyhow!(
                    "Workspace path does not exist: {}",
                    workspace.display()
                ));
            }

            let beads_path = workspace.join(".beads");
            if !beads_path.exists() || !beads_path.is_dir() {
                return Err(anyhow::anyhow!(
                    ".beads directory not found at: {}",
                    workspace.display()
                ));
            }
        }

        // Initialize bead readers for each workspace
        let mut local_bead_readers = Vec::new();
        for workspace in &workspaces {
            let bead_reader_config = BeadReaderConfig {
                workspace_path: workspace.to_path_buf(),
            };

            let mut bead_reader = BeadReader::new(bead_reader_config)
                .with_context(|| format!("Failed to create bead reader for {}", workspace.display()))?;

            // Replay existing beads
            let issues_path = workspace.join(".beads").join("issues.jsonl");
            if issues_path.exists() {
                bead_reader.replay_file()
                    .with_context(|| format!("Failed to replay beads for {}", workspace.display()))?;
            }

            bead_reader.start()
                .with_context(|| format!("Failed to start bead reader for {}", workspace.display()))?;

            // Subscribe to bead events
            let mut rx = bead_reader.subscribe();
            let workspace_clone = workspace.clone();
            let beads_clone = beads.clone();
            let bead_tx_clone = bead_tx.clone();
            let _project_name_clone = project_name.clone();
            let error_tx_clone = error_tx.clone();

            tokio::spawn(async move {
                while let Ok(event) = rx.recv().await {
                    match event {
                        BeadEvent::BeadsUpdated { beads: new_beads } => {
                            // Update shared beads store
                            let mut all_beads = beads_clone.write().unwrap().clone();
                            let workspace_bead_ids: std::collections::HashSet<String> = new_beads
                                .iter()
                                .map(|b| b.id.clone())
                                .collect();

                            // Remove old beads from this workspace
                            all_beads.retain(|b| !workspace_bead_ids.contains(&b.id) || new_beads.iter().any(|nb| nb.id == b.id));
                            // Add new beads from this workspace
                            all_beads.extend(new_beads.clone());
                            // Sort by created_at descending
                            all_beads.sort_by_key(|b| std::cmp::Reverse(b.created_at));

                            *beads_clone.write().unwrap() = all_beads.clone();

                            // Forward to broadcast
                            let _ = bead_tx_clone.send(BeadEvent::BeadsUpdated { beads: new_beads });

                            debug!("Beads updated for workspace: {}", workspace_clone.display());
                        }
                        BeadEvent::Error(e) => {
                            error!("Bead reader error for {}: {}", workspace_clone.display(), e);
                            // Send error to runtime via channel
                            let _ = error_tx_clone.send(anyhow::anyhow!("Bead reader error for {}: {}", workspace_clone.display(), e)).await;
                        }
                    }
                }
            });

            local_bead_readers.push(bead_reader);
        }

        // Store bead readers in shared reference for external access and graceful shutdown
        {
            let mut bead_readers_ref = bead_readers_clone.lock().unwrap();
            *bead_readers_ref = local_bead_readers;
        }

        // Initialize session tailer for this project
        // Use the first workspace as the project path for session filtering
        let project_path = workspaces.first().cloned().unwrap_or_default();

        let session_tailer_config = SessionTailerConfig {
            claude_projects_dir: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".claude")
                .join("projects"),
            project_path: Some(project_path.clone()),
            discovery_concurrency: 16,
            poll_interval_secs: 5,
            enabled_adapters: vec![],
        };

        let mut session_tailer = SessionTailer::new(session_tailer_config)
            .context("Failed to create session tailer")?;

        // Subscribe to session events
        let mut session_rx = session_tailer.subscribe();
        let worker_registry_clone = worker_registry.clone();
        let error_tx_clone = error_tx.clone();
        let project_name_for_tailer = project_name.clone();

        tokio::spawn(async move {
            while let Ok(event) = session_rx.recv().await {
                match event {
                    SessionEvent::ConversationsUpdated { sessions } => {
                        worker_registry_clone.update_conversations(sessions).await;
                    }
                    SessionEvent::SessionBound { .. } => {
                        // Registry will handle this via the WebSocket
                    }
                    SessionEvent::Error(e) => {
                        error!("Session tailer error for project {}: {}", project_name_for_tailer, e);
                        // Send error to runtime via channel
                        let _ = error_tx_clone.send(anyhow::anyhow!("Session tailer error for {}: {}", project_name_for_tailer, e)).await;
                    }
                    SessionEvent::TagJoinBound { .. } => {}
                }
            }
        });

        session_tailer.start()
            .context("Failed to start session tailer")?;

        info!(
            "Project runtime running: {} ({} workspaces)",
            project_name,
            workspaces.len()
        );

        // Store the session tailer in the shared reference for external access
        {
            let mut tailer_ref = session_tailer_clone.lock().unwrap();
            *tailer_ref = Some(session_tailer);
        }

        // Wait for shutdown signal or FlushState phase
        loop {
            tokio::select! {
                // Listen for shutdown phases
                phase = shutdown_rx.recv() => {
                    match phase {
                        Ok(ShutdownPhase::FlushState) => {
                            info!("Project runtime {}: flushing in-flight state", project_name);
                            // Flush session tailer to ensure all pending data is written
                            let tailer_opt = session_tailer_clone.lock().unwrap().take();
                            if let Some(mut tailer) = tailer_opt {
                                if let Err(e) = tailer.stop().await {
                                    warn!("Error flushing session tailer for {}: {}", project_name, e);
                                }
                            }
                            // Bead readers are file-based and don't need explicit flushing
                            debug!("Project runtime {}: flushed state", project_name);
                        }
                        Ok(ShutdownPhase::Exit) => {
                            info!("Project runtime {}: exiting", project_name);
                            break;
                        }
                        Ok(_) => {
                            // Other phases - continue
                        }
                        Err(_) => {
                            // Channel closed - exit
                            break;
                        }
                    }
                }
                // Also listen for the local shutdown signal
                _ = tokio::signal::ctrl_c() => {
                    info!("Project runtime {}: received Ctrl-C", project_name);
                    break;
                }
            }
        }

        info!("Project runtime shut down: {}", project_name);
        Ok(())
    }
}

/// Convert a panic payload to a string
#[allow(dead_code)]
fn panic_payload_to_string(payload: &dyn std::any::Any) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "(unknown panic type)".to_string()
    }
}

/// Count open beads for a given set of workspace paths
/// Note: Currently beads don't have workspace association, so we count all open beads
/// TODO: Add workspace/path association to beads for proper filtering
fn count_open_beads_for_workspaces(beads: &[Bead], _workspaces: &[PathBuf]) -> usize {
    beads
        .iter()
        .filter(|b| matches!(b.status, crate::BeadStatus::Open))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panic_payload_to_string_str() {
        let payload = "test panic";
        assert_eq!(panic_payload_to_string(&payload), "test panic");
    }

    #[test]
    fn test_panic_payload_to_string_string() {
        let payload = String::from("test panic");
        assert_eq!(panic_payload_to_string(&payload), "test panic");
    }

    #[test]
    fn test_panic_payload_to_string_unknown() {
        let payload = 42i32;
        assert_eq!(panic_payload_to_string(&payload), "(unknown panic type)");
    }

    #[test]
    fn test_project_runtime_state_is_running() {
        assert!(ProjectRuntimeState::Starting.is_running());
        assert!(ProjectRuntimeState::Healthy.is_running());
        assert!(!ProjectRuntimeState::Failed {
            error: "test".to_string(),
            failed_at: Utc::now(),
            consecutive_failures: 1,
            next_restart_at: Utc::now(),
        }
        .is_running());
        assert!(!ProjectRuntimeState::Error {
            error: "test".to_string(),
            errored_at: Utc::now(),
        }
        .is_running());
        assert!(!ProjectRuntimeState::Abandoned {
            error: "test".to_string(),
            abandoned_at: Utc::now(),
        }
        .is_running());
    }

    #[test]
    fn test_is_permanent_error() {
        assert!(ProjectSupervisor::is_permanent_error("Workspace path does not exist: /path"));
        assert!(ProjectSupervisor::is_permanent_error(".beads directory not found at: /path"));
        assert!(!ProjectSupervisor::is_permanent_error("Connection refused"));
        assert!(!ProjectSupervisor::is_permanent_error("Timeout"));
    }
}
