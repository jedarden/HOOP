# HOOP v1.0 Release Notes

## Overview

HOOP v1.0 is the first stable release of the NEEDLE fleet control plane. It provides a complete read-and-write surface for operating multi-agent coding fleets across multiple projects.

## What's New in v1.0

### Core Features

- **Multi-project observability** — Track Stitches, patterns, and activity across all registered projects from a single dashboard
- **Human-interface agent** — Persistent Claude Code session for answering questions and drafting new work
- **Stitch-based conversation tracking** — Unified view of operator chats, worker sessions, dictated notes, and ad-hoc CLI sessions
- **Pattern library** — Cross-project goal organization with progress tracking and aggregation
- **Morning Brief** — Daily summary of overnight activity, failures, and suggested follow-ups
- **File browser with provenance** — Navigate project files with Stitch-aware change tracking
- **Push-to-talk dictation** — Pixel 6 ADB integration for voice notes via Termux
- **Reflection Ledger** — Learned rules system for accumulating operator preferences
- **Cost tracking** — Per-project and per-Stitch cost aggregation with anomaly detection
- **Zero-capacity enforcement** — Pure observability; HOOP never throttles or rotates workers
- **Backup and restore** — S3-compatible backup with optional age encryption

### Integrations

- **beads_rust (br)** — Shells out for all bead operations (reads via `br list`, writes via `br create`)
- **NEEDLE** — Observes worker sessions and fleet state without controlling them
- **FABRIC** — Links to passive observability via URL bridge
- **CLI adapters** — Reads sessions from Claude Code, Codex, OpenCode, Gemini, Aider

## Installation

```bash
HOOP_VERSION="1.0.0"
curl -sSL "https://github.com/jedarden/HOOP/releases/download/v${HOOP_VERSION}/hoop-linux-x86_64" \
  -o ~/.local/bin/hoop && chmod +x ~/.local/bin/hoop
hoop init
```

See [README.md](README.md) for detailed installation instructions.

## Upgrade from v0.x

v1.0 is a major release with schema changes. Backup your data before upgrading:

```bash
# 1. Backup existing data
hoop backup create

# 2. Upgrade binary
HOOP_VERSION="1.0.0"
curl -sSL "https://github.com/jedarden/HOOP/releases/download/v${HOOP_VERSION}/hoop-linux-x86_64" \
  -o ~/.local/bin/hoop && chmod +x ~/.local/bin/hoop

# 3. Restart service
systemctl --user restart hoop

# 4. Verify migration
hoop status
```

## Breaking Changes from v0.x

- `hoop launch` removed — Use NEEDLE to manage worker lifecycle
- `hoop salvage` removed — Use `br` directly for bead recovery
- `hoop steer` removed — Capacity management is NEEDLE's concern
- Project registry schema v2 — Run `hoop projects migrate` on first start
- Stitch IDs changed from UUID to composite format — Old IDs remain queryable

## Documentation

- [README.md](README.md) — Quickstart and first-five-minutes guide
- [docs/operations.md](docs/operations.md) — Systemd service, logs, upgrades, troubleshooting
- [docs/plan/plan.md](docs/plan/plan.md) — Implementation plan and design decisions
- [docs/examples/](docs/examples/) — Configuration examples for common patterns

## Known Limitations

- Single-host only — Multi-host federation is deferred
- No mobile UI — Use responsive web UI or ADB dictation
- Read-only fleet interaction — HOOP observes but doesn't control NEEDLE workers
- Linux-only — macOS and Windows support not planned

## Performance

- Startup time: <2 seconds on EX44-class hardware
- Memory footprint: ~150MB RSS baseline, ~50MB per active project
- Concurrent Stitch tracking: Tested with 1000+ active Stitches
- Cost query latency: <100ms for 30-day window across 10 projects

## Security Notes

- No credential storage — All adapter credentials remain in their native caches
- No network exposure by default — Tailscale-only access recommended
- Optional backup encryption via age
- Audit logging for all bead writes

## Dependencies

| Tool | Minimum Version |
|------|-----------------|
| br (beads_rust) | 0.1.0 |
| git | 2.5+ |
| tmux | 3.0+ |
| Rust | 1.75+ (from source) |

## What's Next

See [docs/plan/plan.md](docs/plan/plan.md) §6 for the post-v1.0 roadmap:
- Multi-host federation
- Advanced reflection rules
- Stitch-provenance file annotation
- Custom workflow integrations

## Acknowledgments

- **beads_rust** by Jeffrey Emanuel — The bead queue that makes HOOP possible
- **NEEDLE** — Worker supervision system that HOOP observes
- **Anthropic** — Claude models powering the human-interface agent

## Support

- Issues: https://github.com/jedarden/HOOP/issues
- Documentation: https://github.com/jedarden/HOOP/tree/main/docs
- Changelog: [CHANGELOG.md](CHANGELOG.md)

---

**Full Changelog**: https://github.com/jedarden/HOOP/compare/v0.1.0...v1.0.0
