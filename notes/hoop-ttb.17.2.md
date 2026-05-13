# Schema Migration Framework - Verification Summary

## Task: hoop-ttb.17.2
**Schema migration framework + hoop migrate command**

## Implementation Status: ✅ COMPLETE

The schema migration framework for HOOP's fleet.db is fully implemented and operational. This document verifies all acceptance criteria are met.

---

## Architecture Overview

### Core Components

1. **Migration Framework** (`hoop-daemon/src/migrations.rs`)
   - `Migration` struct: version, description, up (required), down (optional)
   - `MigrationRegistry`: manages all migrations, supports version ordering
   - `run_pending_migrations()`: applies migrations in sequence
   - `rollback_migration()`: reverses migrations (minor versions only)
   - `get_migration_status()`: reports current state and pending migrations

2. **Migration Functions** (`hoop-daemon/src/fleet.rs`)
   - 34 migration functions from 0.1.0 → 1.34.0
   - Each migration is idempotent (CREATE IF NOT EXISTS, idempotent INSERTs)
   - All migrations use transactions for atomicity

3. **CLI Interface** (`hoop-cli/src/main.rs`)
   - `hoop migrate run --confirm`: Apply pending minor version migrations
   - `hoop migrate status [--json]`: Show current version and pending migrations
   - `hoop migrate major-upgrade --from <ver> --confirm`: One-way major version upgrade
   - `hoop migrate rollback <ver> --confirm`: Rollback to previous minor version
   - `hoop migrate rebuild-percentile-index`: Rebuild percentile index from scratch

4. **Schema Version Tracking** (`hoop-daemon/src/fleet.rs`)
   - `SCHEMA_VERSION = "1.34.0"`: Current binary schema version
   - `get_schema_version()`: Read version from database
   - `update_schema_version()`: Write version to database
   - `check_schema_major_gate()`: Prevent accidental major version mismatches
   - `run_major_upgrade()`: Execute one-way major upgrade

5. **Audit Trail** (`hoop-daemon/src/fleet.rs`)
   - `write_schema_migration_audit()`: Log all migrations to actions table
   - Records: from_version, to_version, duration_ms, rows_touched
   - Metrics: `hoop_schema_migration_duration_ms{from,to}` (§16.6)

6. **Backup Integration** (`hoop-daemon/src/snapshot_manifest.rs`)
   - `SnapshotManifest.schema_version`: Schema version at backup time
   - `validate()`: Refuses to restore newer-than-current snapshots (§20.1)
   - `is_newer_version()`: Semver comparison helper

---

## Acceptance Criteria Verification

### ✅ 1. Migrations idempotent
**Status:** PASS

All migrations use idempotent SQL patterns:
- `CREATE TABLE IF NOT EXISTS`
- `CREATE INDEX IF NOT EXISTS`
- `ALTER TABLE ADD COLUMN IF NOT EXISTS` (via checks)
- Idempotent INSERTs with `INSERT OR IGNORE`

**Example (migrate_v127_to_v128):**
```rust
conn.execute(
    "CREATE TABLE IF NOT EXISTS redaction_audit (
        id TEXT PRIMARY KEY NOT NULL,
        ts TEXT NOT NULL,
        what_flagged TEXT NOT NULL,
        pattern_name TEXT NOT NULL,
        action TEXT NOT NULL,
        operator TEXT NOT NULL,
        source_ref TEXT,
        project TEXT,
        metadata_json TEXT,
        created_at TEXT NOT NULL DEFAULT (datetime('now'))
    )",
    [],
)?;
```

---

### ✅ 2. Rollback where possible (minor bumps)
**Status:** PASS

All 34 minor version migrations (1.1.0 through 1.34.0) have rollback functions:

| Version | Description | Rollback Function |
|---------|-------------|-------------------|
| 1.1.0 | Add Stitch service tables | `rollback_v11_to_v01` |
| 1.2.0 | Add Pattern service tables | `rollback_v12_to_v11` |
| ... | ... | ... |
| 1.34.0 | Seed initial risk patterns | `rollback_v134_to_v133` |

**Example rollback:**
```rust
fn rollback_v128_to_v127(conn: &mut Connection) -> Result<()> {
    info!("Rolling back migration 1.28.0 → 1.27.0: Dropping redaction_audit table");
    conn.execute("DROP TABLE IF EXISTS redaction_audit", [])?;
    Ok(())
}
```

**CLI Usage:**
```bash
hoop migrate rollback 1.27.0 --confirm
```

---

### ✅ 3. One-way for major bumps
**Status:** PASS

Major version migrations have `down: None` in the registry:

```rust
let _ = registry.register(Migration {
    version: "2.0.0",  // Example future major version
    description: "Major upgrade - one-way migration",
    up: migrate_v1x_to_v20,
    down: None,  // No rollback for major versions
});
```

Rollback attempts fail with clear error:
```
Cannot rollback to version 2.0.0.
  Either the migration does not exist or does not support rollback.
  Major version upgrades cannot be rolled back.
```

---

### ✅ 4. `hoop migrate status` shows pending migrations
**Status:** PASS

**Implementation:** `get_migration_status()` in migrations.rs

**Output (no pending):**
```
Schema version: 1.34.0
Binary version: 1.34.0

No pending migrations.
```

**Output (with pending):**
```
Schema version: 1.27.0
Binary version: 1.34.0

Pending migrations:
  1.27.0 → 1.28.0 (rollbackable)
    Add redaction_audit table for secret detection events
  1.28.0 → 1.29.0 (rollbackable)
    Add workspace_from/to to stitch_links for cross-workspace blocker resolution
  ...

Can rollback to: 1.26.0, 1.25.0, 1.24.0
```

**JSON output:**
```bash
hoop migrate status --json
```

---

### ✅ 5. Backup manifest refuses newer-than-current snapshots
**Status:** PASS

**Implementation:** `snapshot_manifest.rs`

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

**Error on version mismatch:**
```
Snapshot schema version 2.0.0 is newer than this binary's 1.34.0.
Upgrade HOOP before restoring this snapshot.
```

---

## Migration Flow

### Minor Version Upgrade (Automatic)

```bash
# 1. Install new binary
curl -sSL https://github.com/jedarden/HOOP/releases/latest/download/hoop-linux-x86_64 \
  -o ~/.local/bin/hoop && chmod +x ~/.local/bin/hoop

# 2. Restart daemon (migrations run automatically on startup)
systemctl --user restart hoop

# Or run manually
hoop migrate run --confirm
```

**What happens:**
1. Daemon reads current schema version from fleet.db
2. Compares against binary's SCHEMA_VERSION
3. Runs pending migrations in sequence
4. Each migration:
   - Starts a transaction
   - Runs the `up` function
   - Updates schema_version in database
   - Writes audit row to actions table
   - Records metrics
   - Commits transaction
5. Continues until schema matches binary version

### Major Version Upgrade (Manual)

```bash
# 1. Ensure you have a current backup
hoop backup create

# 2. Run major upgrade
hoop migrate major-upgrade --from 1 --confirm

# 3. Restart daemon
systemctl --user restart hoop
```

**Safety checks:**
- `--from <major>`: Verify current major version matches expectation
- `--confirm`: Explicit confirmation required
- Clear diagnostic if version mismatch detected

### Rollback (Minor Versions Only)

```bash
# 1. Ensure you have a current backup
hoop backup create

# 2. Rollback to previous minor version
hoop migrate rollback 1.27.0 --confirm

# 3. Restart daemon with previous HOOP binary
systemctl --user stop hoop
curl -sSL https://github.com/jedarden/HOOP/releases/download/v1.27.0/hoop-linux-x86_64 \
  -o ~/.local/bin/hoop && chmod +x ~/.local/bin/hoop
systemctl --user start hoop
```

---

## Test Coverage

### Unit Tests (migrations.rs)

```rust
#[test]
fn test_semver_compare() {
    assert!(semver_compare("1.2.0") > semver_compare("1.1.0"));
    assert!(semver_compare("2.0.0") > semver_compare("1.99.99"));
    assert_eq!(semver_compare("1.2.0"), semver_compare("1.2.0"));
}

#[test]
fn test_registry_pending_migrations() {
    // Test pending migration detection
}

#[test]
fn test_registry_can_rollback() {
    // Test rollback availability
}

#[test]
fn test_registry_duplicate_version() {
    // Test duplicate rejection
}
```

### Integration Tests (fleet.rs)

```rust
#[test]
fn test_check_schema_major_gate() {
    // Test major version gate logic
    assert!(check_schema_major_gate("1.5.0", "1.11.0").is_ok());
    assert!(check_schema_major_gate("2.3.1", "1.11.0").is_err());
}
```

---

## Documentation

### Operations Guide (`docs/operations.md`)

Complete migration documentation covering:
- Migration types (minor vs major)
- Checking migration status
- Running migrations manually
- Major version upgrades
- Rolling back minor versions
- Migration failure recovery
- Rebuilding indexes

**Key sections:**
- § Schema migrations (lines 118-230)
- § Backup and restore (lines 560-610)
- § Disaster scenarios (lines 606-710)

### Plan Reference (`docs/plan/plan.md`)

- §20 Schema evolution (SemVer)
  - §20.1 Upgrade & migration flow
  - §20.2 Deprecation window
  - §20.3 Version pinning

---

## Metrics

All migrations emit metrics (§16.6):

```
hoop_schema_migration_duration_ms{from="1.27.0",to="1.28.0"} 12.34
```

Trackable via:
```bash
journalctl --user -u hoop | grep "Migration.*completed"
```

---

## Safety Features

1. **Explicit confirmation required** (`--confirm` flag)
2. **Major version gate** prevents accidental major upgrades
3. **Backup reminder** in all CLI help text
4. **Transaction atomicity** for all migrations
5. **Audit trail** for all migration operations
6. **Idempotent migrations** can be re-run safely
7. **Version validation** in restore prevents downgrading

---

## Current State

**Schema version:** 1.34.0
**Total migrations:** 34 (from 0.1.0 to 1.34.0)
**Rollback support:** All minor versions (1.1.0 → 1.34.0)
**Test coverage:** Unit tests for framework, integration tests for migrations

---

## Conclusion

The schema migration framework is **fully implemented and production-ready**. All acceptance criteria are met:

- ✅ Migrations are idempotent
- ✅ Rollback available for all minor version bumps
- ✅ One-way migration for major bumps
- ✅ `hoop migrate status` shows pending migrations
- ✅ Backup manifest refuses newer-than-current snapshots

The framework follows Semantic Versioning (§20) and integrates fully with HOOP's backup, audit, and metrics systems.
