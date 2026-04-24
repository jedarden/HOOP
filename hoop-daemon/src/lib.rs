//! HOOP daemon - the long-lived server process
//!
//! The daemon hosts the web UI, WebSocket endpoints, and REST API.
//! It reads from projects, beads, sessions, files, events, and heartbeats.
//! Its only write is `br create` for bead creation.

pub mod adb_dictate;
pub mod agent_adapter;
pub mod agent_context;
pub mod agent_session;
pub mod api_agent;
pub mod attachment_sync;
pub mod backup;
pub mod backup_pipeline;
pub mod config_resolver;
pub mod api_attachments;
pub mod api_audit;
pub mod api_beads;
pub mod api_dictated_notes;
pub mod api_draft_queue;
pub mod api_metrics;
pub mod api_preview;
pub mod api_stitch_decompose;
pub mod api_stitch_read;
pub mod api_transcription;
pub mod api_uploads;
pub mod attachments;
pub mod audit;
pub mod beads;
pub mod br_verbs;
pub mod capacity;
pub mod cost;
pub mod dictated_notes;
pub mod events;
pub mod files;
pub mod fleet;
pub mod heartbeats;
pub mod id_validators;
pub mod log_rotation;
pub mod metrics;
pub mod projects;
pub mod sessions;
pub mod shutdown;
pub mod stitch_status;
pub mod stitch_decompose;
pub mod supervisor;
pub mod tag_join;
pub mod transcription;
pub mod uploads;
pub mod ws;
pub mod similarity;
pub mod snapshot_manifest;
pub mod svg_sanitize;
pub mod pdf_sanitize;
pub mod predictor;
pub mod risk_patterns;
pub mod embedding;
pub mod vector_index;
pub mod morning_brief;
pub mod api_morning_brief;
pub mod redaction;

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
    #[serde(default)]
    pub description: Option<String>,
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
    routing::{get, post},
    Json, Router,
};
use hoop_schema::{HealthResponse, ReadinessResponse, DegradedProject};
use hoop_ui::AssetsHandler;
use shutdown::{DbCheckpointHandle, ShutdownCoordinator, SocketCleanupHandle};
use std::sync::Arc;
use std::{
    fs,
    net::SocketAddr,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt},
    net::UnixListener,
    signal,
    sync::broadcast,
    time::Instant,
};
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn};

pub use config_resolver::{CliOverrides, ResolvedConfig, ConfigSource};

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

/// Per-project display metadata from projects.yaml
#[derive(Debug, Clone)]
pub struct ProjectMetadata {
    label: String,
    color: String,
}

/// Daemon state shared across all request handlers
#[derive(Debug, Clone)]
pub struct DaemonState {
    pub config: Config,
    pub started_at: Instant,
    pub worker_registry: Arc<ws::WorkerRegistry>,
    pub beads: Arc<std::sync::RwLock<Vec<Bead>>>,
    pub bead_tx: broadcast::Sender<ws::BeadData>,
    /// Broadcast channel for stitch_created events sent to WS clients
    pub stitch_tx: broadcast::Sender<ws::StitchCreatedData>,
    pub shutdown: Arc<ShutdownCoordinator>,
    pub supervisor: Arc<supervisor::ProjectSupervisor>,
    pub projects: Arc<std::sync::RwLock<Vec<ws::ProjectCardData>>>,
    pub project_metadata: Arc<std::sync::RwLock<std::collections::HashMap<String, ProjectMetadata>>>,
    pub config_status_tx: broadcast::Sender<ws::ConfigStatusData>,
    pub project_status_tx: broadcast::Sender<ws::ProjectCardData>,
    pub capacity_tx: broadcast::Sender<Vec<capacity::AccountCapacity>>,
    pub cost_aggregator: Arc<std::sync::RwLock<cost::CostAggregator>>,
    pub transcription_service: Option<Arc<transcription::TranscriptionService>>,
    pub upload_registry: Arc<uploads::UploadRegistry>,
    /// The project currently focused in the UI, used by the ADB dictation endpoint
    /// to associate notes without requiring an explicit ?project= query parameter.
    pub active_project: Arc<std::sync::RwLock<Option<String>>>,
    /// Vector index for semantic pre-dedup at draft time
    pub vector_index: Arc<std::sync::RwLock<vector_index::VectorIndex>>,
    /// Agent session manager — wraps the config-driven agent adapter with
    /// lifecycle persistence (fleet.db), WS event broadcasting, and cost tracking.
    pub agent_session_manager: Option<Arc<agent_session::AgentSessionManager>>,
    /// Morning brief runner — orchestrates scheduled + on-demand brief generation
    pub morning_brief_runner: Option<Arc<morning_brief::MorningBriefRunner>>,
    /// Broadcast channel for morning brief events sent to WS clients
    pub brief_tx: broadcast::Sender<ws::MorningBriefData>,
    /// Broadcast channel for draft queue events sent to WS clients
    pub draft_tx: broadcast::Sender<ws::DraftUpdateData>,
    /// Fully resolved config with per-key attribution (§17.2)
    pub resolved_config: Arc<ResolvedConfig>,
}

/// Health check endpoint handler — returns 200 if the process is responsive.
async fn healthz() -> axum::Json<HealthResponse> {
    axum::Json(HealthResponse::ok())
}

/// Readiness endpoint handler — returns 200 only when all per-project
/// runtimes are healthy (or explicitly marked degraded). Returns 503 with
/// a JSON body naming any degraded projects.
async fn readyz(
    axum::extract::State(state): axum::extract::State<DaemonState>,
) -> Result<axum::Json<ReadinessResponse>, (axum::http::StatusCode, axum::Json<ReadinessResponse>)> {
    if state.shutdown.is_shutting_down() {
        let body = ReadinessResponse::degraded(vec![DegradedProject {
            project: "_daemon".into(),
            state: "shutting_down".into(),
            error: None,
        }]);
        return Err((axum::http::StatusCode::SERVICE_UNAVAILABLE, axum::Json(body)));
    }

    let snapshot = state.supervisor.snapshot().await;
    let degraded: Vec<DegradedProject> = snapshot
        .iter()
        .filter(|s| !matches!(s.state, supervisor::ProjectRuntimeState::Healthy))
        .map(|s| DegradedProject {
            project: s.project_name.clone(),
            state: s.state.to_display_string().to_string(),
            error: s.state.error().map(|e| e.to_string()),
        })
        .collect();

    if degraded.is_empty() {
        Ok(axum::Json(ReadinessResponse::ok()))
    } else {
        Err((axum::http::StatusCode::SERVICE_UNAVAILABLE, axum::Json(ReadinessResponse::degraded(degraded))))
    }
}

/// Get all beads endpoint handler
async fn get_beads(state: axum::extract::State<DaemonState>) -> Json<Vec<Bead>> {
    let beads = state.beads.read().unwrap();
    Json(beads.clone())
}

/// Get cost buckets endpoint handler
async fn get_cost_buckets(
    axum::extract::State(state): axum::extract::State<DaemonState>,
) -> Json<Vec<cost::CostBucket>> {
    let aggregator = state.cost_aggregator.read().unwrap();
    Json(aggregator.get_buckets())
}

/// Get cost buckets by project endpoint handler
async fn get_cost_buckets_by_project(
    axum::extract::Path(project): axum::extract::Path<String>,
    axum::extract::State(state): axum::extract::State<DaemonState>,
) -> Json<Vec<cost::CostBucket>> {
    let aggregator = state.cost_aggregator.read().unwrap();
    Json(aggregator.get_buckets_by_project(&project))
}

/// Reload pricing configuration endpoint handler
async fn reload_pricing(
    axum::extract::State(state): axum::extract::State<DaemonState>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let mut aggregator = state.cost_aggregator.write().unwrap();
    aggregator.reload_pricing()
        .map_err(|e| {
            warn!("Failed to reload pricing configuration: {}", e);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })?;
    info!("Pricing configuration reloaded via API request");
    Ok(Json(serde_json::json!({
        "status": "ok",
        "message": "Pricing configuration reloaded successfully"
    })))
}

/// Query parameters for the file browser endpoint.
#[derive(serde::Deserialize)]
struct FilesQuery {
    /// Relative path from the project root (empty = root).
    path: Option<String>,
}

/// List immediate children of a project directory.
async fn get_project_files(
    axum::extract::Path(project): axum::extract::Path<String>,
    axum::extract::Query(params): axum::extract::Query<FilesQuery>,
    axum::extract::State(state): axum::extract::State<DaemonState>,
) -> Result<Json<Vec<files::FileEntry>>, axum::http::StatusCode> {
    let project_root = {
        let projects = state.projects.read().unwrap();
        projects
            .iter()
            .find(|p| p.name == project)
            .map(|p| std::path::PathBuf::from(&p.path))
            .ok_or(axum::http::StatusCode::NOT_FOUND)?
    };

    let rel_dir = params.path.unwrap_or_default();

    // Guard against path traversal before handing off to the blocking task.
    if !files::is_safe_rel_path(&rel_dir) {
        return Err(axum::http::StatusCode::FORBIDDEN);
    }

    let entries = tokio::task::spawn_blocking(move || files::list_dir(&project_root, &rel_dir))
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(entries))
}

/// Get bead events for a specific bead (from events.jsonl)
async fn get_bead_events(
    axum::extract::Path(bead_id): axum::extract::Path<String>,
    axum::extract::State(state): axum::extract::State<DaemonState>,
) -> Result<Json<Vec<ws::BeadEventData>>, (axum::http::StatusCode, String)> {
    id_validators::validate_bead_id(&bead_id).map_err(id_validators::rejection)?;
    let events = state.worker_registry.get_bead_events(&bead_id).await;
    Ok(Json(events))
}

/// Get per-account capacity utilization (on-demand compute)
async fn get_capacity(
    axum::extract::State(_state): axum::extract::State<DaemonState>,
) -> Json<Vec<capacity::AccountCapacity>> {
    let config = capacity::CapacityMeterConfig::default();
    let meter = capacity::CapacityMeter::new(config);
    Json(meter.compute())
}

/// /debug/state — resolved config with attribution for every key (§17.2).
///
/// Returns the full `ResolvedConfig` where each key carries:
/// - `value`: the effective value
/// - `source`: which layer won (`cli_flag`, `env_var`, `config_yml`, `default`)
/// - `resolved_from`: human-readable attribution string
async fn get_debug_state(
    axum::extract::State(state): axum::extract::State<DaemonState>,
) -> Json<serde_json::Value> {
    let map = state.resolved_config.to_debug_map();
    Json(serde_json::Value::Object(map.into_iter().map(|(k, v)| (k, v)).collect()))
}

/// Build the daemon router with all endpoints
pub fn router() -> Router<DaemonState> {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/api/beads", get(get_beads))
        .route("/api/beads/:bead_id/events", get(get_bead_events))
        .route("/api/capacity", get(get_capacity))
        .route("/api/cost/buckets", get(get_cost_buckets))
        .route("/api/cost/buckets/:project", get(get_cost_buckets_by_project))
        .route("/api/cost/reload-pricing", post(reload_pricing))
        .route("/api/projects/:project/files", get(get_project_files))
        .route("/api/attachments/:attachment_type/:id/:filename", get(api_attachments::serve_attachment))
        .route("/ws", get(ws::ws_handler))
        .route("/debug/state", get(get_debug_state))
        .merge(api_uploads::router())
        .merge(api_dictated_notes::router())
        .merge(api_transcription::router())
        .merge(adb_dictate::router())
        .merge(api_audit::router())
        .merge(api_beads::router())
        .merge(api_draft_queue::router())
        .merge(api_preview::router())
        .merge(api_stitch_decompose::router())
        .merge(api_stitch_read::router())
        .merge(api_agent::router())
        .merge(api_morning_brief::router())
        .merge(api_metrics::router())
        .nest_service("/assets", AssetsHandler::router())
        .fallback_service(AssetsHandler::router())
        .layer(TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn(
            api_metrics::http_metrics_middleware,
        ))
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

        let response = match serde_json::from_str::<ControlRequest>(line.trim()) {
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
                            runtime_state: Some(r.state.to_display_string().to_string()),
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
                            runtime_state: Some(r.state.to_display_string().to_string()),
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
    log_rotation::init_logging();

    // Resolve full config with attribution (§17.2)
    let cli_overrides = CliOverrides {
        bind_addr: {
            let default = SocketAddr::from(([127, 0, 0, 1], 3000));
            if config.bind_addr != default {
                Some(config.bind_addr)
            } else {
                None
            }
        },
        allow_br_mismatch: if config.allow_br_mismatch { Some(true) } else { None },
    };
    let resolved_config = Arc::new(config_resolver::resolve(cli_overrides));
    info!("Config precedence resolver initialized (§17.2)");

    // Install panic hook that records hoop_panics_total metric before aborting.
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let subsystem = info.location()
            .map(|l| l.file())
            .unwrap_or("unknown")
            .rsplit('/')
            .next()
            .unwrap_or("unknown");
        metrics::metrics().hoop_panics_total.inc(&[subsystem]);
        metrics::metrics().hoop_errors_total.inc(&["panic", "panic"]);
        previous_hook(info);
    }));

    // Run startup audit - refuse to start on critical failures
    info!("Running startup audit...");
    let audit_config = audit::AuditConfig { allow_br_mismatch: config.allow_br_mismatch, ..Default::default() };
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
    let heartbeat_tx = heartbeat_monitor.sender();

    // Initialize session event broadcast channel
    let (session_tx, _) = broadcast::channel::<sessions::SessionEvent>(256);

    // Initialize worker registry
    let worker_registry = Arc::new(ws::WorkerRegistry::new(heartbeat_tx, session_tx.clone()));

    // Initialize bead event broadcast channel
    let (bead_tx, _) = broadcast::channel::<ws::BeadData>(256);

    // Initialize stitch_created event broadcast channel
    let (stitch_tx, _) = broadcast::channel::<ws::StitchCreatedData>(256);

    // Initialize config status broadcast channel
    let (config_status_tx, _) = broadcast::channel::<ws::ConfigStatusData>(64);

    // Initialize project status broadcast channel
    let (project_status_tx, _) = broadcast::channel::<ws::ProjectCardData>(64);

    // Initialize capacity broadcast channel and start refresh loop
    let (capacity_tx, _) = broadcast::channel::<Vec<capacity::AccountCapacity>>(64);
    let capacity_meter_config = capacity::CapacityMeterConfig::default();
    let account_count = capacity_meter_config.account_dirs.len();
    capacity::CapacityMeter::spawn_refresh_loop(capacity_meter_config, capacity_tx.clone());
    info!("Capacity meter refresh loop started ({} account(s))", account_count);

    // Initialize cost aggregator
    let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.push(".hoop");
    let pricing_config_path = home.join("pricing.yml");
    let cost_aggregator = cost::CostAggregator::new(pricing_config_path)?;
    let cost_aggregator: Arc<std::sync::RwLock<cost::CostAggregator>> =
        Arc::new(std::sync::RwLock::new(cost_aggregator));
    info!("Cost aggregator initialized");

    // Initialize transcription service
    let voice_config = transcription::load_voice_config();
    let transcription_config = transcription::build_transcription_config(&voice_config);
    let whisper_model_path = transcription_config.whisper_model_path.clone();
    let transcription_service = Arc::new(transcription::TranscriptionService::new(transcription_config.clone()));

    // Ensure models directory exists
    if let Some(model_dir) = whisper_model_path.parent() {
        if !model_dir.exists() {
            if let Err(e) = std::fs::create_dir_all(model_dir) {
                warn!("Failed to create models directory {}: {}", model_dir.display(), e);
            } else {
                info!("Created models directory at {}", model_dir.display());
                info!("Whisper models should be placed at {}", whisper_model_path.display());
                info!("Download from: https://huggingface.co/ggerganov/whisper.cpp");
            }
        }
    }

    // Check if model exists
    if !whisper_model_path.exists() {
        warn!("Whisper model not found at {}", whisper_model_path.display());
        warn!("Transcription will fail until model is downloaded");
        warn!("Download from: https://huggingface.co/ggerganov/whisper.cpp");
    } else {
        info!("Whisper model found at {}", whisper_model_path.display());
    }
    info!("Transcription service initialized (cli: {}, model: {})",
          transcription_config.whisper_cli_path.display(),
          transcription_config.whisper_model_path.display());

    // Initialize upload registry
    let upload_config = uploads::UploadConfig::default();
    let upload_registry = Arc::new(uploads::UploadRegistry::new(upload_config)?);
    info!("Upload registry initialized");

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
        cost_aggregator.clone(),
    ));

    // Start global event tailer (for bead claim/close/release/update events)
    if let Err(e) = supervisor.start_event_tailer().await {
        warn!("Failed to start event tailer: {}", e);
    }

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

    // Project status update task is spawned after DaemonState creation below

    // Forward session events from supervisor to worker registry and cost aggregator
    let registry_for_sessions = worker_registry.clone();
    let cost_aggregator_for_sessions = cost_aggregator.clone();
    tokio::spawn(async move {
        let mut rx = session_tx.subscribe();
        while let Ok(event) = rx.recv().await {
            match event {
                sessions::SessionEvent::ConversationsUpdated { sessions } => {
                    // Aggregate sessions into cost aggregator
                    {
                        let mut aggregator = cost_aggregator_for_sessions.write().unwrap();
                        for session in &sessions {
                            if let Err(e) = aggregator.aggregate_session(session) {
                                warn!("Failed to aggregate session {} into cost: {}", session.id, e);
                            }
                        }
                    }

                    // Update worker registry with conversations
                    registry_for_sessions.update_conversations(sessions).await;
                }
                sessions::SessionEvent::SessionBound { .. } => {
                    // Registry will handle this via the WebSocket
                }
                sessions::SessionEvent::Error(e) => {
                    error!("Session tailer error: {}", e);
                }
                sessions::SessionEvent::TagJoinBound { .. } => {
                    // Tag join events - sessions are already tracked
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
                                        WorkerState::Executing {
                                            bead: bead.clone(),
                                            pid: 0,
                                            adapter: adapter.clone(),
                                        }
                                    }
                                    ws::WorkerDisplayState::Idle { last_strand } => {
                                        WorkerState::Idle {
                                            last_strand: last_strand.clone(),
                                        }
                                    }
                                    ws::WorkerDisplayState::Knot { reason } => {
                                        WorkerState::Knot {
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

    // Initialize projects data from supervisor snapshot
    let supervisor_snapshot = supervisor.snapshot().await;
    let initial_workers = worker_registry.snapshot().await;
    let initial_beads_guard = beads.read().unwrap().clone();

    // Build project metadata lookup (label, color) from config
    let project_metadata: std::collections::HashMap<String, ProjectMetadata> = initial_config
        .registry
        .projects
        .iter()
        .map(|p| {
            let name = p.name().to_string();
            let label = p.label().unwrap_or(&name).to_string();
            let color = p.color().unwrap_or("#3b82f6").to_string();
            (name, ProjectMetadata { label, color })
        })
        .collect();
    let project_metadata: Arc<std::sync::RwLock<std::collections::HashMap<String, ProjectMetadata>>> =
        Arc::new(std::sync::RwLock::new(project_metadata));

    let meta_for_init = project_metadata.clone();
    let initial_projects: Vec<ws::ProjectCardData> = supervisor_snapshot
        .iter()
        .map(|r| {
            let worker_count = initial_workers.len();
            let stuck_count = initial_workers.iter().filter(|w| matches!(w.state, ws::WorkerDisplayState::Knot { .. })).count();
            let active_stitch_count = initial_beads_guard.iter().filter(|b| b.status == BeadStatus::Open).count();
            let last_activity = initial_workers.iter().map(|w| w.last_heartbeat).max().map(|t| t.to_rfc3339());

            let cost_today = cost_aggregator.read().unwrap().cost_today_for_project(&r.project_name);

            let meta = meta_for_init.read().unwrap().get(&r.project_name).cloned();
            ws::ProjectCardData {
                name: r.project_name.clone(),
                label: meta.as_ref().map(|m| m.label.clone()).unwrap_or_else(|| r.project_name.clone()),
                color: meta.as_ref().map(|m| m.color.clone()).unwrap_or_else(|| "#3b82f6".to_string()),
                path: r.project_path.display().to_string(),
                degraded: !r.state.is_running(),
                runtime_state: Some(r.state.to_display_string().to_string()),
                runtime_error: r.state.error().map(|e| e.to_string()),
                bead_count: r.bead_count,
                worker_count,
                active_stitch_count,
                cost_today,
                stuck_count,
                last_activity,
            }
        })
        .collect();

    // Initialize vector index for semantic pre-dedup
    let vector_index = Arc::new(std::sync::RwLock::new(vector_index::VectorIndex::new()));
    {
        let beads_guard = beads.read().unwrap();
        let projects_guard = initial_projects.clone();
        let items = vector_index::build_index_from_state(&beads_guard, &projects_guard);
        vector_index.write().unwrap().rebuild(items);
    }
    info!("Vector index initialized for semantic pre-dedup");

    // Initialize agent session manager from config.yml agent section
    let adapter_config = agent_adapter::load_adapter_config();
    let session_config = agent_session::AgentAdapterConfig {
        adapter: adapter_config.adapter.clone(),
        model: adapter_config.model.clone(),
        anthropic_api_key: adapter_config.anthropic_api_key.clone(),
        zai_base_url: adapter_config.zai_base_url.clone(),
        zai_api_key: adapter_config.zai_api_key.clone(),
        rate_limit_rpm: adapter_config.rate_limit_rpm,
        cost_cap_usd: adapter_config.cost_cap_usd,
    };
    let agent_session_manager = match agent_session::AgentSessionManager::new(session_config).await {
        Ok(mgr) => {
            info!("Agent session manager initialized (adapter={})", adapter_config.adapter);
            Some(Arc::new(mgr))
        }
        Err(e) => {
            warn!("Failed to initialize agent session manager: {}. Agent features disabled.", e);
            None
        }
    };

    // Initialize morning brief broadcast channel and runner
    let (brief_tx, _) = broadcast::channel::<ws::MorningBriefData>(64);
    let morning_brief_runner = match &agent_session_manager {
        Some(mgr) => {
            let config = morning_brief::MorningBriefConfig::default();
            let runner = Arc::new(morning_brief::MorningBriefRunner::new(
                config,
                mgr.clone(),
                brief_tx.clone(),
            ));
            info!("Morning brief runner initialized");
            Some(runner)
        }
        None => {
            warn!("No agent session manager — morning brief disabled");
            None
        }
    };

    // Start morning brief scheduler (auto-triggers at configured hour)
    if let Some(ref runner) = morning_brief_runner {
        let sched_shutdown = shutdown_coordinator.subscribe();
        runner.clone().start_scheduler(sched_shutdown);
        info!("Morning brief scheduler started");
    }

    // Load backup config and resolve S3 credentials from env vars
    let backup_state = backup::load_backup_config();
    match &backup_state {
        backup::BackupState::Ready { config, .. } => {
            info!(
                "Backup subsystem ready (schedule={}, retention={}d, encryption={})",
                config.schedule, config.retention_days, config.encryption
            );
        }
        backup::BackupState::Disabled { reason, .. } => {
            warn!("Backup subsystem disabled: {}", reason);
        }
        backup::BackupState::NotConfigured => {
            info!("Backup subsystem not configured — no backup: section in config.yml");
        }
    }

    // Start backup scheduler when fully configured
    if let backup::BackupState::Ready { config, credentials } = backup_state {
        let pipeline = backup_pipeline::BackupPipeline::new(config, credentials);
        let sched_shutdown = shutdown_coordinator.subscribe();
        pipeline.start_scheduler(sched_shutdown);
        info!("Backup scheduler started");
    }

    let bead_tx_for_rebuild = bead_tx.clone();
    let stitch_tx_for_rebuild = stitch_tx.clone();
    let vector_index_for_rebuild = vector_index.clone();
    let beads_for_rebuild = beads.clone();
    let initial_projects_for_rebuild = initial_projects.clone();

    let state = DaemonState {
        config: config.clone(),
        started_at: Instant::now(),
        worker_registry,
        beads,
        bead_tx,
        stitch_tx,
        shutdown: shutdown_coordinator.clone(),
        supervisor,
        projects: Arc::new(std::sync::RwLock::new(initial_projects)),
        project_metadata: project_metadata.clone(),
        config_status_tx,
        project_status_tx,
        capacity_tx,
        cost_aggregator: cost_aggregator.clone(),
        transcription_service: Some(transcription_service),
        upload_registry,
        active_project: Arc::new(std::sync::RwLock::new(None)),
        vector_index,
        agent_session_manager,
        morning_brief_runner,
        brief_tx,
        draft_tx: broadcast::channel::<ws::DraftUpdateData>(64).0,
        resolved_config,
    };

    // Forward project runtime status updates to shared store and broadcast
    let projects_for_update = state.projects.clone();
    let registry_for_cards = state.worker_registry.clone();
    let beads_for_cards = state.beads.clone();
    let supervisor_for_cards = state.supervisor.clone();
    let project_status_tx_clone = state.project_status_tx.clone();
    let meta_for_updates = project_metadata.clone();
    let cost_for_updates = cost_aggregator.clone();
    tokio::spawn(async move {
        let mut rx = supervisor_for_cards.subscribe_status();
        while let Ok(runtime_status) = rx.recv().await {
            let workers = registry_for_cards.snapshot().await;
            let all_beads = beads_for_cards.read().unwrap().clone();
            let cost_today = cost_for_updates.read().unwrap().cost_today_for_project(&runtime_status.project_name);

            let worker_count = workers.len();
            let stuck_count = workers.iter().filter(|w| matches!(w.state, ws::WorkerDisplayState::Knot { .. })).count();
            let active_stitch_count = all_beads.iter().filter(|b| b.status == BeadStatus::Open).count();
            let last_activity = workers.iter().map(|w| w.last_heartbeat).max().map(|t| t.to_rfc3339());

            let meta = meta_for_updates.read().unwrap().get(&runtime_status.project_name).cloned();
            let card = ws::ProjectCardData {
                name: runtime_status.project_name.clone(),
                label: meta.as_ref().map(|m| m.label.clone()).unwrap_or_else(|| runtime_status.project_name.clone()),
                color: meta.as_ref().map(|m| m.color.clone()).unwrap_or_else(|| "#3b82f6".to_string()),
                path: runtime_status.project_path.display().to_string(),
                degraded: !runtime_status.state.is_running(),
                runtime_state: Some(runtime_status.state.to_display_string().to_string()),
                runtime_error: runtime_status.state.error().map(|e| e.to_string()),
                bead_count: runtime_status.bead_count,
                worker_count,
                active_stitch_count,
                cost_today,
                stuck_count,
                last_activity,
            };

            {
                let mut projects = projects_for_update.write().unwrap();
                if let Some(p) = projects.iter_mut().find(|p| p.name == card.name) {
                    *p = card.clone();
                } else {
                    projects.push(card.clone());
                }
            }

            let _ = project_status_tx_clone.send(card);
        }
    });

    // Vector index rebuilder: rebuild on bead/Stitch events for real-time dedup
    {
        let vindex_ref = vector_index_for_rebuild;
        let beads_ref = beads_for_rebuild;
        let projects_ref = Arc::new(std::sync::RwLock::new(initial_projects_for_rebuild));
        let mut bead_rx = bead_tx_for_rebuild.subscribe();
        let mut stitch_rx = stitch_tx_for_rebuild.subscribe();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Bead events: rebuild index when beads are created/closed
                    result = bead_rx.recv() => {
                        if result.is_ok() {
                            let beads = beads_ref.read().unwrap().clone();
                            let projects = projects_ref.read().unwrap().clone();
                            let items = vector_index::build_index_from_state(&beads, &projects);
                            vindex_ref.write().unwrap().rebuild(items);
                            tracing::debug!("Vector index rebuilt on bead event");
                        }
                    }
                    // Stitch events: rebuild index when stitches are created/closed
                    result = stitch_rx.recv() => {
                        if result.is_ok() {
                            let beads = beads_ref.read().unwrap().clone();
                            let projects = projects_ref.read().unwrap().clone();
                            let items = vector_index::build_index_from_state(&beads, &projects);
                            vindex_ref.write().unwrap().rebuild(items);
                            tracing::debug!("Vector index rebuilt on stitch event");
                        }
                    }
                }
            }
        });
        info!("Vector index rebuilder started (listens to bead/Stitch events)");
    }

    // Periodic project card refresh for live metrics (every 5s)
    {
        let supervisor_ref = state.supervisor.clone();
        let registry_ref = state.worker_registry.clone();
        let beads_ref = state.beads.clone();
        let cost_ref = cost_aggregator.clone();
        let meta_ref = project_metadata.clone();
        let projects_ref = state.projects.clone();
        let status_ref = state.project_status_tx.clone();
        let vindex_ref = state.vector_index.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            let mut vindex_counter: u32 = 0;
            loop {
                interval.tick().await;
                let runtime_statuses = supervisor_ref.snapshot().await;
                let workers = registry_ref.snapshot().await;
                let all_beads = beads_ref.read().unwrap().clone();
                let meta = meta_ref.read().unwrap().clone();

                let new_cards: Vec<ws::ProjectCardData> = runtime_statuses
                    .iter()
                    .map(|r| {
                        let cost = cost_ref.read().unwrap().cost_today_for_project(&r.project_name);
                        let worker_count = workers.len();
                        let stuck_count = workers.iter().filter(|w| matches!(w.state, ws::WorkerDisplayState::Knot { .. })).count();
                        let active_stitch_count = all_beads.iter().filter(|b| b.status == BeadStatus::Open).count();
                        let last_activity = workers.iter().map(|w| w.last_heartbeat).max().map(|t| t.to_rfc3339());
                        let m = meta.get(&r.project_name);

                        ws::ProjectCardData {
                            name: r.project_name.clone(),
                            label: m.map(|m| m.label.clone()).unwrap_or_else(|| r.project_name.clone()),
                            color: m.map(|m| m.color.clone()).unwrap_or_else(|| "#3b82f6".to_string()),
                            path: r.project_path.display().to_string(),
                            degraded: !r.state.is_running(),
                            runtime_state: Some(r.state.to_display_string().to_string()),
                            runtime_error: r.state.error().map(|e| e.to_string()),
                            bead_count: r.bead_count,
                            worker_count,
                            active_stitch_count,
                            cost_today: cost,
                            stuck_count,
                            last_activity,
                        }
                    })
                    .collect();

                // Rebuild vector index every 30s (6th tick of the 5s interval)
                vindex_counter += 1;
                if vindex_counter >= 6 {
                    vindex_counter = 0;
                    let items = vector_index::build_index_from_state(&all_beads, &new_cards);
                    vindex_ref.write().unwrap().rebuild(items);
                }

                *projects_ref.write().unwrap() = new_cards.clone();
                for card in new_cards {
                    let _ = status_ref.send(card);
                }
            }
        });
    }

    let app = router()
        .with_state(state.clone())
        .into_make_service_with_connect_info::<std::net::SocketAddr>();

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
        let _ = shutdown_coordinator.shutdown(None).await;
    };

    // Run the server with graceful shutdown
    let result = tokio::select! {
        r = tcp_server.with_graceful_shutdown(async move {
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
