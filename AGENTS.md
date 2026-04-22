# AGENTS.md — HOOP repository guide for LLMs

This repository is **HOOP** — a control-plane daemon in the NEEDLE / `br` ecosystem. It is documentation-only at present; no implementation code has been written. If you are an LLM asked to work in this repository, read this file first.

## What HOOP is

A long-lived Rust daemon (planned) that runs on a single operator host and serves as the human-facing interface to a multi-project NEEDLE worker fleet. HOOP reads artifacts across projects (beads, events, conversations, files, costs, capacity) and writes only one thing: it creates beads via `br create` when the operator or the human-interface agent drafts new work.

HOOP does **not** steer NEEDLE workers (no launch / stop / kill / signal / release / reassign). NEEDLE manages itself; HOOP is adjacent.

## Current repository state

- `docs/plan/plan.md` — canonical implementation plan. Read this first; it is always authoritative over anything else.
- `docs/notes/` — prior-art research:
  - `reference-feature-inventory.md`
  - `architecture-patterns.md`
  - `interop-with-needle.md`
  - `orchestrator-problems-and-solutions.md`
- `docs/research/` — reserved for future research material.
- `docs/quickstart.md` — human-facing quickstart (may reference future-state install steps).
- `AGENTS.md` — this file.

There is **no source code yet**. Any Rust crate skeleton, web client scaffolding, or binary artifacts should be created following the plan's phased roadmap, not ad-hoc.

## Key abstractions (must use these terms correctly)

- **Project** — a logical unit as the operator thinks of it. Contains one or more **workspaces** (repos).
- **Workspace** — a single repo on disk with its own `.beads/` queue. Beads are workspace-scoped.
- **Stitch** — a single conversation within a project. Four kinds: `operator` (human ↔ agent chat), `dictated` (voice note), `worker` (NEEDLE worker's CLI session), `ad-hoc` (operator's direct CLI session). Stitches are HOOP's user-facing unit; users don't see beads in normal flow.
- **Pattern** — optional, operator-curated grouping of Stitches toward a goal. May span projects.
- **Bead** — NEEDLE's internal execution unit, managed by `br` (beads_rust). HOOP never touches bead state directly beyond `br create`.
- **Human-interface agent** — persistent Claude Code session (Opus-class) HOOP hosts as the operator's primary conversation partner. Reads everything; writes only by drafting Stitches.
- **Reflection Ledger** — HOOP's learned-rules store. After each closed operator Stitch, the agent proposes rules from repeated patterns; operator approves/rejects; approved rules inject into every subsequent session.

## Non-goals (do not violate these)

Do not plan or build features that do any of these:

1. **Steer NEEDLE workers** — no launch, stop, kill, pause, signal, SIGSTOP, SIGTERM, release-claim, reassign, any action touching a worker process or bead lifecycle.
2. **Mutate bead state beyond creation** — only `br create`. No close, update, depend, claim, release.
3. **Enforce capacity** — HOOP shows utilization (5h/7d Claude Max windows, per-account headroom); never throttles or rotates.
4. **Route work by strand** — strands are worker-immutable (set at launch by model + harness). HOOP displays strand; never predicts or routes by strand.
5. **Expose bead IDs to the operator in normal flow** — users work in Stitches; bead IDs appear only in expert / debug / audit views.
6. **Replace FABRIC** — FABRIC is the passive read-only observer; HOOP is the local host with one write. They link via URL bridge.
7. **Control multiple hosts** — one HOOP, one host. Growth means more projects on the same host, not more hosts.

## Technology conventions

- **Language:** Rust for the daemon (matches NEEDLE direction). Single-binary distribution with embedded static assets for the web UI.
- **Web:** `axum` server, React + Vite + TypeScript + Jotai client, Zod schemas shared via JSON Schema draft-07 + `typify` (Rust) + `json-schema-to-typescript` (TS).
- **Storage:** SQLite (`~/.hoop/fleet.db`) for audit log, Stitch state, Pattern state, Reflection Ledger. Never stores bead state.
- **Bead API:** shell out to `br` (beads_rust by Jeffrey Emanuel / dicklesworthstone). Never open `.beads/beads.db` directly.
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

## How to work here

If asked to make a change:

1. Read `docs/plan/plan.md` end to end before proposing implementation work.
2. Check `docs/notes/` for the problem or feature class — prior-art analysis usually applies.
3. Check which phase the change belongs to. Do not jump ahead — a phase 4 feature should not be started before phase 3 is meaningfully complete.
4. Match terminology (Stitch / Pattern / human-interface agent / Project / Workspace) exactly. Do not use `Mayor`, `polecat`, `swarm`, `convoy`, or Gas Town vocabulary; those were used in earlier drafts and have been deliberately removed.
5. Never suggest features that steer workers, enforce capacity, or route by strand. Refer back to non-goals.

If asked to write code and there is no code yet, create scaffolding that matches the plan's technology decisions; commit with a clear message referencing the plan section it implements.

If asked a question about HOOP, answer from the plan — do not invent semantics. The plan is the source of truth; this file is a synopsis.

## Relationship to sibling projects

- **`br` / beads_rust** — the bead queue. `dicklesworthstone/beads_rust`. HOOP depends on it; shells out to it.
- **NEEDLE** — `jedarden/NEEDLE`. The worker supervision system. HOOP observes NEEDLE's events and writes beads NEEDLE workers will pick up. HOOP does not manage NEEDLE.
- **FABRIC** — `jedarden/FABRIC`. The read-only observability layer. HOOP links to FABRIC via a URL bridge but is not a superset.

## Pointers for specific tasks

- Implementing a new feature → find its phase in the plan, match its deliverables list, read any referenced notes.
- Writing tests → match the Success Criteria for the feature's phase.
- Adding configuration → `~/.hoop/*.yaml`, file-watched, hot-reload, schema-validated, rejected loudly on invalid edits.
- Adding a UI surface → follow the in-flight-isolation rule (streaming content in a separate reactive atom).
- Adding documentation for users → `docs/quickstart.md` or a new `docs/concepts/*.md` file.
- Adding documentation for LLMs → this file.
