# Schema Version Mismatch Investigation (bead bf-mnx8dx)

## Problem Summary

**Error**: `'Unsupported schema version: 1.33.0. Expected 0.1.0–1.28.0'`
**Binary version**: 1.34.0
**Database version**: 1.33.0

## Root Cause

The schema version mismatch is caused by **version inconsistency between HOOP binaries** - specifically, using an older binary after the database has been migrated by a newer one.

### What Happened

1. **Initially**: Database was at schema version 1.28.0
2. **Intermediate step**: A newer HOOP binary (with migrations up to 1.33.0+) was used and migrated the database to 1.33.0
3. **Current state**: An older HOOP binary (with migrations only up to 1.28.0) is now being used
4. **Result**: The old binary sees database at 1.33.0, doesn't recognize this version in its migration paths, and rejects startup

## Schema Versioning System

### Two Separate Schema Version Constants

HOOP has **two different schema version constants** in the codebase:

1. **`hoop-daemon/src/fleet.rs:24`**: `SCHEMA_VERSION = "1.34.0"`
   - Database schema version for DDL and migrations
   - Tracks the structure of tables, columns, indexes

2. **`hoop-schema/src/lib.rs:43`**: `SCHEMA_VERSION = "1.33.0"`
   - Data structure schema version for JSON schemas and typify code generation
   - Tracks the shape of Rust structs and TypeScript interfaces

### Current State in Code

- **Current codebase**: Database schema version is `1.34.0`
- **Current codebase**: Data structure schema version is `1.33.0`
- **Running binary**: Appears to be built from older code that only supported up to database version 1.28.0

## Schema Changes: 1.28.0 → 1.33.0

The migrations between these versions added the following features:

| Version | Migration | Features Added |
|---------|-----------|----------------|
| 1.28.0 | From 1.27.0 | `redaction_audit` table for secret scanning |
| 1.29.0 | From 1.28.0 | `workspace_from` and `workspace_to` columns to `stitch_links` table |
| 1.30.0 | From 1.29.0 | Multi-operator concurrency support |
| 1.31.0 | From 1.30.0 | UNIQUE constraint on `reflection_ledger.content_hash` |
| 1.32.0 | From 1.31.0 | `content_blocks` table for chunked content storage |
| 1.33.0 | From 1.32.0 | `template_id` and `created_by` columns to `fix_patterns` table |

## Where the Compatibility Check is Enforced

### 1. Database Initialization (Main Check)

**Location**: `hoop-daemon/src/fleet.rs:762-808` in `init_fleet_db_at_version()`

When the database is opened during HOOP startup:
- Line 803: `get_schema_version(&conn)` retrieves the current database schema version
- Line 808: `check_schema_major_gate(&version, binary_version)` enforces major version compatibility
- Line 810: `run_migrations(&mut conn, &version)` attempts to run minor version migrations

### 2. Migration Compatibility Check

**Location**: `hoop-daemon/src/fleet.rs:1588-1594` in `run_migrations()`

The migration dispatcher uses a match statement to determine which migrations to run:

```rust
match from_version {
    "0.1.0" => { /* run migrations from 0.1.0 */ }
    "1.1.0" => { /* run migrations from 1.1.0 */ }
    // ... more version cases ...
    "1.28.0" => { /* run migrations from 1.28.0 */ }
    _ => {
        return Err(anyhow::anyhow!(
            "Unsupported schema version: {}. Expected 0.1.0–1.34.0",
            from_version
        ));
    }
}
```

If the database version is not in the match statement, the binary rejects it.

### 3. Major Version Gate

**Location**: `hoop-daemon/src/fleet.rs:3426-3444` in `check_schema_major_gate()`

Prevents startup when the binary's major version exceeds the database's major version:

```rust
pub fn check_schema_major_gate(stored_version: &str, binary_version: &str) -> Result<()> {
    let stored_major = extract_major(stored_version)?;
    let binary_major = extract_major(binary_version)?;

    if binary_major > stored_major {
        anyhow::bail!(
            "Your data is schema version {stored_major}.x; this binary requires {binary_major}.x. \
             Run `hoop migrate major-upgrade --confirm` or restore from a pre-upgrade backup."
        );
    }

    Ok(())
}
```

## Why Binary 1.34.0 Only Supports Up to 1.28.0

The error message "Expected 0.1.0–1.28.0" indicates that:

1. **The binary was built from older code** that only had migration cases up to version 1.28.0
2. **The migration match statement** in that older version didn't include cases for 1.29.0, 1.30.0, 1.31.0, 1.32.0, or 1.33.0
3. **When the binary encounters version 1.33.0**, it falls through to the default case and rejects the database

This is **working as designed** - the system prevents data corruption by refusing to let an older binary open a newer database schema.

## Solution

### Immediate Fix

1. **Build or deploy the current HOOP binary** from the latest code
   - Current code supports migrations from 1.33.0 → 1.34.0
   - Binary version will be 1.34.0 with full migration support

2. **Verify the binary version before deployment**
   ```bash
   hoop --version
   ```

3. **Start HOOP** - it will automatically migrate from 1.33.0 to 1.34.0

### Prevention

**Never downgrade HOOP binaries** after the database has been migrated forward. The schema versioning system is designed to:
- Allow upgrades (old binary → new binary)
- Block downgrades (new binary → old binary)

This prevents data corruption from structural incompatibilities.

## Technical Details

### Schema Version Storage

The schema version is stored in the `schema_version` table in `fleet.db`:

```sql
CREATE TABLE schema_version (
    version TEXT NOT NULL PRIMARY KEY,
    updated_at TEXT NOT NULL
);
```

Current version is retrieved via `get_schema_version()` at `fleet.rs:945`.

### Migration Audit Trail

All schema migrations are logged to the `actions` audit table with:
- `kind = "schema_migrated"`
- `args_json` containing `{from, to, duration_ms, rows_touched}`
- `actor = "hoop:schema:{to_version}"`

This provides a complete audit trail of when and how the database schema evolved.

### Version Policy (§20)

- **Major (X)**: Breaking changes, one-way migration, requires `hoop migrate major-upgrade --confirm`
- **Minor (Y)**: Additive, backwards-compatible changes, auto-migrate on startup
- **Patch (Z)**: Bug fixes, no schema changes

Minor deprecations are readable for at least one full minor version after introduction.

## Files Referenced

- `hoop-daemon/src/fleet.rs:24` - Database SCHEMA_VERSION constant
- `hoop-daemon/src/fleet.rs:762-808` - Database initialization and compatibility checks
- `hoop-daemon/src/fleet.rs:1057-1597` - Migration dispatcher
- `hoop-daemon/src/fleet.rs:3426-3444` - Major version gate
- `hoop-daemon/src/fleet.rs:945-964` - Schema version retrieval
- `hoop-schema/src/lib.rs:43` - Data structure SCHEMA_VERSION constant
- `hoop-daemon/src/migrations.rs` - Migration registry framework

## Related Documentation

- `docs/plan/plan.md §20` - Schema versioning policy
- `docs/operations.md` - Deployment and upgrade procedures
