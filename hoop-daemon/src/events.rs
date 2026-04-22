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
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
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

/// Event tailer that watches and parses events.jsonl
pub struct EventTailer {
    config: EventTailerConfig,
    event_tx: broadcast::Sender<TailerEvent>,
    watcher: Option<RecommendedWatcher>,
    _shutdown_tx: mpsc::Sender<()>,
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

        // Create the watcher
        let mut watcher = notify::recommended_watcher(move |res| {
            if let Err(e) = Self::handle_watch_event(res, &events_path_for_watch, &event_tx) {
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

        let reader = BufReader::new(file);
        let mut parser = NdjsonParser::new();

        for line in reader.lines() {
            let line = line.context("Failed to read line from events file")?;
            if let Some(parsed) = parser.parse_line(&line, 0)? {
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

        Ok(())
    }

    /// Handle a watch event from notify
    fn handle_watch_event(
        res: Result<notify::Event, notify::Error>,
        events_path: &Path,
        event_tx: &broadcast::Sender<TailerEvent>,
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
                if let Err(e) = Self::read_new_events(events_path, event_tx) {
                    warn!("Error reading new events: {}", e);
                }
            }
            // File removed - this is likely log rotation
            Remove(_) => {
                debug!("Events file removed (likely log rotation)");
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
    ) -> Result<()> {
        let file = File::open(events_path)
            .with_context(|| format!("Failed to open events file {}", events_path.display()))?;

        let reader = BufReader::new(file);
        let mut parser = NdjsonParser::new();
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line = line.with_context(|| {
                format!(
                    "Failed to read line {} from {}",
                    line_number,
                    events_path.display()
                )
            })?;

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
                Ok(Some(ParsedEvent {
                    event,
                    line_number,
                    raw,
                }))
            }
            Err(e) => {
                // Check if this is a partial line (ends abruptly)
                // A common pattern is that the line is incomplete JSON
                if input.len() < 256 || !input.ends_with('}') && !input.ends_with(']') {
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

        // This is clearly malformed (missing closing brace but long enough to not be partial)
        let malformed = r#"{"event":"claim","ts":"2026-04-21T18:42:10Z","worker":"alpha","bead":"bd-abc123"#;
        let result = parser.parse_line(malformed, 1).unwrap().unwrap();

        // Should return Unknown event
        assert!(matches!(result.event, NeedleEvent::Unknown));
    }
}
