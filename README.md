# HOOP

**Status: Pre-release — Phase 1 in progress. The daemon does not currently compile ([`cargo build` fails](AGENTS.md#current-repository-state)). Track Phase 1 progress at bead [`bf-5mpcl`](.beads/issues.jsonl).**

Operator observability and control plane for NEEDLE worker fleets. HOOP reads everything — bead state, CLI session transcripts, worker heartbeats, cost data — and surfaces it in a web UI and REST/WebSocket API. It writes exactly one thing: creating new beads via `br create`.

---

## What HOOP does

HOOP keeps the work queue *taut* by surfacing what is stale, stuck, or missing — and prompting the operator to act — rather than acting autonomously. It is passive observability that creates pressure for action, not active control.

**What HOOP reads:**
- Bead queues (`.beads/` in every registered workspace) via `br` CLI
- CLI session transcripts from Claude Code, Codex, OpenCode, Gemini, and Aider (JSONL format)
- Worker heartbeats (`.beads/heartbeats.jsonl`) appended by NEEDLE workers every 10 seconds
- Worker event logs (`.beads/events.jsonl`) on claim, dispatch, complete, and fail

**What HOOP writes:**
- `br create` — the only mutation HOOP performs. The agent drafts a bead, you approve, HOOP calls `br create`. Nothing else.

**What HOOP never does:**
- Launch or stop NEEDLE workers
- Mutate bead state beyond creation (no claims, releases, completions, or priority changes)
- Enforce capacity limits or rotate credentials
- Act without operator approval on the write path

---

## Features

- **Real-time fleet dashboard** — web UI (React/Vite + WebSocket) showing all projects, beads, session transcripts, and worker health in real time
- **Session tailer** — parses JSONL session transcripts from Claude Code, Codex, OpenCode, Gemini, and Aider; links worker sessions to specific beads via dispatch tags
- **Worker health monitoring** — heartbeat monitor classifies workers as Live (fresh heartbeat + PID alive), Hung (PID alive but heartbeat stale), or Dead (PID gone)
- **Stuck detection** — three-timer system: idle silence timeout (3 min default), max runtime ceiling (1 h), content-seen grace (10 min after content appears)
- **Saturation alerting** — fires when an account hits 80% of 5-hour or 7-day usage windows; clears at 75% (5% hysteresis); informational only, never throttles
- **Human-interface agent** — persistent LLM session hosted by HOOP as the operator's conversation partner; reads everything, creates draft beads for approval
- **Tamper-evident audit log** — SHA-256 hash chain on all auditable events; verifiable via `hoop audit verify`
- **Snapshot backups** — S3-compatible backup/restore for the fleet database
- **MCP server** — read-only tools for querying stitches, beads, files, conversations; one write tool (`create_stitch`) requires operator approval
- **Self-contained** — single binary serves embedded web UI; SQLite at `~/.hoop/fleet.db`; no external database required

---

## Key concepts

| Term | What it means |
|------|--------------|
| **Project** | A logical unit you care about — may contain one or more repos. Registered in `~/.hoop/projects.yaml`. You control the list. |
| **Workspace** | A single repo on disk with its own `.beads/` queue. A project can span multiple workspaces. |
| **Stitch** | HOOP's conversation unit: operator chat, voice/dictated note, NEEDLE worker CLI session, or ad-hoc terminal session. Stitches decay by inactivity rather than closing explicitly. |
| **Pattern** | An optional cross-project grouping of Stitches toward a shared goal. Useful for epics and long-running initiatives. |
| **Bead** | NEEDLE's execution unit. HOOP never mutates bead state beyond `br create`. |
| **Human-interface agent** | HOOP's LLM conversation partner for the operator. Reads everything; writes only by drafting Stitches via preview/approval flow. |
| **Reflection Ledger** | Learned rules store. After each Stitch closes, the agent proposes patterns; approved rules inject into future sessions. |

---

## CLI reference

| Subcommand | Description |
|-----------|-------------|
| `serve` | Start the daemon (web UI + WebSocket + REST API) |
| `projects add/scan/list/remove/show` | Project registry management |
| `status [project] [--json]` | Fleet/bead/cost overview |
| `audit check [--strict]` | Startup audit (verifies `br`, `tmux`, CLI adapters, disk space) |
| `audit verify` | Verifies audit log SHA-256 hash chain |
| `new <project> [--dry-run]` | Draft and submit a new Stitch |
| `agent` | Attach to or start the human-interface agent |
| `install-systemd` | Writes `~/.config/systemd/user/hoop.service` |
| `backup create/list/delete/restore` | S3-compatible snapshot backups |
| `migrate run/status/rollback` | SQLite schema migrations |
| `script run/list/create/edit/delete` | Operator scripts |
| `config diff` | Show live vs on-disk config diff |
| `risk-patterns list/add/remove/edit` | Risk pattern management |
| `skills list/add/remove/edit` | Agent-invocable skill management |
| `pattern list/create/add/remove` | Pattern (goal grouping) management |
| `init` | First-time setup wizard |

**Global flags:**

| Flag | Description |
|------|-------------|
| `--no-interactive`, `-y` | Suppress all interactive prompts (for CI/CD and automation) |

### `--no-interactive` flag details

The `--no-interactive` flag (short form: `-y`) controls whether the CLI prompts for user confirmation. When set to `true`, all interactive prompts are suppressed. This is essential for:

- CI/CD pipelines and automation scripts
- Non-interactive environments (e.g., cron jobs)
- Batch operations requiring automatic confirmation

**How `global = true` works:**

The clap `global = true` attribute makes the flag available to all subcommands automatically, without redefining it in each subcommand. Clap propagates the flag value through the entire command tree.

**Usage with subcommands:**

Because of `global = true`, the flag can be specified at any position:

```bash
# Before the subcommand
hoop --no-interactive projects remove my-project --confirm

# After the subcommand
hoop projects remove my-project --no-interactive --confirm

# With the short alias
hoop -y projects remove my-project --confirm

# For scan operations (auto-confirms without --confirm)
hoop --no-interactive scan /path/to/projects
```

**Safety pattern for destructive operations:**

Commands that have interactive prompts follow this pattern:

1. **Safe operations** (e.g., `scan`): Auto-proceed when `--no-interactive` is set
2. **Destructive operations** (e.g., `remove`, `restore`): Require explicit `--confirm` flag when `--no-interactive` is set to prevent accidental data loss

Example destructive operation:
```bash
# This will error with a helpful message
hoop --no-interactive projects remove my-project

# This is the correct usage
hoop --no-interactive projects remove my-project --confirm
```

**Commands that respect `--no-interactive`:**

- `scan` / `projects scan` — Auto-registers all discovered workspaces
- `remove` / `projects remove` — Requires `--confirm` when non-interactive
- `restore` — Requires `--confirm` when non-interactive (destructive DB op)

**Commands that explicitly reject `--no-interactive`:**

- `init` — The init wizard requires interaction and explicitly errors when `--no-interactive` is set, directing the user to run without the flag

Run `hoop --help` for full documentation.

---

## NEEDLE integration

HOOP integrates with NEEDLE via four passive hooks. Three are pure reads; one is a write that requires operator approval.

**1. Dispatch tag**

NEEDLE prefixes the first user message in each worker session with `[needle:<worker>:<bead>:<strand>]`. HOOP extracts this tag to join transcript files to specific beads, enabling the Stitch timeline to link worker activity directly to the work item that drove it.

**2. Event tap**

Workers append JSONL to `.beads/events.jsonl` on claim, dispatch, complete, and fail. HOOP watches this file to track bead lifecycle transitions in real time without polling `br`.

**3. Worker heartbeat**

Each worker appends a JSON line to `.beads/heartbeats.jsonl` every 10 seconds. HOOP uses this to classify worker health:
- **Live** — fresh heartbeat and PID is alive
- **Hung** — PID is alive but heartbeat is stale (past the stuck-detection threshold)
- **Dead** — PID is gone

**4. Stitch label inheritance**

When the human-interface agent calls `br create` to spawn follow-up work, it copies `stitch:*` labels onto the new bead. This means NEEDLE workers picking up that bead carry the Stitch lineage forward, keeping the transcript chain intact across multi-step work.

---

## MCP server

HOOP exposes an MCP server for use by external LLM clients and tool integrations.

**Read tools** (no approval required):

| Tool | What it returns |
|------|----------------|
| `find_stitches` | Search Stitches by project, status, date range, or text |
| `read_stitch` | Full detail for a single Stitch including transcript |
| `find_beads` | Query bead state across registered workspaces |
| `read_bead` | Full bead detail including history |
| `read_file` | File content from a registered workspace |
| `grep` | Text search across project files |
| `search_conversations` | Full-text search over all session transcripts |
| `summarize_project` | Agent-generated summary of a project's current state |
| `summarize_day` | Agent-generated daily summary across all projects |

**Write tool** (operator approval required):

| Tool | Behavior |
|------|---------|
| `create_stitch` | Drafts a new Stitch and presents it for operator approval before any bead is created. Returns a preview; never writes without confirmation. |

**Explicitly forbidden tools** — these return hard errors if called:

`launch_fleet`, `stop_fleet`, `release_claim`, `boost_priority`, `kill_worker`, `pause_worker`

---

## CI/CD

HOOP uses automated CI/CD via Argo Workflows on the `iad-ci` cluster. The pipeline runs automatically on push to the `main` branch.

**Trigger:** GitHub webhook → Argo Events → Sensor → Workflow submission

**Pipeline includes:**
- Rust build, test, and lint (clippy, rustfmt)
- Web UI build and test (pnpm, Playwright E2E)
- Security audit (cargo audit, pnpm audit, trivy)
- OpenAPI spec validation
- Schema drift detection
- Code coverage (80% threshold)
- Load tests (conditional)
- Docker image build and push
- GitHub release creation
- Image security scanning

**Manual trigger:** See [`docs/webhook-setup.md`](docs/webhook-setup.md) for manual workflow execution instructions.

**Setup:** The webhook infrastructure is deployed in the cluster. To enable automatic triggers, configure the GitHub repository webhook as documented in [`docs/webhook-setup.md`](docs/webhook-setup.md).

## Building

HOOP is a Rust workspace with five crates:

| Crate | Purpose |
|-------|---------|
| `hoop-cli` | CLI binary (clap); delegates to hoop-daemon for serve/audit/fleet |
| `hoop-daemon` | Core daemon: axum HTTP/WS server, business logic, SQLite, file watchers |
| `hoop-schema` | JSON Schema source and codegen for Rust and TypeScript; OpenAPI spec |
| `hoop-ui` | Embedded web UI static assets (React/Vite); served by hoop-daemon via rust-embed |
| `hoop-mcp` | MCP server exposing read APIs and create_stitch |

**Prerequisites:** Rust stable toolchain, Node.js (for hoop-ui).

```bash
# Build the web UI first (embedded into the daemon binary)
cd hoop-ui && npm install && npm run build && cd ..

# Build all crates
cargo build --release

# Run tests
cargo test
```

The release binary is at `target/release/hoop`. Install to PATH:

```bash
cp target/release/hoop ~/.local/bin/
```

---

## Running

```bash
# First-time setup wizard (verifies deps, registers projects, optional agent setup)
hoop init

# Start the daemon
hoop serve
# Web UI available at http://127.0.0.1:3000 by default

# CLI status overview
hoop status

# Verify prerequisites
hoop audit check

# Install as a systemd user service (auto-restart on failure)
hoop install-systemd
systemctl --user enable --now hoop
```

When HOOP is stopped, nothing else is affected. NEEDLE keeps running. Workers keep writing heartbeats and events. The next time HOOP starts it rebuilds its view entirely from disk. HOOP is a convenience layer, not a dependency for worker operation.

---

## Configuration

HOOP reads `~/.hoop/config.yml` on startup and hot-reloads most settings on file change. The exception is `server.bind_addr`, which requires a restart.

**Key settings:**

```yaml
server:
  bind_addr: "127.0.0.1:3000"  # Change to "0.0.0.0:3000" to expose on the network [restart required]

agent:
  adapter: claude              # claude | codex | opencode | gemini | aider
  model: claude-sonnet-4-6
  anthropic_api_key: ""        # or set ANTHROPIC_API_KEY env var
  cost_cap_usd: 5.0            # per-session cost ceiling for the human-interface agent

ui:
  theme: auto                  # auto | light | dark | solarized-light | solarized-dark

metrics:
  enabled: false               # set true to expose Prometheus endpoint
  port: 9091

voice:
  whisper_model_path: ~/.hoop/models/ggml-base.en.bin
  hotkey: "ctrl+shift+space"
  max_recording_seconds: 300

backup:
  endpoint: ""                 # S3-compatible endpoint URL
  bucket: ""
  prefix: hoop/
  schedule: "0 4 * * *"       # cron expression
  retention_days: 30
  encryption: false            # set true and provide HOOP_BACKUP_AGE_KEY env var

audit:
  retention_days: 90
  hash_chain: true             # SHA-256 chain on all auditable events

reflection:
  enabled: true
  detection_threshold: 3       # repeated instructions before proposing a rule
```

View the live config vs on-disk diff at any time:

```bash
hoop config diff
```

---

## Documentation map

| File | Purpose |
|------|---------|
| `README.md` (this file) | Overview, concepts, CLI reference, building, running, configuration |
| [`AGENTS.md`](AGENTS.md) | Repository guide for LLM contributors: terminology, non-goals, conventions |
| [`CHANGELOG.md`](CHANGELOG.md) | Version history (Keep-a-Changelog / SemVer) |
| [`docs/webhook-setup.md`](docs/webhook-setup.md) | GitHub webhook configuration for automatic CI/CD execution |
| [`docs/operations.md`](docs/operations.md) | Systemd service, logs, upgrades, backups, migrations |
| [`docs/troubleshooting.md`](docs/troubleshooting.md) | Common failures mapped to `hoop audit` output with recovery steps |
| [`docs/plan/plan.md`](docs/plan/plan.md) | Full implementation plan: vision, principles, architecture, phased roadmap |
| [`docs/notes/`](docs/notes/) | Prior-art research, architecture patterns, NEEDLE interop notes |
| [`docs/cleanup-workflow-guide.md`](docs/cleanup-workflow-guide.md) | Comprehensive guide for HOOP test process cleanup |
| [`docs/cleanup-examples.md`](docs/cleanup-examples.md) | Practical examples and test scenarios for cleanup workflow |
| [`docs/test-process-cleanup-patterns.md`](docs/test-process-cleanup-patterns.md) | Detailed analysis of 27 process cleanup patterns |

---

## Sibling projects

- [`dicklesworthstone/beads_rust`](https://github.com/dicklesworthstone/beads_rust) — `br`, the bead queue. HOOP shells out to it for all bead operations.
- [`jedarden/NEEDLE`](https://github.com/jedarden/NEEDLE) — the worker supervision system. HOOP observes NEEDLE's events and creates beads that NEEDLE workers pick up.
- [`jedarden/FABRIC`](https://github.com/jedarden/FABRIC) — passive read-only observability dashboard. HOOP links to FABRIC via a URL bridge.
