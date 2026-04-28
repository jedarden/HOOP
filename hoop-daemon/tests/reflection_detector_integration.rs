//! Integration tests for the reflection detector.
//!
//! These tests use a temporary fleet.db to test the full flow:
//! 1. Create operator Stitches with repeated patterns
//! 2. Run the reflection detector
//! 3. Verify that patterns are proposed in the reflection ledger
//!
//! Plan reference: §6 Phase 5 marquee #12

use tempfile::TempDir;
use rusqlite::Connection;
use serde_json::json;

/// Test database setup helper
fn setup_test_db(temp_dir: &TempDir) -> rusqlite::Connection {
    let db_path = temp_dir.path().join("fleet.db");
    let conn = Connection::open(&db_path).unwrap();

    // Create schemas
    conn.execute(
        "CREATE TABLE stitches (
            id TEXT PRIMARY KEY NOT NULL,
            project TEXT NOT NULL,
            kind TEXT NOT NULL,
            classification TEXT NOT NULL,
            title TEXT NOT NULL,
            created_by TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT,
            closed_at TEXT,
            last_activity_at TEXT NOT NULL,
            archived INTEGER DEFAULT 0,
            schema_version TEXT NOT NULL
        )",
        [],
    ).unwrap();

    conn.execute(
        "CREATE TABLE stitch_messages (
            stitch_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            ts TEXT NOT NULL,
            PRIMARY KEY (stitch_id, ts)
        )",
        [],
    ).unwrap();

    conn.execute(
        "CREATE TABLE stitch_beads (
            stitch_id TEXT NOT NULL,
            bead_id TEXT NOT NULL,
            linked_at TEXT NOT NULL,
            PRIMARY KEY (stitch_id, bead_id)
        )",
        [],
    ).unwrap();

    conn.execute(
        "CREATE TABLE reflection_ledger (
            id TEXT PRIMARY KEY NOT NULL,
            scope TEXT NOT NULL,
            rule TEXT NOT NULL,
            reason TEXT NOT NULL,
            source_stitches TEXT NOT NULL DEFAULT '[]',
            status TEXT NOT NULL,
            created_at TEXT NOT NULL,
            last_applied TEXT,
            applied_count INTEGER DEFAULT 0
        )",
        [],
    ).unwrap();

    conn.execute(
        "CREATE INDEX idx_stitches_kind_classification ON stitches(kind, classification)",
        [],
    ).unwrap();

    conn.execute(
        "CREATE INDEX idx_stitch_messages_stitch_ts ON stitch_messages(stitch_id, ts)",
        [],
    ).unwrap();

    conn
}

/// Insert a test Stitch
fn insert_stitch(conn: &Connection, id: &str, kind: &str, classification: &str, created_at: &str) {
    conn.execute(
        "INSERT INTO stitches (id, project, kind, classification, title, created_by, created_at, updated_at, last_activity_at, schema_version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        [
            id,
            "test-project",
            kind,
            classification,
            &format!("Test Stitch {}", id),
            "test-operator",
            created_at,
            created_at,
            created_at,
            "1.0.0",
        ],
    ).unwrap();
}

/// Insert a test message
fn insert_message(conn: &Connection, stitch_id: &str, role: &str, content: &str, ts: &str) {
    let content_json = json!({"text": content}).to_string();
    conn.execute(
        "INSERT INTO stitch_messages (stitch_id, role, content, ts) VALUES (?1, ?2, ?3, ?4)",
        [stitch_id, role, &content_json, ts],
    ).unwrap();
}

/// Test: Detect repeated negative patterns across operator Stitches
#[test]
fn test_detect_repeated_negative_patterns() {
    let temp_dir = TempDir::new().unwrap();
    let conn = setup_test_db(&temp_dir);

    // Create 3 operator Stitches with the same negative instruction
    for i in 0..3 {
        let stitch_id = &format!("st-neg-{}", i);
        let created_at = &format!("2026-04-26T10:00:{}Z", i * 10);
        insert_stitch(&conn, stitch_id, "operator", "operator", created_at);

        // User message: "Don't use unwrap()"
        insert_message(
            &conn,
            stitch_id,
            "user",
            "Don't use unwrap() in production code",
            created_at,
        );

        // Assistant response
        insert_message(
            &conn,
            stitch_id,
            "assistant",
            "I'll use proper error handling",
            created_at,
        );
    }

    // Add a non-operator stitch (should be ignored)
    insert_stitch(&conn, "st-worker-1", "worker", "fleet", "2026-04-26T10:01:00Z");
    insert_message(
        &conn,
        "st-worker-1",
        "user",
        "Don't use unwrap()",
        "2026-04-26T10:01:00Z",
    );

    // Set the test database path via environment variable
    std::env::set_var("_HOOP_FLEET_DB_PATH", temp_dir.path().join("fleet.db"));

    // Run the reflection detector
    let config = hoop_daemon::reflection_detector::ReflectionDetectorConfig {
        scan_window_days: 30,
        min_occurrences: 3,
        ..Default::default()
    };

    let result = hoop_daemon::reflection_detector::run_detection(&config);
    assert!(result.is_ok(), "run_detection should succeed");

    let proposed = result.unwrap();
    assert_eq!(proposed, 1, "Should propose 1 pattern from 3 similar negatives");

    // Verify the reflection ledger entry
    let stmt = conn
        .prepare("SELECT rule, status, source_stitches FROM reflection_ledger")
        .unwrap();

    let entries: Vec<(String, String, String)> = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert_eq!(entries.len(), 1, "Should have 1 reflection ledger entry");
    let (rule, status, source_stitches_json) = &entries[0];
    assert_eq!(status, "proposed");
    assert!(
        rule.to_lowercase().contains("unwrap") || rule.to_lowercase().contains("don't"),
        "Rule should mention unwrap or don't: {}",
        rule
    );

    let source_stitches: Vec<String> = serde_json::from_str(source_stitches_json).unwrap();
    assert_eq!(source_stitches.len(), 3, "Should have 3 source stitches");

    // Clean up env var
    std::env::remove_var("_HOOP_FLEET_DB_PATH");
}

/// Test: Detect repeated preference patterns
#[test]
fn test_detect_repeated_preference_patterns() {
    let temp_dir = TempDir::new().unwrap();
    let conn = setup_test_db(&temp_dir);

    // Create 3 operator Stitches with the same preference
    for i in 0..3 {
        let stitch_id = &format!("st-pref-{}", i);
        let created_at = &format!("2026-04-26T11:00:{}Z", i * 10);
        insert_stitch(&conn, stitch_id, "operator", "operator", created_at);

        insert_message(
            &conn,
            stitch_id,
            "user",
            "I prefer early returns over nested if-else",
            created_at,
        );
    }

    std::env::set_var("_HOOP_FLEET_DB_PATH", temp_dir.path().join("fleet.db"));

    let config = hoop_daemon::reflection_detector::ReflectionDetectorConfig {
        scan_window_days: 30,
        min_occurrences: 3,
        ..Default::default()
    };

    let result = hoop_daemon::reflection_detector::run_detection(&config);
    assert!(result.is_ok());

    let proposed = result.unwrap();
    assert_eq!(proposed, 1, "Should propose 1 preference pattern");

    std::env::remove_var("_HOOP_FLEET_DB_PATH");
}

/// Test: Detect repeated correction patterns
#[test]
fn test_detect_repeated_correction_patterns() {
    let temp_dir = TempDir::new().unwrap();
    let conn = setup_test_db(&temp_dir);

    // Create 3 operator Stitches with corrections
    for i in 0..3 {
        let stitch_id = &format!("st-corr-{}", i);
        let created_at = &format!("2026-04-26T12:00:{}Z", i * 10);
        insert_stitch(&conn, stitch_id, "operator", "operator", created_at);

        insert_message(
            &conn,
            stitch_id,
            "user",
            "No, I meant the config file, not the source",
            created_at,
        );
    }

    std::env::set_var("_HOOP_FLEET_DB_PATH", temp_dir.path().join("fleet.db"));

    let config = hoop_daemon::reflection_detector::ReflectionDetectorConfig {
        scan_window_days: 30,
        min_occurrences: 3,
        ..Default::default()
    };

    let result = hoop_daemon::reflection_detector::run_detection(&config);
    assert!(result.is_ok());

    let proposed = result.unwrap();
    assert_eq!(proposed, 1, "Should propose 1 correction pattern");

    std::env::remove_var("_HOOP_FLEET_DB_PATH");
}

/// Test: Only operator Stitches are scanned
#[test]
fn test_only_operator_stitches_scanned() {
    let temp_dir = TempDir::new().unwrap();
    let conn = setup_test_db(&temp_dir);

    // Create 3 worker stitches with the same pattern (should be ignored)
    for i in 0..3 {
        let stitch_id = &format!("st-worker-{}", i);
        let created_at = &format!("2026-04-26T13:00:{}Z", i * 10);
        insert_stitch(&conn, stitch_id, "worker", "fleet", created_at);

        insert_message(
            &conn,
            stitch_id,
            "user",
            "Don't use unwrap() in production code",
            created_at,
        );
    }

    // Create 2 operator Stitches (below threshold)
    for i in 0..2 {
        let stitch_id = &format!("st-operator-{}", i);
        let created_at = &format!("2026-04-26T13:01:{}Z", i * 10);
        insert_stitch(&conn, stitch_id, "operator", "operator", created_at);

        insert_message(
            &conn,
            stitch_id,
            "user",
            "Don't use unwrap() in production code",
            created_at,
        );
    }

    std::env::set_var("_HOOP_FLEET_DB_PATH", temp_dir.path().join("fleet.db"));

    let config = hoop_daemon::reflection_detector::ReflectionDetectorConfig {
        scan_window_days: 30,
        min_occurrences: 3,
        ..Default::default()
    };

    let result = hoop_daemon::reflection_detector::run_detection(&config);
    assert!(result.is_ok());

    let proposed = result.unwrap();
    assert_eq!(proposed, 0, "Should not propose patterns: worker stitches ignored, operator below threshold");

    std::env::remove_var("_HOOP_FLEET_DB_PATH");
}

/// Test: Synthetic repeated-instruction fixtures
///
/// This test matches the acceptance criteria:
/// "Tested against synthetic repeated-instruction fixtures"
#[test]
fn test_synthetic_repeated_instruction_fixtures() {
    let temp_dir = TempDir::new().unwrap();
    let conn = setup_test_db(&temp_dir);

    // Fixture 1: 3 similar negatives about unwrap
    let fixtures = vec![
        ("st-001", "Don't use unwrap() in production code"),
        ("st-002", "Please don't use unwrap() anywhere in the codebase"),
        ("st-003", "Don't use unwrap() — use proper error handling"),
    ];

    for (i, (stitch_id, content)) in fixtures.iter().enumerate() {
        let created_at = &format!("2026-04-26T14:00:{}Z", i * 10);
        insert_stitch(&conn, stitch_id, "operator", "operator", created_at);
        insert_message(&conn, stitch_id, "user", content, created_at);
        insert_message(&conn, stitch_id, "assistant", "Understood", created_at);
    }

    // Fixture 2: 3 similar preferences about Result type
    let result_fixtures = vec![
        ("st-004", "I prefer you use Result instead of unwrap"),
        ("st-005", "Always use Result instead of unwrap in this project"),
        ("st-006", "I always want Result types, not unwrap calls"),
    ];

    for (i, (stitch_id, content)) in result_fixtures.iter().enumerate() {
        let created_at = &format!("2026-04-26T14:01:{}Z", i * 10);
        insert_stitch(&conn, stitch_id, "operator", "operator", created_at);
        insert_message(&conn, stitch_id, "user", content, created_at);
    }

    // Add some noise that shouldn't trigger detection
    insert_stitch(&conn, "st-noise", "operator", "operator", "2026-04-26T14:02:00Z");
    insert_message(&conn, "st-noise", "user", "What does this function do?", "2026-04-26T14:02:00Z");

    std::env::set_var("_HOOP_FLEET_DB_PATH", temp_dir.path().join("fleet.db"));

    let config = hoop_daemon::reflection_detector::ReflectionDetectorConfig {
        scan_window_days: 30,
        min_occurrences: 3,
        similarity_threshold: 0.40,
        ..Default::default()
    };

    let result = hoop_daemon::reflection_detector::run_detection(&config);
    assert!(result.is_ok());

    let proposed = result.unwrap();
    assert!(
        proposed >= 1,
        "Should detect at least 1 pattern from synthetic fixtures, got {}",
        proposed
    );

    // Verify patterns span multiple Stitches
    let stmt = conn
        .prepare("SELECT source_stitches FROM reflection_ledger WHERE status = 'proposed'")
        .unwrap();

    let entries: Vec<Vec<String>> = stmt
        .query_map([], |row| {
            let json: String = row.get(0)?;
            Ok(serde_json::from_str::<Vec<String>>(&json).unwrap())
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    let has_multi_source = entries.iter().any(|stitches| stitches.len() >= 3);
    assert!(
        has_multi_source,
        "At least one pattern should span 3+ Stitches"
    );

    std::env::remove_var("_HOOP_FLEET_DB_PATH");
}

/// Test: Time window filtering
#[test]
fn test_time_window_filtering() {
    let temp_dir = TempDir::new().unwrap();
    let conn = setup_test_db(&temp_dir);

    // Create 3 Stitches, but 2 are outside the 30-day window
    let old_date = "2026-03-01T10:00:00Z"; // More than 30 days ago
    let recent_date = "2026-04-26T10:00:00Z";

    // Old Stitches (should be ignored)
    for i in 0..2 {
        let stitch_id = &format!("st-old-{}", i);
        insert_stitch(&conn, stitch_id, "operator", "operator", old_date);
        insert_message(&conn, stitch_id, "user", "Don't use unwrap()", old_date);
    }

    // Recent Stitch (not enough to hit threshold)
    insert_stitch(&conn, "st-recent", "operator", "operator", recent_date);
    insert_message(&conn, "st-recent", "user", "Don't use unwrap()", recent_date);

    std::env::set_var("_HOOP_FLEET_DB_PATH", temp_dir.path().join("fleet.db"));

    let config = hoop_daemon::reflection_detector::ReflectionDetectorConfig {
        scan_window_days: 30, // Should exclude the old Stitches
        min_occurrences: 3,
        ..Default::default()
    };

    let result = hoop_daemon::reflection_detector::run_detection(&config);
    assert!(result.is_ok());

    let proposed = result.unwrap();
    assert_eq!(proposed, 0, "Should not propose patterns: old stitches outside window");

    std::env::remove_var("_HOOP_FLEET_DB_PATH");
}

/// Test: is_operator_stitch function
#[test]
fn test_is_operator_stitch() {
    let temp_dir = TempDir::new().unwrap();
    let conn = setup_test_db(&temp_dir);

    // Insert an operator stitch
    insert_stitch(&conn, "st-operator-1", "operator", "operator", "2026-04-26T10:00:00Z");

    // Insert a worker stitch
    insert_stitch(&conn, "st-worker-1", "worker", "fleet", "2026-04-26T10:01:00Z");

    // Insert an operator-kind but fleet-classification stitch
    insert_stitch(&conn, "st-fleet-operator", "operator", "fleet", "2026-04-26T10:02:00Z");

    std::env::set_var("_HOOP_FLEET_DB_PATH", temp_dir.path().join("fleet.db"));

    assert!(
        hoop_daemon::fleet::is_operator_stitch("st-operator-1").unwrap(),
        "operator+operator should be true"
    );
    assert!(
        !hoop_daemon::fleet::is_operator_stitch("st-worker-1").unwrap(),
        "worker+fleet should be false"
    );
    assert!(
        !hoop_daemon::fleet::is_operator_stitch("st-fleet-operator").unwrap(),
        "operator+fleet should be false"
    );
    assert!(
        !hoop_daemon::fleet::is_operator_stitch("st-nonexistent").unwrap(),
        "nonexistent stitch should be false"
    );

    std::env::remove_var("_HOOP_FLEET_DB_PATH");
}

/// Test: Reflection injection audit (§4.7)
///
/// Acceptance criteria:
/// - Audit row per injection with kind: reflection_injected
/// - last_applied + applied_count updated atomically
/// - Test: synthetic rule injected → audit query returns correct row
#[test]
fn test_reflection_injection_audit() {
    let temp_dir = TempDir::new().unwrap();
    let conn = setup_test_db(&temp_dir);

    // Insert approved reflection rules
    let rule1_id = uuid::Uuid::new_v4().to_string();
    let rule2_id = uuid::Uuid::new_v4().to_string();
    let now = "2026-04-26T10:00:00Z";

    conn.execute(
        "INSERT INTO reflection_ledger (id, scope, rule, reason, status, created_at, last_applied, applied_count)
         VALUES (?1, 'global', 'always run tests before closing', 'operator repeated 3 times', 'approved', ?2, NULL, 0)",
        [rule1_id.clone(), now],
    ).unwrap();

    conn.execute(
        "INSERT INTO reflection_ledger (id, scope, rule, reason, status, created_at, last_applied, applied_count)
         VALUES (?1, 'project:hoop', 'never edit fleet.db directly', 'one incident of corruption', 'approved', ?2, NULL, 0)",
        [rule2_id.clone(), now],
    ).unwrap();

    // Create actions table for audit
    conn.execute(
        "CREATE TABLE actions (
            id TEXT PRIMARY KEY NOT NULL,
            ts TEXT NOT NULL,
            actor TEXT NOT NULL,
            kind TEXT NOT NULL,
            target TEXT NOT NULL,
            project TEXT,
            args_json TEXT,
            result TEXT NOT NULL,
            error TEXT,
            source TEXT,
            stitch_id TEXT,
            args_hash TEXT,
            hash_prev TEXT NOT NULL,
            hash_self TEXT NOT NULL
        )",
        [],
    ).unwrap();

    // Insert genesis row for hash chain
    let genesis_id = uuid::Uuid::new_v4().to_string();
    let genesis_input = format!("{}{}{}{}{}", genesis_id, now, "system", "\"genesis\"", "system_init");
    let genesis_hash = hex_encode(sha256(genesis_input.as_bytes()));
    conn.execute(
        "INSERT INTO actions (id, ts, actor, kind, target, result, hash_prev, hash_self)
         VALUES (?1, ?2, 'system', 'genesis', 'system_init', 'success', '0000000000000000000000000000000000000000000000000000000000000000', ?3)",
        [genesis_id, now, genesis_hash],
    ).unwrap();

    std::env::set_var("_HOOP_FLEET_DB_PATH", temp_dir.path().join("fleet.db"));

    // Build reflection rules with audit
    let session_id = "test-session-123";
    let turn_index = 5;
    let result = hoop_daemon::fleet::build_reflection_rules_with_audit(session_id, turn_index);

    assert!(result.is_ok(), "build_reflection_rules_with_audit should succeed");
    let rules_string = result.unwrap();
    assert!(rules_string.contains("always run tests before closing"));
    assert!(rules_string.contains("never edit fleet.db directly"));

    // Verify audit rows were written
    let audit_stmt = conn
        .prepare("SELECT id, kind, target, args_json FROM actions WHERE kind = 'reflection_injected'")
        .unwrap();

    let audit_rows: Vec<(String, String, String, String)> = audit_stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert_eq!(audit_rows.len(), 2, "Should have 2 audit rows, one per injected rule");

    // Verify each audit row has the correct structure
    for (audit_id, kind, target, args_json) in &audit_rows {
        assert_eq!(kind, r#""reflection_injected""#);

        let args: serde_json::Value = serde_json::from_str(args_json).unwrap();
        assert_eq!(args["session_id"], session_id);
        assert_eq!(args["turn_index"], turn_index);

        let rule_id = args["rule_id"].as_str().unwrap();
        assert!(rule_id == &rule1_id || rule_id == &rule2_id);

        // Verify target matches rule_id
        assert_eq!(target, rule_id);
    }

    // Verify last_applied and applied_count were updated atomically
    let ledger_stmt = conn
        .prepare("SELECT id, last_applied, applied_count FROM reflection_ledger WHERE status = 'approved'")
        .unwrap();

    let ledger_rows: Vec<(String, Option<String>, i64)> = ledger_stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, i64>(2)?))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert_eq!(ledger_rows.len(), 2);

    for (rule_id, last_applied, applied_count) in ledger_rows {
        assert!(last_applied.is_some(), "last_applied should be set");
        assert_eq!(applied_count, 1, "applied_count should be 1 after injection");
    }

    // Test atomicity: call again and verify applied_count increments
    let result2 = hoop_daemon::fleet::build_reflection_rules_with_audit(session_id, turn_index + 1);
    assert!(result2.is_ok());

    let ledger_stmt2 = conn
        .prepare("SELECT id, applied_count FROM reflection_ledger WHERE status = 'approved'")
        .unwrap();

    let ledger_rows2: Vec<(String, i64)> = ledger_stmt2
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    for (rule_id, applied_count) in ledger_rows2 {
        assert_eq!(applied_count, 2, "applied_count should be 2 after second injection");
    }

    // Verify new audit rows for second injection
    let audit_stmt2 = conn
        .prepare("SELECT COUNT(*) FROM actions WHERE kind = 'reflection_injected'")
        .unwrap();

    let count: i64 = audit_stmt2.query_row([], |row| row.get(0)).unwrap();
    assert_eq!(count, 4, "Should have 4 audit rows total (2 per injection)");

    std::env::remove_var("_HOOP_FLEET_DB_PATH");
}

/// Helper: hex_encode for SHA-256 hashes
fn hex_encode(bytes: [u8; 32]) -> String {
    use std::fmt::Write;
    let mut result = String::with_capacity(64);
    for byte in bytes {
        write!(&mut result, "{:02x}", byte).unwrap();
    }
    result
}

/// Helper: sha256 hash
fn sha256(data: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}
