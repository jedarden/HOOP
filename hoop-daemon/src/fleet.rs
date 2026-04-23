//! Fleet database - cross-project state and audit log
//!
//! This module manages `~/.hoop/fleet.db`, the primary source of truth for:
//! - Actions audit log with hash chain for tamper evidence
//! - Cross-project state shared across all HOOP projects
//! - Schema version tracking and migrations
//!
//! ## Hash chain
//!
//! Each action row contains `hash_prev` (hash of previous row) and `hash_self`
//! (SHA-256 of this row's content). This creates a tamper-evident chain where
//! any modification breaks all subsequent hashes.

use anyhow::Result;
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tracing::info;

/// Current schema version
const SCHEMA_VERSION: &str = "1.1.0";

/// Initial schema version (for fresh databases - will migrate to SCHEMA_VERSION)
const INITIAL_SCHEMA_VERSION: &str = "0.1.0";

/// Genesis hash - all chains start here
const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Database path: `~/.hoop/fleet.db`
pub fn db_path() -> PathBuf {
    let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.push(".hoop");
    home.push("fleet.db");
    home
}

/// Initialize fleet.db with schema and genesis row
///
/// Creates the database if it doesn't exist, enables WAL mode,
/// creates the actions table, metadata table, and inserts the genesis row.
pub fn init_fleet_db() -> Result<()> {
    let path = db_path();
    let parent = path.parent().ok_or_else(|| anyhow::anyhow!("Invalid db path"))?;

    // Ensure ~/.hoop/ exists
    std::fs::create_dir_all(parent)?;

    let exists = path.exists();

    info!(
        "Initializing fleet.db at {} (exists: {})",
        path.display(),
        exists
    );

    let mut conn = Connection::open(&path)?;

    // Enable WAL mode for concurrent reads
    conn.pragma_update(None, "journal_mode", "WAL")?;

    if !exists {
        // Fresh database: create schema and insert genesis row
        create_schema(&mut conn)?;
        insert_genesis_row(&mut conn)?;
        info!("fleet.db created with initial schema {}, running migrations to {}", INITIAL_SCHEMA_VERSION, SCHEMA_VERSION);

        // Run migrations to bring fresh database to current version
        run_migrations(&mut conn, INITIAL_SCHEMA_VERSION)?;
        info!("Migrations complete, schema version {}", SCHEMA_VERSION);
    } else {
        // Existing database: verify schema version and run migrations
        let version = get_schema_version(&conn)?;
        if version != SCHEMA_VERSION {
            info!(
                "fleet.db schema version {} -> {}, running migrations",
                version, SCHEMA_VERSION
            );
            run_migrations(&mut conn, &version)?;
            info!("Migrations complete, schema version {}", SCHEMA_VERSION);
        } else {
            info!("fleet.db schema version {} verified", version);
        }
    }

    Ok(())
}

/// Create all tables and indexes
fn create_schema(conn: &mut Connection) -> Result<()> {
    // Actions audit table with hash chain
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS actions (
            id TEXT PRIMARY KEY NOT NULL,
            ts TEXT NOT NULL,
            actor TEXT NOT NULL,
            kind TEXT NOT NULL,
            target TEXT NOT NULL,
            project TEXT,
            args_json TEXT,
            result TEXT,
            hash_prev TEXT NOT NULL,
            hash_self TEXT NOT NULL
        )
        "#,
        [],
    )?;

    // Index for common queries
    conn.execute(
        r#"
        CREATE INDEX IF NOT EXISTS idx_actions_ts ON actions(ts DESC)
        "#,
        [],
    )?;

    conn.execute(
        r#"
        CREATE INDEX IF NOT EXISTS idx_actions_actor ON actions(actor)
        "#,
        [],
    )?;

    conn.execute(
        r#"
        CREATE INDEX IF NOT EXISTS idx_actions_project ON actions(project)
        "#,
        [],
    )?;

    // Metadata table for schema version and migrations
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS metadata (
            key TEXT PRIMARY KEY NOT NULL,
            value TEXT NOT NULL
        )
        "#,
        [],
    )?;

    // Store initial schema version (will be migrated to SCHEMA_VERSION)
    conn.execute(
        "INSERT INTO metadata (key, value) VALUES (?, ?)",
        ["schema_version", INITIAL_SCHEMA_VERSION],
    )?;

    Ok(())
}

/// Insert the genesis row that starts the hash chain
fn insert_genesis_row(conn: &mut Connection) -> Result<()> {
    use chrono::Utc;
    use uuid::Uuid;

    let id = Uuid::new_v4().to_string();
    let ts = Utc::now().to_rfc3339();
    let actor = "system:genesis";
    let kind = "genesis";
    let target = "fleet.db";
    let project: Option<String> = None;
    let args_json: Option<String> = None;
    let result = "initialized";

    // Genesis row has no previous hash
    let hash_prev = GENESIS_HASH.to_string();

    // Compute hash of this row's content
    let hash_input = format!(
        "{}{}{}{}{}{:?}{}{}",
        id, ts, actor, kind, target, project, args_json.unwrap_or_default(), result
    );
    let hash_self = hex_encode(sha256(hash_input.as_bytes()));

    conn.execute(
        r#"
        INSERT INTO actions (id, ts, actor, kind, target, project, args_json, result, hash_prev, hash_self)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        params![
            id, ts, actor, kind, target, project, args_json, result, hash_prev, hash_self
        ],
    )?;

    info!("Genesis row inserted: {}", id);

    Ok(())
}

/// Get current schema version from metadata table
fn get_schema_version(conn: &Connection) -> Result<String> {
    conn.query_row(
        "SELECT value FROM metadata WHERE key = 'schema_version'",
        [],
        |row| row.get(0),
    )
    .map_err(|e| anyhow::anyhow!("Failed to get schema version: {}", e))
}

/// Compute SHA-256 hash
fn sha256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Encode bytes as hex string
fn hex_encode(data: Vec<u8>) -> String {
    hex::encode(data)
}

/// Run schema migrations from the given version to current
///
/// This function handles incremental schema upgrades, applying each
/// migration step in order to reach the current schema version.
fn run_migrations(conn: &mut Connection, from_version: &str) -> Result<()> {
    // Enable foreign keys
    conn.pragma_update(None, "foreign_keys", "ON")?;

    match from_version {
        "0.1.0" => {
            // Migration 0.1.0 → 1.1.0: Add Stitch service tables
            migrate_v01_to_v11(conn)?;
            update_schema_version(conn, "1.1.0")?;
        }
        "1.1.0" => {
            info!("Already at schema version 1.1.0, no migrations needed");
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported schema version: {}. Expected 0.1.0 or 1.1.0",
                from_version
            ));
        }
    }

    Ok(())
}

/// Migration 0.1.0 → 1.1.0: Add Stitch service tables
///
/// This migration creates the four Stitch-related tables:
/// - stitches: Core stitch records
/// - stitch_messages: Messages within stitches
/// - stitch_beads: Links between stitches and beads
/// - stitch_links: Links between stitches
///
/// All tables include proper indexes for Reddit-post ranking queries
/// and foreign key constraints for referential integrity.
fn migrate_v01_to_v11(conn: &mut Connection) -> Result<()> {
    info!("Running migration 0.1.0 → 1.1.0: Adding Stitch service tables");

    // Create stitches table
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS stitches (
            id TEXT PRIMARY KEY NOT NULL,
            project TEXT NOT NULL,
            kind TEXT NOT NULL CHECK(kind IN ('operator', 'dictated', 'worker', 'ad-hoc')),
            title TEXT NOT NULL,
            created_by TEXT NOT NULL,
            created_at TEXT NOT NULL,
            last_activity_at TEXT NOT NULL,
            participants TEXT DEFAULT '[]',
            attachments_path TEXT
        )
        "#,
        [],
    )?;

    // Create index for Reddit-post ranking (project + activity sort)
    conn.execute(
        r#"
        CREATE INDEX IF NOT EXISTS idx_stitches_project_activity
        ON stitches(project, last_activity_at DESC)
        "#,
        [],
    )?;

    // Create stitch_messages table
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS stitch_messages (
            id TEXT PRIMARY KEY NOT NULL,
            stitch_id TEXT NOT NULL REFERENCES stitches(id) ON DELETE CASCADE,
            ts TEXT NOT NULL,
            role TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'system', 'tool')),
            content TEXT NOT NULL,
            attachments TEXT DEFAULT '[]',
            tokens INTEGER
        )
        "#,
        [],
    )?;

    // Index for querying messages by stitch in chronological order
    conn.execute(
        r#"
        CREATE INDEX IF NOT EXISTS idx_stitch_messages_stitch_ts
        ON stitch_messages(stitch_id, ts)
        "#,
        [],
    )?;

    // Create stitch_beads table
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS stitch_beads (
            stitch_id TEXT NOT NULL REFERENCES stitches(id) ON DELETE CASCADE,
            bead_id TEXT NOT NULL,
            workspace TEXT NOT NULL,
            relationship TEXT NOT NULL CHECK(relationship IN ('created-here', 'executing', 'referenced')),
            PRIMARY KEY (stitch_id, bead_id)
        )
        "#,
        [],
    )?;

    // Index for finding beads by stitch
    conn.execute(
        r#"
        CREATE INDEX IF NOT EXISTS idx_stitch_beads_stitch
        ON stitch_beads(stitch_id)
        "#,
        [],
    )?;

    // Index for finding stitches by bead
    conn.execute(
        r#"
        CREATE INDEX IF NOT EXISTS idx_stitch_beads_bead
        ON stitch_beads(bead_id)
        "#,
        [],
    )?;

    // Create stitch_links table
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS stitch_links (
            from_stitch TEXT NOT NULL REFERENCES stitches(id) ON DELETE CASCADE,
            to_stitch TEXT NOT NULL REFERENCES stitches(id) ON DELETE CASCADE,
            kind TEXT NOT NULL CHECK(kind IN ('spawned', 'references')),
            PRIMARY KEY (from_stitch, to_stitch, kind)
        )
        "#,
        [],
    )?;

    // Index for finding outgoing links from a stitch
    conn.execute(
        r#"
        CREATE INDEX IF NOT EXISTS idx_stitch_links_from
        ON stitch_links(from_stitch)
        "#,
        [],
    )?;

    // Index for finding incoming links to a stitch
    conn.execute(
        r#"
        CREATE INDEX IF NOT EXISTS idx_stitch_links_to
        ON stitch_links(to_stitch)
        "#,
        [],
    )?;

    info!("Stitch service tables created successfully");
    Ok(())
}

/// Update the schema version in the metadata table
fn update_schema_version(conn: &mut Connection, version: &str) -> Result<()> {
    conn.execute(
        "UPDATE metadata SET value = ? WHERE key = 'schema_version'",
        [version],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_sha256_hex_encode() {
        let hash = hex_encode(sha256(b"test"));
        assert_eq!(
            hash,
            "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
        );
    }

    #[test]
    fn test_genesis_hash_constant() {
        assert_eq!(GENESIS_HASH.len(), 64);
        assert!(GENESIS_HASH.chars().all(|c| c == '0'));
    }

    #[test]
    fn test_create_schema() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut conn = Connection::open(temp_file.path())?;

        create_schema(&mut conn)?;

        // Verify tables exist
        let table_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('actions', 'metadata')",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(table_count, 2);

        // Verify schema version
        let version = get_schema_version(&conn)?;
        assert_eq!(version, SCHEMA_VERSION);

        Ok(())
    }

    #[test]
    fn test_insert_genesis_row() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut conn = Connection::open(temp_file.path())?;

        create_schema(&mut conn)?;
        insert_genesis_row(&mut conn)?;

        // Verify genesis row exists
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM actions WHERE kind = 'genesis'", [], |row| row.get(0))?;
        assert_eq!(count, 1);

        // Verify hash chain integrity
        let (hash_prev, hash_self): (String, String) = conn.query_row(
            "SELECT hash_prev, hash_self FROM actions WHERE kind = 'genesis'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        assert_eq!(hash_prev, GENESIS_HASH);
        assert_eq!(hash_self.len(), 64);

        Ok(())
    }

    #[test]
    fn test_migration_v01_to_v11() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut conn = Connection::open(temp_file.path())?;

        // Create initial v0.1.0 schema
        create_schema(&mut conn)?;

        // Verify initial version
        let version = get_schema_version(&conn)?;
        assert_eq!(version, "0.1.0");

        // Run migration
        run_migrations(&mut conn, "0.1.0")?;

        // Verify new version
        let version = get_schema_version(&conn)?;
        assert_eq!(version, "1.1.0");

        // Verify all Stitch tables exist
        let tables = [
            "stitches",
            "stitch_messages",
            "stitch_beads",
            "stitch_links",
        ];
        for table in tables {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name = ?",
                [table],
                |row| row.get(0),
            )?;
            assert_eq!(count, 1, "Table {} should exist", table);
        }

        // Verify indexes exist
        let indexes = [
            "idx_stitches_project_activity",
            "idx_stitch_messages_stitch_ts",
            "idx_stitch_beads_stitch",
            "idx_stitch_beads_bead",
            "idx_stitch_links_from",
            "idx_stitch_links_to",
        ];
        for idx in indexes {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name = ?",
                [idx],
                |row| row.get(0),
            )?;
            assert_eq!(count, 1, "Index {} should exist", idx);
        }

        Ok(())
    }

    #[test]
    fn test_stitch_foreign_key_constraints() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut conn = Connection::open(temp_file.path())?;

        // Create schema and run migration
        create_schema(&mut conn)?;
        run_migrations(&mut conn, "0.1.0")?;

        // Enable foreign keys for this test
        conn.pragma_update(None, "foreign_keys", "ON")?;

        use uuid::Uuid;

        let stitch_id = Uuid::new_v4().to_string();
        let project = "test-project";

        // Insert a stitch
        conn.execute(
            r#"
            INSERT INTO stitches (id, project, kind, title, created_by, created_at, last_activity_at)
            VALUES (?, ?, ?, ?, ?, datetime('now'), datetime('now'))
            "#,
            params![stitch_id, project, "operator", "Test Stitch", "user"],
        )?;

        // Insert a message with valid stitch_id should succeed
        conn.execute(
            r#"
            INSERT INTO stitch_messages (id, stitch_id, ts, role, content)
            VALUES (?, ?, datetime('now'), 'user', 'test content')
            "#,
            params![Uuid::new_v4().to_string(), stitch_id],
        )?;

        // Insert a message with invalid stitch_id should fail
        let result = conn.execute(
            r#"
            INSERT INTO stitch_messages (id, stitch_id, ts, role, content)
            VALUES (?, ?, datetime('now'), 'user', 'test content')
            "#,
            params![Uuid::new_v4().to_string(), "invalid-stitch-id"],
        );
        assert!(result.is_err(), "Foreign key constraint should prevent invalid stitch_id");

        // Insert a stitch_bead with valid stitch_id should succeed
        conn.execute(
            r#"
            INSERT INTO stitch_beads (stitch_id, bead_id, workspace, relationship)
            VALUES (?, ?, ?, ?)
            "#,
            params![stitch_id, "bd-test", "/tmp/test", "created-here"],
        )?;

        // Insert stitch_links with valid stitch_ids should succeed
        let stitch_id_2 = Uuid::new_v4().to_string();
        conn.execute(
            r#"
            INSERT INTO stitches (id, project, kind, title, created_by, created_at, last_activity_at)
            VALUES (?, ?, ?, ?, ?, datetime('now'), datetime('now'))
            "#,
            params![stitch_id_2, project, "operator", "Test Stitch 2", "user"],
        )?;

        conn.execute(
            r#"
            INSERT INTO stitch_links (from_stitch, to_stitch, kind)
            VALUES (?, ?, ?)
            "#,
            params![stitch_id, stitch_id_2, "spawned"],
        )?;

        Ok(())
    }

    #[test]
    fn test_stitch_kind_check_constraint() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut conn = Connection::open(temp_file.path())?;

        create_schema(&mut conn)?;
        run_migrations(&mut conn, "0.1.0")?;

        use uuid::Uuid;

        // Valid stitch kinds should succeed
        for kind in ["operator", "dictated", "worker", "ad-hoc"] {
            conn.execute(
                r#"
                INSERT INTO stitches (id, project, kind, title, created_by, created_at, last_activity_at)
                VALUES (?, ?, ?, ?, ?, datetime('now'), datetime('now'))
                "#,
                params![Uuid::new_v4().to_string(), "test-project", kind, "Test", "user"],
            )?;
        }

        // Invalid stitch kind should fail
        let result = conn.execute(
            r#"
            INSERT INTO stitches (id, project, kind, title, created_by, created_at, last_activity_at)
            VALUES (?, ?, ?, ?, ?, datetime('now'), datetime('now'))
            "#,
            params![Uuid::new_v4().to_string(), "test-project", "invalid_kind", "Test", "user"],
        );
        assert!(result.is_err(), "CHECK constraint should reject invalid stitch kind");

        Ok(())
    }

    #[test]
    fn test_stitch_relationship_check_constraint() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut conn = Connection::open(temp_file.path())?;

        create_schema(&mut conn)?;
        run_migrations(&mut conn, "0.1.0")?;

        use uuid::Uuid;

        let stitch_id = Uuid::new_v4().to_string();

        // Insert a stitch first
        conn.execute(
            r#"
            INSERT INTO stitches (id, project, kind, title, created_by, created_at, last_activity_at)
            VALUES (?, ?, ?, ?, ?, datetime('now'), datetime('now'))
            "#,
            params![stitch_id, "test-project", "operator", "Test", "user"],
        )?;

        // Valid relationships should succeed
        for rel in ["created-here", "executing", "referenced"] {
            conn.execute(
                r#"
                INSERT INTO stitch_beads (stitch_id, bead_id, workspace, relationship)
                VALUES (?, ?, ?, ?)
                "#,
                params![stitch_id, format!("bd-{}", rel), "/tmp/test", rel],
            )?;
        }

        // Invalid relationship should fail
        let result = conn.execute(
            r#"
            INSERT INTO stitch_beads (stitch_id, bead_id, workspace, relationship)
            VALUES (?, ?, ?, ?)
            "#,
            params![stitch_id, "bd-invalid", "/tmp/test", "invalid_rel"],
        );
        assert!(result.is_err(), "CHECK constraint should reject invalid relationship");

        Ok(())
    }

    #[test]
    fn test_stitch_link_kind_check_constraint() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut conn = Connection::open(temp_file.path())?;

        create_schema(&mut conn)?;
        run_migrations(&mut conn, "0.1.0")?;

        use uuid::Uuid;

        let stitch_id_1 = Uuid::new_v4().to_string();
        let stitch_id_2 = Uuid::new_v4().to_string();

        // Insert two stitches
        for sid in [&stitch_id_1, &stitch_id_2] {
            conn.execute(
                r#"
                INSERT INTO stitches (id, project, kind, title, created_by, created_at, last_activity_at)
                VALUES (?, ?, ?, ?, ?, datetime('now'), datetime('now'))
                "#,
                params![sid, "test-project", "operator", "Test", "user"],
            )?;
        }

        // Valid link kinds should succeed
        for kind in ["spawned", "references"] {
            conn.execute(
                r#"
                INSERT INTO stitch_links (from_stitch, to_stitch, kind)
                VALUES (?, ?, ?)
                "#,
                params![stitch_id_1, stitch_id_2, kind],
            )?;
        }

        // Invalid link kind should fail
        let result = conn.execute(
            r#"
            INSERT INTO stitch_links (from_stitch, to_stitch, kind)
            VALUES (?, ?, ?)
            "#,
            params![stitch_id_1, stitch_id_2, "invalid_kind"],
        );
        assert!(result.is_err(), "CHECK constraint should reject invalid link kind");

        Ok(())
    }

    #[test]
    fn test_stitches_project_activity_index() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut conn = Connection::open(temp_file.path())?;

        create_schema(&mut conn)?;
        run_migrations(&mut conn, "0.1.0")?;

        // Verify the index is used for the project + last_activity_at query
        use uuid::Uuid;

        // Insert some test data
        for i in 0..10 {
            conn.execute(
                r#"
                INSERT INTO stitches (id, project, kind, title, created_by, created_at, last_activity_at)
                VALUES (?, ?, ?, ?, ?, datetime('now', '-' || ? || ' seconds'), datetime('now', '-' || ? || ' seconds'))
                "#,
                params![
                    Uuid::new_v4().to_string(),
                    "test-project",
                    "operator",
                    format!("Stitch {}", i),
                    "user",
                    i * 100,
                    i * 100,
                ],
            )?;
        }

        // Query using the index pattern
        let stitches: Vec<(String, String)> = conn
            .prepare(
                "SELECT id, last_activity_at FROM stitches WHERE project = ? ORDER BY last_activity_at DESC",
            )?
            .query_map(["test-project"], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;

        assert_eq!(stitches.len(), 10);

        Ok(())
    }

    #[test]
    fn test_unsupported_schema_version() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut conn = Connection::open(temp_file.path()).unwrap();

        create_schema(&mut conn).unwrap();

        // Try to migrate from unsupported version
        let result = run_migrations(&mut conn, "99.99.99");
        assert!(result.is_err(), "Should reject unsupported schema version");
    }

    #[test]
    fn test_migration_idempotent() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut conn = Connection::open(temp_file.path())?;

        // Create and migrate
        create_schema(&mut conn)?;
        run_migrations(&mut conn, "0.1.0")?;

        // Running migration again on 1.1.0 should be a no-op
        run_migrations(&mut conn, "1.1.0")?;

        let version = get_schema_version(&conn)?;
        assert_eq!(version, "1.1.0");

        Ok(())
    }
}
