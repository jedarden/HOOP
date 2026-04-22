//! Event tailer for NEEDLE events.jsonl
//!
//! Watches `.beads/events.jsonl` for a registered workspace using the `notify` crate.
//! Survives log rotation (handles file-moved events).
//! Uses line-buffered NDJSON with partial-line carry-over.
//! Malformed lines are logged with `warn`, never silent-dropped.
//! Unknown event types emit a progress event + increment a metric.

use anyhow::{Context, Result};
use hoop_schema::{NeedleEvent, ParsedEvent};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::fs::{File, Metadata};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use crate::metrics;

/// Events emitted by the event tailer
#[derive(Debug, Clone)]
pub enum TailerEvent {
    /// A new event was parsed from the log
    Event(ParsedEvent),
    /// The file was rotated (moved/recreated)
    Rotated,
    /// An error occurred
    Error(String),
}

/// Event tailer configuration
#[derive(Debug, Clone)]
pub struct EventTailerConfig {
    /// Path to the events.jsonl file
    pub events_path: PathBuf,
    /// Whether to replay the entire file on startup
    pub replay_on_startup: bool,
}

impl Default for EventTailerConfig {
    fn default() -> Self {
        let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.push(".hoop");
        home.push("events.jsonl");
        Self {
            events_path: home,
            replay_on_startup: true,
        }
    }
}

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
        // File is considered rotated if:
        // 1. The new size is smaller than our offset (file was truncated/recreated)
        // 2. The modification time is older than our last read (unlikely, but indicates rotation)
        if let Some(last_mod) = self.last_modified {
            if let Ok(new_mod) = metadata.modified() {
                // If size decreased or modification time went backwards, likely rotated
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

/// Event tailer that watches and parses events.jsonl
pub struct EventTailer {
    config: EventTailerConfig,
    event_tx: broadcast::Sender<TailerEvent>,
    watcher: Option<RecommendedWatcher>,
    _shutdown_tx: mpsc::Sender<()>,
    /// File position tracking for efficient incremental reads
    position: Arc<Mutex<FilePosition>>,
}

impl EventTailer {
    /// Create a new event tailer
    pub fn new(config: EventTailerConfig) -> Result<Self> {
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

    /// Subscribe to events from this tailer
    pub fn subscribe(&self) -> broadcast::Receiver<TailerEvent> {
        self.event_tx.subscribe()
    }

    /// Start watching the events file
    pub fn start(&mut self) -> Result<()> {
        let events_path = self.config.events_path.clone();
        let events_path_for_watch = events_path.clone();
        let event_tx = self.event_tx.clone();
        let position = self.position.clone();

        // Create the watcher
        let mut watcher = notify::recommended_watcher(move |res| {
            if let Err(e) = Self::handle_watch_event(res, &events_path_for_watch, &event_tx, position.clone()) {
                warn!("Error handling watch event: {}", e);
            }
        })
        .context("Failed to create file watcher")?;

        // Watch the parent directory (since events.jsonl may not exist yet)
        let watch_path = if let Some(parent) = events_path.parent() {
            if parent.exists() {
                parent.to_path_buf()
            } else {
                // If parent doesn't exist, watch the current directory
                PathBuf::from(".")
            }
        } else {
            PathBuf::from(".")
        };

        watcher
            .watch(&watch_path, RecursiveMode::NonRecursive)
            .context("Failed to watch events directory")?;

        self.watcher = Some(watcher);

        // Replay existing events on startup
        if self.config.replay_on_startup && events_path.exists() {
            info!("Replaying events from {}", events_path.display());
            if let Err(e) = self.replay_file() {
                warn!("Error replaying events file: {}", e);
            }
        }

        info!(
            "Event tailer watching {}",
            self.config.events_path.display()
        );

        Ok(())
    }

    /// Replay all events from the existing file
    fn replay_file(&self) -> Result<()> {
        let events_path = &self.config.events_path;
        let file = File::open(events_path)
            .context("Failed to open events file for replay")?;

        let metadata = file.metadata()
            .context("Failed to get events file metadata")?;

        let reader = BufReader::new(file);
        let mut parser = NdjsonParser::new();
        let mut line_number = 0;
        let mut offset = 0u64;

        for line in reader.lines() {
            line_number += 1;
            let line = line.context("Failed to read line from events file")?;
            // Update offset (line bytes + newline)
            offset += line.len() as u64 + 1;

            if let Some(parsed) = parser.parse_line(&line, line_number)? {
                let _ = self.event_tx.send(TailerEvent::Event(parsed));
            }
        }

        // Handle any remaining partial line
        if let Some(partial) = parser.finish() {
            if !partial.is_empty() {
                warn!(
                    "Incomplete final line in events file: {}",
                    partial.trim()
                );
            }
        }

        // Update position tracking after replay
        let mut pos = self.position.lock().unwrap();
        pos.update(offset, &metadata);

        Ok(())
    }

    /// Handle a watch event from notify
    fn handle_watch_event(
        res: Result<notify::Event, notify::Error>,
        events_path: &Path,
        event_tx: &broadcast::Sender<TailerEvent>,
        position: Arc<Mutex<FilePosition>>,
    ) -> Result<()> {
        let event = res?;

        // Check if the event is for our file
        let relevant = event.paths.iter().any(|p| p == events_path);

        if !relevant {
            return Ok(());
        }

        // Handle different event kinds
        use notify::EventKind::*;

        match event.kind {
            // File created or modified - read new lines
            Access(_) | Create(_) | Modify(_) => {
                if let Err(e) = Self::read_new_events(events_path, event_tx, position.clone()) {
                    warn!("Error reading new events: {}", e);
                }
            }
            // File removed - this is likely log rotation
            Remove(_) => {
                debug!("Events file removed (likely log rotation)");
                // Reset position tracking for when the file is recreated
                position.lock().unwrap().reset();
                let _ = event_tx.send(TailerEvent::Rotated);
            }
            _ => {}
        }

        Ok(())
    }

    /// Read new events from the file
    fn read_new_events(
        events_path: &Path,
        event_tx: &broadcast::Sender<TailerEvent>,
        position: Arc<Mutex<FilePosition>>,
    ) -> Result<()> {
        let file = File::open(events_path)
            .with_context(|| format!("Failed to open events file {}", events_path.display()))?;

        let metadata = file.metadata()
            .with_context(|| format!("Failed to get metadata for {}", events_path.display()))?;

        // Check for log rotation
        {
            let pos = position.lock().unwrap();
            if pos.is_rotated(&metadata) {
                debug!("Log rotation detected, resetting position");
                drop(pos);
                position.lock().unwrap().reset();
                let _ = event_tx.send(TailerEvent::Rotated);
            }
        }

        // Get current position
        let (offset, needs_reset) = {
            let pos = position.lock().unwrap();
            (pos.offset, pos.offset == 0)
        };

        // If file hasn't grown since last read, nothing to do
        if metadata.len() <= offset && !needs_reset {
            return Ok(());
        }

        // Seek to our last position
        let mut file = file;
        file.seek(SeekFrom::Start(offset))
            .with_context(|| format!("Failed to seek to offset {} in {}", offset, events_path.display()))?;

        let reader = BufReader::new(file);
        let mut parser = NdjsonParser::new();
        let mut line_number = 0;
        let mut current_offset = offset;

        for line in reader.lines() {
            line_number += 1;
            let line = line.with_context(|| {
                format!(
                    "Failed to read line {} from {}",
                    line_number,
                    events_path.display()
                )
            })?;

            // Update offset (line bytes + newline)
            current_offset += line.len() as u64 + 1;

            if let Some(parsed) = parser.parse_line(&line, line_number)? {
                let _ = event_tx.send(TailerEvent::Event(parsed));
            }
        }

        // Handle any remaining partial line
        if let Some(partial) = parser.finish() {
            if !partial.is_empty() {
                warn!(
                    "Incomplete final line in events file: {}",
                    partial.trim()
                );
            }
        }

        // Update position tracking
        position.lock().unwrap().update(current_offset, &metadata);

        Ok(())
    }
}

/// Line-buffered NDJSON parser with partial-line carry-over
///
/// This implements the critical requirement from §F1 of the plan:
/// "Line-buffered NDJSON reader that carries partial lines across chunks."
struct NdjsonParser {
    /// Carry-over buffer for partial lines
    partial: String,
}

impl NdjsonParser {
    /// Create a new parser
    fn new() -> Self {
        Self {
            partial: String::new(),
        }
    }

    /// Parse a line, carrying over partial lines
    ///
    /// Returns None if the line was incomplete (carried over).
    /// Returns Some(parsed) if a complete event was parsed.
    fn parse_line(&mut self, line: &str, line_number: usize) -> Result<Option<ParsedEvent>> {
        let mut input = line;

        // If we have a partial line from before, prepend it
        if !self.partial.is_empty() {
            self.partial.push_str(line);
            input = self.partial.as_str();
        }

        // Try to parse as JSON
        match serde_json::from_str::<NeedleEvent>(input) {
            Ok(event) => {
                // Successfully parsed - clear the partial buffer
                let raw = input.to_string();
                self.partial.clear();

                // Increment unknown event metric if this is an unknown event type
                if matches!(event, NeedleEvent::Unknown) {
                    metrics::metrics().hoop_unknown_event_total.inc();
                }

                Ok(Some(ParsedEvent {
                    event,
                    line_number,
                    raw,
                }))
            }
            Err(e) => {
                // Check if this is a partial line (ends abruptly)
                // A common pattern is that the line is incomplete JSON
                // Treat as partial if doesn't end with closing bracket AND is short
                if !input.ends_with('}') && !input.ends_with(']') && input.len() < 256 {
                    // Treat as partial line - carry over for next read
                    if self.partial.is_empty() {
                        self.partial = input.to_string();
                    }
                    Ok(None)
                } else {
                    // This is likely a malformed line - log and skip
                    let raw = input.to_string();
                    warn!(
                        "Malformed event on line {}: {}. Line content: {}",
                        line_number,
                        e,
                        raw.chars().take(100).collect::<String>()
                    );

                    // Clear the partial buffer and continue
                    self.partial.clear();

                    // Create an unknown event to preserve the raw data
                    // Increment the unknown event metric
                    metrics::metrics().hoop_unknown_event_total.inc();

                    Ok(Some(ParsedEvent {
                        event: NeedleEvent::Unknown,
                        line_number,
                        raw,
                    }))
                }
            }
        }
    }

    /// Finish parsing and return any remaining partial line
    fn finish(&mut self) -> Option<String> {
        if self.partial.is_empty() {
            None
        } else {
            let partial = self.partial.clone();
            self.partial.clear();
            Some(partial)
        }
    }
}

impl Default for NdjsonParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ndjson_parser_complete_line() {
        let mut parser = NdjsonParser::new();

        let json = r#"{"event":"claim","ts":"2026-04-21T18:42:10Z","worker":"alpha","bead":"bd-abc123"}"#;
        let result = parser.parse_line(json, 1).unwrap().unwrap();

        match result.event {
            NeedleEvent::Claim { worker, bead, .. } => {
                assert_eq!(worker, "alpha");
                assert_eq!(bead, "bd-abc123");
            }
            _ => panic!("Expected Claim event"),
        }
    }

    #[test]
    fn test_ndjson_parser_partial_line() {
        let mut parser = NdjsonParser::new();

        // First part is incomplete
        let partial = r#"{"event":"claim","ts":"2026-04-21T18:42:10Z","worker":"alpha""#;
        assert!(parser.parse_line(partial, 1).unwrap().is_none());

        // Second part completes it
        let completion = r#","bead":"bd-abc123"}"#;
        let result = parser.parse_line(completion, 2).unwrap().unwrap();

        match result.event {
            NeedleEvent::Claim { worker, bead, .. } => {
                assert_eq!(worker, "alpha");
                assert_eq!(bead, "bd-abc123");
            }
            _ => panic!("Expected Claim event"),
        }

        // Line number should be from the completion
        assert_eq!(result.line_number, 2);
    }

    #[test]
    fn test_ndjson_parser_malformed_line() {
        let mut parser = NdjsonParser::new();

        // This is clearly malformed (missing closing brace and long enough to not be partial)
        // Using a long string to exceed the 256-char threshold for partial line detection
        let malformed = r#"{"event":"claim","ts":"2026-04-21T18:42:10Z","worker":"alpha","bead":"bd-abc123","very_long_field_that_makes_this_line_exceed_256_characters_threshold_so_it_will_be_treated_as_malformed_instead_of_partial":"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident."#;
        let result = parser.parse_line(malformed, 1).unwrap().unwrap();

        // Should return Unknown event
        assert!(matches!(result.event, NeedleEvent::Unknown));
    }
}
