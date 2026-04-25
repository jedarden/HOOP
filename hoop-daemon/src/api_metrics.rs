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

/// Debug-state payload schema version (§20: bump on any field addition/removal).
const DEBUG_STATE_SCHEMA_VERSION: &str = "1.0.0";

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
    } else if path.starts_with("/api/diagnostics") {
        "/api/diagnostics"
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

    // ── §L3 JSONL quarantine: today's count ────────────────────────────────
    push_gauge_u64(
        out,
        "hoop_quarantine_today_total",
        "Number of JSONL lines quarantined in the current UTC day.",
        crate::parse_jsonl_safe::quarantine_today_count(),
    );
}

// ---------------------------------------------------------------------------
// /debug/state handler
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct DebugStateResponse {
    schema_version: String,
    uptime_secs: u64,
    version: VersionInfo,
    config_hash: String,
    bind_addr: String,
    workers: Vec<WorkerSnapshot>,
    worker_pids: Vec<WorkerPidEntry>,
    active_claims: Vec<ClaimEntry>,
    ws_clients: Vec<WsClientEntry>,
    session_alias_table: Vec<SessionAliasEntry>,
    backup_timestamps: BackupTimestamps,
    fleet_db_path: String,
    fleet_db_size_bytes: u64,
    fleet_db_wal_size_bytes: u64,
    open_stitches: usize,
    total_beads: usize,
    projects: Vec<String>,
}

#[derive(Serialize)]
struct VersionInfo {
    daemon: String,
    schema: String,
}

#[derive(Serialize)]
struct WorkerSnapshot {
    name: String,
    state: String,
    liveness: String,
    last_heartbeat: String,
    heartbeat_age_secs: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pid: Option<u32>,
}

#[derive(Serialize)]
struct WorkerPidEntry {
    worker: String,
    pid: u32,
}

#[derive(Serialize)]
struct ClaimEntry {
    worker: String,
    bead: String,
    adapter: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pid: Option<u32>,
}

#[derive(Serialize)]
struct WsClientEntry {
    conn_id: u64,
    connected_at: String,
    connected_secs: i64,
}

#[derive(Serialize)]
struct SessionAliasEntry {
    session_id: String,
    provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    worker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bead: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    strand: Option<String>,
}

#[derive(Serialize)]
struct BackupTimestamps {
    last_success_unix: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_success_iso: Option<String>,
    last_size_bytes: i64,
}

async fn debug_state(State(state): State<DaemonState>) -> Json<DebugStateResponse> {
    let workers = state.worker_registry.snapshot().await;
    let worker_pids = state.worker_registry.worker_pids_snapshot().await;

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

    // Config hash: SHA-256 of the serialised resolved config
    let config_hash = {
        let config_json = serde_json::to_string(&*state.resolved_config).unwrap_or_default();
        sha256_hex(&config_json)
    };

    // Worker snapshots with PID when available
    let worker_snapshots: Vec<WorkerSnapshot> = workers
        .iter()
        .map(|w| {
            let pid = if let WorkerDisplayState::Executing { .. } = &w.state {
                worker_pids.get(&w.worker).copied()
            } else {
                None
            };
            WorkerSnapshot {
                name: w.worker.clone(),
                state: format!("{:?}", w.state),
                liveness: format!("{:?}", w.liveness),
                last_heartbeat: w.last_heartbeat.to_rfc3339(),
                heartbeat_age_secs: w.heartbeat_age_secs,
                pid,
            }
        })
        .collect();

    // Every worker PID observed
    let worker_pid_entries: Vec<WorkerPidEntry> = worker_pids
        .iter()
        .map(|(worker, pid)| WorkerPidEntry {
            worker: worker.clone(),
            pid: *pid,
        })
        .collect();

    // Active claims: workers currently executing
    let active_claims: Vec<ClaimEntry> = workers
        .iter()
        .filter_map(|w| match &w.state {
            WorkerDisplayState::Executing { bead, adapter, .. } => Some(ClaimEntry {
                worker: w.worker.clone(),
                bead: bead.clone(),
                adapter: adapter.clone(),
                pid: worker_pids.get(&w.worker).copied(),
            }),
            _ => None,
        })
        .collect();

    // WS clients from the connection tracker
    let ws_clients: Vec<WsClientEntry> = state
        .ws_connection_tracker
        .snapshot()
        .into_iter()
        .map(|c| WsClientEntry {
            conn_id: c.conn_id,
            connected_at: c.connected_at.to_rfc3339(),
            connected_secs: c.connected_secs,
        })
        .collect();

    // Session alias table from conversations
    let session_alias_table: Vec<SessionAliasEntry> = {
        let convos = state.worker_registry.conversations_snapshot().await;
        convos
            .iter()
            .filter_map(|c| {
                let (worker, bead, strand) = match &c.worker_metadata {
                    Some(meta) => (Some(meta.worker.clone()), Some(meta.bead.clone()), meta.strand.clone()),
                    None => (None, None, None),
                };
                // Only include sessions that have a worker binding (the alias table
                // maps CLI session IDs to worker/bead identities).
                if worker.is_some() || bead.is_some() {
                    Some(SessionAliasEntry {
                        session_id: c.session_id.clone(),
                        provider: c.provider.clone(),
                        worker,
                        bead,
                        strand,
                    })
                } else {
                    None
                }
            })
            .collect()
    };

    // Backup timestamps from metrics
    let m = metrics::metrics();
    let backup_ts = m.hoop_backup_last_success_timestamp.get();
    let backup_size = m.hoop_backup_last_size_bytes.get();
    let backup_timestamps = BackupTimestamps {
        last_success_unix: backup_ts,
        last_success_iso: if backup_ts > 0 {
            Some(chrono::DateTime::from_timestamp(backup_ts, 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default())
        } else {
            None
        },
        last_size_bytes: if backup_ts > 0 { backup_size } else { 0 },
    };

    Json(DebugStateResponse {
        schema_version: DEBUG_STATE_SCHEMA_VERSION.to_string(),
        uptime_secs: state.started_at.elapsed().as_secs(),
        version: VersionInfo {
            daemon: env!("CARGO_PKG_VERSION").to_string(),
            schema: hoop_schema::version::SCHEMA_VERSION.to_string(),
        },
        config_hash,
        bind_addr: state.config.bind_addr.to_string(),
        workers: worker_snapshots,
        worker_pids: worker_pid_entries,
        active_claims,
        ws_clients,
        session_alias_table,
        backup_timestamps,
        fleet_db_path: fleet_db.display().to_string(),
        fleet_db_size_bytes,
        fleet_db_wal_size_bytes,
        open_stitches,
        total_beads,
        projects: state
            .projects
            .read()
            .unwrap()
            .iter()
            .map(|p| p.name.clone())
            .collect(),
    })
}

/// Compute a SHA-256 hex digest of the input string.
fn sha256_hex(input: &str) -> String {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(input.as_bytes());
    format!("{hash:x}")
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

/// Response for GET /api/diagnostics/unknown-events
#[derive(Serialize)]
struct UnknownEventsResponse {
    total_count: u64,
    labeled_totals: Vec<LabeledEntry>,
    daemon_version: String,
    schema_version: String,
}

#[derive(Serialize)]
struct LabeledEntry {
    adapter: String,
    event_kind: String,
    count: u64,
}

async fn get_unknown_events() -> Json<UnknownEventsResponse> {
    let m = metrics::metrics();
    let labeled = m.hoop_unknown_event_labeled_total.snapshot();
    let labeled_totals = labeled
        .into_iter()
        .filter_map(|(labels, count)| {
            let adapter = labels.first().map(|s| s.as_str()).unwrap_or("unknown");
            let event_kind = labels.get(1).map(|s| s.as_str()).unwrap_or("unknown");
            Some(LabeledEntry {
                adapter: adapter.to_string(),
                event_kind: event_kind.to_string(),
                count,
            })
        })
        .collect();

    Json(UnknownEventsResponse {
        total_count: m.hoop_unknown_event_total.get(),
        labeled_totals,
        daemon_version: env!("CARGO_PKG_VERSION").to_string(),
        schema_version: hoop_schema::version::SCHEMA_VERSION.to_string(),
    })
}

pub fn router() -> Router<DaemonState> {
    Router::new()
        .route("/metrics", get(get_metrics))
        .route("/debug/state", get(debug_state))
        .route("/api/diagnostics/unknown-events", get(get_unknown_events))
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    /// Build a fixture DebugStateResponse with known values, serialize it,
    /// and verify every required field is present and correctly populated.
    #[test]
    fn debug_state_fixture_all_fields_present() {
        let response = DebugStateResponse {
            schema_version: DEBUG_STATE_SCHEMA_VERSION.to_string(),
            uptime_secs: 42,
            version: VersionInfo {
                daemon: "0.1.0".to_string(),
                schema: "1.0.0".to_string(),
            },
            config_hash: "ab".repeat(32), // 64-char hex
            bind_addr: "127.0.0.1:3000".to_string(),
            workers: vec![WorkerSnapshot {
                name: "worker-alpha".to_string(),
                state: "Executing { bead: \"hoop-ttb.1\", adapter: \"claude\" }".to_string(),
                liveness: "Live".to_string(),
                last_heartbeat: "2024-01-01T00:00:00Z".to_string(),
                heartbeat_age_secs: 3,
                pid: Some(12345),
            }],
            worker_pids: vec![WorkerPidEntry {
                worker: "worker-alpha".to_string(),
                pid: 12345,
            }],
            active_claims: vec![ClaimEntry {
                worker: "worker-alpha".to_string(),
                bead: "hoop-ttb.1".to_string(),
                adapter: "claude".to_string(),
                pid: Some(12345),
            }],
            ws_clients: vec![WsClientEntry {
                conn_id: 1,
                connected_at: "2024-01-01T00:00:00Z".to_string(),
                connected_secs: 10,
            }],
            session_alias_table: vec![SessionAliasEntry {
                session_id: "sess-abc".to_string(),
                provider: "claude".to_string(),
                worker: Some("worker-alpha".to_string()),
                bead: Some("hoop-ttb.1".to_string()),
                strand: None,
            }],
            backup_timestamps: BackupTimestamps {
                last_success_unix: 1704067200,
                last_success_iso: Some("2024-01-01T00:00:00+00:00".to_string()),
                last_size_bytes: 2048,
            },
            fleet_db_path: "/home/user/.hoop/fleet.db".to_string(),
            fleet_db_size_bytes: 4096,
            fleet_db_wal_size_bytes: 512,
            open_stitches: 3,
            total_beads: 10,
            projects: vec!["hoop".to_string(), "other".to_string()],
        };

        let json: Value = serde_json::to_value(&response).expect("serialize");

        // ── Top-level required fields ────────────────────────────────────────
        assert_eq!(json["schema_version"], DEBUG_STATE_SCHEMA_VERSION);
        assert_eq!(json["uptime_secs"], 42);
        assert_eq!(json["bind_addr"], "127.0.0.1:3000");
        assert_eq!(json["config_hash"], "ab".repeat(32));
        assert_eq!(json["open_stitches"], 3);
        assert_eq!(json["total_beads"], 10);
        assert_eq!(json["fleet_db_path"], "/home/user/.hoop/fleet.db");
        assert_eq!(json["fleet_db_size_bytes"], 4096);
        assert_eq!(json["fleet_db_wal_size_bytes"], 512);

        // ── version object ───────────────────────────────────────────────────
        let ver = &json["version"];
        assert_eq!(ver["daemon"], "0.1.0");
        assert_eq!(ver["schema"], "1.0.0");

        // ── workers ──────────────────────────────────────────────────────────
        let workers = json["workers"].as_array().expect("workers array");
        assert_eq!(workers.len(), 1);
        assert_eq!(workers[0]["name"], "worker-alpha");
        assert_eq!(workers[0]["state"], "Executing { bead: \"hoop-ttb.1\", adapter: \"claude\" }");
        assert_eq!(workers[0]["liveness"], "Live");
        assert_eq!(workers[0]["last_heartbeat"], "2024-01-01T00:00:00Z");
        assert_eq!(workers[0]["heartbeat_age_secs"], 3);
        assert_eq!(workers[0]["pid"], 12345);

        // ── worker_pids ──────────────────────────────────────────────────────
        let pids = json["worker_pids"].as_array().expect("worker_pids array");
        assert_eq!(pids.len(), 1);
        assert_eq!(pids[0]["worker"], "worker-alpha");
        assert_eq!(pids[0]["pid"], 12345);

        // ── active_claims ────────────────────────────────────────────────────
        let claims = json["active_claims"].as_array().expect("active_claims array");
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0]["worker"], "worker-alpha");
        assert_eq!(claims[0]["bead"], "hoop-ttb.1");
        assert_eq!(claims[0]["adapter"], "claude");
        assert_eq!(claims[0]["pid"], 12345);

        // ── ws_clients ───────────────────────────────────────────────────────
        let ws = json["ws_clients"].as_array().expect("ws_clients array");
        assert_eq!(ws.len(), 1);
        assert_eq!(ws[0]["conn_id"], 1);
        assert_eq!(ws[0]["connected_at"], "2024-01-01T00:00:00Z");
        assert_eq!(ws[0]["connected_secs"], 10);

        // ── session_alias_table ──────────────────────────────────────────────
        let aliases = json["session_alias_table"].as_array().expect("session_alias_table array");
        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0]["session_id"], "sess-abc");
        assert_eq!(aliases[0]["provider"], "claude");
        assert_eq!(aliases[0]["worker"], "worker-alpha");
        assert_eq!(aliases[0]["bead"], "hoop-ttb.1");
        assert!(aliases[0]["strand"].is_null());

        // ── backup_timestamps ────────────────────────────────────────────────
        let bt = &json["backup_timestamps"];
        assert_eq!(bt["last_success_unix"], 1704067200);
        assert_eq!(bt["last_success_iso"], "2024-01-01T00:00:00+00:00");
        assert_eq!(bt["last_size_bytes"], 2048);

        // ── projects ─────────────────────────────────────────────────────────
        let projects = json["projects"].as_array().expect("projects array");
        assert_eq!(projects.len(), 2);
        assert_eq!(projects[0], "hoop");
        assert_eq!(projects[1], "other");
    }

    /// Verify that optional fields are omitted when None (not serialized as null).
    #[test]
    fn debug_state_optional_fields_omitted_when_none() {
        let response = DebugStateResponse {
            schema_version: DEBUG_STATE_SCHEMA_VERSION.to_string(),
            uptime_secs: 0,
            version: VersionInfo {
                daemon: "0.1.0".to_string(),
                schema: "1.0.0".to_string(),
            },
            config_hash: String::new(),
            bind_addr: String::new(),
            workers: vec![WorkerSnapshot {
                name: "idle-worker".to_string(),
                state: "Idle".to_string(),
                liveness: "Live".to_string(),
                last_heartbeat: "2024-01-01T00:00:00Z".to_string(),
                heartbeat_age_secs: 0,
                pid: None, // None → should be omitted
            }],
            worker_pids: vec![],
            active_claims: vec![ClaimEntry {
                worker: "w".to_string(),
                bead: "b".to_string(),
                adapter: "a".to_string(),
                pid: None, // None → should be omitted
            }],
            ws_clients: vec![],
            session_alias_table: vec![SessionAliasEntry {
                session_id: "s".to_string(),
                provider: "p".to_string(),
                worker: None,
                bead: None,
                strand: None,
            }],
            backup_timestamps: BackupTimestamps {
                last_success_unix: 0,
                last_success_iso: None, // None → should be omitted
                last_size_bytes: 0,
            },
            fleet_db_path: String::new(),
            fleet_db_size_bytes: 0,
            fleet_db_wal_size_bytes: 0,
            open_stitches: 0,
            total_beads: 0,
            projects: vec![],
        };

        let json: Value = serde_json::to_value(&response).expect("serialize");

        // Worker pid omitted when None
        let worker = &json["workers"][0];
        assert!(worker.get("pid").is_none(), "pid should be omitted when None");

        // Claim pid omitted when None
        let claim = &json["active_claims"][0];
        assert!(claim.get("pid").is_none(), "claim pid should be omitted when None");

        // Session alias optional fields omitted
        let alias = &json["session_alias_table"][0];
        assert!(alias.get("worker").is_none());
        assert!(alias.get("bead").is_none());
        assert!(alias.get("strand").is_none());

        // Backup ISO timestamp omitted when never backed up
        assert!(json["backup_timestamps"].get("last_success_iso").is_none());
    }

    /// Verify schema_version follows semver pattern.
    #[test]
    fn debug_state_schema_version_is_semver() {
        let re = regex::Regex::new(r"^\d+\.\d+\.\d+$").unwrap();
        assert!(
            re.is_match(DEBUG_STATE_SCHEMA_VERSION),
            "DEBUG_STATE_SCHEMA_VERSION must be semver, got: {DEBUG_STATE_SCHEMA_VERSION}"
        );
    }

    /// Verify the handler constant matches the schema file's schema_version.
    /// This ensures any schema bump is reflected in the handler (§20).
    #[test]
    fn debug_state_schema_version_matches_schema_file() {
        let schema_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../hoop-schema/schemas/debug_state.json");
        let schema_text = std::fs::read_to_string(&schema_path)
            .unwrap_or_else(|e| panic!("failed to read {:?}: {e}", schema_path));
        let schema: Value = serde_json::from_str(&schema_text).expect("parse schema");
        let file_version = schema["schema_version"]
            .as_str()
            .expect("schema_version in debug_state.json");
        assert_eq!(
            DEBUG_STATE_SCHEMA_VERSION, file_version,
            "Handler constant must match schema file (§20). \
             If you added/removed a field, bump both."
        );
    }

    /// Validate the serialized fixture output against every constraint in the
    /// JSON schema: required fields present, types correct, patterns matched.
    #[test]
    fn debug_state_validates_against_json_schema() {
        let schema_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../hoop-schema/schemas/debug_state.json");
        let schema_text = std::fs::read_to_string(&schema_path).unwrap();
        let schema: Value = serde_json::from_str(&schema_text).unwrap();
        let required = schema["required"]
            .as_array()
            .expect("required array in schema")
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect::<Vec<_>>();

        // Build a response with all fields populated
        let response = DebugStateResponse {
            schema_version: DEBUG_STATE_SCHEMA_VERSION.to_string(),
            uptime_secs: 100,
            version: VersionInfo {
                daemon: "0.1.0".to_string(),
                schema: "1.0.0".to_string(),
            },
            config_hash: "deadbeef".repeat(8),
            bind_addr: "0.0.0.0:3000".to_string(),
            workers: vec![],
            worker_pids: vec![],
            active_claims: vec![],
            ws_clients: vec![],
            session_alias_table: vec![],
            backup_timestamps: BackupTimestamps {
                last_success_unix: 0,
                last_success_iso: None,
                last_size_bytes: 0,
            },
            fleet_db_path: "/tmp/fleet.db".to_string(),
            fleet_db_size_bytes: 0,
            fleet_db_wal_size_bytes: 0,
            open_stitches: 0,
            total_beads: 0,
            projects: vec![],
        };

        let json: Value = serde_json::to_value(&response).expect("serialize");

        // Check every required field is present
        for field in &required {
            assert!(
                json.get(field).is_some(),
                "required field '{field}' missing from DebugStateResponse"
            );
        }

        // Validate schema_version pattern
        let re = regex::Regex::new(r"^\d+\.\d+\.\d+$").unwrap();
        assert!(re.is_match(json["schema_version"].as_str().unwrap()));

        // Validate uptime_secs is a non-negative integer (u64 is always >= 0)
        assert!(json["uptime_secs"].is_number());

        // Validate version sub-object
        assert!(json["version"]["daemon"].is_string());
        assert!(json["version"]["schema"].is_string());

        // Validate config_hash is a string
        assert!(json["config_hash"].is_string());

        // Validate array fields are arrays
        for array_field in &[
            "workers", "worker_pids", "active_claims", "ws_clients",
            "session_alias_table", "projects",
        ] {
            assert!(json[array_field].is_array(), "{array_field} must be array");
        }

        // Validate integer fields
        for int_field in &[
            "fleet_db_size_bytes", "fleet_db_wal_size_bytes",
            "open_stitches", "total_beads",
        ] {
            assert!(json[int_field].is_number(), "{int_field} must be number");
        }
    }

    /// Verify the JSON schema and handler declare the exact same set of fields.
    /// This is bidirectional: no handler-only or schema-only fields allowed,
    /// preventing silent field additions/removals without the other being updated.
    #[test]
    fn schema_and_handler_fields_are_identical() {
        let schema_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../hoop-schema/schemas/debug_state.json");
        let schema_text = std::fs::read_to_string(&schema_path).unwrap();
        let schema: Value = serde_json::from_str(&schema_text).unwrap();
        let schema_props: std::collections::HashSet<String> = schema["properties"]
            .as_object()
            .expect("properties object")
            .keys()
            .cloned()
            .collect();

        let handler_fields: std::collections::HashSet<&str> = [
            "schema_version", "uptime_secs", "version", "config_hash",
            "bind_addr", "workers", "worker_pids", "active_claims",
            "ws_clients", "session_alias_table", "backup_timestamps",
            "fleet_db_path", "fleet_db_size_bytes", "fleet_db_wal_size_bytes",
            "open_stitches", "total_beads", "projects",
        ].into_iter().collect();

        // Handler → schema: every field the handler produces must be in the schema
        for field in &handler_fields {
            assert!(
                schema_props.contains(*field),
                "handler produces '{field}' but schema doesn't declare it — \
                 add it to debug_state.json and bump schema_version (§20)"
            );
        }

        // Schema → handler: every schema property must have a corresponding handler field
        for field in &schema_props {
            assert!(
                handler_fields.contains(field.as_str()),
                "schema declares '{field}' but handler doesn't produce it — \
                 either remove from schema or add to handler and bump schema_version (§20)"
            );
        }

        // Exact set equality
        assert_eq!(
            handler_fields.len(),
            schema_props.len(),
            "handler has {} fields, schema has {} — sets must be identical",
            handler_fields.len(),
            schema_props.len()
        );
    }

    /// Verify the schema has additionalProperties:false on all object types,
    /// enforcing §20 (field additions require schema bump).
    #[test]
    fn schema_enforces_no_additional_properties() {
        let schema_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../hoop-schema/schemas/debug_state.json");
        let schema_text = std::fs::read_to_string(&schema_path).unwrap();
        let schema: Value = serde_json::from_str(&schema_text).unwrap();

        // Top-level must reject additional properties
        assert_eq!(
            schema["additionalProperties"].as_bool(),
            Some(false),
            "top-level schema must have additionalProperties: false (§20)"
        );

        // version sub-object
        assert_eq!(
            schema["properties"]["version"]["additionalProperties"].as_bool(),
            Some(false),
            "version object must have additionalProperties: false"
        );

        // backup_timestamps sub-object
        assert_eq!(
            schema["properties"]["backup_timestamps"]["additionalProperties"].as_bool(),
            Some(false),
            "backup_timestamps object must have additionalProperties: false"
        );

        // Array item objects
        for array_field in &["workers", "worker_pids", "active_claims", "ws_clients", "session_alias_table"] {
            assert_eq!(
                schema["properties"][*array_field]["items"]["additionalProperties"].as_bool(),
                Some(false),
                "{array_field} items must have additionalProperties: false"
            );
        }
    }
}
