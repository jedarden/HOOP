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

---

## 2. The environment HOOP targets

- **Host:** Hetzner EX44-class. Bare metal, long-lived, Tailscale-only.
- **Workspaces:** `~/` holds 5–25 repos. Each has a `.beads/` directory; git worktrees live under `.worktrees/`; some may carry a `fleet.yaml` describing NEEDLE's worker pool (HOOP reads it, doesn't act on it).
- **Tooling:** `br` CLI in PATH. Five CLI adapters installed and credentialed (operator's cache, not HOOP's). NEEDLE installed and running its own workers through its own supervision. git 2.5+.
- **Process model:** `hoop serve` as a systemd user service on a Tailscale hostname. No tmux spawning by HOOP.
- **Parallel workloads:** NEEDLE fleets in tmux (HOOP-observed); ad-hoc CLI sessions in separate terminals (HOOP-observed); everything else (HOOP-ignored).

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
hoop bead new <project>       # CLI shortcut to draft+submit a bead
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

**Success criteria:**
- `hoop projects scan ~/` registers every workspace with `.beads/` in one command.
- Cost figures match `br`/provider summaries within ±2%.
- Capacity meters match Claude Code's `/status` within ±5% per account.
- Visual debug reconstructs a full bead cycle with no gaps (prompts + tools + outcome).
- Killing one project's runtime (delete `.beads/`) shows an error card; other projects unaffected.

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

**Success criteria:**
- File browser usable on a 20k-file repo with <1s directory-expand latency.
- Syntax highlighting for at least: Rust, TS/JS, Python, Go, Clojure, YAML, TOML, Markdown, Shell, SQL, Dockerfile.
- Image/audio/video preview works in Safari, Chrome, Firefox.
- Attached 10MB image in a bead draft stored and referenced correctly.
- Mayor receives attachments in its conversation context.

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
3. **Submit flow:** draft → preview → `br create --json <payload>` against project cwd → audit row → event emitted → UI redirects to the new bead's view.
4. **Chat-driven drafting** (precursor to the full Mayor in phase 5):
   - Lightweight chat pane per project (Haiku-class, not the Mayor)
   - Takes natural language, produces a draft
   - Never submits directly — always routes through the preview flow
5. **Bulk draft:** paste a bullet list or a markdown doc; HOOP splits it into multiple drafts for review + submit.
6. **Audit trail:** every created bead has `created_by: hoop` + operator identity + source (`form` / `chat` / `bulk` / `template:<name>`) recorded in `fleet.db` actions table.
7. **Explicit non-actions:** HOOP does not `close`, `update`, `claim`, `release`, `depend`, or any other `br` verb beyond `create`. If the operator needs those, they use `br` directly.

**Success criteria:**
- A form-drafted bead appears in NEEDLE's queue and is claimed by a worker without human intervention.
- An audit row exists for every HOOP-created bead; `br` log shows the same creation.
- Chat-driven drafting produces a reasonable first draft for common intents ("review the last merge in project X", "investigate the Calico IP issue").
- Bulk draft correctly splits a 10-item markdown list into 10 previewable drafts.

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
   - `create_bead(project, title, body, deps[], priority, attachments[])` — the one write
   - `find_beads(project, filter)`, `read_bead(id)`
   - `read_file(project, path, revision)`, `grep(project, pattern)`
   - `search_conversations(query, project?)`
   - `summarize_project(project)`, `summarize_day()`
   - `escalate_to_operator(message)` — UI banner; no auto-actions
   - **No** `launch_fleet`, `stop_fleet`, `release_claim`, `boost_priority`, `close_bead`. If the Mayor concludes work needs stopping or a bead needs closing, it escalates to the operator.
4. **Notification channel.** When a fleet closes a bead, completes a convoy, hits a capacity threshold, or surfaces a stuck-worker alert, the Mayor receives a structured event. Mayor decides whether to surface it to the operator.
5. **Operator ↔ Mayor chat pane** — primary UI surface. Cross-project by design. Multimodal input (from phase 3). Streams in real time.
6. **Bead drafts via Mayor** route through phase 4's preview flow — no direct submits.
7. **Mayor-off switch.** HOOP remains fully functional without a Mayor. Enabling the Mayor requires an Anthropic API key or a configured Claude Code account and explicit config.
8. **Mayor audit trail.** Every Mayor-drafted bead carries `actor: hoop:mayor:<session>` in the audit log, with a link to the chat turn that produced it.

**Success criteria:**
- Mayor session survives `systemctl restart hoop` with full context intact.
- Operator asks "what did we do today across all projects?" and gets a coherent cross-project summary.
- Operator asks "something feels off on kalshi-weather" and the Mayor reviews recent beads, conversations, and files; responds with a focused answer and (if warranted) a drafted bead for the operator to review.
- Mayor never performs a worker action (no launch/stop/release/close). Attempts to ask for such actions produce an explanation pointing at `br` or NEEDLE.
- Mayor audit log lets the operator reconstruct any drafted bead back to the chat turn that produced it.

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
| HTTP / WS server | `axum` | Standard async stack; embedded static |
| File watching | `notify` | Cross-platform; reliable |
| UI | React + Vite + TypeScript + Jotai | Matches team skill; keyed atoms fit streaming split |
| Syntax highlighting | `syntect` (server) + Shiki (client) | Server-rendered for large files; client for interactive |
| Schema | JSON Schema draft-07 + `typify` + `json-schema-to-typescript` | One source of truth |
| Storage (HOOP) | SQLite (audit log + conversation index only) | Small; portable |
| Event transport (local) | File tail (`notify`) | Cheapest reliable option |
| Mayor host | Claude Code via persistent session | Matches how operator already uses Claude elsewhere |
| Mayor context | MCP server exposing HOOP's read APIs + one write (`create_bead`) | Clean auth boundary |
| Audio transcription | Whisper via local model or Anthropic's transcription endpoint | Multimodal input searchability |
| Service supervisor | systemd (user-scope) | Standard |
| Auth | Tailscale identity via whois | Matches environment |

---

## 8. Non-goals

Explicit. HOOP deliberately does not grow into these.

1. **Orchestrating work.** NEEDLE does this.
2. **Steering workers.** No launch, stop, kill, pause, signal, SIGSTOP, SIGTERM, release-claim, reassign, or any other action that touches a worker process or bead lifecycle.
3. **Capacity enforcement.** HOOP shows utilization; it never throttles, rotates, or pauses on thresholds. Enforcement, if needed, belongs in NEEDLE or a dedicated layer.
4. **Mutating bead state.** Only `br create` is HOOP's write. No close, update, depend, claim, release.
5. **Storing bead state.** `br` owns it.
6. **Replacing FABRIC.** FABRIC read-only, deployable anywhere; HOOP local-host with one write. URL bridge links them.
7. **Multi-host control.** One HOOP, one host. Growth is more projects, not more hosts.
8. **RBAC beyond viewer/drafter.**
9. **Secrets management.** Credentials live in each CLI's native cache.
10. **Browser-only.** HOOP needs a server — it reads filesystem and shells to `br`.

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
| v0.2 | +10 weeks | Multi-project; cost + capacity visibility; visual debug; collision/stuck detection |
| v0.3 | +14 weeks | File browser, syntax highlighting, multimodal attachments |
| v0.4 | +18 weeks | Bead-creation: form + chat + templates + bulk; audit log |
| v0.5 | +24 weeks | The Mayor: persistent Claude Code session, MCP-backed context, drafts through preview flow |
| v0.6 | +28 weeks | Operational polish: systemd, hot-reload, backups, metrics, Tailscale identity |
| v1.0 | +32 weeks | Multi-operator with viewer/drafter roles; public README |

Dates are planning fiction; ordering matters. **Do not build bead creation before observability is real.** Drafting beads against a backlog the operator can't inspect is worse than no drafting at all.

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
