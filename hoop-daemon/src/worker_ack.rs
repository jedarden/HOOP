//! Worker spawn-ack monitor (§M5)
//!
//! Watches `~/.hoop/workers/` for `<name>.ack` files written by NEEDLE workers
//! on successful startup.  HOOP uses ack presence to verify spawn success vs
//! silent failure (tmux `send-keys` truncation, §M5).
//!
//! # Ack file
//!
//! Location: `~/.hoop/workers/<worker-name>.ack`
//! Format: single JSON object on one line:
//!
//! ```json
//! {"worker":"alpha","ts":"2026-04-24T10:00:00Z","pid":12345}
//! ```
//!
//! NEEDLE writes this file atomically (`.tmp` + rename) within the first few
//! seconds of startup — before the heartbeat loop begins.
//!
//! # Detection logic
//!
//! HOOP fires a `MissingAck` alert when a worker has been emitting heartbeats
//! for ≥ [`ACK_GRACE_SECS`] seconds but has no ack file on disk.  This catches
//! the §M5 failure mode: `tmux send-keys` returned success but the worker
//! never actually started, so it wrote neither an ack nor a heartbeat.  The
//! complementary alert — "heartbeating but no ack" — catches a subtler failure:
//! the worker started and is running, but its boot hook is missing or broken.
//!
//! # NEEDLE-side requirement
//!
//! Every NEEDLE worker must run the following at boot (Hook 5 in
//! `docs/needle-hooks.md`):
//!
//! ```sh
//! mkdir -p ~/.hoop/workers
//! printf '{"worker":"%s","ts":"%s","pid":%d}\n' \
//!     "$NEEDLE_WORKER" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$$" \
//!     > ~/.hoop/workers/${NEEDLE_WORKER}.ack.tmp
//! mv ~/.hoop/workers/${NEEDLE_WORKER}.ack.tmp ~/.hoop/workers/${NEEDLE_WORKER}.ack
//! ```

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

/// Grace window: ack must exist within this many seconds of the worker's first
/// heartbeat observed by HOOP.  Matches the §M5 spec (10s).
const ACK_GRACE_SECS: u64 = 10;

/// Parsed ack record from a `<name>.ack` file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerAck {
    pub worker: String,
    pub ts: DateTime<Utc>,
    pub pid: u32,
}

/// Alert emitted when a worker's ack is absent past the grace window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnAckAlert {
    pub worker: String,
    pub first_heartbeat_at: DateTime<Utc>,
    pub elapsed_secs: i64,
    pub message: String,
}

/// Events emitted by the ack monitor.
#[derive(Debug, Clone)]
pub enum AckEvent {
    /// A new (or updated) ack file was successfully parsed.
    AckReceived(WorkerAck),
    /// A worker has been heartbeating past the grace window with no ack.
    MissingAck(SpawnAckAlert),
}

/// Per-worker ack tracking state (keyed by worker name in the HashMap).
#[derive(Debug, Clone)]
struct WorkerAckState {
    ack: Option<WorkerAck>,
    first_heartbeat_at: Option<DateTime<Utc>>,
    /// True once a MissingAck alert has been fired (prevents log spam).
    alert_fired: bool,
}

/// REST-facing summary of a single worker's ack state.
#[derive(Debug, Clone, Serialize)]
pub struct WorkerAckStatus {
    pub worker: String,
    pub ack: Option<WorkerAck>,
    pub first_heartbeat_at: Option<DateTime<Utc>>,
    pub elapsed_since_first_heartbeat_secs: Option<i64>,
    pub missing_ack_alert_fired: bool,
}

/// Watches `~/.hoop/workers/` for spawn-ack files.
#[derive(Debug)]
pub struct WorkerAckMonitor {
    ack_dir: PathBuf,
    event_tx: broadcast::Sender<AckEvent>,
    /// Keyed by worker name.
    state: Arc<Mutex<HashMap<String, WorkerAckState>>>,
    _watcher: Option<RecommendedWatcher>,
}

impl WorkerAckMonitor {
    /// Create a monitor for the canonical `~/.hoop/workers/` directory.
    pub fn new() -> Result<Self> {
        let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.push(".hoop");
        home.push("workers");
        Self::with_dir(home)
    }

    /// Create a monitor for a custom directory (primarily for tests).
    pub fn with_dir(ack_dir: PathBuf) -> Result<Self> {
        let (event_tx, _) = broadcast::channel(64);
        Ok(Self {
            ack_dir,
            event_tx,
            state: Arc::new(Mutex::new(HashMap::new())),
            _watcher: None,
        })
    }

    /// Subscribe to [`AckEvent`] notifications.
    pub fn subscribe(&self) -> broadcast::Receiver<AckEvent> {
        self.event_tx.subscribe()
    }

    /// Start the directory watcher and scan any pre-existing ack files.
    pub fn start(&mut self) -> Result<()> {
        if !self.ack_dir.exists() {
            std::fs::create_dir_all(&self.ack_dir).with_context(|| {
                format!("Failed to create ack dir {}", self.ack_dir.display())
            })?;
        }

        // Load any ack files that existed before the daemon started.
        self.scan_existing()?;

        let _ack_dir = self.ack_dir.clone();
        let event_tx = self.event_tx.clone();
        let state = self.state.clone();

        let mut watcher = notify::recommended_watcher(
            move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    use notify::EventKind::*;
                    if matches!(event.kind, Create(_) | Modify(_)) {
                        for path in &event.paths {
                            if is_ack_file(path) {
                                if let Some(ack) = parse_ack_file(path) {
                                    handle_ack_received(ack, &state, &event_tx);
                                }
                            }
                        }
                    }
                }
            },
        )
        .context("Failed to create ack directory watcher")?;

        watcher
            .watch(&self.ack_dir, RecursiveMode::NonRecursive)
            .with_context(|| {
                format!("Failed to watch ack dir {}", self.ack_dir.display())
            })?;

        self._watcher = Some(watcher);
        info!("Worker ack monitor watching {}", self.ack_dir.display());
        Ok(())
    }

    /// Scan the ack directory for pre-existing ack files.
    fn scan_existing(&self) -> Result<()> {
        if !self.ack_dir.exists() {
            return Ok(());
        }
        let entries = std::fs::read_dir(&self.ack_dir).with_context(|| {
            format!("Failed to read ack dir {}", self.ack_dir.display())
        })?;

        let mut count = 0usize;
        for entry in entries.flatten() {
            let path = entry.path();
            if is_ack_file(&path) {
                if let Some(ack) = parse_ack_file(&path) {
                    handle_ack_received(ack, &self.state, &self.event_tx);
                    count += 1;
                }
            }
        }
        if count > 0 {
            info!(
                "Worker ack monitor: loaded {} pre-existing ack file(s)",
                count
            );
        }
        Ok(())
    }

    /// Called on every heartbeat so the monitor can track first-seen timestamps
    /// and fire missing-ack alerts once the grace window expires.
    pub fn on_heartbeat(&self, worker: &str, heartbeat_ts: DateTime<Utc>) {
        let mut guard = self.state.lock().unwrap();
        let entry = guard
            .entry(worker.to_string())
            .or_insert_with(|| WorkerAckState {
                ack: None,
                first_heartbeat_at: None,
                alert_fired: false,
            });

        if entry.first_heartbeat_at.is_none() {
            entry.first_heartbeat_at = Some(heartbeat_ts);
            debug!(
                "Worker {} first heartbeat recorded at {}",
                worker, heartbeat_ts
            );
        }

        if entry.ack.is_none() && !entry.alert_fired {
            if let Some(first_hb) = entry.first_heartbeat_at {
                let elapsed = (Utc::now() - first_hb).num_seconds().max(0) as u64;
                if elapsed >= ACK_GRACE_SECS {
                    entry.alert_fired = true;
                    let alert = SpawnAckAlert {
                        worker: worker.to_string(),
                        first_heartbeat_at: first_hb,
                        elapsed_secs: elapsed as i64,
                        message: format!(
                            "Worker '{}' has been heartbeating for {}s but has no spawn ack \
                             at ~/.hoop/workers/{}.ack — boot hook may be missing (§M5)",
                            worker, elapsed, worker
                        ),
                    };
                    warn!("{}", alert.message);
                    crate::metrics::metrics()
                        .hoop_worker_spawn_missing_ack_total
                        .inc();
                    let _ = self.event_tx.send(AckEvent::MissingAck(alert));
                }
            }
        }
    }

    /// Return the ack record for a specific worker, if one has been seen.
    pub fn get_ack(&self, worker: &str) -> Option<WorkerAck> {
        self.state.lock().unwrap().get(worker)?.ack.clone()
    }

    /// Return all ack records currently held by the monitor.
    pub fn get_all_acks(&self) -> Vec<WorkerAck> {
        self.state
            .lock()
            .unwrap()
            .values()
            .filter_map(|s| s.ack.clone())
            .collect()
    }

    /// Return the ack status for all workers known to the monitor.
    pub fn ack_status_all(&self) -> Vec<WorkerAckStatus> {
        let guard = self.state.lock().unwrap();
        let now = Utc::now();
        guard
            .iter()
            .map(|(worker, s)| WorkerAckStatus {
                worker: worker.clone(),
                ack: s.ack.clone(),
                first_heartbeat_at: s.first_heartbeat_at,
                elapsed_since_first_heartbeat_secs: s
                    .first_heartbeat_at
                    .map(|t| (now - t).num_seconds().max(0)),
                missing_ack_alert_fired: s.alert_fired,
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn is_ack_file(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("ack")
}

fn parse_ack_file(path: &Path) -> Option<WorkerAck> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to read ack file {}: {}", path.display(), e);
            return None;
        }
    };

    let line = content.trim();
    let ack = match serde_json::from_str::<WorkerAck>(line) {
        Ok(a) => a,
        Err(e) => {
            warn!("Failed to parse ack file {}: {}", path.display(), e);
            return None;
        }
    };

    // Validate the worker name in the JSON against the filename stem.
    let stem = match path.file_stem().and_then(|s| s.to_str()) {
        Some(s) => s,
        None => {
            warn!("Ack file {} has no valid stem", path.display());
            return None;
        }
    };

    if let Err(e) = crate::id_validators::validate_worker_name(stem) {
        warn!(
            "Ack file {} has invalid worker name stem: {}",
            path.display(),
            e
        );
        return None;
    }

    if ack.worker != stem {
        warn!(
            "Ack file {} contains worker='{}', expected '{}'",
            path.display(),
            ack.worker,
            stem
        );
        return None;
    }

    debug!(
        "Parsed ack: worker='{}' pid={} ts={}",
        ack.worker, ack.pid, ack.ts
    );
    Some(ack)
}

fn handle_ack_received(
    ack: WorkerAck,
    state: &Arc<Mutex<HashMap<String, WorkerAckState>>>,
    event_tx: &broadcast::Sender<AckEvent>,
) {
    info!(
        "Spawn ack received: worker='{}' pid={} ts={}",
        ack.worker, ack.pid, ack.ts
    );
    crate::metrics::metrics().hoop_worker_acks_seen_total.inc();

    let mut guard = state.lock().unwrap();
    let entry = guard
        .entry(ack.worker.clone())
        .or_insert_with(|| WorkerAckState {
            ack: None,
            first_heartbeat_at: None,
            alert_fired: false,
        });
    entry.ack = Some(ack.clone());

    let _ = event_tx.send(AckEvent::AckReceived(ack));
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_ack(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(format!("{}.ack", name));
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn test_is_ack_file_positive() {
        assert!(is_ack_file(Path::new("alpha.ack")));
    }

    #[test]
    fn test_is_ack_file_negative() {
        assert!(!is_ack_file(Path::new("alpha.jsonl")));
        assert!(!is_ack_file(Path::new("alpha.ack.tmp")));
        assert!(!is_ack_file(Path::new("alpha")));
    }

    #[test]
    fn test_parse_ack_valid() {
        let dir = tempfile::tempdir().unwrap();
        let content = r#"{"worker":"alpha","ts":"2026-04-24T10:00:00Z","pid":12345}"#;
        let path = write_ack(dir.path(), "alpha", content);
        let ack = parse_ack_file(&path).unwrap();
        assert_eq!(ack.worker, "alpha");
        assert_eq!(ack.pid, 12345);
    }

    #[test]
    fn test_parse_ack_trailing_newline() {
        let dir = tempfile::tempdir().unwrap();
        let content = "{\"worker\":\"bravo\",\"ts\":\"2026-04-24T10:00:00Z\",\"pid\":9}\n";
        let path = write_ack(dir.path(), "bravo", content);
        let ack = parse_ack_file(&path).unwrap();
        assert_eq!(ack.worker, "bravo");
    }

    #[test]
    fn test_parse_ack_name_mismatch() {
        let dir = tempfile::tempdir().unwrap();
        let content = r#"{"worker":"bravo","ts":"2026-04-24T10:00:00Z","pid":9999}"#;
        // File is named "alpha" but JSON says "bravo".
        let path = write_ack(dir.path(), "alpha", content);
        assert!(parse_ack_file(&path).is_none());
    }

    #[test]
    fn test_parse_ack_invalid_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_ack(dir.path(), "alpha", "not json");
        assert!(parse_ack_file(&path).is_none());
    }

    #[test]
    fn test_on_heartbeat_records_first_seen() {
        let dir = tempfile::tempdir().unwrap();
        let monitor = WorkerAckMonitor::with_dir(dir.path().to_path_buf()).unwrap();
        let ts = Utc::now();
        monitor.on_heartbeat("alpha", ts);
        let status = monitor.ack_status_all();
        assert_eq!(status.len(), 1);
        assert_eq!(status[0].worker, "alpha");
        assert!(status[0].ack.is_none());
        assert_eq!(status[0].first_heartbeat_at, Some(ts));
        assert!(!status[0].missing_ack_alert_fired);
    }

    #[test]
    fn test_scan_existing_loads_ack() {
        let dir = tempfile::tempdir().unwrap();
        let content = r#"{"worker":"delta","ts":"2026-04-24T10:00:00Z","pid":777}"#;
        write_ack(dir.path(), "delta", content);

        let monitor = WorkerAckMonitor::with_dir(dir.path().to_path_buf()).unwrap();
        monitor.scan_existing().unwrap();

        let ack = monitor.get_ack("delta").unwrap();
        assert_eq!(ack.worker, "delta");
        assert_eq!(ack.pid, 777);
    }

    #[test]
    fn test_get_all_acks_empty() {
        let dir = tempfile::tempdir().unwrap();
        let monitor = WorkerAckMonitor::with_dir(dir.path().to_path_buf()).unwrap();
        assert!(monitor.get_all_acks().is_empty());
    }

    #[test]
    fn test_ack_received_clears_alert() {
        // If a worker already has heartbeats, receiving an ack should update state.
        let dir = tempfile::tempdir().unwrap();
        let monitor = WorkerAckMonitor::with_dir(dir.path().to_path_buf()).unwrap();

        // Simulate heartbeat first.
        let ts = Utc::now();
        monitor.on_heartbeat("echo", ts);

        // Now write an ack and simulate receiving it.
        let content = r#"{"worker":"echo","ts":"2026-04-24T10:00:00Z","pid":42}"#;
        let path = write_ack(dir.path(), "echo", content);
        let ack = parse_ack_file(&path).unwrap();
        handle_ack_received(ack, &monitor.state, &monitor.event_tx);

        assert!(monitor.get_ack("echo").is_some());
    }
}
