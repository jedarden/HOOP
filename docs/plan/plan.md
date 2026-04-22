# HOOP implementation plan

**Status:** Draft v4 — 2026-04-22
**Scope:** A review-and-request surface for a single long-lived host that holds many repos, many NEEDLE fleets, and many native-CLI conversations. HOOP reads artifacts to answer operator questions and writes beads when new work needs to happen. **HOOP does not steer NEEDLE workers.** Workers live and die under NEEDLE's authority; HOOP observes them and puts items into their queue.

**Lineage.**
- **The prior-art reference ADE ([notes/](../notes/))** — cross-agent conversation aggregation, visual debug of prompts/tool-calls/state transitions, git-worktree isolation as the unit of artifact safety, single unified event union across providers. HOOP adopts these as read-side patterns against the NEEDLE substrate.

---

## 1. Vision

HOOP is the operator's pane of glass and conversational handle. On a single long-lived host like this Hetzner EX44:

- A dozen repos live on disk, each at its own path, each with its own `.beads/` queue.
- Several NEEDLE fleets run at once, each managing its own workers as tmux sessions — **HOOP does not manage them.** NEEDLE and its workers are an independent system; HOOP is adjacent.
- The operator runs ad-hoc `claude` / `codex` / `opencode` / `gemini` / `aider` sessions in their own terminals.
- The operator wants answers: "what did the kalshi-weather fleet do overnight?", "why did bead bd-3qvi fail?", "is the ibkr-mcp review backlog growing?", "draft a bead for fixing the Calico IP selection on iad-acb."

HOOP answers those questions by reading artifacts across every project — bead state, event logs, conversation transcripts, files on disk, diffs, cost data, capacity utilization. When an answer naturally leads to new work, HOOP writes a bead into the target project's `.beads/` via `br create`. That's the only write action HOOP ever takes. The NEEDLE workers already running will claim it when they get to it.

The unit of growth is *another project on the same host*, not *another host*. Everything in v1 assumes the host is singular and long-lived.

---

## 1.5. Mental model: roles

HOOP hosts three distinct roles, each with its own scope and authority:

| Role | What it is | Authority |
|---|---|---|
| **The host (HOOP daemon)** | The long-lived process on the EX44. Project registry, event readers, conversation archive, file browser, Stitch-creation surface, UI. | Reads everything; one write: `br create`. |
| **The human-interface agent** | A persistent, long-running Claude Code session (Opus-class) the host spawns and manages. Cross-project read access, one write: drafts Stitches (which the host decomposes into `br create` calls). The operator's primary conversation partner. Reviews artifacts. Answers questions. Drafts work. Never spawns or stops NEEDLE workers. | Reads everything the host reads; writes only through the host's Stitch-creation API. |
| **NEEDLE workers** | Individual processes claiming and executing beads. **HOOP does not control them.** Born, live, and die under NEEDLE's authority. HOOP only observes. | None from HOOP's perspective. NEEDLE manages them. |

The separation of concerns is hard: NEEDLE owns execution; HOOP owns the interface. The human-interface agent **asks for work** by drafting Stitches; NEEDLE workers decide whether and when to claim the resulting beads based on NEEDLE's own logic.

This is a deliberate narrowing of HOOP's original scope. Earlier drafts had HOOP steering workers, throttling on capacity, rotating accounts, releasing stuck claims. All of that is out — those concerns belong to NEEDLE (or to a future capacity-aware NEEDLE extension), not HOOP. HOOP becomes simpler and more focused: a reader that occasionally writes a single kind of record, and a conversational agent that helps the operator decide what to write.

**Note on strands.** A NEEDLE worker's strand set (Pluck, Explore, Mend, Weave, Unravel, Pulse, Knot) is determined by the combination of *model* and *harness* at worker launch — not by beads, not by work items, not by any runtime decision. Strands are effectively worker-immutable once the worker is running. HOOP observes and displays which strand a worker is executing; HOOP never attempts to route work by strand, and never assumes a per-bead strand preference.

---

## 1.6. The hierarchy: Patterns, Stitches, conversations, beads

HOOP's user-facing structure:

```
Pattern       (optional, cross-project)   a goal composed of many Stitches
└── Stitch    (project-scoped)            a single conversation
    ├── messages / audio / transcripts    what was said
    ├── linked beads                      work the Stitch produced or discussed
    ├── touched files                     artifacts referenced or modified
    └── spawned / references              links to other Stitches
```

### Stitch = a single conversation within a project

A **Stitch** is HOOP's unit of discourse — one conversation, scoped to one project. Not a work item per se; the record of discourse around work, which may or may not produce beads.

There are four kinds of Stitch, distinguished by how the conversation was initiated:

| Kind | Source | Shape |
|---|---|---|
| **Operator** | Human ↔ human-interface agent chat | Multi-turn text + multimodal attachments; may draft beads via the agent's `create_stitch` tool (which creates beads under this Stitch) |
| **Dictated** | Push-to-talk audio (hotkey or phone ADB) | Single-shot audio + Whisper transcript; often one-sided notes |
| **Worker** | NEEDLE worker's CLI session processing beads | Adapter conversation (Claude Code / Codex / etc.); joined to the project by cwd, to the originating operator Stitch (if any) by the `[needle:<worker>:<bead>:<strand>]` prefix tag |
| **Ad-hoc** | Operator runs `claude` / `codex` / etc. directly in a project terminal | Same shape as worker but untagged; classified as operator-initiated, not HOOP-managed |

A Stitch carries:
- Project + kind + title (operator-set, human-interface-agent-synthesized, or derived from first message)
- Participants (operator, agent, worker name, adapter)
- Messages / audio / transcripts / attachments
- Linked beads — beads created within this Stitch, or the bead a worker Stitch is executing
- Touched files — paths referenced or modified within the Stitch
- Links to other Stitches:
  - `spawned: [...]` — worker Stitches that arose from beads this Stitch created
  - `references: [...]` — operator annotations tying Stitches together
- Cost, duration (aggregated from turns + any linked bead executions)
- `last_activity_at` timestamp — the ranking signal, not a lifecycle terminus

### Stitches don't close — they descend into obscurity

A Stitch has no terminal "closed" or "done" state. Like a Reddit post, a Stitch simply accumulates less and less activity over time and drifts down the list. The UI's primary sort is `last_activity_at` descending; stale Stitches are naturally deprioritized but never destroyed.

Optional lightweight indicators on a Stitch card are all **derived**, not stored as lifecycle state:
- Active (turn or linked bead in-flight right now)
- Awaiting review (linked review-kind bead open)
- Quiet N days (no activity for N days; gets dimmer over time)

Operators can *archive* a Stitch (hide from default views — an auto-applied archive filter after N days of inactivity is the default behavior, matching the Reddit mental model). Archiving is a filter change, not a data change; the Stitch, its audio, its linked beads, its ledger entries all remain in place. The human-interface agent can search across archived Stitches via explicit tool call.

This avoids the classic "did I close this bug?" / "is this ticket done?" lifecycle-policing that ruins issue trackers. Work doesn't need to be declared finished; it just stops needing attention.

### Pattern = a grouping of Stitches toward a goal

A **Pattern** is an optional, user-created grouping of Stitches organized around a goal. Patterns may cross projects — a cluster migration Pattern can include Stitches in `ardenone-cluster`, `declarative-config`, and `apexalgo-iad-secrets` all at once. Patterns are how the operator tracks epics, initiatives, and long-running themes.

A Pattern carries:
- Title, description, optional deadline, optional owner
- Status: `Planned | Active | Blocked | Done | Abandoned`
- Member Stitches (cross-project, explicit list + optional saved-query to auto-include matches)
- Optional parent Pattern (for epic-in-epic nesting)
- Aggregated progress (percent of member Stitches closed), cost, duration
- Notes — free-form Stitches that belong to the Pattern itself, not to any single project

Patterns are optional. Simple work lives as a bare Stitch. Patterns exist only when the operator explicitly creates one, or accepts a human-interface agent suggestion to create one. Most Stitches will never join a Pattern; that's fine.

### How it all fits together

- **Beads** are NEEDLE's execution unit, scoped to a workspace (`.beads/`). They record what a worker claimed, executed, and closed.
- **Stitches** are HOOP's conversation unit, scoped to a project. Every bead a worker processes produces a worker Stitch. Every chat with the human-interface agent is an operator Stitch. Every voice note is a dictated Stitch.
- **Patterns** are the operator's optional goal-level organizing layer, aggregating Stitches across projects.
- **Projects** are the filing cabinet: they contain workspaces (where beads live) and Stitches (where conversations happen).

### Stitch ↔ bead linkage

When an operator Stitch drafts beads, HOOP calls `br create --json <payload>` against the target workspace with a `stitch:<stitch-id>` label on each bead. When a NEEDLE worker claims a labeled bead, its session prompt is prefixed with `[needle:<worker>:<bead>:<strand>]` and a `spawned-by: <operator-stitch-id>` marker. HOOP reads both and establishes the Stitch-to-Stitch link. One small NEEDLE hook preserves the stitch label on any follow-up bead a worker creates.

Beads created outside HOOP (plain `br create` from a terminal) remain orphans — visible in workspace views, unaffiliated with any Stitch. Optional orphan-clustering by title/file similarity comes later.

### Why this shape

- Humans think in **conversations** ("that time I worked through the Calico thing with the agent") and in **goals** ("the cluster migration"). The natural units for those are Stitch and Pattern.
- NEEDLE thinks in **beads**. HOOP translates; NEEDLE keeps its model.
- Because a Stitch is a single conversation, even the most trivial interactions — a 30-second voice note — are first-class entities with durable history. Nothing important is ephemeral.
- Because a Pattern is optional and spans projects, the operator can structure work at whatever granularity they care about without forcing a hierarchy on the system.

---

## 2. The environment HOOP targets

- **Host:** Hetzner EX44-class. Bare metal, long-lived, Tailscale-only.
- **Workspaces:** `~/` holds 5–25 repos. Each has a `.beads/` directory; git worktrees live under `.worktrees/`; some may carry a `fleet.yaml` describing NEEDLE's worker pool (HOOP reads it, doesn't act on it).
- **The `br` binary** — **beads_rust** by [Jeffrey Emanuel (dicklesworthstone)](https://github.com/dicklesworthstone/beads_rust), installed at `~/.local/bin/br`. HOOP treats `br` as its sole bead API. Every bead read goes through `br` read verbs (e.g. `br list --json`, `br get <id> --json`); every bead write HOOP performs is `br create --json <payload>`. HOOP never opens `.beads/beads.db` directly, never writes to `.beads/beads.jsonl` directly, and never links against bead library code. Shelling out is deliberate — it pins HOOP to the `br` surface area, lets `br` evolve independently, and keeps HOOP honest about the "one write path" invariant.
- **Other tooling:** Five CLI adapters installed and credentialed (operator's cache, not HOOP's). NEEDLE installed and running its own workers through its own supervision. git 2.5+.
- **Process model:** `hoop serve` as a systemd user service on a Tailscale hostname. No tmux spawning by HOOP.
- **Parallel workloads:** NEEDLE fleets in tmux (HOOP-observed); ad-hoc CLI sessions in separate terminals (HOOP-observed); everything else (HOOP-ignored).

### 2.1. On the `br` dependency

`br` is upstream-authoritative for bead semantics. HOOP tracks compatibility by pinning a *minimum* `br` version in config; on startup audit, HOOP runs `br --version`, compares against the pinned minimum, and refuses to start (or starts with the bead-creation surface disabled) if the binary is missing or too old. Version strings get logged so regressions are visible.

This environment currently runs a local fork of beads_rust with a rusqlite compatibility shim (replacing the upstream FrankenSQLite backend that had recurring index corruption — upstream issue `dicklesworthstone/beads_rust#171`). HOOP is agnostic to this: whatever `br` binary is in PATH, that's what HOOP shells out to. If and when the upstream fork stabilizes and the shim is dropped, HOOP needs no changes.

HOOP does not bundle or vendor `br`. Installing HOOP does not install `br`; the startup audit tells the operator to install it if missing. This keeps the release pipelines for the two projects fully independent.

What changes in a multi-project host vs single-workspace: session discovery has to be scoped per project, cost visibility has to bucket per project, bead-creation targets have to be explicit. This plan treats multi-project correctness as the primary design pressure.

---

## 3. Principles

1. **Events are authoritative; projections are derived.** HOOP reads event rows and computes projections on demand. Never caches to disk what can be rebuilt.
2. **Liveness = process, never file.** When displaying worker liveness, compute from `kill -0 pid` + heartbeat freshness. HOOP never writes a `worker_status.json`.
3. **Server is the epoch.** Client rebuilds state from scratch on every reconnect.
4. **Dual-identity in schema.** Bead id (stable) + provider session id (derived). Explicit `session_bound` event at first join.
5. **JSON Schema as cross-repo contract.** Every record has `schema_version: 1`. TS and Rust types codegen off one source.
6. **Atomic `.tmp` + rename for every write.** Line-buffered NDJSON reader for every read.
7. **Never silent-drop unknown events.** Log, emit progress, count.
8. **`br create` is HOOP's only write.** No bead mutation, no tmux control, no worker lifecycle, no capacity enforcement. If the operator wants to close, release, or boost a bead, they use `br` directly or another tool.
9. **If HOOP dies, nothing else notices.** NEEDLE keeps running. FABRIC keeps working. The next time HOOP starts, it rebuilds its read state from disk. HOOP is a convenience, not a dependency.
10. **Read-first defaults.** Even bead creation requires explicit operator confirmation (chat intent → draft → preview → submit). No silent writes.
11. **Humans speak in Stitches, not beads.** HOOP's UI, human-interface agent, forms, and chat use project-scoped work items. The bead layer is preserved for machine correctness but hidden from normal operator flow. Bead IDs surface only in expert views and audit logs.
12. **Lazy context, not eager context.** The human-interface agent does not receive project contents in its system prompt. It receives a thin index (project names, recent activity summary, open Stitch titles) and reaches for details through tool calls. File contents, full bead bodies, and conversation transcripts are fetched on demand. This keeps the context window sustainable across long sessions and keeps cost predictable — the agent pays for what it actually asks about, not for what *might* have been relevant.
13. **Learn from repetition.** When the operator repeats an instruction, correction, or preference across Stitches, HOOP treats it as a signal that a durable rule can be extracted. The human-interface agent proposes entries for a **Reflection Ledger** — a curated, operator-approved store of standing preferences, conventions, and "don't do X" rules. Approved entries are injected into every subsequent agent session (via the lazy-context index) so the operator doesn't have to say the same thing twice. The ledger is visible, editable, and pruneable; nothing is learned silently.

---

## 4. Component architecture

### 4.1 Single-binary HOOP daemon

One Rust binary. Subcommands:

```
hoop serve                    # run the daemon (web UI + WS + REST)
hoop projects add <path>      # register a workspace
hoop projects scan <root>     # auto-register every workspace with .beads/
hoop projects list
hoop projects remove <name>
hoop status [project]         # CLI overview of fleets / beads / cost
hoop audit                    # startup binary/env audit
hoop agent                    # attach to or start the human-interface agent conversation
hoop new <project>            # CLI shortcut to draft+submit a Stitch (decomposed into beads)
hoop stitch list [project]    # list open Stitches; use --beads for the underlying bead view
```

Notably absent: `launch`, `stop`, `salvage`, `steer`, anything that touches a worker process. Those verbs belong to NEEDLE.

`hoop serve` is the long-lived process; other subcommands are clients over a Unix socket (`~/.hoop/control.sock`).

### 4.2 Project registry

A **project** is a logical unit of work as the operator thinks of it. It may be a single repository or it may span multiple repositories. Each project contains one or more **workspaces** (repos on disk); each workspace has its own `.beads/` queue and git state. Stitches are project-scoped and can span workspaces within the same project; beads remain workspace-scoped (because `br` operates per-`.beads/`).

This matters because real projects are often multi-repo — a deployment project with manifests in one repo and source in another; a migration project touching many service repos; a research project with a docs repo plus a code repo.

`~/.hoop/projects.yaml`, file-watched:

```yaml
projects:
  - name: ardenone-cluster
    label: "Cluster config"
    color: "#8A2BE2"
    workspaces:
      - path: /home/coding/ardenone-cluster
        role: primary
      - path: /home/coding/declarative-config
        role: manifests
  - name: miroir
    label: "Meilisearch orchestrator"
    color: "#FF4500"
    workspaces:
      - path: /home/coding/miroir
        role: primary
  - name: kalshi-weather-migration
    label: "Move kalshi-weather to apexalgo-iad"
    color: "#0080FF"
    workspaces:
      - path: /home/coding/kalshi-weather
        role: source
      - path: /home/coding/declarative-config
        role: manifests
      - path: /home/coding/apexalgo-iad-secrets
        role: secrets
```

Backward-compatible single-workspace shorthand remains supported:

```yaml
projects:
  - name: ibkr-mcp
    path: /home/coding/ibkr-mcp      # shorthand: single workspace, role: primary
    label: "IBKR MCP server"
```

Hot-reloads on change. Each project gets its own per-project runtime that fan-outs per workspace. A workspace may not belong to more than one project at a time — project boundaries are a partition, not a tagging layer.

**Stitches across workspaces.** A Stitch belongs to one project but may have beads in multiple workspaces of that project. The Stitch service records which workspace each bead lives in; cross-workspace dependencies are expressed as Stitch-child relationships, not bead-level deps (because `br` can't express cross-`.beads/` dependencies). This preserves NEEDLE's per-workspace atomicity while still letting the human see a coherent Stitch.

### 4.3 Per-project runtime (read-only)

Pure reader — every interaction with the project is observational:

```
Inputs (disk)                                                  Projections
├─ .beads/events.jsonl         → event tailer               →  active_beads
├─ .beads/heartbeats.jsonl     → heartbeat monitor          →  worker_liveness
├─ .beads/beads.db             → bead state reader (via br) →  backlog_view
├─ files under <project>/**    → file walker + mime detect  →  file_tree
├─ git state under <project>   → git reader                 →  branches, worktrees, diffs
└─ CLI session dirs, filtered by cwd-under-path             →  conversations (fleet vs ad-hoc)

Outputs
├─ WS fan-out with topic <project>
├─ REST endpoints /api/p/<project>/...
└─ One write path: POST /api/p/<project>/beads  →  br create  →  audit row
```

The bead-creation endpoint is the only mutation. It calls `br create` against the project's workspace, records the mutation in `~/.hoop/fleet.db` audit table, and emits a `bead_created_by_hoop` event for downstream consumers (the human-interface agent records its own drafts; the UI records operator-initiated drafts).

### 4.4 Cross-project state

Global state is thin:

- Project registry view
- Runtime status per project
- Audit log (one table in `fleet.db`, records every `br create` HOOP performed with actor, timestamp, project, bead id)
- Cost roll-ups (observation only)
- Capacity roll-ups (observation only, per adapter account)
- Conversation archive index (indexed by project + session id + fleet vs ad-hoc)

No bead content, no conversation content in the global layer. UI aggregates on demand.

### 4.5 Web client

React + Vite + TypeScript + Jotai. Served by the daemon (embedded static assets). Key surfaces:

- **Overview** — dashboard, one card per project
- **Project detail** — bead list/graph, worker timeline, conversation list, file browser, cost + capacity panels
- **File browser** — tree view + preview with syntax highlighting (phase 3)
- **Conversations** — cross-project transcript viewer, fleet vs ad-hoc filter
- **Bead draft** — form-based and chat-based; multimodal attachments (phase 3-4)
- **human-interface agent chat** — operator ↔ human-interface agent, with multimodal input (phase 5)
- **Search** — cmd-K across projects and conversations
- **Audit** — read-only view of HOOP's own `br create` history

Streaming content lives in a separate reactive map. The committed-vs-streaming split is preserved.

### 4.6 Shared schema crate

`hoop-schema/` with JSON Schema source of truth. Rust types via `typify`; TS via `json-schema-to-typescript`. `schema_version: 1` on every record.

### 4.7 Stitch, Pattern, and Reflection services

HOOP-owned state layer. Tables in `~/.hoop/fleet.db`:

```
stitches(id, project, kind, title, created_by, created_at, closed_at,
         participants, attachments_path)
  -- kind ∈ {operator, dictated, worker, ad-hoc}

stitch_messages(id, stitch_id, ts, role, content, attachments, tokens)
  -- every turn / audio entry / tool-call in the Stitch

stitch_beads(stitch_id, bead_id, workspace, relationship)
  -- relationship ∈ {created-here, executing, referenced}

stitch_links(from_stitch, to_stitch, kind)
  -- kind ∈ {spawned, references}

patterns(id, title, description, status, owner, deadline, parent_pattern)
  -- status ∈ {Planned, Active, Blocked, Done, Abandoned}

pattern_members(pattern_id, stitch_id)

pattern_queries(pattern_id, saved_query)
  -- optional auto-include rules

reflection_ledger(id, scope, rule, reason, source_stitches, status,
                  created_at, last_applied, applied_count)
  -- scope ∈ {global, project:<name>, pattern:<id>}
  -- status ∈ {proposed, approved, rejected, archived}
```

**Stitch service.** Stores conversations, appends messages/audio/transcripts, links beads and other Stitches. A Stitch's status is derived on read from the state of its linked beads plus its own turn activity (`In Progress` if streaming; `Awaiting Review` if review-type linked beads are open; `Done`/`Failed` on terminal).

- Operator Stitch creation: UI form or chat intent → row + first turn.
- Worker Stitch creation: session tailer detects a new CLI session with a `[needle:...]` prefix (auto-linked to the spawning operator Stitch via the `spawned-by` marker).
- Dictated Stitch creation: hotkey / ADB capture → audio + Whisper transcript → row.
- Bead creation from within a Stitch: `br create --json <payload>` with `stitch:<id>` label. That label is the only way a bead gets associated with a Stitch; NEEDLE's follow-up-bead hook preserves the label so retries stay linked.
- No direct bead mutation. Ever. A Stitch needing another attempt drafts a *new* bead with the same stitch label; the old bead stays closed as historical record.

**Pattern service.** Operator-curated groups of Stitches toward a goal. May span projects. Membership explicit (operator adds Stitches) or query-driven (saved-query auto-includes matching new Stitches). Progress / cost / duration aggregate across members.

- Patterns are always optional. A Stitch without a Pattern is normal, not neglected.
- A Stitch may belong to multiple Patterns.
- Parent-Pattern nesting supports epic-in-epic structures when the operator needs them.

**Reflection Ledger.** Persistent store of learned rules and preferences. Proposals, not auto-applies.

- Detection runs asynchronously after each closed operator Stitch. The human-interface agent scans recent operator Stitches for: repeated corrections, repeated preferences ("always do X before Y"), repeated negatives ("don't ever touch production config without a review bead"), repeated approvals of a non-obvious choice.
- Each detected pattern produces a proposal: `{rule, reason, source_stitches[]}`. Proposals surface in a dedicated UI pane.
- The operator approves, edits, or rejects each proposal. Approved rules enter the ledger with a scope (global, project-scoped, or Pattern-scoped).
- Approved rules are injected into every new agent session as part of the lazy-context index — one line per rule, with the scope noted.
- Every injection is auditable: the ledger records `last_applied` and `applied_count` per rule; the session log records which rules were injected.
- Rules are editable and pruneable. Nothing is learned silently. An entry marked `archived` stops being injected but is kept for history.

**No direct bead mutation** across all three services. HOOP overlays bead state; never modifies it.

---

## 5. Data flows

### 5.1 Single-project reader flow

```
NEEDLE-managed worker (tmux)                     HOOP daemon
────────────────────────────                     ───────────
br claim bd-abc
   │
   └─► .beads/events.jsonl ─── tail -F ───► event tailer
                                                  │
                                                  ├─► /ws fleet
                                                  │   (topic: <project>)
                                                  │
                                                  ▼
dispatch via NEEDLE's adapter                ─── (observed) ───
(prompt prefix [needle:alpha:bd-abc:pluck])
   │
   └─► ~/.claude/projects/<...>/*.jsonl
                                                  │
                                                  │ 5s poll
                                                  ▼
                                              session tailer ──► tag-join ──► /ws conversation

br close (NEEDLE's action)
   │
   └─► .beads/events.jsonl ─── tail -F ───► event tailer ──► cost aggregator
                                                                │
                                                                └─► /ws fleet
```

HOOP is never on the write path here. Every arrow going into HOOP is a read.

### 5.2 Bead creation (HOOP's only write)

```
Operator                      HOOP daemon                       NEEDLE workspace
────────                      ───────────                       ────────────────
(UI form or human-interface agent chat)
       │
       │ draft bead (title, body, deps, priority, attachments)
       ▼
  ┌─────────────┐
  │ Bead draft  │
  │ preview     │
  └─────┬───────┘
        │ confirm
        ▼                            execute
                               ┌──────────────────┐
                               │ br create ...    │────────► .beads/beads.db (insert)
                               │ (in project cwd) │         .beads/events.jsonl (append)
                               └────────┬─────────┘
                                        │
                                        ├─► audit row in fleet.db
                                        │
                                        └─► event emitted on /ws
                                                      │
                                                      ▼
                                         (NEEDLE workers eventually
                                          claim the bead — HOOP is
                                          not involved)
```

Attachments (images, audio, video) in a multimodal draft are stored at `<project>/.beads/attachments/<bead-id>/` and referenced from the bead body. `br` doesn't need to know about them — they're just files next to the bead.

### 5.3 Ad-hoc vs fleet conversations

Same as before: first message prefix `[needle:<worker>:<bead>:<strand>]` → fleet; no prefix → operator ad-hoc. HOOP classifies; doesn't act on either.

---

## 6. Phased roadmap

### Phase 0 — Foundation (COMPLETE)

### Phase 1 — Single-host daemon, one workspace, read-only (v0.1)

**Goal.** HOOP runs on the EX44 as a pure observer of one workspace. Serves a web UI that shows bead state, worker liveness (observed), conversations, and events. No writes at all.

**Deliverables:**
1. Rust binary with `serve`, `projects add`, `status`, `audit`.
2. Per-project runtime for one workspace: event tailer, heartbeat monitor, session tailer, tag-join resolver, bead state reader.
3. Web UI: bead list, worker timeline (liveness derived from events + heartbeats), conversation viewer with fleet/ad-hoc split, audit overlay, search palette.
4. `~/.hoop/fleet.db` SQLite with only the audit table (empty until phase 4).
5. NEEDLE hooks documented (prompt tag, events append, heartbeat) — requires NEEDLE cooperation. HOOP is read-only on these files.
6. Startup audit: `br`, project's `.beads/` accessibility, CLI session directories readable.
7. Zero-write invariant enforced in code (no code path that calls `br` with anything other than read verbs in phase 1).
8. `br` dependency audit: `br --version` run on startup, pinned minimum version checked, friendly diagnostic on mismatch or missing binary.

**Non-goals:** Multi-project, any write path, cost, graph views.

**Success criteria:**
- HOOP runs alongside a NEEDLE fleet without affecting it.
- Killing HOOP does nothing to the fleet; workers keep claiming and closing beads.
- Restart HOOP; UI rebuilds state entirely from disk in <5s for 500 beads.
- Every bead in the fleet visible in the UI; every worker's transcript viewable with its bead id in the header.

### Phase 2 — Multi-project observability + cost/capacity visibility + visual debug (v0.2)

**Goal.** One HOOP serves every project on the host. Cross-project dashboards, cost and capacity visibility (read-only — no enforcement), visual debug of what workers did per bead.

**Deliverables:**

1. Project registry (`projects.yaml`) with add/remove/scan/hot-reload.
2. Per-project runtime isolation; failure in one doesn't cascade.
3. Fleet-of-fleets dashboard: project cards with worker count, active beads, cost today, stuck count, last activity.
4. Project detail view: fleet map, bead graph (DAG), strand timeline, conversation list.
5. Cross-project dashboards: total spend today/week, total workers running, longest-running beads.
6. Ad-hoc vs fleet classification + filter controls.
7. Unassigned-conversation bucket for sessions outside any project.
8. Search palette across projects with project badges.
9. Cost panel (observation only): per-project, per-adapter, per-model, per-strand, per-day; rate-limit window overlay for Claude (5h + 7d); cost-per-closed-bead.
10. **Capacity visibility** (observation only, no enforcement):
    - Per-account 5h + 7d utilization meters computed from local JSONL logs (same numbers `/status` shows)
    - Per-account spend-based caps where applicable (Codex daily, API-key flows)
    - Burn-rate forecast ("account claude-max-primary full in ~42m at current burn")
    - Saturation alerts when thresholds cross — shown in UI, surfaced to the human-interface agent in phase 5
    - **No actions** — HOOP does not pause, rotate, or throttle anything
11. **Visual debug panel** — per-bead step-through of what the worker actually did: prompts sent, tool calls issued, results, stderr, state transitions. Scrubable timeline at the bead level. Reconstructed from events + tagged conversation transcript. Answers "what did this worker actually do with my $2.80?" in one view.
12. Collision detector (observation only): alerts when active workers touch overlapping files.
13. Stuck detector (observation only): heartbeat-transition silence or repeated retries surfaced as alerts.

**Plus five marquee capabilities:**

14. **Stitch abstraction layer** (foundational). UI, forms, human-interface agent, and dashboards render Stitches (conversations). Bead-level views exist behind an "Expert" toggle. Every new bead HOOP creates carries a `stitch:<id>` label. NEEDLE hook: follow-up beads inherit parent stitch label. Worker Stitches auto-link to their spawning operator Stitches via session-prefix markers.
14b. **Pattern layer** (foundational, phased alongside Stitch). Operator-curated groups of Stitches toward a goal. May span projects. Optional — simple work lives as a bare Stitch. Pattern view aggregates progress, cost, and duration across member Stitches. Saved-query Patterns auto-include matching new Stitches as they appear.
15. **Stitch-Provenance Code Archaeology** — the file preview panel overlays standard git blame with the *Stitch* that introduced each line. Hover → see the Stitch title, status, and conversation; click → jump to the Stitch view. Requires a NEEDLE hook to emit a `Bead-Id:` commit trailer on close (one line); HOOP maintains a bead-id → commit-sha index and joins that with stitch membership.
16. **Stitch Net-Diff Viewer** — when a Stitch's work produces a coherent set of commits (multi-bead Stitch clusters, Patterns with completed member Stitches), HOOP computes the aggregate diff across every commit produced by the Stitch's beads (using the commit trailer from #15) and renders it as if it were a single PR: one tree, one narrative (agent-synthesized), one review surface. Replaces trawling PR history for multi-step work.
17. **Cost-Anomaly with Fix Lineage** — continuous detector flags Stitches whose observed cost exceeds the 2σ band for historically similar Stitches (lexical + embedding similarity). The alert is actionable: it surfaces the closest past matches, the Stitches that fixed them, and a pattern name. A curation UI lets operators tag recurring failure modes with recommended fix templates; the library compounds over time.

**Success criteria:**
- `hoop projects scan ~/` registers every workspace with `.beads/` in one command.
- Cost figures match `br`/provider summaries within ±2%.
- Capacity meters match Claude Code's `/status` within ±5% per account.
- Visual debug reconstructs a full bead cycle with no gaps (prompts + tools + outcome).
- Killing one project's runtime (delete `.beads/`) shows an error card; other projects unaffected.
- Dashboards contain zero bead IDs by default; toggling Expert view reveals them.
- A hover on any line in the file preview surfaces the Stitch that produced it within 200ms.
- Stitch Net-Diff assembles correctly for a 5-bead, 11-commit cluster.
- Cost anomaly detector flags a synthetic 3σ test case with the right historical match.

### Phase 3 — File browser + artifact preview + multimodal (v0.3)

**Goal.** The operator can browse any project's files, preview code/docs/images/media with syntax highlighting, and supply multimodal input for bead drafting and human-interface agent conversations.

**Deliverables:**

1. **Per-project file browser:**
   - Tree view with mtime + size + git status
   - Filterable by extension, by git-modified-since-ref, by contents (grep)
   - Respects `.gitignore` + configurable HOOP ignore list
   - Lazy-loaded directory expansion for large trees
2. **Text preview with syntax highlighting:**
   - Tree-sitter or `syntect` for highlighting
   - Language auto-detect with manual override
   - Line numbers, word wrap toggle, search within file
   - Side-by-side diff view for git-tracked files (working-tree vs HEAD, or two refs)
3. **Non-text preview:**
   - Images (PNG, JPG, WebP, GIF) — inline render with zoom
   - PDFs — pdf.js embed
   - Audio — HTML5 audio player
   - Video — HTML5 video player
   - Binary files — hex dump with offset navigation
4. **Artifact-aware links:** bead view shows "files touched" based on events; click opens in the file browser at the right revision (HEAD at the time of the bead's close).
5. **Multimodal input to bead drafts:**
   - Text body (markdown with preview)
   - Image attachment (paste or upload; stored under `<project>/.beads/attachments/<bead-id>/`)
   - Audio attachment (recording in-browser or upload) — transcribed in-line for searchability
   - Video attachment (upload) — first-frame thumbnail, in-line playback
   - All attachments persisted next to the bead; referenced by path in the bead body
6. **Multimodal input to human-interface agent conversations:**
   - Same attachment types as bead drafts
   - Attachments pass to Claude Code via native multimodal support
   - Transcripts + metadata indexed for later search
7. **Streaming upload** for large files with progress + resumability.
8. **Path-sensitive routing:** drag a file from the tree into a bead draft → the draft picks up the path + current revision + snippet context.

**Plus two capture surfaces:**

9. **Dictated Notes** — a pure note-taking mode, independent of Stitch drafting:
   - Hotkey or phone-ADB push-to-talk starts recording audio.
   - Local Whisper transcribes to text + word-level timestamps.
   - The resulting **Note** is stored as a first-class entity in the project's conversation history: audio file + transcript + timestamp + project scope + optional tags.
   - Both the audio playback control and the rendered transcript appear in the project's timeline alongside fleet conversations, operator ad-hoc sessions, and Stitch activity. Scrubbing the audio highlights the transcript position and vice versa.
   - Notes are searchable by transcript text and optionally linkable to existing Stitches, files, or conversations as references. They are *not* automatically promoted to Stitches — they remain observations until the operator decides otherwise.
   - Later flows (bead drafting, human-interface agent chat) can attach a Note by reference rather than re-uploading audio.
   
   This is the "thinking out loud while walking" surface. Often the most valuable record of *why* something was done is a two-minute voice note from the moment the decision was made.

10. **Voice / Screen Work Capture** — a second capture flow explicitly for drafting Stitches:
    - **Voice-to-Stitch.** Push-to-talk produces a transcribed Note (flow #9) AND kicks it into the human-interface agent, which synthesizes a title and body for a Stitch draft. Audio + transcript attach to the draft. Operator reviews and confirms.
    - **Screen walkthrough.** Browser `MediaRecorder` captures screen + audio while the operator narrates a bug or explains context. Frame-sample + full audio + transcript all attach to a Stitch draft; the agent synthesizes a title and body from the transcript.
    - Two minutes of walking-around thinking becomes a preview-ready Stitch sitting in the queue.
    - A captured session always creates a Note first (same entity as #9); the Stitch draft is an optional follow-on. That way the audio survives even if the operator decides the work isn't worth drafting.

**Success criteria:**
- File browser usable on a 20k-file repo with <1s directory-expand latency.
- Syntax highlighting for at least: Rust, TS/JS, Python, Go, Clojure, YAML, TOML, Markdown, Shell, SQL, Dockerfile.
- Image/audio/video preview works in Safari, Chrome, Firefox.
- Attached 10MB image in a Stitch draft stored and referenced correctly.
- human-interface agent receives attachments in its conversation context.
- End-to-end voice capture on the Pixel 6 produces a transcribed Stitch draft in under 60s.

### Phase 4 — Bead creation interface (v0.4)

**Goal.** HOOP gains its single write path: creating beads via UI form and chat-driven drafting. Every `br create` HOOP performs is audited, reversible (by the operator via `br`), and previewed before submit.

**Deliverables:**

1. **Form-based bead draft:**
   - Target project (required)
   - Title, body (markdown), issue type (default `task`; `genesis`, `review`, `fix` selectable)
   - Priority (with default inferred from current queue)
   - Dependencies (pick from existing beads in that project, searchable)
   - Assignee hint (optional — workers pick their own)
   - Labels
   - Attachments (from phase 3)
   - Preview panel shows rendered markdown + computed dep graph deltas
2. **Template library:** reusable bead templates stored at `~/.hoop/templates/` (shared across projects) and `<project>/.hoop/templates/` (project-scoped). "Review bead for PR #X", "fix bead from incident Y", etc.
3. **Submit flow:** draft → preview → `br create --json <payload>` (via subprocess in the project's cwd) → audit row → event emitted → UI redirects to the new bead's view. HOOP never opens the bead database directly; the authoritative write is always `br`'s own atomic insert into the project's `.beads/beads.db` + JSONL append.
4. **Chat-driven drafting** (precursor to the full human-interface agent in phase 5):
   - Lightweight chat pane per project (Haiku-class, not the human-interface agent)
   - Takes natural language, produces a draft
   - Never submits directly — always routes through the preview flow
5. **Bulk draft:** paste a bullet list or a markdown doc; HOOP splits it into multiple drafts for review + submit.
6. **Audit trail:** every created bead has `created_by: hoop` + operator identity + source (`form` / `chat` / `bulk` / `template:<name>`) recorded in `fleet.db` actions table.
7. **Explicit non-actions:** HOOP does not `close`, `update`, `claim`, `release`, `depend`, or any other `br` verb beyond `create`. If the operator needs those, they use `br` directly.

**Plus three marquee capabilities:**

8. **"What Will This Take?" Preview** — before submit, HOOP simulates the Stitch:
   - Which worker (by adapter + model) in the fleet would most likely claim the first bead (from historical adapter-work-type fit; **not** strand-based — strands are worker-immutable and HOOP never predicts by strand)
   - Expected cost p50 / p90 from percentile bands of closed Stitches with similar title tokens, body length, labels, attachments
   - Estimated duration p50 / p90 from the same source
   - Closest matching failure pattern from the fix-lineage library (#18 in phase 2)
   - Whether any currently-claimed bead might conflict (file-overlap prediction)
   
   Preview card: "likely picked up by `codex-mid-charlie`; p50 $1.80 / p90 $4.20; ETA 12–40 min; risk: `large_codegen_stack_overflow` pattern matched — consider narrowing scope." Operator can edit and re-preview before committing.
   
9. **Already-Started Detection.** At draft time, HOOP embeds the title + description and searches all open Stitches across projects. If similarity crosses a threshold, the draft UI interrupts: "this looks like `kalshi-weather / evening-flake-investigation`, which is In Progress. Continue that one, add this as a child, or proceed as new?" Prevents the most common fleet-productivity leak: accidental duplicate work.

10. **Stitch Replay from Failure Point.** When a Stitch fails, HOOP reconstructs the full state at the moment of failure: the rendered prompt sequence, every tool call and result up to the crash, the partial worktree git state, the last assistant turn. Presents two options:
    - **Resume as a new Stitch attempt** — HOOP creates a new bead labeled with the same stitch id, pre-populated with a "pick up from step N" prompt and the failure context; NEEDLE workers claim it normally.
    - **Continue in human-interface agent** — the human-interface agent inherits the reconstructed state in its conversation and can continue the work interactively with the operator.
    
    Only HOOP has the joined view (NEEDLE events + CLI session JSONL + worktree git state) needed to reconstruct the moment cleanly.

**Success criteria:**
- A form-drafted Stitch appears in NEEDLE's queue (as the underlying beads) and is claimed by a worker without human intervention.
- An audit row exists for every HOOP-created bead; `br` log shows the same creation.
- Chat-driven drafting produces a reasonable first Stitch draft for common intents ("review the last merge in project X", "investigate the Calico IP issue").
- Bulk draft correctly splits a 10-item markdown list into 10 previewable Stitch drafts.
- "What Will This Take?" estimates land within the p50 / p90 bands for 80% of closed Stitches after 30 days of operation.
- Already-Started Detection catches a synthetic duplicate across two projects with >95% recall at threshold.
- Stitch Replay reconstructs the failure moment and successfully resumes a bead that completes.

### Phase 5 — The human-interface agent (v0.5)

**Goal.** Seat the human-interface agent — a persistent, cross-project conversation partner who reviews artifacts, answers operator questions, and drafts beads. The human-interface agent is the operator's main interface to HOOP once it's seated.

**Deliverables:**

1. **human-interface agent session** — a long-running Claude Code session (Opus by default, configurable) hosted by HOOP as a first-class resource. Persists across HOOP restarts via Claude Code's native session store; HOOP tracks the session id.
2. **human-interface agent context (read-only via MCP server):**
   - Project registry + per-project status
   - Bead queue summaries per project (open / claimed / blocked / recently closed)
   - Worker liveness (observed, not controlled)
   - Conversation archive search
   - File tree + file read + grep per project
   - Cost + capacity roll-ups
   - Visual-debug reconstructions per bead
   - Recent audit log
3. **human-interface agent tool belt — one write, many reads:**
   - `create_stitch(project, title, description, kind, attachments[])` — the one write. Internally decomposes to one or more `br create` calls; the human-interface agent speaks in Stitches, not beads.
   - `find_stitches(project, filter)`, `read_stitch(id)` (returns the aggregated view)
   - `find_beads(project, filter)`, `read_bead(id)` — expert-only, available but rarely needed
   - `read_file(project, path, revision)`, `grep(project, pattern)`
   - `search_conversations(query, project?)`
   - `summarize_project(project)`, `summarize_day()`
   - `escalate_to_operator(message)` — UI banner; no auto-actions
   - **No** `launch_fleet`, `stop_fleet`, `release_claim`, `boost_priority`, `close_stitch`, `close_bead`. If the human-interface agent concludes work needs stopping or a Stitch needs closing, it escalates to the operator.
4. **Notification channel.** When a fleet closes a bead, completes a convoy, hits a capacity threshold, or surfaces a stuck-worker alert, the human-interface agent receives a structured event. human-interface agent decides whether to surface it to the operator.
5. **Operator ↔ human-interface agent chat pane** — primary UI surface. Cross-project by design. Multimodal input (from phase 3). Streams in real time.
6. **Bead drafts via human-interface agent** route through phase 4's preview flow — no direct submits.
7. **Agent-off switch.** HOOP remains fully functional without the human-interface agent. Enabling it requires explicit adapter config in `~/.hoop/config.yml` — Claude Code account, Anthropic API key, ZAI proxy credentials, or any NEEDLE-compatible adapter. HOOP matches NEEDLE's adapter configuration schema so the operator doesn't learn two systems. Switching adapters mid-stream (e.g. Anthropic outage → failover to ZAI/GLM) is supported; the agent session's continuity is adapter-dependent but HOOP's own Stitch and audit state are not.
8. **Audit trail.** Every Stitch the human-interface agent drafts carries `actor: hoop:agent:<session>` in the audit log, with a link to the chat turn that produced it.

**Plus two marquee capabilities:**

9. **Morning Brief** — at operator login (or a configured time), the human-interface agent autonomously reviews overnight activity across every project and produces a structured briefing:
   - What closed successfully, what failed (with cost impact), what's stuck, what's anomalous (via #18 cost lineage), what's blocked on human input
   - Pre-drafted Stitches (always unsubmitted, always preview flow) for follow-ups the human-interface agent thinks are important
   - Cross-project propagation suggestions (see #10)
   - **One headline** — the single thing the human-interface agent thinks should take priority today, with evidence
   
   Turns the "what happened overnight?" question from a 20-minute trawl into a two-minute read. The agent proposes the starting move each morning; the operator confirms, edits, or redirects.

10. **Cross-Project Stitch Propagation.** The human-interface agent recognizes when a fix pattern applied in one project has structural siblings in other projects (same config shape, same file layout, same dependency, similar recent failure signals). Surfaces: "you just closed `fix Calico IP selection` in `iad-acb`. The same pattern exists in `iad-ci`, `rs-manager`, `ardenone-cluster`. Draft matching Stitches for each?" Always preview; operator accepts per-project or all-at-once. Uniquely HOOP because cross-project visibility is HOOP's core position — no single-project tool can make this connection.

11. **Reflection Ledger** — HOOP learns from repetition. After each closed operator Stitch, the human-interface agent scans recent operator Stitches for repeated corrections, preferences, negatives, and non-obvious approvals. Detected signals become *proposals*: "I notice you've said 'don't edit production config without a review bead' three times this month across two projects. Promote to a rule?" The operator approves (optionally editing) or rejects; approved entries enter the Reflection Ledger with a scope (global / project / Pattern) and are injected into every subsequent agent session as part of the lazy-context index.
    
    Nothing is learned silently. Every injection is logged. The ledger is a pane the operator can read, edit, or prune at any time. It's the closest thing to "teaching the system once and having it remembered" — and the mechanism by which a single operator working across many projects stops having to repeat themselves.

**Success criteria:**
- human-interface agent session survives `systemctl restart hoop` with full context intact.
- Operator asks "what did we do today across all projects?" and gets a coherent cross-project summary in Stitch language.
- Operator asks "something feels off on kalshi-weather" and the human-interface agent reviews recent Stitches, conversations, and files; responds with a focused answer and (if warranted) a drafted Stitch for the operator to review.
- human-interface agent never performs a worker action (no launch/stop/release/close). Attempts to ask for such actions produce an explanation pointing at `br` or NEEDLE.
- Morning Brief produces a useful daily summary and at least one correctly-scoped pre-drafted Stitch per typical overnight run.
- Cross-Project Propagation catches a real fix-sibling across 3+ projects with operator-rated useful accuracy (tracked manually over first 30 days).
- human-interface agent audit log lets the operator reconstruct any drafted Stitch back to the chat turn that produced it.

### Phase 6 — Operational polish (v0.6)

Make HOOP pleasant to run for the long haul.

**Deliverables:**

1. systemd user service template
2. Config hot-reload (projects.yaml, templates, human-interface agent config)
3. Log rotation
4. `/healthz` + `/readyz`
5. Daily snapshot of `fleet.db`
6. Drop-in binary upgrade flow
7. Optional Prometheus `/metrics`
8. Tailscale-identity-aware auth
9. Performance budget: 20 projects × 5 workers observed × 300 beads each, UI responsive
10. Graceful degradation on per-project failures

**Success criteria:**
- `systemctl --user restart hoop` resumes full state in <5s
- A bad `projects.yaml` edit is rejected; old config continues running
- One month of operation produces <1GB in logs + backups

### Phase 6.5 — Marquee capabilities summary

The ten features that earn HOOP its keep, collected in one place with phase assignments:

| # | Capability | Phase | One-line pitch |
|---|---|---|---|
| 1 | Stitch + Pattern layer | 2 | Humans work in project-scoped Stitches and optional cross-project Patterns; beads stay hidden |
| 2 | Stitch-Provenance Code Archaeology | 2 | git blame with the *Stitch* that introduced each line |
| 3 | Stitch Net-Diff Viewer | 2 | Multi-bead Stitch clusters reviewed as one unified PR-like surface |
| 4 | Cost-Anomaly with Fix Lineage | 2 | Over-cost Stitches link to past matches and recommended fixes |
| 5 | Dictated Notes | 3 | Push-to-talk voice notes with audio + transcript in project history |
| 6 | Voice / Screen Work Capture | 3 | Describe work by voice or screencast; HOOP drafts the Stitch |
| 7 | "What Will This Take?" Preview | 4 | Cost / duration / risk preview before submitting a Stitch |
| 8 | Already-Started Detection | 4 | Semantic check catches duplicates across projects at draft time |
| 9 | Stitch Replay from Failure Point | 4 | Reconstruct a failed Stitch's state and resume from there |
| 10 | Morning Brief | 5 | The human-interface agent's daily briefing + pre-drafted Stitches + one headline |
| 11 | Cross-Project Stitch Propagation | 5 | The agent suggests matching fixes for sibling projects |
| 12 | Reflection Ledger | 5 | Learn from repetition; proposed rules the operator approves once, applied forever |

Common thread: each exploits HOOP's unique position as the *join* across projects × Stitches × conversations × files × cost × time. Every one demos in under a minute. None crosses the no-worker-steering line. Collectively they are the difference between "HOOP is a dashboard" and "HOOP is the operator's primary interface to a long-running agent fleet."

### Phase 7 — Multi-operator (v1.0)

**Deliverables:**

1. Roles: viewer (read-only) and drafter (read + create beads). Two levels only.
2. Tailscale identity-based role assignment (config list).
3. Audit log carries real operator identity on every bead creation.
4. Per-operator UI state.
5. Optional presence indicators.
6. Public README, examples, user docs.

**Success criteria:**
- Two operators see consistent state.
- Viewer role cannot access the bead-creation endpoint at the schema boundary.
- README enables a stranger to run HOOP against their own NEEDLE workspace in <30 min.

---

## 7. Technology decisions

| Layer | Choice | Why |
|---|---|---|
| Daemon language | Rust | Matches NEEDLE direction; single-binary distribution |
| **Bead API** | **`br` (beads_rust) by Jeffrey Emanuel — shell-out subprocess** | Upstream-authoritative for bead semantics; HOOP never touches `.beads/` storage directly; version-pinned at startup |
| HTTP / WS server | `axum` | Standard async stack; embedded static |
| File watching | `notify` | Cross-platform; reliable |
| UI | React + Vite + TypeScript + Jotai | Matches team skill; keyed atoms fit streaming split |
| Syntax highlighting | `syntect` (server) + Shiki (client) | Server-rendered for large files; client for interactive |
| Schema | JSON Schema draft-07 + `typify` + `json-schema-to-typescript` | One source of truth |
| Storage (HOOP) | SQLite (audit log + conversation index only) | Small; portable; never holds bead state |
| Event transport (local) | File tail (`notify`) | Cheapest reliable option |
| Agent adapter (primary) | Claude Code via persistent session OR Anthropic API | Default option; matches existing operator workflow |
| Agent adapter (alternate) | ZAI proxy with GLM models, or any NEEDLE-compatible adapter | **HOOP is LLM-agnostic** — the agent is an adapter-configured resource, same shape as NEEDLE worker adapters. Switch by editing `~/.hoop/config.yml`, no code change. Anthropic outage or model deprecation is operator-recoverable, not an incident |
| Agent context | MCP server exposing HOOP's read APIs + one write (`create_stitch` → `br create`) | Clean auth boundary; adapter-independent |
| Audio transcription | Whisper via local model or Anthropic's transcription endpoint | Multimodal input searchability |
| Service supervisor | systemd (user-scope) | Standard |
| Auth | Tailscale identity via whois | Matches environment |

---

## 8. Non-goals

Explicit. HOOP deliberately does not grow into these.

1. **Orchestrating work.** NEEDLE does this.
2. **Steering workers.** No launch, stop, kill, pause, signal, SIGSTOP, SIGTERM, release-claim, reassign, or any other action that touches a worker process or bead lifecycle.
3. **Capacity enforcement.** HOOP shows utilization; it never throttles, rotates, or pauses on thresholds. Enforcement, if needed, belongs in NEEDLE or a dedicated layer.
4. **Routing by strand.** Strands are set at worker launch from the (model, harness) pair and are worker-immutable. HOOP displays which strand a worker is on; HOOP never tries to predict, match, or route work by strand.
5. **Mutating bead state.** Only `br create` is HOOP's write. No close, update, depend, claim, release.
6. **Storing bead state.** `br` owns it. HOOP owns Stitch state (a derived overlay) in `fleet.db`, not bead state.
7. **Replacing FABRIC.** FABRIC read-only, deployable anywhere; HOOP local-host with one write. URL bridge links them.
8. **Multi-host control.** One HOOP, one host. Growth is more projects, not more hosts.
9. **RBAC beyond viewer/drafter.**
10. **Secrets management.** Credentials live in each CLI's native cache.
11. **Browser-only.** HOOP needs a server — it reads filesystem and shells to `br`.
12. **Making operators learn bead semantics.** Humans work in Stitches. Bead IDs are a debugging detail; a normal operator can use HOOP productively without ever seeing one.
13. **Mapping Stitches to outside artifacts.** No GitHub PR linking, Slack / email notifications, iCal deadlines, Sentry feed ingestion, or similar integrations. HOOP's world is the host's filesystem + its own state. External-artifact coupling brings lifecycle, auth, and privacy concerns that multiply maintenance cost for little gain; operators can bridge to external systems via `scripts/` extensions (see §22) if they want.
14. **Stitch lifecycle policing.** Stitches don't close. They descend into obscurity by recency of activity, the way a Reddit post does. No tickets-reopened-by-bots, no backlog-grooming rituals, no "is this still relevant?" debates.

---

## 9. Open questions

1. **Hot-reload granularity** for `projects.yaml`: partial reload or full project-runtime restart? Lean partial. Resolve in phase 2.
2. **human-interface agent: direct Claude Code process or headless session?** Lean direct — spawn a `claude` subprocess HOOP owns, keep its session id, reattach on restart. Resolve in phase 5.
3. **human-interface agent MCP server: in-process or separate?** Lean separate binary `hoop-mcp` so HOOP doesn't depend on MCP libraries at all. Resolve in phase 5.
4. **Audio/video storage** for large attachments: in `.beads/attachments/` (git-tracked risk) or in `~/.hoop/attachments/` (outside the repo)? Lean `.beads/attachments/` with sensible `.gitignore` guidance. Resolve in phase 3.
5. **Transcription model** for audio: local Whisper (no external dep, slower) or Anthropic transcription (fast, network required)? Lean local default + cloud override. Resolve in phase 3.
6. **Conversation history expiry.** LRU eviction with lazy reload on demand. Resolve if/when it becomes a problem.
7. **Session tailer scaling** on a host with years of accumulation: cache parsed-digests in `~/.hoop/session-cache.db`. Resolve if startup exceeds 10s.
8. **Bulk draft limits.** How many drafts to allow in one bulk submit? Lean hard cap at 50 with explicit override. Resolve in phase 4.

---

## 10. Milestones

| Version | Target | Definition of done |
|---|---|---|
| v0.1 | +4 weeks | Single-project EX44 observer; read-only UI; zero write paths |
| v0.2 | +12 weeks | Multi-project + Stitch layer + cost/capacity viz + visual debug + 5 marquee features (stitch-blame, time-machine, net-diff, cost-anomaly) |
| v0.3 | +16 weeks | File browser + syntax highlighting + multimodal attachments + voice/screen capture |
| v0.4 | +22 weeks | Stitch creation: form + chat + templates + bulk + what-will-this-take + already-started detection + replay |
| v0.5 | +28 weeks | The human-interface agent: persistent Claude Code session + MCP tool belt + morning brief + cross-project propagation |
| v0.6 | +32 weeks | Operational polish: systemd, hot-reload, backups, metrics, Tailscale identity |
| v1.0 | +36 weeks | Multi-operator with viewer/drafter roles; public README |

Dates are planning fiction; ordering matters. **Do not build Stitch creation before observability is real.** Drafting work against a backlog the operator can't inspect is worse than no drafting at all. **Do not seat the human-interface agent before direct Stitch creation and the marquee observability features exist** — the human-interface agent is a productivity multiplier on top of those surfaces; without them it's a chatbot without tools.

---

## 11. Relationship diagram

```
                         operator (browser)
                                │
                                │ HTTPS / WS over Tailscale
                                ▼
                ┌────────────────────────────────┐
                │       HOOP daemon (EX44)        │
                │                                 │
                │  read: projects / beads /       │
                │         sessions / files /      │
                │         events / heartbeats     │
                │                                 │
                │  write: br create  (only!)      │
                │                                 │
                │  human-interface agent (phase 5+)               │────── URL bridge ──► FABRIC
                │    ↳ MCP context (reads)        │                     (passive observer)
                │    ↳ create_bead (write)        │
                └──────────────┬─────────────────┘
                               │
                               │ reads only
                               ▼
              ┌────────────────────────────────────┐
              │ NEEDLE — independently managed      │
              │ (tmux fleets, supervised elsewhere) │
              │                                     │
              │ - br claim / close / update         │
              │ - CLI adapters                      │
              │ - events.jsonl                      │
              │ - heartbeats.jsonl                  │
              └──────────────────┬──────────────────┘
                                 │
                                 ▼
                         ┌──────────────┐
                         │  br / .beads │
                         │  (SQLite +   │
                         │   JSONL)     │
                         └──────────────┘
                                ▲
                                │ br create (the one write)
                                │
                    ┌───────────┴────────────┐
                    │      HOOP daemon       │
                    └────────────────────────┘
```

HOOP reads everywhere; writes one kind of record to one queue. NEEDLE is a separate system HOOP has no authority over.

---

## 12. Onboarding & documentation

Onboarding is not a phase — it is a cross-cutting concern that delivers alongside each new surface. Consolidating the approach here so it doesn't get lost between phases.

### Three onboarding surfaces

**1. `hoop init` — interactive CLI wizard.** First-time setup:

- Dependency check: `br` (with version pin), tmux, each configured CLI adapter, Tailscale membership, port availability, disk room, systemd user-scope enablement. Any failure reports the exact command to fix it.
- First project registration: offer `scan <root>` with a preview; operator approves per project.
- Agent setup (optional): Anthropic credentials or Claude Code account, model choice, a quick "hello" turn to verify. Skippable — HOOP is fully functional in read-only mode without the agent.
- systemd install + enable.
- Health check, then print the Tailscale URL.

Target: under 5 minutes if tools are already installed.

**2. In-UI first-run experience.** When the operator opens the web UI the first time:

- Welcome overlay: two paragraphs explaining Stitches, Patterns, and the agent briefly. Bead terminology introduced only once and flagged as "internal — you won't usually see these."
- Guided tour: soft highlights on project switcher, Stitch list, agent chat, file browser, audit log.
- Three starter prompts: "dictate a first note," "register another project," "ask the agent something."
- Dismissable; re-openable from settings.

**3. Progressive capability introduction.** Onboarding isn't a single event. As capabilities come online or go unused:

- Upgrade to a version that adds a feature → "What's new" card.
- Reflection Ledger empty after 30 days of use → "You've worked a while; want me to start proposing rules?"
- 10+ Stitches share a theme but no Pattern → suggest creating one.
- Agent never been used → inline prompt on the chat pane.
- Mic never been used → prompt near the hotkey icon.

### Specific onboarding aids

- **Explain-this hover** — every non-obvious UI element has a one-sentence "what this is / when to use it" on hover. Implemented as a shared `ExplainThis` component reading from a central glossary.
- **Dry-run mode for first Stitch drafts** — preview what would happen without creating beads. Operator gets comfortable before committing.
- **Sample Stitches** — an optional "tour" project HOOP can spin up against a sandbox workspace, populated with example Stitches demonstrating typical patterns. Removable in one click.
- **Agent pre-priming** — on first chat, the agent opens with the operator's own data ("I see 4 projects with 12 open Stitches across them; what's on your mind?") rather than a generic greeting.

### Repository documentation

Living in the repo itself:

- **`README.md`** at repo root — combined install + run + concepts + quickstart. Under-30-min path for a human visitor. The first thing anyone reads.
- **`AGENTS.md`** at repo root — LLM-facing guide. Summarizes scope, non-goals, terminology, conventions. Points at the plan as authoritative. Keeps LLMs from re-introducing removed vocabulary (Mayor, polecat, Gas Town) or proposing disallowed features (worker steering, capacity enforcement).
- **`docs/plan/plan.md`** — this document. The canonical, detailed implementation plan. What the README links to for depth.
- **`docs/concepts/`** — one-page-per-concept docs for Stitch, Pattern, Project, Workspace, Agent, Reflection Ledger, Bead (as an internal detail).
- **`docs/operations.md`** — systemd, backups, upgrades, schema migrations, Tailscale routing.
- **`docs/troubleshooting.md`** — common failures with recovery steps, mapped to `hoop audit` output.

`README.md` and `AGENTS.md` at repo root are created alongside the plan so the repo is immediately useful to a visitor — LLM or human — even before any binary exists. They evolve with each phase.

### Phase-by-phase onboarding deliverables

| Phase | Onboarding additions |
|---|---|
| 1 | `hoop audit` + `hoop init` wizard (minimum viable); README at repo root with install + quickstart flow |
| 2 | UI first-run tour; project-scan guidance; capacity-widget explanations |
| 3 | File browser quick-start tooltip; first-dictation prompt near the mic hotkey |
| 4 | Stitch-draft form with inline field hints; sample templates library; dry-run preview |
| 5 | Agent setup wizard; Morning Brief self-introduction on first run; Reflection Ledger first-proposal tutorial |
| 7 | Invite flow for additional operators; role explanation for new viewers vs drafters; per-role cheat sheet |

### Onboarding principles

- **Progressive, never front-loaded.** Don't teach every concept on day one. Introduce each only when it becomes relevant.
- **Viewable opt-out, invisible opt-in.** All onboarding aids are dismissable; none of them gate functionality.
- **Operator-specific, not generic.** "Welcome to HOOP" greetings include the operator's actual data. The agent's first message references their actual projects. Personalized from second zero.
- **Re-playable.** Every tour, tutorial, and introduction can be re-opened from settings. Operators who came back after a month shouldn't be locked out of the first-run context.
- **LLM-first documentation path matters.** A repo with no code but good `AGENTS.md` is immediately useful to a contributor LLM starting fresh. A repo with code and no `AGENTS.md` is a hazard.

---

## 13. Security model

**HOOP inherits security from its host environment.** It does not implement its own authentication layer. The design:

- HOOP binds its listener to the Tailscale interface (or `localhost` for SSH-tunnel access). No public ingress, ever.
- **Whoever can reach the port is authenticated.** If the operator SSHes into the EX44 and port-forwards HOOP's port, they have authenticated themselves via SSH. If they reach HOOP directly over Tailscale, Tailscale ACLs are the auth. The tools the operator already trusts (SSH keys, Tailscale identity, the OS user account) are the trust root.
- Identity for audit purposes comes from Tailscale whois where available, falling back to the OS user running the HOOP process. Multi-operator role enforcement (phase 7) uses Tailscale identities; no password prompts, no session tokens managed by HOOP.
- The human-interface agent's MCP server binds to a Unix domain socket with the same user:group, not a TCP port. Only processes under the same OS user can reach it.
- Agent credentials (Anthropic API keys, ZAI tokens, etc.) live in each adapter's native credential cache — HOOP never stores or proxies them. Agent adapter configuration in `~/.hoop/config.yml` references credentials by adapter-native paths, not by value.

**What HOOP still owns:**
- **Path-traversal hardening** on every filesystem operation — every path derived from a wire ID must match a canonicalized allowlist (project workspace, `.beads/` path, attachments directory). Regex-validate bead IDs (`^bd-[a-z0-9]+$`), Stitch IDs, Pattern IDs, worker names (`^[a-z]+$`) before any filesystem use.
- **Attachment sanitization** — SVG stripped of scripts, PDF metadata validated, file extensions verified by content sniffing not by name.
- **Audit-log append-only** — `fleet.db` actions table never edited; only inserted. A daily hash chain (each row includes a hash of the previous row + its own content) provides tamper evidence for post-hoc review.
- **Secrets scanning** on captured attachments and transcripts (see §18 Privacy).

**Explicit non-authentication patterns**: HOOP never prompts for a password, never issues a session cookie, never verifies a JWT, never talks to an identity provider. If the operator's SSH and Tailscale ACLs are configured correctly, HOOP's security is correct. If they aren't, no amount of application-layer auth would save the host anyway.

---

## 14. Testing strategy

A system that reads everyone's files, shells out to `br`, spawns Whisper, and drives an LLM has multiple layers to test. Strategy:

### 14.1 Test fixtures — the `testrepo/` dummy project

The plan ships a **dummy test workspace** at `testrepo/` in the HOOP repo. It contains:
- A realistic file tree (Rust crate + docs + config, ~500 files)
- A pre-populated `.beads/` with synthetic beads in known states
- Pre-recorded CLI session JSONL files for each adapter (Claude, Codex, OpenCode, Gemini, Aider) tagged with the `[needle:<worker>:<bead>:<strand>]` prefix convention
- A canned `events.jsonl` and `heartbeats.jsonl` that drive deterministic runs
- Example attachments (image, audio, video) for multimodal tests

HOOP's test suite operates against `testrepo/` as the canonical integration environment. No live NEEDLE, no live CLI, no live LLM required.

### 14.2 Test layers

- **Unit tests** (Rust `cargo test`) — pure functions: event parsers, tag extraction, path canonicalization, cost computation, capacity-window arithmetic, Stitch status derivation, schema version negotiation.
- **Integration tests** — daemon starts, tails fixture files, serves a WS endpoint; a test client drives interactions and asserts the resulting state. `br` shells out to a stub binary checked into `testrepo/` that records calls without requiring a real installation.
- **End-to-end tests** — full `hoop init` + project registration + Stitch creation + `br create` against a dummy `.beads/`, asserting the audit log, UI state projections, and event stream.
- **Property tests** (`proptest`) — invariants: (a) event tailer never emits out-of-order events, (b) derived status functions are monotonic in the expected way, (c) replay from disk reproduces live state exactly.
- **Load tests** — a driver generates 20 projects × 5 workers × 200 beads of synthetic activity; assert UI responsiveness budget, memory ceiling, WS fan-out lag.
- **UI tests** (Playwright) — headless browser against the production binary serving the embedded client; asserts responsiveness on desktop and mobile viewports.

### 14.3 What to not test, deliberately

- Real `br` integration is tested against a pinned `br` version in CI, not every `br` version ever. Compatibility matrix is pinned via a single minimum-version audit.
- Real LLM integration is tested via recorded fixtures (a "golden transcripts" directory per adapter). Live agent tests run as a separate opt-in suite for release validation.
- No effort to test actual Tailscale behavior; HOOP runs against `localhost` in CI.

### 14.4 Test targets per phase

Each phase's success criteria translate directly into automated tests. A phase is not considered "done" until its criteria run green in CI against `testrepo/`.

---

## 15. Backups & disaster recovery

### 15.1 What HOOP owns

Everything in `~/.hoop/`:
- `config.yml` — non-secret configuration
- `projects.yaml` — project registry
- `fleet.db` — audit log, Stitches, Patterns, Reflection Ledger
- `attachments/` — Note audio, image/video uploads, screen-capture recordings
- `skills/`, `scripts/`, `notes/`, `prompts/` — operator extensions (see §22)
- `templates/` — Stitch templates

### 15.2 Backup target

Configurable **S3-compatible endpoint** (B2, AWS S3, MinIO, Garage, or any S3 API). Credentials in env vars; endpoint + bucket in `config.yml`:

```yaml
backup:
  endpoint: https://s3.us-west-000.backblazeb2.com
  bucket: hoop-backups-<operator>
  prefix: ex44/
  schedule: "0 4 * * *"         # daily 04:00 local
  retention_days: 30
  # credentials via env: HOOP_BACKUP_ACCESS_KEY_ID, HOOP_BACKUP_SECRET_ACCESS_KEY
```

Defaults to B2 because that matches the existing environment's backup infrastructure (ARMOR encrypted S3 proxy for B2 — see `project_armor_backup_strategy.md` in operator memory). The S3-compatible interface means any endpoint works; operators are not locked to B2.

### 15.3 What gets backed up, how

- `fleet.db` — SQLite `VACUUM INTO` to a temp snapshot, then upload. Daily.
- Attachments — incremental sync (only new/changed files since last successful backup). Daily.
- Config files — on every change plus daily.
- Each backup run writes a `manifest.json` tying together the snapshot's pieces and recording schema version.

All uploads are compressed (zstd) and optionally encrypted (age with a key in env var `HOOP_BACKUP_AGE_KEY`). Encryption recommended when the S3 endpoint is not trusted end-to-end (e.g. egress via public CDN — matches ARMOR's model).

### 15.4 Restore

```bash
hoop restore --from s3://<bucket>/<prefix>/<snapshot-id>
```

`hoop restore` fetches the snapshot, verifies the manifest, restores `fleet.db` + attachments + config to `~/.hoop/` (after moving the existing directory aside for rollback), then runs schema migrations to bring the restored state up to the current HOOP version.

Restore is idempotent; a stopped `hoop serve` is the precondition. The restore flow explicitly does not reconstruct NEEDLE or `br` state — those live in their own workspaces and have their own recovery paths.

### 15.5 Disaster scenarios covered

- **Disk death** — restore to a new host from latest S3 snapshot.
- **`fleet.db` corruption** — restore from the most recent backup; lose at most one day of Stitches and audit.
- **Accidental deletion** (operator ran `rm -rf ~/.hoop/`) — same recovery path.
- **Migration to a new host** — fresh HOOP install + restore; same end state.

### 15.6 What's explicitly not backed up

- Bead state (that's `br`'s job, in each workspace's `.beads/`)
- NEEDLE worker state (separate)
- CLI session files (each CLI owns these)
- Git worktree state (git's job)

---

## 16. Self-observability — metrics HOOP exposes about itself

HOOP exposes Prometheus-format metrics on `/metrics` from phase 6 onward. The metric set is chosen so the operator can diagnose HOOP itself without grepping logs.

### 16.1 Operational

- `hoop_uptime_seconds` (counter)
- `hoop_process_memory_bytes`, `hoop_process_open_fds`, `hoop_process_tasks_total`
- `hoop_panics_total{subsystem}`, `hoop_errors_total{subsystem,kind}`
- `hoop_last_restart_reason{reason}` (gauge, discrete)

### 16.2 Event ingestion

- `hoop_event_tailer_lag_seconds{project}` — distance between event disk-write and HOOP broadcast
- `hoop_session_tailer_lag_seconds{adapter}` — same, per CLI adapter
- `hoop_heartbeat_freshness_seconds_p50/p95/p99{worker}` — heartbeat age distribution
- `hoop_unknown_event_total{adapter,event_kind}` — silent-drop-safeguard counter (should always be growing *slowly*; spikes signal CLI version drift)
- `hoop_event_parse_errors_total{adapter}`

### 16.3 WebSocket & HTTP

- `hoop_ws_clients_connected`
- `hoop_ws_broadcast_lag_ms_p50/p95/p99`
- `hoop_http_requests_total{route,status}`
- `hoop_http_request_duration_ms{route}`

### 16.4 Bead & Stitch operations

- `hoop_br_subprocess_total{verb,result}` — `br create` / `br list` / etc. success/failure
- `hoop_br_subprocess_duration_ms{verb}`
- `hoop_stitch_created_total{project,kind}`
- `hoop_bead_created_by_hoop_total{project}`
- `hoop_audit_append_rate_per_second`
- `hoop_orphan_bead_count{project}` (gauge)

### 16.5 Agent & AI

- `hoop_agent_turn_duration_ms{adapter,model,phase}` — `to_first_token`, `to_completion`
- `hoop_agent_tool_calls_total{tool,result}`
- `hoop_agent_tokens_total{adapter,model,direction}` (input/output)
- `hoop_agent_session_cost_usd` (gauge, current session)
- `hoop_whisper_transcription_duration_ms`
- `hoop_whisper_transcription_errors_total`
- `hoop_reflection_proposal_total{source}` — proposals generated
- `hoop_reflection_approval_rate` — ratio of approved to proposed

### 16.6 Storage

- `hoop_fleet_db_size_bytes`
- `hoop_fleet_db_wal_size_bytes`
- `hoop_attachments_size_bytes`
- `hoop_schema_migration_duration_ms{from,to}`
- `hoop_backup_last_success_timestamp`
- `hoop_backup_last_size_bytes`

### 16.7 Business / meaningful-to-operator

- `hoop_cost_per_stitch_usd_p50/p95{adapter}`
- `hoop_stitches_created_per_day`
- `hoop_cost_anomaly_alerts_total`
- `hoop_already_started_dedup_hits_total`
- `hoop_capacity_meter_exhaustion_warnings_total{account}`

### 16.8 `/debug/state` endpoint

Complementing metrics, a JSON endpoint surfaces runtime structure for incident triage: fleet roster, open Stitches with statuses, agent session IDs, every pid HOOP observes, every WS client, current config hash, backup timestamps. Local-only access by default.

---

## 17. Configuration precedence & hot-reload

### 17.1 Layout

- **`~/.hoop/config.yml`** — non-secret configuration. The single source of truth for almost everything HOOP needs. File-watched; changes hot-reload.
- **Environment variables** — **secrets only**. Tokens, API keys, S3 credentials, age encryption keys. Never in `config.yml`, never in audit log, never in HOOP's own logs at default verbosity.
- **CLI flags** — only for `hoop serve`'s bootstrap (config-file path, socket path, log level). Not used for runtime configuration.

### 17.2 Precedence

```
CLI flags  >  env vars  >  config.yml  >  compiled defaults
```

Precedence is deterministic and documented; no surprises.

### 17.3 What's in `config.yml`

- `agent:` — adapter selection, model, rate limits, cost cap per session
- `projects_file:` — path to projects.yaml (default `~/.hoop/projects.yaml`)
- `backup:` — endpoint, bucket, prefix, schedule, retention
- `ui:` — theme, default project sort, archive-after-days
- `voice:` — Whisper model path, hotkey binding, max recording seconds
- `agent_extensions:` — paths to skills / scripts / notes / prompts directories (§22)
- `metrics:` — whether `/metrics` is enabled, port binding
- `audit:` — retention policy, hash-chain on/off
- `reflection:` — detection thresholds, auto-archive of old proposals

### 17.4 Hot-reload semantics

**Everything reloads without restart**, including:
- Projects registry — adds/removes/renames take effect within 5s
- Agent adapter configuration — next new agent session picks up the change; current session unaffected (operator can `/reload` inside the chat to force a switch)
- Backup schedule — next scheduled run uses new config
- Logging verbosity — next log line uses new level
- UI theme and preferences — next client connection sees them

Exceptions (require `systemctl --user restart hoop`):
- Socket paths and listen addresses
- `fleet.db` location

### 17.5 Validation

Any `config.yml` edit is schema-validated *before* apply. On invalid YAML or schema violation, HOOP rejects the change loudly (UI banner, metric increment, log entry) and **continues running with the previous valid configuration**. A bad config never takes down the daemon.

---

## 18. Privacy & redaction

Dictation, screen captures, file browsing, and agent conversations inevitably touch sensitive content. HOOP's defenses:

### 18.1 Attachment redaction

On each attachment upload — audio transcript, screen-capture frames, image, video — HOOP runs a **secrets scanner** before storage:

- Common secret patterns (API keys in known formats: `sk-...`, `xoxb-...`, AWS access keys, Anthropic keys, GitHub tokens, JWTs)
- Environment-variable-like leaks in text (`OPENAI_API_KEY=...`)
- High-entropy strings above a configurable threshold, with context-aware exclusions (hashes in git commits are fine)
- Email addresses matching operator-configured PII patterns

Detected items are **flagged, not blocked**. The operator sees a warning banner listing what was found ("looks like a Stripe key on line 42 of the transcript") and can choose to redact-in-place (transcript word replaced with `[REDACTED]`, audio muted at word timestamps), redact-and-rewind (delete the attachment entirely), or proceed anyway. Nothing is silently deleted.

### 18.2 Transcript redaction

Whisper transcripts are scanned same as attachments. Word-level timestamps let HOOP mute specific seconds in the stored audio when the operator redacts a word. The transcript edit and audio mute are atomic; scrubbing past a redacted word shows `[REDACTED]` in the transcript and hears silence in the audio.

### 18.3 Session tailer redaction (read-side)

When HOOP reads CLI session JSONL files, it does **not** mutate them (they belong to each CLI, not to HOOP). But HOOP's own projections, transcripts shown in the UI, and any data forwarded to the human-interface agent go through the redaction filter. A `claude` session with an API key in scrollback does not leak through HOOP's lens.

### 18.4 Log hygiene

HOOP's own logs default to `WARN` verbosity in production. `INFO` and `DEBUG` surface more detail for troubleshooting; both have redaction applied to log lines before write.

### 18.5 Operator control

- **Redaction policy** in `config.yml`: pattern sets, threshold for entropy check, action on detection (`warn` / `redact` / `reject`)
- **Per-project override** for teams with stricter policies (e.g. customer-data project: `reject` by default)
- **Audit trail** of every redaction: what was flagged, what the operator did, when

### 18.6 What HOOP cannot prevent

The CLI agents themselves write session JSONL files on the host's disk. If an agent's session contained sensitive content, that's in the CLI's native cache — not HOOP's responsibility to clean up. HOOP's redaction applies to HOOP's surface: its UI, its transcripts, its attachments, its forwarded-to-agent content.

---

## 19. Multi-operator concurrency (phase 7)

HOOP supports multiple simultaneous operators once phase 7 lands. The specifics the plan previously glossed:

### 19.1 Stitch draft concurrency

- Drafts are **server-persisted** from the moment the operator opens a draft form, not just on save. Two operators opening drafts cannot lose work to the other's submit.
- When two operators draft Stitches in the same project concurrently, the server accepts both. No optimistic-lock conflicts; the Stitches are independent entities.
- Presence indicators show "operator X is drafting in project Y" on project cards.

### 19.2 Reflection Ledger concurrency

- Proposals are deduplicated on create — if operator A's turn produces a proposal identical (by content hash) to one already in the pending queue, no new row.
- Approvals are single-operator actions; approved rules list who approved them.
- An operator may reject a proposal another operator would have approved. Track rejection reason; propose again only after another N signals, not immediately.

### 19.3 Agent session ownership

- Each operator has their own agent session. No shared-agent model (that would lose per-operator context).
- A view-only operator can read *other* operators' agent session transcripts (after the fact), but not inject into them.
- The `actions` audit log always attributes to the operator whose agent drafted the Stitch.

### 19.4 Presence

- Optional presence indicators per project and per Stitch ("operator X is viewing this"). Operator-toggleable privacy setting: show me / hide me.
- Does not block writes; multiple operators can concurrently scroll the same Stitch.

### 19.5 Conflict resolution

Primary conflict class: two operators both drafting a Stitch targeting the same workspace. Not a conflict — both submit, both land. HOOP's Already-Started Detection (marquee #8) alerts each operator that the other is about to submit something similar before they do, and offers to combine drafts if both are still pending.

---

## 20. Schema evolution (SemVer)

HOOP's data schema (event records, Stitch rows, Pattern rows, Reflection Ledger entries, config format, audit log rows) follows **Semantic Versioning**:

- **Major version increment** — breaking changes with **no backwards compatibility**. Restore from backup requires an explicit `--major-upgrade` flag with acknowledgement; schema migration is one-way.
- **Minor version increment** — additive / backwards-compatible. Old records read correctly; new features may produce records old readers don't fully understand but can safely ignore.
- **Patch version increment** — bug fixes, no schema shape change.

Every durable record carries `schema_version: "X.Y.Z"` (three parts, not just one integer). Schemas live in the `hoop-schema/` crate; breaking changes are reviewed deliberately and documented in a `CHANGELOG.md`.

### 20.1 Upgrade & migration flow

- Minor / patch upgrades: run on startup, apply transparently, log migration duration to metrics.
- Major upgrades: HOOP refuses to start with a clear diagnostic: "Your data is schema version 1.x; this binary requires 2.x. Run `hoop migrate --from-1 --confirm` or restore from a pre-upgrade backup."
- Backups always include the schema version in the manifest; `hoop restore` refuses to restore a newer-than-current snapshot.

### 20.2 Deprecation window

- Minor version deprecations (old field renamed, etc.) remain readable for at least **one full minor version** after introduction.
- Major version deprecations are one-way; the operator consciously accepts them at the major-upgrade gate.

### 20.3 Version pinning across repos

HOOP pins a minimum compatible `br` major version. A `br` major-version bump triggers a HOOP compatibility audit; HOOP may require its own major bump to follow.

---

## 21. Mobile UX

HOOP's UI is **responsive and mobile-compatible** from day one. The environment's phone-ADB integration (Pixel 6 on Tailscale) makes the phone a first-class input surface, not an afterthought.

### 21.1 Target breakpoints

- Phone portrait (375px wide minimum; Pixel 6 at 412px natively)
- Phone landscape (~700px wide)
- Tablet (768px+)
- Desktop (1280px+ primary target)

### 21.2 Mobile-optimized flows

- **Morning Brief viewing** — card-per-headline layout; swipe to next; one-tap approve or redirect.
- **Dictation** — large mic button always reachable; push-to-talk with haptic feedback; transcript preview before Stitch draft.
- **Stitch list** — compact cards showing project, title, last activity, status indicator; tap to expand.
- **Agent chat** — optimized message composer; attachment picker supports phone camera/photos/audio natively.
- **File browser** — read-only on phone (mobile editing of code is rarely useful); syntax-highlighted previews with side-scroll for long lines.
- **Approval dialogs** — oversized buttons, high contrast.

### 21.3 Not-for-phone flows

- Stitch Net-Diff viewer (too much on a phone screen) — graceful message directing to desktop.
- Time-range multi-project dashboards — compact summary only; full view is desktop-only.
- Reflection Ledger curation — phone shows proposals list; approve on desktop for the editing surface.

### 21.4 Testing

Mobile UX is part of the v0.1 success criteria. Playwright test suite runs at phone viewport sizes; visual regressions blocked in CI.

---

## 22. Extensibility — skills, scripts, notes, prompts

Modeled on the shape of OpenClaw / Hermes (and Claude Code's own skills/slash-command pattern): HOOP is extensible without code changes via four directory-based plugin types.

### 22.1 Plugin types

| Type | Purpose | Directory |
|---|---|---|
| **Skills** | Custom tools the human-interface agent can call | `~/.hoop/skills/<name>/` |
| **Scripts** | Operator-triggered or event-triggered automation | `~/.hoop/scripts/<name>` |
| **Notes** | Structured knowledge files the agent can read | `~/.hoop/notes/<name>.md` |
| **Prompts** | Reusable prompt library referenced by name | `~/.hoop/prompts/<name>.md` |

### 22.2 Skills

A Skill is a directory with:
- `manifest.yml` — name, description, argument schema, the agent-facing one-line summary
- `run` — executable (any language) that reads args from stdin as JSON, writes result to stdout as JSON
- Optional `README.md` — human documentation

HOOP auto-discovers skills on startup and on directory watch. The human-interface agent's tool-belt is augmented with every skill as an available tool; the skill's `manifest.yml` description drives when the agent picks it.

Examples: `lookup-deploy-status`, `fetch-prod-logs-summary`, `check-secret-rotation-age`, `list-open-github-prs`.

### 22.3 Scripts

A Script is a single executable file. Triggered by:
- Operator-invoked (via UI button or `hoop script run <name>`)
- Event-triggered via a manifest (e.g. "when a Stitch in project X is archived, run this")
- Scheduled (cron-style schedule in the manifest)

Scripts have full operator privileges (they run as the HOOP user). The ability to trigger side effects — open a browser, POST to a webhook, send a pushover notification — gives operators the external-integration escape hatch the plan otherwise forbids (per non-goal #13).

### 22.4 Notes

Plain markdown files the agent can read via its `read_note(name)` tool. Use cases:
- Project glossaries
- Team conventions ("we always prefer A over B")
- Reference material the operator wants the agent to have at hand
- Operator's own running notes on long-term work

Project-scoped notes live at `<project-workspace>/.hoop/notes/`; global notes at `~/.hoop/notes/`.

### 22.5 Prompts

Reusable prompt bodies the operator or agent can reference by name (`@prompt:<name>`). Example: a standardized "fix a linting violation" prompt, or "write a plan.md stub."

Prompts can take parameters: `{{project}}`, `{{file}}`, `{{stitch}}` — substituted at invocation time.

### 22.6 Discovery & hot-reload

All four plugin directories are file-watched. Adding a new skill takes effect within seconds without restart; the agent's next turn sees the new tool. Same for scripts, notes, prompts. Hot-reload matches the Reflection Ledger and `config.yml` patterns.

### 22.7 Sharing

Plugin directories are plain files. Operators can share skills via git (`git clone` into `~/.hoop/skills/`), via tarballs, or via a community registry (not built by HOOP; the flat-file format makes any external sharing mechanism work).

### 22.8 Security

Skills and scripts run with the HOOP user's privileges. HOOP does not sandbox them — the operator owns their extensions and the responsibility for what they do. Audit log records every skill invocation with arguments; scripts log their invocation but not their stdout/stderr (those are the script's responsibility).

---

## 23. Appendix — Kubernetes worker deployment (someday, sketched)

If the EX44 saturates, NEEDLE can graduate worker execution to Kubernetes pods — this is a NEEDLE concern, not a HOOP concern. HOOP's role when that happens is the same as today: read the bead events (now streamed from cluster sidecars into a shared log or event bus), offer UI, create beads on operator intent. HOOP does not become a cluster controller.

Rackspace-spot-terraform automation was retired 2026-04-22; spot clusters are now manually provisioned — fine for this deferred work since cluster churn is no longer a background concern.

The trigger for this extension is "NEEDLE's fleet needs to leave the host," not "HOOP wants to be K8s-aware." HOOP's design doesn't change when execution moves.
