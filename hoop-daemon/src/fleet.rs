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
const SCHEMA_VERSION: &str = "0.1.0";

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
        info!("fleet.db created with schema {}", SCHEMA_VERSION);
    } else {
        // Existing database: verify schema version
        let version = get_schema_version(&conn)?;
        if version != SCHEMA_VERSION {
            // TODO: run migrations
            info!(
                "fleet.db schema version {} (current: {})",
                version, SCHEMA_VERSION
            );
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

    // Store schema version
    conn.execute(
        "INSERT INTO metadata (key, value) VALUES (?, ?)",
        ["schema_version", SCHEMA_VERSION],
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

/// Migration framework (stubbed for v0.1)
///
/// In future versions, this will handle schema migrations.
/// For now, it's a placeholder that logs the current version.
#[allow(dead_code)]
fn run_migrations(conn: &mut Connection) -> Result<()> {
    // TODO: implement migrations when schema changes are needed
    info!("No migrations to run for schema {}", SCHEMA_VERSION);
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
}
