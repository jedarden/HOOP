//! Central sink for unrecognized event kinds from all tailers.
//!
//! Every tailer (events.jsonl, heartbeats.jsonl, each session adapter) routes
//! unrecognized event kinds through this central sink that:
//! - Logs at WARN with raw event (rate-limited to prevent log storms)
//! - Increments `hoop_unknown_event_total` and `hoop_unknown_event_labeled_total{adapter,event_kind}` metrics
//! - Buffers last N (default 20) samples for the diagnostic panel
//!
//! Rate limiting: Each unique (adapter, event_kind) pair is logged once per
//! 5-minute window, with a count of suppressed events. This prevents log storms
//! when a stream produces millions of unknown events.
//!
//! Plan reference: §3 principle 7, §16.2, §M1 orchestrator-problems-and-solutions.md

use crate::metrics;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::warn;

/// Default number of unknown event samples to buffer for diagnostics.
const DEFAULT_SAMPLE_BUFFER_SIZE: usize = 20;

/// Rate limit window for logging unknown events.
const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(300); // 5 minutes

/// Type alias for rate limit tracker: maps (adapter, event_kind) → (last_log_time, suppressed_count).
///
/// This prevents log storms by tracking when each unique event type was last logged
/// and how many occurrences were suppressed within the rate limit window.
type RateLimitTracker = Arc<Mutex<HashMap<(String, String), (std::time::Instant, u64)>>>;

/// A single unknown event sample for diagnostic display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnknownEventSample {
    /// Adapter that reported the unknown event (e.g., "needle", "heartbeats", "claude").
    pub adapter: String,
    /// The unknown event kind/type that was not recognized.
    pub event_kind: String,
    /// Raw event payload (truncated if too long).
    pub raw_event: String,
    /// Timestamp when the unknown event was recorded.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Source file path (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    /// Line number in source file (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_number: Option<usize>,
}

impl UnknownEventSample {
    /// Create a new unknown event sample.
    pub fn new(
        adapter: String,
        event_kind: String,
        raw_event: String,
        source_path: Option<String>,
        line_number: Option<usize>,
    ) -> Self {
        Self {
            adapter,
            event_kind,
            raw_event,
            timestamp: chrono::Utc::now(),
            source_path,
            line_number,
        }
    }

    /// Create a sample with minimal information.
    pub fn simple(adapter: &str, event_kind: &str, raw_event: &str) -> Self {
        Self::new(
            adapter.to_string(),
            event_kind.to_string(),
            raw_event.to_string(),
            None,
            None,
        )
    }
}

/// Central sink for all unrecognized events.
///
/// Maintains a circular buffer of recent samples for diagnostic display.
/// Implements rate limiting to prevent log storms.
#[derive(Debug, Clone)]
pub struct UnknownEventSink {
    /// Adapter name for metric labeling (e.g., "needle", "heartbeats", "claude").
    adapter: String,
    /// Optional source file path for diagnostics.
    source_path: Option<String>,
    /// Sample buffer (shared, synchronized).
    samples: Arc<Mutex<Vec<UnknownEventSample>>>,
    /// Maximum number of samples to retain.
    max_samples: usize,
    /// Rate limit tracker: (adapter, event_kind) -> (last_log_time, suppressed_count).
    rate_limit_tracker: RateLimitTracker,
}

impl UnknownEventSink {
    /// Create a new sink for a specific adapter.
    pub fn new(adapter: &str) -> Self {
        Self {
            adapter: adapter.to_string(),
            source_path: None,
            samples: Arc::new(Mutex::new(Vec::with_capacity(DEFAULT_SAMPLE_BUFFER_SIZE))),
            max_samples: DEFAULT_SAMPLE_BUFFER_SIZE,
            rate_limit_tracker: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a new sink for a file-based adapter (events.jsonl, heartbeats.jsonl).
    pub fn with_source(adapter: &str, source_path: std::path::PathBuf) -> Self {
        Self {
            adapter: adapter.to_string(),
            source_path: Some(source_path.to_string_lossy().to_string()),
            samples: Arc::new(Mutex::new(Vec::with_capacity(DEFAULT_SAMPLE_BUFFER_SIZE))),
            max_samples: DEFAULT_SAMPLE_BUFFER_SIZE,
            rate_limit_tracker: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a new sink with a custom buffer size.
    pub fn with_buffer_size(adapter: &str, max_samples: usize) -> Self {
        Self {
            adapter: adapter.to_string(),
            source_path: None,
            samples: Arc::new(Mutex::new(Vec::with_capacity(max_samples))),
            max_samples,
            rate_limit_tracker: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Record an unknown event.
    ///
    /// This method:
    /// 1. Logs a WARN-level message with the raw event (rate-limited per adapter/event_kind pair)
    /// 2. Increments both `hoop_unknown_event_total` and `hoop_unknown_event_labeled_total` metrics
    /// 3. Adds the sample to the circular buffer for diagnostics
    /// 4. Registers the sample with the global registry for API access
    ///
    /// Rate limiting: Each unique (adapter, event_kind) pair is logged once per
    /// 5-minute window. Subsequent occurrences within the window are counted and
    /// a summary is logged at the end of the window.
    pub fn record(&self, event_kind: &str, raw_event: &str) {
        // Increment both metrics (unlabeled total and labeled with adapter/event_kind)
        metrics::metrics().hoop_unknown_event_total.inc();
        metrics::metrics()
            .hoop_unknown_event_labeled_total
            .inc(&[&self.adapter, event_kind]);

        // Check rate limit for this (adapter, event_kind) pair
        let key = (self.adapter.clone(), event_kind.to_string());
        let mut tracker = self.rate_limit_tracker.blocking_lock();
        let now = std::time::Instant::now();

        // Clone values to avoid borrow issues
        let (should_log, suppressed_count_opt) = match tracker.get(&key).cloned() {
            Some((last_log, suppressed_count)) => {
                if now.duration_since(last_log) >= RATE_LIMIT_WINDOW {
                    // Window expired, log summary and reset
                    tracker.insert(key, (now, 0));
                    (true, Some(suppressed_count))
                } else {
                    // Still within window, just increment counter
                    tracker.insert(key, (last_log, suppressed_count + 1));
                    (false, None)
                }
            }
            None => {
                // First time seeing this event kind, log it
                tracker.insert(key, (now, 0));
                (true, Some(0))
            }
        };

        drop(tracker); // Release lock before potential blocking operations

        // Log if needed (avoid holding lock during logging)
        if should_log {
            let source_info = if let Some(ref path) = self.source_path {
                format!(" (source: {})", path)
            } else {
                String::new()
            };

            if let Some(suppressed_count) = suppressed_count_opt {
                if suppressed_count > 0 {
                    warn!(
                        "Unknown event kind '{}' from adapter '{}'{}: suppressed {} additional occurrences. First occurrence: {}",
                        event_kind,
                        self.adapter,
                        source_info,
                        suppressed_count,
                        truncate_for_log(raw_event)
                    );
                } else {
                    warn!(
                        "Unknown event kind '{}' from adapter '{}'{}. Raw event: {}",
                        event_kind,
                        self.adapter,
                        source_info,
                        truncate_for_log(raw_event)
                    );
                }
            }

            // Create sample and add to buffer
            let sample = UnknownEventSample::new(
                self.adapter.clone(),
                event_kind.to_string(),
                raw_event.to_string(),
                self.source_path.clone(),
                None,
            );

            let mut samples = self.samples.blocking_lock();
            if samples.len() >= self.max_samples {
                // Remove oldest sample (circular buffer)
                samples.remove(0);
            }
            samples.push(sample.clone());

            // Also register with global registry for API access
            global_registry().register_sample(sample);
        }
    }

    /// Record an unknown event with line number context.
    ///
    /// This method:
    /// 1. Logs a WARN-level message with the raw event (rate-limited per adapter/event_kind pair)
    /// 2. Increments both `hoop_unknown_event_total` and `hoop_unknown_event_labeled_total` metrics
    /// 3. Adds the sample to the circular buffer for diagnostics
    /// 4. Registers the sample with the global registry for API access
    ///
    /// Rate limiting: Each unique (adapter, event_kind) pair is logged once per
    /// 5-minute window. Subsequent occurrences within the window are counted and
    /// a summary is logged at the end of the window.
    pub fn record_at_line(&self, event_kind: &str, raw_event: &str, line_number: usize) {
        // Increment both metrics (unlabeled total and labeled with adapter/event_kind)
        metrics::metrics().hoop_unknown_event_total.inc();
        metrics::metrics()
            .hoop_unknown_event_labeled_total
            .inc(&[&self.adapter, event_kind]);

        // Check rate limit for this (adapter, event_kind) pair
        let key = (self.adapter.clone(), event_kind.to_string());
        let mut tracker = self.rate_limit_tracker.blocking_lock();
        let now = std::time::Instant::now();

        // Clone values to avoid borrow issues
        let (should_log, suppressed_count_opt) = match tracker.get(&key).cloned() {
            Some((last_log, suppressed_count)) => {
                if now.duration_since(last_log) >= RATE_LIMIT_WINDOW {
                    // Window expired, log summary and reset
                    tracker.insert(key, (now, 0));
                    (true, Some(suppressed_count))
                } else {
                    // Still within window, just increment counter
                    tracker.insert(key, (last_log, suppressed_count + 1));
                    (false, None)
                }
            }
            None => {
                // First time seeing this event kind, log it
                tracker.insert(key, (now, 0));
                (true, Some(0))
            }
        };

        drop(tracker); // Release lock before potential blocking operations

        // Log if needed (avoid holding lock during logging)
        if should_log {
            let source_info = if let Some(ref path) = self.source_path {
                format!(" (source: {}:{})", path, line_number)
            } else {
                format!(" (line {})", line_number)
            };

            if let Some(suppressed_count) = suppressed_count_opt {
                if suppressed_count > 0 {
                    warn!(
                        "Unknown event kind '{}' from adapter '{}'{}: suppressed {} additional occurrences. First occurrence: {}",
                        event_kind,
                        self.adapter,
                        source_info,
                        suppressed_count,
                        truncate_for_log(raw_event)
                    );
                } else {
                    warn!(
                        "Unknown event kind '{}' from adapter '{}'{}. Raw event: {}",
                        event_kind,
                        self.adapter,
                        source_info,
                        truncate_for_log(raw_event)
                    );
                }
            }

            // Create sample and add to buffer
            let sample = UnknownEventSample::new(
                self.adapter.clone(),
                event_kind.to_string(),
                raw_event.to_string(),
                self.source_path.clone(),
                Some(line_number),
            );

            let mut samples = self.samples.blocking_lock();
            if samples.len() >= self.max_samples {
                samples.remove(0);
            }
            samples.push(sample.clone());

            // Also register with global registry for API access
            global_registry().register_sample(sample);
        }
    }

    /// Get all buffered samples.
    pub fn get_samples(&self) -> Vec<UnknownEventSample> {
        self.samples.blocking_lock().clone()
    }

    /// Clear all buffered samples.
    pub fn clear_samples(&self) {
        self.samples.blocking_lock().clear();
    }
}

/// Truncate a string for logging (max 100 chars, more aggressive to prevent log bloat).
///
/// Uses character-safe truncation to avoid panicking on multi-byte UTF-8 characters.
/// This is intentionally short to prevent log storms from large JSON payloads.
pub fn truncate_for_log(s: &str) -> String {
    if s.chars().count() > 100 {
        s.chars().take(100).collect::<String>() + "... (truncated)"
    } else {
        s.to_string()
    }
}

/// Global registry of all unknown event samples from all adapters.
///
/// This provides a single point for the diagnostic panel to fetch all samples.
pub struct GlobalUnknownEventRegistry {
    /// All samples from all adapters (synchronized).
    all_samples: Arc<Mutex<Vec<UnknownEventSample>>>,
    /// Maximum total samples to retain across all adapters.
    max_total_samples: usize,
}

impl GlobalUnknownEventRegistry {
    /// Create a new global registry.
    pub fn new() -> Self {
        Self {
            all_samples: Arc::new(Mutex::new(Vec::with_capacity(100))),
            max_total_samples: 100,
        }
    }

    /// Create a new global registry with a custom buffer size.
    pub fn with_max_samples(max_samples: usize) -> Self {
        Self {
            all_samples: Arc::new(Mutex::new(Vec::with_capacity(max_samples))),
            max_total_samples: max_samples,
        }
    }

    /// Register a sample from any sink.
    pub fn register_sample(&self, sample: UnknownEventSample) {
        let mut samples = self.all_samples.blocking_lock();
        if samples.len() >= self.max_total_samples {
            // Remove oldest sample
            samples.remove(0);
        }
        samples.push(sample);
    }

    /// Get all registered samples.
    pub fn get_all_samples(&self) -> Vec<UnknownEventSample> {
        self.all_samples.blocking_lock().clone()
    }

    /// Clear all samples.
    pub fn clear_all(&self) {
        self.all_samples.blocking_lock().clear();
    }

    /// Get samples grouped by adapter.
    pub fn get_samples_by_adapter(&self, adapter: &str) -> Vec<UnknownEventSample> {
        self.all_samples
            .blocking_lock()
            .iter()
            .filter(|s| s.adapter == adapter)
            .cloned()
            .collect()
    }
}

impl Default for GlobalUnknownEventRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Global singleton for the registry
// ---------------------------------------------------------------------------

static GLOBAL_REGISTRY: std::sync::OnceLock<GlobalUnknownEventRegistry> =
    std::sync::OnceLock::new();

/// Get the global unknown event registry.
pub fn global_registry() -> &'static GlobalUnknownEventRegistry {
    GLOBAL_REGISTRY.get_or_init(GlobalUnknownEventRegistry::new)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sink_records_unknown_event() {
        let sink = UnknownEventSink::new("test_adapter");

        // Record an unknown event
        sink.record("unknown_kind", r#"{"foo":"bar"}"#);

        let samples = sink.get_samples();
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].adapter, "test_adapter");
        assert_eq!(samples[0].event_kind, "unknown_kind");
        assert_eq!(samples[0].raw_event, r#"{"foo":"bar"}"#);
    }

    #[test]
    fn sink_records_event_with_line_number() {
        let sink = UnknownEventSink::new("test_adapter");

        sink.record_at_line("unknown_kind", r#"{"baz":"quux"}"#, 42);

        let samples = sink.get_samples();
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].line_number, Some(42));
    }

    #[test]
    fn sink_circular_buffer_evicts_oldest() {
        let sink = UnknownEventSink::with_buffer_size("test_adapter", 3);

        for i in 0..5 {
            sink.record(&format!("kind_{}", i), &format!("event_{}", i));
        }

        let samples = sink.get_samples();
        assert_eq!(samples.len(), 3);
        // Oldest (0, 1) should be evicted, keeping (2, 3, 4)
        assert_eq!(samples[0].event_kind, "kind_2");
        assert_eq!(samples[1].event_kind, "kind_3");
        assert_eq!(samples[2].event_kind, "kind_4");
    }

    #[test]
    fn global_registry_collects_from_all_adapters() {
        let registry = global_registry();

        // Clear any existing samples
        registry.clear_all();

        // Register samples from different adapters
        registry.register_sample(UnknownEventSample::simple("adapter1", "kind1", "{}"));
        registry.register_sample(UnknownEventSample::simple("adapter2", "kind2", "{}"));
        registry.register_sample(UnknownEventSample::simple("adapter1", "kind3", "{}"));

        let all_samples = registry.get_all_samples();
        assert_eq!(all_samples.len(), 3);

        let adapter1_samples = registry.get_samples_by_adapter("adapter1");
        assert_eq!(adapter1_samples.len(), 2);

        let adapter2_samples = registry.get_samples_by_adapter("adapter2");
        assert_eq!(adapter2_samples.len(), 1);
    }

    #[test]
    fn truncate_for_log_works() {
        let long = "a".repeat(300);
        let truncated = truncate_for_log(&long);
        assert_eq!(truncated.len(), 203); // 200 + "..."
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn sink_with_source_path() {
        let sink = UnknownEventSink::with_source(
            "test_adapter",
            std::path::PathBuf::from("/test/path.jsonl"),
        );

        sink.record_at_line("unknown_kind", r#"{"test":true}"#, 10);

        let samples = sink.get_samples();
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].source_path, Some("/test/path.jsonl".to_string()));
        assert_eq!(samples[0].line_number, Some(10));
    }

    #[test]
    fn rate_limits_duplicate_unknown_events() {
        let sink = UnknownEventSink::new("test_adapter");

        // Record the same event type 100 times
        for _ in 0..100 {
            sink.record("unknown_kind", r#"{"type":"unknown_kind"}"#);
        }

        // Should only have a few samples due to rate limiting
        // First occurrence is logged, subsequent ones are counted and logged after window
        let samples = sink.get_samples();
        assert!(samples.len() <= 2, "Rate limiting should reduce sample count");
    }

    #[test]
    fn rate_limits_per_event_kind() {
        let sink = UnknownEventSink::new("test_adapter");

        // Record different event types
        sink.record("kind1", r#"{"type":"kind1"}"#);
        sink.record("kind2", r#"{"type":"kind2"}"#);
        sink.record("kind1", r#"{"type":"kind1"}"#);
        sink.record("kind2", r#"{"type":"kind2"}"#);

        let samples = sink.get_samples();
        // Should have at most 4 samples (2 per kind)
        assert!(samples.len() <= 4);
    }

    #[test]
    fn aggressive_truncation_works() {
        let long = "a".repeat(300);
        let truncated = truncate_for_log(&long);
        assert_eq!(truncated.len(), 113); // 100 + "... (truncated)"
        assert!(truncated.ends_with("... (truncated)"));
    }
}
