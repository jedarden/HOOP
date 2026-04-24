//! Prometheus `/metrics` endpoint and `/debug/state` diagnostic endpoint.
//!
//! # `/metrics`
//! Emits all catalogued metrics in Prometheus text exposition format
//! (Content-Type: `text/plain; version=0.0.4`).  The response combines
//! accumulated metrics from [`metrics::metrics()`] with scrape-time
//! values computed from [`DaemonState`] (uptime, process stats, DB sizes,
//! worker liveness).
//!
//! # `/debug/state`
//! Returns a JSON snapshot of live daemon state for incident triage.
//! Intended for local / trusted-network use only.

use std::path::Path;
use std::time::Instant;

use axum::{
    extract::State,
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use serde::Serialize;

use crate::{heartbeats::WorkerLiveness, metrics, ws::WorkerDisplayState, BeadStatus, DaemonState};

// ---------------------------------------------------------------------------
// HTTP metrics middleware
// ---------------------------------------------------------------------------

/// Axum middleware that records per-request counters and duration histograms.
///
/// Add to the router with:
/// ```ignore
/// .layer(axum::middleware::from_fn(api_metrics::http_metrics_middleware))
/// ```
/// The route label is derived from the request path and mapped to a bounded
/// set of known prefixes to prevent cardinality explosion.
pub async fn http_metrics_middleware(
    req: axum::http::Request<axum::body::Body>,
    next: Next,
) -> axum::response::Response {
    let route = categorize_path(req.uri().path());
    let start = Instant::now();

    let response = next.run(req).await;

    let status = response.status().as_u16().to_string();
    let elapsed_ms = start.elapsed().as_secs_f64() * 1_000.0;

    metrics::metrics()
        .hoop_http_requests_total
        .inc(&[route, &status]);
    metrics::metrics()
        .hoop_http_request_duration_ms
        .observe(&[route], elapsed_ms);

    response
}

/// Map a raw request path to one of a fixed set of route-label strings.
/// This keeps `hoop_http_requests_total` cardinality bounded.
fn categorize_path(path: &str) -> &'static str {
    if path == "/metrics" {
        "/metrics"
    } else if path == "/healthz" {
        "/healthz"
    } else if path == "/readyz" {
        "/readyz"
    } else if path == "/debug/state" {
        "/debug/state"
    } else if path == "/ws" {
        "/ws"
    } else if path.starts_with("/api/beads") {
        "/api/beads"
    } else if path.starts_with("/api/cost") {
        "/api/cost"
    } else if path.starts_with("/api/capacity") {
        "/api/capacity"
    } else if path.starts_with("/api/projects") {
        "/api/projects"
    } else if path.starts_with("/api/audit") {
        "/api/audit"
    } else if path.starts_with("/api/preview") {
        "/api/preview"
    } else if path.starts_with("/api/stitch") {
        "/api/stitch"
    } else if path.starts_with("/api/agent") {
        "/api/agent"
    } else if path.starts_with("/api/uploads") {
        "/api/uploads"
    } else if path.starts_with("/api/transcription") {
        "/api/transcription"
    } else if path.starts_with("/api/dictated-notes") {
        "/api/dictated-notes"
    } else if path.starts_with("/api/attachments") {
        "/api/attachments"
    } else if path.starts_with("/assets") {
        "/assets"
    } else {
        "/other"
    }
}

// ---------------------------------------------------------------------------
// /metrics handler
// ---------------------------------------------------------------------------

async fn get_metrics(State(state): State<DaemonState>) -> impl IntoResponse {
    let mut body = metrics::metrics().render_text();
    append_scrape_time_metrics(&mut body, &state).await;
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        body,
    )
}

/// Append metrics that are cheapest to compute fresh at each scrape rather
/// than being maintained as running accumulators.
async fn append_scrape_time_metrics(out: &mut String, state: &DaemonState) {
    // ── §16.1 Operational: uptime ──────────────────────────────────────────
    push_gauge_u64(
        out,
        "hoop_uptime_seconds",
        "Seconds elapsed since the daemon started.",
        state.started_at.elapsed().as_secs(),
    );

    // ── §16.1 Operational: process stats (Linux /proc) ────────────────────
    let (mem_bytes, open_fds, tasks) = read_proc_stats();
    push_gauge_u64(
        out,
        "hoop_process_memory_bytes",
        "Resident set size of the daemon process in bytes.",
        mem_bytes,
    );
    push_gauge_u64(
        out,
        "hoop_process_open_fds",
        "Number of open file descriptors.",
        open_fds,
    );
    push_gauge_u64(
        out,
        "hoop_process_tasks_total",
        "Number of OS threads in the daemon process.",
        tasks,
    );

    // ── §16.2 Heartbeat freshness per worker ───────────────────────────────
    let workers = state.worker_registry.snapshot().await;

    out.push_str(
        "# HELP hoop_heartbeat_freshness_seconds Seconds since each worker last sent a heartbeat.\n",
    );
    out.push_str("# TYPE hoop_heartbeat_freshness_seconds gauge\n");
    let now = chrono::Utc::now();
    for w in &workers {
        let age_secs = (now - w.last_heartbeat).num_seconds().max(0);
        let escaped = w.worker.replace('\\', "\\\\").replace('"', "\\\"");
        out.push_str(&format!(
            "hoop_heartbeat_freshness_seconds{{worker=\"{escaped}\"}} {age_secs}\n"
        ));
    }

    // Worker liveness tallies
    let live = workers
        .iter()
        .filter(|w| matches!(w.liveness, WorkerLiveness::Live))
        .count() as u64;
    let hung = workers
        .iter()
        .filter(|w| matches!(w.liveness, WorkerLiveness::Hung))
        .count() as u64;
    let dead = workers
        .iter()
        .filter(|w| matches!(w.liveness, WorkerLiveness::Dead))
        .count() as u64;
    let stuck = workers
        .iter()
        .filter(|w| matches!(w.state, WorkerDisplayState::Knot { .. }))
        .count() as u64;

    push_gauge_u64(out, "hoop_workers_live", "Workers with a fresh heartbeat.", live);
    push_gauge_u64(out, "hoop_workers_hung", "Workers whose heartbeat is stale.", hung);
    push_gauge_u64(out, "hoop_workers_dead", "Workers presumed dead.", dead);
    push_gauge_u64(
        out,
        "hoop_workers_stuck",
        "Workers currently in a knot (stuck) state.",
        stuck,
    );
    push_gauge_u64(
        out,
        "hoop_agent_workers_total",
        "Total registered agent worker processes.",
        workers.len() as u64,
    );

    // ── §16.4 Open Stitch / bead counts ───────────────────────────────────
    let (open_count, total_count) = {
        let beads = state.beads.read().unwrap();
        let open = beads.iter().filter(|b| b.status == BeadStatus::Open).count() as u64;
        (open, beads.len() as u64)
    };
    push_gauge_u64(out, "hoop_open_stitches", "Number of currently open Stitches.", open_count);
    push_gauge_u64(out, "hoop_total_beads", "Total beads known to the daemon.", total_count);

    // ── §16.6 Storage: fleet.db sizes ─────────────────────────────────────
    let fleet_db = crate::fleet::db_path();
    let db_bytes = file_size_bytes(&fleet_db);
    let wal_bytes = file_size_bytes(&wal_path(&fleet_db));

    push_gauge_u64(out, "hoop_fleet_db_size_bytes", "fleet.db file size in bytes.", db_bytes);
    push_gauge_u64(
        out,
        "hoop_fleet_db_wal_size_bytes",
        "fleet.db WAL file size in bytes.",
        wal_bytes,
    );

    // ── §16.6 Storage: attachments directory ──────────────────────────────
    let attachments_dir = dirs::home_dir()
        .unwrap_or_default()
        .join(".hoop")
        .join("attachments");
    push_gauge_u64(
        out,
        "hoop_attachments_size_bytes",
        "Total size of all stored attachment files in bytes.",
        dir_size_bytes(&attachments_dir),
    );

    // ── §16.7 Business: total cost today across all projects ───────────────
    let cost_today: f64 = {
        let agg = state.cost_aggregator.read().unwrap();
        state
            .projects
            .read()
            .unwrap()
            .iter()
            .map(|p| agg.cost_today_for_project(&p.name))
            .sum()
    };
    push_gauge_f64(
        out,
        "hoop_cost_today_usd",
        "Total agent cost for the current UTC day across all projects, in USD.",
        cost_today,
    );
}

// ---------------------------------------------------------------------------
// /debug/state handler
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct DebugStateResponse {
    uptime_secs: u64,
    workers: Vec<WorkerSnapshot>,
    open_stitches: usize,
    total_beads: usize,
    projects: Vec<String>,
    fleet_db_path: String,
    fleet_db_size_bytes: u64,
    fleet_db_wal_size_bytes: u64,
    bind_addr: String,
    ws_connections: i64,
}

#[derive(Serialize)]
struct WorkerSnapshot {
    name: String,
    state: String,
    liveness: String,
    last_heartbeat: String,
    heartbeat_age_secs: i64,
}

async fn debug_state(State(state): State<DaemonState>) -> Json<DebugStateResponse> {
    let workers = state.worker_registry.snapshot().await;

    let fleet_db = crate::fleet::db_path();
    let fleet_db_size_bytes = file_size_bytes(&fleet_db);
    let fleet_db_wal_size_bytes = file_size_bytes(&wal_path(&fleet_db));

    let (open_stitches, total_beads) = {
        let beads = state.beads.read().unwrap();
        (
            beads.iter().filter(|b| b.status == BeadStatus::Open).count(),
            beads.len(),
        )
    };

    Json(DebugStateResponse {
        uptime_secs: state.started_at.elapsed().as_secs(),
        workers: workers
            .iter()
            .map(|w| WorkerSnapshot {
                name: w.worker.clone(),
                state: format!("{:?}", w.state),
                liveness: format!("{:?}", w.liveness),
                last_heartbeat: w.last_heartbeat.to_rfc3339(),
                heartbeat_age_secs: w.heartbeat_age_secs,
            })
            .collect(),
        open_stitches,
        total_beads,
        projects: state
            .projects
            .read()
            .unwrap()
            .iter()
            .map(|p| p.name.clone())
            .collect(),
        fleet_db_path: fleet_db.display().to_string(),
        fleet_db_size_bytes,
        fleet_db_wal_size_bytes,
        bind_addr: state.config.bind_addr.to_string(),
        ws_connections: metrics::metrics().hoop_ws_clients_connected.get(),
    })
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<DaemonState> {
    Router::new()
        .route("/metrics", get(get_metrics))
        .route("/debug/state", get(debug_state))
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn push_gauge_u64(out: &mut String, name: &str, help: &str, value: u64) {
    out.push_str(&format!(
        "# HELP {name} {help}\n# TYPE {name} gauge\n{name} {value}\n"
    ));
}

fn push_gauge_f64(out: &mut String, name: &str, help: &str, value: f64) {
    out.push_str(&format!(
        "# HELP {name} {help}\n# TYPE {name} gauge\n{name} {value:.6}\n"
    ));
}

/// Read resident-set size, open-fd count, and thread count from
/// `/proc/self/`.  Returns `(0, 0, 0)` on non-Linux or on any I/O error.
fn read_proc_stats() -> (u64, u64, u64) {
    let mem_bytes = std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("VmRSS:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|v| v.parse::<u64>().ok())
        })
        .map(|kb| kb * 1024)
        .unwrap_or(0);

    let open_fds = std::fs::read_dir("/proc/self/fd")
        .map(|e| e.count() as u64)
        .unwrap_or(0);

    let tasks = std::fs::read_dir("/proc/self/task")
        .map(|e| e.count() as u64)
        .unwrap_or(0);

    (mem_bytes, open_fds, tasks)
}

/// Return the size of a single file in bytes, or 0 if it does not exist.
fn file_size_bytes(path: &Path) -> u64 {
    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

/// Compute the SQLite WAL path for `db_path` (appends `-wal` to the extension).
/// For `fleet.db` this produces `fleet.db-wal`.
fn wal_path(db_path: &Path) -> std::path::PathBuf {
    let ext = db_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("db");
    let mut p = db_path.to_path_buf();
    p.set_extension(format!("{ext}-wal"));
    p
}

/// Recursively sum the byte size of all regular files under `dir`.
/// Returns 0 if `dir` does not exist or is not readable.
fn dir_size_bytes(dir: &Path) -> u64 {
    if !dir.is_dir() {
        return 0;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    entries.flatten().fold(0u64, |acc, entry| {
        let path = entry.path();
        if path.is_dir() {
            acc + dir_size_bytes(&path)
        } else {
            acc + file_size_bytes(&path)
        }
    })
}
