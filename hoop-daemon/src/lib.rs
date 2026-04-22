//! HOOP daemon - the long-lived server process
//!
//! The daemon hosts the web UI, WebSocket endpoints, and REST API.
//! It reads from projects, beads, sessions, files, events, and heartbeats.
//! Its only write is `br create` for bead creation.

pub mod audit;

use axum::{
    routing::get,
    Router,
};
use hoop_schema::{ControlRequest, ControlResponse, HealthResponse, ProjectStatus, StatusResponse};
use hoop_ui::AssetsHandler;
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
}

/// Health check endpoint handler
async fn healthz() -> axum::Json<HealthResponse> {
    axum::Json(HealthResponse::ok())
}

/// Build the daemon router with all endpoints
pub fn router() -> Router<DaemonState> {
    Router::new()
        .route("/healthz", get(healthz))
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

    let state = DaemonState {
        config: config.clone(),
        started_at: Instant::now(),
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
