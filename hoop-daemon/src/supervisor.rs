//! Per-project runtime supervisor
//!
//! This module implements the supervisor subsystem responsible for managing isolated
//! project runtimes with fault tolerance and hot-reload capabilities.
//!
//! # Responsibilities
//!
//! ## 1. Restart-on-Panic (Per-Project)
//!
//! Each project runtime runs in a supervised tokio task. Panics are caught via
//! `JoinError` detection and trigger automatic restart with exponential backoff.
//!
//! - Transient errors (network, temporary failures) → retry with backoff
//! - Permanent errors (missing workspace/.beads) → Error state (no auto-restart)
//! - After `MAX_CONSECUTIVE_FAILURES` (5) → Abandoned state
//! - Each panic increments `hoop_errors_total{subsystem=supervisor,kind=project_panic}`
//!
//! Backoff formula: `BASE_RESTART_DELAY_SECS * 2^(failures-1)` capped at `MAX_RESTART_DELAY_SECS`
//! - 1st failure: 1s, 2nd: 2s, 3rd: 4s, 4th: 8s, 5th: 16s (max: 300s)
//!
//! ## 2. Hot-Reload Apply (New Project Registration)
//!
//! The `reconcile()` method compares desired project configuration against active
//! runtimes and applies changes:
//!
//! - New projects → spawn runtime immediately
//! - Removed projects → graceful shutdown, cleanup
//! - Workspace path changes → restart runtime with new paths
//! - No-op (unchanged) → leave runtime untouched
//!
//! Each state transition broadcasts `ProjectRuntimeStatus` for UI updates.
//!
//! ## 3. Per-Project Isolation
//!
//! Each project runtime is fully isolated:
//!
//! - Separate tokio task with independent panic recovery
//! - Separate BeadReader instances per workspace
//! - Separate SessionTailer scoped to project path
//! - Shared broadcast channels for cross-project events only
//!
//! Isolation guarantee: N panics in project A → project B continues unaffected.
//!
//! ## 4. Graceful Shutdown Coordination
//!
//! The supervisor coordinates graceful shutdown via `ShutdownCoordinator`:
//!
//! - `FlushState` phase → flush session tailer state to disk
//! - `Exit` phase → all runtimes exit cleanly
//! - Bead readers are stopped (file-based, no explicit flush needed)
//! - Session tailers flush pending data before stopping
//! - Task handles aborted after 2s grace period
//!
//! ## 5. Health Reporting for /readyz
//!
//! The supervisor provides health status via:
//!
//! - `snapshot()` → current state of all runtimes
//! - `subscribe_status()` → broadcast channel for live updates
//! - Each runtime reports state: Starting, Healthy, Failed, Error, Abandoned
//!
//! Health check logic (used by /readyz):
//! - At least one runtime in Healthy/Starting → ready
//! - All runtimes in Failed/Error/Abandoned → not ready
//!
//! # State Machine
//!
//! ```text
//! Starting → Healthy (runtime initialized successfully)
//! Starting → Error (permanent error like missing .beads)
//! Healthy → Failed (panic or transient error)
//! Failed → Healthy (after successful restart)
//! Failed → Abandoned (after MAX_CONSECUTIVE_FAILURES)
//! Any → Exit (on shutdown signal)
//! ```
//!
//! # Metrics
//!
//! - `hoop_errors_total{subsystem=supervisor,kind=project_panic}` - incremented per panic
//! - Status broadcasts consumed by UI for runtime state display

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{debug, error, info, warn};

use crate::beads::{BeadEvent, BeadReader, BeadReaderConfig};
use crate::metrics::metrics;
use crate::cost::CostAggregator;
use crate::events::{BeadEventData, EventTailer, EventTailerConfig, TailerEvent};
use crate::projects::ProjectsConfig;
use crate::script_trigger::{trigger_matching_scripts, EventContext};
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
            Self::Failed { error, .. }
            | Self::Abandoned { error, .. }
            | Self::Error { error, .. } => Some(error),
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
    /// Number of workers running (total across all projects)
    pub worker_count: usize,
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
    /// Vector index for semantic deduplication (hoop-ttb.5.9.1)
    vector_index: Arc<std::sync::RwLock<crate::vector_index::VectorIndex>>,
    /// Scripts directory for event-triggered scripts
    scripts_dir: PathBuf,
    /// Stuck detector for worker health monitoring (§C1, hoop-ttb.3.25)
    stuck_detector: Arc<std::sync::Mutex<crate::stuck_detector::StuckDetector>>,
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
        vector_index: Arc<std::sync::RwLock<crate::vector_index::VectorIndex>>,
        scripts_dir: PathBuf,
        stuck_detector: Arc<std::sync::Mutex<crate::stuck_detector::StuckDetector>>,
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
            vector_index,
            scripts_dir,
            stuck_detector,
        }
    }

    /// Set the scripts directory (for event-triggered scripts)
    pub fn set_scripts_dir(&mut self, scripts_dir: PathBuf) {
        self.scripts_dir = scripts_dir;
    }

    /// Start the global event tailer (events.jsonl)
    pub async fn start_event_tailer(&self) -> Result<()> {
        let mut event_tailer = EventTailer::new(EventTailerConfig {
            replay_on_startup: true,
            ..Default::default()
        })
        .context("Failed to create event tailer")?;

        event_tailer
            .start()
            .context("Failed to start event tailer")?;

        // Subscribe to event tailer events and forward to worker registry
        let mut event_rx = event_tailer.subscribe();
        let worker_registry = self.worker_registry.clone();
        let beads = self.beads.clone();
        let scripts_dir = self.scripts_dir.clone();
        let stuck_detector = self.stuck_detector.clone();

        tokio::spawn(async move {
            while let Ok(event) = event_rx.recv().await {
                match event {
                    TailerEvent::Event(parsed) => {
                        // Update stuck detector with worker lifecycle events (§C1, hoop-ttb.3.25)
                        match &parsed.event {
                            crate::events::NeedleEvent::Dispatch { ts, worker, bead, adapter, .. } => {
                                // Worker started executing a bead
                                if let Ok(started_at) = ts.parse::<chrono::DateTime<chrono::Utc>>() {
                                    stuck_detector.lock().unwrap().on_worker_started(
                                        worker,
                                        bead,
                                        adapter.as_deref(),
                                        started_at,
                                    );
                                }
                            }
                            crate::events::NeedleEvent::Complete { worker, .. }
                            | crate::events::NeedleEvent::Fail { worker, .. }
                            | crate::events::NeedleEvent::Timeout { worker, .. }
                            | crate::events::NeedleEvent::Crash { worker, .. }
                            | crate::events::NeedleEvent::Close { worker, .. }
                            | crate::events::NeedleEvent::Release { worker, .. } => {
                                // Worker completed (terminal events)
                                stuck_detector.lock().unwrap().on_worker_complete(worker);
                            }
                            _ => {
                                // Any other event counts as activity
                                if let Some(bead_event) = BeadEventData::from_event(&parsed.event) {
                                    stuck_detector.lock().unwrap().on_worker_event(&bead_event.worker, false);
                                }
                            }
                        }

                        // Convert to BeadEventData and add to registry
                        if let Some(bead_event) = BeadEventData::from_event(&parsed.event) {
                            let ws_event = crate::ws::BeadEventData {
                                timestamp: bead_event.timestamp.clone(),
                                event_type: bead_event.event_type.clone(),
                                bead_id: bead_event.bead_id.clone(),
                                worker: bead_event.worker.clone(),
                                line_number: Some(parsed.line_number),
                                raw: parsed.raw.clone(),
                                stash_sha: bead_event.stash_sha.clone(),
                            };
                            worker_registry.add_bead_event(ws_event).await;
                        }
                        // Update fleet.db cross-project tables from this event
                        update_fleet_from_event(&parsed.event, &beads);

                        // Emit fleet notifications when bead events occur
                        check_and_emit_notifications(&parsed.event, &beads);

                        // Trigger event-subscribed scripts
                        let bead_id = match &parsed.event {
                            crate::events::NeedleEvent::Claim { bead, .. }
                            | crate::events::NeedleEvent::Dispatch { bead, .. }
                            | crate::events::NeedleEvent::Complete { bead, .. }
                            | crate::events::NeedleEvent::Fail { bead, .. }
                            | crate::events::NeedleEvent::Timeout { bead, .. }
                            | crate::events::NeedleEvent::Crash { bead, .. }
                            | crate::events::NeedleEvent::Close { bead, .. }
                            | crate::events::NeedleEvent::Release { bead, .. }
                            | crate::events::NeedleEvent::Update { bead, .. } => bead.clone(),
                            crate::events::NeedleEvent::Unknown => continue,
                        };

                        // Look up bead info for project/kind filtering
                        let (project, kind) = lookup_bead_info(&bead_id, &beads);

                        // Build EventContext and trigger matching scripts
                        let mut ctx = EventContext::from_event(&parsed.event, &parsed.raw);
                        ctx.project = project;
                        ctx.kind = kind;

                        let results = trigger_matching_scripts(&scripts_dir, &ctx).await;
                        for result in results {
                            if result.attempted && !result.succeeded {
                                warn!(
                                    "Event-triggered script '{}' failed: {}",
                                    result.script_name,
                                    result.error.unwrap_or_default()
                                );
                            }
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
        // Clone beads before await to avoid holding RwLockReadGuard across await
        let beads_clone: Vec<Bead> = self.beads.read().unwrap().clone();
        let workers = self.worker_registry.snapshot().await;

        // Build a map of bead_id -> project name from the beads store
        let bead_to_project: std::collections::HashMap<String, String> = beads_clone
            .iter()
            .filter(|b| !b.project.is_empty())
            .map(|b| (b.id.clone(), b.project.clone()))
            .collect();

        runtimes
            .values()
            .map(|r| {
                let bead_count = count_open_beads_for_workspaces(&beads_clone, &r.workspaces);

                // Count workers for this project: workers executing beads belonging to this project
                let worker_count = workers
                    .iter()
                    .filter(|w| {
                        if let crate::ws::WorkerDisplayState::Executing { bead, .. } = &w.state {
                            bead_to_project.get(bead).map_or(false, |p| p == &r.name)
                        } else {
                            false
                        }
                    })
                    .count();

                ProjectRuntimeStatus {
                    project_name: r.name.clone(),
                    project_path: r.workspaces.first().cloned().unwrap_or_default(),
                    state: r.state.clone(),
                    workspace_count: r.workspaces.len(),
                    bead_count,
                    worker_count,
                }
            })
            .collect()
    }

    /// Reconcile runtimes with the given projects configuration
    pub async fn reconcile(&self, config: &ProjectsConfig) -> Result<()> {
        let mut runtimes = self.runtimes.write().await;

        // Build map of existing projects
        let existing: std::collections::HashSet<String> = runtimes.keys().cloned().collect();

        // Build map of desired projects using canonical paths for joins.
        // Uses canonical_for() which resolves via fs::canonicalize (not just
        // the stored canonical_path field, which may be absent in legacy YAML).
        let mut desired: HashMap<String, Vec<PathBuf>> = HashMap::new();
        for project in &config.registry.projects {
            let name = project.name();
            let paths: Vec<PathBuf> = project
                .workspace_views()
                .iter()
                .map(|v| config.canonical_for(name, &v.path))
                .collect();
            desired.insert(name.to_string(), paths);
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
                    info!(
                        "Restarting runtime for project {} (workspaces changed)",
                        name
                    );
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
        let bead_readers = runtime
            .bead_readers
            .lock()
            .unwrap()
            .drain(..)
            .collect::<Vec<_>>();
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
        let vector_index = self.vector_index.clone();
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
                vector_index,
            )
            .await;

            match result {
                Ok(()) => {
                    info!(
                        "Project runtime shut down gracefully: {}",
                        project_name_clone
                    );
                }
                Err(e) => {
                    error!(
                        "Project runtime failed: {} - error: {}",
                        project_name_clone, e
                    );
                    supervisor_clone
                        .handle_failure(&project_name_clone, &e.to_string())
                        .await;
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
        error_lower.contains("workspace path does not exist")
            || error_lower.contains(".beads directory not found")
            || error_lower.contains("does not exist")
    }

    /// Handle runtime failure with exponential backoff and auto-restart
    async fn handle_failure(&self, project_name: &str, error: &str) {
        // Increment panic metric for all failures
        metrics()
            .hoop_errors_total
            .inc(&["supervisor", "project_panic"]);

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
                    worker_count: 0,
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
                        worker_count: 0,
                    });

                    return;
                }

                // Calculate exponential backoff delay
                let delay_secs = (BASE_RESTART_DELAY_SECS
                    * 2_u64.pow(runtime.consecutive_failures as u32 - 1))
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
                    runtime.consecutive_failures,
                    MAX_CONSECUTIVE_FAILURES,
                    project_name,
                    delay_secs
                );

                // Send status update
                let _ = self.status_tx.send(ProjectRuntimeStatus {
                    project_name: project_name.to_string(),
                    project_path: runtime.workspaces.first().cloned().unwrap_or_default(),
                    state: runtime.state.clone(),
                    workspace_count: runtime.workspaces.len(),
                    bead_count: 0,
                    worker_count: 0,
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
        vector_index: Arc<std::sync::RwLock<crate::vector_index::VectorIndex>>,
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

            let mut bead_reader = BeadReader::new(bead_reader_config).with_context(|| {
                format!("Failed to create bead reader for {}", workspace.display())
            })?;

            // Replay existing beads
            let issues_path = workspace.join(".beads").join("issues.jsonl");
            if issues_path.exists() {
                bead_reader.replay_file().with_context(|| {
                    format!("Failed to replay beads for {}", workspace.display())
                })?;
            }

            bead_reader.start().with_context(|| {
                format!("Failed to start bead reader for {}", workspace.display())
            })?;

            // Subscribe to bead events
            let mut rx = bead_reader.subscribe();
            let workspace_clone = workspace.clone();
            let beads_clone = beads.clone();
            let bead_tx_clone = bead_tx.clone();
            let project_name_clone = project_name.clone();
            let error_tx_clone = error_tx.clone();
            let vector_index_clone = vector_index.clone();

            tokio::spawn(async move {
                while let Ok(event) = rx.recv().await {
                    match event {
                        BeadEvent::BeadsUpdated { beads: new_beads } => {
                            // Tag each bead with the project name and workspace path before merging
                            let new_beads: Vec<Bead> = new_beads
                                .into_iter()
                                .map(|mut b| {
                                    b.project = project_name_clone.clone();
                                    b.workspace = workspace_clone.display().to_string();
                                    b
                                })
                                .collect();

                            // Track which beads were open before this update (for vector index removal)
                            let old_beads: std::collections::HashSet<String> = beads_clone
                                .read()
                                .unwrap()
                                .iter()
                                .filter(|b| {
                                    b.project == project_name_clone
                                        && b.status == crate::BeadStatus::Open
                                })
                                .map(|b| b.id.clone())
                                .collect();

                            // Update shared beads store
                            let mut all_beads = beads_clone.write().unwrap().clone();
                            let workspace_bead_ids: std::collections::HashSet<String> =
                                new_beads.iter().map(|b| b.id.clone()).collect();

                            // Remove old beads from this workspace
                            all_beads.retain(|b| {
                                !workspace_bead_ids.contains(&b.id)
                                    || new_beads.iter().any(|nb| nb.id == b.id)
                            });
                            // Add new beads from this workspace
                            all_beads.extend(new_beads.clone());
                            // Sort by created_at descending
                            all_beads.sort_by_key(|b| std::cmp::Reverse(b.created_at));

                            *beads_clone.write().unwrap() = all_beads.clone();

                            // Update vector index: remove closed beads, add new open beads (hoop-ttb.5.9.1)
                            let mut index = vector_index_clone.write().unwrap();
                            let new_open_bead_ids: std::collections::HashSet<String> = new_beads
                                .iter()
                                .filter(|b| b.status == crate::BeadStatus::Open)
                                .map(|b| b.id.clone())
                                .collect();

                            // Remove beads that are no longer open
                            for bead_id in old_beads.difference(&new_open_bead_ids) {
                                let _ = index.remove_from_db(bead_id);
                            }

                            // Add new open beads to vector index
                            for bead in &new_beads {
                                if bead.status == crate::BeadStatus::Open
                                    && !old_beads.contains(&bead.id)
                                {
                                    let item = crate::embedding::IndexedItem {
                                        id: bead.id.clone(),
                                        project: bead.project.clone(),
                                        title: bead.title.clone(),
                                        kind: format!("{:?}", bead.issue_type).to_lowercase(),
                                        description: bead.description.clone(),
                                    };
                                    let _ = index.add_to_db(item);
                                }
                            }

                            // Forward to broadcast
                            let _ =
                                bead_tx_clone.send(BeadEvent::BeadsUpdated { beads: new_beads });

                            debug!("Beads updated for workspace: {}", workspace_clone.display());
                        }
                        BeadEvent::Error(e) => {
                            error!("Bead reader error for {}: {}", workspace_clone.display(), e);
                            // Send error to runtime via channel
                            let _ = error_tx_clone
                                .send(anyhow::anyhow!(
                                    "Bead reader error for {}: {}",
                                    workspace_clone.display(),
                                    e
                                ))
                                .await;
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

        let mut session_tailer =
            SessionTailer::new(session_tailer_config).context("Failed to create session tailer")?;

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
                        error!(
                            "Session tailer error for project {}: {}",
                            project_name_for_tailer, e
                        );
                        // Send error to runtime via channel
                        let _ = error_tx_clone
                            .send(anyhow::anyhow!(
                                "Session tailer error for {}: {}",
                                project_name_for_tailer,
                                e
                            ))
                            .await;
                    }
                    SessionEvent::TagJoinBound { .. } => {}
                }
            }
        });

        session_tailer
            .start()
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

/// Look up the project name for a given bead ID from the shared beads store.
fn lookup_project_for_bead(
    bead_id: &str,
    beads: &Arc<std::sync::RwLock<Vec<Bead>>>,
) -> Option<String> {
    let guard = beads.read().unwrap();
    guard
        .iter()
        .find(|b| b.id == bead_id)
        .map(|b| b.project.clone())
}

/// Look up project and kind for a given bead ID from the shared beads store.
fn lookup_bead_info(
    bead_id: &str,
    beads: &Arc<std::sync::RwLock<Vec<Bead>>>,
) -> (Option<String>, Option<String>) {
    let guard = beads.read().unwrap();
    if let Some(bead) = guard.iter().find(|b| b.id == bead_id) {
        let kind = format!("{:?}", bead.issue_type).to_lowercase();
        (Some(bead.project.clone()), Some(kind))
    } else {
        (None, None)
    }
}

/// Update fleet.db cross-project tables on receipt of a NEEDLE event.
///
/// - `Claim`  → upsert collision_index entry + touch project last_event_at
/// - terminal events (Complete/Close/Release/Fail/Timeout/Crash)
///            → remove collision_index entry + touch project last_event_at
/// - other events → touch project last_event_at only
fn update_fleet_from_event(
    event: &crate::events::NeedleEvent,
    beads: &Arc<std::sync::RwLock<Vec<Bead>>>,
) {
    use crate::events::NeedleEvent;
    use crate::fleet;

    // Extract (ts, worker, bead_id) from the event — all known variants carry these.
    let (ts, worker, bead_id) = match event {
        NeedleEvent::Claim {
            ts, worker, bead, ..
        }
        | NeedleEvent::Dispatch {
            ts, worker, bead, ..
        }
        | NeedleEvent::Complete {
            ts, worker, bead, ..
        }
        | NeedleEvent::Fail {
            ts, worker, bead, ..
        }
        | NeedleEvent::Timeout {
            ts, worker, bead, ..
        }
        | NeedleEvent::Crash {
            ts, worker, bead, ..
        }
        | NeedleEvent::Close {
            ts, worker, bead, ..
        }
        | NeedleEvent::Release {
            ts, worker, bead, ..
        }
        | NeedleEvent::Update {
            ts, worker, bead, ..
        } => (ts.as_str(), worker.as_str(), bead.as_str()),
        NeedleEvent::Unknown => return,
    };

    // Resolve project from in-memory bead store
    let project = lookup_project_for_bead(bead_id, beads);

    match event {
        NeedleEvent::Claim { .. } => {
            // Register in collision index so concurrent-work detection can fire
            if let Some(ref proj) = project {
                let now = chrono::Utc::now().to_rfc3339();
                let entry = fleet::CollisionIndexEntry {
                    bead_id: bead_id.to_string(),
                    project: proj.clone(),
                    worker: Some(worker.to_string()),
                    claimed_at: ts.to_string(),
                    file_paths: vec![],
                    updated_at: now,
                };
                if let Err(e) = fleet::upsert_collision_entry(&entry) {
                    warn!(
                        "fleet: upsert_collision_entry failed for {}: {}",
                        bead_id, e
                    );
                }
            }
        }
        NeedleEvent::Complete { .. }
        | NeedleEvent::Close { .. }
        | NeedleEvent::Release { .. }
        | NeedleEvent::Fail { .. }
        | NeedleEvent::Timeout { .. }
        | NeedleEvent::Crash { .. } => {
            // Free the collision index entry — bead is no longer active
            if let Err(e) = fleet::remove_collision_entry(bead_id) {
                warn!(
                    "fleet: remove_collision_entry failed for {}: {}",
                    bead_id, e
                );
            }
        }
        _ => {}
    }

    // Advance last_event_at for the project (best-effort; warns on failure)
    if let Some(ref proj) = project {
        if let Err(e) = fleet::touch_project_event_at(proj, ts) {
            warn!("fleet: touch_project_event_at failed for {}: {}", proj, e);
        }
    }
}

/// Check and emit fleet notifications when bead events occur.
///
/// This function is called for each event from events.jsonl to determine
/// if a fleet notification should be emitted to the agent's notification ring.
/// Notifications are emitted for:
/// - StitchBeadsClosed: when all beads linked to a stitch are closed
/// - ConvoyComplete: when all workers for a stitch have terminal events
fn check_and_emit_notifications(
    event: &crate::events::NeedleEvent,
    beads: &Arc<std::sync::RwLock<Vec<Bead>>>,
) {
    use crate::events::NeedleEvent;

    // Extract bead_id from the event
    let bead_id = match event {
        NeedleEvent::Claim { bead, .. }
        | NeedleEvent::Dispatch { bead, .. }
        | NeedleEvent::Complete { bead, .. }
        | NeedleEvent::Fail { bead, .. }
        | NeedleEvent::Timeout { bead, .. }
        | NeedleEvent::Crash { bead, .. }
        | NeedleEvent::Close { bead, .. }
        | NeedleEvent::Release { bead, .. }
        | NeedleEvent::Update { bead, .. } => bead.clone(),
        NeedleEvent::Unknown => return,
    };

    // Look up the stitch for this bead
    let stitch_id = match lookup_stitch_for_bead(&bead_id) {
        Some(sid) => sid,
        None => return, // Not linked to any stitch
    };

    // Check for StitchBeadsClosed when a bead is closed
    if matches!(event, NeedleEvent::Close { .. }) {
        let beads_snapshot = beads.read().unwrap().clone();
        if let Err(e) = crate::check_and_emit_stitch_beads_closed(&stitch_id, &beads_snapshot) {
            warn!("Failed to check/emit StitchBeadsClosed notification: {}", e);
        }
    }

    // Check for ConvoyComplete on any terminal event
    if matches!(
        event,
        NeedleEvent::Complete { .. }
            | NeedleEvent::Close { .. }
            | NeedleEvent::Fail { .. }
            | NeedleEvent::Timeout { .. }
            | NeedleEvent::Crash { .. }
            | NeedleEvent::Release { .. }
    ) {
        if let Err(e) = crate::check_and_emit_convoy_complete(&stitch_id) {
            warn!("Failed to check/emit ConvoyComplete notification: {}", e);
        }
    }
}

/// Look up the stitch ID for a given bead ID from fleet.db.
///
/// Returns None if the bead is not linked to any stitch.
fn lookup_stitch_for_bead(bead_id: &str) -> Option<String> {
    use rusqlite::Connection;

    let db_path = std::path::PathBuf::from(
        dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from(".")),
    )
    .join(".hoop")
    .join("fleet.db");

    if !db_path.exists() {
        return None;
    }

    let conn = Connection::open(&db_path).ok()?;

    conn.query_row(
        "SELECT stitch_id FROM stitch_beads WHERE bead_id = ?1 LIMIT 1",
        rusqlite::params![bead_id],
        |row| row.get(0),
    )
    .ok()
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

/// Count open beads for a given set of workspace paths.
///
/// Beads are tagged with their workspace path at load time (in run_project_runtime),
/// so we can properly filter beads per-workspace for multi-workspace projects.
fn count_open_beads_for_workspaces(beads: &[Bead], workspaces: &[PathBuf]) -> usize {
    use std::path::Path;
    beads
        .iter()
        .filter(|b| {
            matches!(b.status, crate::BeadStatus::Open)
                && (b.workspace.is_empty()
                    || workspaces.iter().any(|ws| {
                        // Match by display string (handles both canonical and non-canonical paths)
                        ws.display().to_string() == b.workspace
                            || Path::new(&b.workspace) == ws
                    }))
        })
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
        assert!(ProjectSupervisor::is_permanent_error(
            "Workspace path does not exist: /path"
        ));
        assert!(ProjectSupervisor::is_permanent_error(
            ".beads directory not found at: /path"
        ));
        assert!(!ProjectSupervisor::is_permanent_error("Connection refused"));
        assert!(!ProjectSupervisor::is_permanent_error("Timeout"));
    }
}
