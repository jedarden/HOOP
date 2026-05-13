//! Schema migration framework for fleet.db
//!
//! This module provides a structured approach to schema migrations with:
//! - Up/Down migration support for minor version bumps
//! - One-way migrations for major version bumps
//! - Idempotent execution
//! - Audit trail integration
//!
//! Plan reference: §20 Schema migration

use crate::fleet::{
    get_schema_version, update_schema_version, write_schema_migration_audit,
    // Migration functions from fleet.rs
    migrate_v01_to_v11,
    migrate_v11_to_v12,
    migrate_v12_to_v13,
    migrate_v13_to_v14,
    migrate_v14_to_v15,
    migrate_v15_to_v16,
    migrate_v16_to_v17,
    migrate_v17_to_v18,
    migrate_v18_to_v19,
    migrate_v19_to_v110,
    migrate_v110_to_v111,
    migrate_v111_to_v112,
    migrate_v112_to_v113,
    migrate_v113_to_v114,
    migrate_v114_to_v115,
    migrate_v115_to_v116,
    migrate_v116_to_v117,
    migrate_v117_to_v118,
    migrate_v118_to_v119,
    migrate_v119_to_v120,
    migrate_v120_to_v121,
    migrate_v121_to_v122,
    migrate_v122_to_v123,
    migrate_v123_to_v124,
    migrate_v124_to_v125,
    migrate_v125_to_v126,
    migrate_v126_to_v127,
    migrate_v127_to_v128,
    migrate_v128_to_v129,
    migrate_v129_to_v130,
    migrate_v130_to_v131,
    migrate_v131_to_v132,
    migrate_v132_to_v133,
    migrate_v133_to_v134,
};
use anyhow::{bail, Result};
use rusqlite::Connection;
use std::collections::HashMap;
use tracing::info;

// Re-export metrics for migration duration tracking (§16.6)
pub use crate::metrics;

/// A single schema migration
pub struct Migration {
    /// Target version (e.g., "1.25.0")
    pub version: &'static str,
    /// Human-readable description
    pub description: &'static str,
    /// Forward migration (required)
    pub up: MigrationFn,
    /// Rollback migration (optional - only for minor version bumps)
    pub down: Option<MigrationFn>,
}

/// Migration function type
pub type MigrationFn = fn(conn: &mut Connection) -> Result<()>;

/// Migration registry - maps from version to Migration
pub struct MigrationRegistry {
    migrations: HashMap<&'static str, Migration>,
}

impl MigrationRegistry {
    /// Create a new migration registry
    pub fn new() -> Self {
        Self {
            migrations: HashMap::new(),
        }
    }

    /// Register a migration
    pub fn register(&mut self, migration: Migration) -> Result<()> {
        if self.migrations.contains_key(migration.version) {
            bail!("Migration {} is already registered", migration.version);
        }
        self.migrations.insert(migration.version, migration);
        Ok(())
    }

    /// Get all registered migrations in version order
    pub fn all_migrations(&self) -> Vec<&Migration> {
        let mut migrations: Vec<_> = self.migrations.values().collect();
        migrations.sort_by_key(|m| semver_compare(m.version));
        migrations
    }

    /// Get pending migrations from a given version
    pub fn pending_migrations(&self, from_version: &str) -> Result<Vec<&Migration>> {
        let from = semver_compare(from_version);
        let mut pending = Vec::new();

        for migration in self.all_migrations() {
            let ver = semver_compare(migration.version);
            if ver > from {
                pending.push(migration);
            }
        }

        Ok(pending)
    }

    /// Get a migration by version
    pub fn get(&self, version: &str) -> Option<&Migration> {
        self.migrations.get(version)
    }

    /// Check if a rollback is possible for a given version
    pub fn can_rollback(&self, version: &str) -> bool {
        self.migrations
            .get(version)
            .map(|m| m.down.is_some())
            .unwrap_or(false)
    }
}

impl Default for MigrationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Compare two semver strings for ordering
/// Returns a comparable value (major * 1_000_000 + minor * 1000 + patch)
fn semver_compare(version: &str) -> u64 {
    let parts: Vec<u32> = version
        .split('.')
        .filter_map(|p| p.parse().ok())
        .collect();

    let major = *parts.get(0).unwrap_or(&0);
    let minor = *parts.get(1).unwrap_or(&0);
    let patch = *parts.get(2).unwrap_or(&0);

    (major as u64) * 1_000_000 + (minor as u64) * 1000 + (patch as u64)
}

/// Run pending migrations from the current version
pub fn run_pending_migrations(
    conn: &mut Connection,
    registry: &MigrationRegistry,
    current_version: &str,
) -> Result<()> {
    let pending = registry.pending_migrations(current_version)?;

    if pending.is_empty() {
        info!("No pending migrations (at version {})", current_version);
        return Ok(());
    }

    info!(
        "Running {} pending migration(s) from {}",
        pending.len(),
        current_version
    );

    let mut from_version = current_version.to_string();

    for migration in pending {
        let start = std::time::Instant::now();

        info!(
            "Running migration {} → {}: {}",
            from_version,
            migration.version,
            migration.description
        );

        // Clear the change counter before migration
        let _ = conn.execute("SELECT 1", [])?;

        // Run the migration
        (migration.up)(conn)?;

        let elapsed_ms = start.elapsed().as_secs_f64() * 1_000.0;
        let rows_touched = conn.changes();

        // Update schema version
        update_schema_version(conn, migration.version)?;

        info!(
            "Migration {} → {} completed in {:.2} ms ({} rows touched)",
            from_version,
            migration.version,
            elapsed_ms,
            rows_touched
        );

        // Write audit row
        let _ = write_schema_migration_audit(
            &from_version,
            migration.version,
            elapsed_ms,
            rows_touched as i64,
        );

        // Record migration duration metric (§16.6)
        metrics::metrics()
            .hoop_schema_migration_duration_ms
            .observe(&[from_version, migration.version], elapsed_ms);

        from_version = migration.version.to_string();
    }

    Ok(())
}

/// Rollback a single migration
pub fn rollback_migration(
    conn: &mut Connection,
    registry: &MigrationRegistry,
    target_version: &str,
    current_version: &str,
) -> Result<()> {
    let migration = registry
        .get(target_version)
        .ok_or_else(|| anyhow::anyhow!("Migration {} not found", target_version))?;

    let down_fn = migration.down.ok_or_else(|| {
        anyhow::anyhow!(
            "Migration {} does not support rollback (major version bump)",
            target_version
        )
    })?;

    let start = std::time::Instant::now();

    info!(
        "Rolling back migration {} → {}: {}",
        current_version,
        target_version,
        migration.description
    );

    // Clear the change counter before rollback
    let _ = conn.execute("SELECT 1", [])?;

    // Run the rollback
    down_fn(conn)?;

    let elapsed_ms = start.elapsed().as_secs_f64() * 1_000.0;
    let rows_touched = conn.changes();

    // Update schema version
    update_schema_version(conn, target_version)?;

    info!(
        "Rollback {} → {} completed in {:.2} ms ({} rows touched)",
        current_version,
        target_version,
        elapsed_ms,
        rows_touched
    );

    // Write audit row for rollback
    let _ = write_schema_migration_audit(current_version, target_version, elapsed_ms, rows_touched as i64);

    // Record migration duration metric (§16.6)
    metrics::metrics()
        .hoop_schema_migration_duration_ms
        .observe(&[current_version, target_version], elapsed_ms);

    Ok(())
}

/// Get migration status information
#[derive(Debug, Clone)]
pub struct MigrationStatus {
    pub current_version: String,
    pub pending_migrations: Vec<PendingMigrationInfo>,
    pub can_rollback_to: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PendingMigrationInfo {
    pub version: String,
    pub description: String,
    pub can_rollback: bool,
}

/// Get the migration status
pub fn get_migration_status(
    conn: &Connection,
    registry: &MigrationRegistry,
) -> Result<MigrationStatus> {
    let current_version = get_schema_version(conn)?;
    let pending = registry.pending_migrations(&current_version)?;

    let pending_info: Vec<PendingMigrationInfo> = pending
        .iter()
        .map(|m| PendingMigrationInfo {
            version: m.version.to_string(),
            description: m.description.to_string(),
            can_rollback: m.down.is_some(),
        })
        .collect();

    // Determine rollback targets
    let can_rollback_to = if current_version == "0.1.0" {
        Vec::new()
    } else {
        let all = registry.all_migrations();
        let current_idx = all
            .iter()
            .position(|m| m.version == current_version)
            .unwrap_or(0);

        all[..current_idx]
            .iter()
            .filter(|m| m.down.is_some())
            .map(|m| m.version.to_string())
            .collect()
    };

    Ok(MigrationStatus {
        current_version,
        pending_migrations: pending_info,
        can_rollback_to,
    })
}

/// Get the global migration registry with all registered migrations
///
/// This function constructs the registry with all known migrations.
/// Each migration is registered with its up function and, for minor
/// version bumps, an optional down function for rollback.
pub fn get_migration_registry() -> MigrationRegistry {
    let mut registry = MigrationRegistry::new();

    // Register all migrations from 1.1.0 to current
    // Minor version migrations support rollback; major versions do not

    let _ = registry.register(Migration {
        version: "1.1.0",
        description: "Add Stitch service tables",
        up: migrate_v01_to_v11,
        down: Some(rollback_v11_to_v01),
    });

    let _ = registry.register(Migration {
        version: "1.2.0",
        description: "Add Pattern service tables",
        up: migrate_v11_to_v12,
        down: Some(rollback_v12_to_v11),
    });

    let _ = registry.register(Migration {
        version: "1.3.0",
        description: "Add dictated_notes metadata table",
        up: migrate_v12_to_v13,
        down: Some(rollback_v13_to_v12),
    });

    let _ = registry.register(Migration {
        version: "1.4.0",
        description: "Add word-level timestamps to dictated_notes",
        up: migrate_v13_to_v14,
        down: Some(rollback_v14_to_v13),
    });

    let _ = registry.register(Migration {
        version: "1.5.0",
        description: "Add transcription_jobs table",
        up: migrate_v14_to_v15,
        down: Some(rollback_v15_to_v14),
    });

    let _ = registry.register(Migration {
        version: "1.6.0",
        description: "Add transcription_status to dictated_notes",
        up: migrate_v15_to_v16,
        down: Some(rollback_v16_to_v15),
    });

    let _ = registry.register(Migration {
        version: "1.7.0",
        description: "Add audit trail columns to actions",
        up: migrate_v16_to_v17,
        down: Some(rollback_v17_to_v16),
    });

    let _ = registry.register(Migration {
        version: "1.8.0",
        description: "Add agent_sessions table",
        up: migrate_v17_to_v18,
        down: Some(rollback_v18_to_v17),
    });

    let _ = registry.register(Migration {
        version: "1.9.0",
        description: "Add reflection_ledger table",
        up: migrate_v18_to_v19,
        down: Some(rollback_v19_to_v18),
    });

    let _ = registry.register(Migration {
        version: "1.10.0",
        description: "Add draft_queue table",
        up: migrate_v19_to_v110,
        down: Some(rollback_v110_to_v19),
    });

    let _ = registry.register(Migration {
        version: "1.11.0",
        description: "Add morning_briefs table",
        up: migrate_v110_to_v111,
        down: Some(rollback_v111_to_v110),
    });

    let _ = registry.register(Migration {
        version: "1.12.0",
        description: "Add has_started_session to agent_sessions",
        up: migrate_v111_to_v112,
        down: Some(rollback_v112_to_v111),
    });

    let _ = registry.register(Migration {
        version: "1.13.0",
        description: "Add cross-project state tables",
        up: migrate_v112_to_v113,
        down: Some(rollback_v113_to_v112),
    });

    let _ = registry.register(Migration {
        version: "1.14.0",
        description: "Add classification column to stitches",
        up: migrate_v113_to_v114,
        down: Some(rollback_v114_to_v113),
    });

    let _ = registry.register(Migration {
        version: "1.15.0",
        description: "Add codex_account_daily_spend table",
        up: migrate_v114_to_v115,
        down: Some(rollback_v115_to_v114),
    });

    let _ = registry.register(Migration {
        version: "1.16.0",
        description: "Add stitch-based forecast columns to capacity_rollup",
        up: migrate_v115_to_v116,
        down: Some(rollback_v116_to_v115),
    });

    let _ = registry.register(Migration {
        version: "1.17.0",
        description: "Add canonical_workspace to stitch_beads",
        up: migrate_v116_to_v117,
        down: Some(rollback_v117_to_v116),
    });

    let _ = registry.register(Migration {
        version: "1.18.0",
        description: "Add bead_commits index tables",
        up: migrate_v117_to_v118,
        down: Some(rollback_v118_to_v117),
    });

    let _ = registry.register(Migration {
        version: "1.19.0",
        description: "Add turn_id to draft_queue",
        up: migrate_v118_to_v119,
        down: Some(rollback_v119_to_v118),
    });

    let _ = registry.register(Migration {
        version: "1.20.0",
        description: "Add audit fields to stitches",
        up: migrate_v119_to_v120,
        down: Some(rollback_v120_to_v119),
    });

    let _ = registry.register(Migration {
        version: "1.21.0",
        description: "Add turn_id to stitches",
        up: migrate_v120_to_v121,
        down: Some(rollback_v121_to_v120),
    });

    let _ = registry.register(Migration {
        version: "1.22.0",
        description: "Add draft persistence fields",
        up: migrate_v121_to_v122,
        down: Some(rollback_v122_to_v121),
    });

    let _ = registry.register(Migration {
        version: "1.23.0",
        description: "Add redacted_words column to dictated_notes",
        up: migrate_v122_to_v123,
        down: Some(rollback_v123_to_v122),
    });

    let _ = registry.register(Migration {
        version: "1.24.0",
        description: "Add vector_index table for semantic dedup persistence",
        up: migrate_v123_to_v124,
        down: Some(rollback_v124_to_v123),
    });

    let _ = registry.register(Migration {
        version: "1.25.0",
        description: "Add agent_turns table for audit trail",
        up: migrate_v124_to_v125,
        down: Some(rollback_v125_to_v124),
    });

    let _ = registry.register(Migration {
        version: "1.26.0",
        description: "Add stitch_percentile_index table",
        up: migrate_v125_to_v126,
        down: Some(rollback_v126_to_v125),
    });

    let _ = registry.register(Migration {
        version: "1.27.0",
        description: "Add fix_patterns table for reusable fix templates",
        up: migrate_v126_to_v127,
        down: Some(rollback_v127_to_v126),
    });

    let _ = registry.register(Migration {
        version: "1.28.0",
        description: "Add redaction_audit table for secret detection events",
        up: migrate_v127_to_v128,
        down: Some(rollback_v128_to_v127),
    });

    let _ = registry.register(Migration {
        version: "1.29.0",
        description: "Add workspace_from/to to stitch_links for cross-workspace blocker resolution",
        up: migrate_v128_to_v129,
        down: Some(rollback_v129_to_v128),
    });

    let _ = registry.register(Migration {
        version: "1.30.0",
        description: "Multi-operator concurrency (§19)",
        up: migrate_v129_to_v130,
        down: Some(rollback_v130_to_v129),
    });

    let _ = registry.register(Migration {
        version: "1.31.0",
        description: "Add UNIQUE constraint on reflection_ledger.content_hash",
        up: migrate_v130_to_v131,
        down: Some(rollback_v131_to_v130),
    });

    let _ = registry.register(Migration {
        version: "1.32.0",
        description: "Add content_blocks table for multimodal input",
        up: migrate_v131_to_v132,
        down: Some(rollback_v132_to_v131),
    });

    let _ = registry.register(Migration {
        version: "1.33.0",
        description: "Add template_id and created_by to fix_patterns",
        up: migrate_v132_to_v133,
        down: Some(rollback_v133_to_v132),
    });

    let _ = registry.register(Migration {
        version: "1.34.0",
        description: "Seed initial risk patterns",
        up: migrate_v133_to_v134,
        down: Some(rollback_v134_to_v133),
    });

    registry
}

// ---------------------------------------------------------------------------
// Rollback functions for minor version migrations
// ---------------------------------------------------------------------------

/// Rollback 1.1.0 → 0.1.0: Drop Stitch service tables
fn rollback_v11_to_v01(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.1.0 → 0.1.0: Dropping Stitch service tables");

    // Drop tables in reverse order of creation (due to foreign keys)
    conn.execute("DROP TABLE IF EXISTS stitch_links", [])?;
    conn.execute("DROP TABLE IF EXISTS stitch_beads", [])?;
    conn.execute("DROP TABLE IF EXISTS stitches", [])?;

    Ok(())
}

/// Rollback 1.2.0 → 1.1.0: Drop Pattern service tables
fn rollback_v12_to_v11(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.2.0 → 1.1.0: Dropping Pattern service tables");

    conn.execute("DROP TABLE IF EXISTS pattern_query_matches", [])?;
    conn.execute("DROP TABLE IF EXISTS pattern_queries", [])?;
    conn.execute("DROP TABLE IF EXISTS pattern_members", [])?;
    conn.execute("DROP TABLE IF EXISTS patterns", [])?;

    Ok(())
}

/// Rollback 1.3.0 → 1.2.0: Drop dictated_notes table
fn rollback_v13_to_v12(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.3.0 → 1.2.0: Dropping dictated_notes table");

    conn.execute("DROP TABLE IF EXISTS dictated_notes", [])?;

    Ok(())
}

/// Rollback 1.4.0 → 1.3.0: Remove transcript_words column
fn rollback_v14_to_v13(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.4.0 → 1.3.0: Removing transcript_words column");

    // SQLite doesn't support DROP COLUMN, need to recreate table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS dictated_notes_backup (
            id TEXT PRIMARY KEY NOT NULL,
            transcript TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            transcription_job_id TEXT
        )",
        [],
    )?;

    conn.execute(
        "INSERT INTO dictated_notes_backup (id, transcript, created_at, transcription_job_id)
         SELECT id, transcript, created_at, transcription_job_id FROM dictated_notes",
        [],
    )?;

    conn.execute("DROP TABLE dictated_notes", [])?;
    conn.execute("ALTER TABLE dictated_notes_backup RENAME TO dictated_notes", [])?;

    Ok(())
}

/// Rollback 1.5.0 → 1.4.0: Drop transcription_jobs table
fn rollback_v15_to_v14(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.5.0 → 1.4.0: Dropping transcription_jobs table");

    conn.execute("DROP TABLE IF EXISTS transcription_jobs", [])?;

    Ok(())
}

/// Rollback 1.6.0 → 1.5.0: Remove transcription_status column
fn rollback_v16_to_v15(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.6.0 → 1.5.0: Removing transcription_status column");

    // SQLite doesn't support DROP COLUMN
    conn.execute(
        "CREATE TABLE IF NOT EXISTS dictated_notes_backup (
            id TEXT PRIMARY KEY NOT NULL,
            transcript TEXT NOT NULL,
            transcript_words TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            transcription_job_id TEXT
        )",
        [],
    )?;

    conn.execute(
        "INSERT INTO dictated_notes_backup (id, transcript, transcript_words, created_at, transcription_job_id)
         SELECT id, transcript, transcript_words, created_at, transcription_job_id FROM dictated_notes",
        [],
    )?;

    conn.execute("DROP TABLE dictated_notes", [])?;
    conn.execute("ALTER TABLE dictated_notes_backup RENAME TO dictated_notes", [])?;

    Ok(())
}

/// Rollback 1.7.0 → 1.6.0: Remove audit trail columns from actions
fn rollback_v17_to_v16(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.7.0 → 1.6.0: Removing audit trail columns from actions");

    // SQLite doesn't support DROP COLUMN
    conn.execute(
        "CREATE TABLE IF NOT EXISTS actions_backup (
            id TEXT PRIMARY KEY NOT NULL,
            project TEXT NOT NULL,
            kind TEXT NOT NULL,
            payload TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    conn.execute(
        "INSERT INTO actions_backup (id, project, kind, payload, created_at)
         SELECT id, project, kind, payload, created_at FROM actions",
        [],
    )?;

    conn.execute("DROP TABLE actions", [])?;
    conn.execute("ALTER TABLE actions_backup RENAME TO actions", [])?;

    // Recreate index
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_actions_project_created ON actions(project, created_at DESC)",
        [],
    )?;

    Ok(())
}

/// Rollback 1.8.0 → 1.7.0: Drop agent_sessions table
fn rollback_v18_to_v17(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.8.0 → 1.7.0: Dropping agent_sessions table");

    conn.execute("DROP TABLE IF EXISTS agent_sessions", [])?;

    Ok(())
}

/// Rollback 1.9.0 → 1.8.0: Drop reflection_ledger table
fn rollback_v19_to_v18(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.9.0 → 1.8.0: Dropping reflection_ledger table");

    conn.execute("DROP TABLE IF EXISTS reflection_ledger", [])?;

    Ok(())
}

/// Rollback 1.10.0 → 1.9.0: Drop draft_queue table
fn rollback_v110_to_v19(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.10.0 → 1.9.0: Dropping draft_queue table");

    conn.execute("DROP TABLE IF EXISTS draft_queue", [])?;

    Ok(())
}

/// Rollback 1.11.0 → 1.10.0: Drop morning_briefs table
fn rollback_v111_to_v110(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.11.0 → 1.10.0: Dropping morning_briefs table");

    conn.execute("DROP TABLE IF EXISTS morning_briefs", [])?;

    Ok(())
}

/// Rollback 1.12.0 → 1.11.0: Remove has_started_session column
fn rollback_v112_to_v111(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.12.0 → 1.11.0: Removing has_started_session column");

    // SQLite doesn't support DROP COLUMN
    conn.execute(
        "CREATE TABLE IF NOT EXISTS agent_sessions_backup (
            id TEXT PRIMARY KEY NOT NULL,
            project TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            closed_at TEXT
        )",
        [],
    )?;

    conn.execute(
        "INSERT INTO agent_sessions_backup (id, project, created_at, closed_at)
         SELECT id, project, created_at, closed_at FROM agent_sessions",
        [],
    )?;

    conn.execute("DROP TABLE agent_sessions", [])?;
    conn.execute("ALTER TABLE agent_sessions_backup RENAME TO agent_sessions", [])?;

    Ok(())
}

/// Rollback 1.13.0 → 1.12.0: Drop cross-project state tables
fn rollback_v113_to_v112(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.13.0 → 1.12.0: Dropping cross-project state tables");

    conn.execute("DROP TABLE IF EXISTS cross_project_state", [])?;
    conn.execute("DROP TABLE IF EXISTS cross_project_state_history", [])?;

    Ok(())
}

/// Rollback 1.14.0 → 1.13.0: Remove classification column from stitches
fn rollback_v114_to_v113(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.14.0 → 1.13.0: Removing classification column from stitches");

    // SQLite doesn't support DROP COLUMN
    conn.execute(
        "CREATE TABLE IF NOT EXISTS stitches_backup (
            id TEXT PRIMARY KEY NOT NULL,
            project TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            status TEXT NOT NULL DEFAULT 'draft',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            closed_at TEXT
        )",
        [],
    )?;

    conn.execute(
        "INSERT INTO stitches_backup (id, project, title, description, status, created_at, updated_at, closed_at)
         SELECT id, project, title, description, status, created_at, updated_at, closed_at FROM stitches",
        [],
    )?;

    conn.execute("DROP TABLE stitches", [])?;
    conn.execute("ALTER TABLE stitches_backup RENAME TO stitches", [])?;

    // Recreate indexes
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_stitches_project_created ON stitches(project, created_at DESC)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_stitches_status ON stitches(status)",
        [],
    )?;

    Ok(())
}

/// Rollback 1.15.0 → 1.14.0: Drop codex_account_daily_spend table
fn rollback_v115_to_v114(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.15.0 → 1.14.0: Dropping codex_account_daily_spend table");

    conn.execute("DROP TABLE IF EXISTS codex_account_daily_spend", [])?;

    Ok(())
}

/// Rollback 1.16.0 → 1.15.0: Remove stitch-based forecast columns
fn rollback_v116_to_v115(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.16.0 → 1.15.0: Removing stitch-based forecast columns");

    // For column removal in SQLite, need to recreate table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS capacity_rollup_backup (
            id TEXT PRIMARY KEY NOT NULL,
            project TEXT NOT NULL,
            date TEXT NOT NULL,
            forecast_5h_capacity_min REAL NOT NULL,
            forecast_24h_capacity_min REAL NOT NULL,
            forecast_7d_capacity_min REAL NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    conn.execute(
        "INSERT INTO capacity_rollup_backup (id, project, date, forecast_5h_capacity_min, forecast_24h_capacity_min, forecast_7d_capacity_min, created_at)
         SELECT id, project, date, forecast_5h_capacity_min, forecast_24h_capacity_min, forecast_7d_capacity_min, created_at FROM capacity_rollup",
        [],
    )?;

    conn.execute("DROP TABLE capacity_rollup", [])?;
    conn.execute("ALTER TABLE capacity_rollup_backup RENAME TO capacity_rollup", [])?;

    // Recreate index
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_capacity_rollup_project_date ON capacity_rollup(project, date)",
        [],
    )?;

    Ok(())
}

/// Rollback 1.17.0 → 1.16.0: Remove canonical_workspace column
fn rollback_v117_to_v116(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.17.0 → 1.16.0: Removing canonical_workspace column");

    // SQLite doesn't support DROP COLUMN
    conn.execute(
        "CREATE TABLE IF NOT EXISTS stitch_beads_backup (
            stitch_id TEXT NOT NULL,
            bead_id TEXT NOT NULL,
            project TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            PRIMARY KEY (stitch_id, bead_id)
        )",
        [],
    )?;

    conn.execute(
        "INSERT INTO stitch_beads_backup (stitch_id, bead_id, project, created_at)
         SELECT stitch_id, bead_id, project, created_at FROM stitch_beads",
        [],
    )?;

    conn.execute("DROP TABLE stitch_beads", [])?;
    conn.execute("ALTER TABLE stitch_beads_backup RENAME TO stitch_beads", [])?;

    // Recreate index
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_stitch_beads_project ON stitch_beads(project)",
        [],
    )?;

    Ok(())
}

/// Rollback 1.18.0 → 1.17.0: Drop bead_commits index tables
fn rollback_v118_to_v117(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.18.0 → 1.17.0: Dropping bead_commits index tables");

    conn.execute("DROP TABLE IF EXISTS bead_commits", [])?;
    conn.execute("DROP TABLE IF EXISTS bead_commit_beads", [])?;

    Ok(())
}

/// Rollback 1.19.0 → 1.18.0: Remove turn_id from draft_queue
fn rollback_v119_to_v118(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.19.0 → 1.18.0: Removing turn_id from draft_queue");

    // SQLite doesn't support DROP COLUMN
    conn.execute(
        "CREATE TABLE IF NOT EXISTS draft_queue_backup (
            id TEXT PRIMARY KEY NOT NULL,
            project TEXT NOT NULL,
            stitch_id TEXT NOT NULL,
            payload TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    conn.execute(
        "INSERT INTO draft_queue_backup (id, project, stitch_id, payload, status, created_at, updated_at)
         SELECT id, project, stitch_id, payload, status, created_at, updated_at FROM draft_queue",
        [],
    )?;

    conn.execute("DROP TABLE draft_queue", [])?;
    conn.execute("ALTER TABLE draft_queue_backup RENAME TO draft_queue", [])?;

    // Recreate indexes
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_draft_queue_project_status ON draft_queue(project, status)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_draft_queue_created ON draft_queue(created_at DESC)",
        [],
    )?;

    Ok(())
}

/// Rollback 1.20.0 → 1.19.0: Remove audit fields from stitches
fn rollback_v120_to_v119(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.20.0 → 1.19.0: Removing audit fields from stitches");

    // SQLite doesn't support DROP COLUMN
    conn.execute(
        "CREATE TABLE IF NOT EXISTS stitches_backup (
            id TEXT PRIMARY KEY NOT NULL,
            project TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            status TEXT NOT NULL DEFAULT 'draft',
            classification TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            closed_at TEXT,
            turn_id TEXT
        )",
        [],
    )?;

    conn.execute(
        "INSERT INTO stitches_backup (id, project, title, description, status, classification, created_at, updated_at, closed_at, turn_id)
         SELECT id, project, title, description, status, classification, created_at, updated_at, closed_at, turn_id FROM stitches",
        [],
    )?;

    conn.execute("DROP TABLE stitches", [])?;
    conn.execute("ALTER TABLE stitches_backup RENAME TO stitches", [])?;

    // Recreate indexes
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_stitches_project_created ON stitches(project, created_at DESC)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_stitches_status ON stitches(status)",
        [],
    )?;

    Ok(())
}

/// Rollback 1.21.0 → 1.20.0: Remove turn_id from stitches
fn rollback_v121_to_v120(_conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.21.0 → 1.20.0: Removing turn_id from stitches");

    // The 1.21.0 migration added turn_id to stitches, but 1.20.0 already has it
    // This is essentially a no-op rollback since the column exists in both versions
    Ok(())
}

/// Rollback 1.22.0 → 1.21.0: Remove draft persistence fields
fn rollback_v122_to_v121(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.22.0 → 1.21.0: Removing draft persistence fields");

    // SQLite doesn't support DROP COLUMN
    conn.execute(
        "CREATE TABLE IF NOT EXISTS draft_queue_backup (
            id TEXT PRIMARY KEY NOT NULL,
            project TEXT NOT NULL,
            stitch_id TEXT NOT NULL,
            payload TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            turn_id TEXT
        )",
        [],
    )?;

    conn.execute(
        "INSERT INTO draft_queue_backup (id, project, stitch_id, payload, status, created_at, updated_at, turn_id)
         SELECT id, project, stitch_id, payload, status, created_at, updated_at, turn_id FROM draft_queue",
        [],
    )?;

    conn.execute("DROP TABLE draft_queue", [])?;
    conn.execute("ALTER TABLE draft_queue_backup RENAME TO draft_queue", [])?;

    // Recreate indexes
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_draft_queue_project_status ON draft_queue(project, status)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_draft_queue_created ON draft_queue(created_at DESC)",
        [],
    )?;

    Ok(())
}

/// Rollback 1.23.0 → 1.22.0: Remove redacted_words column
fn rollback_v123_to_v122(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.23.0 → 1.22.0: Removing redacted_words column");

    // SQLite doesn't support DROP COLUMN
    conn.execute(
        "CREATE TABLE IF NOT EXISTS dictated_notes_backup (
            id TEXT PRIMARY KEY NOT NULL,
            transcript TEXT NOT NULL,
            transcript_words TEXT,
            transcription_status TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            transcription_job_id TEXT
        )",
        [],
    )?;

    conn.execute(
        "INSERT INTO dictated_notes_backup (id, transcript, transcript_words, transcription_status, created_at, transcription_job_id)
         SELECT id, transcript, transcript_words, transcription_status, created_at, transcription_job_id FROM dictated_notes",
        [],
    )?;

    conn.execute("DROP TABLE dictated_notes", [])?;
    conn.execute("ALTER TABLE dictated_notes_backup RENAME TO dictated_notes", [])?;

    Ok(())
}

/// Rollback 1.24.0 → 1.23.0: Drop vector_index table
fn rollback_v124_to_v123(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.24.0 → 1.23.0: Dropping vector_index table");

    conn.execute("DROP TABLE IF EXISTS vector_index", [])?;

    Ok(())
}

/// Rollback 1.25.0 → 1.24.0: Drop agent_turns table
fn rollback_v125_to_v124(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.25.0 → 1.24.0: Dropping agent_turns table");

    conn.execute("DROP TABLE IF EXISTS agent_turns", [])?;

    Ok(())
}

/// Rollback 1.26.0 → 1.25.0: Drop stitch_percentile_index tables
fn rollback_v126_to_v125(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.26.0 → 1.25.0: Dropping stitch_percentile_index tables");

    conn.execute("DROP TABLE IF EXISTS stitch_percentile_index", [])?;
    conn.execute("DROP TABLE IF EXISTS stitch_percentile_index_meta", [])?;

    Ok(())
}

/// Rollback 1.27.0 → 1.26.0: Drop fix_patterns table
fn rollback_v127_to_v126(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.27.0 → 1.26.0: Dropping fix_patterns table");

    conn.execute("DROP TABLE IF EXISTS fix_patterns", [])?;

    Ok(())
}

/// Rollback 1.28.0 → 1.27.0: Drop redaction_audit table
fn rollback_v128_to_v127(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.28.0 → 1.27.0: Dropping redaction_audit table");

    conn.execute("DROP TABLE IF EXISTS redaction_audit", [])?;

    Ok(())
}

/// Rollback 1.29.0 → 1.28.0: Remove workspace_from/to from stitch_links
fn rollback_v129_to_v128(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.29.0 → 1.28.0: Removing workspace_from/to from stitch_links");

    // SQLite doesn't support DROP COLUMN, need to recreate table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS stitch_links_backup (
            from_stitch TEXT NOT NULL,
            to_stitch TEXT NOT NULL,
            kind TEXT NOT NULL CHECK(kind IN ('spawned', 'references')),
            PRIMARY KEY (from_stitch, to_stitch, kind),
            FOREIGN KEY (from_stitch) REFERENCES stitches(id) ON DELETE CASCADE,
            FOREIGN KEY (to_stitch) REFERENCES stitches(id) ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "INSERT INTO stitch_links_backup (from_stitch, to_stitch, kind)
         SELECT from_stitch, to_stitch, kind FROM stitch_links",
        [],
    )?;

    conn.execute("DROP TABLE stitch_links", [])?;
    conn.execute("ALTER TABLE stitch_links_backup RENAME TO stitch_links", [])?;

    // Recreate indexes
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_stitch_links_from ON stitch_links(from_stitch)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_stitch_links_to ON stitch_links(to_stitch)",
        [],
    )?;

    Ok(())
}

/// Rollback 1.30.0 → 1.29.0: Remove multi-operator concurrency features
fn rollback_v130_to_v129(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.30.0 → 1.29.0: Removing multi-operator concurrency features");

    // Drop presence table
    conn.execute("DROP TABLE IF EXISTS presence", [])?;

    // Remove columns from reflection_ledger (SQLite doesn't support DROP COLUMN)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS reflection_ledger_backup (
            id TEXT PRIMARY KEY NOT NULL,
            rule TEXT NOT NULL,
            reason TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'proposed',
            source_stitches TEXT NOT NULL,
            proposed_at TEXT NOT NULL DEFAULT (datetime('now')),
            reviewed_at TEXT,
            archived_at TEXT
        )",
        [],
    )?;

    conn.execute(
        "INSERT INTO reflection_ledger_backup (id, rule, reason, status, source_stitches, proposed_at, reviewed_at, archived_at)
         SELECT id, rule, reason, status, source_stitches, proposed_at, reviewed_at, archived_at FROM reflection_ledger",
        [],
    )?;

    conn.execute("DROP TABLE reflection_ledger", [])?;
    conn.execute("ALTER TABLE reflection_ledger_backup RENAME TO reflection_ledger", [])?;

    // Recreate indexes
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_reflection_ledger_status ON reflection_ledger(status)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_reflection_ledger_proposed_at ON reflection_ledger(proposed_at DESC)",
        [],
    )?;

    Ok(())
}

/// Rollback 1.31.0 → 1.30.0: Remove UNIQUE constraint on reflection_ledger.content_hash
fn rollback_v131_to_v130(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.31.0 → 1.30.0: Removing UNIQUE constraint on reflection_ledger.content_hash");

    // SQLite doesn't support dropping constraints, need to recreate table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS reflection_ledger_backup (
            id TEXT PRIMARY KEY NOT NULL,
            rule TEXT NOT NULL,
            reason TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'proposed',
            source_stitches TEXT NOT NULL,
            proposed_at TEXT NOT NULL DEFAULT (datetime('now')),
            reviewed_at TEXT,
            archived_at TEXT,
            content_hash TEXT NOT NULL DEFAULT '',
            rejection_count INTEGER NOT NULL DEFAULT 0,
            approved_by TEXT,
            approved_at TEXT
        )",
        [],
    )?;

    conn.execute(
        "INSERT INTO reflection_ledger_backup (id, rule, reason, status, source_stitches, proposed_at, reviewed_at, archived_at, content_hash, rejection_count, approved_by, approved_at)
         SELECT id, rule, reason, status, source_stitches, proposed_at, reviewed_at, archived_at, content_hash, rejection_count, approved_by, approved_at FROM reflection_ledger",
        [],
    )?;

    conn.execute("DROP TABLE reflection_ledger", [])?;
    conn.execute("ALTER TABLE reflection_ledger_backup RENAME TO reflection_ledger", [])?;

    // Recreate indexes (without UNIQUE constraint)
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_reflection_ledger_status ON reflection_ledger(status)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_reflection_ledger_proposed_at ON reflection_ledger(proposed_at DESC)",
        [],
    )?;

    Ok(())
}

/// Rollback 1.32.0 → 1.31.0: Drop content_blocks table
fn rollback_v132_to_v131(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.32.0 → 1.31.0: Dropping content_blocks table");

    conn.execute("DROP TABLE IF EXISTS content_blocks", [])?;

    Ok(())
}

/// Rollback 1.33.0 → 1.32.0: Remove template_id and created_by from fix_patterns
fn rollback_v133_to_v132(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.33.0 → 1.32.0: Removing template_id and created_by from fix_patterns");

    // SQLite doesn't support DROP COLUMN
    conn.execute(
        "CREATE TABLE IF NOT EXISTS fix_patterns_backup (
            id TEXT PRIMARY KEY NOT NULL,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            pattern_regex TEXT NOT NULL,
            example_match TEXT NOT NULL,
            fix_instructions TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    conn.execute(
        "INSERT INTO fix_patterns_backup (id, title, description, pattern_regex, example_match, fix_instructions, created_at, updated_at)
         SELECT id, title, description, pattern_regex, example_match, fix_instructions, created_at, updated_at FROM fix_patterns",
        [],
    )?;

    conn.execute("DROP TABLE fix_patterns", [])?;
    conn.execute("ALTER TABLE fix_patterns_backup RENAME TO fix_patterns", [])?;

    Ok(())
}

/// Rollback 1.34.0 → 1.33.0: Drop risk_patterns table
fn rollback_v134_to_v133(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.34.0 → 1.33.0: Dropping risk_patterns table");

    conn.execute("DROP TABLE IF EXISTS risk_patterns", [])?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_compare() {
        assert!(semver_compare("1.2.0") > semver_compare("1.1.0"));
        assert!(semver_compare("2.0.0") > semver_compare("1.99.99"));
        assert!(semver_compare("1.0.1") > semver_compare("1.0.0"));
        assert_eq!(semver_compare("1.2.0"), semver_compare("1.2.0"));
    }

    #[test]
    fn test_registry_pending_migrations() {
        let mut registry = MigrationRegistry::new();

        // Register some migrations
        registry
            .register(Migration {
                version: "1.1.0",
                description: "First migration",
                up: |_| Ok(()),
                down: Some(|_| Ok(())),
            })
            .unwrap();

        registry
            .register(Migration {
                version: "1.2.0",
                description: "Second migration",
                up: |_| Ok(()),
                down: Some(|_| Ok(())),
            })
            .unwrap();

        registry
            .register(Migration {
                version: "2.0.0",
                description: "Major upgrade",
                up: |_| Ok(()),
                down: None, // No rollback for major
            })
            .unwrap();

        // Test pending migrations from 1.0.0
        let pending = registry.pending_migrations("1.0.0").unwrap();
        assert_eq!(pending.len(), 3);
        assert_eq!(pending[0].version, "1.1.0");
        assert_eq!(pending[1].version, "1.2.0");
        assert_eq!(pending[2].version, "2.0.0");

        // Test pending migrations from 1.1.0
        let pending = registry.pending_migrations("1.1.0").unwrap();
        assert_eq!(pending.len(), 2);
        assert_eq!(pending[0].version, "1.2.0");

        // Test no pending from latest
        let pending = registry.pending_migrations("2.0.0").unwrap();
        assert_eq!(pending.len(), 0);
    }

    #[test]
    fn test_registry_can_rollback() {
        let mut registry = MigrationRegistry::new();

        registry
            .register(Migration {
                version: "1.1.0",
                description: "Minor migration",
                up: |_| Ok(()),
                down: Some(|_| Ok(())),
            })
            .unwrap();

        registry
            .register(Migration {
                version: "2.0.0",
                description: "Major upgrade",
                up: |_| Ok(()),
                down: None,
            })
            .unwrap();

        assert!(registry.can_rollback("1.1.0"));
        assert!(!registry.can_rollback("2.0.0"));
        assert!(!registry.can_rollback("1.0.0"));
    }

    #[test]
    fn test_registry_duplicate_version() {
        let mut registry = MigrationRegistry::new();

        registry
            .register(Migration {
                version: "1.1.0",
                description: "First",
                up: |_| Ok(()),
                down: Some(|_| Ok(())),
            })
            .unwrap();

        let result = registry.register(Migration {
            version: "1.1.0",
            description: "Duplicate",
            up: |_| Ok(()),
            down: Some(|_| Ok(())),
        });

        assert!(result.is_err());
    }
}
