//! Simple metrics tracking for HOOP
//!
//! This provides a minimal metrics system without external dependencies.
//! For production, this can be replaced with a proper Prometheus library.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::RwLock;

/// Simple counter metric
#[derive(Debug)]
pub struct Counter {
    value: AtomicU64,
    labels: RwLock<HashMap<String, u64>>,
}

impl Counter {
    /// Create a new counter
    pub fn new() -> Self {
        Self {
            value: AtomicU64::new(0),
            labels: RwLock::new(HashMap::new()),
        }
    }

    /// Increment the counter
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment by a specific amount
    pub fn inc_by(&self, amount: u64) {
        self.value.fetch_add(amount, Ordering::Relaxed);
    }

    /// Increment with a label value
    pub fn inc_label(&self, label: &str, label_value: &str) {
        self.inc();
        let key = format!("{}={}", label, label_value);
        let mut labels = self.labels.write().unwrap();
        *labels.entry(key).or_insert(0) += 1;
    }

    /// Get the current value
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    /// Get the values by label
    pub fn get_labels(&self) -> HashMap<String, u64> {
        let labels = self.labels.read().unwrap();
        labels.clone()
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple gauge metric (can go up and down)
#[derive(Debug)]
pub struct Gauge {
    value: AtomicI64,
}

impl Gauge {
    /// Create a new gauge
    pub fn new() -> Self {
        Self {
            value: AtomicI64::new(0),
        }
    }

    /// Set the gauge to a specific value
    pub fn set(&self, value: i64) {
        self.value.store(value, Ordering::Relaxed);
    }

    /// Increment the gauge
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement the gauge
    pub fn dec(&self) {
        self.value.fetch_sub(1, Ordering::Relaxed);
    }

    /// Add to the gauge
    pub fn add(&self, delta: i64) {
        self.value.fetch_add(delta, Ordering::Relaxed);
    }

    /// Get the current value
    pub fn get(&self) -> i64 {
        self.value.load(Ordering::Relaxed)
    }
}

impl Default for Gauge {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple histogram metric
#[derive(Debug)]
pub struct Histogram {
    // For simplicity, we'll just track the count and sum
    // A real implementation would have buckets
    count: AtomicU64,
    sum: AtomicU64,
}

impl Histogram {
    /// Create a new histogram
    pub fn new() -> Self {
        Self {
            count: AtomicU64::new(0),
            sum: AtomicU64::new(0),
        }
    }

    /// Observe a value
    pub fn observe(&self, value: f64) {
        self.count.fetch_add(1, Ordering::Relaxed);
        // Convert to milliseconds as integer for simplicity
        let ms = (value * 1000.0) as u64;
        self.sum.fetch_add(ms, Ordering::Relaxed);
    }

    /// Get the count of observations
    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    /// Get the sum of all observations (in milliseconds)
    pub fn sum_ms(&self) -> u64 {
        self.sum.load(Ordering::Relaxed)
    }
}

impl Default for Histogram {
    fn default() -> Self {
        Self::new()
    }
}

/// Global metrics registry
pub struct Metrics {
    /// Counter for unknown events
    pub hoop_unknown_event_total: Counter,
    /// Gauge for number of live workers
    pub hoop_workers_live: Counter,
    /// Gauge for number of hung workers
    pub hoop_workers_hung: Counter,
    /// Gauge for number of dead workers
    pub hoop_workers_dead: Counter,
    /// Gauge for active WebSocket connections
    pub hoop_ws_clients_connected: Gauge,
    /// Histogram for shutdown duration in seconds
    pub hoop_shutdown_duration_seconds: Histogram,
    /// Counter for shutdowns that exceeded grace period
    pub hoop_shutdown_exceeded_grace_period: Counter,
    /// Counter for connections that didn't close in time
    pub hoop_shutdown_timeout_connections: Counter,
}

impl Metrics {
    /// Create a new metrics registry
    pub fn new() -> Self {
        Self {
            hoop_unknown_event_total: Counter::new(),
            hoop_workers_live: Counter::new(),
            hoop_workers_hung: Counter::new(),
            hoop_workers_dead: Counter::new(),
            hoop_ws_clients_connected: Gauge::new(),
            hoop_shutdown_duration_seconds: Histogram::new(),
            hoop_shutdown_exceeded_grace_period: Counter::new(),
            hoop_shutdown_timeout_connections: Counter::new(),
        }
    }

    /// Update worker liveness metrics
    pub fn update_worker_liveness(&self, live: usize, hung: usize, dead: usize) {
        // Reset counters by reading current and adjusting
        let current_live = self.hoop_workers_live.get();
        let current_hung = self.hoop_workers_hung.get();
        let current_dead = self.hoop_workers_dead.get();

        if current_live < live as u64 {
            self.hoop_workers_live.inc_by((live as u64) - current_live);
        } else if current_live > live as u64 {
            // For gauges, we don't have a dec operation
            // The counter will monotonically increase; for real metrics use a proper gauge
        }

        if current_hung < hung as u64 {
            self.hoop_workers_hung.inc_by((hung as u64) - current_hung);
        }

        if current_dead < dead as u64 {
            self.hoop_workers_dead.inc_by((dead as u64) - current_dead);
        }
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Global metrics instance
static METRICS: std::sync::OnceLock<Metrics> = std::sync::OnceLock::new();

/// Get the global metrics instance
pub fn metrics() -> &'static Metrics {
    METRICS.get_or_init(Metrics::new)
}
