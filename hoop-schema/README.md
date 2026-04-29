# hoop-schema

Shared data types and JSON schemas for HOOP.

## Overview

This crate provides type-safe data structures used across the HOOP codebase. All types are generated from JSON Schema files in the `schemas/` directory using:

- **Rust types**: Generated via `typify` in `build.rs` → `OUT_DIR/types.rs`
- **TypeScript types**: Generated via `json-schema-to-typescript` in `hoop-ui/web/scripts/codegen-types.mjs` → `hoop-ui/web/src/types.gen.ts`

Both Rust and TypeScript types are generated from the **same source of truth** — the JSON Schema files in `schemas/`. This ensures consistency across the codebase.

## OpenAPI REST API Specification

This crate also includes the OpenAPI 3.1 specification for the HOOP REST API at `openapi.yaml`. The spec is generated from `utoipa` annotations in the daemon's API handlers and provides:

- **API documentation**: Complete description of all REST endpoints
- **Type safety**: Request/response schemas for all endpoints
- **Client generation**: TypeScript clients for UI, agent, and external scripts

### Generating the OpenAPI Spec

The OpenAPI spec is generated from utoipa annotations in `hoop-daemon/src/api_*.rs` files:

```bash
# Generate the spec (writes to hoop-schema/openapi.yaml)
make openapi-generate
# or
cargo run --bin generate_openapi --features openapi > hoop-schema/openapi.yaml
```

### Checking Spec Parity (CI)

Per §20, the OpenAPI spec must be kept in sync with the API handlers. CI enforces this:

```bash
# Check if spec is up to date (used in CI)
make openapi-check
# or
./scripts/check-openapi-spec.sh
```

If the check fails, regenerate the spec and review the changes:
```bash
make openapi-generate
git diff hoop-schema/openapi.yaml
```

### Generating TypeScript Clients

Generate TypeScript types and client code from the OpenAPI spec:

```bash
# Generate TypeScript client (writes to hoop-ui/web/src/api.gen.ts)
make ts-client-generate
# or
./scripts/generate-ts-client.sh
```

This uses `openapi-typescript` to generate type-safe API client code.

### Available Endpoints

The OpenAPI spec covers 36+ API modules:

- **Agent**: Session lifecycle, spawn, disable, switch adapter, send turn
- **Beads**: Create, list, dedup check, vector index
- **Audit**: Query audit log, verify hash chain, redaction audit
- **Attachments**: Serve bead/stitch files
- **Config**: Running config, secrets patterns
- **Content Blocks**: CRUD for content blocks
- **Conversations**: List conversations
- **Cost**: Stitch cost trends and analysis
- **Dictated Notes**: Voice note capture, transcription, redaction
- **Draft Queue**: Create, approve, edit, reject drafts
- **Files**: Browse project files, search, get content
- **Fix Patterns**: Manage fix pattern suggestions
- **Metrics**: Debug state, unknown events
- **Morning Brief**: Daily brief generation
- **Onboarding**: Onboarding prompts and feature usage
- **Orphans**: Detect orphaned files
- **Patterns**: Multi-stitch pattern management
- **Presence**: Operator presence tracking
- **Preview**: Stitch prediction (cost/duration)
- **Prompts**: Prompt library
- **Reflection Ledger**: Reflection proposals and approvals
- **Screen Capture**: Screen capture management
- **Scripts**: Scheduled script management
- **Stitch Decompose**: Preview and submit stitches
- **Stitch Links**: Create/delete stitch relationships
- **Stitch Read**: Read stitch content
- **Stitch Replay**: Replay stitch workspace
- **Stitch Traversal**: Graph traversal (parents, children, closure)
- **Timeline**: Worker timeline queries
- **Tour Project**: Project tour mode
- **Transcription**: Audio transcription jobs
- **UI State**: UI state persistence
- **Unassigned**: Unassigned session management
- **Uploads**: Chunked resumable file upload
- **Diff**: Git diff queries
- **Blame**: Git blame queries
- **Backup**: Trigger backup

### API Documentation

When the daemon is running, interactive API documentation is available:

- **Swagger UI**: `http://localhost:3000/api/docs`
- **ReDoc**: `http://localhost:3000/api/docs/redoc`
- **RapiDoc**: `http://localhost:3000/api/docs/rapidoc`
- **JSON spec**: `http://localhost:3000/api/openapi.json`
- **YAML spec**: `http://localhost:3000/api/openapi.yaml`

## Schema Evolution Policy (§20)

### Version Format

All schemas use **Semantic Versioning (SemVer)** `MAJOR.MINOR.PATCH`:

- **MAJOR**: Breaking changes (requires migration)
- **MINOR**: Backwards-compatible additions
- **PATCH**: Backwards-compatible bug fixes

### Schema Version Field

Every durable record includes a `schema_version` field:

```json
{
  "schema_version": "1.0.0",
  ...
}
```

The version format is validated by the regex: `^\d+\.\d+\.\d+$`

### When to Bump Versions

#### Adding a New Schema (MINOR bump)

When adding a completely new schema type:

1. Create the JSON Schema file in `schemas/`
2. Set `schema_version` to the current global version (e.g., `1.0.0`)
3. Run `cargo build` to regenerate Rust types
4. Run `cd hoop-ui/web && pnpm run codegen` to regenerate TypeScript types
5. Add the new schema to `codegen-types.mjs`'s `schemaOrder` array

**No migration needed** — new schemas don't affect existing data.

#### Adding Optional Fields (MINOR bump)

When adding optional fields to existing schemas:

1. Add the field with `"type": ["<type>", "null"]` or make it optional
2. **Increment MINOR version** (e.g., `1.0.0` → `1.1.0`)
3. Update the schema file's `schema_version` property
4. Regenerate types
5. Add a migration in `hoop-daemon/src/migrations.rs` to handle existing data

**Example:**

```json
// Before: version 1.0.0
{
  "type": "object",
  "properties": {
    "name": { "type": "string" }
  }
}

// After: version 1.1.0 (added optional field)
{
  "type": "object",
  "properties": {
    "name": { "type": "string" },
    "description": { "type": ["string", "null"] }
  }
}
```

#### Breaking Changes (MAJOR bump)

When making breaking changes:

- Removing fields
- Changing field types
- Renaming fields
- Making required fields optional (without default)
- Changing enum values

1. **Increment MAJOR version** (e.g., `1.0.0` → `2.0.0`)
2. Update the schema file's `schema_version` property
3. Add a **one-way migration** in `hoop-daemon/src/migrations.rs` (no `down` function)
4. Document the migration in `CHANGELOG.md`

**Example:**

```json
// Before: version 1.0.0
{
  "type": "object",
  "properties": {
    "full_name": { "type": "string" }
  },
  "required": ["full_name"]
}

// After: version 2.0.0 (split into first/last name)
{
  "type": "object",
  "properties": {
    "first_name": { "type": "string" },
    "last_name": { "type": "string" }
  },
  "required": ["first_name", "last_name"]
}
```

#### Per-Schema Versioning

Some schemas have independent versioning from the global `SCHEMA_VERSION`:

- `UiState`: Currently at `1.1.0` (evolved independently)
- `ReflectionLedger`: Currently at `1.1.0` (evolved independently)

This allows specific types to evolve without bumping the entire schema version.

### Migration Framework

Migrations are defined in `hoop-daemon/src/migrations.rs`:

```rust
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
```

- **MINOR bumps**: Must include `down` migration (rollback-safe)
- **MAJOR bumps**: Omit `down` (one-way migration)

## Adding a New Schema

1. **Create the JSON Schema file** in `schemas/`:

```bash
# schemas/your_type.json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://hoop.dev/schemas/your_type.json",
  "title": "YourType",
  "description": "Your type description",
  "type": "object",
  "required": ["schema_version", "id"],
  "properties": {
    "id": {
      "type": "string",
      "description": "Unique identifier"
    },
    "schema_version": {
      "type": "string",
      "pattern": "^\\d+\\.\\d+\\.\\d+$"
    }
  },
  "schema_version": "1.0.0"
}
```

2. **Add to schema order** in `hoop-ui/web/scripts/codegen-types.mjs`:

```javascript
const schemaOrder = [
  // ... existing schemas
  'your_type.json',
];
```

3. **Regenerate types**:

```bash
# Rust types
cargo build

# TypeScript types
cd hoop-ui/web && pnpm run codegen
```

4. **Add round-trip tests**:

- In `hoop-schema/src/lib.rs` (Rust)
- In `hoop-ui/web/src/schemaDrift.test.ts` (TypeScript)

5. **Generate fixtures** (for TS testing):

```bash
cd hoop-schema
cargo test generate_schema_fixtures -- --ignored
```

## Testing

### Round-Trip Tests

Rust → JSON → Rust round-trip tests ensure serialization/deserialization symmetry:

```bash
cd hoop-schema
cargo test
```

### Schema Drift Tests

TypeScript tests verify Rust-generated fixtures can be parsed by TS types:

```bash
cd hoop-ui/web
pnpm test schemaDrrift.test.ts
```

### Schema Version Validation

All `DurableRecord` types must emit `schema_version` matching the compiled constant:

```bash
cd hoop-schema
cargo test every_durable_record_carries_schema_version
```

## File Structure

```
hoop-schema/
├── schemas/           # JSON Schema files (source of truth)
├── src/
│   ├── lib.rs        # Crate root with tests
│   ├── id_validators.rs
│   └── path_security.rs
├── tests/
│   └── schema_drift.rs  # Fixture generation for TS tests
├── build.rs          # Rust code generation via typify
├── Cargo.toml
└── README.md
```

## Type Generation Pipeline

```
┌─────────────────────────────────────────────────────────────┐
│                    schemas/*.json                           │
│                  (JSON Schema Draft-07)                     │
└────────────────────┬────────────────────────────────────────┘
                     │
        ┌────────────┴────────────┐
        │                         │
        ▼                         ▼
┌───────────────┐         ┌──────────────────┐
│   build.rs    │         │ codegen-types.mjs │
│   (typify)    │         │ (json-schema-to- │
│               │         │    typescript)   │
└───────┬───────┘         └────────┬─────────┘
        │                          │
        ▼                          ▼
┌──────────────────┐      ┌─────────────────┐
│  Rust types      │      │  TypeScript     │
│  (OUT_DIR/types) │      │  types.gen.ts   │
└──────────────────┘      └─────────────────┘
```

## Durable Records

Types that implement `DurableRecord` are persisted to durable storage (SQLite, JSONL, config). These must include `schema_version` and are validated by the `write_versioned!` macro.

Current `DurableRecord` types:

- `AuditRow`
- `Bead`
- `CapacityAccount`
- `DictatedNote`
- `HoopConfig`
- `Pattern`
- `PatternMember`
- `PatternQuery`
- `ReflectionLedger`
- `Stitch`
- `StitchBead`
- `StitchLink`
- `StitchMessage`
- `StitchPreview`
- `UiState`

## References

- Plan §20: Schema migration
- Plan §7: Tech decisions
- `hoop-daemon/src/migrations.rs`: Migration implementation
