# Changelog

All notable changes to HOOP documented here. Follows [SemVer](https://semver.org/) (§20).

## Format

Entries are organized by version bump kind:

- **MAJOR** — Breaking changes with no backwards compatibility. Requires `--major-upgrade` flag for migration. One-way schema migration.
- **MINOR** — Additive/backwards-compatible changes. Old records read correctly; new features produce records old readers can safely ignore.
- **PATCH** — Bug fixes without schema shape changes.

## Schema Change Policy (§20)

Every PR that modifies `hoop-schema/schemas/*.json` MUST add a row to the `[Unreleased]` section documenting:
- **Version kind** — MAJOR, MINOR, or PATCH (based on SemVer rules above)
- **Affected schemas** — Which JSON schemas changed
- **Migration note** — Required actions (none for PATCH, data migration for MAJOR, etc.)

CI blocks PRs that modify schemas without a CHANGELOG entry.

---

## [Unreleased]

### MAJOR
<!-- Breaking schema changes (no backwards compatibility) -->
<!-- Format: ### Description (schema_version: X.Y.Z)
     - Affected: schema1.json, schema2.json
     - Migration: Required action for existing data -->

### MINOR
<!-- Additive/backwards-compatible schema changes -->
<!-- Format: ### Description (schema_version: X.Y.Z)
     - Affected: schema1.json
     - Migration: None (additive field) -->

### PATCH
<!-- Bug fixes without schema shape changes -->

---

## [0.1.0] — TBD
Initial v0.1 (read-only daemon). See [docs/plan/plan.md](docs/plan/plan.md) §6 Phase 1.
