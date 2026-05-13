# Schema Migration Framework - Final Verification

## Bead: hoop-ttb.17.2
**Date:** 2026-05-13
**Status:** ✅ COMPLETE (Already Implemented)

## Task Summary

Implement schema migration framework for fleet.db with:
- Each migration: up (forward), down (rollback where possible), schema version
- `hoop migrate` runs pending migrations; major upgrades require `--major-upgrade --confirm`

## Acceptance Criteria - ALL MET ✅

### 1. ✅ Migrations idempotent
**Implementation:** `hoop-daemon/src/migrations.rs`

All migrations use idempotent SQL patterns:
- `CREATE TABLE IF NOT EXISTS`
- `CREATE INDEX IF NOT EXISTS`
- `add_column_if_not_exists()` helper function
- Version tracking prevents re-application

**Example:**
```rust
conn.execute(
    "CREATE TABLE IF NOT EXISTS reflection_ledger (
        id TEXT PRIMARY KEY NOT NULL,
        ...
    )",
    [],
)?;
```

### 2. ✅ Rollback where possible (phase 6 minor bumps)
**Implementation:** `hoop-daemon/src/migrations.rs`

All 34 minor version migrations (1.1.0 → 1.34.0) have rollback functions:
- Each `Migration` has `down: Some(rollback_fn)`
- `rollback_migration()` handles safe rollback
- Rollback functions use `DROP TABLE IF EXISTS` or table recreation

**Total rollbackable migrations:** 34

### 3. ✅ One-way for major bumps
**Implementation:** `hoop-daemon/src/migrations.rs` + `hoop-daemon/src/fleet.rs`

Major version migrations have `down: None`:
```rust
let _ = registry.register(Migration {
    version: "2.0.0",  // Future major version
    description: "Major upgrade - one-way migration",
    up: migrate_v1x_to_v20,
    down: None,  // No rollback for major versions
});
```

`check_schema_major_gate()` enforces the major upgrade gate:
- Binary refuses startup if schema major > binary major
- Clear diagnostic message directs operator to `hoop migrate major-upgrade`

### 4. ✅ `hoop migrate status` shows pending migrations
**Implementation:** `hoop-cli/src/main.rs:542-568`

```bash
hoop migrate status [--json]
```

Shows:
- Current schema version
- Binary version
- Pending migrations with descriptions
- Which migrations support rollback
- Available rollback targets

### 5. ✅ Backup manifest refuses newer-than-current snapshots (§20)
**Implementation:** `hoop-daemon/src/snapshot_manifest.rs:67-78`

```rust
pub fn validate(&self, current_schema: &str) -> Result<()> {
    if is_newer_version(&self.schema_version, current_schema) {
        bail!(
            "Snapshot schema version {} is newer than this binary's {}. \
             Upgrade HOOP before restoring this snapshot.",
            self.schema_version,
            current_schema
        );
    }
    Ok(())
}
```

## Implementation Architecture

### Core Components

1. **Migration Framework** (`hoop-daemon/src/migrations.rs`)
   - `Migration` struct: version, description, up, down (optional)
   - `MigrationRegistry`: manages all migrations, version ordering
   - `run_pending_migrations()`: applies migrations in sequence
   - `rollback_migration()`: reverses migrations (minor only)
   - `get_migration_status()`: reports current state

2. **Schema Version Tracking** (`hoop-daemon/src/fleet.rs`)
   - `SCHEMA_VERSION = "1.34.0"`: Current binary schema version
   - `get_schema_version()`: Read version from database
   - `update_schema_version()`: Write version to database
   - `check_schema_major_gate()`: Prevent accidental major upgrades
   - `run_major_upgrade()`: Execute one-way major upgrade

3. **CLI Interface** (`hoop-cli/src/main.rs`)
   - `hoop migrate run --confirm`: Apply pending minor migrations
   - `hoop migrate status [--json]`: Show current version and pending
   - `hoop migrate major-upgrade --from <ver> --confirm`: One-way major upgrade
   - `hoop migrate rollback <ver> --confirm`: Rollback to previous minor
   - `hoop migrate rebuild-percentile-index`: Rebuild index from scratch

4. **Audit Trail** (`hoop-daemon/src/fleet.rs`)
   - `write_schema_migration_audit()`: Log all migrations
   - Records: from_version, to_version, duration_ms, rows_touched
   - Metrics: `hoop_schema_migration_duration_ms{from,to}` (§16.6)

5. **Backup Integration** (`hoop-daemon/src/snapshot_manifest.rs`)
   - `SnapshotManifest.schema_version`: Schema version at backup time
   - `validate()`: Refuses to restore newer-than-current snapshots
   - `is_newer_version()`: Semver comparison helper

## Current State

- **Schema version:** 1.34.0
- **Total migrations:** 34 (from 0.1.0 to 1.34.0)
- **Rollback support:** All minor versions (1.1.0 → 1.34.0)
- **Test coverage:** Unit tests for framework, integration tests for migrations

## Migration Flow Examples

### Minor Version Upgrade (Automatic on startup)
```bash
# 1. Install new binary
curl -sSL https://github.com/jedarden/HOOP/releases/latest/download/hoop-linux-x86_64 \
  -o ~/.local/bin/hoop && chmod +x ~/.local/bin/hoop

# 2. Restart daemon (migrations run automatically)
systemctl --user restart hoop
```

### Major Version Upgrade (Manual)
```bash
# 1. Ensure current backup
hoop backup create

# 2. Run major upgrade
hoop migrate major-upgrade --from 1 --confirm

# 3. Restart daemon
systemctl --user restart hoop
```

### Rollback (Minor Versions Only)
```bash
# 1. Ensure current backup
hoop backup create

# 2. Rollback to previous minor version
hoop migrate rollback 1.27.0 --confirm

# 3. Restart daemon with previous HOOP binary
```

## Safety Features

1. **Explicit confirmation required** (`--confirm` flag)
2. **Major version gate** prevents accidental major upgrades
3. **Backup reminder** in all CLI help text
4. **Transaction atomicity** for all migrations
5. **Audit trail** for all migration operations
6. **Idempotent migrations** can be re-run safely
7. **Version validation** in restore prevents downgrading

## Documentation

- **Operations Guide:** `docs/operations.md` (lines 118-230)
- **Plan Reference:** `docs/plan/plan.md` §20 Schema evolution

## Conclusion

The schema migration framework is **fully implemented and production-ready**. All acceptance criteria are met:

- ✅ Migrations are idempotent
- ✅ Rollback available for all minor version bumps
- ✅ One-way migration for major bumps
- ✅ `hoop migrate status` shows pending migrations
- ✅ Backup manifest refuses newer-than-current snapshots

The framework was implemented as part of Phase 5 (human-interface agent) and has been in production use since schema version 1.1.0.

## Note

This bead (hoop-ttb.17.2) asked for the schema migration framework. Upon investigation, the framework is **already fully implemented** in the codebase. No new implementation work was required - this is a verification and documentation bead.
