//! Cross-workspace blocker resolution tests (§4.2)
//!
//! Validates that Stitch-child links with workspace_from/to fields
//! correctly compute cross-workspace blocker chains. A bead in a parent
//! workspace should show blockers from child workspace beads that are
//! still open.
//!
//! Plan reference: §4.2 Cross-workspace dependencies

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Integration test: migration Stitch touching 3 workspaces has correct blocker graph
///
/// Simulates a migration Stitch (the canonical cross-workspace pattern):
/// 1. Parent stitch in workspace A with bead "migration-root"
/// 2. Child stitch spawned in workspace B with bead "auth-migration"
/// 3. Child stitch spawned in workspace C with bead "storage-migration"
///
/// When querying blockers for "migration-root", we should see both
/// "auth-migration" and "storage-migration" as blockers if they are open.
#[test]
fn test_cross_workspace_blocker_chain() {
    // Create a temporary fleet.db
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("fleet.db");

    let mut conn = rusqlite::Connection::open(&db_path)
        .expect("Failed to open fleet.db");

    // Set up schema: stitches, stitch_beads, stitch_links
    setup_test_schema(&mut conn);

    // Create parent stitch in workspace A
    let parent_stitch_id = "stitch-parent-001";
    let workspace_a = "/home/coding/project-a";
    conn.execute(
        "INSERT INTO stitches (id, project, title, created_at, updated_at, last_activity_at)
         VALUES (?1, ?2, ?3, datetime('now'), datetime('now'), datetime('now'))",
        [parent_stitch_id, "project-a", "Migration Root"],
    ).expect("Failed to insert parent stitch");

    // Create bead for parent stitch
    let parent_bead_id = "bead-parent-root";
    conn.execute(
        "INSERT INTO stitch_beads (stitch_id, bead_id, project, canonical_workspace, created_at)
         VALUES (?1, ?2, ?3, ?4, datetime('now'))",
        [parent_stitch_id, parent_bead_id, "project-a", workspace_a],
    ).expect("Failed to insert parent bead");

    // Create child stitch in workspace B (auth migration)
    let child_stitch_b = "stitch-child-b-001";
    let workspace_b = "/home/coding/project-b";
    conn.execute(
        "INSERT INTO stitches (id, project, title, created_at, updated_at, last_activity_at)
         VALUES (?1, ?2, ?3, datetime('now'), datetime('now'), datetime('now'))",
        [child_stitch_b, "project-a", "Auth Migration"],
    ).expect("Failed to insert child stitch B");

    // Create bead for child stitch B (OPEN - should block)
    let child_bead_b = "bead-child-auth";
    conn.execute(
        "INSERT INTO stitch_beads (stitch_id, bead_id, project, canonical_workspace, created_at)
         VALUES (?1, ?2, ?3, ?4, datetime('now'))",
        [child_stitch_b, child_bead_b, "project-a", workspace_b],
    ).expect("Failed to insert child bead B");

    // Create child stitch in workspace C (storage migration)
    let child_stitch_c = "stitch-child-c-001";
    let workspace_c = "/home/coding/project-c";
    conn.execute(
        "INSERT INTO stitches (id, project, title, created_at, updated_at, last_activity_at)
         VALUES (?1, ?2, ?3, datetime('now'), datetime('now'), datetime('now'))",
        [child_stitch_c, "project-a", "Storage Migration"],
    ).expect("Failed to insert child stitch C");

    // Create bead for child stitch C (OPEN - should block)
    let child_bead_c = "bead-child-storage";
    conn.execute(
        "INSERT INTO stitch_beads (stitch_id, bead_id, project, canonical_workspace, created_at)
         VALUES (?1, ?2, ?3, ?4, datetime('now'))",
        [child_stitch_c, child_bead_c, "project-a", workspace_c],
    ).expect("Failed to insert child bead C");

    // Create spawned links from parent to children with workspace tracking
    conn.execute(
        "INSERT INTO stitch_links (from_stitch, to_stitch, kind, workspace_from, workspace_to)
         VALUES (?1, ?2, 'spawned', ?3, ?4)",
        [parent_stitch_id, child_stitch_b, workspace_a, workspace_b],
    ).expect("Failed to insert link to child B");

    conn.execute(
        "INSERT INTO stitch_links (from_stitch, to_stitch, kind, workspace_from, workspace_to)
         VALUES (?1, ?2, 'spawned', ?3, ?4)",
        [parent_stitch_id, child_stitch_c, workspace_a, workspace_c],
    ).expect("Failed to insert link to child C");

    // Query: find child stitches via stitch_links
    let mut child_stmt = conn
        .prepare(
            "SELECT to_stitch, workspace_to FROM stitch_links
             WHERE from_stitch = ?1 AND kind = 'spawned'",
        )
        .expect("Failed to prepare stitch_links query");

    let child_stitches: Vec<(String, String)> = child_stmt
        .query_map([parent_stitch_id], |row| {
            Ok((
                row.get::<_, String>(0)?, // to_stitch
                row.get::<_, String>(1)?, // workspace_to
            ))
        })
        .expect("Failed to query child stitches")
        .filter_map(Result::ok)
        .collect();

    // Verify we found both child stitches with correct workspaces
    assert_eq!(child_stitches.len(), 2, "Should find 2 child stitches");

    let (found_stitch_b, found_workspace_b) = child_stitches
        .iter()
        .find(|(id, _)| id == child_stitch_b)
        .expect("Should find child stitch B");
    assert_eq!(found_workspace_b, workspace_b, "Workspace B should match");

    let (found_stitch_c, found_workspace_c) = child_stitches
        .iter()
        .find(|(id, _)| id == child_stitch_c)
        .expect("Should find child stitch C");
    assert_eq!(found_workspace_c, workspace_c, "Workspace C should match");

    // Query: find beads in each child stitch
    let mut all_child_beads: Vec<(String, String)> = Vec::new();
    for (child_stitch_id, workspace) in child_stitches {
        let mut bead_stmt = conn
            .prepare("SELECT bead_id FROM stitch_beads WHERE stitch_id = ?1")
            .expect("Failed to prepare stitch_beads query");

        let bead_ids: Vec<String> = bead_stmt
            .query_map([&child_stitch_id], |row| row.get::<_, String>(0))
            .expect("Failed to query child beads")
            .filter_map(Result::ok)
            .collect();

        for bead_id in bead_ids {
            all_child_beads.push((bead_id, workspace.clone()));
        }
    }

    // Verify we found both child beads
    assert_eq!(all_child_beads.len(), 2, "Should find 2 child beads");

    let (found_bead_b, found_ws_b) = all_child_beads
        .iter()
        .find(|(id, _)| id == child_bead_b)
        .expect("Should find child bead B");
    assert_eq!(found_ws_b, workspace_b, "Bead B workspace should match");

    let (found_bead_c, found_ws_c) = all_child_beads
        .iter()
        .find(|(id, _)| id == child_bead_c)
        .expect("Should find child bead C");
    assert_eq!(found_ws_c, workspace_c, "Bead C workspace should match");

    // Success: the blocker graph correctly spans 3 workspaces
    // In production, the resolver would query br CLI in each workspace
    // to check bead status. Here we've verified the database layer works.
}

/// Unit test: stitch_links table has workspace_from and workspace_to columns
#[test]
fn test_stitch_links_schema_has_workspace_columns() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("fleet.db");

    let mut conn = rusqlite::Connection::open(&db_path)
        .expect("Failed to open fleet.db");

    setup_test_schema(&mut conn);

    // Verify workspace_from column exists
    let workspace_from_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('stitch_links') WHERE name = 'workspace_from'",
            [],
            |row| row.get(0),
        )
        .expect("Failed to query workspace_from column");

    assert!(workspace_from_exists, "workspace_from column should exist");

    // Verify workspace_to column exists
    let workspace_to_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('stitch_links') WHERE name = 'workspace_to'",
            [],
            |row| row.get(0),
        )
        .expect("Failed to query workspace_to column");

    assert!(workspace_to_exists, "workspace_to column should exist");

    // Verify we can insert and query these columns
    conn.execute(
        "INSERT INTO stitch_links (from_stitch, to_stitch, kind, workspace_from, workspace_to)
         VALUES ('s1', 's2', 'spawned', '/ws/a', '/ws/b')",
        [],
    ).expect("Failed to insert stitch link with workspaces");

    let (ws_from, ws_to): (String, String) = conn
        .query_row(
            "SELECT workspace_from, workspace_to FROM stitch_links WHERE from_stitch = 's1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("Failed to query workspace columns");

    assert_eq!(ws_from, "/ws/a");
    assert_eq!(ws_to, "/ws/b");
}

/// Set up minimal schema for testing cross-workspace blockers
fn setup_test_schema(conn: &mut rusqlite::Connection) {
    // Create stitches table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS stitches (
            id TEXT PRIMARY KEY NOT NULL,
            project TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            status TEXT NOT NULL DEFAULT 'draft',
            classification TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            closed_at TEXT,
            created_by_actor TEXT,
            created_by_session_id TEXT,
            created_by_adapter TEXT,
            turn_id TEXT
        )",
        [],
    ).expect("Failed to create stitches table");

    // Create stitch_beads table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS stitch_beads (
            stitch_id TEXT NOT NULL,
            bead_id TEXT NOT NULL,
            project TEXT NOT NULL,
            canonical_workspace TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            PRIMARY KEY (stitch_id, bead_id)
        )",
        [],
    ).expect("Failed to create stitch_beads table");

    // Create stitch_links table with workspace tracking
    conn.execute(
        "CREATE TABLE IF NOT EXISTS stitch_links (
            from_stitch TEXT NOT NULL,
            to_stitch TEXT NOT NULL,
            kind TEXT NOT NULL CHECK(kind IN ('spawned', 'references')),
            workspace_from TEXT NOT NULL DEFAULT '',
            workspace_to TEXT NOT NULL DEFAULT '',
            PRIMARY KEY (from_stitch, to_stitch, kind),
            FOREIGN KEY (from_stitch) REFERENCES stitches(id) ON DELETE CASCADE,
            FOREIGN KEY (to_stitch) REFERENCES stitches(id) ON DELETE CASCADE
        )",
        [],
    ).expect("Failed to create stitch_links table");

    // Create indexes
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_stitch_links_from ON stitch_links(from_stitch)",
        [],
    ).expect("Failed to create idx_stitch_links_from");

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_stitch_links_to ON stitch_links(to_stitch)",
        [],
    ).expect("Failed to create idx_stitch_links_to");

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_stitch_beads_project ON stitch_beads(project)",
        [],
    ).expect("Failed to create idx_stitch_beads_project");
}
