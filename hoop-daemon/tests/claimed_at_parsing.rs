//! Test for claimed_at parsing error reproduction
//!
//! This test reproduces the issue where invalid claimed_at formats cause
//! "premature end of input" errors when processing beads.
//!
//! Background: bead bf-5i1ln failed to close with error:
//! "Invalid claimed_at format: premature end of input"
//!
//! This test verifies that:
//! 1. Valid RFC3339 timestamps parse correctly
//! 2. Invalid timestamp formats are detected and handled gracefully
//! 3. Edge cases (empty strings, partial timestamps) don't cause panics

use hoop_daemon::fleet;

// ---------------------------------------------------------------------------
// Test data
// ---------------------------------------------------------------------------

/// Valid RFC3339 timestamp (the expected format)
const VALID_TIMESTAMP_RFC3339: &str = "2026-04-21T18:42:10Z";

/// Valid RFC3339 timestamp with milliseconds
const VALID_TIMESTAMP_WITH_MS: &str = "2026-04-21T18:42:10.123Z";

/// Valid RFC3339 timestamp with timezone offset
const VALID_TIMESTAMP_WITH_OFFSET: &str = "2026-04-21T18:42:10+00:00";

/// Invalid timestamp (empty string) - reproduces "premature end of input"
const INVALID_TIMESTAMP_EMPTY: &str = "";

/// Invalid timestamp (partial date) - missing time component
const INVALID_TIMESTAMP_PARTIAL: &str = "2026-04-21";

/// Invalid timestamp (wrong format)
const INVALID_TIMESTAMP_WRONG_FORMAT: &str = "April 21, 2026";

/// Invalid timestamp (garbage)
const INVALID_TIMESTAMP_GARBAGE: &str = "not-a-timestamp";

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Create a CollisionIndexEntry for testing
fn create_test_entry(claimed_at: &str) -> fleet::CollisionIndexEntry {
    let now = chrono::Utc::now().to_rfc3339();
    fleet::CollisionIndexEntry {
        bead_id: "bd-test001".to_string(),
        project: "test-project".to_string(),
        worker: Some("worker-alpha".to_string()),
        claimed_at: claimed_at.to_string(),
        file_paths: vec!["src/main.rs".to_string()],
        updated_at: now,
    }
}

/// Verify a timestamp string is valid RFC3339
fn is_valid_rfc3339(ts: &str) -> bool {
    chrono::DateTime::parse_from_rfc3339(ts).is_ok()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn valid_rfc3339_timestamp_parses() {
    assert!(
        is_valid_rfc3339(VALID_TIMESTAMP_RFC3339),
        "Valid RFC3339 timestamp should parse"
    );
}

#[test]
fn valid_rfc3339_with_milliseconds_parses() {
    assert!(
        is_valid_rfc3339(VALID_TIMESTAMP_WITH_MS),
        "Valid RFC3339 timestamp with milliseconds should parse"
    );
}

#[test]
fn valid_rfc3339_with_offset_parses() {
    assert!(
        is_valid_rfc3339(VALID_TIMESTAMP_WITH_OFFSET),
        "Valid RFC3339 timestamp with offset should parse"
    );
}

#[test]
fn empty_timestamp_is_invalid() {
    assert!(
        !is_valid_rfc3339(INVALID_TIMESTAMP_EMPTY),
        "Empty timestamp should be invalid (reproduces 'premature end of input')"
    );
}

#[test]
fn partial_timestamp_is_invalid() {
    assert!(
        !is_valid_rfc3339(INVALID_TIMESTAMP_PARTIAL),
        "Partial timestamp (date only) should be invalid"
    );
}

#[test]
fn wrong_format_timestamp_is_invalid() {
    assert!(
        !is_valid_rfc3339(INVALID_TIMESTAMP_WRONG_FORMAT),
        "Wrong format timestamp should be invalid"
    );
}

#[test]
fn garbage_timestamp_is_invalid() {
    assert!(
        !is_valid_rfc3339(INVALID_TIMESTAMP_GARBAGE),
        "Garbage timestamp should be invalid"
    );
}

#[test]
fn collision_entry_with_valid_timestamp_creates_successfully() {
    let entry = create_test_entry(VALID_TIMESTAMP_RFC3339);

    // Verify the entry has the expected claimed_at value
    assert_eq!(entry.claimed_at, VALID_TIMESTAMP_RFC3339);
    assert_eq!(entry.bead_id, "bd-test001");
    assert_eq!(entry.project, "test-project");
    assert_eq!(entry.worker, Some("worker-alpha".to_string()));
}

#[test]
fn collision_entry_with_empty_timestamp_has_field_set() {
    let entry = create_test_entry(INVALID_TIMESTAMP_EMPTY);

    // The entry should still be creatable with an invalid timestamp
    // (the field is stored as TEXT in SQLite)
    assert_eq!(entry.claimed_at, INVALID_TIMESTAMP_EMPTY);
    assert_eq!(entry.bead_id, "bd-test001");
}

#[test]
fn collision_entry_with_partial_timestamp_has_field_set() {
    let entry = create_test_entry(INVALID_TIMESTAMP_PARTIAL);

    // The entry should still be creatable with a partial timestamp
    assert_eq!(entry.claimed_at, INVALID_TIMESTAMP_PARTIAL);
    assert_eq!(entry.bead_id, "bd-test001");
}

/// Test that demonstrates the issue: when a timestamp string is invalid,
/// it can still be stored in the CollisionIndexEntry, but any code that
/// tries to parse it later will fail.
#[test]
fn demonstrates_premature_end_of_input_issue() {
    let invalid_timestamps = vec![
        INVALID_TIMESTAMP_EMPTY,
        INVALID_TIMESTAMP_PARTIAL,
        INVALID_TIMESTAMP_WRONG_FORMAT,
        INVALID_TIMESTAMP_GARBAGE,
    ];

    for ts in invalid_timestamps {
        let entry = create_test_entry(ts);

        // The entry accepts the invalid timestamp
        assert_eq!(entry.claimed_at, ts);

        // But parsing it fails
        let parse_result = chrono::DateTime::parse_from_rfc3339(&entry.claimed_at);
        assert!(
            parse_result.is_err(),
            "Timestamp '{}' should fail to parse",
            ts
        );

        // The error message for empty string is "premature end of input"
        if ts.is_empty() {
            let err = parse_result.unwrap_err();
            let err_msg = err.to_string();
            assert!(
                err_msg.contains("premature end of input") || err_msg.contains("empty"),
                "Empty timestamp should produce 'premature end of input' error, got: {}",
                err_msg
            );
        }
    }
}

/// Test various edge cases that might cause parsing issues
#[test]
fn edge_case_timestamps() {
    let edge_cases = vec![
        ("2026-04-21T18:42:10Z", true),           // Standard format
        ("2026-04-21T18:42:10.0Z", true),         // Zero milliseconds
        ("2026-04-21T18:42:10.000Z", true),       // Three-digit milliseconds
        ("2026-04-21T18:42:10.000000Z", true),   // Six-digit fractional (microseconds) - chrono accepts these
        ("2026-04-21T18:42:10+00:00", true),     // Positive offset
        ("2026-04-21T18:42:10-00:00", true),     // Negative offset
        ("2026-04-21 18:42:10Z", true),          // Space instead of T - chrono accepts this
        ("2026-04-21T18:42:10", false),          // Missing Z (no timezone - invalid)
    ];

    for (ts, should_parse) in edge_cases {
        let result = is_valid_rfc3339(ts);
        assert_eq!(
            result,
            should_parse,
            "Timestamp '{}' parse result mismatch: expected {}, got {}",
            ts,
            should_parse,
            result
        );
    }
}
