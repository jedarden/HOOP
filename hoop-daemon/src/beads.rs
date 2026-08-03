//! Bead reader for .beads/issues.jsonl
//!
//! Reads bead data from the registered project's bead queue.
//! Uses file watching for real-time updates.
//! Survives log rotation and handles partial lines.

use crate::Bead;
use anyhow::{Context, Result};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::fs::{File, Metadata};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Events emitted by the bead reader
#[derive(Debug, Clone)]
pub enum BeadEvent {
    /// Beads were updated
    BeadsUpdated { beads: Vec<Bead> },
    /// An error occurred
    Error(String),
}

/// Bead reader configuration
#[derive(Debug, Clone)]
pub struct BeadReaderConfig {
    /// Path to the workspace with .beads/ directory
    pub workspace_path: PathBuf,
}

impl Default for BeadReaderConfig {
    fn default() -> Self {
        Self {
            workspace_path: PathBuf::from("."),
        }
    }
}

/// File position tracking for efficient incremental reads
#[derive(Debug)]
struct FilePosition {
    offset: u64,
    last_size: u64,
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

    fn reset(&mut self) {
        self.offset = 0;
        self.last_size = 0;
        self.last_modified = None;
    }

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

/// Bead reader for issues.jsonl
pub struct BeadReader {
    config: BeadReaderConfig,
    event_tx: broadcast::Sender<BeadEvent>,
    watcher: Option<RecommendedWatcher>,
    _shutdown_tx: mpsc::Sender<()>,
    position: Arc<Mutex<FilePosition>>,
}

impl BeadReader {
    pub fn new(config: BeadReaderConfig) -> Result<Self> {
        let (event_tx, _) = broadcast::channel(256);
        let (shutdown_tx, _) = mpsc::channel(1);

        Ok(Self {
            config,
            event_tx,
            watcher: None,
            _shutdown_tx: shutdown_tx,
            position: Arc::new(Mutex::new(FilePosition::new())),
        })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<BeadEvent> {
        self.event_tx.subscribe()
    }

    pub fn start(&mut self) -> Result<()> {
        let issues_path = self
            .config
            .workspace_path
            .join(".beads")
            .join("issues.jsonl");
        let issues_path_for_watch = issues_path.clone();
        let event_tx = self.event_tx.clone();
        let position = self.position.clone();

        let mut watcher = notify::recommended_watcher(move |res| {
            if let Err(e) =
                Self::handle_watch_event(res, &issues_path_for_watch, &event_tx, position.clone())
            {
                warn!("Error handling bead watch event: {}", e);
            }
        })
        .context("Failed to create file watcher")?;

        let watch_path = if let Some(parent) = issues_path.parent() {
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
            .context("Failed to watch beads directory")?;

        self.watcher = Some(watcher);

        if issues_path.exists() {
            info!("Replaying beads from {}", issues_path.display());
            if let Err(e) = self.replay_file() {
                warn!("Error replaying beads file: {}", e);
            }
        }

        info!("Bead reader watching {}", issues_path.display());

        Ok(())
    }

    pub fn replay_file(&self) -> Result<()> {
        let issues_path = self
            .config
            .workspace_path
            .join(".beads")
            .join("issues.jsonl");
        let file = File::open(&issues_path).context("Failed to open beads file for replay")?;

        let metadata = file
            .metadata()
            .context("Failed to get beads file metadata")?;

        let reader = BufReader::new(file);
        let beads = Self::parse_all(reader, &issues_path)?;

        let _ = self.event_tx.send(BeadEvent::BeadsUpdated { beads });

        let mut pos = self.position.lock().unwrap();
        pos.update(metadata.len(), &metadata);

        Ok(())
    }

    fn handle_watch_event(
        res: Result<notify::Event, notify::Error>,
        issues_path: &Path,
        event_tx: &broadcast::Sender<BeadEvent>,
        position: Arc<Mutex<FilePosition>>,
    ) -> Result<()> {
        let event = res?;

        let relevant = event.paths.iter().any(|p| p == issues_path);

        if !relevant {
            return Ok(());
        }

        use notify::EventKind::*;

        match event.kind {
            Access(_) | Create(_) | Modify(_) => {
                if let Err(e) = Self::read_updates(issues_path, event_tx, position.clone()) {
                    warn!("Error reading bead updates: {}", e);
                }
            }
            Remove(_) => {
                debug!("Beads file removed (likely log rotation)");
                position.lock().unwrap().reset();
            }
            _ => {}
        }

        Ok(())
    }

    fn read_updates(
        issues_path: &Path,
        event_tx: &broadcast::Sender<BeadEvent>,
        position: Arc<Mutex<FilePosition>>,
    ) -> Result<()> {
        let file = File::open(issues_path)
            .with_context(|| format!("Failed to open beads file {}", issues_path.display()))?;

        let metadata = file
            .metadata()
            .with_context(|| format!("Failed to get metadata for {}", issues_path.display()))?;

        {
            let pos = position.lock().unwrap();
            if pos.is_rotated(&metadata) {
                debug!("Bead file rotation detected, resetting position");
                drop(pos);
                position.lock().unwrap().reset();
            }
        }

        let (offset, needs_reset) = {
            let pos = position.lock().unwrap();
            (pos.offset, pos.offset == 0)
        };

        if metadata.len() <= offset && !needs_reset {
            return Ok(());
        }

        let mut file = file;
        file.seek(SeekFrom::Start(offset)).with_context(|| {
            format!(
                "Failed to seek to offset {} in {}",
                offset,
                issues_path.display()
            )
        })?;

        let reader = BufReader::new(file);
        let beads = Self::parse_all(reader, issues_path)?;

        if !beads.is_empty() {
            let _ = event_tx.send(BeadEvent::BeadsUpdated { beads });
        }

        position.lock().unwrap().update(metadata.len(), &metadata);

        Ok(())
    }

    fn parse_all<R: BufRead>(reader: R, file_path: &Path) -> Result<Vec<Bead>> {
        let mut beads = Vec::new();

        for (idx, line) in reader.lines().enumerate() {
            let line = line.context("Failed to read line from beads file")?;
            let source = crate::parse_jsonl_safe::LineSource {
                tag: "beads",
                file_path: file_path.to_path_buf(),
                line_number: idx + 1,
            };

            match crate::parse_jsonl_safe::parse_line::<Bead>(&line, &source) {
                crate::parse_jsonl_safe::ParseResult::Ok(bead) => beads.push(bead),
                crate::parse_jsonl_safe::ParseResult::Empty => {}
                crate::parse_jsonl_safe::ParseResult::Quarantined => {
                    warn!(
                        "Quarantined malformed bead line {} in {}",
                        idx + 1,
                        file_path.display()
                    );
                }
            }
        }

        beads.sort_by_key(|b| std::cmp::Reverse(b.created_at));

        Ok(beads)
    }

    /// Stop the bead reader gracefully
    ///
    /// Flushes any pending reads and stops the file watcher.
    /// This should be called during shutdown to ensure clean state.
    pub async fn stop(&mut self) -> Result<()> {
        debug!("Stopping bead reader");

        // Drop the watcher to stop file watching
        drop(self.watcher.take());

        // Give the file watcher a moment to clean up
        tokio::time::sleep(Duration::from_millis(50)).await;

        debug!("Bead reader stopped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BeadStatus, BeadType};

    #[test]
    fn test_parse_bead() {
        let json = r#"{
            "id": "hoop-ttb.1",
            "title": "Test bead",
            "description": "Test description",
            "status": "open",
            "priority": 0,
            "issue_type": "task",
            "created_at": "2026-04-22T19:48:33Z",
            "created_by": "coding",
            "updated_at": "2026-04-22T19:48:33Z",
            "source_repo": ".",
            "dependencies": []
        }"#;

        let bead: Bead = serde_json::from_str(json).unwrap();
        assert_eq!(bead.id, "hoop-ttb.1");
        assert_eq!(bead.title, "Test bead");
        assert!(matches!(bead.status, BeadStatus::Open));
        assert!(matches!(bead.issue_type, BeadType::Task));
    }

    #[test]
    fn test_real_br_line_1_closed_task() {
        // Real br line from .beads/issues.jsonl line 1
        let json = r#"{"id":"bf-10k1y","title":"§1.8 S4: Daemon restart acceptance test (continuity)","description":"Integration test for acceptance scenario S4: kill hoop-daemon, verify in-flight br subprocess completes or is cleanly abandoned, restart daemon, verify fleet.db stitches are intact and project supervisors re-attach. Acceptance: no stitch data loss, restart <5s.","design":"","acceptance_criteria":"","notes":"","status":"closed","priority":1,"issue_type":"task","created_at":"2026-05-01T22:17:41.906031666Z","updated_at":"2026-05-13T23:39:12.881456262Z","closed_at":"2026-05-13T23:39:12.881456262Z","close_reason":"Completed","source_repo":".","compaction_level":0}"#;

        let bead: Bead = serde_json::from_str(json).unwrap();
        assert_eq!(bead.id, "bf-10k1y");
        assert_eq!(bead.status, BeadStatus::Closed);
        assert_eq!(bead.issue_type, BeadType::Task);
        assert_eq!(bead.created_by, ""); // missing field defaults to empty
        assert!(bead.dependencies.is_empty()); // missing field defaults to empty
    }

    #[test]
    fn test_real_br_line_2_open_task() {
        // Real br line from .beads/issues.jsonl - open task with dependencies, missing created_by
        let json = r#"{"id":"bf-114ko","title":"Apply pin! macro fixes","description":"Test description","status":"open","priority":2,"issue_type":"task","created_at":"2026-07-10T18:42:14.846403250Z","updated_at":"2026-07-10T18:42:14.846403250Z","source_repo":".","compaction_level":0,"dependencies":[{"issue_id":"bf-114ko","depends_on_id":"bf-422y8","type":"blocks","created_at":"2026-07-10T18:42:27.720711647Z","created_by":"cli","thread_id":""}]}"#;

        let bead: Bead = serde_json::from_str(json).unwrap();
        assert_eq!(bead.id, "bf-114ko");
        assert_eq!(bead.status, BeadStatus::Open);
        assert_eq!(bead.issue_type, BeadType::Task);
    }

    #[test]
    fn test_real_br_line_3_blocked_task() {
        // Real br line from .beads/issues.jsonl - blocked status
        let json = r#"{"id":"bf-12627","title":"Check ToSchema imports","description":"Check imports","status":"blocked","priority":2,"issue_type":"task","created_at":"2026-07-03T06:19:23.611881906Z","updated_at":"2026-08-01T00:20:40.002621310Z","source_repo":".","compaction_level":0}"#;

        let bead: Bead = serde_json::from_str(json).unwrap();
        assert_eq!(bead.id, "bf-12627");
        assert_eq!(bead.status, BeadStatus::Blocked);
        assert_eq!(bead.issue_type, BeadType::Task);
    }

    #[test]
    fn test_real_br_line_bug_type() {
        // Real br line - bug type
        let json = r#"{"id":"bf-12m0i","title":"nix-shell not available on HOOP build server","description":"The nix-shell command required for HOOP Rust builds is not available on the build server. Tried nix-shell --run rustc --version but got command not found. System is NOT NixOS per os-release. However rustc 1.95.0 IS available directly. This blocks verification of bf-3lu60 which requires nix-shell access.","design":"","acceptance_criteria":"","notes":"Investigation complete - nix-shell not required on Debian build server. All dependencies available natively. See notes/bf-12m0i.md for details.","status":"closed","priority":2,"issue_type":"bug","assignee":"claude-code-glm47-alpha","created_at":"2026-07-03T03:34:03.778372007Z","updated_at":"2026-07-09T13:47:45.975209252Z","closed_at":"2026-07-09T13:47:45.975209252Z","close_reason":"Data-hygiene remediation (bf-wre): status was completed, a non-terminal Custom status invisible to is_terminal()/get_ready_candidates, silently freezing dependents. Underlying work was already done per bead history; correcting to closed.","source_repo":".","compaction_level":0}"#;

        let bead: Bead = serde_json::from_str(json).unwrap();
        assert_eq!(bead.id, "bf-12m0i");
        assert_eq!(bead.status, BeadStatus::Closed);
        assert_eq!(bead.issue_type, BeadType::Bug);
    }

    #[test]
    fn test_all_bead_status_variants() {
        // Test all BeadStatus lowercase variants that br/bead-forge writes
        let statuses = [
            ("open", BeadStatus::Open),
            ("closed", BeadStatus::Closed),
            ("blocked", BeadStatus::Blocked),
            ("completed", BeadStatus::Completed),
            ("done", BeadStatus::Done),
        ];

        for (status_str, expected_status) in statuses {
            let json = r#"{
                "id": "test-1",
                "title": "Test",
                "status": ""#.to_string() + status_str + r#"",
                "priority": 1,
                "issue_type": "task",
                "created_at": "2026-04-22T19:48:33Z",
                "updated_at": "2026-04-22T19:48:33Z"
            }"#;

            let bead: Bead = serde_json::from_str(&json).unwrap();
            assert_eq!(bead.status, expected_status, "Failed to deserialize status: {}", status_str);
        }
    }

    #[test]
    fn test_all_bead_type_variants() {
        // Test all BeadType lowercase variants that br/bead-forge writes
        let types = [
            ("task", BeadType::Task),
            ("bug", BeadType::Bug),
            ("chore", BeadType::Chore),
            ("feature", BeadType::Feature),
            ("test", BeadType::Test),
            ("docs", BeadType::Docs),
            ("story", BeadType::Story),
            ("epic", BeadType::Epic),
            ("genesis", BeadType::Genesis),
            ("review", BeadType::Review),
            ("fix", BeadType::Fix),
        ];

        for (type_str, expected_type) in types {
            let json = r#"{
                "id": "test-1",
                "title": "Test",
                "status": "open",
                "priority": 1,
                "issue_type": ""#.to_string() + type_str + r#"",
                "created_at": "2026-04-22T19:48:33Z",
                "updated_at": "2026-04-22T19:48:33Z"
            }"#;

            let bead: Bead = serde_json::from_str(&json).unwrap();
            assert_eq!(bead.issue_type, expected_type, "Failed to deserialize issue_type: {}", type_str);
        }
    }

    #[test]
    fn test_missing_created_by_defaults_to_empty() {
        // br/bead-forge may omit created_by on older beads
        let json = r#"{
            "id": "test-1",
            "title": "Test",
            "status": "open",
            "priority": 1,
            "issue_type": "task",
            "created_at": "2026-04-22T19:48:33Z",
            "updated_at": "2026-04-22T19:48:33Z"
        }"#;

        let bead: Bead = serde_json::from_str(json).unwrap();
        assert_eq!(bead.created_by, "");
    }

    #[test]
    fn test_missing_dependencies_defaults_to_empty() {
        // br/bead-forge may omit dependencies when a bead has no blockers
        let json = r#"{
            "id": "test-1",
            "title": "Test",
            "status": "open",
            "priority": 1,
            "issue_type": "task",
            "created_at": "2026-04-22T19:48:33Z",
            "updated_at": "2026-04-22T19:48:33Z"
        }"#;

        let bead: Bead = serde_json::from_str(json).unwrap();
        assert!(bead.dependencies.is_empty());
    }

    #[test]
    fn test_unknown_status_deserializes_to_unknown() {
        // Unknown status should deserialize to Unknown variant, not fail
        let json = r#"{
            "id": "test-1",
            "title": "Test",
            "status": "custom_status",
            "priority": 1,
            "issue_type": "task",
            "created_at": "2026-04-22T19:48:33Z",
            "updated_at": "2026-04-22T19:48:33Z"
        }"#;

        let bead: Bead = serde_json::from_str(json).unwrap();
        assert_eq!(bead.status, BeadStatus::Unknown);
    }

    #[test]
    fn test_unknown_type_deserializes_to_unknown() {
        // Unknown issue_type should deserialize to Unknown variant, not fail
        let json = r#"{
            "id": "test-1",
            "title": "Test",
            "status": "open",
            "priority": 1,
            "issue_type": "custom_type",
            "created_at": "2026-04-22T19:48:33Z",
            "updated_at": "2026-04-22T19:48:33Z"
        }"#;

        let bead: Bead = serde_json::from_str(json).unwrap();
        assert_eq!(bead.issue_type, BeadType::Unknown);
    }

    #[test]
    fn test_extra_unknown_keys_ignored() {
        // Extra keys should be silently ignored (serde default behavior)
        let json = r#"{
            "id": "test-1",
            "title": "Test",
            "status": "open",
            "priority": 1,
            "issue_type": "task",
            "created_at": "2026-04-22T19:48:33Z",
            "updated_at": "2026-04-22T19:48:33Z",
            "unknown_key": "some_value",
            "another_unknown": 123
        }"#;

        let bead: Bead = serde_json::from_str(json).unwrap();
        assert_eq!(bead.id, "test-1");
        assert_eq!(bead.status, BeadStatus::Open);
    }
}
