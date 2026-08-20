//! Integration test: pattern query evaluator
//!
//! Verifies:
//! 1. Query parsing and evaluation
//! 2. Idempotent inserts into pattern_members
//! 3. Multiple pattern matches
//! 4. Slow query logging
//! 5. Full integration with stitch creation

use tempfile::TempDir;
use uuid::Uuid;

#[test]
fn pattern_query_basic_evaluation() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("fleet.db");

    // Initialize the database with required tables
    let mut conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.pragma_update(None, "journal_mode", "WAL").unwrap();

    // Create patterns table
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS patterns (
            id TEXT PRIMARY KEY NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            status TEXT NOT NULL DEFAULT 'active',
            owner TEXT,
            deadline TEXT,
            parent_pattern TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT
        )
        "#,
        [],
    )
    .unwrap();

    // Create pattern_queries table
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS pattern_queries (
            id TEXT PRIMARY KEY NOT NULL,
            pattern_id TEXT NOT NULL,
            saved_query TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (pattern_id) REFERENCES patterns(id) ON DELETE CASCADE
        )
        "#,
        [],
    )
    .unwrap();

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_pattern_queries_pattern ON pattern_queries(pattern_id)",
        [],
    )
    .unwrap();

    // Create pattern_members table
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS pattern_members (
            pattern_id TEXT NOT NULL,
            stitch_id TEXT NOT NULL,
            added_at TEXT NOT NULL DEFAULT (datetime('now')),
            PRIMARY KEY (pattern_id, stitch_id),
            FOREIGN KEY (pattern_id) REFERENCES patterns(id) ON DELETE CASCADE,
            FOREIGN KEY (stitch_id) REFERENCES stitches(id) ON DELETE CASCADE
        )
        "#,
        [],
    )
    .unwrap();

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_pattern_members_pattern ON pattern_members(pattern_id)",
        [],
    )
    .unwrap();

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_pattern_members_stitch ON pattern_members(stitch_id)",
        [],
    )
    .unwrap();

    // Create stitches table
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
            classification TEXT,
            created_by_actor TEXT,
            created_by_session_id TEXT,
            created_by_adapter TEXT,
            created_by_model TEXT,
            turn_id TEXT
        )
        "#,
        [],
    )
    .unwrap();

    // Create stitch_beads table for label lookup
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS stitch_beads (
            stitch_id TEXT NOT NULL,
            bead_id TEXT NOT NULL,
            workspace TEXT NOT NULL,
            canonical_workspace TEXT NOT NULL,
            relationship TEXT NOT NULL,
            PRIMARY KEY (stitch_id, bead_id)
        )
        "#,
        [],
    )
    .unwrap();

    // Set environment variable so the evaluator finds our test DB
    std::env::set_var("_HOOP_FLEET_DB_PATH", db_path.to_str().unwrap());

    // Create a test pattern with a query
    let pattern_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO patterns (id, title, status) VALUES (?1, ?2, ?3)",
        [&pattern_id, "Ugent Fixes", "active"],
    )
    .unwrap();

    let query_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO pattern_queries (id, pattern_id, saved_query) VALUES (?1, ?2, ?3)",
        [&query_id, &pattern_id, "title:fix.*urgent"],
    )
    .unwrap();

    // Create a test stitch
    let stitch_id = Uuid::new_v4().to_string();
    conn.execute(
        r#"
        INSERT INTO stitches (id, project, kind, title, created_by, created_at, last_activity_at)
        VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'), datetime('now'))
        "#,
        [&stitch_id, "HOOP", "operator", "fix memory leak urgent"],
    )
    .unwrap();

    // Evaluate pattern queries
    let results = hoop_daemon::pattern_query_evaluator::evaluate_pattern_queries(
        &hoop_daemon::pattern_query_evaluator::StitchContext {
            stitch_id: stitch_id.clone(),
            project: "HOOP".to_string(),
            kind: "operator".to_string(),
            title: "fix memory leak urgent".to_string(),
            labels: vec![],
        },
    )
    .unwrap();

    assert_eq!(results.len(), 1, "should have 1 pattern query result");
    assert_eq!(results[0].pattern_id, pattern_id);
    assert!(results[0].matched, "query should match the stitch title");
    assert!(!results[0].is_slow, "query should not be slow");

    // Insert into pattern_members
    let inserted =
        hoop_daemon::pattern_query_evaluator::insert_pattern_member(&pattern_id, &stitch_id)
            .unwrap();
    assert!(inserted, "first insert should succeed");

    // Verify idempotency - second insert should return false
    let inserted_again =
        hoop_daemon::pattern_query_evaluator::insert_pattern_member(&pattern_id, &stitch_id)
            .unwrap();
    assert!(
        !inserted_again,
        "second insert should return false (idempotent)"
    );

    // Verify the member was added
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pattern_members WHERE pattern_id = ?1 AND stitch_id = ?2",
            [&pattern_id, &stitch_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1, "should have exactly 1 pattern member");

    std::env::remove_var("_HOOP_FLEET_DB_PATH");
}

#[test]
fn pattern_query_multiple_patterns() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("fleet.db");

    let mut conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.pragma_update(None, "journal_mode", "WAL").unwrap();

    // Create all required tables
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS patterns (
            id TEXT PRIMARY KEY NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            status TEXT NOT NULL DEFAULT 'active',
            owner TEXT,
            deadline TEXT,
            parent_pattern TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT
        )
        "#,
        [],
    )
    .unwrap();

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS pattern_queries (
            id TEXT PRIMARY KEY NOT NULL,
            pattern_id TEXT NOT NULL,
            saved_query TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (pattern_id) REFERENCES patterns(id) ON DELETE CASCADE
        )
        "#,
        [],
    )
    .unwrap();

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
            classification TEXT,
            created_by_actor TEXT,
            created_by_session_id TEXT,
            created_by_adapter TEXT,
            created_by_model TEXT,
            turn_id TEXT
        )
        "#,
        [],
    )
    .unwrap();

    std::env::set_var("_HOOP_FLEET_DB_PATH", db_path.to_str().unwrap());

    // Create multiple patterns with different queries
    let pattern1_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO patterns (id, title, status) VALUES (?1, ?2, ?3)",
        [&pattern1_id, "HOOP Tasks", "active"],
    )
    .unwrap();
    let query1_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO pattern_queries (id, pattern_id, saved_query) VALUES (?1, ?2, ?3)",
        [&query1_id, &pattern1_id, "project:HOOP"],
    )
    .unwrap();

    let pattern2_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO patterns (id, title, status) VALUES (?1, ?2, ?3)",
        [&pattern2_id, "Urgent Fixes", "active"],
    )
    .unwrap();
    let query2_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO pattern_queries (id, pattern_id, saved_query) VALUES (?1, ?2, ?3)",
        [&query2_id, &pattern2_id, "title:fix.*urgent"],
    )
    .unwrap();

    let pattern3_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO patterns (id, title, status) VALUES (?1, ?2, ?3)",
        [&pattern3_id, "Non-matching", "active"],
    )
    .unwrap();
    let query3_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO pattern_queries (id, pattern_id, saved_query) VALUES (?1, ?2, ?3)",
        [&query3_id, &pattern3_id, "title:feature.*"],
    )
    .unwrap();

    // Create a test stitch that matches pattern1 and pattern2
    let stitch_id = Uuid::new_v4().to_string();
    conn.execute(
        r#"
        INSERT INTO stitches (id, project, kind, title, created_by, created_at, last_activity_at)
        VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'), datetime('now'))
        "#,
        [&stitch_id, "HOOP", "operator", "fix urgent bug in auth"],
    )
    .unwrap();

    // Evaluate pattern queries
    let results = hoop_daemon::pattern_query_evaluator::evaluate_pattern_queries(
        &hoop_daemon::pattern_query_evaluator::StitchContext {
            stitch_id: stitch_id.clone(),
            project: "HOOP".to_string(),
            kind: "operator".to_string(),
            title: "fix urgent bug in auth".to_string(),
            labels: vec![],
        },
    )
    .unwrap();

    assert_eq!(results.len(), 3, "should have 3 pattern query results");

    // Find matches
    let matched: Vec<_> = results.iter().filter(|r| r.matched).collect();
    assert_eq!(matched.len(), 2, "should match 2 patterns");

    let matched_ids: Vec<_> = matched.iter().map(|r| &r.pattern_id).collect();
    assert!(matched_ids.contains(&&pattern1_id));
    assert!(matched_ids.contains(&&pattern2_id));
    assert!(!matched_ids.contains(&&pattern3_id));

    std::env::remove_var("_HOOP_FLEET_DB_PATH");
}

#[test]
fn pattern_query_complex_expressions() {
    // Test AND, OR, and NOT operators
    let ctx = hoop_daemon::pattern_query_evaluator::StitchContext {
        stitch_id: "test-stitch".to_string(),
        project: "HOOP".to_string(),
        kind: "operator".to_string(),
        title: "fix urgent memory leak".to_string(),
        labels: vec!["bug".to_string(), "p0".to_string()],
    };

    // Test that parsing works for complex queries
    let queries = vec![
        "title:fix.*urgent AND label:bug",
        "label:p0 OR label:p1",
        "kind:operator AND (label:urgent OR label:p0)",
        "project:HOOP AND NOT label:enhancement",
    ];

    for query in queries {
        let result = hoop_daemon::pattern_query_evaluator::parse_query(query);
        assert!(
            result.is_ok(),
            "should parse query '{}': {:?}",
            query,
            result.err()
        );
    }

    // Verify AND query matches
    let and_expr =
        hoop_daemon::pattern_query_evaluator::parse_query("title:fix.*urgent AND label:bug")
            .unwrap();
    let matches = hoop_daemon::pattern_query_evaluator::evaluate_query(&and_expr, &ctx).unwrap();
    assert!(matches, "AND query should match");

    // Verify NOT query matches
    let not_expr =
        hoop_daemon::pattern_query_evaluator::parse_query("project:HOOP AND NOT label:enhancement")
            .unwrap();
    let matches = hoop_daemon::pattern_query_evaluator::evaluate_query(&not_expr, &ctx).unwrap();
    assert!(matches, "NOT query should match");

    // Verify OR query matches
    let or_expr =
        hoop_daemon::pattern_query_evaluator::parse_query("label:p0 OR label:p1").unwrap();
    let matches = hoop_daemon::pattern_query_evaluator::evaluate_query(&or_expr, &ctx).unwrap();
    assert!(matches, "OR query should match");

    // Verify query that doesn't match
    let non_match_expr =
        hoop_daemon::pattern_query_evaluator::parse_query("title:feature.*").unwrap();
    let matches =
        hoop_daemon::pattern_query_evaluator::evaluate_query(&non_match_expr, &ctx).unwrap();
    assert!(!matches, "non-matching query should not match");
}

#[test]
fn pattern_query_kind_filter() {
    let ctx_operator = hoop_daemon::pattern_query_evaluator::StitchContext {
        stitch_id: "test-stitch-1".to_string(),
        project: "HOOP".to_string(),
        kind: "operator".to_string(),
        title: "some task".to_string(),
        labels: vec![],
    };

    let ctx_worker = hoop_daemon::pattern_query_evaluator::StitchContext {
        stitch_id: "test-stitch-2".to_string(),
        project: "HOOP".to_string(),
        kind: "worker".to_string(),
        title: "some task".to_string(),
        labels: vec![],
    };

    let expr = hoop_daemon::pattern_query_evaluator::parse_query("kind:operator").unwrap();

    let matches_operator =
        hoop_daemon::pattern_query_evaluator::evaluate_query(&expr, &ctx_operator).unwrap();
    assert!(
        matches_operator,
        "kind:operator should match operator stitch"
    );

    let matches_worker =
        hoop_daemon::pattern_query_evaluator::evaluate_query(&expr, &ctx_worker).unwrap();
    assert!(
        !matches_worker,
        "kind:operator should not match worker stitch"
    );
}

#[test]
fn pattern_query_standalone_word_as_label() {
    let ctx = hoop_daemon::pattern_query_evaluator::StitchContext {
        stitch_id: "test-stitch".to_string(),
        project: "HOOP".to_string(),
        kind: "operator".to_string(),
        title: "some task".to_string(),
        labels: vec!["urgent".to_string()],
    };

    // Standalone word should be treated as label filter
    let expr = hoop_daemon::pattern_query_evaluator::parse_query("urgent").unwrap();
    let matches = hoop_daemon::pattern_query_evaluator::evaluate_query(&expr, &ctx).unwrap();
    assert!(matches, "standalone word should match as label");

    // Non-matching standalone word
    let expr = hoop_daemon::pattern_query_evaluator::parse_query("p0").unwrap();
    let matches = hoop_daemon::pattern_query_evaluator::evaluate_query(&expr, &ctx).unwrap();
    assert!(!matches, "non-matching standalone word should not match");
}
