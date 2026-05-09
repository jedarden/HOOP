# Schema Evolution (SemVer) Verification — §20

## Task Completion Summary

Bead `hoop-ttb.17` required verification that the Schema evolution (SemVer) system per plan §20 is fully implemented.

## Closing Criteria Verification

### ✅ 1. Every record type emits `schema_version`

All durable record schemas in `hoop-schema/schemas/` include `schema_version`:

| Schema File | Version | Status |
|------------|---------|--------|
| bead.json | 1.0.0 | ✅ |
| stitch.json | 1.0.0 | ✅ |
| pattern.json | 1.0.0 | ✅ |
| dictated_note.json | 1.0.0 | ✅ |
| ui_state.json | 1.1.0 | ✅ |
| reflection_ledger.json | 1.1.0 | ✅ |
| audit_row.json | 1.0.0 | ✅ |
| project_entry.json | 1.0.0 | ✅ |
| hoop_config.json | 1.0.0 | ✅ |

### ✅ 2. Minor deprecation path documented (one full version window)

- `docs/schema-evolution.md` — Comprehensive documentation of SemVer policy
- `hoop-schema/DEPRECATION.md` — Deprecation guidelines
- Policy: Fields deprecated in version X.Y remain readable through X.(Y+1)

### ✅ 3. Major upgrade gate implemented with explicit flag

**Files:**
- `hoop-daemon/src/fleet.rs` — `check_schema_major_gate()`, `run_major_upgrade()`
- `hoop-cli/src/main.rs` — `handle_migrate()` with `MigrateCommands::MajorUpgrade`

**Command:**
```bash
hoop migrate major-upgrade --from <N> --confirm
```

**Safety checks:**
- `--from <N>` verifies current major version matches expectation
- `--confirm` flag requires explicit operator confirmation
- Error message matches plan §20.1 specification

### ✅ 4. Restore refuses newer snapshots

**Files:**
- `hoop-daemon/src/snapshot_manifest.rs` — `validate()`, `is_newer_version()`
- `hoop-cli/src/restore.rs` — Uses `manifest.validate()` before restore

**Behavior:**
```bash
$ hoop restore --from s3://bucket/prefix/2.0.0-snapshot
Error: Snapshot schema version 2.0.0 is newer than this binary's 1.34.0.
Upgrade HOOP before restoring this snapshot.
```

### ✅ 5. Schema source of truth in `hoop-schema/` with Rust + TS codegen

**Directory structure:**
```
hoop-schema/
├── schemas/         # JSON Schema source files
├── src/lib.rs       # DurableRecord trait
├── build.rs         # typify code generation
└── Cargo.toml
```

**Codegen pipeline:**
- Rust types: `typify` (via build.rs)
- TS types: `json-schema-to-typescript` (documented, used in UI)

### ✅ 6. CHANGELOG.md exists and documents schema changes

- `CHANGELOG.md` with [Unreleased] section for schema changes
- Policy: Every schema change MUST add a row documenting affected schemas

## Migration System

**File:** `hoop-daemon/src/migrations.rs`

**Features:**
- MigrationRegistry with up/down functions
- `run_pending_migrations()` for minor version upgrades
- `rollback_migration()` for minor version rollbacks
- `get_migration_status()` for status queries
- Comprehensive audit trail integration

**Current schema version:** 1.34.0 (defined in `hoop-daemon/src/fleet.rs`)

## Interaction with br (beads_rust)

**Plan reference:** §2.1

- HOOP pins minimum compatible `br` major version
- `br` major bump triggers HOOP compatibility audit
- Documented in `docs/schema-evolution.md`

## Conclusion

All closing criteria for plan §20 (Schema evolution / SemVer) are **already implemented and operational**. The system includes:

1. Schema version fields on all durable records
2. Documented deprecation policy with one-version window
3. Major upgrade gate with explicit operator confirmation
4. Restore validation refusing newer snapshots
5. Schema source of truth with code generation
6. Comprehensive migration framework with rollback support

No implementation work was required for this bead — the task was verification-only.
