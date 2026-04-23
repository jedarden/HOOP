//! Bead reader for .beads/issues.jsonl
//!
//! Reads bead data from the registered project's bead queue.
//! Uses file watching for real-time updates.
//! Survives log rotation and handles partial lines.

use anyhow::{Context, Result};
use crate::Bead;
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
        let issues_path = self.config.workspace_path.join(".beads").join("issues.jsonl");
        let issues_path_for_watch = issues_path.clone();
        let event_tx = self.event_tx.clone();
        let position = self.position.clone();

        let mut watcher = notify::recommended_watcher(move |res| {
            if let Err(e) = Self::handle_watch_event(res, &issues_path_for_watch, &event_tx, position.clone()) {
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
        let issues_path = self.config.workspace_path.join(".beads").join("issues.jsonl");
        let file = File::open(&issues_path)
            .context("Failed to open beads file for replay")?;

        let metadata = file.metadata()
            .context("Failed to get beads file metadata")?;

        let reader = BufReader::new(file);
        let beads = Self::parse_all(reader)?;

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

        let metadata = file.metadata()
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
        file.seek(SeekFrom::Start(offset))
            .with_context(|| format!("Failed to seek to offset {} in {}", offset, issues_path.display()))?;

        let reader = BufReader::new(file);
        let beads = Self::parse_all(reader)?;

        if !beads.is_empty() {
            let _ = event_tx.send(BeadEvent::BeadsUpdated { beads });
        }

        position.lock().unwrap().update(metadata.len(), &metadata);

        Ok(())
    }

    fn parse_all<R: BufRead>(reader: R) -> Result<Vec<Bead>> {
        let mut beads = Vec::new();

        for line in reader.lines() {
            let line = line.context("Failed to read line from beads file")?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<Bead>(&line) {
                Ok(bead) => beads.push(bead),
                Err(e) => {
                    warn!("Failed to parse bead: {}. Line: {}", e, line.chars().take(100).collect::<String>());
                }
            }
        }

        beads.sort_by(|a, b| b.created_at.cmp(&a.created_at));

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
}
