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

## [1.0.0] — RETRACTED (never released; phase gates not passed)

This entry was added prematurely and does not represent a completed release.

**Actual state as of 2026-06-28:**
- Phase 0: Complete
- Phase 1: In progress — `cargo build` FAILS (36 compilation errors), `cargo clippy` fails
- Phase 1 CI gate (bead `bf-5mpcl`): Still open
- Phases 2–7: Code exists but has NOT been run or verified

See [`AGENTS.md`](AGENTS.md#current-repository-state) for the authoritative state.
The genesis bead `hoop-ttb` was closed prematurely; the bead tracker is the source of truth.

**Original (fictitious) entry text preserved for reference:**

Initial production release of HOOP — the NEEDLE fleet control plane.

#### All Seven Phases Complete (CLAIMED; NOT ACTUALLY VERIFIED)

**Phase 0: Foundation**
- Documentation scaffolding with comprehensive implementation plan
- Onboarding guide for LLM contributors (AGENTS.md)
- Prior-art research documenting architecture patterns

**Phase 1: Single-host daemon, one workspace, read-only**
- `hoop-daemon` binary with serve, projects add, status, audit commands
- Per-project runtime with event tailer and heartbeat monitor
- Web UI with bead list, worker timeline, and conversation viewer
- SQLite-based fleet.db with audit logging
- Startup audit for br dependency verification

**Phase 2: Multi-project observability**
- Project registry with projects.yaml configuration
- Per-project runtime isolation
- Fleet-of-fleets dashboard with cross-project views
- Cost and capacity visibility (read-only)
- Visual debug panel for transcript archaeology
- Collision and stuck detection
- Stitch and Pattern abstraction layers
- Stitch-Provenance Code Archaeology
- Stitch Net-Diff Viewer
- Cost-Anomaly detection with Fix Lineage

**Phase 3: File browser + artifact preview + multimodal**
- Per-project file browser with tree view
- Text preview with syntax highlighting (syntect)
- Non-text preview (images, PDFs, audio, video)
- Artifact-aware links
- Multimodal input to bead drafts and agent conversations
- Streaming upload with path-sensitive routing
- Dictated Notes capture with voice/screen work support

**Phase 4: Bead creation interface**
- Form-based bead drafting with template library
- Submit flow via br create --json
- Chat-driven drafting with bulk support
- "What Will This Take?" preview
- Already-Started Detection
- Stitch Replay from Failure Point

**Phase 5: Human-interface agent**
- Persistent Claude Code session with LLM-agnostic adapter design
- MCP server with comprehensive tool belt
- Lazy context loading (§3.12)
- Operator ↔ Agent chat pane with notification channel
- Morning Brief (marquee feature #10)
- Cross-Project Stitch Propagation (marquee feature #11)
- Reflection Ledger for learned rules (marquee feature #12)

**Phase 6: Operational polish**
- systemd user service template with auto-restart
- Config hot-reload with file watching
- Log rotation and health endpoints (/healthz, /readyz)
- Daily fleet.db snapshots
- Drop-in binary upgrade flow
- Optional Prometheus /metrics endpoint
- Tailscale-identity-aware authentication
- Performance budget enforcement

**Phase 7: Multi-operator**
- Role-based access control (viewer, drafter)
- Tailscale identity-based role assignment
- Per-operator UI state with optional presence indicators
- Public README with comprehensive documentation

#### Breaking Changes from Pre-Releases
- `hoop launch` removed — Use NEEDLE to manage worker lifecycle
- `hoop salvage` removed — Use `br` directly for bead recovery
- `hoop steer` removed — Capacity management is NEEDLE's concern
- Project registry schema v2 — Run `hoop projects migrate` on first start
- Stitch IDs changed from UUID to composite format — Old IDs remain queryable

#### Affected Schemas
- All schemas migrated to version 1.33.0
- Migration: Run `hoop projects migrate` on first start (automatic)

---

## [0.1.0] — TBD (Never Released)

Initial v0.1 (read-only daemon). See [docs/plan/plan.md](docs/plan/plan.md) §6 Phase 1.
