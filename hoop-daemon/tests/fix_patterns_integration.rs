//! Integration test: fix_patterns CRUD and signature matching.
//!
//! Verifies:
//! 1. Create, read, update, delete operations
//! 2. Pattern matching by signature vector
//! 3. Keyword search functionality

use tempfile::TempDir;

#[test]
fn fix_pattern_crud() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("fleet.db");

    // Initialize the database with fix_patterns table
    let mut conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.pragma_update(None, "journal_mode", "WAL").unwrap();

    // Create the fix_patterns table directly (copying migration logic)
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS fix_patterns (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            signature_vector_json TEXT NOT NULL,
            keywords TEXT NOT NULL,
            recommended_fix_template_md TEXT NOT NULL,
            example_source_stitches_json TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            applied_count INTEGER NOT NULL DEFAULT 0
        )
        "#,
        [],
    )
    .unwrap();

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_fix_patterns_keywords ON fix_patterns(keywords)",
        [],
    )
    .unwrap();

    // Set environment variable so FixPatternService finds our test DB
    std::env::set_var("_HOOP_FLEET_DB_PATH", db_path.to_str().unwrap());

    // Test CREATE
    let create_req = hoop_daemon::fix_patterns::CreatePatternRequest {
        name: "Unwrap Option".to_string(),
        signature_vector: vec![0.8, 0.2, 0.0, 0.5],
        keywords: "unwrap,option,panic,null".to_string(),
        recommended_fix_template_md: "Use `unwrap_or()` or pattern matching instead.".to_string(),
        example_source_stitches: vec!["stitch-1".to_string(), "stitch-2".to_string()],
    };

    let id = hoop_daemon::fix_patterns::FixPatternService::create(&create_req).unwrap();
    assert!(!id.is_empty(), "create should return non-empty ID");

    // Test GET
    let pattern = hoop_daemon::fix_patterns::FixPatternService::get(&id)
        .unwrap()
        .expect("pattern should exist");
    assert_eq!(pattern.name, "Unwrap Option");
    assert_eq!(pattern.keywords, "unwrap,option,panic,null");
    assert_eq!(pattern.applied_count, 0);

    // Test LIST
    let patterns = hoop_daemon::fix_patterns::FixPatternService::list().unwrap();
    assert_eq!(patterns.len(), 1, "should have 1 pattern");
    assert_eq!(patterns[0].id, id);

    // Test UPDATE
    let update_req = hoop_daemon::fix_patterns::UpdatePatternRequest {
        id: id.clone(),
        name: Some("Unwrap Option (Fixed)".to_string()),
        signature_vector: None,
        keywords: Some("unwrap,option,pattern-matching".to_string()),
        recommended_fix_template_md: None,
        example_source_stitches: None,
    };

    hoop_daemon::fix_patterns::FixPatternService::update(&update_req).unwrap();

    let updated = hoop_daemon::fix_patterns::FixPatternService::get(&id)
        .unwrap()
        .expect("pattern should exist after update");
    assert_eq!(updated.name, "Unwrap Option (Fixed)");
    assert_eq!(updated.keywords, "unwrap,option,pattern-matching");
    // Signature should remain unchanged
    assert_eq!(updated.signature_vector, vec![0.8, 0.2, 0.0, 0.5]);

    // Test record_application
    hoop_daemon::fix_patterns::FixPatternService::record_application(&id).unwrap();
    let applied = hoop_daemon::fix_patterns::FixPatternService::get(&id)
        .unwrap()
        .expect("pattern should exist");
    assert_eq!(applied.applied_count, 1);

    // Test DELETE
    hoop_daemon::fix_patterns::FixPatternService::delete(&id).unwrap();
    let deleted = hoop_daemon::fix_patterns::FixPatternService::get(&id).unwrap();
    assert!(deleted.is_none(), "pattern should be deleted");

    std::env::remove_var("_HOOP_FLEET_DB_PATH");
}

#[test]
fn fix_pattern_signature_matching() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("fleet.db");

    let mut conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.pragma_update(None, "journal_mode", "WAL").unwrap();

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS fix_patterns (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            signature_vector_json TEXT NOT NULL,
            keywords TEXT NOT NULL,
            recommended_fix_template_md TEXT NOT NULL,
            example_source_stitches_json TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            applied_count INTEGER NOT NULL DEFAULT 0
        )
        "#,
        [],
    )
    .unwrap();

    std::env::set_var("_HOOP_FLEET_DB_PATH", db_path.to_str().unwrap());

    // Create patterns with different signature vectors
    let patterns = vec![
        hoop_daemon::fix_patterns::CreatePatternRequest {
            name: "Exact Match Pattern".to_string(),
            signature_vector: vec![1.0, 0.5, 0.2],
            keywords: "exact,match".to_string(),
            recommended_fix_template_md: "Fix for exact match".to_string(),
            example_source_stitches: vec!["stitch-a".to_string()],
        },
        hoop_daemon::fix_patterns::CreatePatternRequest {
            name: "Similar Pattern".to_string(),
            signature_vector: vec![0.9, 0.45, 0.18], // Scaled version of [1.0, 0.5, 0.2]
            keywords: "similar,scaled".to_string(),
            recommended_fix_template_md: "Fix for similar pattern".to_string(),
            example_source_stitches: vec!["stitch-b".to_string()],
        },
        hoop_daemon::fix_patterns::CreatePatternRequest {
            name: "Different Pattern".to_string(),
            signature_vector: vec![0.1, 0.9, 0.3],
            keywords: "different,orthogonal".to_string(),
            recommended_fix_template_md: "Fix for different pattern".to_string(),
            example_source_stitches: vec!["stitch-c".to_string()],
        },
    ];

    for req in &patterns {
        hoop_daemon::fix_patterns::FixPatternService::create(req).unwrap();
    }

    // Test matching: Query vector should match "Exact Match Pattern" perfectly
    let query = vec![1.0, 0.5, 0.2];
    let matches = hoop_daemon::fix_patterns::FixPatternService::match_by_signature(
        &query,
        0.5, // threshold
        10,  // limit
    )
    .unwrap();

    assert_eq!(matches.len(), 3, "should match all 3 patterns above threshold 0.5");

    // Top 2 matches should be the exact match and scaled version (both have similarity 1.0)
    let names: Vec<_> = matches.iter().map(|m| &m.pattern.name).collect();
    assert!(names.contains(&&"Exact Match Pattern".to_string()));
    assert!(names.contains(&&"Similar Pattern".to_string()));

    // First two matches should both have similarity ~1.0 (order not deterministic)
    assert!(matches[0].similarity > 0.99, "first match similarity should be > 0.99");
    assert!(matches[1].similarity > 0.99, "second match similarity should be > 0.99");

    // Third match is "Different Pattern" - still has similarity > 0.5
    assert_eq!(matches[2].pattern.name, "Different Pattern");
    assert!(matches[2].similarity > 0.5, "different pattern should have similarity > 0.5");

    // Test with higher threshold - only the highly similar patterns match
    let matches_strict = hoop_daemon::fix_patterns::FixPatternService::match_by_signature(
        &query,
        0.99, // higher threshold
        10,
    )
    .unwrap();

    assert_eq!(matches_strict.len(), 2, "should match 2 patterns with threshold 0.99");

    // Test with lower threshold - should include the different pattern too
    let matches_loose = hoop_daemon::fix_patterns::FixPatternService::match_by_signature(
        &query,
        0.0, // no threshold
        10,
    )
    .unwrap();

    assert_eq!(matches_loose.len(), 3, "should match all patterns with zero threshold");

    // Test limit
    let matches_limited = hoop_daemon::fix_patterns::FixPatternService::match_by_signature(
        &query,
        0.0,
        2, // limit to 2
    )
    .unwrap();

    assert_eq!(matches_limited.len(), 2, "should limit results");

    std::env::remove_var("_HOOP_FLEET_DB_PATH");
}

#[test]
fn fix_pattern_keyword_search() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("fleet.db");

    let mut conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.pragma_update(None, "journal_mode", "WAL").unwrap();

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS fix_patterns (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            signature_vector_json TEXT NOT NULL,
            keywords TEXT NOT NULL,
            recommended_fix_template_md TEXT NOT NULL,
            example_source_stitches_json TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            applied_count INTEGER NOT NULL DEFAULT 0
        )
        "#,
        [],
    )
    .unwrap();

    std::env::set_var("_HOOP_FLEET_DB_PATH", db_path.to_str().unwrap());

    let patterns = vec![
        hoop_daemon::fix_patterns::CreatePatternRequest {
            name: "Null Pointer Pattern".to_string(),
            signature_vector: vec![1.0],
            keywords: "null,pointer,dereference,panic".to_string(),
            recommended_fix_template_md: "Fix null pointer".to_string(),
            example_source_stitches: vec![],
        },
        hoop_daemon::fix_patterns::CreatePatternRequest {
            name: "Index Out of Bounds".to_string(),
            signature_vector: vec![0.5],
            keywords: "index,bounds,panic,array".to_string(),
            recommended_fix_template_md: "Fix bounds check".to_string(),
            example_source_stitches: vec![],
        },
        hoop_daemon::fix_patterns::CreatePatternRequest {
            name: "Type Mismatch".to_string(),
            signature_vector: vec![0.3],
            keywords: "type,mismatch,conversion".to_string(),
            recommended_fix_template_md: "Fix type conversion".to_string(),
            example_source_stitches: vec![],
        },
    ];

    for req in &patterns {
        hoop_daemon::fix_patterns::FixPatternService::create(req).unwrap();
    }

    // Test keyword search
    let results = hoop_daemon::fix_patterns::FixPatternService::search_by_keywords("panic")
        .unwrap();

    assert_eq!(results.len(), 2, "should find 2 patterns with 'panic'");
    let names: Vec<_> = results.iter().map(|p| &p.name).collect();
    assert!(names.contains(&&"Null Pointer Pattern".to_string()));
    assert!(names.contains(&&"Index Out of Bounds".to_string()));

    // Test name search
    let results = hoop_daemon::fix_patterns::FixPatternService::search_by_keywords("bounds")
        .unwrap();

    assert_eq!(results.len(), 1, "should find 1 pattern with 'bounds'");
    assert_eq!(results[0].name, "Index Out of Bounds");

    // Test case-insensitive search
    let results = hoop_daemon::fix_patterns::FixPatternService::search_by_keywords("TYPE")
        .unwrap();

    assert_eq!(results.len(), 1, "case-insensitive search should work");
    assert_eq!(results[0].name, "Type Mismatch");

    std::env::remove_var("_HOOP_FLEET_DB_PATH");
}

#[test]
fn fix_pattern_cosine_similarity_edge_cases() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("fleet.db");

    let mut conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.pragma_update(None, "journal_mode", "WAL").unwrap();

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS fix_patterns (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            signature_vector_json TEXT NOT NULL,
            keywords TEXT NOT NULL,
            recommended_fix_template_md TEXT NOT NULL,
            example_source_stitches_json TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            applied_count INTEGER NOT NULL DEFAULT 0
        )
        "#,
        [],
    )
    .unwrap();

    std::env::set_var("_HOOP_FLEET_DB_PATH", db_path.to_str().unwrap());

    // Create pattern with orthogonal vectors
    hoop_daemon::fix_patterns::FixPatternService::create(
        &hoop_daemon::fix_patterns::CreatePatternRequest {
            name: "Orthogonal Pattern".to_string(),
            signature_vector: vec![1.0, 0.0],
            keywords: "orthogonal".to_string(),
            recommended_fix_template_md: "Fix".to_string(),
            example_source_stitches: vec![],
        },
    )
    .unwrap();

    // Query with orthogonal vector should have low similarity
    let matches = hoop_daemon::fix_patterns::FixPatternService::match_by_signature(
        &[0.0, 1.0], // orthogonal to [1.0, 0.0]
        0.1,         // low threshold
        10,
    )
    .unwrap();

    // Orthogonal vectors have cosine similarity of 0.0
    if !matches.is_empty() {
        assert!(matches[0].similarity < 0.01, "orthogonal vectors should have near-zero similarity");
    }

    std::env::remove_var("_HOOP_FLEET_DB_PATH");
}
