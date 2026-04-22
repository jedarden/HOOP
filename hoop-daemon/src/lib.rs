//! HOOP daemon - the long-lived server process
//!
//! The daemon hosts the web UI, WebSocket endpoints, and REST API.
//! It reads from projects, beads, sessions, files, events, and heartbeats.
//! Its only write is `br create` for bead creation.

pub mod audit;
pub mod beads;
pub mod br_verbs;
pub mod events;
pub mod fleet;
pub mod heartbeats;
pub mod metrics;
pub mod projects;
pub mod sessions;
pub mod shutdown;
pub mod supervisor;
pub mod ws;

/// Worker execution state from heartbeats
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum WorkerState {
    /// Worker is executing a bead
    Executing {
        bead: String,
        pid: u32,
        adapter: String,
    },
    /// Worker is idle
    Idle {
        last_strand: Option<String>,
    },
    /// Worker is in a knot state
    Knot {
        reason: String,
    },
}

/// Bead representation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Bead {
    pub id: String,
    pub title: String,
    pub status: BeadStatus,
    pub priority: i64,
    pub issue_type: BeadType,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_by: String,
    pub dependencies: Vec<String>,
}

/// Bead status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum BeadStatus {
    Open,
    Closed,
}

/// Bead type/issue type
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum BeadType {
    Task,
    Bug,
    Epic,
    Genesis,
    Review,
    Fix,
}

/// Control request over Unix socket
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ControlRequest {
    Status { project: Option<String> },
}

/// Control response over Unix socket
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ControlResponse {
    Status(StatusResponse),
    Error { message: String },
}

/// Project status for CLI display
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectStatus {
    pub name: String,
    pub path: String,
    pub active_beads: usize,
    pub workers: usize,
    pub runtime_state: Option<String>,
    pub runtime_error: Option<String>,
}

/// Status response for CLI
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StatusResponse {
    pub daemon_running: bool,
    pub uptime_secs: u64,
    pub projects: Vec<ProjectStatus>,
}

use axum::{
    routing::get,
    Json, Router,
};
use hoop_schema::HealthResponse;
use hoop_ui::AssetsHandler;
use shutdown::{DbCheckpointHandle, ShutdownCoordinator, SocketCleanupHandle};
use std::sync::Arc;
use std::{
    fs,
    net::SocketAddr,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt},
    net::UnixListener,
    signal,
    sync::broadcast,
    time::Instant,
};
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn, Level};

/// Daemon configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub control_socket_path: PathBuf,
    /// Allow br version mismatch (dev override for --allow-br-mismatch)
    pub allow_br_mismatch: bool,
}

impl Default for Config {
    fn default() -> Self {
        let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.push(".hoop");
        Self {
            bind_addr: SocketAddr::from(([127, 0, 0, 1], 3000)),
            control_socket_path: home.join("control.sock"),
            allow_br_mismatch: false,
        }
    }
}

/// Daemon state shared across all request handlers
#[derive(Debug, Clone)]
pub struct DaemonState {
    pub config: Config,
    pub started_at: Instant,
    pub worker_registry: Arc<ws::WorkerRegistry>,
    pub beads: Arc<std::sync::RwLock<Vec<Bead>>>,
    pub bead_tx: broadcast::Sender<ws::BeadData>,
    pub shutdown: Arc<ShutdownCoordinator>,
    pub supervisor: Arc<supervisor::ProjectSupervisor>,
    pub projects: Arc<std::sync::RwLock<Vec<ws::ProjectCardData>>>,
    pub config_status_tx: broadcast::Sender<ws::ConfigStatusData>,
}

/// Health check endpoint handler
async fn healthz() -> axum::Json<HealthResponse> {
    axum::Json(HealthResponse::ok())
}

/// Get all beads endpoint handler
async fn get_beads(state: axum::extract::State<DaemonState>) -> Json<Vec<Bead>> {
    let beads = state.beads.read().unwrap();
    Json(beads.clone())
}

/// Build the daemon router with all endpoints
pub fn router() -> Router<DaemonState> {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/api/beads", get(get_beads))
        .route("/ws", get(ws::ws_handler))
        .nest_service("/assets", AssetsHandler::router())
        .fallback_service(AssetsHandler::router())
        .layer(TraceLayer::new_for_http())
}

/// Handle a single control socket connection
async fn handle_control_socket(
    socket: tokio::net::UnixStream,
    state: DaemonState,
) -> anyhow::Result<()> {
    let (reader, writer) = tokio::io::split(socket);
    let mut reader = tokio::io::BufReader::new(reader);
    let mut writer = tokio::io::BufWriter::new(writer);

    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break;
        }

        let response = match serde_json::from_str::<ControlRequest>(&line.trim()) {
            Ok(ControlRequest::Status { project }) => {
                let runtimes = state.supervisor.snapshot().await;
                let projects = if let Some(proj) = project {
                    runtimes
                        .into_iter()
                        .filter(|r| r.project_name == proj)
                        .map(|r| ProjectStatus {
                            name: r.project_name,
                            path: r.project_path.display().to_string(),
                            active_beads: r.bead_count,
                            workers: 0, // TODO: track workers per project
                            runtime_state: Some(format!("{:?}", r.state)),
                            runtime_error: r.state.error().map(|e| e.to_string()),
                        })
                        .collect()
                } else {
                    runtimes
                        .into_iter()
                        .map(|r| ProjectStatus {
                            name: r.project_name,
                            path: r.project_path.display().to_string(),
                            active_beads: r.bead_count,
                            workers: 0,
                            runtime_state: Some(format!("{:?}", r.state)),
                            runtime_error: r.state.error().map(|e| e.to_string()),
                        })
                        .collect()
                };
                let status = StatusResponse {
                    daemon_running: true,
                    uptime_secs: state.started_at.elapsed().as_secs(),
                    projects,
                };
                ControlResponse::Status(status)
            }
            Err(e) => ControlResponse::Error {
                message: format!("Invalid request: {}", e),
            },
        };

        let response_json = serde_json::to_string(&response)?;
        let line = format!("{}\n", response_json);
        writer.write_all(line.as_bytes()).await?;
        writer.flush().await?;
    }

    Ok(())
}

/// Run the control socket server
async fn run_control_socket(
    state: DaemonState,
    mut shutdown: broadcast::Receiver<()>,
) -> anyhow::Result<()> {
    let socket_path = &state.config.control_socket_path;

    if let Some(parent) = socket_path.parent() {
        fs::create_dir_all(parent)?;
    }

    if socket_path.exists() {
        fs::remove_file(socket_path)?;
    }

    let listener = UnixListener::bind(socket_path)?;
    fs::set_permissions(socket_path, fs::Permissions::from_mode(0o600))?;

    info!("Control socket listening at {}", socket_path.display());

    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((socket, _addr)) => {
                        let state = state.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_control_socket(socket, state).await {
                                error!("Control socket handler error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Control socket accept error: {}", e);
                    }
                }
            }
            _ = shutdown.recv() => {
                info!("Control socket shutting down");
                drop(listener);
                if socket_path.exists() {
                    let _ = fs::remove_file(socket_path);
                }
                break;
            }
        }
    }

    Ok(())
}

/// Start the daemon server
///
/// This function blocks until the server is shut down.
pub async fn serve(config: Config) -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    // Run startup audit - refuse to start on critical failures
    info!("Running startup audit...");
    let mut audit_config = audit::AuditConfig::default();
    audit_config.allow_br_mismatch = config.allow_br_mismatch;
    if let Err(e) = audit::daemon_startup_check(&audit_config) {
        error!("{}", e);
        return Err(e);
    }
    info!("Startup audit passed");

    // Validate zero-write invariant at startup
    br_verbs::validate_zero_write_invariant();

    // Initialize fleet.db
    info!("Initializing fleet.db...");
    if let Err(e) = fleet::init_fleet_db() {
        error!("Failed to initialize fleet.db: {}", e);
        return Err(e);
    }
    info!("fleet.db initialized");

    // Initialize heartbeat monitor
    let mut heartbeat_monitor = heartbeats::HeartbeatMonitor::new(
        heartbeats::HeartbeatMonitorConfig::default()
    )?;
    heartbeat_monitor.start()?;
    let heartbeat_tx = heartbeat_monitor.subscribe();

    // Initialize session event broadcast channel
    let (session_tx, _) = broadcast::channel::<sessions::SessionEvent>(256);

    // Initialize worker registry
    let worker_registry = Arc::new(ws::WorkerRegistry::new(heartbeat_tx, session_tx.clone()));

    // Initialize bead event broadcast channel
    let (bead_tx, _) = broadcast::channel::<ws::BeadData>(256);

    // Initialize config status broadcast channel
    let (config_status_tx, _) = broadcast::channel::<ws::ConfigStatusData>(64);

    // Initialize shared beads store
    let beads: Arc<std::sync::RwLock<Vec<Bead>>> = Arc::new(std::sync::RwLock::new(Vec::new()));

    // Initialize shutdown coordinator (needed for supervisor)
    let shutdown_coordinator = Arc::new(ShutdownCoordinator::new());
    let mut shutdown_rx = shutdown_coordinator.subscribe();

    // Initialize project supervisor
    info!("Initializing project supervisor...");
    let supervisor = Arc::new(supervisor::ProjectSupervisor::new(
        broadcast::channel(256).0, // bead events (internal)
        session_tx.clone(),
        worker_registry.clone(),
        beads.clone(),
        shutdown_coordinator.clone(),
    ));

    // Start projects watcher
    let mut projects_watcher = projects::ProjectsWatcher::new()?;
    projects_watcher.start()?;

    // Initial reconcile with current projects config
    let initial_config = projects_watcher.config().await;
    if let Err(e) = supervisor.reconcile(&initial_config).await {
        warn!("Initial project reconcile failed: {}", e);
    }

    // Spawn task to handle projects config changes
    let supervisor_for_reconcile = supervisor.clone();
    let config_tx_for_reload = config_status_tx.clone();
    tokio::spawn(async move {
        let mut rx = projects_watcher.subscribe();
        while let Ok(event) = rx.recv().await {
            match event {
                projects::ProjectsEvent::ConfigReloaded { config } => {
                    info!("Projects configuration reloaded, reconciling runtimes");
                    if let Err(e) = supervisor_for_reconcile.reconcile(&config).await {
                        error!("Failed to reconcile runtimes: {}", e);
                    }
                    // Broadcast valid config status
                    let _ = config_tx_for_reload.send(ws::ConfigStatusData {
                        valid: true,
                        error: None,
                    });
                }
                projects::ProjectsEvent::ConfigError { error } => {
                    warn!("Projects configuration error: {}", error.message);
                    // Broadcast config error
                    let _ = config_tx_for_reload.send(ws::ConfigStatusData {
                        valid: false,
                        error: Some(ws::ConfigErrorData {
                            message: error.message.clone(),
                            line: error.line,
                            col: error.col,
                        }),
                    });
                }
            }
        }
    });

    // Forward bead events from supervisor to WebSocket broadcast
    let bead_tx_clone = bead_tx.clone();
    tokio::spawn(async move {
        let mut rx = supervisor.subscribe_status();
        while let Ok(status) = rx.recv().await {
            // Runtime status updates - could be used for UI project cards
            debug!("Project runtime status update: {} - {:?}", status.project_name, status.state);
        }
    });

    // Forward session events from supervisor to worker registry
    let registry_for_sessions = worker_registry.clone();
    tokio::spawn(async move {
        let mut rx = session_tx.subscribe();
        while let Ok(event) = rx.recv().await {
            match event {
                sessions::SessionEvent::ConversationsUpdated { sessions } => {
                    registry_for_sessions.update_conversations(sessions).await;
                }
                sessions::SessionEvent::SessionBound { .. } => {
                    // Registry will handle this via the WebSocket
                }
                sessions::SessionEvent::Error(e) => {
                    error!("Session tailer error: {}", e);
                }
            }
        }
    });

    // Spawn task to process heartbeat events and update registry
    let registry_clone = worker_registry.clone();
    tokio::spawn(async move {
        use heartbeats::MonitorEvent;
        let mut rx = registry_clone.subscribe();
        while let Ok(event) = rx.recv().await {
            match event {
                MonitorEvent::Heartbeat(hb) => {
                    let liveness = registry_clone
                        .snapshot()
                        .await
                        .iter()
                        .find(|w| w.worker == hb.worker)
                        .map(|w| w.liveness)
                        .unwrap_or(heartbeats::WorkerLiveness::Dead);
                    registry_clone.update_worker(hb, liveness).await;
                }
                MonitorEvent::LivenessChange(t) => {
                    // Update worker liveness
                    let snapshot = registry_clone.snapshot().await;
                    if let Some(w) = snapshot.iter().find(|w| w.worker == t.worker) {
                        registry_clone.update_worker(
                            heartbeats::WorkerHeartbeat {
                                ts: w.last_heartbeat,
                                worker: w.worker.clone(),
                                state: match &w.state {
                                    ws::WorkerDisplayState::Executing { bead, adapter, .. } => {
                                        hoop_schema::WorkerState::Executing {
                                            bead: bead.clone(),
                                            pid: 0,
                                            adapter: adapter.clone(),
                                        }
                                    }
                                    ws::WorkerDisplayState::Idle { last_strand } => {
                                        hoop_schema::WorkerState::Idle {
                                            last_strand: last_strand.clone(),
                                        }
                                    }
                                    ws::WorkerDisplayState::Knot { reason } => {
                                        hoop_schema::WorkerState::Knot {
                                            reason: reason.clone(),
                                        }
                                    }
                                },
                            },
                            t.new_state,
                        ).await;
                    }
                }
                _ => {}
            }
        }
    });

    let state = DaemonState {
        config: config.clone(),
        started_at: Instant::now(),
        worker_registry,
        beads,
        bead_tx,
        shutdown: shutdown_coordinator.clone(),
        supervisor,
        projects: Arc::new(std::sync::RwLock::new(Vec::new())),
        config_status_tx,
    };

    let app = router().with_state(state.clone());

    info!("HOOP daemon listening on {}", config.bind_addr);

    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    let tcp_server = axum::serve(listener, app);

    // Spawn a task to broadcast shutdown to the simple broadcast channel
    // (for compatibility with existing control socket logic)
    let (shutdown_tx, _) = broadcast::channel::<()>(1);
    let shutdown_tx_for_shutdown = shutdown_tx.clone();
    let shutdown_coordinator_clone = shutdown_coordinator.clone();

    tokio::spawn(async move {
        // Wait for any shutdown phase to start the graceful shutdown
        let mut rx = shutdown_coordinator_clone.subscribe();
        while let Ok(phase) = rx.recv().await {
            if matches!(phase, shutdown::ShutdownPhase::Initiated) {
                warn!("Shutdown initiated, notifying all components");
                let _ = shutdown_tx_for_shutdown.send(());
                break;
            }
        }
    });

    let control_state = state.clone();
    let control_shutdown = shutdown_tx.subscribe();
    let control_socket_task = tokio::spawn(async move {
        run_control_socket(control_state, control_shutdown).await
    });

    // Set up signal handling
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>;

    // The graceful shutdown future
    let graceful_shutdown = async {
        tokio::select! {
            _ = ctrl_c => {
                info!("Received SIGINT (Ctrl-C), initiating graceful shutdown");
            }
            _ = terminate => {
                info!("Received SIGTERM, initiating graceful shutdown");
            }
        }
        shutdown_coordinator.shutdown(None).await;
    };

    // Run the server with graceful shutdown
    let result = tokio::select! {
        r = tcp_server.with_graceful_shutdown(async {
            // Wait for shutdown to be initiated
            while let Ok(phase) = shutdown_rx.recv().await {
                if matches!(phase, shutdown::ShutdownPhase::CloseNewConnections) {
                    info!("Closing new connections per shutdown phase");
                    break;
                }
            }
        }) => {
            r.map_err(|e| anyhow::anyhow!(e))
        }
        r = control_socket_task => match r {
            Ok(Ok(())) => Ok::<(), anyhow::Error>(()),
            Ok(Err(e)) => Err(e),
            Err(e) => Err(anyhow::anyhow!("Control socket task join failed: {}", e)),
        },
        _ = graceful_shutdown => Ok::<(), anyhow::Error>(()),
    };

    // Perform final cleanup after all tasks have stopped
    info!("All tasks stopped, performing final cleanup");

    // Checkpoint fleet.db WAL
    let db_checkpoint = DbCheckpointHandle::new(fleet::db_path());
    if let Err(e) = db_checkpoint.checkpoint() {
        warn!("Failed to checkpoint fleet.db: {}", e);
    }

    // Clean up control socket
    let socket_cleanup = SocketCleanupHandle::new(config.control_socket_path.clone());
    if let Err(e) = socket_cleanup.cleanup() {
        warn!("Failed to cleanup socket: {}", e);
    }

    result?;
    info!("HOOP daemon shut down gracefully");

    Ok(())
}
