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
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tracing::{info, warn};
use uuid::Uuid;

/// Current schema version
pub const SCHEMA_VERSION: &str = "1.12.0";

/// Initial schema version (for fresh databases - will migrate to SCHEMA_VERSION)
const INITIAL_SCHEMA_VERSION: &str = "0.1.0";

/// Genesis hash - all chains start here
const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Action kind for audit log
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    BeadCreated,
    StitchCreated,
    ConfigChanged,
    ProjectAdded,
    ProjectRemoved,
    DraftCreated,
    DraftApproved,
    DraftEdited,
    DraftRejected,
}

/// Action result for audit log
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionResult {
    Success,
    Failure,
    Partial,
}

/// Source of a bead creation action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BeadSource {
    Form,
    Chat,
    Bulk,
    Template,
}

/// Arguments for a bead creation action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeadActionArgs {
    pub source: BeadSource,
    pub stitch_id: Option<String>,
    pub title: String,
    pub issue_type: String,
    pub priority: Option<i64>,
    pub dependencies: Vec<String>,
    pub labels: Vec<String>,
}

impl BeadActionArgs {
    /// Compute hash of args for integrity verification
    pub fn args_hash(&self) -> String {
        let json = serde_json::to_string(self).unwrap_or_default();
        hex_encode(sha256(json.as_bytes()))
    }
}

/// Audit row for the actions table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRow {
    pub id: String,
    pub ts: String,
    pub actor: String,
    pub kind: ActionKind,
    pub target: String,
    pub project: Option<String>,
    pub args_json: Option<String>,
    pub result: ActionResult,
    pub error: Option<String>,
    pub source: Option<String>,
    pub stitch_id: Option<String>,
    pub args_hash: Option<String>,
    pub hash_prev: String,
    pub hash_self: String,
}

/// Write an audit row to the actions table
///
/// This function maintains the hash chain by:
/// 1. Fetching the most recent row's hash_self as hash_prev
/// 2. Computing hash_self from the row content
/// 3. Inserting the new row
#[allow(clippy::too_many_arguments)]
pub fn write_audit_row(
    actor: &str,
    kind: ActionKind,
    target: &str,
    project: Option<&str>,
    args_json: Option<String>,
    result: ActionResult,
    error: Option<String>,
    source: Option<&str>,
    stitch_id: Option<&str>,
    args_hash: Option<&str>,
) -> Result<AuditRow> {
    let id = Uuid::new_v4().to_string();
    let ts = Utc::now().to_rfc3339();
    let kind_str = serde_json::to_string(&kind)?;
    let result_str = serde_json::to_string(&result)?;

    // Get the previous hash (hash of the last row in the chain)
    let path = db_path();
    let conn = Connection::open(&path)?;

    let hash_prev: String = conn.query_row(
        "SELECT hash_self FROM actions ORDER BY rowid DESC LIMIT 1",
        [],
        |row| row.get(0),
    ).unwrap_or_else(|_| GENESIS_HASH.to_string());

    // Compute hash of this row's content
    let hash_input = format!(
        "{}{}{}{}{}{:?}{}",
        id, ts, actor, kind_str, target, project, args_json.as_deref().unwrap_or_default()
    );
    let hash_self = hex_encode(sha256(hash_input.as_bytes()));

    // Insert the audit row
    conn.execute(
        r#"
        INSERT INTO actions (id, ts, actor, kind, target, project, args_json, result, error, source, stitch_id, args_hash, hash_prev, hash_self)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        params![
            id, ts, actor, kind_str, target, project, args_json, result_str,
            error, source, stitch_id, args_hash, hash_prev, hash_self
        ],
    )?;

    // Update audit append rate metric
    crate::metrics::metrics().hoop_audit_append_rate_per_second.inc();

    Ok(AuditRow {
        id,
        ts,
        actor: actor.to_string(),
        kind,
        target: target.to_string(),
        project: project.map(|s| s.to_string()),
        args_json,
        result,
        error,
        source: source.map(|s| s.to_string()),
        stitch_id: stitch_id.map(|s| s.to_string()),
        args_hash: args_hash.map(|s| s.to_string()),
        hash_prev,
        hash_self,
    })
}

/// Query audit rows with optional filters
pub fn query_audit_rows(
    limit: Option<usize>,
    offset: Option<usize>,
    project_filter: Option<&str>,
    kind_filter: Option<ActionKind>,
) -> Result<Vec<AuditRow>> {
    let path = db_path();
    let conn = Connection::open(&path)?;

    let mut query = String::from(
        "SELECT id, ts, actor, kind, target, project, args_json, result, error, source, stitch_id, args_hash, hash_prev, hash_self FROM actions WHERE 1=1"
    );
    let mut params: Vec<String> = vec![];

    if let Some(project) = project_filter {
        query.push_str(&format!(" AND project = ?{}", params.len() + 1));
        params.push(project.to_string());
    }

    if let Some(kind) = kind_filter {
        let kind_str = serde_json::to_string(&kind)?;
        query.push_str(&format!(" AND kind = ?{}", params.len() + 1));
        params.push(kind_str);
    }

    query.push_str(" ORDER BY ts DESC");

    if let Some(limit) = limit {
        query.push_str(&format!(" LIMIT {}", limit));
    }

    if let Some(offset) = offset {
        query.push_str(&format!(" OFFSET {}", offset));
    }

    let mut stmt = conn.prepare(&query)?;

    let rows = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
        let kind_str: String = row.get(3)?;
        let result_str: String = row.get(7)?;
        let kind: ActionKind = serde_json::from_str(&kind_str)
            .unwrap_or(ActionKind::BeadCreated);
        let result: ActionResult = serde_json::from_str(&result_str)
            .unwrap_or(ActionResult::Success);

        Ok(AuditRow {
            id: row.get(0)?,
            ts: row.get(1)?,
            actor: row.get(2)?,
            kind,
            target: row.get(4)?,
            project: row.get(5)?,
            args_json: row.get(6)?,
            result,
            error: row.get(8)?,
            source: row.get(9)?,
            stitch_id: row.get(10)?,
            args_hash: row.get(11)?,
            hash_prev: row.get(12)?,
            hash_self: row.get(13)?,
        })
    })?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }

    Ok(result)
}

/// Verify hash chain integrity from genesis to the latest row
///
/// Returns Ok(()) if the chain is valid, Err with details of the first break.
pub fn verify_hash_chain() -> Result<()> {
    let path = db_path();
    let conn = Connection::open(&path)?;

    let mut stmt = conn.prepare(
        "SELECT id, ts, actor, kind, target, project, args_json, result, hash_prev, hash_self
         FROM actions ORDER BY rowid ASC"
    )?;

    let mut expected_hash = GENESIS_HASH.to_string();

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?, // id
            row.get::<_, String>(1)?, // ts
            row.get::<_, String>(2)?, // actor
            row.get::<_, String>(3)?, // kind
            row.get::<_, String>(4)?, // target
            row.get::<_, Option<String>>(5)?, // project
            row.get::<_, Option<String>>(6)?, // args_json
            row.get::<_, String>(7)?, // result
            row.get::<_, String>(8)?, // hash_prev
            row.get::<_, String>(9)?, // hash_self
        ))
    })?;

    for row in rows {
        let (id, ts, actor, kind, target, project, args_json, _result, hash_prev, hash_self) = row?;

        // Verify hash_prev matches expected
        if hash_prev != expected_hash {
            return Err(anyhow::anyhow!(
                "Hash chain broken at row {}: expected hash_prev={}, got={}",
                id, expected_hash, hash_prev
            ));
        }

        // Verify hash_self is correct
        let hash_input = format!(
            "{}{}{}{}{}{:?}{}",
            id, ts, actor, kind, target, project, args_json.as_deref().unwrap_or_default()
        );
        let computed_hash = hex_encode(sha256(hash_input.as_bytes()));

        if hash_self != computed_hash {
            return Err(anyhow::anyhow!(
                "Hash self mismatch at row {}: expected={}, got={}",
                id, computed_hash, hash_self
            ));
        }

        expected_hash = hash_self;
    }

    Ok(())
}

/// Create a row in the `stitches` table and link beads via `stitch_beads`.
///
/// The `kind` must be one of the CHECK-constrained values: operator, dictated, worker, ad-hoc.
/// For stitch submit from the form, `kind` is always `"operator"`.
pub fn create_stitch(
    stitch_id: &str,
    project: &str,
    kind: &str,
    title: &str,
    created_by: &str,
    bead_ids: &[(&str, &str)], // (bead_id, workspace)
) -> Result<()> {
    let path = db_path();
    let conn = Connection::open(&path)?;
    let now = Utc::now().to_rfc3339();

    conn.execute(
        r#"
        INSERT INTO stitches (id, project, kind, title, created_by, created_at, last_activity_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
        params![stitch_id, project, kind, title, created_by, now, now],
    )?;

    for (bead_id, workspace) in bead_ids {
        conn.execute(
            r#"
            INSERT INTO stitch_beads (stitch_id, bead_id, workspace, relationship)
            VALUES (?, ?, ?, 'created-here')
            "#,
            params![stitch_id, bead_id, workspace],
        )?;
    }

    Ok(())
}

/// Delete a stitch row and its linked bead rows from fleet.db.
///
/// Used during rollback when partial bead creation failure occurs after
/// the stitch row has been inserted. Explicitly removes stitch_beads
/// entries before the stitch row since FK enforcement may not be active.
pub fn delete_stitch(stitch_id: &str) -> Result<()> {
    let path = db_path();
    let conn = Connection::open(&path)?;
    conn.execute("DELETE FROM stitch_beads WHERE stitch_id = ?", params![stitch_id])?;
    conn.execute("DELETE FROM stitches WHERE id = ?", params![stitch_id])?;
    Ok(())
}

/// Database path: `~/.hoop/fleet.db`
pub fn db_path() -> PathBuf {
    // Allow tests to override the database path via env var
    if let Ok(path) = std::env::var("_HOOP_FLEET_DB_PATH") {
        return PathBuf::from(path);
    }
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
    init_fleet_db_at(db_path())
}

/// Initialize fleet.db at an explicit path (for testing).
pub fn init_fleet_db_at(path: PathBuf) -> Result<()> {
    init_fleet_db_at_version(path, SCHEMA_VERSION)
}

/// Initialize fleet.db at an explicit path with an explicit binary version.
///
/// Exposed for integration testing of the §20.1 major-upgrade gate: callers
/// can pass a binary_version of e.g. "2.0.0" against a database that is still
/// at "1.x" to verify that the gate fires with the exact diagnostic message.
pub fn init_fleet_db_at_version(path: PathBuf, binary_version: &str) -> Result<()> {
    let parent = path.parent().ok_or_else(|| anyhow::anyhow!("Invalid db path"))?;

    // Ensure parent directory exists
    std::fs::create_dir_all(parent)?;

    let exists = path.exists();

    info!(
        "Initializing fleet.db at {} (exists: {}, binary schema {})",
        path.display(),
        exists,
        binary_version,
    );

    let mut conn = Connection::open(&path)?;

    // Enable WAL mode for concurrent reads
    conn.pragma_update(None, "journal_mode", "WAL")?;

    if !exists {
        // Fresh database: create schema and insert genesis row
        create_schema(&mut conn)?;
        insert_genesis_row(&mut conn)?;
        info!("fleet.db created with initial schema {}, running migrations to {}", INITIAL_SCHEMA_VERSION, binary_version);

        // Run migrations to bring fresh database to current version
        let start = std::time::Instant::now();
        run_migrations(&mut conn, INITIAL_SCHEMA_VERSION)?;
        let elapsed_ms = start.elapsed().as_secs_f64() * 1_000.0;
        crate::metrics::metrics().hoop_schema_migration_duration_ms.observe(
            &[INITIAL_SCHEMA_VERSION, binary_version],
            elapsed_ms,
        );
        info!("Migrations complete, schema version {}", binary_version);
    } else {
        // Existing database: verify schema version and run migrations
        let version = get_schema_version(&conn)?;

        // §20.1 major-upgrade gate: refuse startup when binary major > stored major.
        // "0.x" is the pre-migration bootstrap version — always upgradeable through
        // the minor-migration chain regardless of the binary's major.
        check_schema_major_gate(&version, binary_version)?;

        if version != binary_version {
            info!(
                "fleet.db schema version {} -> {}, running migrations",
                version, binary_version
            );
            let start = std::time::Instant::now();
            run_migrations(&mut conn, &version)?;
            let elapsed_ms = start.elapsed().as_secs_f64() * 1_000.0;
            crate::metrics::metrics().hoop_schema_migration_duration_ms.observe(
                &[&version, binary_version],
                elapsed_ms,
            );
            info!("Migrations complete, schema version {}", binary_version);
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
            error TEXT,
            source TEXT,
            stitch_id TEXT,
            args_hash TEXT,
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
        "{}{}{}{}{}{:?}{}",
        id, ts, actor, kind, target, project, args_json.as_deref().unwrap_or_default()
    );
    let hash_self = hex_encode(sha256(hash_input.as_bytes()));

    conn.execute(
        r#"
        INSERT INTO actions (id, ts, actor, kind, target, project, args_json, result, error, source, stitch_id, args_hash, hash_prev, hash_self)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        params![
            id, ts, actor, kind, target, project, args_json, result,
            None::<String>, None::<String>, None::<String>, None::<String>,
            hash_prev, hash_self
        ],
    )?;

    info!("Genesis row inserted: {}", id);

    Ok(())
}

/// Get current schema version from metadata table
pub fn get_schema_version(conn: &Connection) -> Result<String> {
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
            // Fall through to 1.2.0
            migrate_v11_to_v12(conn)?;
            // Fall through to 1.3.0
            migrate_v12_to_v13(conn)?;
            // Fall through to 1.4.0
            migrate_v13_to_v14(conn)?;
            // Fall through to 1.5.0
            migrate_v14_to_v15(conn)?;
            // Fall through to 1.6.0
            migrate_v15_to_v16(conn)?;
            // Fall through to 1.7.0
            migrate_v16_to_v17(conn)?;
            // Fall through to 1.8.0
            migrate_v17_to_v18(conn)?;
            // Fall through to 1.9.0
            migrate_v18_to_v19(conn)?;
            // Fall through to 1.10.0
            migrate_v19_to_v110(conn)?;
            // Fall through to 1.11.0
            migrate_v110_to_v111(conn)?;
            // Fall through to 1.12.0
            migrate_v111_to_v112(conn)?;
        }
        "1.1.0" => {
            migrate_v11_to_v12(conn)?;
            migrate_v12_to_v13(conn)?;
            migrate_v13_to_v14(conn)?;
            migrate_v14_to_v15(conn)?;
            migrate_v15_to_v16(conn)?;
            migrate_v16_to_v17(conn)?;
            migrate_v17_to_v18(conn)?;
            migrate_v18_to_v19(conn)?;
            migrate_v19_to_v110(conn)?;
            migrate_v110_to_v111(conn)?;
            migrate_v111_to_v112(conn)?;
        }
        "1.2.0" => {
            migrate_v12_to_v13(conn)?;
            migrate_v13_to_v14(conn)?;
            migrate_v14_to_v15(conn)?;
            migrate_v15_to_v16(conn)?;
            migrate_v16_to_v17(conn)?;
            migrate_v17_to_v18(conn)?;
            migrate_v18_to_v19(conn)?;
            migrate_v19_to_v110(conn)?;
            migrate_v110_to_v111(conn)?;
            migrate_v111_to_v112(conn)?;
        }
        "1.3.0" => {
            migrate_v13_to_v14(conn)?;
            migrate_v14_to_v15(conn)?;
            migrate_v15_to_v16(conn)?;
            migrate_v16_to_v17(conn)?;
            migrate_v17_to_v18(conn)?;
            migrate_v18_to_v19(conn)?;
            migrate_v19_to_v110(conn)?;
            migrate_v110_to_v111(conn)?;
            migrate_v111_to_v112(conn)?;
        }
        "1.4.0" => {
            migrate_v14_to_v15(conn)?;
            migrate_v15_to_v16(conn)?;
            migrate_v16_to_v17(conn)?;
            migrate_v17_to_v18(conn)?;
            migrate_v18_to_v19(conn)?;
            migrate_v19_to_v110(conn)?;
            migrate_v110_to_v111(conn)?;
            migrate_v111_to_v112(conn)?;
        }
        "1.5.0" => {
            migrate_v15_to_v16(conn)?;
            migrate_v16_to_v17(conn)?;
            migrate_v17_to_v18(conn)?;
            migrate_v18_to_v19(conn)?;
            migrate_v19_to_v110(conn)?;
            migrate_v110_to_v111(conn)?;
            migrate_v111_to_v112(conn)?;
        }
        "1.6.0" => {
            migrate_v16_to_v17(conn)?;
            migrate_v17_to_v18(conn)?;
            migrate_v18_to_v19(conn)?;
            migrate_v19_to_v110(conn)?;
            migrate_v110_to_v111(conn)?;
            migrate_v111_to_v112(conn)?;
        }
        "1.7.0" => {
            migrate_v17_to_v18(conn)?;
            migrate_v18_to_v19(conn)?;
            migrate_v19_to_v110(conn)?;
            migrate_v110_to_v111(conn)?;
            migrate_v111_to_v112(conn)?;
        }
        "1.8.0" => {
            migrate_v18_to_v19(conn)?;
            migrate_v19_to_v110(conn)?;
            migrate_v110_to_v111(conn)?;
            migrate_v111_to_v112(conn)?;
        }
        "1.9.0" => {
            migrate_v19_to_v110(conn)?;
            migrate_v110_to_v111(conn)?;
            migrate_v111_to_v112(conn)?;
        }
        "1.10.0" => {
            migrate_v110_to_v111(conn)?;
            migrate_v111_to_v112(conn)?;
        }
        "1.11.0" => {
            migrate_v111_to_v112(conn)?;
        }
        "1.12.0" => {
            info!("Already at schema version 1.12.0, no migrations needed");
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported schema version: {}. Expected 0.1.0–1.11.0",
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
    update_schema_version(conn, "1.1.0")?;
    Ok(())
}

/// Migration 1.1.0 → 1.2.0: Add Pattern service tables
///
/// This migration creates three Pattern-related tables:
/// - patterns: Operator-curated groupings of Stitches toward a goal
/// - pattern_members: Links between patterns and stitches (many-to-many)
/// - pattern_queries: Saved queries for auto-including matching stitches
///
/// Includes a recursive-CTE trigger to prevent parent_pattern cycles
/// and indexes for efficient member lookups.
fn migrate_v11_to_v12(conn: &mut Connection) -> Result<()> {
    info!("Running migration 1.1.0 → 1.2.0: Adding Pattern service tables");

    // Create patterns table
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS patterns (
            id TEXT PRIMARY KEY NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            status TEXT NOT NULL DEFAULT 'planned'
                CHECK(status IN ('planned', 'active', 'blocked', 'done', 'abandoned')),
            owner TEXT,
            deadline TEXT,
            parent_pattern TEXT REFERENCES patterns(id) ON DELETE SET NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            CHECK(parent_pattern IS NULL OR parent_pattern != id)
        )
        "#,
        [],
    )?;

    conn.execute(
        r#"
        CREATE INDEX IF NOT EXISTS idx_patterns_status
        ON patterns(status)
        "#,
        [],
    )?;

    conn.execute(
        r#"
        CREATE INDEX IF NOT EXISTS idx_patterns_parent
        ON patterns(parent_pattern)
        "#,
        [],
    )?;

    // Trigger: prevent parent_pattern cycles on UPDATE using recursive CTE
    conn.execute(
        r#"
        CREATE TRIGGER IF NOT EXISTS check_pattern_cycle
        BEFORE UPDATE OF parent_pattern ON patterns
        WHEN NEW.parent_pattern IS NOT NULL
        BEGIN
            SELECT RAISE(ABORT, 'Pattern parent cycle detected')
            WHERE EXISTS (
                WITH RECURSIVE ancestors(ancestor_id) AS (
                    SELECT NEW.parent_pattern
                    UNION ALL
                    SELECT p.parent_pattern
                    FROM patterns p
                    INNER JOIN ancestors a ON p.id = a.ancestor_id
                    WHERE p.parent_pattern IS NOT NULL
                )
                SELECT 1 FROM ancestors WHERE ancestor_id = NEW.id
            );
        END
        "#,
        [],
    )?;

    // Trigger: prevent self-reference on INSERT (defensive, CHECK also covers this)
    conn.execute(
        r#"
        CREATE TRIGGER IF NOT EXISTS check_pattern_self_ref_insert
        BEFORE INSERT ON patterns
        WHEN NEW.parent_pattern IS NOT NULL
        BEGIN
            SELECT RAISE(ABORT, 'Pattern cannot reference itself as parent')
            WHERE EXISTS (
                SELECT 1 WHERE NEW.parent_pattern = NEW.id
            );
        END
        "#,
        [],
    )?;

    // Create pattern_members table
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS pattern_members (
            pattern_id TEXT NOT NULL REFERENCES patterns(id) ON DELETE CASCADE,
            stitch_id TEXT NOT NULL REFERENCES stitches(id) ON DELETE CASCADE,
            added_at TEXT NOT NULL DEFAULT (datetime('now')),
            PRIMARY KEY (pattern_id, stitch_id)
        )
        "#,
        [],
    )?;

    conn.execute(
        r#"
        CREATE INDEX IF NOT EXISTS idx_pattern_members_pattern
        ON pattern_members(pattern_id)
        "#,
        [],
    )?;

    conn.execute(
        r#"
        CREATE INDEX IF NOT EXISTS idx_pattern_members_stitch
        ON pattern_members(stitch_id)
        "#,
        [],
    )?;

    // Create pattern_queries table
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS pattern_queries (
            pattern_id TEXT NOT NULL REFERENCES patterns(id) ON DELETE CASCADE,
            saved_query TEXT NOT NULL,
            PRIMARY KEY (pattern_id, saved_query)
        )
        "#,
        [],
    )?;

    conn.execute(
        r#"
        CREATE INDEX IF NOT EXISTS idx_pattern_queries_pattern
        ON pattern_queries(pattern_id)
        "#,
        [],
    )?;

    info!("Pattern service tables created successfully");
    update_schema_version(conn, "1.2.0")?;
    Ok(())
}

/// Migration 1.2.0 → 1.3.0: Add dictated_notes metadata table
///
/// Dictated notes are Stitches with `kind='dictated'`. This table stores
/// note-specific metadata (audio filename, transcript, timestamps) that
/// doesn't belong on the generic stitch row.
fn migrate_v12_to_v13(conn: &mut Connection) -> Result<()> {
    info!("Running migration 1.2.0 → 1.3.0: Adding dictated_notes table");

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS dictated_notes (
            stitch_id TEXT PRIMARY KEY NOT NULL REFERENCES stitches(id) ON DELETE CASCADE,
            recorded_at TEXT NOT NULL,
            transcribed_at TEXT NOT NULL,
            audio_filename TEXT NOT NULL,
            transcript TEXT NOT NULL,
            duration_secs REAL,
            language TEXT,
            tags TEXT DEFAULT '[]'
        )
        "#,
        [],
    )?;

    conn.execute(
        r#"
        CREATE INDEX IF NOT EXISTS idx_dictated_notes_recorded_at
        ON dictated_notes(recorded_at DESC)
        "#,
        [],
    )?;

    info!("dictated_notes table created successfully");
    update_schema_version(conn, "1.3.0")?;
    Ok(())
}

/// Migration 1.3.0 → 1.4.0: Add word-level timestamps to dictated_notes
///
/// Adds transcript_words column for storing Whisper word-level timestamps
/// to enable audio player with transcript sync functionality.
fn migrate_v13_to_v14(conn: &mut Connection) -> Result<()> {
    info!("Running migration 1.3.0 → 1.4.0: Adding transcript_words column");

    // Add transcript_words column (JSON array of word timestamps)
    conn.execute(
        "ALTER TABLE dictated_notes ADD COLUMN transcript_words TEXT",
        [],
    )?;

    info!("transcript_words column added successfully");
    update_schema_version(conn, "1.4.0")?;
    Ok(())
}

/// Migration 1.4.0 → 1.5.0: Add transcription_jobs table
///
/// Creates the transcription_jobs table for async job queue management.
/// Tracks transcription job status, retry attempts, and error messages.
fn migrate_v14_to_v15(conn: &mut Connection) -> Result<()> {
    info!("Running migration 1.4.0 → 1.5.0: Adding transcription_jobs table");

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS transcription_jobs (
            id TEXT PRIMARY KEY NOT NULL,
            stitch_id TEXT NOT NULL,
            audio_path TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            attempts INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            started_at TEXT,
            completed_at TEXT,
            error_message TEXT
        )
        "#,
        [],
    )?;

    conn.execute(
        r#"
        CREATE INDEX IF NOT EXISTS idx_transcription_jobs_stitch_id
        ON transcription_jobs(stitch_id)
        "#,
        [],
    )?;

    conn.execute(
        r#"
        CREATE INDEX IF NOT EXISTS idx_transcription_jobs_status
        ON transcription_jobs(status)
        "#,
        [],
    )?;

    info!("transcription_jobs table created successfully");
    update_schema_version(conn, "1.5.0")?;
    Ok(())
}

/// Migration 1.5.0 → 1.6.0: Add transcription_status to dictated_notes
///
/// Tracks whether transcription is pending, completed, or failed so the UI
/// can render warning cards for partial/failed transcriptions.
fn migrate_v15_to_v16(conn: &mut Connection) -> Result<()> {
    info!("Running migration 1.5.0 → 1.6.0: Adding transcription_status column");

    conn.execute(
        "ALTER TABLE dictated_notes ADD COLUMN transcription_status TEXT NOT NULL DEFAULT 'pending'",
        [],
    )?;

    update_schema_version(conn, "1.6.0")?;
    Ok(())
}

/// Migration 1.6.0 → 1.7.0: Add audit trail columns to actions table
///
/// Adds error, source, stitch_id, and args_hash columns for queryable audit
/// trail per §5.2 / §13. The source field tracks form/chat/bulk/template,
/// stitch_id links to the originating Stitch, and args_hash provides a
/// quick integrity checksum of the serialized args.
fn migrate_v16_to_v17(conn: &mut Connection) -> Result<()> {
    info!("Running migration 1.6.0 → 1.7.0: Adding audit trail columns to actions");

    add_column_if_not_exists(conn, "actions", "error", "TEXT")?;
    add_column_if_not_exists(conn, "actions", "source", "TEXT")?;
    add_column_if_not_exists(conn, "actions", "stitch_id", "TEXT")?;
    add_column_if_not_exists(conn, "actions", "args_hash", "TEXT")?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_actions_source ON actions(source)",
        [],
    )?;

    update_schema_version(conn, "1.7.0")?;
    Ok(())
}

/// Add a column to a table only if it doesn't already exist.
fn add_column_if_not_exists(conn: &mut Connection, table: &str, column: &str, col_type: &str) -> Result<()> {
    let exists: bool = conn.query_row(
        &format!("SELECT COUNT(*) > 0 FROM pragma_table_info('{}') WHERE name = '{}'", table, column),
        [],
        |row| row.get(0),
    )?;
    if !exists {
        conn.execute(&format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, col_type), [])?;
    }
    Ok(())
}

/// Migration 1.7.0 → 1.8.0: Add agent_sessions table
///
/// Tracks persistent agent sessions across daemon restarts. Each row records
/// the adapter-native session ID, which adapter created it, the model in use,
/// cost and token accumulators, and the timestamps needed to compute session
/// age. On daemon restart HOOP reads the most recent active row and reattaches
/// via the adapter's native resume mechanism.
fn migrate_v17_to_v18(conn: &mut Connection) -> Result<()> {
    info!("Running migration 1.7.0 → 1.8.0: Adding agent_sessions table");

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS agent_sessions (
            id TEXT PRIMARY KEY NOT NULL,
            adapter_session_id TEXT NOT NULL,
            adapter TEXT NOT NULL,
            model TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'active'
                CHECK(status IN ('active', 'archived', 'switched', 'disabled')),
            stitch_id TEXT,
            cost_usd REAL NOT NULL DEFAULT 0.0,
            input_tokens INTEGER NOT NULL DEFAULT 0,
            output_tokens INTEGER NOT NULL DEFAULT 0,
            turn_count INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            last_activity_at TEXT NOT NULL,
            archived_at TEXT,
            archived_reason TEXT
        )
        "#,
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_agent_sessions_status ON agent_sessions(status)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_agent_sessions_adapter ON agent_sessions(adapter)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_agent_sessions_created_at ON agent_sessions(created_at DESC)",
        [],
    )?;

    update_schema_version(conn, "1.8.0")?;
    Ok(())
}

/// Migration 1.8.0 → 1.9.0: Add reflection_ledger table
///
/// The Reflection Ledger stores operator-approved rules extracted from repeated
/// patterns in operator Stitches. Entries are scoped (global / project / pattern),
/// carry a status lifecycle (proposed → approved → archived), and track how often
/// they are injected into agent sessions.
fn migrate_v18_to_v19(conn: &mut Connection) -> Result<()> {
    info!("Running migration 1.8.0 → 1.9.0: Adding reflection_ledger table");

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS reflection_ledger (
            id TEXT PRIMARY KEY NOT NULL,
            scope TEXT NOT NULL,
            rule TEXT NOT NULL,
            reason TEXT NOT NULL,
            source_stitches TEXT NOT NULL DEFAULT '[]',
            status TEXT NOT NULL DEFAULT 'proposed'
                CHECK(status IN ('proposed', 'approved', 'rejected', 'archived')),
            created_at TEXT NOT NULL,
            last_applied TEXT,
            applied_count INTEGER NOT NULL DEFAULT 0
        )
        "#,
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_reflection_ledger_status ON reflection_ledger(status)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_reflection_ledger_scope ON reflection_ledger(scope)",
        [],
    )?;

    update_schema_version(conn, "1.9.0")?;
    Ok(())
}

/// Migration 1.9.0 → 1.10.0: Add draft_queue table
///
/// The draft queue holds agent-created stitch drafts pending operator review.
/// Agent calls to `create_stitch` insert here instead of calling `br create`.
/// The operator reviews, edits, approves, or rejects drafts through the UI.
fn migrate_v19_to_v110(conn: &mut Connection) -> Result<()> {
    info!("Running migration 1.9.0 → 1.10.0: Adding draft_queue table");

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS draft_queue (
            id TEXT PRIMARY KEY NOT NULL,
            project TEXT NOT NULL,
            title TEXT NOT NULL,
            kind TEXT NOT NULL,
            description TEXT,
            has_acceptance_criteria INTEGER NOT NULL DEFAULT 0,
            priority INTEGER,
            labels TEXT DEFAULT '[]',
            created_by TEXT NOT NULL,
            created_at TEXT NOT NULL,
            source TEXT NOT NULL DEFAULT 'agent',
            agent_session_id TEXT,
            status TEXT NOT NULL DEFAULT 'pending'
                CHECK(status IN ('pending', 'approved', 'submitted', 'rejected', 'edited')),
            version INTEGER NOT NULL DEFAULT 1,
            original_json TEXT,
            resolved_by TEXT,
            resolved_at TEXT,
            rejection_reason TEXT,
            stitch_id TEXT,
            preview_json TEXT
        )
        "#,
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_draft_queue_status ON draft_queue(status)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_draft_queue_project ON draft_queue(project)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_draft_queue_created_at ON draft_queue(created_at DESC)",
        [],
    )?;

    update_schema_version(conn, "1.10.0")?;
    Ok(())
}

/// Migration 1.10.0 → 1.11.0: Add morning_briefs table
///
/// Stores generated morning briefs with their markdown content, headline,
/// and references to any draft Stitches created during generation.
fn migrate_v110_to_v111(conn: &mut Connection) -> Result<()> {
    info!("Running migration 1.10.0 → 1.11.0: Adding morning_briefs table");

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS morning_briefs (
            id TEXT PRIMARY KEY NOT NULL,
            generated_at TEXT NOT NULL,
            window_from TEXT NOT NULL,
            window_to TEXT NOT NULL,
            headline TEXT NOT NULL,
            markdown_content TEXT NOT NULL,
            draft_ids TEXT NOT NULL DEFAULT '[]',
            session_id TEXT,
            status TEXT NOT NULL DEFAULT 'complete'
                CHECK(status IN ('running', 'complete', 'failed')),
            error TEXT
        )
        "#,
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_morning_briefs_generated_at ON morning_briefs(generated_at DESC)",
        [],
    )?;

    update_schema_version(conn, "1.11.0")?;
    Ok(())
}

/// Migration 1.11.0 → 1.12.0: Add `has_started_session` to agent_sessions
///
/// Per §A2: the flag gates whether the adapter emits a create-vs-resume invocation.
/// It persists across daemon restarts so that a reattach after crash doesn't
/// accidentally send `--session-id` when the provider already has the session.
fn migrate_v111_to_v112(conn: &mut Connection) -> Result<()> {
    info!("Running migration 1.11.0 → 1.12.0: Adding has_started_session to agent_sessions");

    add_column_if_not_exists(conn, "agent_sessions", "has_started_session", "INTEGER NOT NULL DEFAULT 0")?;

    update_schema_version(conn, "1.12.0")?;
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

// ---------------------------------------------------------------------------
// §20.1 Major-upgrade gate
// ---------------------------------------------------------------------------

/// Extract the major version component from a semver string (e.g. "1.11.0" → 1).
pub fn extract_major(version: &str) -> Option<u64> {
    version.split('.').next()?.parse().ok()
}

/// §20.1 major-upgrade startup gate.
///
/// If `binary_version`'s major exceeds `stored_version`'s major, returns the
/// exact diagnostic message specified in §20.1 so the operator knows precisely
/// what to run.  "0.x" is the pre-migration bootstrap version and is never
/// subject to the gate — it always migrates forward through the minor chain.
pub fn check_schema_major_gate(stored_version: &str, binary_version: &str) -> Result<()> {
    let stored_major = extract_major(stored_version)
        .ok_or_else(|| anyhow::anyhow!("Unparseable stored schema version: {}", stored_version))?;
    let binary_major = extract_major(binary_version)
        .ok_or_else(|| anyhow::anyhow!("Unparseable binary schema version: {}", binary_version))?;

    // "0.x" is the bootstrap version — always upgradeable through minor migrations.
    if stored_major == 0 {
        return Ok(());
    }

    if binary_major > stored_major {
        anyhow::bail!(
            "Your data is schema version {stored_major}.x; this binary requires {binary_major}.x. \
             Run `hoop migrate --from-{stored_major} --confirm` or restore from a pre-upgrade backup."
        );
    }
    Ok(())
}

/// Run the major-upgrade migration from the stored version to `binary_version`.
///
/// Called by `hoop migrate --from-N --confirm` or `--major-upgrade --confirm`.
/// Updates `schema_version` in the database.  When a real 2.x schema is
/// defined, DDL migration steps should be added inside this function before
/// the version update.
pub fn run_major_upgrade_at_version(path: PathBuf, binary_version: &str) -> Result<()> {
    let mut conn = Connection::open(&path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;

    let stored_version = get_schema_version(&conn)?;
    let stored_major = extract_major(&stored_version)
        .ok_or_else(|| anyhow::anyhow!("Unparseable stored schema version: {}", stored_version))?;
    let binary_major = extract_major(binary_version)
        .ok_or_else(|| anyhow::anyhow!("Unparseable binary schema version: {}", binary_version))?;

    if binary_major <= stored_major {
        anyhow::bail!(
            "No major upgrade needed: data is at schema version {} and binary requires {}. \
             Major upgrade only applies when binary major > stored major.",
            stored_version,
            binary_version,
        );
    }

    info!(
        "Running major upgrade: schema {} → {} (major {} → {})",
        stored_version, binary_version, stored_major, binary_major
    );

    // Future: add DDL migration steps for each major transition here.
    // For now (1→2 is the first path) the schema tables carry forward and
    // only the recorded version needs updating.
    update_schema_version(&mut conn, binary_version)?;

    info!("Major upgrade complete: schema_version is now {}", binary_version);
    Ok(())
}

/// Run the major upgrade using the binary's own SCHEMA_VERSION as the target.
///
/// This is the production entry point used by `hoop migrate --major-upgrade --confirm`.
pub fn run_major_upgrade() -> Result<()> {
    run_major_upgrade_at_version(db_path(), SCHEMA_VERSION)
}

// ---------------------------------------------------------------------------
// agent_sessions CRUD
// ---------------------------------------------------------------------------

/// A row from the `agent_sessions` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSessionRow {
    pub id: String,
    pub adapter_session_id: String,
    pub adapter: String,
    pub model: String,
    pub status: String,
    pub stitch_id: Option<String>,
    pub cost_usd: f64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub turn_count: i64,
    pub has_started_session: bool,
    pub created_at: String,
    pub last_activity_at: String,
    pub archived_at: Option<String>,
    pub archived_reason: Option<String>,
}

/// Insert a new agent session row.
pub fn insert_agent_session(row: &AgentSessionRow) -> Result<()> {
    let path = db_path();
    let conn = Connection::open(&path)?;
    conn.execute(
        r#"INSERT INTO agent_sessions
           (id, adapter_session_id, adapter, model, status, stitch_id,
            cost_usd, input_tokens, output_tokens, turn_count, has_started_session,
            created_at, last_activity_at, archived_at, archived_reason)
           VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)"#,
        params![
            row.id,
            row.adapter_session_id,
            row.adapter,
            row.model,
            row.status,
            row.stitch_id,
            row.cost_usd,
            row.input_tokens,
            row.output_tokens,
            row.turn_count,
            row.has_started_session as i64,
            row.created_at,
            row.last_activity_at,
            row.archived_at,
            row.archived_reason,
        ],
    )?;
    Ok(())
}

/// Load the most recent active agent session (for reattach on restart).
pub fn load_active_agent_session() -> Result<Option<AgentSessionRow>> {
    let path = db_path();
    let conn = Connection::open(&path)?;
    let mut stmt = conn.prepare(
        "SELECT id, adapter_session_id, adapter, model, status, stitch_id,
                cost_usd, input_tokens, output_tokens, turn_count,
                has_started_session, created_at, last_activity_at, archived_at, archived_reason
         FROM agent_sessions
         WHERE status = 'active'
         ORDER BY created_at DESC LIMIT 1",
    )?;
    let row = stmt.query_row([], |row| {
        Ok(AgentSessionRow {
            id: row.get(0)?,
            adapter_session_id: row.get(1)?,
            adapter: row.get(2)?,
            model: row.get(3)?,
            status: row.get(4)?,
            stitch_id: row.get(5)?,
            cost_usd: row.get(6)?,
            input_tokens: row.get(7)?,
            output_tokens: row.get(8)?,
            turn_count: row.get(9)?,
            has_started_session: row.get(10)?,
            created_at: row.get(11)?,
            last_activity_at: row.get(12)?,
            archived_at: row.get(13)?,
            archived_reason: row.get(14)?,
        })
    });
    match row {
        Ok(r) => Ok(Some(r)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(anyhow::anyhow!("Failed to load active agent session: {}", e)),
    }
}

/// Accumulate cost and tokens after a completed turn.
pub fn update_agent_session_usage(
    session_id: &str,
    input_tokens: i64,
    output_tokens: i64,
    cost_delta_usd: f64,
) -> Result<()> {
    let path = db_path();
    let conn = Connection::open(&path)?;
    let now = Utc::now().to_rfc3339();
    conn.execute(
        r#"UPDATE agent_sessions
           SET input_tokens = input_tokens + ?1,
               output_tokens = output_tokens + ?2,
               cost_usd = cost_usd + ?3,
               turn_count = turn_count + 1,
               last_activity_at = ?4
           WHERE id = ?5"#,
        params![input_tokens, output_tokens, cost_delta_usd, now, session_id],
    )?;
    Ok(())
}

/// Persist the `has_started_session` flag after the first turn completes.
///
/// Called by `AgentSessionManager::handle_event` on the first `TurnComplete`
/// so that a daemon restart knows to use the resume form (`--resume`, `exec resume`,
/// `--continue`) instead of the create form.
pub fn update_has_started_session(session_id: &str, value: bool) -> Result<()> {
    let path = db_path();
    let conn = Connection::open(&path)?;
    conn.execute(
        "UPDATE agent_sessions SET has_started_session = ?1 WHERE id = ?2",
        params![value as i64, session_id],
    )?;
    Ok(())
}

/// Archive a session (mark as archived/switched/disabled).
pub fn archive_agent_session(session_id: &str, reason: &str) -> Result<()> {
    let path = db_path();
    let conn = Connection::open(&path)?;
    let now = Utc::now().to_rfc3339();
    let status = match reason {
        "switched" => "switched",
        "disabled" => "disabled",
        _ => "archived",
    };
    conn.execute(
        r#"UPDATE agent_sessions
           SET status = ?1, archived_at = ?2, archived_reason = ?3
           WHERE id = ?4"#,
        params![status, now, reason, session_id],
    )?;
    Ok(())
}

/// List recent agent sessions (for the status endpoint).
pub fn list_agent_sessions(limit: usize) -> Result<Vec<AgentSessionRow>> {
    let path = db_path();
    let conn = Connection::open(&path)?;
    let mut stmt = conn.prepare(
        "SELECT id, adapter_session_id, adapter, model, status, stitch_id,
                cost_usd, input_tokens, output_tokens, turn_count,
                has_started_session, created_at, last_activity_at, archived_at, archived_reason
         FROM agent_sessions
         ORDER BY created_at DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit], |row| {
        Ok(AgentSessionRow {
            id: row.get(0)?,
            adapter_session_id: row.get(1)?,
            adapter: row.get(2)?,
            model: row.get(3)?,
            status: row.get(4)?,
            stitch_id: row.get(5)?,
            cost_usd: row.get(6)?,
            input_tokens: row.get(7)?,
            output_tokens: row.get(8)?,
            turn_count: row.get(9)?,
            has_started_session: row.get(10)?,
            created_at: row.get(11)?,
            last_activity_at: row.get(12)?,
            archived_at: row.get(13)?,
            archived_reason: row.get(14)?,
        })
    })?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

// ---------------------------------------------------------------------------
// Draft queue CRUD
// ---------------------------------------------------------------------------

/// A row from the `draft_queue` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftRow {
    pub id: String,
    pub project: String,
    pub title: String,
    pub kind: String,
    pub description: Option<String>,
    pub has_acceptance_criteria: bool,
    pub priority: Option<i64>,
    pub labels: Vec<String>,
    pub created_by: String,
    pub created_at: String,
    pub source: String,
    pub agent_session_id: Option<String>,
    pub status: String,
    pub version: i64,
    pub original_json: Option<String>,
    pub resolved_by: Option<String>,
    pub resolved_at: Option<String>,
    pub rejection_reason: Option<String>,
    pub stitch_id: Option<String>,
    pub preview_json: Option<String>,
}

/// Insert a new draft into the queue.
pub fn insert_draft(row: &DraftRow) -> Result<()> {
    let path = db_path();
    let conn = Connection::open(&path)?;
    let labels_json = serde_json::to_string(&row.labels)?;
    conn.execute(
        r#"INSERT INTO draft_queue
           (id, project, title, kind, description, has_acceptance_criteria,
            priority, labels, created_by, created_at, source, agent_session_id,
            status, version, original_json, resolved_by, resolved_at,
            rejection_reason, stitch_id, preview_json)
           VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20)"#,
        params![
            row.id,
            row.project,
            row.title,
            row.kind,
            row.description,
            row.has_acceptance_criteria as i64,
            row.priority,
            labels_json,
            row.created_by,
            row.created_at,
            row.source,
            row.agent_session_id,
            row.status,
            row.version,
            row.original_json,
            row.resolved_by,
            row.resolved_at,
            row.rejection_reason,
            row.stitch_id,
            row.preview_json,
        ],
    )?;
    Ok(())
}

/// Get a single draft by ID.
pub fn get_draft(draft_id: &str) -> Result<Option<DraftRow>> {
    let path = db_path();
    let conn = Connection::open(&path)?;
    let row = conn.query_row(
        r#"SELECT id, project, title, kind, description, has_acceptance_criteria,
                  priority, labels, created_by, created_at, source, agent_session_id,
                  status, version, original_json, resolved_by, resolved_at,
                  rejection_reason, stitch_id, preview_json
           FROM draft_queue WHERE id = ?1"#,
        [draft_id],
        read_draft_row,
    );
    match row {
        Ok(r) => Ok(Some(r)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(anyhow::anyhow!("Failed to get draft: {}", e)),
    }
}

/// List drafts, optionally filtered by project and/or status.
pub fn list_drafts(
    project: Option<&str>,
    status: Option<&str>,
    limit: usize,
) -> Result<Vec<DraftRow>> {
    let path = db_path();
    let conn = Connection::open(&path)?;
    let mut sql = String::from(
        r#"SELECT id, project, title, kind, description, has_acceptance_criteria,
                  priority, labels, created_by, created_at, source, agent_session_id,
                  status, version, original_json, resolved_by, resolved_at,
                  rejection_reason, stitch_id, preview_json
           FROM draft_queue WHERE 1=1"#,
    );
    let mut p: Vec<String> = Vec::new();
    if let Some(proj) = project {
        sql.push_str(&format!(" AND project = ?{}", p.len() + 1));
        p.push(proj.to_string());
    }
    if let Some(st) = status {
        sql.push_str(&format!(" AND status = ?{}", p.len() + 1));
        p.push(st.to_string());
    }
    sql.push_str(&format!(" ORDER BY created_at DESC LIMIT {}", limit));
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(p.iter()), read_draft_row)?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

/// Update a draft's status and resolution metadata.
pub fn update_draft_status(
    draft_id: &str,
    status: &str,
    resolved_by: Option<&str>,
    resolved_at: Option<&str>,
    rejection_reason: Option<&str>,
    stitch_id: Option<&str>,
) -> Result<()> {
    let path = db_path();
    let conn = Connection::open(&path)?;
    conn.execute(
        r#"UPDATE draft_queue
           SET status = ?1, resolved_by = ?2, resolved_at = ?3,
               rejection_reason = ?4, stitch_id = ?5
           WHERE id = ?6"#,
        params![status, resolved_by, resolved_at, rejection_reason, stitch_id, draft_id],
    )?;
    Ok(())
}

/// Edit a draft's content (creates a new version).
pub fn edit_draft(
    draft_id: &str,
    title: Option<&str>,
    description: Option<&str>,
    kind: Option<&str>,
    priority: Option<i64>,
    labels: Option<&[String]>,
) -> Result<()> {
    let path = db_path();
    let conn = Connection::open(&path)?;
    let now = Utc::now().to_rfc3339();

    if let Some(t) = title {
        conn.execute("UPDATE draft_queue SET title = ?1 WHERE id = ?2", params![t, draft_id])?;
    }
    if let Some(d) = description {
        conn.execute("UPDATE draft_queue SET description = ?1 WHERE id = ?2", params![d, draft_id])?;
    }
    if let Some(k) = kind {
        conn.execute("UPDATE draft_queue SET kind = ?1 WHERE id = ?2", params![k, draft_id])?;
    }
    if let Some(p) = priority {
        conn.execute("UPDATE draft_queue SET priority = ?1 WHERE id = ?2", params![p, draft_id])?;
    }
    if let Some(l) = labels {
        let labels_json = serde_json::to_string(l)?;
        conn.execute("UPDATE draft_queue SET labels = ?1 WHERE id = ?2", params![labels_json, draft_id])?;
    }

    conn.execute(
        "UPDATE draft_queue SET version = version + 1, status = 'edited' WHERE id = ?1",
        params![draft_id],
    )?;

    // Store original on first edit
    conn.execute(
        "UPDATE draft_queue SET original_json = (
            SELECT json_object(
                'title', title, 'description', description, 'kind', kind,
                'priority', priority, 'labels', labels
            )
            FROM draft_queue WHERE id = ?1
        ) WHERE id = ?1 AND original_json IS NULL",
        params![draft_id],
    )?;

    let _ = now; // resolved_at is set on approve/reject, not on edit
    Ok(())
}

/// Helper to read a draft row from a query result.
fn read_draft_row(row: &rusqlite::Row<'_>) -> std::result::Result<DraftRow, rusqlite::Error> {
    let labels_str: String = row.get(7).unwrap_or_else(|_| "[]".to_string());
    let labels: Vec<String> = serde_json::from_str(&labels_str).unwrap_or_default();
    let has_ac: i64 = row.get(5).unwrap_or(0);
    Ok(DraftRow {
        id: row.get(0)?,
        project: row.get(1)?,
        title: row.get(2)?,
        kind: row.get(3)?,
        description: row.get(4)?,
        has_acceptance_criteria: has_ac != 0,
        priority: row.get(6)?,
        labels,
        created_by: row.get(8)?,
        created_at: row.get(9)?,
        source: row.get(10)?,
        agent_session_id: row.get(11)?,
        status: row.get(12)?,
        version: row.get(13)?,
        original_json: row.get(14)?,
        resolved_by: row.get(15)?,
        resolved_at: row.get(16)?,
        rejection_reason: row.get(17)?,
        stitch_id: row.get(18)?,
        preview_json: row.get(19)?,
    })
}

// ---------------------------------------------------------------------------
// Agent enabled persistence (metadata table)
// ---------------------------------------------------------------------------

/// Check whether the agent is enabled. Defaults to true if never set.
pub fn is_agent_enabled() -> Result<bool> {
    let path = db_path();
    let conn = Connection::open(&path)?;
    let val: String = conn
        .query_row(
            "SELECT value FROM metadata WHERE key = 'agent_enabled'",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "true".to_string());
    Ok(val == "true")
}

/// Persist the agent enabled/disabled state.
pub fn set_agent_enabled(enabled: bool) -> Result<()> {
    let path = db_path();
    let conn = Connection::open(&path)?;
    conn.execute(
        "INSERT OR REPLACE INTO metadata (key, value) VALUES ('agent_enabled', ?)",
        params![if enabled { "true" } else { "false" }],
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Reflection Ledger CRUD
// ---------------------------------------------------------------------------

/// A row from the `reflection_ledger` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionLedgerEntry {
    pub id: String,
    pub scope: String,
    pub rule: String,
    pub reason: String,
    pub source_stitches: String, // JSON array
    pub status: String,
    pub created_at: String,
    pub last_applied: Option<String>,
    pub applied_count: i64,
}

/// List approved reflection ledger entries, optionally filtered by scope.
pub fn list_approved_reflection_entries(scope_prefix: Option<&str>) -> Result<Vec<ReflectionLedgerEntry>> {
    let path = db_path();
    let conn = Connection::open(&path)?;
    let mut sql = String::from(
        "SELECT id, scope, rule, reason, source_stitches, status, created_at, last_applied, applied_count
         FROM reflection_ledger WHERE status = 'approved'",
    );
    let mut p: Vec<String> = Vec::new();
    if let Some(prefix) = scope_prefix {
        sql.push_str(&format!(" AND (scope = 'global' OR scope LIKE ?{} || '%')", p.len() + 1));
        p.push(prefix.to_string());
    }
    sql.push_str(" ORDER BY created_at ASC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(p.iter()), |row| {
        Ok(ReflectionLedgerEntry {
            id: row.get(0)?,
            scope: row.get(1)?,
            rule: row.get(2)?,
            reason: row.get(3)?,
            source_stitches: row.get(4)?,
            status: row.get(5)?,
            created_at: row.get(6)?,
            last_applied: row.get(7)?,
            applied_count: row.get(8)?,
        })
    })?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

// ---------------------------------------------------------------------------
// Agent session → Stitch archival
// ---------------------------------------------------------------------------

/// Archive an agent session's transcript as a Stitch.
///
/// Creates a Stitch row of kind "operator" in the "hoop-agent" project,
/// stores in-memory history as stitch_messages, and links the Stitch to
/// the agent_sessions row via the stitch_id column.
pub fn archive_session_as_stitch(
    session_row: &AgentSessionRow,
    history: &[(String, String)], // (role, content) pairs
) -> Result<String> {
    let stitch_id = Uuid::new_v4().to_string();
    let path = db_path();
    let conn = Connection::open(&path)?;
    let now = Utc::now().to_rfc3339();

    let title = format!(
        "Agent session {} ({})",
        session_row.adapter,
        &session_row.created_at[..19].replace('T', " "),
    );

    conn.execute(
        r#"INSERT INTO stitches (id, project, kind, title, created_by, created_at, last_activity_at)
           VALUES (?1, 'hoop-agent', 'operator', ?2, 'hoop:agent', ?3, ?4)"#,
        params![stitch_id, title, session_row.created_at, now],
    )?;

    // Store in-memory history as stitch_messages.
    for (i, (role, content)) in history.iter().enumerate() {
        let msg_id = Uuid::new_v4().to_string();
        let ts = if i == 0 {
            session_row.created_at.clone()
        } else {
            now.clone()
        };
        conn.execute(
            r#"INSERT INTO stitch_messages (id, stitch_id, ts, role, content)
               VALUES (?1, ?2, ?3, ?4, ?5)"#,
            params![msg_id, stitch_id, ts, role, content],
        )?;
    }

    // Link stitch_id on the agent_sessions row.
    conn.execute(
        "UPDATE agent_sessions SET stitch_id = ?1 WHERE id = ?2",
        params![stitch_id, session_row.id],
    )?;

    Ok(stitch_id)
}

/// Load recent Stitches for context carry-forward (last N, any project).
pub fn load_recent_stitches(limit: usize) -> Result<Vec<(String, String, String, String)>> {
    let path = db_path();
    let conn = Connection::open(&path)?;
    let mut stmt = conn.prepare(
        "SELECT id, project, title, last_activity_at FROM stitches
         ORDER BY last_activity_at DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

// ---------------------------------------------------------------------------
// Morning Brief CRUD
// ---------------------------------------------------------------------------

/// A row from the `morning_briefs` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorningBriefRow {
    pub id: String,
    pub generated_at: String,
    pub window_from: String,
    pub window_to: String,
    pub headline: String,
    pub markdown_content: String,
    /// JSON array of draft Stitch IDs created during this brief
    pub draft_ids: Vec<String>,
    pub session_id: Option<String>,
    pub status: String,
    pub error: Option<String>,
}

/// Insert a new morning brief record.
pub fn insert_morning_brief(row: &MorningBriefRow) -> Result<()> {
    // §18.1 secrets scan: flag secrets before storage and lateral propagation (Phase 5)
    {
        let findings = crate::redaction::scan_morning_brief(&row.markdown_content);
        if !findings.is_empty() {
            warn!(
                brief_id = %row.id,
                findings = findings.len(),
                "Morning brief content contains potential secrets — lateral leak risk (§18.1)"
            );
        }
    }

    let path = db_path();
    let conn = Connection::open(&path)?;
    let draft_ids_json = serde_json::to_string(&row.draft_ids)?;
    conn.execute(
        r#"INSERT INTO morning_briefs
           (id, generated_at, window_from, window_to, headline, markdown_content,
            draft_ids, session_id, status, error)
           VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)"#,
        params![
            row.id,
            row.generated_at,
            row.window_from,
            row.window_to,
            row.headline,
            row.markdown_content,
            draft_ids_json,
            row.session_id,
            row.status,
            row.error,
        ],
    )?;
    Ok(())
}

/// Update the status (and optional error) of a morning brief record.
pub fn update_morning_brief_status(id: &str, status: &str, error: Option<&str>) -> Result<()> {
    let path = db_path();
    let conn = Connection::open(&path)?;
    conn.execute(
        "UPDATE morning_briefs SET status = ?1, error = ?2 WHERE id = ?3",
        params![status, error, id],
    )?;
    Ok(())
}

/// Update the session_id of a morning brief record.
pub fn update_morning_brief_session(id: &str, session_id: &str) -> Result<()> {
    let path = db_path();
    let conn = Connection::open(&path)?;
    conn.execute(
        "UPDATE morning_briefs SET session_id = ?1 WHERE id = ?2",
        params![session_id, id],
    )?;
    Ok(())
}

/// Update headline, content, and draft_ids when the brief completes.
pub fn update_morning_brief_content(
    id: &str,
    headline: &str,
    markdown_content: &str,
    draft_ids: &[String],
) -> Result<()> {
    let path = db_path();
    let conn = Connection::open(&path)?;
    let draft_ids_json = serde_json::to_string(draft_ids)?;
    conn.execute(
        "UPDATE morning_briefs SET headline = ?1, markdown_content = ?2, draft_ids = ?3, status = 'complete' WHERE id = ?4",
        params![headline, markdown_content, draft_ids_json, id],
    )?;
    Ok(())
}

/// Load the most recent completed morning brief.
pub fn get_latest_morning_brief() -> Result<Option<MorningBriefRow>> {
    let path = db_path();
    let conn = Connection::open(&path)?;
    let result = conn.query_row(
        r#"SELECT id, generated_at, window_from, window_to, headline, markdown_content,
                  draft_ids, session_id, status, error
           FROM morning_briefs
           WHERE status = 'complete'
           ORDER BY generated_at DESC LIMIT 1"#,
        [],
        read_morning_brief_row,
    );
    match result {
        Ok(r) => Ok(Some(r)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(anyhow::anyhow!("Failed to get latest morning brief: {}", e)),
    }
}

/// List recent morning briefs (most recent first).
pub fn list_morning_briefs(limit: usize) -> Result<Vec<MorningBriefRow>> {
    let path = db_path();
    let conn = Connection::open(&path)?;
    let mut stmt = conn.prepare(
        r#"SELECT id, generated_at, window_from, window_to, headline, markdown_content,
                  draft_ids, session_id, status, error
           FROM morning_briefs
           ORDER BY generated_at DESC LIMIT ?1"#,
    )?;
    let rows = stmt.query_map(params![limit as i64], read_morning_brief_row)?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

fn read_morning_brief_row(
    row: &rusqlite::Row<'_>,
) -> std::result::Result<MorningBriefRow, rusqlite::Error> {
    let draft_ids_str: String = row.get(6).unwrap_or_else(|_| "[]".to_string());
    let draft_ids: Vec<String> = serde_json::from_str(&draft_ids_str).unwrap_or_default();
    Ok(MorningBriefRow {
        id: row.get(0)?,
        generated_at: row.get(1)?,
        window_from: row.get(2)?,
        window_to: row.get(3)?,
        headline: row.get(4)?,
        markdown_content: row.get(5)?,
        draft_ids,
        session_id: row.get(7)?,
        status: row.get(8)?,
        error: row.get(9)?,
    })
}

/// Open a restored fleet.db at an explicit path and run schema migrations.
///
/// Returns the pre-migration schema version for caller logging.
/// Refuses if the snapshot's schema version is newer than this binary's
/// `SCHEMA_VERSION` (per §20).
pub fn restore_and_migrate(db_path: &std::path::Path) -> Result<String> {
    let mut conn = Connection::open(db_path)?;

    // Enable WAL mode
    conn.pragma_update(None, "journal_mode", "WAL")?;

    let version = get_schema_version(&conn)?;

    // Reject newer-than-current snapshots (§20.1)
    if is_newer_version(&version, SCHEMA_VERSION) {
        return Err(anyhow::anyhow!(
            "Snapshot schema version {} is newer than this binary's {}. \
             Upgrade HOOP before restoring this snapshot.",
            version,
            SCHEMA_VERSION
        ));
    }

    if version != SCHEMA_VERSION {
        info!(
            "Restored fleet.db schema {} -> {}, running migrations",
            version, SCHEMA_VERSION
        );
        run_migrations(&mut conn, &version)?;
        info!("Migrations complete, schema version {}", SCHEMA_VERSION);
    } else {
        info!("Restored fleet.db schema version {} verified", version);
    }

    Ok(version)
}

/// Compare two semver strings. Returns true if `a` is strictly newer than `b`.
fn is_newer_version(a: &str, b: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|p| p.parse().ok())
            .collect::<Vec<_>>()
    };
    let va = parse(a);
    let vb = parse(b);
    for i in 0..std::cmp::max(va.len(), vb.len()) {
        let na = va.get(i).unwrap_or(&0);
        let nb = vb.get(i).unwrap_or(&0);
        if na > nb {
            return true;
        }
        if na < nb {
            return false;
        }
    }
    false
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

        // Verify schema version (create_schema stores the initial version)
        let version = get_schema_version(&conn)?;
        assert_eq!(version, INITIAL_SCHEMA_VERSION);

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
    fn test_migration_v01_to_v12() -> Result<()> {
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
        assert_eq!(version, SCHEMA_VERSION);

        // Verify all Stitch tables exist
        let tables = [
            "stitches",
            "stitch_messages",
            "stitch_beads",
            "stitch_links",
            "dictated_notes",
        ];
        for table in tables {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name = ?",
                [table],
                |row| row.get(0),
            )?;
            assert_eq!(count, 1, "Table {} should exist", table);
        }

        // Verify Pattern tables exist
        let pattern_tables = ["patterns", "pattern_members", "pattern_queries"];
        for table in pattern_tables {
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
            "idx_patterns_status",
            "idx_patterns_parent",
            "idx_pattern_members_pattern",
            "idx_pattern_members_stitch",
            "idx_pattern_queries_pattern",
            "idx_dictated_notes_recorded_at",
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
    fn test_migration_v11_to_v12() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut conn = Connection::open(temp_file.path())?;

        // Create schema and migrate to 1.1.0
        create_schema(&mut conn)?;
        run_migrations(&mut conn, "0.1.0")?;
        assert_eq!(get_schema_version(&conn)?, SCHEMA_VERSION);

        // Pattern tables should now exist
        for table in ["patterns", "pattern_members", "pattern_queries"] {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name = ?",
                [table],
                |row| row.get(0),
            )?;
            assert_eq!(count, 1, "Table {} should exist after migration", table);
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

        // Create and migrate to current version
        create_schema(&mut conn)?;
        run_migrations(&mut conn, "0.1.0")?;

        // Running migration again on current version should be a no-op
        run_migrations(&mut conn, SCHEMA_VERSION)?;

        let version = get_schema_version(&conn)?;
        assert_eq!(version, SCHEMA_VERSION);

        Ok(())
    }

    #[test]
    fn test_pattern_status_check_constraint() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut conn = Connection::open(temp_file.path())?;

        create_schema(&mut conn)?;
        run_migrations(&mut conn, "0.1.0")?;

        use uuid::Uuid;

        // Valid statuses should succeed
        for status in ["planned", "active", "blocked", "done", "abandoned"] {
            conn.execute(
                "INSERT INTO patterns (id, title, status) VALUES (?, ?, ?)",
                params![Uuid::new_v4().to_string(), format!("Pattern {}", status), status],
            )?;
        }

        // Invalid status should fail
        let result = conn.execute(
            "INSERT INTO patterns (id, title, status) VALUES (?, ?, ?)",
            params![Uuid::new_v4().to_string(), "Bad", "invalid"],
        );
        assert!(result.is_err(), "CHECK constraint should reject invalid status");

        // NULL status should also fail
        let result = conn.execute(
            "INSERT INTO patterns (id, title, status) VALUES (?, ?, NULL)",
            params![Uuid::new_v4().to_string(), "Null Status"],
        );
        assert!(result.is_err(), "NOT NULL constraint should reject NULL status");

        Ok(())
    }

    #[test]
    fn test_pattern_self_reference_prevention() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut conn = Connection::open(temp_file.path())?;

        create_schema(&mut conn)?;
        run_migrations(&mut conn, "0.1.0")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        use uuid::Uuid;

        let id = Uuid::new_v4().to_string();

        // Insert a pattern
        conn.execute(
            "INSERT INTO patterns (id, title, status) VALUES (?, ?, 'planned')",
            params![id, "Self-ref test"],
        )?;

        // Setting parent_pattern to self should fail (CHECK constraint)
        let result = conn.execute(
            "UPDATE patterns SET parent_pattern = ? WHERE id = ?",
            params![id, id],
        );
        assert!(result.is_err(), "Should prevent self-referencing parent_pattern");

        Ok(())
    }

    #[test]
    fn test_pattern_cycle_prevention() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut conn = Connection::open(temp_file.path())?;

        create_schema(&mut conn)?;
        run_migrations(&mut conn, "0.1.0")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        use uuid::Uuid;

        let a = Uuid::new_v4().to_string();
        let b = Uuid::new_v4().to_string();
        let c = Uuid::new_v4().to_string();

        // Create chain: a → b → c (a is child of b, b is child of c)
        conn.execute(
            "INSERT INTO patterns (id, title, status) VALUES (?, 'C', 'planned')",
            params![c],
        )?;
        conn.execute(
            "INSERT INTO patterns (id, title, status, parent_pattern) VALUES (?, 'B', 'active', ?)",
            params![b, c],
        )?;
        conn.execute(
            "INSERT INTO patterns (id, title, status, parent_pattern) VALUES (?, 'A', 'active', ?)",
            params![a, b],
        )?;

        // 2-node cycle: try to set c's parent to a (would create a→b→c→a)
        let result = conn.execute(
            "UPDATE patterns SET parent_pattern = ? WHERE id = ?",
            params![a, c],
        );
        assert!(result.is_err(), "Should prevent 3-node cycle a→b→c→a");

        // Direct 2-node cycle: try to set c's parent to b (b already has parent c)
        let result = conn.execute(
            "UPDATE patterns SET parent_pattern = ? WHERE id = ?",
            params![b, c],
        );
        assert!(result.is_err(), "Should prevent 2-node cycle b↔c");

        Ok(())
    }

    #[test]
    fn test_pattern_valid_nesting() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut conn = Connection::open(temp_file.path())?;

        create_schema(&mut conn)?;
        run_migrations(&mut conn, "0.1.0")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        use uuid::Uuid;

        let root = Uuid::new_v4().to_string();
        let child = Uuid::new_v4().to_string();
        let grandchild = Uuid::new_v4().to_string();

        // Create a valid 3-level hierarchy
        conn.execute(
            "INSERT INTO patterns (id, title, status) VALUES (?, 'Root', 'active')",
            params![root],
        )?;
        conn.execute(
            "INSERT INTO patterns (id, title, status, parent_pattern) VALUES (?, 'Child', 'active', ?)",
            params![child, root],
        )?;
        conn.execute(
            "INSERT INTO patterns (id, title, status, parent_pattern) VALUES (?, 'Grandchild', 'planned', ?)",
            params![grandchild, child],
        )?;

        // Verify hierarchy
        let (parent_of_grandchild,): (Option<String>,) = conn.query_row(
            "SELECT parent_pattern FROM patterns WHERE id = ?",
            params![grandchild],
            |row| Ok((row.get(0)?,)),
        )?;
        assert_eq!(parent_of_grandchild, Some(child));

        Ok(())
    }

    #[test]
    fn test_pattern_members_foreign_keys() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut conn = Connection::open(temp_file.path())?;

        create_schema(&mut conn)?;
        run_migrations(&mut conn, "0.1.0")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        use uuid::Uuid;

        let pattern_id = Uuid::new_v4().to_string();
        let stitch_id = Uuid::new_v4().to_string();

        // Insert pattern and stitch
        conn.execute(
            "INSERT INTO patterns (id, title, status) VALUES (?, ?, 'active')",
            params![pattern_id, "Test Pattern"],
        )?;
        conn.execute(
            r#"INSERT INTO stitches (id, project, kind, title, created_by, created_at, last_activity_at)
            VALUES (?, ?, 'operator', ?, ?, datetime('now'), datetime('now'))"#,
            params![stitch_id, "test-project", "Test Stitch", "user"],
        )?;

        // Valid membership should succeed
        conn.execute(
            "INSERT INTO pattern_members (pattern_id, stitch_id) VALUES (?, ?)",
            params![pattern_id, stitch_id],
        )?;

        // Duplicate membership should fail (PK constraint)
        let result = conn.execute(
            "INSERT INTO pattern_members (pattern_id, stitch_id) VALUES (?, ?)",
            params![pattern_id, stitch_id],
        );
        assert!(result.is_err(), "Should reject duplicate membership");

        // Invalid pattern_id should fail
        let result = conn.execute(
            "INSERT INTO pattern_members (pattern_id, stitch_id) VALUES (?, ?)",
            params!["nonexistent", stitch_id],
        );
        assert!(result.is_err(), "FK should reject invalid pattern_id");

        // Invalid stitch_id should fail
        let result = conn.execute(
            "INSERT INTO pattern_members (pattern_id, stitch_id) VALUES (?, ?)",
            params![pattern_id, "nonexistent"],
        );
        assert!(result.is_err(), "FK should reject invalid stitch_id");

        // Deleting pattern should cascade to members
        conn.execute("DELETE FROM patterns WHERE id = ?", params![pattern_id])?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM pattern_members WHERE pattern_id = ?",
            params![pattern_id],
            |row| row.get(0),
        )?;
        assert_eq!(count, 0, "Deleting pattern should cascade to members");

        Ok(())
    }

    #[test]
    fn test_pattern_members_multi_pattern() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut conn = Connection::open(temp_file.path())?;

        create_schema(&mut conn)?;
        run_migrations(&mut conn, "0.1.0")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        use uuid::Uuid;

        // One stitch can belong to multiple patterns
        let stitch_id = Uuid::new_v4().to_string();
        conn.execute(
            r#"INSERT INTO stitches (id, project, kind, title, created_by, created_at, last_activity_at)
            VALUES (?, ?, 'operator', ?, ?, datetime('now'), datetime('now'))"#,
            params![stitch_id, "test-project", "Shared Stitch", "user"],
        )?;

        let p1 = Uuid::new_v4().to_string();
        let p2 = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO patterns (id, title, status) VALUES (?, 'Pattern 1', 'active')",
            params![p1],
        )?;
        conn.execute(
            "INSERT INTO patterns (id, title, status) VALUES (?, 'Pattern 2', 'planned')",
            params![p2],
        )?;

        conn.execute(
            "INSERT INTO pattern_members (pattern_id, stitch_id) VALUES (?, ?)",
            params![p1, stitch_id],
        )?;
        conn.execute(
            "INSERT INTO pattern_members (pattern_id, stitch_id) VALUES (?, ?)",
            params![p2, stitch_id],
        )?;

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM pattern_members WHERE stitch_id = ?",
            params![stitch_id],
            |row| row.get(0),
        )?;
        assert_eq!(count, 2, "Stitch should belong to both patterns");

        Ok(())
    }

    #[test]
    fn test_pattern_queries_foreign_keys() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut conn = Connection::open(temp_file.path())?;

        create_schema(&mut conn)?;
        run_migrations(&mut conn, "0.1.0")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        use uuid::Uuid;

        let pattern_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO patterns (id, title, status) VALUES (?, ?, 'active')",
            params![pattern_id, "Query Test"],
        )?;

        // Valid query should succeed
        conn.execute(
            "INSERT INTO pattern_queries (pattern_id, saved_query) VALUES (?, ?)",
            params![pattern_id, "project:kalshi-weather status:active"],
        )?;

        // Duplicate query should fail (PK)
        let result = conn.execute(
            "INSERT INTO pattern_queries (pattern_id, saved_query) VALUES (?, ?)",
            params![pattern_id, "project:kalshi-weather status:active"],
        );
        assert!(result.is_err(), "Should reject duplicate query");

        // Different query for same pattern should succeed
        conn.execute(
            "INSERT INTO pattern_queries (pattern_id, saved_query) VALUES (?, ?)",
            params![pattern_id, "kind:worker cost:>5"],
        )?;

        // Deleting pattern should cascade to queries
        conn.execute("DELETE FROM patterns WHERE id = ?", params![pattern_id])?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM pattern_queries WHERE pattern_id = ?",
            params![pattern_id],
            |row| row.get(0),
        )?;
        assert_eq!(count, 0, "Deleting pattern should cascade to queries");

        Ok(())
    }

    #[test]
    fn test_pattern_parent_set_null_on_delete() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut conn = Connection::open(temp_file.path())?;

        create_schema(&mut conn)?;
        run_migrations(&mut conn, "0.1.0")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        use uuid::Uuid;

        let parent = Uuid::new_v4().to_string();
        let child = Uuid::new_v4().to_string();

        conn.execute(
            "INSERT INTO patterns (id, title, status) VALUES (?, 'Parent', 'active')",
            params![parent],
        )?;
        conn.execute(
            "INSERT INTO patterns (id, title, status, parent_pattern) VALUES (?, 'Child', 'planned', ?)",
            params![child, parent],
        )?;

        // Deleting parent should SET NULL on child's parent_pattern
        conn.execute("DELETE FROM patterns WHERE id = ?", params![parent])?;

        let (child_parent,): (Option<String>,) = conn.query_row(
            "SELECT parent_pattern FROM patterns WHERE id = ?",
            params![child],
            |row| Ok((row.get(0)?,)),
        )?;
        assert_eq!(child_parent, None, "Child's parent should be NULL after parent deletion");

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Agent session persistence tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_migration_v18_to_v19() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut conn = Connection::open(temp_file.path())?;

        create_schema(&mut conn)?;
        run_migrations(&mut conn, "0.1.0")?;
        assert_eq!(get_schema_version(&conn)?, SCHEMA_VERSION);

        // reflection_ledger table should exist
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name = 'reflection_ledger'",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(count, 1, "reflection_ledger table should exist after migration");

        // Check status CHECK constraint
        use uuid::Uuid;
        for status in ["proposed", "approved", "rejected", "archived"] {
            conn.execute(
                "INSERT INTO reflection_ledger (id, scope, rule, reason, status, created_at) VALUES (?, 'global', ?, ?, ?, datetime('now'))",
                params![Uuid::new_v4().to_string(), format!("rule {}", status), format!("reason {}", status), status],
            )?;
        }

        let result = conn.execute(
            "INSERT INTO reflection_ledger (id, scope, rule, reason, status, created_at) VALUES (?, 'global', 'r', 're', 'invalid', datetime('now'))",
            [Uuid::new_v4().to_string()],
        );
        assert!(result.is_err(), "CHECK constraint should reject invalid reflection status");

        Ok(())
    }

    #[test]
    fn test_agent_enabled_persistence() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut conn = Connection::open(temp_file.path())?;

        create_schema(&mut conn)?;
        run_migrations(&mut conn, "0.1.0")?;

        // Default should be true (metadata row doesn't exist yet)
        let default_val: String = conn
            .query_row(
                "SELECT value FROM metadata WHERE key = 'agent_enabled'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "true".to_string());
        assert_eq!(default_val, "true");

        // Set to false
        conn.execute(
            "INSERT OR REPLACE INTO metadata (key, value) VALUES ('agent_enabled', 'false')",
            [],
        )?;

        let val: String = conn.query_row(
            "SELECT value FROM metadata WHERE key = 'agent_enabled'",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(val, "false");

        // Set back to true
        conn.execute(
            "INSERT OR REPLACE INTO metadata (key, value) VALUES ('agent_enabled', 'true')",
            [],
        )?;
        let val: String = conn.query_row(
            "SELECT value FROM metadata WHERE key = 'agent_enabled'",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(val, "true");

        Ok(())
    }

    #[test]
    fn test_reflection_ledger_approved_query() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut conn = Connection::open(temp_file.path())?;

        create_schema(&mut conn)?;
        run_migrations(&mut conn, "0.1.0")?;

        use uuid::Uuid;

        // Insert entries in various statuses
        let approved_id = Uuid::new_v4().to_string();
        let proposed_id = Uuid::new_v4().to_string();
        let rejected_id = Uuid::new_v4().to_string();

        conn.execute(
            "INSERT INTO reflection_ledger (id, scope, rule, reason, status, created_at) VALUES (?, 'global', 'approved rule', 'reason', 'approved', datetime('now'))",
            params![approved_id],
        )?;
        conn.execute(
            "INSERT INTO reflection_ledger (id, scope, rule, reason, status, created_at) VALUES (?, 'global', 'proposed rule', 'reason', 'proposed', datetime('now'))",
            params![proposed_id],
        )?;
        conn.execute(
            "INSERT INTO reflection_ledger (id, scope, rule, reason, status, created_at) VALUES (?, 'global', 'rejected rule', 'reason', 'rejected', datetime('now'))",
            params![rejected_id],
        )?;

        // Only approved should be returned
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM reflection_ledger WHERE status = 'approved'",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(count, 1);

        let rule: String = conn.query_row(
            "SELECT rule FROM reflection_ledger WHERE status = 'approved'",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(rule, "approved rule");

        Ok(())
    }

    #[test]
    fn test_archive_session_as_stitch() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut conn = Connection::open(temp_file.path())?;

        create_schema(&mut conn)?;
        run_migrations(&mut conn, "0.1.0")?;

        use uuid::Uuid;

        // Create an agent session
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            r#"INSERT INTO agent_sessions
               (id, adapter_session_id, adapter, model, status, cost_usd, input_tokens, output_tokens,
                turn_count, created_at, last_activity_at)
               VALUES (?1,?2,'claude','claude-opus-4-7','active',0.05,1000,500,3,?3,?3)"#,
            params![session_id, "adapter-sess-123", now],
        )?;

        // Archive as stitch with history
        let history = vec![
            ("user".to_string(), "What did we do today?".to_string()),
            ("assistant".to_string(), "Here's a summary...".to_string()),
            ("user".to_string(), "Draft a bead for fixing Calico".to_string()),
        ];

        let stitch_id = Uuid::new_v4().to_string();
        let title = format!("Agent session claude ({})", &now[..19].replace('T', " "));
        conn.execute(
            r#"INSERT INTO stitches (id, project, kind, title, created_by, created_at, last_activity_at)
               VALUES (?1, 'hoop-agent', 'operator', ?2, 'hoop:agent', ?3, ?4)"#,
            params![stitch_id, title, now, now],
        )?;

        // Store history as messages
        for (role, content) in &history {
            let msg_id = Uuid::new_v4().to_string();
            conn.execute(
                r#"INSERT INTO stitch_messages (id, stitch_id, ts, role, content)
                   VALUES (?1, ?2, ?3, ?4, ?5)"#,
                params![msg_id, stitch_id, now, role, content],
            )?;
        }

        // Link stitch_id on agent_sessions
        conn.execute(
            "UPDATE agent_sessions SET stitch_id = ?1 WHERE id = ?2",
            params![stitch_id, session_id],
        )?;

        // Verify stitch was created
        let stitch_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM stitches WHERE id = ?",
            params![stitch_id],
            |row| row.get(0),
        )?;
        assert_eq!(stitch_count, 1);

        // Verify messages were stored
        let msg_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM stitch_messages WHERE stitch_id = ?",
            params![stitch_id],
            |row| row.get(0),
        )?;
        assert_eq!(msg_count, 3);

        // Verify link on agent_sessions
        let linked_stitch: Option<String> = conn.query_row(
            "SELECT stitch_id FROM agent_sessions WHERE id = ?",
            params![session_id],
            |row| row.get(0),
        )?;
        assert_eq!(linked_stitch, Some(stitch_id.clone()));

        // Verify stitch project is hoop-agent
        let project: String = conn.query_row(
            "SELECT project FROM stitches WHERE id = ?",
            params![stitch_id],
            |row| row.get(0),
        )?;
        assert_eq!(project, "hoop-agent");

        Ok(())
    }

    #[test]
    fn test_agent_sessions_crud() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut conn = Connection::open(temp_file.path())?;

        create_schema(&mut conn)?;
        run_migrations(&mut conn, "0.1.0")?;

        use uuid::Uuid;

        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        // Insert
        conn.execute(
            r#"INSERT INTO agent_sessions
               (id, adapter_session_id, adapter, model, status, cost_usd, input_tokens,
                output_tokens, turn_count, created_at, last_activity_at)
               VALUES (?1,?2,'claude','claude-opus-4-7','active',0.0,0,0,0,?3,?3)"#,
            params![id, "adapter-sess-1", now],
        )?;

        // Query active
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM agent_sessions WHERE status = 'active'",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(count, 1);

        // Update usage
        conn.execute(
            r#"UPDATE agent_sessions
               SET input_tokens = input_tokens + 100,
                   output_tokens = output_tokens + 50,
                   cost_usd = cost_usd + 0.015,
                   turn_count = turn_count + 1
               WHERE id = ?"#,
            params![id],
        )?;

        let (input, output, cost, turns): (i64, i64, f64, i64) = conn.query_row(
            "SELECT input_tokens, output_tokens, cost_usd, turn_count FROM agent_sessions WHERE id = ?",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )?;
        assert_eq!(input, 100);
        assert_eq!(output, 50);
        assert!((cost - 0.015).abs() < 0.001);
        assert_eq!(turns, 1);

        // Archive
        conn.execute(
            "UPDATE agent_sessions SET status = 'archived', archived_at = datetime('now'), archived_reason = 'test' WHERE id = ?",
            params![id],
        )?;
        let active_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM agent_sessions WHERE status = 'active'",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(active_count, 0);

        Ok(())
    }

    // -----------------------------------------------------------------------
    // §20.1 major-upgrade gate tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_extract_major() {
        assert_eq!(extract_major("1.11.0"), Some(1));
        assert_eq!(extract_major("2.0.0"), Some(2));
        assert_eq!(extract_major("0.1.0"), Some(0));
        assert_eq!(extract_major("10.5.3"), Some(10));
        assert_eq!(extract_major(""), None);
        assert_eq!(extract_major("notanumber"), None);
    }

    #[test]
    fn test_gate_same_major_passes() {
        // Same major, different minor — must not be blocked.
        assert!(check_schema_major_gate("1.5.0", "1.11.0").is_ok());
        assert!(check_schema_major_gate("1.0.0", "1.11.0").is_ok());
        assert!(check_schema_major_gate("1.11.0", "1.11.0").is_ok());
        // Exactly equal — passes.
        assert!(check_schema_major_gate("2.3.1", "2.3.1").is_ok());
    }

    #[test]
    fn test_gate_bootstrap_version_always_passes() {
        // "0.x" is the pre-migration bootstrap — must never be blocked.
        assert!(check_schema_major_gate("0.1.0", "1.11.0").is_ok());
        assert!(check_schema_major_gate("0.1.0", "2.0.0").is_ok());
    }

    #[test]
    fn test_gate_major_mismatch_refuses_with_exact_message() {
        // Integration test: old-schema DB (major 1) + new binary (major 2)
        // → refuses with the exact §20.1 message.
        let err = check_schema_major_gate("1.11.0", "2.0.0").unwrap_err();
        let msg = err.to_string();
        assert_eq!(
            msg,
            "Your data is schema version 1.x; this binary requires 2.x. \
             Run `hoop migrate --from-1 --confirm` or restore from a pre-upgrade backup.",
            "Gate must emit the exact §20.1 diagnostic message"
        );
    }

    #[test]
    fn test_gate_major_3_produces_correct_message() {
        let err = check_schema_major_gate("2.5.0", "3.0.0").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("schema version 2.x"), "msg: {msg}");
        assert!(msg.contains("requires 3.x"), "msg: {msg}");
        assert!(msg.contains("--from-2"), "msg: {msg}");
    }

    #[test]
    fn test_init_fleet_db_at_version_refuses_on_major_mismatch() -> Result<()> {
        // Integration test: initialize a DB at the current schema, then attempt
        // to open it as a hypothetical future 2.x binary — must refuse startup.
        let dir = tempfile::tempdir()?;
        let db_path = dir.path().join("fleet.db");

        // Bootstrap the DB with the current binary version.
        init_fleet_db_at(db_path.clone())?;

        // Verify the DB is at the current schema.
        let conn = Connection::open(&db_path)?;
        let stored = get_schema_version(&conn)?;
        assert_eq!(stored, SCHEMA_VERSION);
        drop(conn);

        // Simulate a 2.x binary attempting startup against a 1.x database.
        let err = init_fleet_db_at_version(db_path.clone(), "2.0.0")
            .expect_err("Must refuse startup with major mismatch");
        let msg = err.to_string();
        assert_eq!(
            msg,
            "Your data is schema version 1.x; this binary requires 2.x. \
             Run `hoop migrate --from-1 --confirm` or restore from a pre-upgrade backup.",
            "Startup refusal must carry the exact §20.1 message"
        );
        Ok(())
    }

    #[test]
    fn test_init_fleet_db_same_major_different_minor_starts_normally() -> Result<()> {
        // Same major, different minor → starts normally (no gate, migrations run).
        let dir = tempfile::tempdir()?;
        let db_path = dir.path().join("fleet.db");

        // Bootstrap DB.
        init_fleet_db_at(db_path.clone())?;

        // Re-open with same binary version — must succeed.
        init_fleet_db_at_version(db_path.clone(), SCHEMA_VERSION)?;

        // Verify the stored version is still current.
        let conn = Connection::open(&db_path)?;
        let stored = get_schema_version(&conn)?;
        assert_eq!(stored, SCHEMA_VERSION);
        Ok(())
    }

    #[test]
    fn test_major_upgrade_at_version_completes_the_path() -> Result<()> {
        // `hoop migrate --major-upgrade --confirm` integration test:
        // after upgrade, a 2.x binary can start (gate passes).
        let dir = tempfile::tempdir()?;
        let db_path = dir.path().join("fleet.db");

        // Start from a fully-migrated 1.x database.
        init_fleet_db_at(db_path.clone())?;

        // Confirm a 2.x binary is currently blocked.
        assert!(
            init_fleet_db_at_version(db_path.clone(), "2.0.0").is_err(),
            "Should be blocked before upgrade"
        );

        // Run the major upgrade (simulating `hoop migrate --major-upgrade --confirm`).
        run_major_upgrade_at_version(db_path.clone(), "2.0.0")?;

        // After upgrade, the gate must pass for a 2.x binary.
        let conn = Connection::open(&db_path)?;
        let stored = get_schema_version(&conn)?;
        drop(conn);
        assert_eq!(stored, "2.0.0");
        check_schema_major_gate(&stored, "2.0.0")?;

        Ok(())
    }

    #[test]
    fn test_major_upgrade_no_op_when_not_needed() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let db_path = dir.path().join("fleet.db");

        init_fleet_db_at(db_path.clone())?;

        // Trying to upgrade to the same major is an error.
        let err = run_major_upgrade_at_version(db_path.clone(), SCHEMA_VERSION)
            .expect_err("Should refuse when binary_major == stored_major");
        assert!(
            err.to_string().contains("No major upgrade needed"),
            "err: {}",
            err
        );
        Ok(())
    }
}
