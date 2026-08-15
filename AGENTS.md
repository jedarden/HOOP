# AGENTS.md — HOOP repository guide for LLMs

This repository is **HOOP** — a control-plane daemon in the NEEDLE / bead-rs ecosystem. If you are an LLM asked to work in this repository, read this file first.

## What HOOP is

A long-lived Rust daemon that runs on a single operator host and serves as the human-facing interface to a multi-project NEEDLE worker fleet. HOOP reads artifacts across projects (beads, events, conversations, files, costs, capacity) and writes only one thing: it creates beads via `bead create` when the operator or the human-interface agent drafts new work.

HOOP does **not** steer NEEDLE workers (no launch / stop / kill / signal / release / reassign). NEEDLE manages itself; HOOP is adjacent.

## Current repository state

**ACTUAL STATE (as of 2026-07-26): Phase 0 complete. Phase 1 in progress. The daemon compiles cleanly — `cargo check --workspace` and `cargo build --workspace` both exit 0 with zero errors/warnings. `cargo test --workspace` does NOT compile: 31 errors in the `hoop-daemon` `lib test` target (stale test fixtures — production structs such as `CapacityMeterConfig`, `DaemonState`, `HoopConfig` gained fields that the test initializers were never updated for), so the unit/integration tests never run. `cargo clippy --workspace -- -D warnings` FAILS with 90 errors across 39 files (28 disallowed `std::fs::write`/`File::create` calls, 13 dead-code, 49 style/complexity lints). `hoop status --json | jq .` PASSES (exit 0, valid JSON). Phase 1 CI gate (`hoop-7ae9fbff`) is OPEN on the test-compile and clippy failures — its comment has the current breakdown. Phases 2–7 code exists but has NOT been run or verified.**

The crate now compiles, but the Phase 1 exit gate is not met (tests do not compile; clippy not clean). The genesis bead (`hoop-ttb`) was closed prematurely; the bead tracker is the authoritative record of what is actually done.

**Do not trust this file's component list as evidence of working software. Run `bead list` and check `docs/plan/plan.md` to know the real state.**

### CRITICAL: Phase sequence lock

**DO NOT implement Phase 2+ features until Phase 1 CI gate (bead `hoop-7ae9fbff`) passes.**

Phases are strictly sequential. Per plan §10: "A phase may not begin until all of the following pass on the same commit for the preceding phase." Partial phase completion does not exist — deliverables move to the next phase intact, not half-finished.

**Phase 1 exit gates:**
- `cargo test` (all unit + integration tests) green
- `cargo clippy -- -D warnings` clean
- `hoop status --json | jq .` succeeds (non-interactive mode verified)
- Phase 1 success criteria have passing automated tests

### Bead workflow

All Phase 1 work beads live in the HOOP workspace (`.beads/`). When working on Phase 1 tasks:
1. `bead claim` the bead to assign it to yourself
2. Complete the work as described
3. `bead close` the bead with a structured retrospective when done
4. Commit your changes before closing — **when you changed files**

A bead does not owe git a commit. Verification-only work, work that turned out to
be already done, and work you found blocked are all legitimate outcomes with
nothing to commit. Record those on the bead with `bead update <id> --notes "..."`.
Never create `notes/<bead-id>.md`, a summary, or a status file to satisfy a commit
requirement — a commit touching only `notes/` or `.beads/` is not shipped work and
NEEDLE's shipped-work gate rejects it.

HOOP code structure (files exist; correctness unverified):

- `hoop-daemon/` — Main Rust daemon with REST API, WebSocket, and agent session management
- `hoop-cli/` — CLI client for project management and status queries
- `hoop-mcp/` — MCP server exposing HOOP's read APIs + one write (`create_stitch`)
- `hoop-schema/` — JSON Schema source + Rust/TS codegen
- `hoop-ui/web/` — React + TypeScript + Jotai web UI with agent chat pane
- `docs/plan/plan.md` — Canonical implementation plan (always authoritative)
- `docs/operations.md` — systemd service management, logs, upgrades, backup runbooks

**Key Phase 5 components:**
- `agent_session.rs` — Persistent agent session manager (spawn, persist, attach-on-restart)
- `agent_adapter.rs` — LLM-agnostic adapter abstraction (Claude Code, Anthropic API, ZAI/GLM)
- `agent_context.rs` — Lazy context index builder (thin index + budget watchdog)
- `morning_brief.rs` — Autonomous daily briefing generator
- `reflection_detector.rs` — Pattern detection for the Reflection Ledger
- `cross_project_propagation.rs` — Sibling project detection for Stitch propagation
- `fleet_notifications.rs` — Notification ring for fleet → agent events
- `api_agent.rs` — REST API for agent lifecycle control
- `api_draft_queue.rs` — Draft preview flow (read-first default)
- `api_reflection_ledger.rs` — Reflection proposal/approval API
- `AgentChatPane.tsx` — Operator ↔ agent chat UI with multimodal input

## Key abstractions (must use these terms correctly)

- **Project** — a logical unit as the operator thinks of it. Contains one or more **workspaces** (repos).
- **Workspace** — a single repo on disk with its own `.beads/` queue. Beads are workspace-scoped.
- **Stitch** — a single conversation within a project. Four kinds: `operator` (human ↔ agent chat), `dictated` (voice note), `worker` (NEEDLE worker's CLI session), `ad-hoc` (operator's direct CLI session). Stitches are HOOP's user-facing unit; users don't see beads in normal flow.
- **Pattern** — optional, operator-curated grouping of Stitches toward a goal. May span projects.
- **Bead** — NEEDLE's internal execution unit, managed by bead-rs. HOOP never touches bead state directly beyond `bead create`.
- **Human-interface agent** — Persistent LLM session (LLM-agnostic: Claude Code / Anthropic API / ZAI+GLM) hosted by HOOP as the operator's primary conversation partner. Reads everything; writes only by drafting Stitches via the preview flow.
- **Reflection Ledger** — HOOP's learned-rules store. After each closed operator Stitch, the agent proposes rules from repeated patterns; operator approves/rejects; approved rules inject into every subsequent session.

## Phase 5: Human-interface agent (v0.5)

### Agent tool belt (canonical)

The human-interface agent has access to these tools via the MCP server:

**Write tools (one write, many reads):**
- `create_stitch(project, title, description, kind, attachments[])` — the ONE write. Creates a draft in the preview queue; operator must approve before any beads are created.

**Read tools:**
- `find_stitches(project, filter)` — List stitches with optional filtering
- `read_stitch(id)` — Get detailed stitch information including messages and linked beads
- `find_beads(project, filter)` — List beads with optional filtering
- `read_bead(id)` — Get detailed bead information
- `read_file(project, path, revision)` — Read a file from a project's repository
- `grep(project, pattern)` — Search for a pattern across files
- `search_conversations(query, project?)` — Search conversation transcripts
- `summarize_project(project)` — Get project activity summary
- `summarize_day()` — Get daily summary across all projects

**Utility tools:**
- `escalate_to_operator(message)` — Send a message to the operator as a UI banner

**Forbidden actions (never available):**
- `launch_fleet`, `stop_fleet`, `release_claim`, `boost_priority`, `close_stitch`, `close_bead`
- If the agent concludes work needs stopping, it MUST escalate to the operator

### Marquee features (Phase 5)

1. **Morning Brief** — Autonomous daily briefing at login or configured time:
   - What closed successfully, what failed (with cost impact)
   - What's stuck, what's anomalous
   - Pre-drafted Stitches for follow-ups (always unsubmitted)
   - **One headline** — the single priority for today

2. **Cross-Project Stitch Propagation** — Sibling project detection:
   - Recognizes when a fix pattern applied in one project has structural siblings
   - Surfaces: "you just closed `fix Calico IP selection` in `iad-acb`. The same pattern exists in `iad-ci`, `rs-manager`..."
   - Always preview; operator accepts per-project or all-at-once

3. **Reflection Ledger** — Learn from repetition:
   - After each closed operator Stitch, scan for repeated patterns
   - Proposals surface in UI for operator approval
   - Approved rules inject into every subsequent agent session
   - Nothing is learned silently

### Agent configuration

Agent settings in `~/.hoop/config.yml`:

```yaml
agent:
  adapter: claude  # claude | anthropic | zai
  model: claude-opus-4-7
  anthropic_api_key: sk-ant-...  # optional for claude adapter
  zai_base_url: https://...      # required for zai adapter
  zai_api_key: ...               # required for zai adapter
  rate_limit_rpm: 50             # optional rate limit
  cost_cap_usd: 100.00           # optional cost cap
  system_prompt_budget_bytes: 4096  # default 4KB

morning_brief:
  window_hours: 24               # how far back to look
  schedule_hour: 7               # when to auto-run
  auto_run_enabled: true         # enable scheduled runs
```

### Agent-off switch

HOOP remains fully functional without the agent. Enabling/disabling:
- UI: "Start Session" / "Disable" buttons in agent chat pane
- API: `POST /api/agent/spawn` / `POST /api/agent/disable`
- Config: Remove `agent` section from `config.yml` to disable at startup

## Vocabulary guard (anti-patterns to avoid)

**Do NOT use these deprecated terms or concepts.** They are from earlier drafts and have been deliberately removed from the HOOP design:

- `Mayor` — removed; HOOP has no mayor role
- `polecat` — removed; old internal name
- `convoy` — removed; not the HOOP pattern
- `Gas Town` — removed; legacy reference
- `swarm` — removed; not the HOOP architecture
- `worker steering` — explicit non-goal; HOOP observes NEEDLE, does not steer workers
- `capacity enforcement` — explicit non-goal; HOOP shows utilization, does not enforce limits

Always use the canonical terminology: **Stitch**, **Pattern**, **Project**, **Workspace**, **human-interface agent**.

## Non-goals (do not violate these)

Do not plan or build features that do any of these:

1. **Steer NEEDLE workers** — no launch, stop, kill, pause, signal, SIGSTOP, SIGTERM, release-claim, reassign, any action touching a worker process or bead lifecycle.
2. **Mutate bead state beyond creation** — only `bead create`. No close, update, depend, claim, release.
3. **Enforce capacity** — HOOP shows utilization (5h/7d Claude Max windows, per-account headroom); never throttles or rotates.
4. **Route work by strand** — strands are worker-immutable (set at launch by model + harness). HOOP displays strand; never predicts or routes by strand.
5. **Expose bead IDs to the operator in normal flow** — users work in Stitches; bead IDs appear only in expert / debug / audit views.
6. **Replace FABRIC** — FABRIC is the passive read-only observer; HOOP is the local host with one write. They link via URL bridge.
7. **Control multiple hosts** — one HOOP, one host. Growth means more projects on the same host, not more hosts.

## Technology conventions

- **Language:** Rust for the daemon (matches NEEDLE direction). Single-binary distribution with embedded static assets for the web UI.
- **Web:** `axum` server, React + Vite + TypeScript + Jotai client, Zod schemas shared via JSON Schema draft-07 + `typify` (Rust) + `json-schema-to-typescript` (TS).
- **Storage:** SQLite (`~/.hoop/fleet.db`) for audit log, Stitch state, Pattern state, Reflection Ledger. Never stores bead state.
- **Bead API:** shell out to the configured `bead` binary. Never open `.beads/beads.db` directly.
- **CI/CD:** Argo Workflows on the `iad-ci` cluster (not GitHub Actions — those are disabled across this environment). Template lives in `jedarden/declarative-config`.
- **Deployment:** single binary installed at `~/.local/bin/hoop`, run as a systemd user service, exposed on a Tailscale hostname. Optional container image as a secondary artifact.

## Event and process invariants

These come from the prior-art research in `docs/notes/` and are locked in:

1. **Events are authoritative; projections are derived.** No `worker_status.json` or similar stale-prone state files.
2. **Liveness = process, never file.** `kill -0 pid && !stopped_record`.
3. **Server is the epoch on reconnect.** Clients do total-replace on `init`.
4. **Dual-identity in schema.** UI-stable id + provider-native session id; explicit `session_bound` event at first join.
5. **Atomic `.tmp` + rename for writes; line-buffered NDJSON reader.**
6. **Never silent-drop unknown events.** Log, emit progress, count.
7. **Lazy context for the human-interface agent.** Thin index by default; tool calls for details on demand.

## When you are stuck: anti-spin rules

**If `bead close <id>` appears to succeed but `bead list` still shows the bead as open:**
Run `bead doctor` and inspect the native checkpoint state. Do not repair or reconstruct the database unless diagnostics explicitly require it. Do NOT commit doc notes as a substitute for closing the bead.

**If a bead is blocked by an infrastructure/environment issue you cannot fix** (e.g., missing system package, broken network, unavailable cluster):
1. Create a bug bead: `bead create --title "describe the blocker" --issue-type bug` with the specific error and what you tried
2. Set it as a dependency: `bead dep add <blocked-bead> <new-blocker-bead> --kind blocks`
3. Stop and let the operator know — do not loop writing docs or fake verification

**Never write a commit claiming "Phase N complete" or "all deliverables verified" unless:**
- `cargo test` passed (run directly or via `nix-shell --run 'cargo test'` on NixOS hosts)
- You ran the actual binary and observed the claimed behavior
- `bead close` succeeded and `bead list` confirms the bead is closed

Fabricated verification commits are worse than no progress — they hide the real state.

## How to work here

If asked to make a change:

1. Read `docs/plan/plan.md` end to end before proposing implementation work.
2. Check `docs/notes/` for the problem or feature class — prior-art analysis usually applies.
3. Check which phase the change belongs to. **Phase sequence lock per plan §10:** phases are strictly sequential; no partial completion. Do not start Phase N+1 features until Phase N exit gates pass (`cargo test`, `cargo clippy`, success criteria tests, non-interactive mode). A phase 4 feature should not be started before phases 1–3 are fully complete and verified.
4. Match terminology (Stitch / Pattern / human-interface agent / Project / Workspace) exactly. Do not use `Mayor`, `polecat`, `swarm`, `convoy`, or Gas Town vocabulary; those were used in earlier drafts and have been deliberately removed. Do not use "worker steering" or "capacity enforcement" — these are explicitly non-goals (HOOP observes NEEDLE; it does not steer workers or enforce capacity).
5. Never suggest features that steer workers, enforce capacity, or route by strand. Refer back to non-goals.

## Build environment (Debian/NixOS — read before running cargo)

The HOOP repository supports two build environments:

### Debian (primary build server)
The primary build server runs Debian 13 (trixie) with all dependencies installed via system packages and local toolchain installations. On this system, `cargo check` / `cargo build` / `cargo test` work directly without any wrapper:

```bash
# All cargo commands work directly
cargo check
cargo test
cargo build --release
```

Required dependencies (already installed):
- rustc 1.95.0 (via ~/.cargo/bin/rustc)
- cargo 1.95.0 (via ~/.local/bin/cargo)
- pkg-config (system package)
- libssl-dev (system package, for openssl-sys)
- node v20.19.2 (system package)
- pnpm 10.33.1 (via ~/.local/bin/pnpm)
- sqlite3 (system package)

### NixOS (development environments)
Some development environments may run NixOS. On NixOS, bare `cargo` commands will fail with an `openssl-sys` / `pkg-config not found` error. Use `nix-shell`:

```bash
# One-shot
nix-shell --run 'cargo check'
nix-shell --run 'cargo test'
nix-shell --run 'cargo build --release'

# Interactive
nix-shell   # then run cargo commands normally
```

`shell.nix` at the repo root provides all required deps (pkg-config, openssl, rustc, node, pnpm).

### Detecting which environment you're on
```bash
# Check OS type
cat /etc/os-release

# If nix-shell is available and you're on NixOS, use it
which nix-shell && cat /etc/os-release | grep -q nixos && echo "Use nix-shell"
```

If asked to write code:
1. Run `cargo check` (or `nix-shell --run 'cargo check'` on NixOS) first to confirm the baseline compiles
2. Check existing implementations — but treat them as unverified until `cargo test` passes
3. Add tests for new functionality (see existing test files for patterns)
4. Commit with a clear message referencing the plan section it implements

If asked a question about HOOP, answer from the plan — do not invent semantics. The plan is the source of truth; this file is a synopsis.

## Relationship to sibling projects

- **bead-rs** — the native bead queue used by NEEDLE. HOOP depends on it and shells out to `bead`.
- **NEEDLE** — `jedarden/NEEDLE`. The worker supervision system. HOOP observes NEEDLE's events and writes beads NEEDLE workers will pick up. HOOP does not manage NEEDLE.
- **FABRIC** — `jedarden/FABRIC`. The read-only observability layer. HOOP links to FABRIC via a URL bridge but is not a superset.

## Pointers for specific tasks

- Implementing a new feature → find its phase in the plan, match its deliverables list, read any referenced notes.
- Writing tests → match the Success Criteria for the feature's phase.
- Adding configuration → `~/.hoop/*.yaml`, file-watched, hot-reload, schema-validated, rejected loudly on invalid edits.
- Adding a UI surface → follow the in-flight-isolation rule (streaming content in a separate reactive atom).
- Adding documentation for users → `README.md` at repo root (quickstart) or a new `docs/concepts/*.md` file for deep-dives.
- Adding documentation for LLMs → this file.
