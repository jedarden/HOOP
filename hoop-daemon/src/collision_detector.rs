//! Collision detector: alerts when two active workers edit overlapping files
//!
//! §6 Phase 2, deliverable 12. Observation-only — no worker steering (§8).

use crate::ws::{CollisionAlertData, SessionMessageData, WorkerDisplayState, WorkerRegistry};
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::info;

/// Tool names that signal a file write or edit (Claude Code / NEEDLE adapter names)
const FILE_WRITE_TOOLS: &[&str] = &[
    "write_file",
    "Write",
    "edit",
    "Edit",
    "create_file",
    "str_replace_editor",
    "replace_in_file",
    "MultiEdit",
    "str_replace_based_edit_tool",
    "create_or_overwrite_file",
    "overwrite_file",
];

/// Extract touched file paths from a session's messages.
///
/// Scans assistant-role messages for `tool_use` content blocks whose tool
/// name is in [`FILE_WRITE_TOOLS`], then pulls the `path` / `file_path` /
/// `target_file` field from the tool input.  Relative paths are resolved
/// against `cwd`.
fn extract_touched_files(messages: &[SessionMessageData], cwd: &str) -> Vec<String> {
    let mut paths = Vec::new();
    for msg in messages {
        if msg.role != "assistant" {
            continue;
        }
        let blocks = match &msg.content {
            serde_json::Value::Array(arr) => arr.as_slice(),
            _ => continue,
        };
        for block in blocks {
            if block.get("type").and_then(|v| v.as_str()) != Some("tool_use") {
                continue;
            }
            let tool_name = block.get("name").and_then(|v| v.as_str()).unwrap_or("");
            if !FILE_WRITE_TOOLS.contains(&tool_name) {
                continue;
            }
            let input = match block.get("input") {
                Some(v) => v,
                None => continue,
            };
            let path_str = input
                .get("path")
                .or_else(|| input.get("file_path"))
                .or_else(|| input.get("target_file"))
                .and_then(|v| v.as_str());
            if let Some(p) = path_str {
                let abs = if p.starts_with('/') {
                    p.to_string()
                } else {
                    format!("{}/{}", cwd.trim_end_matches('/'), p)
                };
                paths.push(abs);
            }
        }
    }
    paths
}

struct WorkerSnapshot {
    worker: String,
    bead: String,
    paths: HashSet<String>,
}

/// Collision detector
///
/// Polls worker sessions on a fixed interval, compares touched-file sets
/// pairwise across live+executing workers, and broadcasts a
/// [`CollisionAlertData`] event for each new overlap.
pub struct CollisionDetector {
    registry: Arc<WorkerRegistry>,
    alert_tx: broadcast::Sender<CollisionAlertData>,
    /// Stable dedup keys for alerts already emitted this daemon session.
    /// Prevents re-emitting the same alert every 15 s for an ongoing conflict.
    seen: std::sync::Mutex<HashSet<String>>,
}

impl CollisionDetector {
    pub fn new(registry: Arc<WorkerRegistry>, alert_tx: broadcast::Sender<CollisionAlertData>) -> Self {
        Self {
            registry,
            alert_tx,
            seen: std::sync::Mutex::new(HashSet::new()),
        }
    }

    /// Run one detection cycle synchronously (called from the spawned task).
    pub async fn run_once(&self) {
        let conversations = self.registry.conversations_snapshot().await;
        let workers = self.registry.snapshot().await;

        // Restrict to Live + Executing workers (only they can touch files right now)
        let live_executing: HashMap<String, String> = workers
            .iter()
            .filter(|w| matches!(w.liveness, crate::heartbeats::WorkerLiveness::Live))
            .filter_map(|w| {
                if let WorkerDisplayState::Executing { ref bead, .. } = w.state {
                    Some((w.worker.clone(), bead.clone()))
                } else {
                    None
                }
            })
            .collect();

        if live_executing.len() < 2 {
            return; // need ≥2 active workers for any collision
        }

        // Build per-worker touched-file snapshots
        let mut snapshots: Vec<WorkerSnapshot> = Vec::new();
        for conv in &conversations {
            let Some(meta) = &conv.worker_metadata else {
                continue;
            };
            let Some(expected_bead) = live_executing.get(&meta.worker) else {
                continue;
            };
            if *expected_bead != meta.bead {
                continue; // stale conversation — worker moved on to another bead
            }
            let paths: HashSet<String> =
                extract_touched_files(&conv.messages, &conv.cwd).into_iter().collect();
            if !paths.is_empty() {
                snapshots.push(WorkerSnapshot {
                    worker: meta.worker.clone(),
                    bead: meta.bead.clone(),
                    paths,
                });
            }
        }

        if snapshots.len() < 2 {
            return;
        }

        // Pairwise overlap check — O(n²), n ≤ ~20 workers in practice
        let now = Utc::now().to_rfc3339();
        for i in 0..snapshots.len() {
            for j in (i + 1)..snapshots.len() {
                let a = &snapshots[i];
                let b = &snapshots[j];
                let mut overlapping: Vec<String> =
                    a.paths.intersection(&b.paths).cloned().collect();
                if overlapping.is_empty() {
                    continue;
                }
                overlapping.sort();

                // Canonical ordering: alphabetically earlier worker is "a"
                let (w_a, b_a, w_b, b_b) = if a.worker <= b.worker {
                    (a.worker.as_str(), a.bead.as_str(), b.worker.as_str(), b.bead.as_str())
                } else {
                    (b.worker.as_str(), b.bead.as_str(), a.worker.as_str(), a.bead.as_str())
                };

                // Dedup key: (worker_a, worker_b, sorted_files)
                let dedup_key = format!("{}|{}|{}", w_a, w_b, overlapping.join("|"));
                if self.seen.lock().unwrap().contains(&dedup_key) {
                    continue;
                }

                let alert = CollisionAlertData {
                    alert_id: uuid::Uuid::new_v4().to_string(),
                    detected_at: now.clone(),
                    worker_a: w_a.to_string(),
                    bead_a: b_a.to_string(),
                    worker_b: w_b.to_string(),
                    bead_b: b_b.to_string(),
                    overlapping_files: overlapping,
                };

                info!(
                    worker_a = %alert.worker_a,
                    bead_a = %alert.bead_a,
                    worker_b = %alert.worker_b,
                    bead_b = %alert.bead_b,
                    files = alert.overlapping_files.len(),
                    "collision detected"
                );

                let _ = self.alert_tx.send(alert);
                self.seen.lock().unwrap().insert(dedup_key);
            }
        }
    }

    /// Spawn a background task that polls every `interval_secs` seconds.
    pub fn spawn(
        self: Arc<Self>,
        interval_secs: u64,
        mut shutdown: broadcast::Receiver<crate::shutdown::ShutdownPhase>,
    ) {
        tokio::spawn(async move {
            let mut ticker =
                tokio::time::interval(tokio::time::Duration::from_secs(interval_secs));
            ticker.tick().await; // consume the immediate first tick
            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        self.run_once().await;
                    }
                    Ok(phase) = shutdown.recv() => {
                        if matches!(phase, crate::shutdown::ShutdownPhase::Initiated) {
                            break;
                        }
                    }
                }
            }
        });
    }
}
