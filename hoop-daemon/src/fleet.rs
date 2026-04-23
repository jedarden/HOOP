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
use tracing::info;
use uuid::Uuid;

/// Current schema version
const SCHEMA_VERSION: &str = "1.7.0";

/// Initial schema version (for fresh databases - will migrate to SCHEMA_VERSION)
const INITIAL_SCHEMA_VERSION: &str = "0.1.0";

/// Genesis hash - all chains start here
const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Action kind for audit log
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    BeadCreated,
    StitchCreated,
    ConfigChanged,
    ProjectAdded,
    ProjectRemoved,
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
        }
        "1.1.0" => {
            migrate_v11_to_v12(conn)?;
            migrate_v12_to_v13(conn)?;
            migrate_v13_to_v14(conn)?;
            migrate_v14_to_v15(conn)?;
            migrate_v15_to_v16(conn)?;
            migrate_v16_to_v17(conn)?;
        }
        "1.2.0" => {
            migrate_v12_to_v13(conn)?;
            migrate_v13_to_v14(conn)?;
            migrate_v14_to_v15(conn)?;
            migrate_v15_to_v16(conn)?;
            migrate_v16_to_v17(conn)?;
        }
        "1.3.0" => {
            migrate_v13_to_v14(conn)?;
            migrate_v14_to_v15(conn)?;
            migrate_v15_to_v16(conn)?;
            migrate_v16_to_v17(conn)?;
        }
        "1.4.0" => {
            migrate_v14_to_v15(conn)?;
            migrate_v15_to_v16(conn)?;
            migrate_v16_to_v17(conn)?;
        }
        "1.5.0" => {
            migrate_v15_to_v16(conn)?;
            migrate_v16_to_v17(conn)?;
        }
        "1.6.0" => {
            migrate_v16_to_v17(conn)?;
        }
        "1.7.0" => {
            info!("Already at schema version 1.7.0, no migrations needed");
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported schema version: {}. Expected 0.1.0–1.7.0",
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

    conn.execute("ALTER TABLE actions ADD COLUMN error TEXT", [],)?;
    conn.execute("ALTER TABLE actions ADD COLUMN source TEXT", [],)?;
    conn.execute("ALTER TABLE actions ADD COLUMN stitch_id TEXT", [],)?;
    conn.execute("ALTER TABLE actions ADD COLUMN args_hash TEXT", [],)?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_actions_source ON actions(source)",
        [],
    )?;

    update_schema_version(conn, "1.7.0")?;
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
        assert_eq!(get_schema_version(&conn)?, "1.3.0");

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

        // Create and migrate
        create_schema(&mut conn)?;
        run_migrations(&mut conn, "0.1.0")?;

        // Running migration again on 1.3.0 should be a no-op
        run_migrations(&mut conn, "1.3.0")?;

        let version = get_schema_version(&conn)?;
        assert_eq!(version, "1.3.0");

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
}
