# HOOP implementation plan

**Status:** Draft v4 — 2026-04-22
**Scope:** A review-and-request surface for a single long-lived host that holds many repos, many NEEDLE fleets, and many native-CLI conversations. HOOP reads artifacts to answer operator questions and writes beads when new work needs to happen. **HOOP does not steer NEEDLE workers.** Workers live and die under NEEDLE's authority; HOOP observes them and puts items into their queue.

**Lineage.**
- **Steve Yegge's Gas Town** — the "city hosts a Mayor who orchestrates work" mental model. The Mayor is HOOP's primary conversational role; polecats are NEEDLE's workers. Where Gas Town's Mayor commands the polecats, HOOP's Mayor talks to the operator and writes into the bead queue — the polecats still pick up work autonomously through NEEDLE.
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

## 1.5. Mental model: the city, the Mayor, the polecats

Borrowed from Gas Town vocabulary, adapted to HOOP's narrower responsibility:

| Gas Town term | HOOP-over-NEEDLE mapping |
|---|---|
| **City** | The HOOP daemon on the EX44. Infrastructure: project registry, event readers, conversation archive, file browser, bead-creation surface, UI. |
| **Mayor** | A persistent, long-running Claude Code session (Opus-class) hosted by HOOP with cross-project read access and one write: `br create`. The operator's primary conversation partner. Reviews artifacts. Answers questions. Drafts beads. Never spawns or stops workers. |
| **Polecat** | A single NEEDLE worker claiming and executing one bead. **HOOP does not control polecats.** They are born, live, and die under NEEDLE's authority, coordinated through the shared bead queue. HOOP only watches. |
| **Swarm** | A NEEDLE fleet. Same comment — HOOP observes fleets, it does not command them. |
| **Convoy** | A coordinated multi-bead work unit expressed through the bead dependency graph. The Mayor drafts convoys by creating chains of beads with dependencies; NEEDLE processes them. |
| **Merge queue** | The review-and-merge path expressed as review-type beads blocking implementation beads. HOOP can draft review beads; it doesn't close them. |

Where Gas Town's Mayor commands polecats to do work, HOOP's Mayor **asks for work** by writing beads. The polecats decide whether and when to claim them based on NEEDLE's own logic. The separation of concerns: NEEDLE owns execution; HOOP owns the interface to it.

This is a deliberate narrowing of HOOP's original scope. Earlier drafts had HOOP steering workers, throttling on capacity, rotating accounts, releasing stuck claims. All of that is out — those concerns belong to NEEDLE (or to a future capacity-aware NEEDLE extension), not HOOP. HOOP becomes simpler and more focused: a reader that occasionally writes a single kind of record.

**Note on strands.** A NEEDLE worker's strand set (Pluck, Explore, Mend, Weave, Unravel, Pulse, Knot) is determined by the combination of *model* and *harness* at worker launch — not by beads, not by work items, not by any runtime decision. Strands are effectively worker-immutable once the worker is running. HOOP observes and displays which strand a worker is executing; HOOP never attempts to route work by strand, and never assumes a per-bead strand preference.

---

## 1.6. Stitches — the human-facing unit of work

**Humans work in Stitches, not beads.** Beads are NEEDLE's internal unit of execution. Stitches are HOOP's user-facing abstraction — a project-scoped, human-titled unit of work that may be backed by one or many beads.

When the operator expresses intent through HOOP ("investigate the kalshi-weather evening-window flake"), HOOP decomposes that intent into the beads NEEDLE needs and presents the result as a single **Stitch** the operator can track. Reverse direction: as beads are claimed, closed, fail, or spawn follow-ups, HOOP aggregates those events back into the Stitch's view. The operator sees "the kalshi-weather flake investigation is stuck on step 3"; the beads underneath are a mechanical detail.

**A Stitch has:**
- A human title ("Fix Calico IP selection on iad-acb")
- A status: `Planned | In Progress | Blocked | Awaiting Review | Done | Failed`
- A list of backing beads (in-flight, closed, planned)
- Optional child Stitches (multi-step work as a convoy)
- Aggregated cost, duration, outcome drawn from its beads
- **Stable identity across bead retries** — a new bead for a retried attempt attaches to the same Stitch

**Stitch ↔ bead mapping.** Every bead HOOP creates carries a `stitch:<stitch-id>` label. Follow-up beads created by NEEDLE workers inherit the parent bead's stitch label (one-line NEEDLE hook: copy the stitch label when creating a follow-up). Beads created outside HOOP remain orphan beads — visible, usable, unassociated. Optional clustering of orphans into synthetic Stitches (by title similarity and dep-graph connectivity) can come later.

**UI consequence.** Dashboards show Stitches. The Mayor speaks in Stitches. Bead IDs appear only in expert / debug / audit views. Users never need to know what a bead is to operate HOOP — they work with stitches, HOOP handles the decomposition.

**Why this matters.** Humans think at the scope of work ("fix the Calico IP selection"), not at the scope of bead IDs (`bd-3hv` + `bd-3hv-review` + `bd-3qvi-fix` + `bd-3qvi-followup`). NEEDLE's bead-level granularity is correct for machine execution and wrong for human cognition. The Stitch abstraction is pure UI / audit; NEEDLE itself keeps using beads internally. HOOP translates in both directions.

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
11. **Humans speak in Stitches, not beads.** HOOP's UI, Mayor, forms, and chat use project-scoped work items. The bead layer is preserved for machine correctness but hidden from normal operator flow. Bead IDs surface only in expert views and audit logs.

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
hoop mayor                    # attach to or start the Mayor conversation
hoop new <project>            # CLI shortcut to draft+submit a Stitch (decomposed into beads)
hoop stitch list [project]    # list open Stitches; use --beads for the underlying bead view
```

Notably absent: `launch`, `stop`, `salvage`, `steer`, anything that touches a worker process. Those verbs belong to NEEDLE.

`hoop serve` is the long-lived process; other subcommands are clients over a Unix socket (`~/.hoop/control.sock`).

### 4.2 Project registry

`~/.hoop/projects.yaml`, file-watched:

```yaml
projects:
  - name: ardenone-cluster
    path: /home/coding/ardenone-cluster
    beads: /home/coding/ardenone-cluster/.beads
    label: "Cluster config"
    color: "#8A2BE2"
  - name: miroir
    path: /home/coding/miroir
    beads: /home/coding/miroir/.beads
    label: "Meilisearch orchestrator"
    color: "#FF4500"
  - name: ibkr-mcp
    path: /home/coding/ibkr-mcp
    beads: /home/coding/ibkr-mcp/.beads
    label: "IBKR MCP server"
```

Hot-reloads on change. Each project gets its own per-project runtime.

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

The bead-creation endpoint is the only mutation. It calls `br create` against the project's workspace, records the mutation in `~/.hoop/fleet.db` audit table, and emits a `bead_created_by_hoop` event for downstream consumers (the Mayor records its own drafts; the UI records operator-initiated drafts).

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
- **Mayor chat** — operator ↔ Mayor, with multimodal input (phase 5)
- **Search** — cmd-K across projects and conversations
- **Audit** — read-only view of HOOP's own `br create` history

Streaming content lives in a separate reactive map. The committed-vs-streaming split is preserved.

### 4.6 Shared schema crate

`hoop-schema/` with JSON Schema source of truth. Rust types via `typify`; TS via `json-schema-to-typescript`. `schema_version: 1` on every record.

### 4.7 Stitch service

HOOP-owned state layer that translates between operator intent (Stitches) and NEEDLE's execution substrate (beads). Tables in `~/.hoop/fleet.db`:

```
stitches(id, project, title, description, status, created_by, created_at,
         closed_at, parent_stitch_id, template_id, attachments_path)

stitch_beads(stitch_id, bead_id, relationship)
  -- relationship ∈ {primary, review, fix, followup, retry-of}

stitch_events(ts, stitch_id, event, source, payload)
  -- event stream mirroring derived from NEEDLE events that touch
  -- this stitch's beads; used for the stitch timeline
```

The service exposes a read-and-derive API: given a stitch id, compute status from the states of its beads (`Planned` if no beads claimed; `In Progress` if any bead claimed; `Awaiting Review` if implementation beads closed and review beads open; `Done` if all closed with a merged/closed-no-artifact resolution; `Failed` if hard-ceilings exceeded). Aggregate cost and duration from the bead events.

**Decomposition.** When the operator submits Stitch intent through form/chat/Mayor, HOOP's decomposition service proposes a bead graph (e.g. an investigation Stitch → [one `task` bead with open-ended prompt, one `review` bead depending on it]). The operator confirms; HOOP issues `br create` calls labeled with the stitch id.

**Aggregation.** When a bead closes, HOOP emits a derived stitch event if the bead carries a stitch label. When a bead spawns a follow-up (via the NEEDLE hook that preserves stitch labels), the new bead auto-attaches. When a bead with no stitch label closes, it shows up as an orphan bead; operator can manually attach it to a Stitch if desired.

**No direct bead mutation.** The Stitch service never closes, updates, or releases a bead — it only creates them (via `br create`) and observes them. If a Stitch needs a new attempt, HOOP creates a *new* bead labeled with the same stitch id; the old failed bead stays closed as historical record.

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
(UI form or Mayor chat)
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
    - Saturation alerts when thresholds cross — shown in UI, surfaced to the Mayor in phase 5
    - **No actions** — HOOP does not pause, rotate, or throttle anything
11. **Visual debug panel** — per-bead step-through of what the polecat actually did: prompts sent, tool calls issued, results, stderr, state transitions. Scrubable timeline at the bead level. Reconstructed from events + tagged conversation transcript. Answers "what did this polecat actually do with my $2.80?" in one view.
12. Collision detector (observation only): alerts when active workers touch overlapping files.
13. Stuck detector (observation only): heartbeat-transition silence or repeated retries surfaced as alerts.

**Plus five marquee capabilities:**

14. **Stitch abstraction layer** (foundational). UI, forms, Mayor, and dashboards render Stitches. Bead-level views exist behind an "Expert" toggle. `stitch_service` as described in §4.7 lands here. Every new bead HOOP creates carries a `stitch:<id>` label. NEEDLE hook: follow-up beads inherit parent stitch label.
15. **Stitch-Provenance Code Archaeology** — the file preview panel overlays standard git blame with the *Stitch* that introduced each line. Hover → see the Stitch title, status, and conversation; click → jump to the Stitch view. Requires a NEEDLE hook to emit a `Bead-Id:` commit trailer on close (one line); HOOP maintains a bead-id → commit-sha index and joins that with stitch membership.
16. **Time-Machine UI Scrubber** — a global timeline slider at the top of any view re-renders full system state at the selected moment: Stitches, fleets, costs, capacity, file tree as of that moment. Leverages the event-as-authority invariant — all state is already derivable from event logs. Seek index lives in `fleet.db`; stale-state banner warns the operator they're viewing the past.
17. **Stitch Net-Diff Viewer** — when a Stitch reaches `Awaiting Review` or `Done`, HOOP computes the aggregate diff across every commit produced by the Stitch's beads (using the commit trailer from #15) and renders it as if it were a single PR: one tree, one narrative (Mayor-synthesized), one review surface. Replaces trawling PR history for multi-step convoys.
18. **Cost-Anomaly with Fix Lineage** — continuous detector flags Stitches whose observed cost exceeds the 2σ band for historically similar Stitches (lexical + embedding similarity). The alert is actionable: it surfaces the closest past matches, the Stitches that fixed them, and a pattern name. A curation UI lets operators tag recurring failure modes with recommended fix templates; the library compounds over time.

**Success criteria:**
- `hoop projects scan ~/` registers every workspace with `.beads/` in one command.
- Cost figures match `br`/provider summaries within ±2%.
- Capacity meters match Claude Code's `/status` within ±5% per account.
- Visual debug reconstructs a full bead cycle with no gaps (prompts + tools + outcome).
- Killing one project's runtime (delete `.beads/`) shows an error card; other projects unaffected.
- Dashboards contain zero bead IDs by default; toggling Expert view reveals them.
- A hover on any line in the file preview surfaces the Stitch that produced it within 200ms.
- Time-Machine scrubber moves to any point in the last 30 days and rerenders correctly within 2s.
- Stitch Net-Diff assembles correctly for a 5-bead, 11-commit convoy.
- Cost anomaly detector flags a synthetic 3σ test case with the right historical match.

### Phase 3 — File browser + artifact preview + multimodal (v0.3)

**Goal.** The operator can browse any project's files, preview code/docs/images/media with syntax highlighting, and supply multimodal input for bead drafting and Mayor conversations.

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
6. **Multimodal input to Mayor conversations:**
   - Same attachment types as bead drafts
   - Attachments pass to Claude Code via native multimodal support
   - Transcripts + metadata indexed for later search
7. **Streaming upload** for large files with progress + resumability.
8. **Path-sensitive routing:** drag a file from the tree into a bead draft → the draft picks up the path + current revision + snippet context.

**Plus one marquee capability:**

9. **Voice / Screen Work Capture.** Hotkey or phone-ADB trigger starts a capture session:
   - **Voice flow.** Push-to-talk on the Pixel 6 (ADB already present in this environment) or a browser hotkey records audio. Local Whisper transcribes to text + timestamps. The Mayor drafts a Stitch with the transcript as description and the audio attached. Operator walks back to keyboard, reviews the draft, confirms.
   - **Screen walkthrough.** Browser `MediaRecorder` captures screen + audio while the operator narrates a bug or explains context. Frame-sample + full audio + transcript all attach to a Stitch draft; Mayor synthesizes a title and body from the transcript.
   - Two minutes of walking-around thinking becomes a preview-ready Stitch sitting in the queue.

**Success criteria:**
- File browser usable on a 20k-file repo with <1s directory-expand latency.
- Syntax highlighting for at least: Rust, TS/JS, Python, Go, Clojure, YAML, TOML, Markdown, Shell, SQL, Dockerfile.
- Image/audio/video preview works in Safari, Chrome, Firefox.
- Attached 10MB image in a Stitch draft stored and referenced correctly.
- Mayor receives attachments in its conversation context.
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
4. **Chat-driven drafting** (precursor to the full Mayor in phase 5):
   - Lightweight chat pane per project (Haiku-class, not the Mayor)
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
    - **Continue in Mayor** — the Mayor inherits the reconstructed state in its conversation and can continue the work interactively with the operator.
    
    Only HOOP has the joined view (NEEDLE events + CLI session JSONL + worktree git state) needed to reconstruct the moment cleanly.

**Success criteria:**
- A form-drafted Stitch appears in NEEDLE's queue (as the underlying beads) and is claimed by a worker without human intervention.
- An audit row exists for every HOOP-created bead; `br` log shows the same creation.
- Chat-driven drafting produces a reasonable first Stitch draft for common intents ("review the last merge in project X", "investigate the Calico IP issue").
- Bulk draft correctly splits a 10-item markdown list into 10 previewable Stitch drafts.
- "What Will This Take?" estimates land within the p50 / p90 bands for 80% of closed Stitches after 30 days of operation.
- Already-Started Detection catches a synthetic duplicate across two projects with >95% recall at threshold.
- Stitch Replay reconstructs the failure moment and successfully resumes a bead that completes.

### Phase 5 — The Mayor (v0.5)

**Goal.** Seat the Mayor — a persistent, cross-project conversation partner who reviews artifacts, answers operator questions, and drafts beads. The Mayor is the operator's main interface to HOOP once it's seated.

**Deliverables:**

1. **Mayor session** — a long-running Claude Code session (Opus by default, configurable) hosted by HOOP as a first-class resource. Persists across HOOP restarts via Claude Code's native session store; HOOP tracks the session id.
2. **Mayor context (read-only via MCP server):**
   - Project registry + per-project status
   - Bead queue summaries per project (open / claimed / blocked / recently closed)
   - Worker liveness (observed, not controlled)
   - Conversation archive search
   - File tree + file read + grep per project
   - Cost + capacity roll-ups
   - Visual-debug reconstructions per bead
   - Recent audit log
3. **Mayor tool belt — one write, many reads:**
   - `create_stitch(project, title, description, kind, attachments[])` — the one write. Internally decomposes to one or more `br create` calls; the Mayor speaks in Stitches, not beads.
   - `find_stitches(project, filter)`, `read_stitch(id)` (returns the aggregated view)
   - `find_beads(project, filter)`, `read_bead(id)` — expert-only, available but rarely needed
   - `read_file(project, path, revision)`, `grep(project, pattern)`
   - `search_conversations(query, project?)`
   - `summarize_project(project)`, `summarize_day()`
   - `escalate_to_operator(message)` — UI banner; no auto-actions
   - **No** `launch_fleet`, `stop_fleet`, `release_claim`, `boost_priority`, `close_stitch`, `close_bead`. If the Mayor concludes work needs stopping or a Stitch needs closing, it escalates to the operator.
4. **Notification channel.** When a fleet closes a bead, completes a convoy, hits a capacity threshold, or surfaces a stuck-worker alert, the Mayor receives a structured event. Mayor decides whether to surface it to the operator.
5. **Operator ↔ Mayor chat pane** — primary UI surface. Cross-project by design. Multimodal input (from phase 3). Streams in real time.
6. **Bead drafts via Mayor** route through phase 4's preview flow — no direct submits.
7. **Mayor-off switch.** HOOP remains fully functional without a Mayor. Enabling the Mayor requires an Anthropic API key or a configured Claude Code account and explicit config.
8. **Mayor audit trail.** Every Mayor-drafted bead carries `actor: hoop:mayor:<session>` in the audit log, with a link to the chat turn that produced it.

**Plus two marquee capabilities:**

9. **Morning Brief** — at operator login (or a configured time), the Mayor autonomously reviews overnight activity across every project and produces a structured briefing:
   - What closed successfully, what failed (with cost impact), what's stuck, what's anomalous (via #18 cost lineage), what's blocked on human input
   - Pre-drafted Stitches (always unsubmitted, always preview flow) for follow-ups the Mayor thinks are important
   - Cross-project propagation suggestions (see #10)
   - **One headline** — the single thing the Mayor thinks should take priority today, with evidence
   
   Turns the "what happened overnight?" question from a 20-minute trawl into a two-minute read. Realizes Gas Town's "Mayor kicks off most work" within HOOP's read-and-draft scope.

10. **Cross-Project Stitch Propagation.** The Mayor recognizes when a fix pattern applied in one project has structural siblings in other projects (same config shape, same file layout, same dependency, similar recent failure signals). Surfaces: "you just closed `fix Calico IP selection` in `iad-acb`. The same pattern exists in `iad-ci`, `rs-manager`, `ardenone-cluster`. Draft matching Stitches for each?" Always preview; operator accepts per-project or all-at-once. Uniquely HOOP because cross-project visibility is HOOP's core position — no single-project tool can make this connection.

**Success criteria:**
- Mayor session survives `systemctl restart hoop` with full context intact.
- Operator asks "what did we do today across all projects?" and gets a coherent cross-project summary in Stitch language.
- Operator asks "something feels off on kalshi-weather" and the Mayor reviews recent Stitches, conversations, and files; responds with a focused answer and (if warranted) a drafted Stitch for the operator to review.
- Mayor never performs a worker action (no launch/stop/release/close). Attempts to ask for such actions produce an explanation pointing at `br` or NEEDLE.
- Morning Brief produces a useful daily summary and at least one correctly-scoped pre-drafted Stitch per typical overnight run.
- Cross-Project Propagation catches a real fix-sibling across 3+ projects with operator-rated useful accuracy (tracked manually over first 30 days).
- Mayor audit log lets the operator reconstruct any drafted Stitch back to the chat turn that produced it.

### Phase 6 — Operational polish (v0.6)

Make HOOP pleasant to run for the long haul.

**Deliverables:**

1. systemd user service template
2. Config hot-reload (projects.yaml, templates, Mayor config)
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
| 1 | Stitch abstraction layer | 2 | Humans work in project-scoped Stitches; beads stay hidden |
| 2 | Stitch-Provenance Code Archaeology | 2 | git blame with the *Stitch* that introduced each line |
| 3 | Time-Machine UI Scrubber | 2 | Drag a slider; the whole UI re-renders state at that past moment |
| 4 | Stitch Net-Diff Viewer | 2 | Multi-bead convoys reviewed as one unified PR-like surface |
| 5 | Cost-Anomaly with Fix Lineage | 2 | Over-cost Stitches link to past matches and recommended fixes |
| 6 | Voice / Screen Work Capture | 3 | Describe work by voice or screencast; HOOP drafts the Stitch |
| 7 | "What Will This Take?" Preview | 4 | Cost / duration / risk preview before submitting a Stitch |
| 8 | Already-Started Detection | 4 | Semantic check catches duplicates across projects at draft time |
| 9 | Stitch Replay from Failure Point | 4 | Reconstruct a failed Stitch's state and resume from there |
| 10 | Morning Brief | 5 | The Mayor's daily briefing + pre-drafted Stitches + one headline |
| (10+) | Cross-Project Stitch Propagation | 5 | Mayor suggests matching fixes for sibling projects |

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
| Mayor host | Claude Code via persistent session | Matches how operator already uses Claude elsewhere |
| Mayor context | MCP server exposing HOOP's read APIs + one write (`create_bead` → `br create`) | Clean auth boundary |
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

---

## 9. Open questions

1. **Hot-reload granularity** for `projects.yaml`: partial reload or full project-runtime restart? Lean partial. Resolve in phase 2.
2. **Mayor: direct Claude Code process or headless session?** Lean direct — spawn a `claude` subprocess HOOP owns, keep its session id, reattach on restart. Resolve in phase 5.
3. **Mayor MCP server: in-process or separate?** Lean separate binary `hoop-mcp` so HOOP doesn't depend on MCP libraries at all. Resolve in phase 5.
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
| v0.5 | +28 weeks | The Mayor: persistent Claude Code session + MCP tool belt + morning brief + cross-project propagation |
| v0.6 | +32 weeks | Operational polish: systemd, hot-reload, backups, metrics, Tailscale identity |
| v1.0 | +36 weeks | Multi-operator with viewer/drafter roles; public README |

Dates are planning fiction; ordering matters. **Do not build Stitch creation before observability is real.** Drafting work against a backlog the operator can't inspect is worse than no drafting at all. **Do not seat the Mayor before direct Stitch creation and the marquee observability features exist** — the Mayor is a productivity multiplier on top of those surfaces; without them it's a chatbot without tools.

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
                │  Mayor (phase 5+)               │────── URL bridge ──► FABRIC
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

## 12. Appendix — Kubernetes worker deployment (someday, sketched)

If the EX44 saturates, NEEDLE can graduate worker execution to Kubernetes pods — this is a NEEDLE concern, not a HOOP concern. HOOP's role when that happens is the same as today: read the bead events (now streamed from cluster sidecars into a shared log or event bus), offer UI, create beads on operator intent. HOOP does not become a cluster controller.

Rackspace-spot-terraform automation was retired 2026-04-22; spot clusters are now manually provisioned — fine for this deferred work since cluster churn is no longer a background concern.

The trigger for this extension is "NEEDLE's fleet needs to leave the host," not "HOOP wants to be K8s-aware." HOOP's design doesn't change when execution moves.
