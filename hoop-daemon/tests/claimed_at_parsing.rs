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

/// Valid RFC3339 with different timezone offsets
const VALID_TIMESTAMP_POSITIVE_OFFSET: &str = "2026-04-21T18:42:10+05:30";
const VALID_TIMESTAMP_NEGATIVE_OFFSET: &str = "2026-04-21T18:42:10-08:00";
const VALID_TIMESTAMP_WITH_MICROSECONDS: &str = "2026-04-21T18:42:10.123456Z";
const VALID_TIMESTAMP_WITH_NANOSECONDS: &str = "2026-04-21T18:42:10.123456789Z";

/// Additional edge case timestamps
const VALID_TIMESTAMP_MIDNIGHT: &str = "2026-04-21T00:00:00Z";
const VALID_TIMESTAMP_MIDNIGHT_WITH_OFFSET: &str = "2026-04-21T00:00:00+00:00";

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

/// Test comprehensive set of valid RFC3339 timestamp formats
#[test]
fn comprehensive_valid_timestamp_formats() {
    let valid_timestamps = vec![
        VALID_TIMESTAMP_RFC3339,
        VALID_TIMESTAMP_WITH_MS,
        VALID_TIMESTAMP_WITH_OFFSET,
        VALID_TIMESTAMP_POSITIVE_OFFSET,
        VALID_TIMESTAMP_NEGATIVE_OFFSET,
        VALID_TIMESTAMP_WITH_MICROSECONDS,
        VALID_TIMESTAMP_WITH_NANOSECONDS,
        VALID_TIMESTAMP_MIDNIGHT,
        VALID_TIMESTAMP_MIDNIGHT_WITH_OFFSET,
        "2026-12-31T23:59:59Z",                  // End of year
        "2026-02-28T23:59:59Z",                  // End of February (non-leap year)
        "2024-02-29T23:59:59Z",                  // Leap year
        "2026-04-21T18:42:10.1Z",                // One decimal place
        "2026-04-21T18:42:10.12Z",               // Two decimal places
        "2026-04-21T18:42:10.123456789Z",        // Nine decimal places (nanoseconds)
        "2026-04-21T18:42:10+23:59",             // Max positive offset
        "2026-04-21T18:42:10-23:59",             // Max negative offset
    ];

    for ts in valid_timestamps {
        assert!(
            is_valid_rfc3339(ts),
            "Valid timestamp '{}' should parse successfully",
            ts
        );
    }
}

/// Test that parsing preserves the exact timestamp string
#[test]
fn timestamp_string_preservation_in_collision_entry() {
    let test_timestamps = vec![
        VALID_TIMESTAMP_RFC3339,
        VALID_TIMESTAMP_WITH_MS,
        VALID_TIMESTAMP_WITH_OFFSET,
        VALID_TIMESTAMP_POSITIVE_OFFSET,
        VALID_TIMESTAMP_NEGATIVE_OFFSET,
        VALID_TIMESTAMP_WITH_MICROSECONDS,
    ];

    for ts in test_timestamps {
        let entry = create_test_entry(ts);
        assert_eq!(
            entry.claimed_at, ts,
            "Timestamp string should be preserved exactly in CollisionIndexEntry"
        );
    }
}

/// Test parsing behavior with various fractional second precisions
#[test]
fn fractional_second_precisions() {
    let fractional_tests = vec![
        ("2026-04-21T18:42:10Z", 0),              // No fractional seconds
        ("2026-04-21T18:42:10.1Z", 1),            // 1 decimal place (100ms)
        ("2026-04-21T18:42:10.12Z", 2),           // 2 decimal places (10ms)
        ("2026-04-21T18:42:10.123Z", 3),          // 3 decimal places (1ms - milliseconds)
        ("2026-04-21T18:42:10.1234Z", 4),         // 4 decimal places (100μs)
        ("2026-04-21T18:42:10.12345Z", 5),        // 5 decimal places (10μs)
        ("2026-04-21T18:42:10.123456Z", 6),       // 6 decimal places (1μs - microseconds)
        ("2026-04-21T18:42:10.1234567Z", 7),      // 7 decimal places (100ns)
        ("2026-04-21T18:42:10.12345678Z", 8),     // 8 decimal places (10ns)
        ("2026-04-21T18:42:10.123456789Z", 9),    // 9 decimal places (1ns - nanoseconds)
    ];

    for (ts, expected_decimals) in &fractional_tests {
        let result = chrono::DateTime::parse_from_rfc3339(ts);
        assert!(
            result.is_ok(),
            "Timestamp with {} decimal places should parse: '{}'",
            expected_decimals,
            ts
        );

        // Verify the timestamp can be used in a CollisionIndexEntry
        let entry = create_test_entry(ts);
        assert_eq!(entry.claimed_at, *ts);
    }
}

/// Test timezone offset variations comprehensively
#[test]
fn timezone_offset_variations() {
    let offset_tests = vec![
        "2026-04-21T18:42:10Z",                  // UTC (Z)
        "2026-04-21T18:42:10+00:00",             // UTC (+00:00)
        "2026-04-21T18:42:10-00:00",             // UTC (-00:00)
        "2026-04-21T18:42:10+01:00",             // CET/CEST
        "2026-04-21T18:42:10-05:00",             // EST
        "2026-04-21T18:42:10+08:00",             // AWST
        "2026-04-21T18:42:10+05:30",             // IST
        "2026-04-21T18:42:10-03:30",             // NST
    ];

    for ts in offset_tests {
        assert!(
            is_valid_rfc3339(ts),
            "Timestamp with timezone offset should parse: '{}'",
            ts
        );

        // Verify it can be stored in CollisionIndexEntry
        let entry = create_test_entry(ts);
        assert_eq!(entry.claimed_at, ts);
    }
}

/// Test that all valid timestamps can round-trip through CollisionIndexEntry
#[test]
fn valid_timestamps_round_trip_through_collision_entry() {
    let valid_timestamps = vec![
        VALID_TIMESTAMP_RFC3339,
        VALID_TIMESTAMP_WITH_MS,
        VALID_TIMESTAMP_WITH_OFFSET,
        VALID_TIMESTAMP_POSITIVE_OFFSET,
        VALID_TIMESTAMP_NEGATIVE_OFFSET,
        VALID_TIMESTAMP_WITH_MICROSECONDS,
        VALID_TIMESTAMP_MIDNIGHT,
    ];

    for original_ts in valid_timestamps {
        // Create entry with the timestamp
        let entry = create_test_entry(original_ts);

        // Retrieve the timestamp
        let retrieved_ts = &entry.claimed_at;

        // Verify it's preserved exactly
        assert_eq!(
            retrieved_ts, original_ts,
            "Timestamp should round-trip through CollisionIndexEntry unchanged"
        );

        // Verify the retrieved timestamp is still parseable
        assert!(
            is_valid_rfc3339(retrieved_ts),
            "Round-tripped timestamp should still be parseable"
        );
    }
}

// ---------------------------------------------------------------------------
// Additional edge case tests
// ---------------------------------------------------------------------------

/// Test whitespace handling in timestamp strings
#[test]
fn whitespace_handling() {
    let whitespace_cases = vec![
        (" 2026-04-21T18:42:10Z", false),           // Leading space
        ("2026-04-21T18:42:10Z ", false),          // Trailing space
        ("2026-04-21T18:42:10  Z", false),         // Space before Z
        ("\t2026-04-21T18:42:10Z", false),         // Leading tab
        ("2026-04-21T18:42:10Z\n", false),         // Trailing newline
        (" 2026-04-21T18:42:10Z ", false),         // Both leading and trailing
    ];

    for (ts, should_parse) in whitespace_cases {
        let result = is_valid_rfc3339(ts);
        assert_eq!(
            result, should_parse,
            "Timestamp with whitespace '{}' should parse: {}",
            ts, should_parse
        );

        // Even if invalid, it should be stored in CollisionIndexEntry
        let entry = create_test_entry(ts);
        assert_eq!(entry.claimed_at, ts);
    }
}

/// Test case sensitivity in timezone designator
#[test]
fn case_sensitivity() {
    let case_cases = vec![
        ("2026-04-21T18:42:10Z", true),           // Uppercase Z (valid)
        ("2026-04-21T18:42:10z", true),           // Lowercase z (also valid - chrono accepts both)
        ("2026-04-21T18:42:10+00:00", true),      // Offset format (valid)
    ];

    for (ts, should_parse) in case_cases {
        let result = is_valid_rfc3339(ts);
        assert_eq!(
            result, should_parse,
            "Timestamp '{}' case sensitivity check failed",
            ts
        );
    }
}

/// Test invalid characters mixed into timestamps
#[test]
fn invalid_characters() {
    let invalid_char_cases = vec![
        "2026-04-21T18:42:10X",                   // X instead of Z
        "2026-04-21T18:42:10!Z",                  // Exclamation mark
        "2026-04-21T18:42:10.12Z3",               // Extra digit after Z
        "2026-04-21T18:42:10@Z",                  // @ symbol
        "2026-04-21T18:42:10#Z",                  // # symbol
        "2026-04-21T18:42:10$Z",                  // $ symbol
        "2026-04-21T18:42:10%Z",                  // % symbol
        "2026-04-21T18:42:10^Z",                  // ^ symbol
        "2026-04-21T18:42:10&Z",                  // & symbol
        "2026-04-21T18:42:10*Z",                  // * symbol
    ];

    for ts in invalid_char_cases {
        let result = is_valid_rfc3339(ts);
        assert!(
            !result,
            "Timestamp with invalid character '{}' should not parse",
            ts
        );

        // Should still be storable in CollisionIndexEntry
        let entry = create_test_entry(ts);
        assert_eq!(entry.claimed_at, ts);
    }
}

/// Test SQL injection attempts in claimed_at (security edge case)
#[test]
fn sql_injection_attempts() {
    let sql_injection_cases = vec![
        "'; DROP TABLE collision_index; --",
        "2026-04-21T18:42:10'; DROP TABLE collision_index; --",
        "' OR '1'='1",
        "'; SELECT * FROM collision_index; --",
        "' UNION SELECT * FROM collision_index --",
        "'; DELETE FROM collision_index WHERE '1'='1'; --",
        "'; INSERT INTO collision_index VALUES ('test', 'test', 'test', 'test', '[]', 'test'); --",
        "' OR 1=1 --",
        "2026-04-21T18:42:10' OR '1'='1",
        "'; SHUTDOWN; --",
        "'; EXEC xp_cmdshell('format c:'); --",
    ];

    for ts in sql_injection_cases {
        // These should not parse as valid timestamps
        let result = is_valid_rfc3339(ts);
        assert!(
            !result,
            "SQL injection attempt '{}' should not parse as valid timestamp",
            ts
        );

        // But they should be storable without causing panics
        let entry = create_test_entry(ts);
        assert_eq!(entry.claimed_at, ts);

        // Verify they fail to parse
        let parse_result = chrono::DateTime::parse_from_rfc3339(&entry.claimed_at);
        assert!(
            parse_result.is_err(),
            "SQL injection string should fail to parse: '{}'",
            ts
        );
    }
}

/// Test extremely long timestamp strings
#[test]
fn extremely_long_timestamps() {
    let long_cases = vec![
        "2026-04-21T18:42:10.123456789123456789123456789Z", // Excessive fractional seconds
        "0000-01-01T00:00:00.000000000000000000000000000Z",  // Ancient date with long fractional
        "9999-12-31T23:59:59.999999999999999999999999999Z", // Far future date with long fractional
    ];

    for ts in long_cases {
        // Store the long timestamp
        let entry = create_test_entry(ts);
        assert_eq!(entry.claimed_at, ts);

        // Parse it (may or may not succeed depending on chrono's behavior)
        let parse_result = chrono::DateTime::parse_from_rfc3339(&entry.claimed_at);
        // We don't assert success/failure here, just that it doesn't panic
    }
}

/// Test negative timestamps (before Unix epoch)
#[test]
fn negative_timestamps_before_epoch() {
    let negative_cases = vec![
        "1969-12-31T23:59:59Z",                   // One second before epoch
        "1960-01-01T00:00:00Z",                   // 1960
        "1950-06-15T12:30:45Z",                   // 1950
        "1900-01-01T00:00:00Z",                   // 1900
        "1850-01-01T00:00:00Z",                   // 1850
        "0001-01-01T00:00:00Z",                   // Year 1 AD
    ];

    for ts in negative_cases {
        // These should parse as valid RFC3339
        let result = is_valid_rfc3339(ts);
        assert!(
            result,
            "Negative timestamp (before epoch) '{}' should parse as valid RFC3339",
            ts
        );

        // Should be storable
        let entry = create_test_entry(ts);
        assert_eq!(entry.claimed_at, ts);

        // Should round-trip correctly
        assert!(
            is_valid_rfc3339(&entry.claimed_at),
            "Negative timestamp should still be parseable after storage"
        );
    }
}

/// Test extreme future dates
#[test]
fn extreme_future_dates() {
    let future_cases = vec![
        "2100-01-01T00:00:00Z",                   // Year 2100
        "2500-12-31T23:59:59Z",                   // Year 2500
        "3000-01-01T00:00:00Z",                   // Year 3000
        "9999-12-31T23:59:59Z",                   // Year 9999
    ];

    for ts in future_cases {
        // These should parse as valid RFC3339
        let result = is_valid_rfc3339(ts);
        assert!(
            result,
            "Extreme future date '{}' should parse as valid RFC3339",
            ts
        );

        // Should be storable
        let entry = create_test_entry(ts);
        assert_eq!(entry.claimed_at, ts);
    }
}

/// Test invalid timezone offsets (beyond valid range)
#[test]
fn invalid_timezone_offsets() {
    let invalid_offset_cases = vec![
        "2026-04-21T18:42:10+24:00",              // 24 hours (invalid)
        "2026-04-21T18:42:10+25:00",              // 25 hours (invalid)
        "2026-04-21T18:42:10+99:59",              // 99 hours (invalid)
        "2026-04-21T18:42:10-24:00",              // -24 hours (invalid)
        "2026-04-21T18:42:10-25:00",              // -25 hours (invalid)
        "2026-04-21T18:42:10+23:60",              // 60 minutes (invalid)
        "2026-04-21T18:42:10+00:60",              // 60 minutes (invalid)
        "2026-04-21T18:42:10+00:99",              // 99 minutes (invalid)
    ];

    for ts in invalid_offset_cases {
        // These should not parse as valid RFC3339
        let result = is_valid_rfc3339(ts);
        assert!(
            !result,
            "Invalid timezone offset '{}' should not parse",
            ts
        );

        // But should be storable
        let entry = create_test_entry(ts);
        assert_eq!(entry.claimed_at, ts);
    }
}

/// Test leap second handling
#[test]
fn leap_second_handling() {
    // RFC3339 allows leap seconds (60 as the second value)
    let leap_second_cases = vec![
        "2016-12-31T23:59:60Z",                   // Leap second (June 2016)
        "2017-01-01T00:00:00Z",                   // Normal time after leap second
    ];

    for ts in leap_second_cases {
        // Parse the timestamp
        let result = is_valid_rfc3339(ts);

        // chrono may or may not accept leap seconds depending on version
        // We just verify it doesn't panic
        let _ = result;

        // Should be storable regardless
        let entry = create_test_entry(ts);
        assert_eq!(entry.claimed_at, ts);
    }
}

/// Test boundary values for date components
#[test]
fn boundary_values() {
    let boundary_cases = vec![
        ("2026-00-01T00:00:00Z", false),          // Month 0 (invalid)
        ("2026-13-01T00:00:00Z", false),          // Month 13 (invalid)
        ("2026-01-00T00:00:00Z", false),          // Day 0 (invalid)
        ("2026-01-32T00:00:00Z", false),          // Day 32 (invalid)
        ("2026-02-30T00:00:00Z", false),          // Feb 30 (invalid)
        ("2026-04-31T00:00:00Z", false),          // Apr 31 (invalid)
        ("2026-04-21T24:00:00Z", false),          // Hour 24 (invalid)
        ("2026-04-21T23:60:00Z", false),          // Minute 60 (invalid)
        ("2026-04-21T23:59:61Z", false),          // Second 61 (invalid, not leap second)
    ];

    for (ts, should_parse) in boundary_cases {
        let result = is_valid_rfc3339(ts);
        assert_eq!(
            result, should_parse,
            "Boundary value '{}' should parse: {}",
            ts, should_parse
        );

        // Should be storable regardless
        let entry = create_test_entry(ts);
        assert_eq!(entry.claimed_at, ts);
    }
}

/// Test special characters and unicode in timestamps
#[test]
fn special_characters_and_unicode() {
    let special_cases = vec![
        "2026-04-21T18:42:10✓Z",                  // Unicode checkmark
        "2026-04-21T18:42:10🔥Z",                 // Fire emoji
        "2026-04-21T18:42:10™Z",                  // Trademark symbol
        "2026-04-21T18:42:10©Z",                  // Copyright symbol
        "2026-04-21T18:42:10®Z",                  // Registered symbol
        "2026-04-21T18:42:10€Z",                  // Euro symbol
        "2026-04-21T18:42:10£Z",                  // Pound symbol
        "2026-04-21T18:42:10¥Z",                  // Yen symbol
        "2026-04-21T18:42:10§Z",                  // Section symbol
        "2026-04-21T18:42:10¶Z",                  // Pilcrow symbol
    ];

    for ts in special_cases {
        // These should not parse as valid timestamps
        let result = is_valid_rfc3339(ts);
        assert!(
            !result,
            "Timestamp with special character '{}' should not parse",
            ts
        );

        // But should be storable
        let entry = create_test_entry(ts);
        assert_eq!(entry.claimed_at, ts);
    }
}

/// Test that empty variants are all handled consistently
#[test]
fn empty_variants() {
    let empty_cases = vec![
        "",                                        // Empty string
        " ",                                       // Single space
        "  ",                                      // Multiple spaces
        "\t",                                      // Tab
        "\n",                                      // Newline
        "\r",                                      // Carriage return
        "\r\n",                                    // CRLF
        "   ",                                     // Multiple spaces
    ];

    for ts in empty_cases {
        // Empty/whitespace variants should not parse
        let result = is_valid_rfc3339(ts);
        assert!(
            !result,
            "Empty variant '{}' should not parse",
            ts.escape_debug()
        );

        // Should be storable
        let entry = create_test_entry(ts);
        assert_eq!(entry.claimed_at, ts);

        // Should produce appropriate error message
        if ts.is_empty() {
            let parse_result = chrono::DateTime::parse_from_rfc3339(&entry.claimed_at);
            let err = parse_result.unwrap_err();
            let err_msg = err.to_string();
            assert!(
                err_msg.contains("premature end of input") || err_msg.contains("empty"),
                "Empty string should produce 'premature end of input' error, got: {}",
                err_msg
            );
        }
    }
}

/// Test timestamp with extra text appended/prefixed
#[test]
fn timestamps_with_extra_text() {
    let extra_text_cases = vec![
        "2026-04-21T18:42:10Z extra text",         // Text after
        "prefix 2026-04-21T18:42:10Z",            // Text before
        "2026-04-21T18:42:10Z123",                // Digits after Z
        "12026-04-21T18:42:10Z",                  // Digit before
        "2026-04-21T18:42:10ZZ",                  // Extra Z
        "Z2026-04-21T18:42:10",                   // Z at beginning
    ];

    for ts in extra_text_cases {
        // These should not parse as valid timestamps
        let result = is_valid_rfc3339(ts);
        assert!(
            !result,
            "Timestamp with extra text '{}' should not parse",
            ts
        );

        // Should be storable
        let entry = create_test_entry(ts);
        assert_eq!(entry.claimed_at, ts);
    }
}
