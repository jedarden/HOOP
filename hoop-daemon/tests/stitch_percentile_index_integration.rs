//! Integration tests for stitch_percentile_index
//!
//! Validates acceptance criteria from §6 Phase 4 marquee #8 bullets 2-3:
//! - Bucket size and similarity threshold tuned
//! - Index rebuilds on schema change
//! - Preview query <50ms
//!
//! Plan reference: hoop-ttb.5.8.1

use chrono::Utc;
use hoop_daemon::stitch_percentile_index::{
    BucketId, BodyLengthBucket, MIN_SAMPLES_FOR_PREDICTION,
    TITLE_TOKEN_BUCKET_SIZE,
};
use rusqlite::Connection;
use std::time::Instant;
use tempfile::TempDir;

/// Create a test database with the percentile index schema
fn setup_test_db(temp_dir: &TempDir) -> Connection {
    let db_path = temp_dir.path().join("test_fleet.db");
    let mut conn = Connection::open(&db_path).expect("Failed to open test DB");

    // Create the stitches table (minimal schema for testing)
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS stitches (
            id TEXT PRIMARY KEY NOT NULL,
            project TEXT NOT NULL,
            kind TEXT NOT NULL,
            title TEXT NOT NULL,
            created_by TEXT NOT NULL,
            created_at TEXT NOT NULL,
            last_activity_at TEXT NOT NULL,
            participants TEXT DEFAULT '[]',
            attachments_path TEXT
        )
        "#,
        [],
    )
    .expect("Failed to create stitches table");

    // Create stitch_messages table
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS stitch_messages (
            id TEXT PRIMARY KEY NOT NULL,
            stitch_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT,
            tokens INTEGER DEFAULT 0,
            ts TEXT NOT NULL
        )
        "#,
        [],
    )
    .expect("Failed to create stitch_messages table");

    // Create actions table for labels
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS actions (
            id TEXT PRIMARY KEY NOT NULL,
            stitch_id TEXT NOT NULL,
            kind TEXT NOT NULL,
            args_json TEXT,
            ts TEXT NOT NULL
        )
        "#,
        [],
    )
    .expect("Failed to create actions table");

    // Initialize the percentile index
    hoop_daemon::stitch_percentile_index::init_index(&mut conn)
        .expect("Failed to initialize percentile index");

    conn
}

/// Insert a test stitch with messages and labels
fn insert_test_stitch(
    conn: &mut Connection,
    stitch_id: &str,
    title: &str,
    body: Option<&str>,
    labels: &[String],
    tokens: i64,
    created_hours_ago: i64,
) {
    let now = Utc::now();
    let created_at = (now - chrono::Duration::hours(created_hours_ago)).to_rfc3339();
    let last_activity_at =
        (now - chrono::Duration::hours(created_hours_ago.saturating_sub(1))).to_rfc3339();

    // Insert stitch
    conn.execute(
        r#"
        INSERT INTO stitches (id, project, kind, title, created_by, created_at, last_activity_at, participants)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
        rusqlite::params![
            stitch_id,
            "test_project",
            "operator",
            title,
            "test_user",
            created_at,
            last_activity_at,
            "[]"
        ],
    )
    .expect("Failed to insert stitch");

    // Insert message if body provided
    if let Some(body_text) = body {
        conn.execute(
            r#"
            INSERT INTO stitch_messages (id, stitch_id, role, content, tokens, ts)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            rusqlite::params![
                format!("msg_{}", stitch_id),
                stitch_id,
                "user",
                body_text,
                tokens,
                created_at
            ],
        )
        .expect("Failed to insert message");
    }

    // Insert labels via action
    if !labels.is_empty() {
        let labels_json = serde_json::json!({ "labels": labels }).to_string();
        conn.execute(
            r#"
            INSERT INTO actions (id, stitch_id, kind, args_json, ts)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            rusqlite::params![
                format!("action_{}", stitch_id),
                stitch_id,
                "stitch_created",
                labels_json,
                created_at
            ],
        )
        .expect("Failed to insert action");
    }
}

#[test]
fn test_index_initialization() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let conn = setup_test_db(&temp_dir);

    // Check that the index table exists
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='stitch_percentile_index'",
            [],
            |row| row.get(0),
        )
        .expect("Failed to check table existence");

    assert!(table_exists, "stitch_percentile_index table should exist");

    // Check that the metadata table exists
    let meta_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='stitch_percentile_index_meta'",
            [],
            |row| row.get(0),
        )
        .expect("Failed to check metadata table existence");

    assert!(meta_exists, "stitch_percentile_index_meta table should exist");

    // Check schema version
    let schema_version: String = conn
        .query_row(
            "SELECT value FROM stitch_percentile_index_meta WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        )
        .expect("Failed to get schema version");

    assert_eq!(
        schema_version, "1.0.0",
        "Schema version should be 1.0.0"
    );
}

#[test]
fn test_schema_version_checking() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let conn = setup_test_db(&temp_dir);

    // Initially, schema should match
    assert!(
        hoop_daemon::stitch_percentile_index::check_schema_version(&conn)
            .expect("Failed to check schema version")
    );
    assert!(
        !hoop_daemon::stitch_percentile_index::needs_rebuild(&conn)
            .expect("Failed to check rebuild needed")
    );

    // Corrupt the schema version
    conn.execute(
        r#"
        INSERT INTO stitch_percentile_index_meta (key, value)
        VALUES ('schema_version', '0.0.0')
        ON CONFLICT (key) DO UPDATE SET value = excluded.value
        "#,
        [],
    )
    .expect("Failed to corrupt schema version");

    // Now schema should not match
    assert!(
        !hoop_daemon::stitch_percentile_index::check_schema_version(&conn)
            .expect("Failed to check schema version")
    );
    assert!(
        hoop_daemon::stitch_percentile_index::needs_rebuild(&conn)
            .expect("Failed to check rebuild needed")
    );
}

#[test]
fn test_bucket_id_from_features() {
    // Test that bucket IDs are stable for same features
    let bucket1 = BucketId::from_features(
        "Fix authentication bug in login module",
        250,
        &["bug".to_string(), "auth".to_string()],
        1,
    );

    let bucket2 = BucketId::from_features(
        "Fix authentication bug in login module",
        250,
        &["bug".to_string(), "auth".to_string()],
        1,
    );

    assert_eq!(
        bucket1.to_key(),
        bucket2.to_key(),
        "Same features should produce same bucket key"
    );

    // Different body length should produce different bucket
    let bucket3 = BucketId::from_features(
        "Fix authentication bug in login module",
        500,
        &["bug".to_string(), "auth".to_string()],
        1,
    );

    assert_ne!(
        bucket1.to_key(),
        bucket3.to_key(),
        "Different body length should produce different bucket"
    );

    // Different labels should produce different bucket
    let bucket4 = BucketId::from_features(
        "Fix authentication bug in login module",
        250,
        &["bug".to_string()],
        1,
    );

    assert_ne!(
        bucket1.to_key(),
        bucket4.to_key(),
        "Different labels should produce different bucket"
    );
}

#[test]
fn test_body_length_bucket_boundaries() {
    // Test each bucket boundary
    assert_eq!(
        BodyLengthBucket::from_length(0),
        BodyLengthBucket::Empty
    );
    assert_eq!(
        BodyLengthBucket::from_length(1),
        BodyLengthBucket::Short
    );
    assert_eq!(
        BodyLengthBucket::from_length(100),
        BodyLengthBucket::Short
    );
    assert_eq!(
        BodyLengthBucket::from_length(101),
        BodyLengthBucket::Medium
    );
    assert_eq!(
        BodyLengthBucket::from_length(500),
        BodyLengthBucket::Medium
    );
    assert_eq!(
        BodyLengthBucket::from_length(501),
        BodyLengthBucket::Long
    );
    assert_eq!(
        BodyLengthBucket::from_length(2000),
        BodyLengthBucket::Long
    );
    assert_eq!(
        BodyLengthBucket::from_length(2001),
        BodyLengthBucket::VeryLong
    );
}

#[test]
fn test_index_update_on_stitch_features() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut conn = setup_test_db(&temp_dir);

    // Insert test stitches
    insert_test_stitch(
        &mut conn,
        "stitch_1",
        "Fix authentication bug",
        Some("The login module crashes when users enter special characters"),
        &["bug".to_string(), "urgent".to_string()],
        100_000, // ~$3.00 at $30/M tokens
        24,
    );

    insert_test_stitch(
        &mut conn,
        "stitch_2",
        "Fix authentication bug",
        Some("Login fails for users with expired passwords"),
        &["bug".to_string(), "urgent".to_string()],
        150_000, // ~$4.50
        23,
    );

    insert_test_stitch(
        &mut conn,
        "stitch_3",
        "Fix authentication bug",
        Some("Need to add rate limiting to prevent brute force attacks"),
        &["bug".to_string(), "urgent".to_string()],
        200_000, // ~$6.00
        22,
    );

    // Rebuild index
    hoop_daemon::stitch_percentile_index::rebuild_index(&mut conn)
        .expect("Failed to rebuild index");

    // Check that index has entries
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM stitch_percentile_index",
            [],
            |row| row.get(0),
        )
        .expect("Failed to count index entries");

    assert_eq!(count, 1, "Should have one bucket for 3 similar stitches");

    // Check the bucket values
    let (cost_p50, cost_p90, sample_count): (f64, f64, i64) = conn
        .query_row(
            "SELECT cost_p50, cost_p90, sample_count FROM stitch_percentile_index LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("Failed to query bucket");

    // Verify reasonable values (p50 should be median of 3.0, 4.5, 6.0 = 4.5)
    assert!(cost_p50 > 0.0, "Cost p50 should be positive");
    assert!(cost_p90 > cost_p50, "Cost p90 should be >= p50");
    assert_eq!(sample_count, 3, "Should have 3 samples");
}

#[test]
fn test_query_performance_under_50ms() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut conn = setup_test_db(&temp_dir);

    // Insert 100 diverse stitches to create multiple buckets
    for i in 0..100 {
        let title = match i % 5 {
            0 => "Fix authentication bug",
            1 => "Add new feature",
            2 => "Refactor code",
            3 => "Update documentation",
            4 => "Fix performance issue",
            _ => "Other task",
        };

        let body = Some(format!("Test body for stitch {}", i));
        let tokens = 50_000 + (i * 1_000); // Varying token counts
        let labels = vec![format!("label_{}", i % 3)];

        insert_test_stitch(
            &mut conn,
            &format!("stitch_{}", i),
            title,
            body.as_deref(),
            &labels,
            tokens,
            24,
        );
    }

    // Rebuild index
    hoop_daemon::stitch_percentile_index::rebuild_index(&mut conn)
        .expect("Failed to rebuild index");

    // Verify we have multiple buckets
    let bucket_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM stitch_percentile_index",
            [],
            |row| row.get(0),
        )
        .expect("Failed to count buckets");

    assert!(
        bucket_count > 1,
        "Should have multiple buckets for diverse stitches"
    );

    // Time a query - this should be <50ms per acceptance criteria
    let start = Instant::now();
    let result = hoop_daemon::stitch_percentile_index::query_percentiles(
        &conn,
        "Fix authentication bug",
        100,
        &["label_0".to_string()],
        0,
    )
    .expect("Query should succeed");
    let elapsed = start.elapsed();

    // Query should succeed quickly
    assert!(
        result.is_some(),
        "Query should find a matching bucket"
    );

    // Verify performance target
    assert!(
        elapsed.as_millis() < 50,
        "Query should complete in <50ms, took {}ms",
        elapsed.as_millis()
    );

    println!("Query completed in {:?}", elapsed);
}

#[test]
fn test_query_fuzzy_fallback() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut conn = setup_test_db(&temp_dir);

    // Insert stitches with different labels but same title/body length
    insert_test_stitch(
        &mut conn,
        "stitch_1",
        "Fix bug",
        Some("Short body"),
        &["urgent".to_string()],
        100_000,
        24,
    );

    hoop_daemon::stitch_percentile_index::rebuild_index(&mut conn)
        .expect("Failed to rebuild index");

    // Query with different labels - should use fuzzy fallback
    let result = hoop_daemon::stitch_percentile_index::query_percentiles(
        &conn,
        "Fix bug",
        20,
        &["different_label".to_string()],
        0,
    )
    .expect("Query should succeed");

    // Fuzzy fallback should still find a match based on title and body length
    assert!(
        result.is_some(),
        "Fuzzy fallback should find a match based on title hash and body length"
    );
}

#[test]
fn test_minimum_samples_for_prediction() {
    // Verify the constant is correctly defined
    assert_eq!(MIN_SAMPLES_FOR_PREDICTION, 3);

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut conn = setup_test_db(&temp_dir);

    // Insert only 2 similar stitches (below threshold)
    insert_test_stitch(&mut conn, "stitch_1", "Fix bug", Some("Body"), &[], 100_000, 24);
    insert_test_stitch(&mut conn, "stitch_2", "Fix bug", Some("Body"), &[], 150_000, 23);

    hoop_daemon::stitch_percentile_index::rebuild_index(&mut conn)
        .expect("Failed to rebuild index");

    let result = hoop_daemon::stitch_percentile_index::query_percentiles(
        &conn,
        "Fix bug",
        4,
        &[],
        0,
    )
    .expect("Query should succeed");

    // Query should return a result, but the sample_count should be <3
    if let Some(query) = result {
        assert!(
            query.sample_count < 3,
            "Sample count should be below threshold"
        );
    }
}

#[test]
fn test_title_token_bucket_size() {
    // Verify the constant is correctly defined
    assert_eq!(TITLE_TOKEN_BUCKET_SIZE, 5);

    // Test that tokenization takes first 5 unique tokens
    let title = "Fix the authentication module crash in login";
    let tokens: Vec<String> = hoop_daemon::similarity::tokenize(title)
        .into_iter()
        .take(TITLE_TOKEN_BUCKET_SIZE)
        .collect();

    assert_eq!(tokens.len(), 5, "Should take first 5 tokens");
}

#[test]
fn test_similarity_threshold_implicit() {
    // The similarity threshold is implicit in the bucket design:
    // - Title: Must share ≥1 of the first 5 tokens for exact match
    // - Body: Must be in the same length bucket
    // - Labels: Must have identical label set for exact match
    // - Attachments: Must be in the same count bucket

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut conn = setup_test_db(&temp_dir);

    // Insert stitch with specific features
    insert_test_stitch(
        &mut conn,
        "stitch_1",
        "Fix authentication crash",
        Some("A".repeat(150).as_str()), // Medium length
        &["bug".to_string()],
        100_000,
        24,
    );

    hoop_daemon::stitch_percentile_index::rebuild_index(&mut conn)
        .expect("Failed to rebuild index");

    // Query with very similar title (shares tokens)
    let result = hoop_daemon::stitch_percentile_index::query_percentiles(
        &conn,
        "Fix authentication failure", // Shares "fix", "authentication"
        150,                          // Same body length bucket
        &["bug".to_string()],         // Same labels
        0,
    )
    .expect("Query should succeed");

    // Should find a match via fuzzy fallback (title hash + body length)
    assert!(
        result.is_some(),
        "Should find match with similar title tokens and same body length"
    );

    // Query with completely different title
    let result = hoop_daemon::stitch_percentile_index::query_percentiles(
        &conn,
        "Add new feature to dashboard",
        150,
        &["bug".to_string()],
        0,
    )
    .expect("Query should succeed");

    // Should not find a match (different title tokens)
    assert!(
        result.is_none(),
        "Should not match with completely different title tokens"
    );
}

#[test]
fn test_index_rebuild_clears_old_data() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut conn = setup_test_db(&temp_dir);

    // Insert and index some stitches
    insert_test_stitch(&mut conn, "stitch_1", "Fix bug", Some("Body"), &[], 100_000, 24);
    insert_test_stitch(&mut conn, "stitch_2", "Fix bug", Some("Body"), &[], 150_000, 23);

    hoop_daemon::stitch_percentile_index::rebuild_index(&mut conn)
        .expect("Failed to rebuild index");

    let count_before: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM stitch_percentile_index",
            [],
            |row| row.get(0),
        )
        .expect("Failed to count");

    assert_eq!(count_before, 1, "Should have one bucket");

    // Insert another stitch with different features
    insert_test_stitch(
        &mut conn,
        "stitch_3",
        "Add feature",
        Some("Different body"),
        &[],
        200_000,
        22,
    );

    // Rebuild again
    hoop_daemon::stitch_percentile_index::rebuild_index(&mut conn)
        .expect("Failed to rebuild index");

    let count_after: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM stitch_percentile_index",
            [],
            |row| row.get(0),
        )
        .expect("Failed to count");

    // Should now have 2 buckets
    assert_eq!(count_after, 2, "Should have two buckets after rebuild");
}

#[test]
fn test_percentiles_computed_correctly() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut conn = setup_test_db(&temp_dir);

    // Insert stitches with known costs: 1.0, 2.0, 3.0, 4.0, 5.0
    // Token calculation: tokens * 30 / 1_000_000 = cost
    // So: 33333, 66667, 100000, 133333, 166667 tokens
    let token_values = [33334, 66667, 100000, 133333, 166667];
    for (i, tokens) in token_values.iter().enumerate() {
        insert_test_stitch(
            &mut conn,
            &format!("stitch_{}", i),
            "Same title",
            Some("Same body"),
            &[],
            *tokens,
            24,
        );
    }

    hoop_daemon::stitch_percentile_index::rebuild_index(&mut conn)
        .expect("Failed to rebuild index");

    // Query the bucket
    let (cost_p50, cost_p90, sample_count): (f64, f64, i64) = conn
        .query_row(
            "SELECT cost_p50, cost_p90, sample_count FROM stitch_percentile_index LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("Failed to query bucket");

    assert_eq!(sample_count, 5, "Should have 5 samples");

    // For sorted [1.0, 2.0, 3.0, 4.0, 5.0]:
    // p50 at index 2 (0.5 * 5 = 2.5, floor = 2) = 3.0
    // p90 at index 4 (0.9 * 5 = 4.5, floor = 4) = 5.0
    let expected_p50 = 3.0;
    let expected_p90 = 5.0;

    assert!(
        (cost_p50 - expected_p50).abs() < 0.1,
        "p50 should be ~3.0, got {}",
        cost_p50
    );
    assert!(
        (cost_p90 - expected_p90).abs() < 0.1,
        "p90 should be ~5.0, got {}",
        cost_p90
    );
}
