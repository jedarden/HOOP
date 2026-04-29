# Schema Evolution Policy (§20)

This document defines the Semantic Versioning (SemVer) policy for HOOP schema evolution, including deprecation windows, upgrade gates, and compatibility guarantees.

## Version Format

All durable records carry a `schema_version` field in the format `MAJOR.MINOR.PATCH` (e.g., `1.33.0`).

```json
{
  "schema_version": "1.33.0"
}
```

## SemVer Rules

### Major Version (X.0.0)

**Breaking changes** — No backwards compatibility. One-way migration.

- Examples: Removing fields, changing field types, renaming tables, incompatible data format changes
- **Migration gate:** `hoop migrate major-upgrade --from <N> --confirm` required
- **Rollback:** Not supported (one-way migration)
- **Operator acceptance:** Explicit confirmation required via `--confirm` flag
- **Backup required:** Operator must verify current backup exists before upgrade

### Minor Version (x.Y.0)

**Additive changes** — Backwards compatible. Old records read correctly; new records may have fields old readers ignore.

- Examples: Adding new tables, adding new columns, adding new indexes
- **Migration:** Automatic via `hoop migrate run --confirm` or daemon startup
- **Rollback:** Supported via `hoop migrate rollback <version> --confirm`
- **Deprecation window:** Minor deprecations readable at least one full minor version after introduction

### Patch Version (x.x.Z)

**Bug fixes** — No shape changes.

- Examples: Data fixes, index rebuilds, constraint changes
- **Migration:** Automatic, no operator intervention required
- **Rollback:** Not applicable (no schema shape changes)

## Deprecation Window

### Minor Deprecations

Fields or features deprecated in a minor version remain readable for at least one full minor version cycle.

**Example timeline:**
- `1.27.0`: Field `old_field` deprecated (still readable)
- `1.28.0`: `old_field` still readable, new code uses `new_field`
- `1.29.0`: `old_field` may be removed in schema migration

**Implementation:**
- Deprecated fields are marked in the schema with a `deprecated` annotation
- Migration code preserves deprecated fields for at least one version
- Removal occurs in a subsequent minor version with explicit migration

### Major Deprecations

Major version changes are one-way. The operator consciously accepts the change at the upgrade gate.

## Upgrade Gates

### Minor Version Upgrades

Automatic migration on daemon startup:

```bash
# Check pending migrations
hoop migrate status

# Run manually (optional)
hoop migrate run --confirm
```

**Safety checks:**
- Verifies backup exists before migration
- Creates audit trail entry
- Records migration duration and rows touched
- Rollback available via `hoop migrate rollback <version>`

### Major Version Upgrades

Explicit operator action required:

```bash
# 1. Verify backup
hoop migrate status

# 2. Run major upgrade
hoop migrate major-upgrade --from <CURRENT_MAJOR> --confirm
```

**Safety checks:**
- `--from <N>` flag verifies current major version matches expectation
- `--confirm` flag requires explicit operator confirmation
- Prevents accidental upgrades on wrong database
- Cannot be rolled back (one-way migration)

**Example error messages:**

```
$ hoop migrate major-upgrade --from 1 --confirm
Your data is schema version 1.x; this binary requires 2.x.
Run `hoop migrate major-upgrade --confirm` or restore from a pre-upgrade backup.

$ hoop migrate major-upgrade --from 2 --confirm
hoop migrate major-upgrade: --from 2 does not match current schema version 1.33.0
  This safety check prevents accidental upgrades on the wrong database.
  Omit --from to skip this check, or verify you're targeting the correct database.
```

## Restore Compatibility

`hoop restore` refuses snapshots newer than the running binary:

```bash
$ hoop restore --from s3://bucket/prefix/2.0.0-snapshot
Error: Snapshot schema version 2.0.0 is newer than this binary's 1.33.0.
Upgrade HOOP before restoring this snapshot.
```

**Implementation:**
- `SnapshotManifest::validate()` checks schema version compatibility
- `is_newer_version()` compares version strings semantically
- Prevents accidental restore of incompatible data

## Backup Manifest

Every backup includes a `manifest.json` with schema version:

```json
{
  "snapshot_id": "20240615T040000Z",
  "created_at": "2024-06-15T04:00:00Z",
  "schema_version": "1.33.0",
  "hoop_version": "0.1.0",
  "encryption": "none",
  "fleet_db_key": "backups/20240615T040000Z/fleet.db.zst"
}
```

Restore validates the manifest before taking any destructive action.

## Schema Source of Truth

The `hoop-schema/` directory contains the canonical schema definitions:

```
hoop-schema/
├── schemas/
│   ├── bead.json           # Durable record schemas
│   ├── stitch.json
│   ├── pattern.json
│   └── ...
├── src/lib.rs              # DurableRecord trait
└── build.rs                # Rust code generation via typify
```

**Type generation pipeline:**
```
schemas/*.json → typify → Rust types (OUT_DIR/types.rs)
schemas/*.json → json-schema-to-typescript → types.gen.ts
```

Schema changes must:
1. Update the JSON Schema file in `hoop-schema/schemas/`
2. Add migration in `hoop-daemon/src/migrations.rs`
3. Bump `SCHEMA_VERSION` in `hoop-daemon/src/fleet.rs`
4. Update OpenAPI spec via `hoop-daemon/src/openapi.rs`

## Interaction with br (beads_rust)

HOOP pins a minimum compatible `br` major version. A `br` major bump triggers HOOP's compatibility audit.

**Current requirement:** `br` >= 0.5.0 (as defined in plan §2.1)

When `br` releases a major version bump:
1. HOOP audits compatibility with new `br` API
2. Update minimum version requirement in documentation
3. Update any integration code if needed
4. Test migration path for existing users

## Durable Record Types

All durable records include `schema_version`:

| Record Type | Schema File | Current Version |
|-------------|-------------|-----------------|
| Bead | `bead.json` | 1.0.0 |
| Stitch | `stitch.json` | 1.0.0 |
| Pattern | `pattern.json` | 1.0.0 |
| DictatedNote | `dictated_note.json` | 1.0.0 |
| UiState | `ui_state.json` | 1.1.0 |
| ReflectionLedger | `reflection_ledger.json` | 1.1.0 |
| AuditRow | `audit_row.json` | 1.0.0 |
| ProjectEntry | `project_entry.json` | 1.0.0 |
| HoopConfig | `hoop_config.json` | 1.0.0 |

## Migration Rollback Matrix

| From | To | Rollback? | Command |
|------|-----|-----------|---------|
| 1.29.0 | 1.28.0 | ✅ Yes | `hoop migrate rollback 1.28.0 --confirm` |
| 1.28.0 | 1.27.0 | ✅ Yes | `hoop migrate rollback 1.27.0 --confirm` |
| ... | ... | ... | ... |
| 2.0.0 | 1.29.0 | ❌ No | (Major upgrade, one-way) |

## Release Checklist

Before releasing a new schema version:

- [ ] Update `SCHEMA_VERSION` in `hoop-daemon/src/fleet.rs`
- [ ] Add migration to `hoop-daemon/src/migrations.rs`
- [ ] For minor versions: Add rollback function
- [ ] For major versions: Document breaking changes
- [ ] Update JSON Schema files in `hoop-schema/schemas/`
- [ ] Run `cargo build` to regenerate Rust types
- [ ] Update `hoop-ui/web/src/types.gen.ts`
- [ ] Update OpenAPI spec
- [ ] Test migration on sample database
- [ ] Test rollback (if minor version)
- [ ] Update CHANGELOG.md
- [ ] Document deprecation timeline (if applicable)

## References

- Plan §20: Schema evolution
- Plan §2.1: br dependency, version pinning
- `hoop-daemon/src/migrations.rs`: Migration registry
- `hoop-daemon/src/snapshot_manifest.rs`: Backup manifest validation
- `hoop-daemon/src/fleet.rs`: Schema version constants and upgrade gate
