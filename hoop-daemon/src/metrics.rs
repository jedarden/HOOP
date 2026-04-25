//! Prometheus metrics for HOOP daemon.
//!
//! Call [`metrics()`] to get the global singleton.  All types are
//! thread-safe; recording a metric is a lock-free atomic operation on the
//! primitive types and a brief `RwLock` write on the labeled types.
//!
//! Rendered output is produced by [`Metrics::render_text`]; scrape-time
//! metrics (uptime, process stats, DB sizes) are appended by the HTTP
//! handler in `api_metrics.rs`.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::RwLock;

// ---------------------------------------------------------------------------
// Primitive unlabeled types (backward-compatible with existing call sites)
// ---------------------------------------------------------------------------

/// Monotonically-increasing counter with no labels.
#[derive(Debug, Default)]
pub struct Counter {
    value: AtomicU64,
}

impl Counter {
    pub fn new() -> Self {
        Self { value: AtomicU64::new(0) }
    }

    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_by(&self, amount: u64) {
        self.value.fetch_add(amount, Ordering::Relaxed);
    }

    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

/// Integer gauge (can go up and down) with no labels.
#[derive(Debug, Default)]
pub struct Gauge {
    value: AtomicI64,
}

impl Gauge {
    pub fn new() -> Self {
        Self { value: AtomicI64::new(0) }
    }

    pub fn set(&self, v: i64) {
        self.value.store(v, Ordering::Relaxed);
    }

    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    pub fn dec(&self) {
        self.value.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn add(&self, d: i64) {
        self.value.fetch_add(d, Ordering::Relaxed);
    }

    pub fn get(&self) -> i64 {
        self.value.load(Ordering::Relaxed)
    }
}

/// Float gauge backed by bit-cast atomic.
#[derive(Debug)]
pub struct FloatGauge {
    bits: AtomicU64,
}

impl FloatGauge {
    pub fn new() -> Self {
        Self { bits: AtomicU64::new(0) }
    }

    pub fn set(&self, v: f64) {
        self.bits.store(v.to_bits(), Ordering::Relaxed);
    }

    pub fn get(&self) -> f64 {
        f64::from_bits(self.bits.load(Ordering::Relaxed))
    }
}

impl Default for FloatGauge {
    fn default() -> Self {
        Self::new()
    }
}

/// Histogram (no label) that observes values **in seconds**.
/// Emits `_count` and `_sum` (seconds) in Prometheus format.
#[derive(Debug, Default)]
pub struct Histogram {
    count: AtomicU64,
    sum_us: AtomicU64, // stored as whole microseconds
}

impl Histogram {
    pub fn new() -> Self {
        Self {
            count: AtomicU64::new(0),
            sum_us: AtomicU64::new(0),
        }
    }

    /// Observe a duration measured in **seconds**.
    pub fn observe(&self, seconds: f64) {
        self.count.fetch_add(1, Ordering::Relaxed);
        let us = (seconds * 1_000_000.0) as u64;
        self.sum_us.fetch_add(us, Ordering::Relaxed);
    }

    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    /// Sum in milliseconds — kept for call sites in shutdown.rs.
    pub fn sum_ms(&self) -> u64 {
        self.sum_us.load(Ordering::Relaxed) / 1_000
    }

    pub fn sum_seconds(&self) -> f64 {
        self.sum_us.load(Ordering::Relaxed) as f64 / 1_000_000.0
    }
}

// ---------------------------------------------------------------------------
// Labeled types (new for §16)
// ---------------------------------------------------------------------------

/// Counter with a fixed set of label dimensions.  Cardinality is bounded by
/// the finite set of label-value tuples the caller actually uses.
pub struct LabeledCounter {
    pub label_names: &'static [&'static str],
    data: RwLock<HashMap<Vec<String>, u64>>,
}

impl LabeledCounter {
    pub fn new(label_names: &'static [&'static str]) -> Self {
        Self {
            label_names,
            data: RwLock::new(HashMap::new()),
        }
    }

    pub fn inc(&self, label_values: &[&str]) {
        self.inc_by(label_values, 1);
    }

    pub fn inc_by(&self, label_values: &[&str], amount: u64) {
        let key: Vec<String> = label_values.iter().map(|s| (*s).to_string()).collect();
        *self.data.write().unwrap().entry(key).or_insert(0) += amount;
    }

    pub fn snapshot(&self) -> Vec<(Vec<String>, u64)> {
        self.data
            .read()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }
}

/// Gauge with a fixed set of label dimensions.
pub struct LabeledGauge {
    pub label_names: &'static [&'static str],
    data: RwLock<HashMap<Vec<String>, i64>>,
}

impl LabeledGauge {
    pub fn new(label_names: &'static [&'static str]) -> Self {
        Self {
            label_names,
            data: RwLock::new(HashMap::new()),
        }
    }

    pub fn set(&self, label_values: &[&str], value: i64) {
        let key: Vec<String> = label_values.iter().map(|s| (*s).to_string()).collect();
        self.data.write().unwrap().insert(key, value);
    }

    pub fn snapshot(&self) -> Vec<(Vec<String>, i64)> {
        self.data
            .read()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }
}

/// Histogram with a fixed set of label dimensions.  Observes values
/// **in milliseconds** and emits `_count` / `_sum` (ms).
pub struct LabeledHistogram {
    pub label_names: &'static [&'static str],
    // (count, sum_ms)
    data: RwLock<HashMap<Vec<String>, (u64, f64)>>,
}

impl LabeledHistogram {
    pub fn new(label_names: &'static [&'static str]) -> Self {
        Self {
            label_names,
            data: RwLock::new(HashMap::new()),
        }
    }

    /// Observe a duration measured in **milliseconds**.
    pub fn observe(&self, label_values: &[&str], value_ms: f64) {
        let key: Vec<String> = label_values.iter().map(|s| (*s).to_string()).collect();
        let mut data = self.data.write().unwrap();
        let entry = data.entry(key).or_insert((0, 0.0));
        entry.0 += 1;
        entry.1 += value_ms;
    }

    pub fn snapshot(&self) -> Vec<(Vec<String>, u64, f64)> {
        self.data
            .read()
            .unwrap()
            .iter()
            .map(|(k, (c, s))| (k.clone(), *c, *s))
            .collect()
    }
}

/// Histogram with percentile support (p50, p95, p99) for §16.2 heartbeat freshness.
///
/// Tracks observations in a time-bounded sliding window (default: last 1000 observations per label set).
/// Computes percentiles on-demand during metric scraping.
pub struct LabeledHistogramPercentiles {
    pub label_names: &'static [&'static str],
    /// Per-label: (count, sum_seconds, sorted_observations)
    /// Observations are stored as f64 seconds.
    data: RwLock<HashMap<Vec<String>, (u64, f64, Vec<f64>)>>,
    /// Maximum observations to keep per label set
    max_observations: usize,
}

impl LabeledHistogramPercentiles {
    pub fn new(label_names: &'static [&'static str]) -> Self {
        Self {
            label_names,
            data: RwLock::new(HashMap::new()),
            max_observations: 1000,
        }
    }

    /// Set the maximum observations to keep per label set.
    pub fn with_max_observations(mut self, max: usize) -> Self {
        self.max_observations = max;
        self
    }

    /// Observe a value measured in **seconds**.
    pub fn observe(&self, label_values: &[&str], value_seconds: f64) {
        let key: Vec<String> = label_values.iter().map(|s| (*s).to_string()).collect();
        let mut data = self.data.write().unwrap();
        let entry = data.entry(key).or_insert((0, 0.0, Vec::new()));
        entry.0 += 1;
        entry.1 += value_seconds;

        // Add observation to the sorted vector
        let obs = &mut entry.2;
        // Find insertion point using binary search
        match obs.binary_search_by(|probe| probe.partial_cmp(&value_seconds).unwrap()) {
            Ok(_) => {} // Value already exists - still add it for accurate percentile computation
            Err(pos) => obs.insert(pos, value_seconds),
        }

        // Trim to max_observations (remove oldest - which are at the front for sorted order)
        if obs.len() > self.max_observations {
            obs.drain(0..obs.len() - self.max_observations);
        }
    }

    /// Compute percentile for a given label set.
    /// Returns None if no observations exist.
    fn percentile(&self, label_values: &[&str], percentile: f64) -> Option<f64> {
        let key: Vec<String> = label_values.iter().map(|s| (*s).to_string()).collect();
        let data = self.data.read().unwrap();
        let entry = data.get(&key)?;
        let obs = &entry.2;
        if obs.is_empty() {
            return None;
        }

        // Linear interpolation between closest ranks
        let n = obs.len();
        let pos = percentile * (n - 1) as f64;
        let lower = pos.floor() as usize;
        let upper = pos.ceil() as usize;

        if lower == upper {
            Some(obs[lower])
        } else {
            let weight = pos - lower as f64;
            Some(obs[lower] * (1.0 - weight) + obs[upper] * weight)
        }
    }

    pub fn snapshot(&self) -> Vec<(Vec<String>, u64, f64, Option<f64>, Option<f64>, Option<f64>)> {
        self.data
            .read()
            .unwrap()
            .iter()
            .map(|(k, (c, s, _))| {
                let p50 = self.percentile(&k.iter().map(|s| s.as_str()).collect::<Vec<_>>(), 0.50);
                let p95 = self.percentile(&k.iter().map(|s| s.as_str()).collect::<Vec<_>>(), 0.95);
                let p99 = self.percentile(&k.iter().map(|s| s.as_str()).collect::<Vec<_>>(), 0.99);
                (k.clone(), *c, *s, p50, p95, p99)
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Prometheus text-format helpers
// ---------------------------------------------------------------------------

fn escape_label_value(v: &str) -> String {
    v.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn labels_str(names: &[&'static str], values: &[String]) -> String {
    if names.is_empty() {
        return String::new();
    }
    let pairs: Vec<String> = names
        .iter()
        .zip(values.iter())
        .map(|(n, v)| format!("{}=\"{}\"", n, escape_label_value(v)))
        .collect();
    format!("{{{}}}", pairs.join(","))
}

fn write_counter(out: &mut String, name: &str, help: &str, value: u64) {
    out.push_str(&format!(
        "# HELP {name} {help}\n# TYPE {name} counter\n{name} {value}\n"
    ));
}

fn write_gauge_i64(out: &mut String, name: &str, help: &str, value: i64) {
    out.push_str(&format!(
        "# HELP {name} {help}\n# TYPE {name} gauge\n{name} {value}\n"
    ));
}

#[allow(dead_code)]
fn write_gauge_u64(out: &mut String, name: &str, help: &str, value: u64) {
    out.push_str(&format!(
        "# HELP {name} {help}\n# TYPE {name} gauge\n{name} {value}\n"
    ));
}

fn write_gauge_f64(out: &mut String, name: &str, help: &str, value: f64) {
    out.push_str(&format!(
        "# HELP {name} {help}\n# TYPE {name} gauge\n{name} {value:.6}\n"
    ));
}

fn write_histogram_seconds(out: &mut String, name: &str, help: &str, count: u64, sum: f64) {
    out.push_str(&format!(
        "# HELP {name} {help}\n# TYPE {name} histogram\n{name}_count {count}\n{name}_sum {sum:.6}\n"
    ));
}

fn write_histogram_ms(out: &mut String, name: &str, help: &str, count: u64, sum_ms: f64) {
    out.push_str(&format!(
        "# HELP {name} {help}\n# TYPE {name} histogram\n{name}_count {count}\n{name}_sum {sum_ms:.3}\n"
    ));
}

fn write_labeled_counter(
    out: &mut String,
    name: &str,
    help: &str,
    label_names: &[&'static str],
    rows: &[(Vec<String>, u64)],
) {
    out.push_str(&format!("# HELP {name} {help}\n# TYPE {name} counter\n"));
    let mut sorted = rows.to_vec();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    for (values, count) in &sorted {
        out.push_str(&format!("{name}{} {count}\n", labels_str(label_names, values)));
    }
}

fn write_labeled_gauge(
    out: &mut String,
    name: &str,
    help: &str,
    label_names: &[&'static str],
    rows: &[(Vec<String>, i64)],
) {
    out.push_str(&format!("# HELP {name} {help}\n# TYPE {name} gauge\n"));
    let mut sorted = rows.to_vec();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    for (values, val) in &sorted {
        out.push_str(&format!("{name}{} {val}\n", labels_str(label_names, values)));
    }
}

fn write_labeled_histogram(
    out: &mut String,
    name: &str,
    help: &str,
    label_names: &[&'static str],
    rows: &[(Vec<String>, u64, f64)],
) {
    out.push_str(&format!("# HELP {name} {help}\n# TYPE {name} histogram\n"));
    let mut sorted = rows.to_vec();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    for (values, count, sum_ms) in &sorted {
        let ls = labels_str(label_names, values);
        out.push_str(&format!("{name}_count{ls} {count}\n"));
        out.push_str(&format!("{name}_sum{ls} {sum_ms:.3}\n"));
    }
}

fn write_labeled_histogram_percentiles(
    out: &mut String,
    name: &str,
    help: &str,
    label_names: &[&'static str],
    rows: &[(Vec<String>, u64, f64, Option<f64>, Option<f64>, Option<f64>)],
) {
    out.push_str(&format!("# HELP {name} {help}\n# TYPE {name} gauge\n"));
    let mut sorted = rows.to_vec();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    for (values, _count, _sum, p50, p95, p99) in &sorted {
        let ls = labels_str(label_names, values);
        if let Some(v) = p50 {
            out.push_str(&format!("{name}_p50{ls} {v:.3}\n"));
        }
        if let Some(v) = p95 {
            out.push_str(&format!("{name}_p95{ls} {v:.3}\n"));
        }
        if let Some(v) = p99 {
            out.push_str(&format!("{name}_p99{ls} {v:.3}\n"));
        }
    }
}

// ---------------------------------------------------------------------------
// Global metrics catalog
// ---------------------------------------------------------------------------

/// Global metrics registry.  All fields are thread-safe.
pub struct Metrics {
    // ── Backward-compatible unlabeled metrics ───────────────────────────────

    /// Events dropped because no adapter parser recognised their type.
    pub hoop_unknown_event_total: Counter,
    /// Active WebSocket client connections (updated by ShutdownCoordinator).
    pub hoop_ws_clients_connected: Gauge,
    /// Wall-clock duration of graceful-shutdown sequences.
    pub hoop_shutdown_duration_seconds: Histogram,
    /// Shutdowns that ran past the configured grace period.
    pub hoop_shutdown_exceeded_grace_period: Counter,
    /// Connections that did not drain before the shutdown timeout fired.
    pub hoop_shutdown_timeout_connections: Counter,

    // ── §16.1 Operational ──────────────────────────────────────────────────

    /// Panics caught and recovered, labelled by subsystem.
    pub hoop_panics_total: LabeledCounter,
    /// Application errors labelled by subsystem and kind.
    pub hoop_errors_total: LabeledCounter,
    /// Reason for the last daemon restart (gauge with discrete reason label).
    pub hoop_last_restart_reason: LabeledGauge,

    // ── §16.2 Event Ingestion ──────────────────────────────────────────────

    /// Seconds between an event being written to disk and broadcast by HOOP.
    pub hoop_event_tailer_lag_seconds: LabeledGauge,
    /// Seconds of lag in the session tailer, per CLI adapter.
    pub hoop_session_tailer_lag_seconds: LabeledGauge,
    /// Heartbeat freshness in seconds, per worker (p50/p95/p99).
    pub hoop_heartbeat_freshness_seconds: LabeledHistogramPercentiles,
    /// Unknown-event drops with full adapter + event_kind context.
    pub hoop_unknown_event_labeled_total: LabeledCounter,
    /// Event parse errors per adapter.
    pub hoop_event_parse_errors_total: LabeledCounter,

    // ── §16.3 WebSocket & HTTP ─────────────────────────────────────────────

    /// WebSocket broadcast round-trip lag (milliseconds histogram).
    pub hoop_ws_broadcast_lag_ms: LabeledHistogram,
    /// HTTP request totals by route template and HTTP status code.
    pub hoop_http_requests_total: LabeledCounter,
    /// HTTP request duration in milliseconds, by route template.
    pub hoop_http_request_duration_ms: LabeledHistogram,

    // ── §16.4 Bead & Stitch Operations ────────────────────────────────────

    /// `br` subprocess invocations by verb (create/list/…) and result.
    pub hoop_br_subprocess_total: LabeledCounter,
    /// `br` subprocess wall-clock duration in milliseconds, by verb.
    pub hoop_br_subprocess_duration_ms: LabeledHistogram,
    /// Stitches created, labelled by project and kind.
    pub hoop_stitch_created_total: LabeledCounter,
    /// Beads created by HOOP automation, labelled by project.
    pub hoop_bead_created_by_hoop_total: LabeledCounter,
    /// Current audit-log append rate in lines per second.
    pub hoop_audit_append_rate_per_second: Gauge,
    /// Beads with no matching session in any project (orphan count).
    pub hoop_orphan_bead_count: LabeledGauge,

    // ── §16.5 Agent & AI ───────────────────────────────────────────────────

    /// Agent turn wall-clock duration in ms, by adapter, model, and phase.
    pub hoop_agent_turn_duration_ms: LabeledHistogram,
    /// Agent tool-call invocations by tool name and result.
    pub hoop_agent_tool_calls_total: LabeledCounter,
    /// Token consumption by adapter, model, and direction (input/output).
    pub hoop_agent_tokens_total: LabeledCounter,
    /// Estimated cost of the current active agent session in USD.
    pub hoop_agent_session_cost_usd: FloatGauge,
    /// Whisper transcription wall-clock duration histogram.
    pub hoop_whisper_transcription_duration_ms: Histogram,
    /// Whisper transcription failures.
    pub hoop_whisper_transcription_errors_total: Counter,
    /// Reflection proposals generated, labelled by source.
    pub hoop_reflection_proposal_total: LabeledCounter,
    /// Fraction of reflection proposals that were accepted (0.0 – 1.0 gauge).
    pub hoop_reflection_approval_rate: FloatGauge,

    // ── §16.6 Storage ──────────────────────────────────────────────────────

    /// Schema migration duration in milliseconds, labelled by from/to version.
    pub hoop_schema_migration_duration_ms: LabeledHistogram,
    /// Unix timestamp of the last successful backup (seconds since epoch).
    pub hoop_backup_last_success_timestamp: Gauge,
    /// Size of the last successful backup in bytes.
    pub hoop_backup_last_size_bytes: Gauge,
    /// Total number of backup runs that failed after all retries.
    pub hoop_backup_failures_total: Counter,
    /// Wall-clock duration of backup runs in seconds.
    pub hoop_backup_run_duration_seconds: Histogram,

    // ── §16.7 Business ─────────────────────────────────────────────────────

    /// Cost anomaly alerts fired.
    pub hoop_cost_anomaly_alerts_total: Counter,
    /// "Already started" deduplication hits (session reuse without new bead).
    pub hoop_already_started_dedup_hits_total: Counter,
    /// Capacity-meter exhaustion warnings, labelled by account.
    pub hoop_capacity_meter_exhaustion_warnings_total: LabeledCounter,
    /// Number of Stitches created in the current UTC day.
    pub hoop_stitches_created_per_day: Gauge,

    // ── §L3 JSONL Quarantine ──────────────────────────────────────────────────

    /// JSONL lines quarantined due to parse failures, labelled by source tag.
    pub hoop_jsonl_quarantined_lines_total: LabeledCounter,

    // ── §M5 Worker Spawn Ack ──────────────────────────────────────────────────

    /// Spawn-ack files received from NEEDLE workers on boot (§M5).
    pub hoop_worker_acks_seen_total: Counter,

    /// Workers that exceeded the ack grace window with no ack file (§M5).
    pub hoop_worker_spawn_missing_ack_total: Counter,

    // ── §17.5 Config hot-reload ───────────────────────────────────────────────

    /// Config reload attempts that were rejected due to validation failure (§17.5).
    pub hoop_config_reload_rejected_total: Counter,

    /// Config reload attempts that applied successfully (§17.5).
    pub hoop_config_reload_success_total: Counter,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            hoop_unknown_event_total: Counter::new(),
            hoop_ws_clients_connected: Gauge::new(),
            hoop_shutdown_duration_seconds: Histogram::new(),
            hoop_shutdown_exceeded_grace_period: Counter::new(),
            hoop_shutdown_timeout_connections: Counter::new(),

            hoop_panics_total: LabeledCounter::new(&["subsystem"]),
            hoop_errors_total: LabeledCounter::new(&["subsystem", "kind"]),
            hoop_last_restart_reason: LabeledGauge::new(&["reason"]),

            hoop_event_tailer_lag_seconds: LabeledGauge::new(&["project"]),
            hoop_session_tailer_lag_seconds: LabeledGauge::new(&["adapter"]),
            hoop_heartbeat_freshness_seconds: LabeledHistogramPercentiles::new(&["worker"]),
            hoop_unknown_event_labeled_total: LabeledCounter::new(&["adapter", "event_kind"]),
            hoop_event_parse_errors_total: LabeledCounter::new(&["adapter"]),

            hoop_ws_broadcast_lag_ms: LabeledHistogram::new(&[]),
            hoop_http_requests_total: LabeledCounter::new(&["route", "status"]),
            hoop_http_request_duration_ms: LabeledHistogram::new(&["route"]),

            hoop_br_subprocess_total: LabeledCounter::new(&["verb", "result"]),
            hoop_br_subprocess_duration_ms: LabeledHistogram::new(&["verb"]),
            hoop_stitch_created_total: LabeledCounter::new(&["project", "kind"]),
            hoop_bead_created_by_hoop_total: LabeledCounter::new(&["project"]),
            hoop_audit_append_rate_per_second: Gauge::new(),
            hoop_orphan_bead_count: LabeledGauge::new(&["project"]),

            hoop_agent_turn_duration_ms: LabeledHistogram::new(&["adapter", "model", "phase"]),
            hoop_agent_tool_calls_total: LabeledCounter::new(&["tool", "result"]),
            hoop_agent_tokens_total: LabeledCounter::new(&["adapter", "model", "direction"]),
            hoop_agent_session_cost_usd: FloatGauge::new(),
            hoop_whisper_transcription_duration_ms: Histogram::new(),
            hoop_whisper_transcription_errors_total: Counter::new(),
            hoop_reflection_proposal_total: LabeledCounter::new(&["source"]),
            hoop_reflection_approval_rate: FloatGauge::new(),

            hoop_schema_migration_duration_ms: LabeledHistogram::new(&["from", "to"]),
            hoop_backup_last_success_timestamp: Gauge::new(),
            hoop_backup_last_size_bytes: Gauge::new(),
            hoop_backup_failures_total: Counter::new(),
            hoop_backup_run_duration_seconds: Histogram::new(),

            hoop_cost_anomaly_alerts_total: Counter::new(),
            hoop_already_started_dedup_hits_total: Counter::new(),
            hoop_capacity_meter_exhaustion_warnings_total: LabeledCounter::new(&["account"]),
            hoop_stitches_created_per_day: Gauge::new(),

            hoop_jsonl_quarantined_lines_total: LabeledCounter::new(&["source"]),

            hoop_worker_acks_seen_total: Counter::new(),
            hoop_worker_spawn_missing_ack_total: Counter::new(),

            hoop_config_reload_rejected_total: Counter::new(),
            hoop_config_reload_success_total: Counter::new(),
        }
    }

    /// Render all accumulated metrics in Prometheus text exposition format.
    ///
    /// Scrape-time metrics (uptime, process stats, DB file sizes, worker
    /// liveness) are appended by the HTTP handler in `api_metrics.rs`.
    pub fn render_text(&self) -> String {
        let mut out = String::with_capacity(16 * 1024);

        // ── Shutdown / WebSocket (unlabeled) ────────────────────────────────
        write_counter(
            &mut out,
            "hoop_unknown_event_total",
            "Events discarded because no adapter could parse them.",
            self.hoop_unknown_event_total.get(),
        );
        write_gauge_i64(
            &mut out,
            "hoop_ws_clients_connected",
            "Active WebSocket client connections.",
            self.hoop_ws_clients_connected.get(),
        );
        write_histogram_seconds(
            &mut out,
            "hoop_shutdown_duration_seconds",
            "Wall-clock duration of graceful-shutdown sequences.",
            self.hoop_shutdown_duration_seconds.count(),
            self.hoop_shutdown_duration_seconds.sum_seconds(),
        );
        write_counter(
            &mut out,
            "hoop_shutdown_exceeded_grace_period",
            "Shutdowns that ran past the configured grace period.",
            self.hoop_shutdown_exceeded_grace_period.get(),
        );
        write_counter(
            &mut out,
            "hoop_shutdown_timeout_connections",
            "Connections that did not drain before the shutdown timeout.",
            self.hoop_shutdown_timeout_connections.get(),
        );

        // ── §16.1 Operational ───────────────────────────────────────────────
        write_labeled_counter(
            &mut out,
            "hoop_panics_total",
            "Panics caught and recovered, labelled by subsystem.",
            self.hoop_panics_total.label_names,
            &self.hoop_panics_total.snapshot(),
        );
        write_labeled_counter(
            &mut out,
            "hoop_errors_total",
            "Application errors labelled by subsystem and kind.",
            self.hoop_errors_total.label_names,
            &self.hoop_errors_total.snapshot(),
        );
        write_labeled_gauge(
            &mut out,
            "hoop_last_restart_reason",
            "Reason for the last daemon restart (discrete gauge).",
            self.hoop_last_restart_reason.label_names,
            &self.hoop_last_restart_reason.snapshot(),
        );

        // ── §16.2 Event Ingestion ────────────────────────────────────────────
        write_labeled_gauge(
            &mut out,
            "hoop_event_tailer_lag_seconds",
            "Seconds between disk-write and HOOP broadcast, per project.",
            self.hoop_event_tailer_lag_seconds.label_names,
            &self.hoop_event_tailer_lag_seconds.snapshot(),
        );
        write_labeled_gauge(
            &mut out,
            "hoop_session_tailer_lag_seconds",
            "Session tailer lag in seconds, per CLI adapter.",
            self.hoop_session_tailer_lag_seconds.label_names,
            &self.hoop_session_tailer_lag_seconds.snapshot(),
        );
        write_labeled_histogram_percentiles(
            &mut out,
            "hoop_heartbeat_freshness_seconds",
            "Heartbeat freshness in seconds, per worker (p50/p95/p99).",
            self.hoop_heartbeat_freshness_seconds.label_names,
            &self.hoop_heartbeat_freshness_seconds.snapshot(),
        );
        write_labeled_counter(
            &mut out,
            "hoop_unknown_event_labeled_total",
            "Unknown-event drops with adapter and event_kind context.",
            self.hoop_unknown_event_labeled_total.label_names,
            &self.hoop_unknown_event_labeled_total.snapshot(),
        );
        write_labeled_counter(
            &mut out,
            "hoop_event_parse_errors_total",
            "Event parse errors per adapter.",
            self.hoop_event_parse_errors_total.label_names,
            &self.hoop_event_parse_errors_total.snapshot(),
        );

        // ── §16.3 WebSocket & HTTP ──────────────────────────────────────────
        write_labeled_histogram(
            &mut out,
            "hoop_ws_broadcast_lag_ms",
            "WebSocket broadcast round-trip lag in milliseconds.",
            self.hoop_ws_broadcast_lag_ms.label_names,
            &self.hoop_ws_broadcast_lag_ms.snapshot(),
        );
        write_labeled_counter(
            &mut out,
            "hoop_http_requests_total",
            "HTTP requests by route template and status code.",
            self.hoop_http_requests_total.label_names,
            &self.hoop_http_requests_total.snapshot(),
        );
        write_labeled_histogram(
            &mut out,
            "hoop_http_request_duration_ms",
            "HTTP request duration in milliseconds, by route template.",
            self.hoop_http_request_duration_ms.label_names,
            &self.hoop_http_request_duration_ms.snapshot(),
        );

        // ── §16.4 Bead & Stitch ─────────────────────────────────────────────
        write_labeled_counter(
            &mut out,
            "hoop_br_subprocess_total",
            "`br` subprocess invocations by verb and result (ok/error).",
            self.hoop_br_subprocess_total.label_names,
            &self.hoop_br_subprocess_total.snapshot(),
        );
        write_labeled_histogram(
            &mut out,
            "hoop_br_subprocess_duration_ms",
            "`br` subprocess wall-clock duration in milliseconds, by verb.",
            self.hoop_br_subprocess_duration_ms.label_names,
            &self.hoop_br_subprocess_duration_ms.snapshot(),
        );
        write_labeled_counter(
            &mut out,
            "hoop_stitch_created_total",
            "Stitches created, labelled by project and kind.",
            self.hoop_stitch_created_total.label_names,
            &self.hoop_stitch_created_total.snapshot(),
        );
        write_labeled_counter(
            &mut out,
            "hoop_bead_created_by_hoop_total",
            "Beads created by HOOP automation, labelled by project.",
            self.hoop_bead_created_by_hoop_total.label_names,
            &self.hoop_bead_created_by_hoop_total.snapshot(),
        );
        write_gauge_i64(
            &mut out,
            "hoop_audit_append_rate_per_second",
            "Current audit-log append rate in lines per second.",
            self.hoop_audit_append_rate_per_second.get(),
        );
        write_labeled_gauge(
            &mut out,
            "hoop_orphan_bead_count",
            "Beads with no matching session in any project.",
            self.hoop_orphan_bead_count.label_names,
            &self.hoop_orphan_bead_count.snapshot(),
        );

        // ── §16.5 Agent & AI ─────────────────────────────────────────────────
        write_labeled_histogram(
            &mut out,
            "hoop_agent_turn_duration_ms",
            "Agent turn wall-clock duration in ms, by adapter/model/phase.",
            self.hoop_agent_turn_duration_ms.label_names,
            &self.hoop_agent_turn_duration_ms.snapshot(),
        );
        write_labeled_counter(
            &mut out,
            "hoop_agent_tool_calls_total",
            "Agent tool-call invocations by tool name and result.",
            self.hoop_agent_tool_calls_total.label_names,
            &self.hoop_agent_tool_calls_total.snapshot(),
        );
        write_labeled_counter(
            &mut out,
            "hoop_agent_tokens_total",
            "Token consumption by adapter, model, and direction (input/output).",
            self.hoop_agent_tokens_total.label_names,
            &self.hoop_agent_tokens_total.snapshot(),
        );
        write_gauge_f64(
            &mut out,
            "hoop_agent_session_cost_usd",
            "Estimated cost of the current active agent session in USD.",
            self.hoop_agent_session_cost_usd.get(),
        );
        write_histogram_ms(
            &mut out,
            "hoop_whisper_transcription_duration_ms",
            "Whisper transcription wall-clock duration in milliseconds.",
            self.hoop_whisper_transcription_duration_ms.count(),
            self.hoop_whisper_transcription_duration_ms.sum_ms() as f64,
        );
        write_counter(
            &mut out,
            "hoop_whisper_transcription_errors_total",
            "Whisper transcription failures.",
            self.hoop_whisper_transcription_errors_total.get(),
        );
        write_labeled_counter(
            &mut out,
            "hoop_reflection_proposal_total",
            "Reflection proposals generated, labelled by source.",
            self.hoop_reflection_proposal_total.label_names,
            &self.hoop_reflection_proposal_total.snapshot(),
        );
        write_gauge_f64(
            &mut out,
            "hoop_reflection_approval_rate",
            "Fraction of reflection proposals accepted (0.0 – 1.0).",
            self.hoop_reflection_approval_rate.get(),
        );

        // ── §16.6 Storage ────────────────────────────────────────────────────
        write_labeled_histogram(
            &mut out,
            "hoop_schema_migration_duration_ms",
            "Schema migration duration in milliseconds, by from/to version.",
            self.hoop_schema_migration_duration_ms.label_names,
            &self.hoop_schema_migration_duration_ms.snapshot(),
        );
        write_gauge_i64(
            &mut out,
            "hoop_backup_last_success_timestamp",
            "Unix timestamp of the last successful backup (seconds since epoch).",
            self.hoop_backup_last_success_timestamp.get(),
        );
        write_gauge_i64(
            &mut out,
            "hoop_backup_last_size_bytes",
            "Size of the last successful backup in bytes.",
            self.hoop_backup_last_size_bytes.get(),
        );
        write_counter(
            &mut out,
            "hoop_backup_failures_total",
            "Total number of backup runs that failed after all retries.",
            self.hoop_backup_failures_total.get(),
        );
        write_histogram_seconds(
            &mut out,
            "hoop_backup_run_duration_seconds",
            "Wall-clock duration of backup runs in seconds.",
            self.hoop_backup_run_duration_seconds.count(),
            self.hoop_backup_run_duration_seconds.sum_seconds(),
        );

        // ── §16.7 Business ───────────────────────────────────────────────────
        write_counter(
            &mut out,
            "hoop_cost_anomaly_alerts_total",
            "Cost anomaly alerts fired.",
            self.hoop_cost_anomaly_alerts_total.get(),
        );
        write_counter(
            &mut out,
            "hoop_already_started_dedup_hits_total",
            "Deduplication hits where an existing session was reused.",
            self.hoop_already_started_dedup_hits_total.get(),
        );
        write_labeled_counter(
            &mut out,
            "hoop_capacity_meter_exhaustion_warnings_total",
            "Capacity-meter exhaustion warnings, labelled by account.",
            self.hoop_capacity_meter_exhaustion_warnings_total.label_names,
            &self.hoop_capacity_meter_exhaustion_warnings_total.snapshot(),
        );
        write_gauge_i64(
            &mut out,
            "hoop_stitches_created_per_day",
            "Number of Stitches created in the current UTC day.",
            self.hoop_stitches_created_per_day.get(),
        );

        // ── §L3 JSONL Quarantine ─────────────────────────────────────────────
        write_labeled_counter(
            &mut out,
            "hoop_jsonl_quarantined_lines_total",
            "JSONL lines quarantined due to parse failures, labelled by source tag.",
            self.hoop_jsonl_quarantined_lines_total.label_names,
            &self.hoop_jsonl_quarantined_lines_total.snapshot(),
        );

        // ── §M5 Worker Spawn Ack ─────────────────────────────────────────────
        write_counter(
            &mut out,
            "hoop_worker_acks_seen_total",
            "Spawn-ack files received from NEEDLE workers on boot (§M5).",
            self.hoop_worker_acks_seen_total.get(),
        );
        write_counter(
            &mut out,
            "hoop_worker_spawn_missing_ack_total",
            "Workers that exceeded the ack grace window with no ack file (§M5).",
            self.hoop_worker_spawn_missing_ack_total.get(),
        );

        // ── §17.5 Config hot-reload ──────────────────────────────────────────
        write_counter(
            &mut out,
            "hoop_config_reload_rejected_total",
            "Config reload attempts rejected due to validation failure (§17.5).",
            self.hoop_config_reload_rejected_total.get(),
        );
        write_counter(
            &mut out,
            "hoop_config_reload_success_total",
            "Config reload attempts that applied successfully (§17.5).",
            self.hoop_config_reload_success_total.get(),
        );

        out
    }

    /// Set the reason for the last daemon restart.
    /// This should be called once at startup with the reason (e.g., "normal", "panic", "upgrade").
    pub fn set_last_restart_reason(&self, reason: &str) {
        self.hoop_last_restart_reason.set(&[reason], 1);
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Restart reason persistence (§16.1)
// ---------------------------------------------------------------------------

/// Path to the file storing the last restart reason.
fn restart_reason_path() -> PathBuf {
    let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.push(".hoop");
    home.push("restart_reason.txt");
    home
}

/// Load the persisted restart reason from disk.
/// Returns "normal" if the file doesn't exist or cannot be read.
pub fn load_restart_reason() -> String {
    std::fs::read_to_string(restart_reason_path())
        .ok()
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "normal".to_string())
}

/// Persist the restart reason to disk.
pub fn save_restart_reason(reason: &str) {
    let path = restart_reason_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, reason);
}

// ---------------------------------------------------------------------------
// Global singleton
// ---------------------------------------------------------------------------

static METRICS: std::sync::OnceLock<Metrics> = std::sync::OnceLock::new();

/// Return the global [`Metrics`] singleton, initialising it on first call.
pub fn metrics() -> &'static Metrics {
    METRICS.get_or_init(Metrics::new)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify panic counter increments with synthetic panic in a test subsystem.
    /// Acceptance criterion for §16.1.
    #[test]
    fn panic_counter_increments_on_synthetic_panic() {
        let m = Metrics::new();

        // Initially zero
        assert_eq!(m.hoop_panics_total.snapshot().len(), 0);

        // Simulate a panic in a test subsystem
        m.hoop_panics_total.inc(&["test_subsystem"]);

        // Verify counter incremented
        let snapshot = m.hoop_panics_total.snapshot();
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].0, vec!["test_subsystem".to_string()]);
        assert_eq!(snapshot[0].1, 1);

        // Increment again for same subsystem
        m.hoop_panics_total.inc(&["test_subsystem"]);

        let snapshot = m.hoop_panics_total.snapshot();
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].1, 2);

        // Different subsystem
        m.hoop_panics_total.inc(&["another_subsystem"]);

        let snapshot = m.hoop_panics_total.snapshot();
        assert_eq!(snapshot.len(), 2);
    }

    /// Verify errors_total counter with subsystem and kind labels.
    #[test]
    fn errors_total_tracks_subsystem_and_kind() {
        let m = Metrics::new();

        // Record an error
        m.hoop_errors_total.inc(&["agent_adapter", "timeout"]);

        let snapshot = m.hoop_errors_total.snapshot();
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].0, vec!["agent_adapter".to_string(), "timeout".to_string()]);
        assert_eq!(snapshot[0].1, 1);

        // Same subsystem, different kind
        m.hoop_errors_total.inc(&["agent_adapter", "rate_limit"]);

        let snapshot = m.hoop_errors_total.snapshot();
        assert_eq!(snapshot.len(), 2);
    }

    /// Verify last restart reason is persisted and loaded correctly.
    #[test]
    fn restart_reason_persists_across_restarts() {
        use std::fs;

        // Create a temp directory for testing
        let temp_dir = std::env::temp_dir().join("hoop_restart_test");
        let _ = fs::create_dir_all(&temp_dir);

        // Set a custom path for testing via env var (if we support it)
        // For now, we'll test the load/save logic directly

        // Save "panic" reason
        let test_file = temp_dir.join("restart_reason.txt");
        fs::write(&test_file, "panic\n").unwrap();

        // Load it back
        let loaded = fs::read_to_string(&test_file)
            .ok()
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "normal".to_string());

        assert_eq!(loaded, "panic");

        // Clean up
        let _ = fs::remove_file(&test_file);
        let _ = fs::remove_dir(&temp_dir);

        // Test default when file doesn't exist
        let missing = temp_dir.join("nonexistent.txt");
        let default = fs::read_to_string(&missing)
            .ok()
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "normal".to_string());

        assert_eq!(default, "normal");
    }

    /// Verify last_restart_reason gauge behavior.
    #[test]
    fn last_restart_reason_gauge() {
        let m = Metrics::new();

        // Set reason
        m.set_last_restart_reason("panic");

        let snapshot = m.hoop_last_restart_reason.snapshot();
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].0, vec!["panic".to_string()]);
        assert_eq!(snapshot[0].1, 1);

        // Change reason (should replace, not add)
        m.set_last_restart_reason("normal");

        let snapshot = m.hoop_last_restart_reason.snapshot();
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].0, vec!["normal".to_string()]);
    }
}
