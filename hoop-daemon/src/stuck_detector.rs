//! Three-timer stuck detector (§C1)
//!
//! Implements the three-timer approach for detecting stuck workers:
//! - `idle_timeout` — no events at all
//! - `max_runtime` — hard ceiling on total duration
//! - `content_seen_grace` — extended idle tolerance once real content appeared
//!
//! Alert events include `saw_content: bool` to distinguish silence-before-first-content
//! from silence-after.
//!
//! # Design rationale
//!
//! Single-timer alerts fire during legitimate tool-use silence → false positives
//! → operator fatigue → real alerts ignored. The three-timer approach prevents
//! false positives by:
//! 1. Using separate timers for different failure modes
//! 2. Extending grace period once we know the worker has produced real content
//! 3. Preserving the content-seen signal in telemetry

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

/// Default idle timeout: no events for N seconds (180s = 3 minutes)
const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 180;

/// Default max runtime: hard ceiling on total duration (3600s = 1 hour)
const DEFAULT_MAX_RUNTIME_SECS: u64 = 3600;

/// Default content-seen grace: extended idle tolerance after content (600s = 10 minutes)
const DEFAULT_CONTENT_SEEN_GRACE_SECS: u64 = 600;

/// Default heartbeat transition silence threshold: no state change for N seconds (300s = 5 minutes)
const DEFAULT_HEARTBEAT_TRANSITION_THRESHOLD_SECS: u64 = 300;

/// Default retry threshold: max times a worker can retry the same bead (3 attempts)
const DEFAULT_RETRY_THRESHOLD: u32 = 3;

/// Known worker adapter types for stuck detector configuration
///
/// These correspond to the CLI adapters that workers can use:
/// - claude: Anthropic Claude adapter (opus, sonnet, haiku models)
/// - codex: OpenAI Codex adapter
/// - opencode: OpenCode adapter
/// - gemini: Google Gemini adapter
/// - zai: Zai GLM adapter
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WorkerAdapterType {
    #[serde(rename = "claude")]
    Claude,
    #[serde(rename = "codex")]
    Codex,
    #[serde(rename = "opencode")]
    OpenCode,
    #[serde(rename = "gemini")]
    Gemini,
    #[serde(rename = "zai")]
    Zai,
}

impl WorkerAdapterType {
    /// Parse from string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "claude" => Some(Self::Claude),
            "codex" => Some(Self::Codex),
            "opencode" => Some(Self::OpenCode),
            "gemini" => Some(Self::Gemini),
            "zai" => Some(Self::Zai),
            _ => None,
        }
    }

    /// Convert to lowercase string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::OpenCode => "opencode",
            Self::Gemini => "gemini",
            Self::Zai => "zai",
        }
    }
}

/// Stuck detector timer configuration for a single worker type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StuckDetectorConfig {
    /// No events, any type, for N seconds
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout_secs: u64,
    /// Hard ceiling on total duration regardless of activity
    #[serde(default = "default_max_runtime")]
    pub max_runtime_secs: u64,
    /// Extended idle tolerance once real content appeared
    #[serde(default = "default_content_seen_grace")]
    pub content_seen_grace_secs: u64,
    /// No heartbeat state transition (Live<->Hung) for N seconds
    #[serde(default = "default_heartbeat_transition_threshold")]
    pub heartbeat_transition_threshold_secs: u64,
    /// Maximum retry attempts on the same bead before alerting
    #[serde(default = "default_retry_threshold")]
    pub retry_threshold: u32,
}

impl Default for StuckDetectorConfig {
    fn default() -> Self {
        Self {
            idle_timeout_secs: DEFAULT_IDLE_TIMEOUT_SECS,
            max_runtime_secs: DEFAULT_MAX_RUNTIME_SECS,
            content_seen_grace_secs: DEFAULT_CONTENT_SEEN_GRACE_SECS,
            heartbeat_transition_threshold_secs: DEFAULT_HEARTBEAT_TRANSITION_THRESHOLD_SECS,
            retry_threshold: DEFAULT_RETRY_THRESHOLD,
        }
    }
}

fn default_idle_timeout() -> u64 {
    DEFAULT_IDLE_TIMEOUT_SECS
}

fn default_max_runtime() -> u64 {
    DEFAULT_MAX_RUNTIME_SECS
}

fn default_content_seen_grace() -> u64 {
    DEFAULT_CONTENT_SEEN_GRACE_SECS
}

fn default_heartbeat_transition_threshold() -> u64 {
    DEFAULT_HEARTBEAT_TRANSITION_THRESHOLD_SECS
}

fn default_retry_threshold() -> u32 {
    DEFAULT_RETRY_THRESHOLD
}

/// Per-worker-type stuck detector configuration map
///
/// Maps adapter types to their specific stuck detector configs.
/// Falls back to the default config for unmapped adapter types.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StuckDetectorConfigMap {
    /// Default configuration for unmapped worker types
    #[serde(default)]
    pub default: StuckDetectorConfig,
    /// Per-adapter configurations (optional)
    #[serde(default)]
    pub adapters: std::collections::HashMap<String, StuckDetectorConfig>,
}

impl StuckDetectorConfigMap {
    /// Get the configuration for a specific adapter type, falling back to default
    pub fn get_for_adapter(&self, adapter: &str) -> StuckDetectorConfig {
        self.adapters
            .get(adapter)
            .cloned()
            .unwrap_or_else(|| self.default.clone())
    }
}

/// Alert emitted when a worker is stuck
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StuckAlert {
    pub worker: String,
    pub bead: String,
    pub started_at: DateTime<Utc>,
    pub last_event_at: DateTime<Utc>,
    pub elapsed_secs: i64,
    pub idle_secs: i64,
    pub saw_content: bool,
    pub reason: StuckReason,
    pub message: String,
    /// Last heartbeat timestamp (for transition silence detection)
    pub last_heartbeat_at: Option<DateTime<Utc>>,
    /// Last heartbeat state transition timestamp
    pub last_transition_at: Option<DateTime<Utc>>,
    /// Number of times this bead has been retried
    pub retry_count: u32,
}

/// Why the worker was detected as stuck
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StuckReason {
    /// No events for idle_timeout_secs
    IdleTimeout,
    /// Exceeded max_runtime_secs
    MaxRuntimeExceeded,
    /// No events for content_seen_grace_secs after content was seen
    ContentSeenGraceExceeded,
    /// No heartbeat state transition (Live<->Hung) for heartbeat_transition_threshold_secs
    HeartbeatTransitionSilence,
    /// Retrying the same bead more than retry_threshold times
    RepeatedRetry,
}

/// Events emitted by the stuck detector
#[derive(Debug, Clone)]
pub enum StuckDetectorEvent {
    /// Worker is stuck
    Stuck(StuckAlert),
    /// Worker is no longer stuck (cleared on new event)
    Cleared { worker: String, bead: String },
}

/// Per-worker stuck detector state
#[derive(Debug, Clone)]
struct WorkerStuckState {
    bead: Option<String>,
    adapter: Option<String>,
    started_at: Option<DateTime<Utc>>,
    last_event_at: Option<DateTime<Utc>>,
    saw_content: bool,
    alert_fired: bool,
    /// Last heartbeat timestamp
    last_heartbeat_at: Option<DateTime<Utc>>,
    /// Last heartbeat state transition timestamp (Live<->Hung)
    last_transition_at: Option<DateTime<Utc>>,
    /// Retry count per bead: maps bead ID to number of retries
    retry_count: u32,
    /// Previous bead for detecting retries (when worker starts a new bead, check if it's the same as before)
    previous_bead: Option<String>,
}

impl Default for WorkerStuckState {
    fn default() -> Self {
        Self {
            bead: None,
            adapter: None,
            started_at: None,
            last_event_at: None,
            saw_content: false,
            alert_fired: false,
            last_heartbeat_at: None,
            last_transition_at: None,
            retry_count: 0,
            previous_bead: None,
        }
    }
}

/// Three-timer stuck detector
///
/// Monitors worker events and fires stuck alerts when:
/// 1. No events for idle_timeout_secs (idle_timeout)
/// 2. Total duration exceeds max_runtime_secs (max_runtime)
/// 3. No events for content_seen_grace_secs after seeing content (content_seen_grace)
///
/// Uses per-worker-type (adapter) configuration when available, falling back to default.
#[derive(Debug)]
pub struct StuckDetector {
    config_map: StuckDetectorConfigMap,
    event_tx: broadcast::Sender<StuckDetectorEvent>,
    /// Keyed by worker name
    state: Arc<Mutex<HashMap<String, WorkerStuckState>>>,
}

impl StuckDetector {
    /// Create a new stuck detector with default configuration
    pub fn new() -> Self {
        Self::with_config_map(StuckDetectorConfigMap::default())
    }

    /// Create a new stuck detector with custom configuration (single config for all)
    pub fn with_config(config: StuckDetectorConfig) -> Self {
        let config_map = StuckDetectorConfigMap {
            default: config,
            adapters: std::collections::HashMap::new(),
        };
        Self::with_config_map(config_map)
    }

    /// Create a new stuck detector with per-worker-type configuration map
    pub fn with_config_map(config_map: StuckDetectorConfigMap) -> Self {
        let (event_tx, _) = broadcast::channel(256);
        Self {
            config_map,
            event_tx,
            state: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Subscribe to stuck detector events
    pub fn subscribe(&self) -> broadcast::Receiver<StuckDetectorEvent> {
        self.event_tx.subscribe()
    }

    /// Get the current configuration map
    pub fn config_map(&self) -> &StuckDetectorConfigMap {
        &self.config_map
    }

    /// Update configuration map (hot-reload)
    pub fn update_config_map(&mut self, config_map: StuckDetectorConfigMap) {
        self.config_map = config_map;
        info!("Stuck detector config map updated");
    }

    /// Update single configuration for all workers (backward compatibility)
    pub fn update_config(&mut self, config: StuckDetectorConfig) {
        self.config_map.default = config;
        info!("Stuck detector config updated");
    }

    /// Get configuration for a specific adapter
    fn get_config_for_adapter(&self, adapter: &str) -> StuckDetectorConfig {
        self.config_map.get_for_adapter(adapter)
    }

    /// Called when a worker starts executing a bead
    pub fn on_worker_started(
        &self,
        worker: &str,
        bead: &str,
        adapter: Option<&str>,
        started_at: DateTime<Utc>,
    ) {
        let mut guard = self.state.lock().unwrap();
        let state = guard.entry(worker.to_string()).or_default();

        // Check if this is a retry (same bead as before)
        let is_retry = state.previous_bead.as_deref() == Some(bead);
        let retry_count = if is_retry {
            state.retry_count + 1
        } else {
            // New bead, reset retry count
            state.previous_bead = Some(bead.to_string());
            1 // First attempt
        };

        // Reset state for new bead
        state.bead = Some(bead.to_string());
        state.adapter = adapter.map(|s| s.to_string());
        state.started_at = Some(started_at);
        state.last_event_at = Some(started_at);
        state.saw_content = false;
        state.alert_fired = false;
        state.retry_count = retry_count;

        debug!(
            "Worker {} started bead {} at {} (adapter: {:?}, attempt: {})",
            worker, bead, started_at, state.adapter, retry_count
        );
    }

    /// Called when any event is received from a worker
    ///
    /// `has_content` should be true if the event represents real content (not just
    /// a heartbeat or status update). This is used to determine when to apply
    /// the content_seen_grace period.
    pub fn on_worker_event(&self, worker: &str, has_content: bool) {
        let mut guard = self.state.lock().unwrap();
        let state = guard.entry(worker.to_string()).or_default();

        let now = Utc::now();
        state.last_event_at = Some(now);

        if has_content {
            state.saw_content = true;
        }

        // Clear alert if one was previously fired (worker is now responsive)
        if state.alert_fired {
            state.alert_fired = false;
            if let Some(ref bead) = state.bead {
                let _ = self.event_tx.send(StuckDetectorEvent::Cleared {
                    worker: worker.to_string(),
                    bead: bead.clone(),
                });
            }
        }

        debug!(
            "Worker {} event: has_content={}, saw_content={}",
            worker, has_content, state.saw_content
        );
    }

    /// Called when a worker completes (successful or failed)
    pub fn on_worker_complete(&self, worker: &str) {
        let mut guard = self.state.lock().unwrap();
        guard.remove(worker);
        debug!("Worker {} completed, removed from stuck detector", worker);
    }

    /// Called when a heartbeat state transition occurs
    ///
    /// This tracks liveness transitions (Live<->Hung) to detect workers that
    /// are stuck in a state with no transitions.
    pub fn on_heartbeat_state_transition(
        &self,
        worker: &str,
        heartbeat_at: DateTime<Utc>,
        _old_state: crate::heartbeats::WorkerLiveness,
        _new_state: crate::heartbeats::WorkerLiveness,
    ) {
        let mut guard = self.state.lock().unwrap();
        let state = guard.entry(worker.to_string()).or_default();

        // Update last heartbeat and transition timestamps
        state.last_heartbeat_at = Some(heartbeat_at);
        state.last_transition_at = Some(heartbeat_at);

        debug!(
            "Worker {} heartbeat transition at {}: {:?} -> {:?}",
            worker, heartbeat_at, _old_state, _new_state
        );
    }

    /// Called when a heartbeat is received (no state change)
    ///
    /// This updates the last heartbeat timestamp without updating the transition time.
    pub fn on_heartbeat(&self, worker: &str, heartbeat_at: DateTime<Utc>) {
        let mut guard = self.state.lock().unwrap();
        let state = guard.entry(worker.to_string()).or_default();

        // Only update last heartbeat, not transition time
        state.last_heartbeat_at = Some(heartbeat_at);

        debug!("Worker {} heartbeat at {}", worker, heartbeat_at);
    }

    /// Check for stuck workers (called periodically by supervisor)
    pub fn check_stuck_workers(&self) {
        let mut guard = self.state.lock().unwrap();
        let now = Utc::now();

        for (worker, state) in guard.iter_mut() {
            // Skip if no active bead or already fired alert
            if state.bead.is_none() || state.alert_fired {
                continue;
            }

            let Some(started_at) = state.started_at else {
                continue;
            };
            let Some(last_event_at) = state.last_event_at else {
                continue;
            };
            let Some(ref bead) = state.bead else { continue };

            // Get config for this worker's adapter type
            let adapter = state.adapter.as_deref().unwrap_or("unknown");
            let config = self.get_config_for_adapter(adapter);

            let elapsed_secs = (now - started_at).num_seconds().max(0) as u64;
            let idle_secs = (now - last_event_at).num_seconds().max(0) as u64;

            // Determine which idle timeout to apply
            let idle_threshold = if state.saw_content {
                config.content_seen_grace_secs
            } else {
                config.idle_timeout_secs
            };

            // Check for stuck conditions
            // 1. Check retry threshold first (most specific)
            let alert = if state.retry_count > config.retry_threshold {
                let transition_secs = if let Some(ts) = state.last_transition_at {
                    (now - ts).num_seconds().max(0)
                } else {
                    0
                };
                Some(StuckAlert {
                    worker: worker.clone(),
                    bead: bead.clone(),
                    started_at,
                    last_event_at,
                    elapsed_secs: elapsed_secs as i64,
                    idle_secs: idle_secs as i64,
                    saw_content: state.saw_content,
                    reason: StuckReason::RepeatedRetry,
                    message: format!(
                        "Worker '{}' on bead '{}' has retried {} times (threshold: {}, adapter: {})",
                        worker, bead, state.retry_count, config.retry_threshold, adapter
                    ),
                    last_heartbeat_at: state.last_heartbeat_at,
                    last_transition_at: state.last_transition_at,
                    retry_count: state.retry_count,
                })
            } else if let Some(last_transition) = state.last_transition_at {
                // 2. Check heartbeat transition silence
                let transition_silence_secs = (now - last_transition).num_seconds().max(0) as u64;
                if transition_silence_secs >= config.heartbeat_transition_threshold_secs {
                    Some(StuckAlert {
                        worker: worker.clone(),
                        bead: bead.clone(),
                        started_at,
                        last_event_at,
                        elapsed_secs: elapsed_secs as i64,
                        idle_secs: idle_secs as i64,
                        saw_content: state.saw_content,
                        reason: StuckReason::HeartbeatTransitionSilence,
                        message: format!(
                            "Worker '{}' on bead '{}' has no heartbeat state transition for {}s (threshold: {}s, adapter: {})",
                            worker, bead, transition_silence_secs, config.heartbeat_transition_threshold_secs, adapter
                        ),
                        last_heartbeat_at: state.last_heartbeat_at,
                        last_transition_at: state.last_transition_at,
                        retry_count: state.retry_count,
                    })
                } else if idle_secs >= idle_threshold {
                    // 3. Check idle timeout (original logic)
                    Some(StuckAlert {
                        worker: worker.clone(),
                        bead: bead.clone(),
                        started_at,
                        last_event_at,
                        elapsed_secs: elapsed_secs as i64,
                        idle_secs: idle_secs as i64,
                        saw_content: state.saw_content,
                        reason: if state.saw_content {
                            StuckReason::ContentSeenGraceExceeded
                        } else {
                            StuckReason::IdleTimeout
                        },
                        message: format!(
                            "Worker '{}' on bead '{}' has been idle for {}s (threshold: {}s, adapter: {}). {}",
                            worker,
                            bead,
                            idle_secs,
                            idle_threshold,
                            adapter,
                            if state.saw_content {
                                "Content was seen, using extended grace period"
                            } else {
                                "No content seen yet"
                            }
                        ),
                        last_heartbeat_at: state.last_heartbeat_at,
                        last_transition_at: state.last_transition_at,
                        retry_count: state.retry_count,
                    })
                } else if elapsed_secs >= config.max_runtime_secs {
                    Some(StuckAlert {
                        worker: worker.clone(),
                        bead: bead.clone(),
                        started_at,
                        last_event_at,
                        elapsed_secs: elapsed_secs as i64,
                        idle_secs: idle_secs as i64,
                        saw_content: state.saw_content,
                        reason: StuckReason::MaxRuntimeExceeded,
                        message: format!(
                            "Worker '{}' on bead '{}' exceeded max runtime of {}s (elapsed: {}s, adapter: {})",
                            worker, bead, config.max_runtime_secs, elapsed_secs, adapter
                        ),
                        last_heartbeat_at: state.last_heartbeat_at,
                        last_transition_at: state.last_transition_at,
                        retry_count: state.retry_count,
                    })
                } else {
                    None
                }
            } else if idle_secs >= idle_threshold {
                // No transition yet, fall back to idle timeout check
                Some(StuckAlert {
                    worker: worker.clone(),
                    bead: bead.clone(),
                    started_at,
                    last_event_at,
                    elapsed_secs: elapsed_secs as i64,
                    idle_secs: idle_secs as i64,
                    saw_content: state.saw_content,
                    reason: if state.saw_content {
                        StuckReason::ContentSeenGraceExceeded
                    } else {
                        StuckReason::IdleTimeout
                    },
                    message: format!(
                        "Worker '{}' on bead '{}' has been idle for {}s (threshold: {}s, adapter: {}). {}",
                        worker,
                        bead,
                        idle_secs,
                        idle_threshold,
                        adapter,
                        if state.saw_content {
                            "Content was seen, using extended grace period"
                        } else {
                            "No content seen yet"
                        }
                    ),
                    last_heartbeat_at: state.last_heartbeat_at,
                    last_transition_at: state.last_transition_at,
                    retry_count: state.retry_count,
                })
            } else if elapsed_secs >= config.max_runtime_secs {
                Some(StuckAlert {
                    worker: worker.clone(),
                    bead: bead.clone(),
                    started_at,
                    last_event_at,
                    elapsed_secs: elapsed_secs as i64,
                    idle_secs: idle_secs as i64,
                    saw_content: state.saw_content,
                    reason: StuckReason::MaxRuntimeExceeded,
                    message: format!(
                        "Worker '{}' on bead '{}' exceeded max runtime of {}s (elapsed: {}s, adapter: {})",
                        worker, bead, config.max_runtime_secs, elapsed_secs, adapter
                    ),
                    last_heartbeat_at: state.last_heartbeat_at,
                    last_transition_at: state.last_transition_at,
                    retry_count: state.retry_count,
                })
            } else {
                None
            };

            if let Some(alert) = alert {
                state.alert_fired = true;
                warn!("{}", alert.message);
                crate::metrics::metrics().hoop_worker_stuck_total.inc();
                let _ = self.event_tx.send(StuckDetectorEvent::Stuck(alert));
            }
        }
    }

    /// Get status for all tracked workers
    pub fn status_all(&self) -> Vec<WorkerStuckStatus> {
        let guard = self.state.lock().unwrap();
        let now = Utc::now();

        guard
            .iter()
            .map(|(worker, state)| {
                let (elapsed_secs, idle_secs) =
                    if let (Some(started), Some(last)) = (state.started_at, state.last_event_at) {
                        (
                            Some((now - started).num_seconds().max(0)),
                            Some((now - last).num_seconds().max(0)),
                        )
                    } else {
                        (None, None)
                    };

                WorkerStuckStatus {
                    worker: worker.clone(),
                    bead: state.bead.clone(),
                    started_at: state.started_at,
                    last_event_at: state.last_event_at,
                    elapsed_secs,
                    idle_secs,
                    saw_content: state.saw_content,
                    alert_fired: state.alert_fired,
                }
            })
            .collect()
    }

    /// Load stuck detector configuration from `~/.hoop/config.yml`.
    ///
    /// Reads the `stuck_detector` section. Falls back to defaults if the section or file
    /// is missing, so the daemon always starts with a valid configuration.
    ///
    /// The config structure:
    /// ```yaml
    /// stuck_detector:
    ///   default:
    ///     idle_timeout_secs: 180
    ///     max_runtime_secs: 3600
    ///     content_seen_grace_secs: 600
    ///     heartbeat_transition_threshold_secs: 300
    ///     retry_threshold: 3
    ///   adapters:
    ///     claude:
    ///       idle_timeout_secs: 120
    ///       heartbeat_transition_threshold_secs: 180
    ///     zai:
    ///       idle_timeout_secs: 300
    /// ```
    pub fn load_config() -> StuckDetectorConfigMap {
        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        let config_path = home.join(".hoop").join("config.yml");

        if !config_path.exists() {
            debug!(
                "Config file not found at {}, using default stuck detector config",
                config_path.display()
            );
            return StuckDetectorConfigMap::default();
        }

        match std::fs::read_to_string(&config_path) {
            Ok(contents) => {
                match serde_yaml::from_str::<serde_yaml::Value>(&contents) {
                    Ok(root) => {
                        // Look for the stuck_detector section
                        if let Some(sd_value) = root.get("stuck_detector") {
                            match serde_yaml::from_value::<StuckDetectorConfigMap>(
                                sd_value.clone(),
                            ) {
                                Ok(config_map) => {
                                    info!(
                                        "Loaded stuck detector config from {} ({} adapter configs)",
                                        config_path.display(),
                                        config_map.adapters.len()
                                    );
                                    config_map
                                }
                                Err(e) => {
                                    warn!(
                                        "Failed to parse stuck_detector config from {}: {}, using defaults",
                                        config_path.display(), e
                                    );
                                    StuckDetectorConfigMap::default()
                                }
                            }
                        } else {
                            debug!("No stuck_detector section in config.yml, using defaults");
                            StuckDetectorConfigMap::default()
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Failed to parse config.yml from {}: {}, using defaults",
                            config_path.display(), e
                        );
                        StuckDetectorConfigMap::default()
                    }
                }
            }
            Err(e) => {
                warn!(
                    "Failed to read config.yml from {}: {}, using defaults",
                    config_path.display(), e
                );
                StuckDetectorConfigMap::default()
            }
        }
    }
}

impl Default for StuckDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// REST-facing status for a single worker's stuck detector state
#[derive(Debug, Clone, Serialize)]
pub struct WorkerStuckStatus {
    pub worker: String,
    pub bead: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub last_event_at: Option<DateTime<Utc>>,
    pub elapsed_secs: Option<i64>,
    pub idle_secs: Option<i64>,
    pub saw_content: bool,
    pub alert_fired: bool,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = StuckDetectorConfig::default();
        assert_eq!(config.idle_timeout_secs, DEFAULT_IDLE_TIMEOUT_SECS);
        assert_eq!(config.max_runtime_secs, DEFAULT_MAX_RUNTIME_SECS);
        assert_eq!(
            config.content_seen_grace_secs,
            DEFAULT_CONTENT_SEEN_GRACE_SECS
        );
    }

    #[test]
    fn test_on_worker_started() {
        let detector = StuckDetector::new();
        let worker = "alpha";
        let bead = "bd-test";
        let started_at = Utc::now();

        detector.on_worker_started(worker, bead, Some("claude"), started_at);

        let status = detector.status_all();
        assert_eq!(status.len(), 1);
        assert_eq!(status[0].worker, worker);
        assert_eq!(status[0].bead, Some(bead.to_string()));
        assert_eq!(status[0].started_at, Some(started_at));
        assert!(!status[0].saw_content);
        assert!(!status[0].alert_fired);
    }

    #[test]
    fn test_on_worker_event_without_content() {
        let detector = StuckDetector::new();
        let worker = "alpha";
        let bead = "bd-test";
        let started_at = Utc::now();

        detector.on_worker_started(worker, bead, Some("claude"), started_at);
        detector.on_worker_event(worker, false);

        let status = detector.status_all();
        assert_eq!(status.len(), 1);
        assert!(!status[0].saw_content);
    }

    #[test]
    fn test_on_worker_event_with_content() {
        let detector = StuckDetector::new();
        let worker = "alpha";
        let bead = "bd-test";
        let started_at = Utc::now();

        detector.on_worker_started(worker, bead, Some("claude"), started_at);
        detector.on_worker_event(worker, true);

        let status = detector.status_all();
        assert_eq!(status.len(), 1);
        assert!(status[0].saw_content);
    }

    #[test]
    fn test_on_worker_complete() {
        let detector = StuckDetector::new();
        let worker = "alpha";
        let bead = "bd-test";
        let started_at = Utc::now();

        detector.on_worker_started(worker, bead, Some("claude"), started_at);
        detector.on_worker_complete(worker);

        let status = detector.status_all();
        assert_eq!(status.len(), 0);
    }

    #[test]
    fn test_idle_timeout_detection() {
        let config = StuckDetectorConfig {
            idle_timeout_secs: 1,
            max_runtime_secs: 3600,
            content_seen_grace_secs: 600,
            heartbeat_transition_threshold_secs: DEFAULT_HEARTBEAT_TRANSITION_THRESHOLD_SECS,
            retry_threshold: DEFAULT_RETRY_THRESHOLD,
        };
        let detector = StuckDetector::with_config(config);
        let worker = "alpha";
        let bead = "bd-test";
        let started_at = Utc::now();

        detector.on_worker_started(worker, bead, Some("claude"), started_at);

        // Wait for idle timeout
        std::thread::sleep(std::time::Duration::from_secs(2));
        detector.check_stuck_workers();

        let status = detector.status_all();
        assert_eq!(status.len(), 1);
        assert!(status[0].alert_fired);
    }

    #[test]
    fn test_max_runtime_detection() {
        let config = StuckDetectorConfig {
            idle_timeout_secs: 180,
            max_runtime_secs: 1,
            content_seen_grace_secs: 600,
            heartbeat_transition_threshold_secs: DEFAULT_HEARTBEAT_TRANSITION_THRESHOLD_SECS,
            retry_threshold: DEFAULT_RETRY_THRESHOLD,
        };
        let detector = StuckDetector::with_config(config);
        let worker = "alpha";
        let bead = "bd-test";
        let started_at = Utc::now();

        detector.on_worker_started(worker, bead, Some("claude"), started_at);

        // Wait for max runtime
        std::thread::sleep(std::time::Duration::from_secs(2));
        detector.check_stuck_workers();

        let status = detector.status_all();
        assert_eq!(status.len(), 1);
        assert!(status[0].alert_fired);
    }

    #[test]
    fn test_content_seen_grace_prevents_false_positive() {
        // This test verifies that after content is seen, we use the longer grace period
        let config = StuckDetectorConfig {
            idle_timeout_secs: 1,
            max_runtime_secs: 3600,
            content_seen_grace_secs: 10,
            heartbeat_transition_threshold_secs: DEFAULT_HEARTBEAT_TRANSITION_THRESHOLD_SECS,
            retry_threshold: DEFAULT_RETRY_THRESHOLD,
        };
        let detector = StuckDetector::with_config(config);
        let worker = "alpha";
        let bead = "bd-test";
        let started_at = Utc::now();

        detector.on_worker_started(worker, bead, Some("claude"), started_at);
        detector.on_worker_event(worker, true); // Mark content seen

        // Wait for short idle timeout (should NOT trigger because content was seen)
        std::thread::sleep(std::time::Duration::from_secs(2));
        detector.check_stuck_workers();

        let status = detector.status_all();
        assert_eq!(status.len(), 1);
        // Alert should NOT fire because we use content_seen_grace (10s) not idle_timeout (1s)
        assert!(!status[0].alert_fired);
    }

    /// Test: synthetic tool-use silence doesn't trigger stuck alert (acceptance criteria)
    ///
    /// This simulates a worker that has produced real content and then goes silent
    /// during tool execution (e.g., a long-running tool). The extended content_seen_grace
    /// should prevent false positives.
    #[test]
    fn test_tool_use_silence_does_not_trigger_stuck_alert() {
        let config = StuckDetectorConfig {
            idle_timeout_secs: 1, // Short timeout for no content
            max_runtime_secs: 3600,
            content_seen_grace_secs: 10, // Longer grace after content seen
            heartbeat_transition_threshold_secs: DEFAULT_HEARTBEAT_TRANSITION_THRESHOLD_SECS,
            retry_threshold: DEFAULT_RETRY_THRESHOLD,
        };
        let detector = StuckDetector::with_config(config);
        let worker = "alpha";
        let bead = "bd-test";
        let started_at = Utc::now();

        // Worker starts and produces real content
        detector.on_worker_started(worker, bead, Some("claude"), started_at);
        detector.on_worker_event(worker, true); // Content seen!

        // Simulate tool-use silence: wait past idle_timeout but within content_seen_grace
        std::thread::sleep(std::time::Duration::from_secs(2));
        detector.check_stuck_workers();

        let status = detector.status_all();
        assert_eq!(status.len(), 1);
        // Should NOT trigger stuck alert because we're using content_seen_grace (10s)
        assert!(!status[0].alert_fired,
            "Tool-use silence should not trigger stuck alert when content_seen_grace hasn't expired");

        // Verify the state
        assert!(
            status[0].saw_content,
            "Content should have been marked as seen"
        );
    }

    /// Test: per-worker-type configuration is respected
    #[test]
    fn test_per_worker_type_configuration() {
        let mut config_map = StuckDetectorConfigMap::default();
        // Configure claude with very short idle timeout
        config_map.adapters.insert(
            "claude".to_string(),
            StuckDetectorConfig {
                idle_timeout_secs: 1,
                max_runtime_secs: 3600,
                content_seen_grace_secs: 5,
                heartbeat_transition_threshold_secs: DEFAULT_HEARTBEAT_TRANSITION_THRESHOLD_SECS,
                retry_threshold: DEFAULT_RETRY_THRESHOLD,
            },
        );
        // Configure zai with much longer idle timeout
        config_map.adapters.insert(
            "zai".to_string(),
            StuckDetectorConfig {
                idle_timeout_secs: 10,
                max_runtime_secs: 3600,
                content_seen_grace_secs: 20,
                heartbeat_transition_threshold_secs: DEFAULT_HEARTBEAT_TRANSITION_THRESHOLD_SECS,
                retry_threshold: DEFAULT_RETRY_THRESHOLD,
            },
        );

        let detector = StuckDetector::with_config_map(config_map);

        // Claude worker should timeout quickly
        let started_at = Utc::now();
        detector.on_worker_started("alpha", "bd-1", Some("claude"), started_at);
        std::thread::sleep(std::time::Duration::from_secs(2));
        detector.check_stuck_workers();

        let status = detector.status_all();
        let claude_status = status.iter().find(|s| s.worker == "alpha").unwrap();
        assert!(
            claude_status.alert_fired,
            "Claude worker should trigger alert with 1s idle timeout"
        );

        // Zai worker should NOT timeout yet
        let started_at = Utc::now();
        detector.on_worker_started("beta", "bd-2", Some("zai"), started_at);
        std::thread::sleep(std::time::Duration::from_secs(2));
        detector.check_stuck_workers();

        let status = detector.status_all();
        let zai_status = status.iter().find(|s| s.worker == "beta").unwrap();
        assert!(
            !zai_status.alert_fired,
            "Zai worker should NOT trigger alert with 10s idle timeout after only 2s"
        );
    }

    #[test]
    fn test_event_clears_alert() {
        let config = StuckDetectorConfig {
            idle_timeout_secs: 1,
            max_runtime_secs: 3600,
            content_seen_grace_secs: 600,
            heartbeat_transition_threshold_secs: DEFAULT_HEARTBEAT_TRANSITION_THRESHOLD_SECS,
            retry_threshold: DEFAULT_RETRY_THRESHOLD,
        };
        let detector = StuckDetector::with_config(config);
        let worker = "alpha";
        let bead = "bd-test";
        let started_at = Utc::now();

        detector.on_worker_started(worker, bead, Some("claude"), started_at);

        // Wait for idle timeout
        std::thread::sleep(std::time::Duration::from_secs(2));
        detector.check_stuck_workers();

        let status = detector.status_all();
        assert!(status[0].alert_fired);

        // New event should clear the alert
        detector.on_worker_event(worker, false);
        let status = detector.status_all();
        assert!(!status[0].alert_fired);
    }

    #[test]
    fn test_subscribe() {
        let mut detector = StuckDetector::new();
        let mut rx = detector.subscribe();

        // Trigger an alert
        let config = StuckDetectorConfig {
            idle_timeout_secs: 1,
            max_runtime_secs: 3600,
            content_seen_grace_secs: 600,
            heartbeat_transition_threshold_secs: DEFAULT_HEARTBEAT_TRANSITION_THRESHOLD_SECS,
            retry_threshold: DEFAULT_RETRY_THRESHOLD,
        };
        detector.update_config(config);

        let worker = "alpha";
        let bead = "bd-test";
        let started_at = Utc::now();

        detector.on_worker_started(worker, bead, Some("claude"), started_at);
        std::thread::sleep(std::time::Duration::from_secs(2));
        detector.check_stuck_workers();

        // Should receive an event
        let event = rx.try_recv();
        assert!(event.is_ok());
    }

    /// Test: default config includes heartbeat transition and retry thresholds
    #[test]
    fn test_default_config_includes_new_fields() {
        let config = StuckDetectorConfig::default();
        assert_eq!(
            config.heartbeat_transition_threshold_secs,
            DEFAULT_HEARTBEAT_TRANSITION_THRESHOLD_SECS
        );
        assert_eq!(config.retry_threshold, DEFAULT_RETRY_THRESHOLD);
    }

    /// Test: heartbeat transition silence detection (§C1, hoop-ttb.3.25)
    ///
    /// Verifies that a worker with no heartbeat state transition (Live<->Hung)
    /// for the configured threshold triggers a stuck alert.
    #[test]
    fn test_heartbeat_transition_silence_detection() {
        let config = StuckDetectorConfig {
            idle_timeout_secs: 60, // Longer than transition threshold
            max_runtime_secs: 3600,
            content_seen_grace_secs: 600,
            heartbeat_transition_threshold_secs: 1, // Short threshold for testing
            retry_threshold: 3,
        };
        let detector = StuckDetector::with_config(config);
        let worker = "alpha";
        let bead = "bd-test";
        let started_at = Utc::now();

        // Worker starts executing
        detector.on_worker_started(worker, bead, Some("claude"), started_at);

        // Simulate a heartbeat state transition (Live -> Hung)
        let transition_time = Utc::now();
        detector.on_heartbeat_state_transition(
            worker,
            transition_time,
            crate::heartbeats::WorkerLiveness::Live,
            crate::heartbeats::WorkerLiveness::Hung,
        );

        // Wait for transition threshold to expire
        std::thread::sleep(std::time::Duration::from_secs(2));
        detector.check_stuck_workers();

        let status = detector.status_all();
        assert_eq!(status.len(), 1);
        assert!(
            status[0].alert_fired,
            "Heartbeat transition silence should trigger alert"
        );
    }

    /// Test: repeated retry detection (§C1, hoop-ttb.3.25)
    ///
    /// Verifies that a worker retrying the same bead more than the configured
    /// threshold triggers a stuck alert.
    #[test]
    fn test_repeated_retry_detection() {
        let config = StuckDetectorConfig {
            idle_timeout_secs: 60,
            max_runtime_secs: 3600,
            content_seen_grace_secs: 600,
            heartbeat_transition_threshold_secs: 300,
            retry_threshold: 2, // Low threshold for testing
        };
        let detector = StuckDetector::with_config(config);
        let worker = "alpha";
        let bead = "bd-test";

        // First attempt
        detector.on_worker_started(worker, bead, Some("claude"), Utc::now());
        detector.on_worker_complete(worker); // Complete

        // Second attempt (same bead)
        detector.on_worker_started(worker, bead, Some("claude"), Utc::now());
        detector.on_worker_complete(worker); // Complete

        // Third attempt (same bead - should trigger alert)
        detector.on_worker_started(worker, bead, Some("claude"), Utc::now());
        detector.check_stuck_workers();

        let status = detector.status_all();
        assert_eq!(status.len(), 1);
        assert!(
            status[0].alert_fired,
            "Repeated retry should trigger alert after threshold exceeded"
        );
    }

    /// Test: new bead resets retry count
    ///
    /// Verifies that starting a different bead resets the retry counter.
    #[test]
    fn test_new_bead_resets_retry_count() {
        let config = StuckDetectorConfig {
            idle_timeout_secs: 60,
            max_runtime_secs: 3600,
            content_seen_grace_secs: 600,
            heartbeat_transition_threshold_secs: 300,
            retry_threshold: 2,
        };
        let detector = StuckDetector::with_config(config);
        let worker = "alpha";

        // Retry bead-1 twice
        detector.on_worker_started(worker, "bd-1", Some("claude"), Utc::now());
        detector.on_worker_complete(worker);

        detector.on_worker_started(worker, "bd-1", Some("claude"), Utc::now());
        detector.on_worker_complete(worker);

        // Switch to a different bead - should reset retry count
        detector.on_worker_started(worker, "bd-2", Some("claude"), Utc::now());
        detector.check_stuck_workers();

        let status = detector.status_all();
        assert_eq!(status.len(), 1);
        assert!(
            !status[0].alert_fired,
            "Switching to a new bead should reset retry count"
        );
    }

    /// Test: heartbeat without transition doesn't reset transition timer
    ///
    /// Verifies that regular heartbeats (without state change) don't update
    /// the last_transition_at timestamp.
    #[test]
    fn test_heartbeat_without_transition_doesnt_reset_transition_timer() {
        let config = StuckDetectorConfig {
            idle_timeout_secs: 60,
            max_runtime_secs: 3600,
            content_seen_grace_secs: 600,
            heartbeat_transition_threshold_secs: 1,
            retry_threshold: 3,
        };
        let detector = StuckDetector::with_config(config);
        let worker = "alpha";
        let bead = "bd-test";
        let started_at = Utc::now();

        detector.on_worker_started(worker, bead, Some("claude"), started_at);

        // Initial transition
        let transition_time = Utc::now();
        detector.on_heartbeat_state_transition(
            worker,
            transition_time,
            crate::heartbeats::WorkerLiveness::Live,
            crate::heartbeats::WorkerLiveness::Hung,
        );

        // Send regular heartbeats (no state change)
        std::thread::sleep(std::time::Duration::from_millis(500));
        detector.on_heartbeat(worker, Utc::now());

        // Wait for transition threshold to expire
        std::thread::sleep(std::time::Duration::from_secs(2));
        detector.check_stuck_workers();

        let status = detector.status_all();
        assert_eq!(status.len(), 1);
        assert!(
            status[0].alert_fired,
            "Regular heartbeats should not reset transition silence timer"
        );
    }

    /// Test: per-worker-type configuration for heartbeat transition threshold
    ///
    /// Verifies that different adapter types can have different transition thresholds.
    #[test]
    fn test_per_worker_type_heartbeat_transition_config() {
        let mut config_map = StuckDetectorConfigMap::default();
        // Configure claude with very short transition threshold
        config_map.adapters.insert(
            "claude".to_string(),
            StuckDetectorConfig {
                idle_timeout_secs: 60,
                max_runtime_secs: 3600,
                content_seen_grace_secs: 600,
                heartbeat_transition_threshold_secs: 1,
                retry_threshold: 3,
            },
        );
        // Configure zai with much longer transition threshold
        config_map.adapters.insert(
            "zai".to_string(),
            StuckDetectorConfig {
                idle_timeout_secs: 60,
                max_runtime_secs: 3600,
                content_seen_grace_secs: 600,
                heartbeat_transition_threshold_secs: 10,
                retry_threshold: 3,
            },
        );

        let detector = StuckDetector::with_config_map(config_map);

        // Claude worker should trigger transition silence quickly
        let started_at = Utc::now();
        detector.on_worker_started("alpha", "bd-1", Some("claude"), started_at);
        detector.on_heartbeat_state_transition(
            "alpha",
            started_at,
            crate::heartbeats::WorkerLiveness::Live,
            crate::heartbeats::WorkerLiveness::Hung,
        );
        std::thread::sleep(std::time::Duration::from_secs(2));
        detector.check_stuck_workers();

        let status = detector.status_all();
        let claude_status = status.iter().find(|s| s.worker == "alpha").unwrap();
        assert!(
            claude_status.alert_fired,
            "Claude worker should trigger transition silence alert with 1s threshold"
        );

        // Zai worker should NOT trigger yet
        let started_at = Utc::now();
        detector.on_worker_started("beta", "bd-2", Some("zai"), started_at);
        detector.on_heartbeat_state_transition(
            "beta",
            started_at,
            crate::heartbeats::WorkerLiveness::Live,
            crate::heartbeats::WorkerLiveness::Hung,
        );
        std::thread::sleep(std::time::Duration::from_secs(2));
        detector.check_stuck_workers();

        let status = detector.status_all();
        let zai_status = status.iter().find(|s| s.worker == "beta").unwrap();
        assert!(
            !zai_status.alert_fired,
            "Zai worker should NOT trigger transition silence alert with 10s threshold after only 2s"
        );
    }

    /// Test: stuck alert includes all required fields (§C1, hoop-ttb.3.25)
    ///
    /// Verifies that StuckAlert includes last_heartbeat_at, last_transition_at,
    /// and retry_count for operator visibility.
    #[test]
    fn test_stuck_alert_includes_all_required_fields() {
        let mut rx = StuckDetector::new().subscribe();

        let config = StuckDetectorConfig {
            idle_timeout_secs: 1,
            max_runtime_secs: 3600,
            content_seen_grace_secs: 600,
            heartbeat_transition_threshold_secs: 300,
            retry_threshold: 3,
        };
        let detector = StuckDetector::with_config(config);
        let worker = "alpha";
        let bead = "bd-test";
        let started_at = Utc::now();

        detector.on_worker_started(worker, bead, Some("claude"), started_at);

        // Add a heartbeat and transition
        let hb_time = Utc::now();
        detector.on_heartbeat(worker, hb_time);
        detector.on_heartbeat_state_transition(
            worker,
            hb_time,
            crate::heartbeats::WorkerLiveness::Live,
            crate::heartbeats::WorkerLiveness::Hung,
        );

        std::thread::sleep(std::time::Duration::from_secs(2));
        detector.check_stuck_workers();

        // Check that we received an alert with all required fields
        let event = rx.try_recv();
        assert!(event.is_ok());
        if let Ok(StuckDetectorEvent::Stuck(alert)) = event {
            assert_eq!(alert.worker, worker);
            assert_eq!(alert.bead, bead);
            assert!(alert.last_heartbeat_at.is_some());
            assert!(alert.last_transition_at.is_some());
            assert_eq!(alert.retry_count, 1);
        } else {
            panic!("Expected Stuck event");
        }
    }
}
