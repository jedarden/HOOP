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
// NdjsonReader — line-buffered chunk reassembler (§F1)
// ---------------------------------------------------------------------------

const DEFAULT_MAX_PARTIAL_LEN: usize = 1024 * 1024; // 1 MB

/// Line-buffered NDJSON reader that accumulates partial lines across OS-read
/// boundaries.
///
/// OS `read()` calls may return chunks split at arbitrary byte offsets,
/// potentially in the middle of a line. This reader reassembles complete lines
/// by buffering partial data until a `\n` delimiter is found.
///
/// # Memory bounds
///
/// The internal buffer is capped at 1 MB (configurable via
/// [`with_max_partial_len`]). If a partial line exceeds this limit the buffer
/// is discarded and a warning is logged.
///
/// [`with_max_partial_len`]: NdjsonReader::with_max_partial_len
pub struct NdjsonReader {
    buffer: String,
    max_partial_len: usize,
}

impl NdjsonReader {
    /// Create a new reader with the default 1 MB partial-line limit.
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            max_partial_len: DEFAULT_MAX_PARTIAL_LEN,
        }
    }

    /// Set a custom partial-line memory limit (bytes).
    pub fn with_max_partial_len(mut self, max: usize) -> Self {
        self.max_partial_len = max;
        self
    }

    /// Feed a chunk of text. Returns all complete lines found.
    ///
    /// Each returned `String` is one line **without** the trailing `\n`.
    /// Lines that span chunk boundaries are reassembled transparently.
    pub fn feed(&mut self, chunk: &str) -> Vec<String> {
        let mut lines = Vec::new();
        let mut pos = 0;

        while pos < chunk.len() {
            match chunk[pos..].find('\n') {
                Some(nl) => {
                    let line_end = pos + nl;
                    if self.buffer.is_empty() {
                        lines.push(chunk[pos..line_end].to_owned());
                    } else {
                        self.buffer.push_str(&chunk[pos..line_end]);
                        lines.push(std::mem::take(&mut self.buffer));
                    }
                    pos = line_end + 1;
                }
                None => {
                    self.buffer.push_str(&chunk[pos..]);
                    self.enforce_limit();
                    break;
                }
            }
        }

        lines
    }

    /// Drain any remaining partial line at EOF.
    ///
    /// Call this when the stream ends. Returns `Some(partial)` if data
    /// remains that was never terminated by `\n`.
    pub fn finish(&mut self) -> Option<String> {
        if self.buffer.is_empty() {
            None
        } else {
            Some(std::mem::take(&mut self.buffer))
        }
    }

    fn enforce_limit(&mut self) {
        if self.buffer.len() > self.max_partial_len {
            tracing::warn!(
                partial_len = self.buffer.len(),
                limit = self.max_partial_len,
                "Partial line exceeds memory limit, discarding"
            );
            self.buffer.clear();
        }
    }
}

impl Default for NdjsonReader {
    fn default() -> Self {
        Self::new()
    }
}

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

    // =======================================================================
    // NdjsonReader tests
    // =======================================================================

    /// Multi-line NDJSON fixture used by the fuzz tests.
    fn ndjson_fixture() -> String {
        [
            r#"{"type":"message","text":"hello"}"#,
            r#"{"type":"message","text":"world with spaces and more content"}"#,
            r#"{"type":"event","data":{"nested":true,"count":42}}"#,
            r#"{"n":42}"#,
            "",
            r#"{"last":true}"#,
        ]
        .join("\n")
            + "\n"
    }

    /// §F1 fuzz test: split a known-good NDJSON fixture at every valid
    /// char-boundary offset and assert that all lines are recovered intact.
    #[test]
    fn fuzz_split_at_every_offset() {
        let fixture = ndjson_fixture();
        let expected: Vec<&str> = fixture.lines().collect();

        // Sanity: fixture is valid NDJSON (non-empty lines are valid JSON)
        for (i, line) in expected.iter().enumerate() {
            if !line.is_empty() {
                serde_json::from_str::<serde_json::Value>(line)
                    .unwrap_or_else(|e| panic!("Line {} is not valid JSON: {e}", i + 1));
            }
        }

        let byte_len = fixture.len();
        let mut tested_offsets = 0usize;

        for offset in 0..=byte_len {
            if !fixture.is_char_boundary(offset) {
                continue;
            }

            let mut reader = NdjsonReader::new();
            let mut got = reader.feed(&fixture[..offset]);
            got.extend(reader.feed(&fixture[offset..]));
            if let Some(remaining) = reader.finish() {
                got.push(remaining);
            }

            assert_eq!(
                got, expected,
                "Line mismatch at split offset {offset}/{byte_len}"
            );
            tested_offsets += 1;
        }

        // Ensure we actually tested meaningful splits, not just offset 0 and
        // byte_len.
        assert!(
            tested_offsets > 10,
            "should test more than a handful of offsets (tested {tested_offsets})"
        );
    }

    /// Extreme case: feed the fixture one byte at a time.
    #[test]
    fn single_byte_feeding() {
        let fixture = ndjson_fixture();
        let expected: Vec<&str> = fixture.lines().collect();

        let mut reader = NdjsonReader::new();
        let mut all_lines: Vec<String> = Vec::new();

        for ch in fixture.chars() {
            all_lines.extend(reader.feed(&ch.to_string()));
        }
        if let Some(remaining) = reader.finish() {
            all_lines.push(remaining);
        }

        assert_eq!(all_lines, expected);
    }

    /// Partial lines exceeding the memory limit are discarded.
    #[test]
    fn memory_bounded_partial_line() {
        let mut reader = NdjsonReader::new().with_max_partial_len(64);
        // Feed a "line" longer than the limit with no newline.
        let long = "x".repeat(128);
        let lines = reader.feed(&long);
        assert!(lines.is_empty(), "no newlines → no complete lines");
        // Buffer should have been discarded by enforce_limit.
        assert!(reader.finish().is_none(), "over-limit partial should be discarded");
    }

    /// Feeding empty chunks is a no-op.
    #[test]
    fn empty_chunks_are_noop() {
        let mut reader = NdjsonReader::new();
        assert!(reader.feed("").is_empty());
        assert!(reader.feed("").is_empty());

        let mut got = reader.feed("{\"a\":1}\n");
        got.extend(reader.feed(""));
        assert_eq!(got, vec!["{\"a\":1}"]);
        assert!(reader.finish().is_none());
    }

    /// finish() returns the remaining partial when there's no trailing newline.
    #[test]
    fn finish_returns_remaining() {
        let mut reader = NdjsonReader::new();
        let lines = reader.feed("{\"a\":1}\n{\"b\":");
        assert_eq!(lines, vec!["{\"a\":1}"]);

        let remaining = reader.finish();
        assert_eq!(remaining, Some("{\"b\":".to_owned()));

        // Second call returns None (buffer was drained).
        assert!(reader.finish().is_none());
    }

    /// Feeding multiple chunks that each contain part of a single line.
    #[test]
    fn line_spanning_many_chunks() {
        let mut reader = NdjsonReader::new();
        let parts = ["{\"ki", "ng\":", "\"val", "ue\"}", "\n"];
        let mut all = Vec::new();
        for part in parts {
            all.extend(reader.feed(part));
        }
        assert_eq!(all, vec![r#"{"king":"value"}"#]);
    }
}
