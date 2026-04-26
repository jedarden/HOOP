//! HOOP daemon - the long-lived server process
//!
//! The daemon hosts the web UI, WebSocket endpoints, and REST API.
//! It reads from projects, beads, sessions, files, events, and heartbeats.
//! Its only write is `br create` for bead creation.

pub mod adb_dictate;
pub mod agent_adapter;
pub mod atomic_write;
pub mod agent_context;
pub mod agent_session;
pub mod api_agent;
pub mod api_config;
pub mod api_conversations;
pub mod attachment_sync;
pub mod audio_redaction;
pub mod backup;
pub mod backup_pipeline;
pub mod config_resolver;
pub mod config_watcher;
pub mod api_attachments;
pub mod api_audit;
pub mod api_beads;
pub mod api_dictated_notes;
pub mod api_draft_queue;
pub mod api_metrics;
pub mod api_preview;
pub mod api_stitch_decompose;
pub mod api_stitch_links;
pub mod api_stitch_read;
pub mod api_timeline;
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
pub mod parse_jsonl_safe;
pub mod path_security;
pub mod pattern_query_evaluator;
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
pub mod api_patterns;
pub mod redaction;
pub mod redaction_policy;
pub mod syntax_highlight;
pub mod worker_ack;
// TODO: Uncomment when collision_detector is complete
// pub mod collision_detector;
pub mod bead_commit_index;
pub mod api_diff;
pub mod api_fix_patterns;
pub mod api_blame;
pub mod api_screen_capture;
pub mod api_orphans;
pub mod orphan_beads;
pub mod net_diff;
pub mod screen_capture;
// TODO: Uncomment when observer is complete
// pub mod observer;
pub mod cost_anomaly;
pub mod fix_patterns;
pub mod stitch_percentile_index;

/// Worker execution state from heartbeats
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "state", rename_all = "lowercase")]
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
    /// Project name assigned by HOOP at load time — not stored in issues.jsonl
    #[serde(skip_deserializing, default)]
    pub project: String,
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
use sha2::{Digest, Sha256};
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
    /// Observer mode: read-only attach to primary daemon
    pub observer_mode: bool,
    /// Primary daemon address for observer mode
    pub primary_addr: SocketAddr,
}

impl Default for Config {
    fn default() -> Self {
        let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.push(".hoop");
        Self {
            bind_addr: SocketAddr::from(([127, 0, 0, 1], 3000)),
            control_socket_path: home.join("control.sock"),
            allow_br_mismatch: false,
            observer_mode: false,
            primary_addr: SocketAddr::from(([127, 0, 0, 1], 3000)),
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
    /// Per-connection WS client registry for /debug/state (§16.8)
    pub ws_connection_tracker: Arc<ws::WsConnectionTracker>,
    /// Spawn-ack monitor — verifies workers wrote ~/.hoop/workers/<name>.ack (§M5)
    pub worker_ack_monitor: Arc<worker_ack::WorkerAckMonitor>,
    /// Broadcast channel for collision alert events (§6 Phase 2, deliverable 12)
    pub collision_alert_tx: broadcast::Sender<ws::CollisionAlertData>,
    /// Broadcast channel for pattern saved query synced events (§4.7)
    pub pattern_tx: broadcast::Sender<ws::PatternSavedQuerySyncedData>,
    /// Per-project redaction policy resolver (§18.5)
    pub redaction_policy_state: Arc<tokio::sync::RwLock<redaction_policy::RedactionPolicyState>>,
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
) -> Result<Json<Vec<cost::CostBucket>>, (axum::http::StatusCode, String)> {
    id_validators::validate_project_name(&project).map_err(id_validators::rejection)?;
    let aggregator = state.cost_aggregator.read().unwrap();
    Ok(Json(aggregator.get_buckets_by_project(&project)))
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

/// Query parameters for the Codex account daily spend endpoint.
#[derive(Debug, serde::Deserialize)]
struct CodexSpendQuery {
    /// Filter to a specific account_id (optional)
    account_id: Option<String>,
    /// Start date inclusive, YYYY-MM-DD (optional)
    date_from: Option<String>,
    /// End date inclusive, YYYY-MM-DD (optional)
    date_to: Option<String>,
}

/// GET /api/cost/codex-accounts — per-account daily spend rows
async fn get_codex_account_daily_spend(
    axum::extract::Query(params): axum::extract::Query<CodexSpendQuery>,
) -> Result<Json<Vec<fleet::CodexAccountDailySpendRow>>, (axum::http::StatusCode, String)> {
    tokio::task::spawn_blocking(move || {
        fleet::query_codex_account_daily_spend(
            params.account_id.as_deref(),
            params.date_from.as_deref(),
            params.date_to.as_deref(),
        )
    })
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .map(Json)
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Query parameters for the Codex account monthly rollup endpoint.
#[derive(Debug, serde::Deserialize)]
struct CodexMonthlyQuery {
    /// Filter to a specific account_id (optional)
    account_id: Option<String>,
    /// Start month inclusive, YYYY-MM (optional)
    month_from: Option<String>,
    /// End month inclusive, YYYY-MM (optional)
    month_to: Option<String>,
}

/// GET /api/cost/codex-accounts/monthly — per-account monthly spend rollup
async fn get_codex_account_monthly_rollup(
    axum::extract::Query(params): axum::extract::Query<CodexMonthlyQuery>,
) -> Result<Json<Vec<fleet::CodexAccountMonthlyRollupRow>>, (axum::http::StatusCode, String)> {
    tokio::task::spawn_blocking(move || {
        fleet::query_codex_account_monthly_rollup(
            params.account_id.as_deref(),
            params.month_from.as_deref(),
            params.month_to.as_deref(),
        )
    })
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .map(Json)
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
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
    if id_validators::validate_project_name(&project).is_err() {
        return Err(axum::http::StatusCode::BAD_REQUEST);
    }

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

/// Query parameters for the file search endpoint.
#[derive(serde::Deserialize)]
struct FileSearchQuery {
    /// Extension filter: "rs", "ts,tsx", "*.{ts,tsx}", etc.
    ext: Option<String>,
    /// Git ref; only files changed since this ref are returned (e.g. "HEAD~3").
    modified_since: Option<String>,
    /// Ripgrep content pattern.
    grep: Option<String>,
}

/// Search files in a project using extension, modified-since, and/or grep filters.
async fn search_project_files(
    axum::extract::Path(project): axum::extract::Path<String>,
    axum::extract::Query(params): axum::extract::Query<FileSearchQuery>,
    axum::extract::State(state): axum::extract::State<DaemonState>,
) -> Result<Json<Vec<files::FileSearchResult>>, axum::http::StatusCode> {
    if id_validators::validate_project_name(&project).is_err() {
        return Err(axum::http::StatusCode::BAD_REQUEST);
    }

    let project_root = {
        let projects = state.projects.read().unwrap();
        projects
            .iter()
            .find(|p| p.name == project)
            .map(|p| std::path::PathBuf::from(&p.path))
            .ok_or(axum::http::StatusCode::NOT_FOUND)?
    };

    let ext_exts = params
        .ext
        .as_deref()
        .map(files::parse_ext_patterns)
        .unwrap_or_default();

    let modified_since = params.modified_since.clone();
    let grep = params.grep.clone();

    // Validate modified_since ref (no shell metacharacters).
    if let Some(ref r) = modified_since {
        if r.contains(|c: char| matches!(c, ';' | '|' | '&' | '$' | '`' | '\n')) {
            return Err(axum::http::StatusCode::BAD_REQUEST);
        }
    }

    let results = tokio::task::spawn_blocking(move || {
        files::search_files(
            &project_root,
            &ext_exts,
            modified_since.as_deref(),
            grep.as_deref(),
        )
    })
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(results))
}

/// Query parameters for the file content / syntax-highlight endpoint.
#[derive(serde::Deserialize)]
struct FileContentQuery {
    /// Relative path from the project root (required).
    path: String,
    /// Theme alias: dark | light | solarized-dark | solarized-light | eighties | mocha-dark | ocean-light
    theme: Option<String>,
    /// If true, return raw text/plain content (≤50 KB) instead of highlighted JSON.
    raw: Option<bool>,
    /// If true, return raw binary bytes with MIME-type detection (for images ≤50 MB).
    image: Option<bool>,
}

/// Return file content — either raw text/plain (≤50 KB, `?raw=true`) or
/// syntect-highlighted JSON.  The raw mode is used by the client-side Shiki
/// highlighter; the JSON mode is kept for legacy / server-rendered callers.
async fn get_file_content(
    axum::extract::Path(project): axum::extract::Path<String>,
    axum::extract::Query(params): axum::extract::Query<FileContentQuery>,
    axum::extract::State(state): axum::extract::State<DaemonState>,
) -> Result<axum::response::Response, (axum::http::StatusCode, String)> {
    if id_validators::validate_project_name(&project).is_err() {
        return Err((axum::http::StatusCode::BAD_REQUEST, "invalid project name".into()));
    }

    let project_root = {
        let projects = state.projects.read().unwrap();
        projects
            .iter()
            .find(|p| p.name == project)
            .map(|p| std::path::PathBuf::from(&p.path))
            .ok_or((axum::http::StatusCode::NOT_FOUND, "project not found".into()))?
    };

    let rel_path = params.path;
    if !files::is_safe_rel_path(&rel_path) {
        return Err((axum::http::StatusCode::FORBIDDEN, "unsafe path".into()));
    }

    let want_raw = params.raw.unwrap_or(false);
    let want_image = params.image.unwrap_or(false);
    let theme = params.theme.unwrap_or_else(|| "dark".to_owned());

    if want_image {
        // Binary image mode: stream raw bytes with detected MIME type (≤50 MB).
        // Path security and metadata are resolved in a blocking task; the file is
        // then streamed asynchronously so large images (10 MB+) are delivered
        // progressively without buffering the whole file in server memory.
        const IMAGE_MAX: u64 = 50 * 1024 * 1024;
        let (abs_path, file_size, mime) =
            tokio::task::spawn_blocking(move || -> anyhow::Result<(std::path::PathBuf, u64, &'static str)> {
                use crate::path_security::{canonicalize_and_check, PathAllowlist};
                let allowlist = PathAllowlist::for_workspace(&project_root)
                    .map_err(|e| anyhow::anyhow!("workspace allowlist: {e}"))?;
                let abs_path = canonicalize_and_check(&project_root.join(&rel_path), &allowlist)
                    .map_err(|e| anyhow::anyhow!("path traversal: {e}"))?;
                if !abs_path.is_file() {
                    anyhow::bail!("not a file");
                }
                let meta = std::fs::metadata(&abs_path)?;
                let file_size = meta.len();
                if file_size > IMAGE_MAX {
                    anyhow::bail!("image too large (>50 MB)");
                }
                let mime = match abs_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase()
                    .as_str()
                {
                    "jpg" | "jpeg" => "image/jpeg",
                    "png" => "image/png",
                    "gif" => "image/gif",
                    "webp" => "image/webp",
                    "svg" | "svgz" => "image/svg+xml",
                    "bmp" => "image/bmp",
                    "ico" => "image/x-icon",
                    _ => "application/octet-stream",
                };
                Ok((abs_path, file_size, mime))
            })
            .await
            .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .map_err(|e| {
                let msg = e.to_string();
                let status = if msg.contains("not a file") || msg.contains("not found") {
                    axum::http::StatusCode::NOT_FOUND
                } else if msg.contains("too large") {
                    axum::http::StatusCode::PAYLOAD_TOO_LARGE
                } else {
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR
                };
                (status, msg)
            })?;

        let file = tokio::fs::File::open(&abs_path)
            .await
            .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let stream = tokio_util::io::ReaderStream::new(file);

        return Ok(axum::response::Response::builder()
            .header("Content-Type", mime)
            .header("Content-Length", file_size.to_string())
            .header("Cache-Control", "private, max-age=60")
            .header("Accept-Ranges", "bytes")
            .body(axum::body::Body::from_stream(stream))
            .unwrap());
    }

    if want_raw {
        // Raw mode: return plain UTF-8 text, capped at 50 KB.
        const RAW_MAX: u64 = 50 * 1024;
        let content = tokio::task::spawn_blocking(move || -> anyhow::Result<String> {
            use crate::path_security::{canonicalize_and_check, PathAllowlist};
            let allowlist = PathAllowlist::for_workspace(&project_root)
                .map_err(|e| anyhow::anyhow!("workspace allowlist: {e}"))?;
            let abs_path = canonicalize_and_check(&project_root.join(&rel_path), &allowlist)
                .map_err(|e| anyhow::anyhow!("path traversal: {e}"))?;
            if !abs_path.is_file() {
                anyhow::bail!("not a file");
            }
            let meta = std::fs::metadata(&abs_path)?;
            if meta.len() > RAW_MAX {
                anyhow::bail!("file too large for raw mode (>50 KB)");
            }
            std::fs::read_to_string(&abs_path).map_err(|e| anyhow::anyhow!("read: {e}"))
        })
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map_err(|e| {
            let msg = e.to_string();
            let status = if msg.contains("not a file") || msg.contains("not found") {
                axum::http::StatusCode::NOT_FOUND
            } else if msg.contains("too large") {
                axum::http::StatusCode::PAYLOAD_TOO_LARGE
            } else {
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            };
            (status, msg)
        })?;

        return Ok(axum::response::Response::builder()
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(axum::body::Body::from(content))
            .unwrap());
    }

    // Highlighted JSON mode (syntect — server-side, legacy).
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<syntax_highlight::HighlightResult> {
        use crate::path_security::{canonicalize_and_check, PathAllowlist};

        let allowlist = PathAllowlist::for_workspace(&project_root)
            .map_err(|e| anyhow::anyhow!("workspace allowlist: {e}"))?;
        let abs_path = project_root.join(&rel_path);
        let abs_path = canonicalize_and_check(&abs_path, &allowlist)
            .map_err(|e| anyhow::anyhow!("path traversal: {e}"))?;

        if !abs_path.is_file() {
            anyhow::bail!("not a file");
        }

        let meta = std::fs::metadata(&abs_path)?;
        if meta.len() > 100 * 1024 * 1024 {
            anyhow::bail!("file too large (>100 MB)");
        }

        let content = std::fs::read_to_string(&abs_path)
            .map_err(|e| anyhow::anyhow!("read: {e}"))?;

        let filename = abs_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        syntax_highlight::highlight_content(&content, filename, &theme)
    })
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .map_err(|e| {
        let msg = e.to_string();
        if msg.contains("not a file") || msg.contains("not found") {
            (axum::http::StatusCode::NOT_FOUND, msg)
        } else {
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, msg)
        }
    })?;

    let body = serde_json::to_string(&result)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(axum::response::Response::builder()
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(body))
        .unwrap())
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

/// Spend aggregated by project for the cross-project dashboard
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectSpend {
    pub project: String,
    pub cost_usd: f64,
}

/// Spend aggregated by adapter for the cross-project dashboard
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdapterSpend {
    pub adapter: String,
    pub cost_usd: f64,
}

/// Worker count aggregated by project for the cross-project dashboard
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectWorkers {
    pub project: String,
    pub worker_count: usize,
}

/// Longest-running active stitch (open bead) entry
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LongestRunningStitch {
    pub project: String,
    pub bead_id: String,
    pub title: String,
    pub created_at: String,
    pub duration_secs: i64,
}

/// Response for the cross-project dashboard endpoint
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CrossProjectDashboardResponse {
    /// The requested range key ("today" | "week" | "month")
    pub range: String,
    /// Human-readable label for the range
    pub range_label: String,
    /// Total spend across all projects and adapters for this range (USD)
    pub total_spend_usd: f64,
    /// Spend broken down by project, descending
    pub spend_by_project: Vec<ProjectSpend>,
    /// Spend broken down by adapter, descending
    pub spend_by_adapter: Vec<AdapterSpend>,
    /// Total active workers across all projects
    pub total_workers: usize,
    /// Worker counts per project
    pub workers_by_project: Vec<ProjectWorkers>,
    /// Up to 10 longest-running open beads, descending by duration
    pub longest_running_stitches: Vec<LongestRunningStitch>,
}

/// Query parameters for the cross-project dashboard
#[derive(Debug, serde::Deserialize)]
struct CrossProjectDashboardQuery {
    /// "today" | "week" | "month" (default: "today")
    #[serde(default = "default_range")]
    range: String,
}

fn default_range() -> String {
    "today".to_string()
}

/// Cross-project dashboard endpoint — aggregates spend, workers, and longest-running stitches
async fn get_cross_project_dashboard(
    axum::extract::Query(params): axum::extract::Query<CrossProjectDashboardQuery>,
    axum::extract::State(state): axum::extract::State<DaemonState>,
) -> Json<CrossProjectDashboardResponse> {
    use chrono::{Utc, Duration, Datelike};

    let now = Utc::now();
    let today = now.date_naive();

    let (start_date, range_label) = match params.range.as_str() {
        "week" => (today - Duration::days(6), "Last 7 days"),
        "month" => {
            let month_start = chrono::NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
                .unwrap_or(today);
            (month_start, "This month")
        }
        _ => (today, "Today"),
    };
    let range_key = match params.range.as_str() {
        "week" => "week",
        "month" => "month",
        _ => "today",
    };

    // --- Cost aggregation ---
    let buckets = {
        let aggregator = state.cost_aggregator.read().unwrap();
        aggregator.get_buckets_by_date_range(start_date, today)
    };

    let mut by_project: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    let mut by_adapter: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    let mut total_spend = 0.0f64;

    for bucket in &buckets {
        *by_project.entry(bucket.project.clone()).or_default() += bucket.cost_usd;
        *by_adapter.entry(bucket.adapter.clone()).or_default() += bucket.cost_usd;
        total_spend += bucket.cost_usd;
    }

    let mut spend_by_project: Vec<ProjectSpend> = by_project
        .into_iter()
        .map(|(project, cost_usd)| ProjectSpend { project, cost_usd })
        .collect();
    spend_by_project.sort_by(|a, b| b.cost_usd.partial_cmp(&a.cost_usd).unwrap_or(std::cmp::Ordering::Equal));

    let mut spend_by_adapter: Vec<AdapterSpend> = by_adapter
        .into_iter()
        .map(|(adapter, cost_usd)| AdapterSpend { adapter, cost_usd })
        .collect();
    spend_by_adapter.sort_by(|a, b| b.cost_usd.partial_cmp(&a.cost_usd).unwrap_or(std::cmp::Ordering::Equal));

    // --- Worker counts per project ---
    let project_cards = state.projects.read().unwrap().clone();
    let total_workers: usize = project_cards.iter().map(|c| c.worker_count).sum();
    let mut workers_by_project: Vec<ProjectWorkers> = project_cards
        .iter()
        .filter(|c| c.worker_count > 0)
        .map(|c| ProjectWorkers {
            project: c.name.clone(),
            worker_count: c.worker_count,
        })
        .collect();
    workers_by_project.sort_by(|a, b| b.worker_count.cmp(&a.worker_count));

    // --- Longest-running stitches (open beads sorted by age) ---
    let beads_guard = state.beads.read().unwrap();
    let mut open_beads: Vec<&Bead> = beads_guard
        .iter()
        .filter(|b| b.status == BeadStatus::Open)
        .collect();
    // Sort oldest first (longest running)
    open_beads.sort_by_key(|b| b.created_at);

    let longest_running_stitches: Vec<LongestRunningStitch> = open_beads
        .iter()
        .take(10)
        .map(|b| {
            let duration_secs = (now - b.created_at).num_seconds().max(0);
            LongestRunningStitch {
                // project is tagged at load time by the supervisor
                project: if b.project.is_empty() {
                    project_cards.first().map(|c| c.name.clone()).unwrap_or_else(|| "unknown".to_string())
                } else {
                    b.project.clone()
                },
                bead_id: b.id.clone(),
                title: b.title.clone(),
                created_at: b.created_at.to_rfc3339(),
                duration_secs,
            }
        })
        .collect();

    Json(CrossProjectDashboardResponse {
        range: range_key.to_string(),
        range_label: range_label.to_string(),
        total_spend_usd: total_spend,
        spend_by_project,
        spend_by_adapter,
        total_workers,
        workers_by_project,
        longest_running_stitches,
    })
}

/// GET /api/fleet/runtime-status — cross-project runtime status with liveness derived on read.
async fn get_runtime_status() -> Result<Json<Vec<fleet::RuntimeStatusRow>>, (axum::http::StatusCode, String)> {
    tokio::task::spawn_blocking(fleet::query_runtime_status)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map(Json)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Query parameters for the bead-commits index endpoint.
#[derive(serde::Deserialize)]
struct BeadCommitsQuery {
    /// Filter by bead_id (e.g. `?bead_id=hoop-ttb.3.35`)
    bead_id: Option<String>,
    /// Filter by commit SHA (e.g. `?sha=abc123`)
    sha: Option<String>,
}

/// GET /api/bead-commits — query the bead-to-commit index by bead_id or sha.
async fn get_bead_commits(
    axum::extract::Query(params): axum::extract::Query<BeadCommitsQuery>,
) -> Result<Json<Vec<fleet::BeadCommitRow>>, (axum::http::StatusCode, String)> {
    tokio::task::spawn_blocking(move || {
        if let Some(bead_id) = params.bead_id {
            fleet::query_bead_commits_by_bead_id(&bead_id)
        } else if let Some(sha) = params.sha {
            fleet::query_bead_commits_by_sha(&sha)
        } else {
            Err(anyhow::anyhow!("Requires ?bead_id= or ?sha= query parameter"))
        }
    })
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .map(Json)
    .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, e.to_string()))
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
        .route("/api/cost/codex-accounts", get(get_codex_account_daily_spend))
        .route("/api/cost/codex-accounts/monthly", get(get_codex_account_monthly_rollup))
        .route("/api/dashboard/cross-project", get(get_cross_project_dashboard))
        .route("/api/fleet/runtime-status", get(get_runtime_status))
        .route("/api/bead-commits", get(get_bead_commits))
        .route("/api/projects/:project/files", get(get_project_files))
        .route("/api/projects/:project/files/search", get(search_project_files))
        .route("/api/projects/:project/files/content", get(get_file_content))
        .route("/api/attachments/:attachment_type/:id/:filename", get(api_attachments::serve_attachment))
        .route("/ws", get(ws::ws_handler))
        .merge(api_uploads::router())
        .merge(api_dictated_notes::router())
        .merge(api_transcription::router())
        .merge(adb_dictate::router())
        .merge(api_audit::router())
        .merge(api_conversations::router())
        .merge(api_beads::router())
        .merge(api_draft_queue::router())
        .merge(api_preview::router())
        .merge(api_stitch_decompose::router())
        .merge(api_stitch_read::router())
        .merge(api_stitch_links::router())
        .merge(api_patterns::router())
        .merge(api_diff::router())
        .merge(api_blame::router())
        .merge(api_screen_capture::router())
        .merge(api_orphans::router())
        .merge(api_fix_patterns::router())
        .merge(net_diff::router())
        .route("/api/workers/timeline", get(api_timeline::get_worker_timeline))
        .merge(api_agent::router())
        .merge(api_morning_brief::router())
        .merge(api_metrics::router())
        .merge(api_config::router())
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
    // Observer mode: run read-only attach to primary daemon
    if config.observer_mode {
        // TODO: Uncomment when observer is complete
        // return observer::serve_observer(config).await;
        return Err(anyhow::anyhow!("Observer mode is not yet implemented"));
    }

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
    let resolved_config = Arc::new(config_resolver::resolve(cli_overrides.clone()));
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

    // Initialize worker spawn-ack monitor (§M5)
    let mut worker_ack_monitor = worker_ack::WorkerAckMonitor::new()?;
    worker_ack_monitor.start()?;
    let worker_ack_monitor = Arc::new(worker_ack_monitor);
    info!("Worker ack monitor started");

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
    // Trigger channel: bead close events wake the capacity meter immediately
    let (capacity_trigger_tx, capacity_trigger_rx) = broadcast::channel::<()>(256);
    capacity::CapacityMeter::spawn_refresh_loop(
        capacity_meter_config,
        capacity_tx.clone(),
        Some(capacity_trigger_rx),
    );
    info!("Capacity meter refresh loop started ({} account(s))", account_count);

    // Forward bead close events → capacity trigger for immediate recompute
    {
        let mut bead_close_rx = bead_tx.subscribe();
        let cap_trig = capacity_trigger_tx.clone();
        tokio::spawn(async move {
            loop {
                match bead_close_rx.recv().await {
                    Ok(bead) if bead.status == "closed" => { let _ = cap_trig.send(()); }
                    Ok(_) => {}
                    Err(broadcast::error::RecvError::Lagged(_)) => {}
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });
    }

    // Update percentile index on bead close events
    {
        let mut bead_close_rx = bead_tx.subscribe();
        tokio::spawn(async move {
            loop {
                match bead_close_rx.recv().await {
                    Ok(bead) if bead.status == "closed" => {
                        // Check if this bead belongs to a Stitch and update the index
                        if let Some(stitch_id) = find_stitch_for_bead(&bead.id).await {
                            tokio::task::spawn_blocking(move || {
                                if let Err(e) = crate::stitch_percentile_index::update_on_stitch_close(&stitch_id) {
                                    tracing::debug!("Failed to update percentile index for stitch {}: {}", stitch_id, e);
                                }
                            }).await.ok();
                        }
                    }
                    Ok(_) => {}
                    Err(broadcast::error::RecvError::Lagged(_)) => {}
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });
        info!("Percentile index updater started (listens to bead close events)");
    }

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

    // Spawn bead-to-commit indexer (full walk on startup, incremental every 30s)
    {
        let workspace_paths: Vec<String> = initial_config
            .all_workspace_paths()
            .into_iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        bead_commit_index::spawn_indexer(workspace_paths);
    }

    // Spawn task to handle projects config changes
    let supervisor_for_reconcile = supervisor.clone();
    let config_tx_for_reload = config_status_tx.clone();
    tokio::spawn(async move {
        let mut rx = projects_watcher.subscribe();
        while let Ok(event) = rx.recv().await {
            match event {
                projects::ProjectsEvent::ConfigReloaded { config, prev_hash, delta_keys } => {
                    info!("Projects configuration reloaded, reconciling runtimes");
                    if let Err(e) = supervisor_for_reconcile.reconcile(&config).await {
                        error!("Failed to reconcile runtimes: {}", e);
                    }

                    // Increment success metric (§17.5)
                    metrics::metrics().hoop_config_reload_success_total.inc();

                    // Audit the successful reload (§17.4)
                    let audit_args = projects::ConfigReloadAudit {
                        file: config.path.display().to_string(),
                        prev_hash: prev_hash.clone(),
                        new_hash: config.content_hash.clone(),
                        delta_keys: delta_keys.clone(),
                        actor: "system:hot-reload".to_string(),
                    };
                    if let Ok(args_json) = serde_json::to_string(&audit_args) {
                        if let Err(e) = fleet::write_audit_row(
                            "system:hot-reload",
                            fleet::ActionKind::ConfigReloaded,
                            &config.path.display().to_string(),
                            None,
                            Some(args_json),
                            fleet::ActionResult::Success,
                            None,
                            None,
                            None,
                            None,
                        ) {
                            warn!("Failed to write config_reloaded audit row: {}", e);
                        }
                    }

                    // Broadcast valid config status
                    let _ = config_tx_for_reload.send(ws::ConfigStatusData {
                        valid: true,
                        error: None,
                    });
                }
                projects::ProjectsEvent::ConfigError { error, prev_hash } => {
                    warn!("Projects configuration error: {}", error.message);

                    // Increment rejection metric (§17.5)
                    metrics::metrics().hoop_config_reload_rejected_total.inc();

                    // Audit the rejected reload (§17.4)
                    let registry_path = projects::ProjectsConfig::load()
                        .map(|c| c.path.display().to_string())
                        .unwrap_or_else(|_| "~/.hoop/projects.yaml".to_string());
                    let audit_args = projects::ConfigReloadRejectedAudit {
                        file: registry_path,
                        prev_hash: prev_hash.clone(),
                        error: error.message.clone(),
                        actor: "system:hot-reload".to_string(),
                    };
                    if let Ok(args_json) = serde_json::to_string(&audit_args) {
                        if let Err(e) = fleet::write_audit_row(
                            "system:hot-reload",
                            fleet::ActionKind::ConfigReloadRejected,
                            &audit_args.file,
                            None,
                            Some(args_json),
                            fleet::ActionResult::Failure,
                            Some(error.message.clone()),
                            None,
                            None,
                            None,
                        ) {
                            warn!("Failed to write config_reload_rejected audit row: {}", e);
                        }
                    }

                    // Broadcast config error with structured details (§17.5)
                    let _ = config_tx_for_reload.send(ws::ConfigStatusData {
                        valid: false,
                        error: Some(ws::ConfigErrorData {
                            message: error.message.clone(),
                            line: error.line,
                            col: error.col,
                            field: error.field.clone(),
                            expected: error.expected.clone(),
                            got: error.got.clone(),
                        }),
                    });
                }
            }
        }
    });

    // Start config.yml watcher (§17)
    let mut config_watcher = config_watcher::ConfigWatcher::new(cli_overrides)?;
    config_watcher.start()?;

    // Spawn task to handle config.yml changes
    let config_tx_for_config = config_status_tx.clone();
    tokio::spawn(async move {
        let mut rx = config_watcher.subscribe();
        while let Ok(event) = rx.recv().await {
            match event {
                config_watcher::ConfigEvent::ConfigReloaded { config: _, prev_hash } => {
                    info!("config.yml reloaded successfully");
                    // Note: config.yml changes don't trigger runtime reconcile
                    // because they only affect defaults for new sessions

                    // Increment success metric (§17.5)
                    metrics::metrics().hoop_config_reload_success_total.inc();

                    // Audit the successful reload (§17.4)
                    let config_path = dirs::home_dir()
                        .unwrap_or_else(|| PathBuf::from("."))
                        .join(".hoop")
                        .join("config.yml");
                    let audit_args = config_watcher::ConfigReloadAudit {
                        file: config_path.display().to_string(),
                        prev_hash: prev_hash.clone(),
                        new_hash: hex::encode(Sha256::digest(
                            std::fs::read_to_string(&config_path)
                                .unwrap_or_default()
                                .as_bytes()
                        )),
                        actor: "system:hot-reload".to_string(),
                    };
                    if let Ok(args_json) = serde_json::to_string(&audit_args) {
                        if let Err(e) = fleet::write_audit_row(
                            "system:hot-reload",
                            fleet::ActionKind::ConfigReloaded,
                            &audit_args.file,
                            None,
                            Some(args_json),
                            fleet::ActionResult::Success,
                            None,
                            None,
                            None,
                            None,
                        ) {
                            warn!("Failed to write config.yml reload audit row: {}", e);
                        }
                    }

                    // Broadcast valid config status
                    let _ = config_tx_for_config.send(ws::ConfigStatusData {
                        valid: true,
                        error: None,
                    });
                }
                config_watcher::ConfigEvent::ConfigError { error, prev_hash } => {
                    warn!("config.yml error: {}", error.message);

                    // Increment rejection metric (§17.5)
                    metrics::metrics().hoop_config_reload_rejected_total.inc();

                    // Audit the rejected reload (§17.4)
                    let config_path = dirs::home_dir()
                        .unwrap_or_else(|| PathBuf::from("."))
                        .join(".hoop")
                        .join("config.yml");
                    let audit_args = config_watcher::ConfigReloadRejectedAudit {
                        file: config_path.display().to_string(),
                        prev_hash: prev_hash.clone(),
                        error: error.message.clone(),
                        actor: "system:hot-reload".to_string(),
                    };
                    if let Ok(args_json) = serde_json::to_string(&audit_args) {
                        if let Err(e) = fleet::write_audit_row(
                            "system:hot-reload",
                            fleet::ActionKind::ConfigReloadRejected,
                            &audit_args.file,
                            None,
                            Some(args_json),
                            fleet::ActionResult::Failure,
                            Some(error.message.clone()),
                            None,
                            None,
                            None,
                        ) {
                            warn!("Failed to write config.yml reject audit row: {}", e);
                        }
                    }

                    // Broadcast config error with structured details (§17.5)
                    let _ = config_tx_for_config.send(ws::ConfigStatusData {
                        valid: false,
                        error: Some(ws::ConfigErrorData {
                            message: error.message.clone(),
                            line: error.line,
                            col: error.col,
                            field: error.field.clone(),
                            expected: error.expected.clone(),
                            got: error.got.clone(),
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
    let ack_monitor_clone = worker_ack_monitor.clone();
    let beads_for_hb = beads.clone();
    tokio::spawn(async move {
        use heartbeats::MonitorEvent;
        let mut rx = registry_clone.subscribe();
        while let Ok(event) = rx.recv().await {
            match event {
                MonitorEvent::Heartbeat(hb) => {
                    // Notify ack monitor so it can track first-seen and fire §M5 alerts.
                    ack_monitor_clone.on_heartbeat(&hb.worker, hb.ts);
                    let liveness = registry_clone
                        .snapshot()
                        .await
                        .iter()
                        .find(|w| w.worker == hb.worker)
                        .map(|w| w.liveness)
                        .unwrap_or(heartbeats::WorkerLiveness::Dead);
                    // Advance last_heartbeat_at for the project this worker is executing
                    if let WorkerState::Executing { ref bead, .. } = hb.state {
                        let proj = {
                            let guard = beads_for_hb.read().unwrap();
                            guard.iter().find(|b| b.id == *bead).map(|b| b.project.clone())
                        };
                        if let Some(ref p) = proj {
                            if let Err(e) = fleet::touch_project_heartbeat_at(p, &hb.ts.to_rfc3339()) {
                                warn!("fleet: touch_project_heartbeat_at failed: {}", e);
                            }
                        }
                    }
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

    // Initialize pattern saved query synced broadcast channel (§4.7)
    let (pattern_tx, _) = broadcast::channel::<ws::PatternSavedQuerySyncedData>(64);

    // Load HoopConfig from config.yml for redaction policy state (§18.5)
    async fn load_hoop_config_for_redaction() -> hoop_schema::HoopConfig {
        let fallback_config = hoop_schema::HoopConfig {
            schema_version: hoop_schema::HoopConfigSchemaVersion::default(),
            agent: None,
            projects_file: None,
            backup: None,
            ui: None,
            voice: None,
            agent_extensions: None,
            metrics: None,
            audit: None,
            reflection: None,
            pricing: None,
            redaction: None,
            server: None,
        };

        tokio::task::spawn_blocking(|| {
            use std::path::PathBuf;

            let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            let config_path = home.join(".hoop").join("config.yml");

            let minimal_config = hoop_schema::HoopConfig {
                schema_version: hoop_schema::HoopConfigSchemaVersion::default(),
                agent: None,
                projects_file: None,
                backup: None,
                ui: None,
                voice: None,
                agent_extensions: None,
                metrics: None,
                audit: None,
                reflection: None,
                pricing: None,
                redaction: None,
                server: None,
            };

            if !config_path.exists() {
                tracing::debug!("config.yml not found, using minimal config for redaction policy");
                return minimal_config;
            }

            match std::fs::read_to_string(&config_path) {
                Ok(contents) => {
                    match serde_yaml::from_str::<hoop_schema::HoopConfig>(&contents) {
                        Ok(config) => config,
                        Err(e) => {
                            tracing::warn!("Failed to parse config.yml for redaction policy: {}, using minimal config", e);
                            minimal_config
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to read config.yml for redaction policy: {}, using minimal config", e);
                    minimal_config
                }
            }
        }).await.unwrap_or(fallback_config)
    }

    // Initialize redaction policy state (§18.5)
    let hoop_config = load_hoop_config_for_redaction().await;
    let redaction_policy_state = Arc::new(tokio::sync::RwLock::new(
        redaction_policy::RedactionPolicyState::new(&hoop_config, initial_config.registry.clone())
    ));
    info!("Redaction policy state initialized");

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
        ws_connection_tracker: Arc::new(ws::WsConnectionTracker::new()),
        worker_ack_monitor,
        collision_alert_tx: broadcast::channel::<ws::CollisionAlertData>(64).0,
        pattern_tx,
        redaction_policy_state,
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

    // Orphan bead count updater: update metrics on stitch/bead events
    {
        let projects_ref = state.projects.clone();
        let mut stitch_rx = state.stitch_tx.subscribe();
        let mut bead_rx = state.bead_tx.subscribe();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Update orphan metrics on stitch events (bead created/updated via HOOP)
                    result = stitch_rx.recv() => {
                        if result.is_ok() {
                            let projects = projects_ref.read().unwrap().clone();
                            tokio::task::spawn_blocking(move || {
                                crate::orphan_beads::update_all_orphan_metrics(&projects);
                            }).await.ok();
                        }
                    }
                    // Update orphan metrics on bead events (external bead changes)
                    result = bead_rx.recv() => {
                        if result.is_ok() {
                            let projects = projects_ref.read().unwrap().clone();
                            tokio::task::spawn_blocking(move || {
                                crate::orphan_beads::update_all_orphan_metrics(&projects);
                            }).await.ok();
                        }
                    }
                }
            }
        });
        info!("Orphan metrics updater started (listens to bead/Stitch events)");
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

                // Flush project_status to fleet.db (§4.4)
                {
                    let now = chrono::Utc::now().to_rfc3339();
                    for r in &runtime_statuses {
                        let open_beads = all_beads.iter()
                            .filter(|b| b.project == r.project_name && b.status == BeadStatus::Open)
                            .count() as i64;
                        let closed_beads = all_beads.iter()
                            .filter(|b| b.project == r.project_name && b.status != BeadStatus::Open)
                            .count() as i64;
                        let stuck_beads = workers.iter()
                            .filter(|w| matches!(w.state, ws::WorkerDisplayState::Knot { .. }))
                            .count() as i64;
                        let worker_count = workers.len() as i64;
                        let row = fleet::ProjectStatusRow {
                            project: r.project_name.clone(),
                            open_beads,
                            closed_beads,
                            stuck_beads,
                            worker_count,
                            last_event_at: None,
                            last_heartbeat_at: None,
                            updated_at: now.clone(),
                        };
                        // Partial upsert: only write bead/worker counts; preserve
                        // last_event_at and last_heartbeat_at that the event tailer set.
                        if let Err(e) = fleet::upsert_project_status_counts(&row) {
                            warn!("fleet: upsert_project_status_counts failed: {}", e);
                        }
                    }
                }

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

    // Periodic cost_rollup snapshot to fleet.db (every 60 s) — §4.4
    {
        let cost_ref = cost_aggregator.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                let (project_rows, codex_rows) = {
                    let agg = cost_ref.read().unwrap();
                    (agg.get_project_date_rollup(), agg.get_codex_account_date_rollup())
                };
                for (project, date, cost_usd, input, output, cache_read, cache_write) in project_rows {
                    if let Err(e) = fleet::snapshot_project_cost_row(
                        &project, &date, cost_usd, input, output, cache_read, cache_write,
                    ) {
                        warn!("fleet: snapshot_project_cost_row failed for {}/{}: {}", project, date, e);
                    }
                }
                for (account_id, date, plan_tier, cost_usd, input, output) in codex_rows {
                    if let Err(e) = fleet::snapshot_codex_account_daily_spend(
                        &account_id, &date, &plan_tier, cost_usd, input, output,
                    ) {
                        warn!("fleet: snapshot_codex_account_daily_spend failed for {}/{}: {}", account_id, date, e);
                    }
                }
            }
        });
    }

    // Capacity rollup → fleet.db: persist every capacity refresh broadcast (§4.4)
    {
        let mut cap_rx = state.capacity_tx.subscribe();
        tokio::spawn(async move {
            while let Ok(capacities) = cap_rx.recv().await {
                for cap in &capacities {
                    let row = fleet::CapacityRollupRow {
                        account_id: cap.account_id.clone(),
                        adapter: cap.adapter.clone(),
                        computed_at: cap.computed_at.to_rfc3339(),
                        window_5h_pct: cap.utilization_5h,
                        window_7d_pct: cap.utilization_7d,
                        tokens_5h: cap.tokens_5h as i64,
                        tokens_7d: cap.tokens_7d as i64,
                        cost_7d_usd: 0.0,
                        stitch_close_rate_per_min: cap.stitch_close_rate_per_min,
                        mean_cost_per_stitch_tokens: cap.mean_cost_per_stitch_tokens,
                        forecast_5h_stitch_min: cap.forecast_full_5h_stitch_min,
                        forecast_7d_stitch_min: cap.forecast_full_7d_stitch_min,
                    };
                    if let Err(e) = fleet::upsert_capacity_rollup(&row) {
                        warn!("fleet: upsert_capacity_rollup failed for {}: {}", cap.account_id, e);
                    }
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

#[cfg(test)]
mod dashboard_tests {
    use super::*;
    use chrono::Utc;

    fn make_bead(id: &str, title: &str, status: BeadStatus, project: &str, created_ago_secs: i64) -> Bead {
        let created_at = Utc::now() - chrono::Duration::seconds(created_ago_secs);
        Bead {
            id: id.to_string(),
            title: title.to_string(),
            description: None,
            status,
            priority: 0,
            issue_type: BeadType::Task,
            created_at,
            updated_at: created_at,
            created_by: "test".to_string(),
            dependencies: vec![],
            project: project.to_string(),
        }
    }

    #[test]
    fn test_longest_running_stitches_ordering() {
        let now = Utc::now();
        let beads = vec![
            make_bead("bd-001", "Short task", BeadStatus::Open, "proj-a", 60),
            make_bead("bd-002", "Long task", BeadStatus::Open, "proj-b", 3600),
            make_bead("bd-003", "Medium task", BeadStatus::Open, "proj-a", 600),
            make_bead("bd-004", "Closed task", BeadStatus::Closed, "proj-b", 7200),
        ];

        let mut open_beads: Vec<&Bead> = beads.iter().filter(|b| b.status == BeadStatus::Open).collect();
        open_beads.sort_by_key(|b| b.created_at);

        assert_eq!(open_beads.len(), 3);
        // Oldest (longest-running) first
        assert_eq!(open_beads[0].id, "bd-002");
        assert_eq!(open_beads[1].id, "bd-003");
        assert_eq!(open_beads[2].id, "bd-001");

        // Verify durations are positive
        for b in &open_beads {
            let dur = (now - b.created_at).num_seconds().max(0);
            assert!(dur > 0, "duration should be positive for open bead {}", b.id);
        }
    }

    #[test]
    fn test_longest_running_top_10_cap() {
        let beads: Vec<Bead> = (0..15)
            .map(|i| make_bead(&format!("bd-{:03}", i), &format!("task {}", i), BeadStatus::Open, "proj", (i as i64 + 1) * 60))
            .collect();

        let mut open_beads: Vec<&Bead> = beads.iter().filter(|b| b.status == BeadStatus::Open).collect();
        open_beads.sort_by_key(|b| b.created_at);

        let top10: Vec<_> = open_beads.iter().take(10).collect();
        assert_eq!(top10.len(), 10);
        // All should be open
        for b in top10 {
            assert_eq!(b.status, BeadStatus::Open);
        }
    }

    #[test]
    fn test_project_spend_sorted_descending() {
        let mut by_project: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
        by_project.insert("alpha".to_string(), 1.50);
        by_project.insert("beta".to_string(), 5.00);
        by_project.insert("gamma".to_string(), 0.25);

        let mut spend: Vec<ProjectSpend> = by_project
            .into_iter()
            .map(|(project, cost_usd)| ProjectSpend { project, cost_usd })
            .collect();
        spend.sort_by(|a, b| b.cost_usd.partial_cmp(&a.cost_usd).unwrap_or(std::cmp::Ordering::Equal));

        assert_eq!(spend[0].project, "beta");
        assert_eq!(spend[1].project, "alpha");
        assert_eq!(spend[2].project, "gamma");
    }

    #[test]
    fn test_adapter_spend_sorted_descending() {
        let mut by_adapter: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
        by_adapter.insert("claude".to_string(), 3.00);
        by_adapter.insert("codex".to_string(), 7.50);

        let mut spend: Vec<AdapterSpend> = by_adapter
            .into_iter()
            .map(|(adapter, cost_usd)| AdapterSpend { adapter, cost_usd })
            .collect();
        spend.sort_by(|a, b| b.cost_usd.partial_cmp(&a.cost_usd).unwrap_or(std::cmp::Ordering::Equal));

        assert_eq!(spend[0].adapter, "codex");
        assert_eq!(spend[1].adapter, "claude");
    }

    #[test]
    fn test_range_label_today() {
        let label = match "today" {
            "week" => "Last 7 days",
            "month" => "This month",
            _ => "Today",
        };
        assert_eq!(label, "Today");
    }

    #[test]
    fn test_range_label_week() {
        let label = match "week" {
            "week" => "Last 7 days",
            "month" => "This month",
            _ => "Today",
        };
        assert_eq!(label, "Last 7 days");
    }

    #[test]
    fn test_range_label_month() {
        let label = match "month" {
            "week" => "Last 7 days",
            "month" => "This month",
            _ => "Today",
        };
        assert_eq!(label, "This month");
    }

    #[test]
    fn test_total_spend_aggregation() {
        let mut by_project: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
        by_project.insert("alpha".to_string(), 1.50);
        by_project.insert("beta".to_string(), 5.00);
        by_project.insert("gamma".to_string(), 0.25);

        let total: f64 = by_project.values().sum();
        assert!((total - 6.75).abs() < 1e-9, "total should be 6.75, got {}", total);
    }

    #[test]
    fn test_closed_beads_excluded_from_longest_running() {
        let beads = vec![
            make_bead("bd-open", "Open", BeadStatus::Open, "proj", 300),
            make_bead("bd-closed", "Closed", BeadStatus::Closed, "proj", 9999),
        ];

        let open: Vec<&Bead> = beads.iter().filter(|b| b.status == BeadStatus::Open).collect();
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].id, "bd-open");
    }
}

/// Load HoopConfig from config.yml for redaction policy state.
///
/// Returns the parsed HoopConfig or a minimal config if the file doesn't exist
/// or fails to parse. This is used during daemon initialization.
async fn load_hoop_config() -> Option<hoop_schema::HoopConfig> {
    tokio::task::spawn_blocking(|| {
        use std::path::PathBuf;

        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let config_path = home.join(".hoop").join("config.yml");

        let minimal_config = hoop_schema::HoopConfig {
            schema_version: hoop_schema::HoopConfigSchemaVersion::default(),
            agent: None,
            projects_file: None,
            backup: None,
            ui: None,
            voice: None,
            agent_extensions: None,
            metrics: None,
            audit: None,
            reflection: None,
            pricing: None,
            redaction: None,
            server: None,
        };

        if !config_path.exists() {
            tracing::debug!("config.yml not found, using minimal config for redaction policy");
            return Some(minimal_config);
        }

        match std::fs::read_to_string(&config_path) {
            Ok(contents) => {
                match serde_yaml::from_str::<hoop_schema::HoopConfig>(&contents) {
                    Ok(config) => Some(config),
                    Err(e) => {
                        tracing::warn!("Failed to parse config.yml for redaction policy: {}, using minimal config", e);
                        Some(minimal_config)
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to read config.yml for redaction policy: {}, using minimal config", e);
                Some(minimal_config)
            }
        }
    }).await.ok().flatten()
}

/// Find the Stitch ID for a given bead ID.
///
/// Queries fleet.db to find which Stitch (if any) a bead belongs to.
/// Returns None if the bead is not linked to any Stitch.
async fn find_stitch_for_bead(bead_id: &str) -> Option<String> {
    let bead_id = bead_id.to_string();
    tokio::task::spawn_blocking(move || {
        use rusqlite::Connection;

        let db_path = std::path::PathBuf::from(
            dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."))
        ).join(".hoop").join("fleet.db");

        if !db_path.exists() {
            return None;
        }

        let conn = Connection::open(&db_path).ok()?;

        conn.query_row(
            "SELECT stitch_id FROM stitch_beads WHERE bead_id = ?1 LIMIT 1",
            rusqlite::params![&bead_id],
            |row| row.get(0),
        ).ok()
    })
    .await
    .ok()
    .flatten()
}
