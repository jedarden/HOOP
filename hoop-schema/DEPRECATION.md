# Schema Deprecation Policy (§20)

## SemVer Versioning

HOOP uses Semantic Versioning (SemVer) for schema changes: `X.Y.Z`

- **Major (X)**: Breaking changes, no backwards compatibility. One-way migration.
- **Minor (Y)**: Additive, backwards-compatible changes. Old readers ignore new fields.
- **Patch (Z)**: Bug fix, no shape change.

## Deprecation Windows

### Minor Deprecations

Fields or features deprecated in a minor version remain readable for **at least one full minor version** after introduction.

**Example:**
- A field is deprecated in `1.33.0`
- The field remains readable in `1.34.0`
- The field may be removed in `1.35.0`

This gives operators a full minor version cycle to migrate away from deprecated features.

### Major Deprecations

Major version changes are **one-way** migrations. The operator must consciously accept these at the upgrade gate by running:

```bash
hoop migrate major-upgrade --confirm
```

Major upgrades cannot be rolled back. Operators should:
1. Create a backup before upgrading
2. Test the upgrade in a non-production environment
3. Verify all integrations work with the new schema

## Schema Version Field

Every durable record carries a `schema_version` field following the pattern `^\d+\.\d+\.\d+$`:

```json
{
  "schema_version": "1.33.0",
  ...
}
```

This field is:
- **Required** for all new records
- **Optional** for legacy audit rows (written before schema versioning)
- **Validated** at serialization time via the `DurableRecord` trait

## Upgrade Paths

### Minor Version Upgrade (e.g., 1.32.0 → 1.33.0)

Automatic on daemon start. Migrations are applied incrementally:

```bash
hoop migrate run --confirm
```

Minor upgrades can be rolled back:

```bash
hoop migrate rollback 1.32.0 --confirm
```

### Major Version Upgrade (e.g., 1.x → 2.0.0)

Requires explicit operator confirmation:

```bash
hoop migrate major-upgrade --confirm
```

Major upgrades:
- Are one-way (no rollback)
- May require data migration
- May break compatibility with older binaries
- Should be tested before production use

## Validation

### Restore Validation

`hoop restore` refuses to restore snapshots newer than the running binary:

```
Snapshot schema version 2.0.0 is newer than this binary's 1.33.0.
Upgrade HOOP before restoring this snapshot.
```

### Runtime Validation

The daemon validates the stored schema version at startup and applies any pending minor migrations automatically.

## br Compatibility

HOOP pins a minimum compatible `br` major version. A `br` major bump triggers HOOP's compatibility audit.

See `hoop-schema/src/version.rs` for the current minimum `br` version.
