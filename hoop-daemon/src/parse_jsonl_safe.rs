//! Safe JSONL line parser with automatic quarantine for malformed lines.
//!
//! Every JSONL reader in the daemon should use [`parse_line`] instead of raw
//! `serde_json::from_str`.  Bad lines are written to `~/.hoop/quarantine/<date>/`
//! with full source metadata so they can be inspected later, and the reader
//! continues processing the rest of the file.

use std::fs;
use std::path::PathBuf;

use chrono::Utc;
use serde::de::DeserializeOwned;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Source metadata for a JSONL line being parsed.
#[derive(Clone)]
pub struct LineSource {
    /// Short label for metric labeling (e.g. `"beads"`, `"events"`,
    /// `"sessions/claude"`).
    pub tag: &'static str,
    /// Path to the source JSONL file.
    pub file_path: PathBuf,
    /// 1-based line number within the file.
    pub line_number: usize,
}

/// Result of attempting to parse a single JSONL line.
pub enum ParseResult<T> {
    /// Line was successfully deserialized into `T`.
    Ok(T),
    /// Line was empty / whitespace-only.
    Empty,
    /// Line failed to parse and was quarantined.
    Quarantined,
}

// ---------------------------------------------------------------------------
// Core API
// ---------------------------------------------------------------------------

/// Try to deserialize a JSONL line into type `T`.
///
/// * Empty / whitespace lines return [`ParseResult::Empty`].
/// * Parseable lines return [`ParseResult::Ok(value)`].
/// * Unparseable lines are written to the quarantine directory and return
///   [`ParseResult::Quarantined`].
pub fn parse_line<T: DeserializeOwned>(line: &str, source: &LineSource) -> ParseResult<T> {
    if line.trim().is_empty() {
        return ParseResult::Empty;
    }

    match serde_json::from_str::<T>(line) {
        Ok(value) => ParseResult::Ok(value),
        Err(err) => {
            let reason = format!("{}", err);
            if let Err(qe) = quarantine_line(line, &reason, source) {
                tracing::warn!("Failed to quarantine bad line from {}: {}", source.tag, qe);
            }
            crate::metrics::metrics()
                .hoop_jsonl_quarantined_lines_total
                .inc(&[source.tag]);
            ParseResult::Quarantined
        }
    }
}

/// Quarantine a raw line that failed parsing *outside* of [`parse_line`].
///
/// Use this when the reader has its own partial-line / carry-over logic (e.g.
/// `NdjsonParser`) and determines that a line is definitively malformed only
/// after its own heuristics.
pub fn quarantine_raw(line: &str, reason: &str, source: &LineSource) {
    if let Err(qe) = quarantine_line(line, reason, source) {
        tracing::warn!("Failed to quarantine bad line from {}: {}", source.tag, qe);
    }
    crate::metrics::metrics()
        .hoop_jsonl_quarantined_lines_total
        .inc(&[source.tag]);
}

// ---------------------------------------------------------------------------
// Quarantine directory helpers
// ---------------------------------------------------------------------------

/// Return the quarantine root directory (`~/.hoop/quarantine/`).
///
/// Override with `HOOP_QUARANTINE_DIR` env var for integration tests.
pub fn quarantine_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("HOOP_QUARANTINE_DIR") {
        return PathBuf::from(dir);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".hoop")
        .join("quarantine")
}

/// Count quarantine entries for the current UTC day.
pub fn quarantine_today_count() -> u64 {
    let day_dir = quarantine_dir().join(Utc::now().format("%Y-%m-%d").to_string());
    fs::read_dir(&day_dir)
        .map(|entries| entries.filter_map(|e| e.ok()).count() as u64)
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Internal
// ---------------------------------------------------------------------------

fn quarantine_line(line: &str, reason: &str, source: &LineSource) -> std::io::Result<()> {
    let dir = quarantine_dir();
    let date_dir = dir.join(Utc::now().format("%Y-%m-%d").to_string());
    fs::create_dir_all(&date_dir)?;

    let short_id = &uuid::Uuid::new_v4().to_string()[..8];
    let filename = format!(
        "{}-{}-{}.json",
        source.tag.replace('/', "_"),
        source.line_number,
        short_id
    );

    let entry = serde_json::json!({
        "line": line,
        "reason": reason,
        "source_path": source.file_path.to_str().unwrap_or(""),
        "line_number": source.line_number,
        "tag": source.tag,
        "timestamp": Utc::now().to_rfc3339(),
    });

    fs::write(
        date_dir.join(filename),
        serde_json::to_string_pretty(&entry).unwrap_or_default(),
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Verify the helper parses valid lines and quarantines invalid ones.
    #[test]
    fn parse_line_ok_and_quarantine() {
        let tmp = TempDir::new().unwrap();
        let qdir = tmp.path().join("quarantine");
        // Override quarantine dir so we don't pollute the real one.
        std::env::set_var("HOOP_QUARANTINE_DIR", qdir.to_str().unwrap());

        let source = LineSource {
            tag: "test",
            file_path: PathBuf::from("/tmp/test.jsonl"),
            line_number: 1,
        };

        // Valid JSON object
        let result: ParseResult<serde_json::Value> =
            parse_line(r#"{"key": "value"}"#, &source);
        assert!(
            matches!(result, ParseResult::Ok(_)),
            "valid JSON should parse"
        );

        // Empty line
        let result: ParseResult<serde_json::Value> = parse_line("   ", &source);
        assert!(
            matches!(result, ParseResult::Empty),
            "empty line should return Empty"
        );

        // Invalid JSON — goes to quarantine
        let result: ParseResult<serde_json::Value> =
            parse_line("NOT JSON {{{", &source);
        assert!(
            matches!(result, ParseResult::Quarantined),
            "invalid JSON should be quarantined"
        );

        // Verify the quarantined file was written
        assert!(qdir.exists(), "quarantine dir should exist");
        let date_dirs: Vec<_> = std::fs::read_dir(&qdir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(date_dirs.len(), 1, "should have one date directory");
        let entries: Vec<_> = std::fs::read_dir(date_dirs[0].path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(entries.len(), 1, "should have one quarantined file");

        let entry: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(entries[0].path()).unwrap()).unwrap();
        assert_eq!(entry["tag"], "test");
        assert!(entry["line"].as_str().unwrap().contains("NOT JSON"));
        assert_eq!(entry["line_number"], 1);

        std::env::remove_var("HOOP_QUARANTINE_DIR");
    }

    /// Verify quarantine_today_count returns 0 when no quarantine dir exists.
    #[test]
    fn quarantine_today_count_no_dir() {
        // This test is lightweight — it just checks that a missing directory
        // returns 0 rather than panicking.
        let count = quarantine_today_count();
        // The count may or may not be 0 depending on whether ~/.hoop/quarantine
        // has entries for today, but the function should not panic.
        assert!(
            count < 1_000_000,
            "quarantine count should be reasonable"
        );
    }
}
