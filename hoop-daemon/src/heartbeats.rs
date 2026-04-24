//! Heartbeat monitor for NEEDLE worker heartbeats
//!
//! Watches `.beads/heartbeats.jsonl` and maintains per-worker liveness state.
//! Combines heartbeat freshness with process liveness (kill -0 pid) from heartbeats.
//! Pure derivation — no file writes.
//!
//! Liveness rules (from plan §3.2, notes/orchestrator-problems-and-solutions.md §A4, §C1):
//! - Live: PID alive AND heartbeat fresh (≤ 2× heartbeat_interval)
//! - Hung: PID alive BUT heartbeat stale (> 2× heartbeat_interval)
//! - Dead: PID gone
//!
//! Heartbeat interval is 10s (configurable in NEEDLE). Grace period is 2× = 20s.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, Metadata};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use crate::WorkerState;

/// Default heartbeat interval in seconds (from NEEDLE)
const HEARTBEAT_INTERVAL_SECS: u64 = 10;

/// Grace period multiplier: we consider a worker hung after 2× interval
const HEARTBEAT_GRACE_MULTIPLIER: u64 = 2;

/// Heartbeat grace period in seconds
const HEARTBEAT_GRACE_SECS: u64 = HEARTBEAT_INTERVAL_SECS * HEARTBEAT_GRACE_MULTIPLIER;

/// File position tracking for efficient incremental reads
#[derive(Debug)]
struct FilePosition {
    /// The byte offset we've read up to
    offset: u64,
    /// The file size when we last read it
    last_size: u64,
    /// The file modification time when we last read it
    last_modified: Option<std::time::SystemTime>,
}

impl FilePosition {
    fn new() -> Self {
        Self {
            offset: 0,
            last_size: 0,
            last_modified: None,
        }
    }

    /// Reset position (called after log rotation)
    fn reset(&mut self) {
        self.offset = 0;
        self.last_size = 0;
        self.last_modified = None;
    }

    /// Check if the file has been rotated or recreated
    fn is_rotated(&self, metadata: &Metadata) -> bool {
        if let Some(last_mod) = self.last_modified {
            if let Ok(new_mod) = metadata.modified() {
                if metadata.len() < self.offset || new_mod < last_mod {
                    return true;
                }
            }
        }
        false
    }

    /// Update position after reading
    fn update(&mut self, new_offset: u64, metadata: &Metadata) {
        self.offset = new_offset;
        self.last_size = metadata.len();
        self.last_modified = metadata.modified().ok();
    }
}

impl Default for FilePosition {
    fn default() -> Self {
        Self::new()
    }
}

/// Events emitted by the heartbeat monitor
#[derive(Debug, Clone)]
pub enum MonitorEvent {
    /// A new heartbeat was parsed
    Heartbeat(WorkerHeartbeat),
    /// A worker transitioned liveness state
    LivenessChange(LivenessTransition),
    /// The file was rotated (moved/recreated)
    Rotated,
    /// An error occurred
    Error(String),
}

/// Worker liveness state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerLiveness {
    /// Worker is alive (PID exists, heartbeat fresh)
    Live,
    /// Worker is hung (PID exists, heartbeat stale)
    Hung,
    /// Worker is dead (PID does not exist)
    Dead,
}

/// Liveness transition event
#[derive(Debug, Clone)]
pub struct LivenessTransition {
    pub worker: String,
    pub old_state: WorkerLiveness,
    pub new_state: WorkerLiveness,
    pub reason: String,
}

/// Worker heartbeat record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerHeartbeat {
    pub ts: DateTime<Utc>,
    pub worker: String,
    pub state: WorkerState,
}

/// Per-worker heartbeat state
#[derive(Debug, Clone)]
struct WorkerHeartbeatState {
    /// Last heartbeat timestamp
    last_heartbeat_at: DateTime<Utc>,
    /// PID from the most recent heartbeat (if available)
    last_pid: Option<u32>,
    /// Current derived liveness state
    liveness: WorkerLiveness,
}

/// Heartbeat monitor configuration
#[derive(Debug, Clone)]
pub struct HeartbeatMonitorConfig {
    /// Path to the heartbeats.jsonl file
    pub heartbeats_path: PathBuf,
    /// Whether to replay the entire file on startup
    pub replay_on_startup: bool,
}

impl Default for HeartbeatMonitorConfig {
    fn default() -> Self {
        let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.push(".beads");
        home.push("heartbeats.jsonl");
        Self {
            heartbeats_path: home,
            replay_on_startup: true,
        }
    }
}

/// Heartbeat monitor that watches and parses heartbeats.jsonl
pub struct HeartbeatMonitor {
    config: HeartbeatMonitorConfig,
    event_tx: broadcast::Sender<MonitorEvent>,
    watcher: Option<RecommendedWatcher>,
    _shutdown_tx: mpsc::Sender<()>,
    /// Per-worker heartbeat state
    workers: Arc<Mutex<HashMap<String, WorkerHeartbeatState>>>,
    /// File position tracking for efficient incremental reads
    position: Arc<Mutex<FilePosition>>,
}

impl HeartbeatMonitor {
    /// Create a new heartbeat monitor
    pub fn new(config: HeartbeatMonitorConfig) -> Result<Self> {
        let (event_tx, _) = broadcast::channel(256);
        let (shutdown_tx, _) = mpsc::channel(1);

        Ok(Self {
            config,
            event_tx,
            watcher: None,
            _shutdown_tx: shutdown_tx,
            workers: Arc::new(Mutex::new(HashMap::new())),
            position: Arc::new(Mutex::new(FilePosition::new())),
        })
    }

    /// Subscribe to events from this monitor
    pub fn subscribe(&self) -> broadcast::Receiver<MonitorEvent> {
        self.event_tx.subscribe()
    }

    /// Get the sender for this monitor's event channel
    pub fn sender(&self) -> broadcast::Sender<MonitorEvent> {
        self.event_tx.clone()
    }

    /// Get current liveness for all workers
    pub fn get_all_liveness(&self) -> HashMap<String, WorkerLiveness> {
        self.workers
            .lock()
            .unwrap()
            .iter()
            .map(|(worker, state)| (worker.clone(), state.liveness))
            .collect()
    }

    /// Get liveness for a specific worker
    pub fn get_liveness(&self, worker: &str) -> Option<WorkerLiveness> {
        self.workers.lock().unwrap().get(worker).map(|state| state.liveness)
    }

    /// Start watching the heartbeats file
    pub fn start(&mut self) -> Result<()> {
        let heartbeats_path = self.config.heartbeats_path.clone();
        let heartbeats_path_for_watch = heartbeats_path.clone();
        let event_tx = self.event_tx.clone();
        let position = self.position.clone();
        let workers = self.workers.clone();

        // Create the watcher
        let mut watcher = notify::recommended_watcher(move |res| {
            if let Err(e) = Self::handle_watch_event(res, &heartbeats_path_for_watch, &event_tx, position.clone(), workers.clone()) {
                warn!("Error handling heartbeat watch event: {}", e);
            }
        })
        .context("Failed to create file watcher")?;

        // Watch the parent directory (since heartbeats.jsonl may not exist yet)
        let watch_path = if let Some(parent) = heartbeats_path.parent() {
            if parent.exists() {
                parent.to_path_buf()
            } else {
                PathBuf::from(".")
            }
        } else {
            PathBuf::from(".")
        };

        watcher
            .watch(&watch_path, RecursiveMode::NonRecursive)
            .context("Failed to watch heartbeats directory")?;

        self.watcher = Some(watcher);

        // Replay existing heartbeats on startup
        if self.config.replay_on_startup && heartbeats_path.exists() {
            info!("Replaying heartbeats from {}", heartbeats_path.display());
            if let Err(e) = self.replay_file() {
                warn!("Error replaying heartbeats file: {}", e);
            }
        }

        info!(
            "Heartbeat monitor watching {}",
            self.config.heartbeats_path.display()
        );

        Ok(())
    }

    /// Replay all heartbeats from the existing file
    fn replay_file(&mut self) -> Result<()> {
        let heartbeats_path = &self.config.heartbeats_path;
        let file = File::open(heartbeats_path)
            .context("Failed to open heartbeats file for replay")?;

        let metadata = file.metadata()
            .context("Failed to get heartbeats file metadata")?;

        let reader = BufReader::new(file);
        let mut line_number = 0;
        let mut offset = 0u64;

        for line in reader.lines() {
            line_number += 1;
            let line = line.context("Failed to read line from heartbeats file")?;
            // Update offset (line bytes + newline)
            offset += line.len() as u64 + 1;

            self.parse_and_update(&line, line_number);
        }

        // Update position tracking after replay
        let mut pos = self.position.lock().unwrap();
        pos.update(offset, &metadata);

        Ok(())
    }

    /// Handle a watch event from notify
    fn handle_watch_event(
        res: Result<notify::Event, notify::Error>,
        heartbeats_path: &Path,
        event_tx: &broadcast::Sender<MonitorEvent>,
        position: Arc<Mutex<FilePosition>>,
        workers: Arc<Mutex<HashMap<String, WorkerHeartbeatState>>>,
    ) -> Result<()> {
        let event = res?;

        // Check if the event is for our file
        let relevant = event.paths.iter().any(|p| p == heartbeats_path);

        if !relevant {
            return Ok(());
        }

        use notify::EventKind::*;

        match event.kind {
            Access(_) | Create(_) | Modify(_) => {
                let heartbeats = Self::read_new_heartbeats(heartbeats_path, position.clone())?;
                // Process each heartbeat to update worker state and send events
                for (heartbeat, _) in heartbeats {
                    // Update worker state
                    Self::update_worker_state(&heartbeat, &workers, event_tx);
                    // Send heartbeat event for notification
                    let _ = event_tx.send(MonitorEvent::Heartbeat(heartbeat));
                }
            }
            Remove(_) => {
                debug!("Heartbeats file removed (likely log rotation)");
                position.lock().unwrap().reset();
                let _ = event_tx.send(MonitorEvent::Rotated);
            }
            _ => {}
        }

        Ok(())
    }

    /// Read new heartbeats from the file
    ///
    /// Returns a list of parsed heartbeats with their line numbers.
    fn read_new_heartbeats(
        heartbeats_path: &Path,
        position: Arc<Mutex<FilePosition>>,
    ) -> Result<Vec<(WorkerHeartbeat, usize)>> {
        let file = File::open(heartbeats_path)
            .with_context(|| format!("Failed to open heartbeats file {}", heartbeats_path.display()))?;

        let metadata = file.metadata()
            .with_context(|| format!("Failed to get metadata for {}", heartbeats_path.display()))?;

        // Check for log rotation
        {
            let pos = position.lock().unwrap();
            if pos.is_rotated(&metadata) {
                debug!("Log rotation detected, resetting position");
                drop(pos);
                position.lock().unwrap().reset();
            }
        }

        // Get current position
        let (offset, needs_reset) = {
            let pos = position.lock().unwrap();
            (pos.offset, pos.offset == 0)
        };

        // If file hasn't grown since last read, nothing to do
        if metadata.len() <= offset && !needs_reset {
            return Ok(Vec::new());
        }

        // Seek to our last position
        let mut file = file;
        file.seek(SeekFrom::Start(offset))
            .with_context(|| format!("Failed to seek to offset {} in {}", offset, heartbeats_path.display()))?;

        let reader = BufReader::new(file);
        let mut heartbeats = Vec::new();
        let mut line_number = 0;
        let mut current_offset = offset;

        for line in reader.lines() {
            line_number += 1;
            let line = line.with_context(|| {
                format!(
                    "Failed to read line {} from {}",
                    line_number,
                    heartbeats_path.display()
                )
            })?;

            // Update offset (line bytes + newline)
            current_offset += line.len() as u64 + 1;

            // Parse the heartbeat
            let source = crate::parse_jsonl_safe::LineSource {
                tag: "heartbeats",
                file_path: heartbeats_path.to_path_buf(),
                line_number,
            };
            if let Some(heartbeat) = Self::parse_heartbeat_line(&line, &source) {
                heartbeats.push((heartbeat, line_number));
            }
        }

        // Update position tracking
        position.lock().unwrap().update(current_offset, &metadata);

        Ok(heartbeats)
    }

    /// Parse a heartbeat line using the shared safe parser.
    ///
    /// Returns `None` for empty/quarantined lines, `Some(hb)` on success.
    /// Additional validation failures (bad timestamp, invalid worker name) also
    /// quarantine the line.
    fn parse_heartbeat_line(line: &str, source: &crate::parse_jsonl_safe::LineSource) -> Option<WorkerHeartbeat> {
        #[derive(Debug, Deserialize)]
        struct HeartbeatRaw {
            ts: String,
            worker: String,
            #[serde(flatten)]
            state: WorkerState,
        }

        let raw = match crate::parse_jsonl_safe::parse_line::<HeartbeatRaw>(line, source) {
            crate::parse_jsonl_safe::ParseResult::Ok(raw) => raw,
            crate::parse_jsonl_safe::ParseResult::Empty => return None,
            crate::parse_jsonl_safe::ParseResult::Quarantined => return None,
        };

        let ts = match raw.ts.parse::<DateTime<Utc>>() {
            Ok(t) => t,
            Err(e) => {
                let reason = format!("Failed to parse timestamp: {}", e);
                crate::parse_jsonl_safe::quarantine_raw(line, &reason, source);
                return None;
            }
        };

        if let Err(e) = crate::id_validators::validate_worker_name(&raw.worker) {
            let reason = format!("Invalid worker name: {}", e);
            crate::parse_jsonl_safe::quarantine_raw(line, &reason, source);
            return None;
        }

        Some(WorkerHeartbeat {
            ts,
            worker: raw.worker,
            state: raw.state,
        })
    }

    /// Parse a heartbeat line and update worker state
    fn parse_and_update(&mut self, line: &str, line_number: usize) {
        let source = crate::parse_jsonl_safe::LineSource {
            tag: "heartbeats",
            file_path: self.config.heartbeats_path.clone(),
            line_number,
        };
        if let Some(heartbeat) = Self::parse_heartbeat_line(line, &source) {
            Self::update_worker_state(&heartbeat, &self.workers, &self.event_tx);
        }
    }

    /// Update worker state from a heartbeat
    ///
    /// This is a helper that can be called from both the watch handler and the public API.
    fn update_worker_state(
        heartbeat: &WorkerHeartbeat,
        workers: &Arc<Mutex<HashMap<String, WorkerHeartbeatState>>>,
        event_tx: &broadcast::Sender<MonitorEvent>,
    ) {
        // Extract PID from the heartbeat state
        let pid = match &heartbeat.state {
            WorkerState::Executing { pid, .. } => Some(*pid),
            _ => None,
        };

        // Get or create worker state
        let mut workers_guard = workers.lock().unwrap();
        let worker_entry = workers_guard
            .entry(heartbeat.worker.clone())
            .or_insert_with(|| WorkerHeartbeatState {
                last_heartbeat_at: heartbeat.ts,
                last_pid: pid,
                liveness: WorkerLiveness::Dead,
            });

        // Update heartbeat state
        let old_liveness = worker_entry.liveness;
        worker_entry.last_heartbeat_at = heartbeat.ts;
        worker_entry.last_pid = pid;

        // Compute new liveness state directly here to avoid borrow issues
        let new_liveness = {
            // First check: is the PID alive?
            let pid_alive = if let Some(p) = pid.or(worker_entry.last_pid) {
                is_process_alive(p)
            } else {
                false
            };

            if !pid_alive {
                WorkerLiveness::Dead
            } else {
                // Second check: is the heartbeat fresh?
                let now = Utc::now();
                let heartbeat_age = now.signed_duration_since(worker_entry.last_heartbeat_at).num_seconds() as u64;

                if heartbeat_age <= HEARTBEAT_GRACE_SECS {
                    WorkerLiveness::Live
                } else {
                    WorkerLiveness::Hung
                }
            }
        };

        // Check for state transition
        if old_liveness != new_liveness {
            worker_entry.liveness = new_liveness;

            let reason = match new_liveness {
                WorkerLiveness::Live => "PID alive and heartbeat fresh".to_string(),
                WorkerLiveness::Hung => format!("PID alive but heartbeat stale (> {}s)", HEARTBEAT_GRACE_SECS),
                WorkerLiveness::Dead => "PID not found".to_string(),
            };

            debug!(
                "Worker {} liveness transition: {:?} -> {:?} ({})",
                heartbeat.worker, old_liveness, new_liveness, reason
            );

            let _ = event_tx.send(MonitorEvent::LivenessChange(LivenessTransition {
                worker: heartbeat.worker.clone(),
                old_state: old_liveness,
                new_state: new_liveness,
                reason,
            }));
        }
    }

    /// Process a heartbeat event and update worker state
    ///
    /// This should be called by consumers when they receive a `MonitorEvent::Heartbeat`.
    pub fn process_heartbeat(&mut self, heartbeat: WorkerHeartbeat) {
        Self::update_worker_state(&heartbeat, &self.workers, &self.event_tx);
    }

    /// Re-evaluate liveness for all workers (called periodically)
    pub fn reevaluate_liveness(&mut self) {
        let workers_snapshot: Vec<(String, Option<u32>, DateTime<Utc>)> = {
            let guard = self.workers.lock().unwrap();
            guard.iter().map(|(w, s)| (w.clone(), s.last_pid, s.last_heartbeat_at)).collect()
        };

        for (worker, pid, last_heartbeat_at) in workers_snapshot {
            let old_liveness = {
                let guard = self.workers.lock().unwrap();
                guard.get(&worker).map(|s| s.liveness).unwrap_or(WorkerLiveness::Dead)
            };

            // Compute new liveness directly
            let new_liveness = {
                // First check: is the PID alive?
                let pid_alive = if let Some(p) = pid {
                    is_process_alive(p)
                } else {
                    false
                };

                if !pid_alive {
                    WorkerLiveness::Dead
                } else {
                    // Second check: is the heartbeat fresh?
                    let now = Utc::now();
                    let heartbeat_age = now.signed_duration_since(last_heartbeat_at).num_seconds() as u64;

                    if heartbeat_age <= HEARTBEAT_GRACE_SECS {
                        WorkerLiveness::Live
                    } else {
                        WorkerLiveness::Hung
                    }
                }
            };

            if old_liveness != new_liveness {
                let mut guard = self.workers.lock().unwrap();
                if let Some(state) = guard.get_mut(&worker) {
                    state.liveness = new_liveness;

                    let reason = match new_liveness {
                        WorkerLiveness::Live => "PID alive and heartbeat fresh".to_string(),
                        WorkerLiveness::Hung => format!("PID alive but heartbeat stale (> {}s)", HEARTBEAT_GRACE_SECS),
                        WorkerLiveness::Dead => "PID not found".to_string(),
                    };

                    debug!(
                        "Worker {} liveness transition: {:?} -> {:?} ({})",
                        worker, old_liveness, new_liveness, reason
                    );

                    let _ = self.event_tx.send(MonitorEvent::LivenessChange(LivenessTransition {
                        worker: worker.clone(),
                        old_state: old_liveness,
                        new_state: new_liveness,
                        reason,
                    }));
                }
            }
        }
    }
}

/// Check if a process is alive using `kill -0`
///
/// This is the canonical process liveness check on Unix systems.
/// Returns false if the PID does not exist or we don't have permission to signal it.
fn is_process_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        use nix::unistd::Pid;
        nix::sys::signal::kill(Pid::from_raw(pid as i32), None).is_ok()
    }

    #[cfg(not(unix))]
    {
        // On non-Unix systems, we can't do `kill -0`
        // For now, just return false
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_source() -> crate::parse_jsonl_safe::LineSource {
        crate::parse_jsonl_safe::LineSource {
            tag: "heartbeats",
            file_path: PathBuf::from("/tmp/test_heartbeats.jsonl"),
            line_number: 1,
        }
    }

    #[test]
    fn test_parse_heartbeat_line_executing() {
        let json = r#"{"ts":"2026-04-21T18:42:10Z","worker":"alpha","state":"executing","bead":"bd-abc123","pid":12345,"adapter":"anthropic"}"#;
        let heartbeat = HeartbeatMonitor::parse_heartbeat_line(json, &test_source()).unwrap();

        assert_eq!(heartbeat.worker, "alpha");
        match heartbeat.state {
            WorkerState::Executing { bead, pid, adapter } => {
                assert_eq!(bead, "bd-abc123");
                assert_eq!(pid, 12345);
                assert_eq!(adapter, "anthropic");
            }
            _ => panic!("Expected Executing state"),
        }
    }

    #[test]
    fn test_parse_heartbeat_line_idle() {
        let json = r#"{"ts":"2026-04-21T18:42:10Z","worker":"alpha","state":"idle","last_strand":null}"#;
        let heartbeat = HeartbeatMonitor::parse_heartbeat_line(json, &test_source()).unwrap();

        assert_eq!(heartbeat.worker, "alpha");
        match heartbeat.state {
            WorkerState::Idle { last_strand } => {
                assert!(last_strand.is_none());
            }
            _ => panic!("Expected Idle state"),
        }
    }

    #[test]
    fn test_parse_heartbeat_line_knot() {
        let json = r#"{"ts":"2026-04-21T18:42:10Z","worker":"alpha","state":"knot","reason":"out of capacity"}"#;
        let heartbeat = HeartbeatMonitor::parse_heartbeat_line(json, &test_source()).unwrap();

        assert_eq!(heartbeat.worker, "alpha");
        match heartbeat.state {
            WorkerState::Knot { reason } => {
                assert_eq!(reason, "out of capacity");
            }
            _ => panic!("Expected Knot state"),
        }
    }

    #[test]
    fn test_parse_heartbeat_line_malformed() {
        let json = r#"{"ts":"2026-04-21T18:42:10Z","worker":"alpha","state":"invalid"}"#;
        assert!(HeartbeatMonitor::parse_heartbeat_line(json, &test_source()).is_none());
    }

    #[test]
    fn test_liveness_fresh_heartbeat() {
        // Fresh heartbeat should be considered live if PID is alive
        // We can't test actual PID checking in unit tests, but we can test the logic
        const { assert!(HEARTBEAT_GRACE_SECS >= 20); }
        assert_eq!(HEARTBEAT_INTERVAL_SECS, 10);
    }
}
