//! HOOP daemon - the long-lived server process
//!
//! The daemon hosts the web UI, WebSocket endpoints, and REST API.
//! It reads from projects, beads, sessions, files, events, and heartbeats.
//! Its only write is `br create` for bead creation.

pub mod audit;
pub mod beads;
pub mod events;
pub mod heartbeats;
pub mod metrics;
pub mod sessions;
pub mod ws;

use axum::{
    routing::get,
    Json, Router,
};
use hoop_schema::{Bead, ControlRequest, ControlResponse, HealthResponse, ProjectStatus, StatusResponse};
use hoop_ui::AssetsHandler;
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
use tracing::{error, info, Level};

/// Daemon configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub control_socket_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.push(".hoop");
        Self {
            bind_addr: SocketAddr::from(([127, 0, 0, 1], 3000)),
            control_socket_path: home.join("control.sock"),
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
                let status = StatusResponse {
                    daemon_running: true,
                    uptime_secs: state.started_at.elapsed().as_secs(),
                    projects: if let Some(proj) = project {
                        vec![ProjectStatus {
                            name: proj.clone(),
                            path: format!("/home/coding/{}", proj),
                            active_beads: 0,
                            workers: 0,
                        }]
                    } else {
                        vec![]
                    },
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
    let audit_config = audit::AuditConfig::default();
    if let Err(e) = audit::daemon_startup_check(&audit_config) {
        error!("{}", e);
        return Err(e);
    }
    info!("Startup audit passed");

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

    // Initialize session tailer for current workspace
    let mut session_tailer = sessions::SessionTailer::new(sessions::SessionTailerConfig {
        claude_projects_dir: dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".claude")
            .join("projects"),
        project_path: Some(PathBuf::from(".")), // Current workspace
        discovery_concurrency: 16,
        poll_interval_secs: 5,
    })?;
    session_tailer.start()?;

    // Spawn task to forward session events to the worker registry
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

    // Initialize bead reader for current workspace
    let mut bead_reader = beads::BeadReader::new(beads::BeadReaderConfig {
        workspace_path: PathBuf::from("."),
    })?;
    bead_reader.start()?;
    let beads: Arc<std::sync::RwLock<Vec<Bead>>> = Arc::new(std::sync::RwLock::new(Vec::new()));

    // Spawn task to handle bead events
    let beads_clone = beads.clone();
    let bead_tx_clone = bead_tx.clone();
    tokio::spawn(async move {
        let mut rx = bead_reader.subscribe();
        while let Ok(event) = rx.recv().await {
            match event {
                beads::BeadEvent::BeadsUpdated { beads: new_beads } => {
                    *beads_clone.write().unwrap() = new_beads.clone();
                    // Broadcast bead updates
                    for bead in &new_beads {
                        let bead_data = ws::BeadData {
                            id: bead.id.clone(),
                            title: bead.title.clone(),
                            status: format!("{:?}", bead.status).to_lowercase(),
                            priority: bead.priority,
                            issue_type: format!("{:?}", bead.issue_type).to_lowercase(),
                            created_at: bead.created_at.to_rfc3339(),
                            updated_at: bead.updated_at.to_rfc3339(),
                            created_by: bead.created_by.clone(),
                            dependencies: bead.dependencies.clone(),
                        };
                        let _ = bead_tx_clone.send(bead_data);
                    }
                }
                beads::BeadEvent::Error(e) => {
                    error!("Bead reader error: {}", e);
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
    };

    let app = router().with_state(state.clone());

    info!("HOOP daemon listening on {}", config.bind_addr);

    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    let tcp_server = axum::serve(listener, app);

    let (shutdown_tx, _) = broadcast::channel::<()>(1);

    let control_state = state.clone();
    let control_shutdown = shutdown_tx.subscribe();
    let control_socket_task = tokio::spawn(async move {
        run_control_socket(control_state, control_shutdown).await
    });

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
    let terminate = std::future::pending::<()>();

    let shutdown = async move {
        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }
        let _ = shutdown_tx.send(());
    };

    let result = tokio::select! {
        r = tcp_server.with_graceful_shutdown(shutdown) => {
            r.map_err(|e| anyhow::anyhow!(e))
        }
        r = control_socket_task => match r {
            Ok(Ok(())) => Ok::<(), anyhow::Error>(()),
            Ok(Err(e)) => Err(e),
            Err(e) => Err(anyhow::anyhow!("Control socket task join failed: {}", e)),
        },
    };

    result?;
    info!("HOOP daemon shut down gracefully");

    Ok(())
}
