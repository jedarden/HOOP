# HOOP implementation plan

**Status:** Draft v3 — 2026-04-22
**Scope:** Control plane for a single long-lived host that holds many repos, many NEEDLE fleets, and many native-CLI conversations. Kubernetes worker deployment is a deferred aspiration, not part of the primary roadmap — sketched at the end so the v1 design isn't warped by it.

**Lineage.** Two explicit influences:
- **Steve Yegge's Gas Town** — the "city hosts a Mayor who orchestrates polecats working in swarms" mental model. NEEDLE workers are HOOP's polecats; HOOP's central coordinating agent is its Mayor. Gas Town is built on Beads (same `br` substrate NEEDLE uses), which makes the conceptual translation direct.
- **The prior-art reference ADE ([notes/](../notes/))** — swarm-first design, git-worktree isolation, multi-provider dynamic routing within one swarm, visual debugging of prompts/tool-calls/state, BYO-account cost reduction. HOOP adopts these patterns against the NEEDLE substrate rather than reinventing them.

---

## 1. Vision

HOOP is the operator's pane of glass for a server that does real work. On a single long-lived host like this Hetzner EX44:

- A dozen repos live on disk, each at its own path, each with its own `.beads/` queue.
- Several NEEDLE fleets run at once, each scoped to a workspace, worker processes living in named tmux sessions.
- The operator also runs ad-hoc `claude` / `codex` / `opencode` / `gemini` / `aider` sessions in their own terminals, often in the same repos the fleets are working in.
- kubectl, `br`, ArgoCD read-only calls, ADB to the phone, and a half-dozen other tools run side-by-side.

HOOP makes sense of all of it. One URL over Tailscale shows every project, every fleet, every conversation, every cost figure, every stuck worker. From the same URL the operator launches fleets, steers them, or reassigns work. HOOP never owns any of the data — bead state stays in `br`, conversation state stays in each CLI's native directory — it joins those sources so the operator doesn't have to.

The unit of growth here is *another project on the same host*, not *another host*. Everything in v1 assumes the host is singular and long-lived.

---

## 1.5. Mental model: the city, the Mayor, the polecats

Borrowed from Steve Yegge's Gas Town vocabulary because the mapping is clean and the terms hold up under load. HOOP is the **city** — the long-lived infrastructure that houses roles, not a role itself:

| Gas Town term | HOOP-over-NEEDLE mapping |
|---|---|
| **City** | The HOOP daemon on the EX44. Infrastructure: project registry, event streams, audit log, UI. |
| **Mayor** | A persistent, long-running Claude Code session (Opus-class) hosted by HOOP with cross-project context and a tool-belt of HOOP operations. The operator's primary conversation partner. Receives notifications, kicks off work, escalates stuck beads, coordinates across projects. |
| **Polecat** | A single NEEDLE worker claiming and executing one bead. Ephemeral per-bead lifecycle (claim → dispatch → execute → close → repeat). |
| **Swarm** | A NEEDLE fleet — a named group of polecats working the same workspace under one `fleet.yaml`. |
| **Convoy** | A coordinated multi-bead work unit (e.g. "refactor the auth module" → implementation bead + review bead + fix beads) expressed through the bead dependency graph. |
| **Merge Queue** | The review-and-merge path: review-type beads blocking implementation beads, with PR merges driven by closure resolution. |

The Mayor is the operator's handle on the city. The polecats are the city's workforce. The city is what both of them live in. When we talk about HOOP's roadmap, phases 1-2 are **building the city** (infrastructure, observability, multi-project correctness); phases 3-4 are **seating the Mayor and giving them tools** (steering, Mayor persistence, visual debug, cost controls); phase 5 is **letting more than one citizen live there** (multi-operator).

The Mayor is not optional polish. It is the difference between "a dashboard that shows fleets" and "a coordinator you talk to who runs fleets." The second is the actual goal.

---

## 2. The environment HOOP targets

- **Host:** Hetzner EX44-class (or equivalent). Bare metal, long-lived, rebooted rarely, Tailscale-only. Same shape as this coding environment.
- **Workspaces:** `~/` holds 5–25 repos. Each has a `.beads/` directory, git worktrees live under `.worktrees/` or similar, and may carry a `fleet.yaml` describing its NEEDLE worker pool.
- **Tooling:** `br` CLI in PATH. Five CLI adapters installed and credentialed: Claude Code, Codex, OpenCode, Gemini, Aider. Credentials persist in each CLI's native cache; HOOP never handles them.
- **Process model:** `hoop serve` as a systemd user service, listening on a Tailscale hostname (e.g. `hoop.tail1b1987.ts.net`). tmux available for worker supervision. git 2.5+ for worktrees.
- **Parallel workloads:** NEEDLE fleets in tmux (HOOP-aware); ad-hoc CLI sessions in separate operator terminals (HOOP-observed but not controlled); kubectl / ArgoCD / backup jobs / everything else (HOOP-ignored).

What changes when you move from "one workspace" to "multi-project on one host" is: **everything**. Session discovery has to be scoped per project or cross-project queries return soup. Cost aggregation has to bucket per project or you can't tell which repo cost what. Steering has to respect project boundaries or a boost-priority command leaks across workspaces. Worker supervision has to isolate failures per project so a crash in one fleet doesn't cascade. This plan treats multi-project correctness as the primary design pressure, not a bolt-on.

---

## 3. Principles

Locked in from day one. These come directly from the prior-art problems-and-solutions note and NEEDLE's existing invariants.

1. **Events are authoritative; projections are derived.** HOOP writes event rows and reads projections. No `fleet_status.json` that goes stale.
2. **Liveness = process, never file.** `kill -0 pid && !stopped_record`.
3. **Server is the epoch.** Client rebuilds state from scratch on every reconnect.
4. **Dual-identity in schema.** Bead id (stable) + provider session id (derived). Explicit `session_bound` event at first join.
5. **JSON Schema as cross-repo contract.** Every record has `schema_version: 1`. TS and Rust types codegen off one source.
6. **Atomic `.tmp` + rename for every write.** Line-buffered NDJSON reader for every read.
7. **Never silent-drop unknown events.** Log, emit progress, count.
8. **Never mutate bead state directly.** Use `br` CLI verbs. Every write audited with `actor: hoop:<user>`.
9. **Workers are independent.** If HOOP dies, workers keep working. If one project runtime dies, others keep running.
10. **Read-only is the default.** Steering is opt-in per workspace.

---

## 4. Component architecture

### 4.1 Single-binary HOOP daemon

One Rust binary. Subcommands:

```
hoop serve                    # run the control-plane daemon
hoop projects add <path>      # register a workspace
hoop projects scan <root>     # auto-register every workspace with .beads/
hoop projects list            # list registered projects
hoop projects remove <name>
hoop launch <project>         # spawn the fleet from <project>/fleet.yaml
hoop stop <project>           # stop workers for a project
hoop salvage <project>        # resume dead workers whose worktrees are still intact
hoop status [project]         # CLI view of active fleets
hoop audit                    # startup binary/permission audit
hoop steer                    # open an interactive steering chat session
```

`hoop serve` is the long-lived process. Every other subcommand is a client that speaks to it over a local Unix socket (`~/.hoop/control.sock`). CLI clients never touch project state directly.

### 4.2 The project registry

One file, `~/.hoop/projects.yaml`, authoritative:

```yaml
projects:
  - name: ardenone-cluster
    path: /home/coding/ardenone-cluster
    beads: /home/coding/ardenone-cluster/.beads
    label: "Cluster config"
    color: "#8A2BE2"
    fleet_manifest: fleet.yaml       # resolved relative to path
  - name: miroir
    path: /home/coding/miroir
    beads: /home/coding/miroir/.beads
    label: "Meilisearch orchestrator"
    color: "#FF4500"
  - name: ibkr-mcp
    path: /home/coding/ibkr-mcp
    beads: /home/coding/ibkr-mcp/.beads
    label: "IBKR MCP server"
    steering: read-only              # defaults: steering permitted
```

The daemon watches this file with `notify`. Additions spin up a new per-project runtime within 5s; removals tear one down gracefully. Renames are a remove + add.

Explicit naming (`name:` field) is required — it's the stable id used in URLs, logs, and audit rows. Paths can move; names shouldn't.

### 4.3 Per-project runtime

One per registered project, running inside the daemon as its own supervised task. Each runtime owns:

```
┌──────────────────────────────────────────────────────────────┐
│ Per-project runtime (example: "ardenone-cluster")             │
│                                                               │
│  Inputs (disk)                                                │
│  ├─ .beads/events.jsonl       ──► event tailer               │
│  ├─ .beads/heartbeats.jsonl   ──► heartbeat monitor          │
│  ├─ .beads/beads.db           ──► bead state reader (via br) │
│  ├─ tmux sessions "needle-<project>-*"  ──► worker supervisor│
│  └─ CLI session dirs, filtered by cwd-under-path:            │
│     ~/.claude/projects/<hash-of-path>/                        │
│     ~/.codex/sessions/**     (filtered)                       │
│     ~/.gemini/tmp/**         (filtered)                       │
│     ~/.local/share/opencode/ (filtered)                       │
│                                                               │
│  In-memory projections                                        │
│  ├─ active_beads: Map<bead_id, ActiveBead>                   │
│  ├─ workers: Map<worker_name, WorkerState>                   │
│  ├─ conversations: Map<conv_id, Conversation>                │
│  │   (split: fleet vs ad-hoc, by prefix-tag presence)        │
│  └─ cost_today: AdapterCostMap                               │
│                                                               │
│  Outputs                                                      │
│  ├─ WS fan-out with topic <project>                           │
│  ├─ REST endpoints prefixed /api/p/<project>/...              │
│  └─ Audit rows in fleet.db (global)                          │
└──────────────────────────────────────────────────────────────┘
```

Runtimes are fully isolated. A panic in one project's runtime is caught by its supervisor, logged, and the runtime restarts; other projects are unaffected. If a project's `.beads/` is unreachable or malformed, its card in the UI shows the error — other cards stay healthy.

### 4.4 Cross-project state

Kept deliberately thin. The daemon's global state is:

- **Project registry** (the `projects.yaml` view)
- **Runtime status** per project (running, degraded, stopped, error)
- **Audit log** (actions table in `fleet.db`)
- **Cost roll-ups** (per project, per adapter, per day)
- **Collision index** (files touched across all active workers — detects cross-project file overlap, rare but real, e.g. a shared monorepo slice)

No conversation content, no bead content, no transcripts at the global layer. The UI aggregates from per-project topic streams on demand.

### 4.5 Web client

React + Vite + TypeScript + Jotai. Served by the daemon itself (embedded static assets). Key surfaces:

- **Overview** — fleet-of-fleets dashboard, one card per project, aggregate stats.
- **Project detail** — fleet map, bead graph, worker timeline, conversation list, cost panel, project-scoped chat pane.
- **Conversations** — cross-project transcript viewer with project filter.
- **Search** — cmd-K palette, substring across all projects, 50-result cap, project badges on results.
- **Audit** — read-only view of the global actions log.
- **Steering chat** — project-scoped or global, with a tool-belt agent (phase 3).

Atoms are keyed by `(project, id)` so switching projects is a filter, not a reload. Streaming content lives in a separate reactive map (never mixed with committed messages) — the in-flight isolation rule.

### 4.6 Shared schema crate

- `hoop-schema/` with JSON Schema source of truth.
- Rust types via `typify`, TS types via `json-schema-to-typescript`.
- `schema_version: 1` on every record from day one.

---

## 5. Data flows

### 5.1 Event / session / heartbeat (single project)

```
NEEDLE worker (tmux: needle-ardenone-cluster-alpha)     HOOP daemon
────────────────────────────────────────────────        ───────────
br claim bd-abc
   │
   ├─► .beads/events.jsonl  ◄── tail -F ──► [ardenone-cluster runtime]
   │     {event: claim, ...}                      │
   │                                              ├─► /ws fan-out
   ▼                                              │   (topic: ardenone-cluster)
dispatch via YAML adapter                         │
(prompt prefixed with
 [needle:alpha:bd-abc:pluck])                     │
   │                                              │
   ▼                                              │
CLI writes to                                     │
~/.claude/projects/-home-coding-ardenone-         │
cluster/<session>.jsonl                           │
   │                                              │
   │                                    ◄─ 5s poll ── session tailer
   │                                              │       │
   │                                              │       ├─► tag-join
   │                                              │       │   (tag → bead alias)
   │                                              │       │
   │                                              │       └─► /ws conversation
   ▼
CLI exits → br close / fail
   │
   └─► .beads/events.jsonl ◄── tail -F ──────────►
         {event: complete, outcome, duration, tokens}
                                                  │
                                                  ├─► cost aggregator
                                                  └─► /ws fan-out

heartbeat every 10s ──► .beads/heartbeats.jsonl ──► heartbeat monitor
                                                  │
                                                  └─► liveness projection
```

### 5.2 Multi-project fan-out

```
                    ┌─ ardenone-cluster runtime ────────┐
                    │ event tailer / session tailer ...  │─┐
                    └────────────────────────────────────┘ │
                                                           │
                    ┌─ miroir runtime ───────────────────┐ │
                    │ event tailer / session tailer ...  │─┤
                    └────────────────────────────────────┘ │
                                                           │
                    ┌─ ibkr-mcp runtime ─────────────────┐ │
                    │ event tailer / session tailer ...  │─┤
                    └────────────────────────────────────┘ │
                                                           │
                                                           ▼
                                            ┌──────────────────────────┐
                                            │ Global state + WS server │
                                            │                          │
                                            │  - project_status{}      │
                                            │  - cost_rollup{}         │
                                            │  - collision_index{}     │
                                            │  - actions log (SQLite)  │
                                            └──────────────┬───────────┘
                                                           │
                                                        /ws
                                                           │
                                                           ▼
                                                    Web client
                                                 (subscribes to
                                                  topics by project
                                                  or to "global")
```

### 5.3 Ad-hoc vs fleet conversations

When the operator runs `claude` directly in `~/ardenone-cluster/` (not via NEEDLE), the CLI writes to `~/.claude/projects/-home-coding-ardenone-cluster/`. The session tailer picks it up like any other file.

Classification:

- **First user message starts with `[needle:<worker>:<bead-id>:<strand>]`** → fleet worker conversation. Joined to bead; counted in fleet analytics; appears under "workers" in the project view.
- **No such prefix** → ad-hoc operator conversation. Shown under the project (because it was run in that project's cwd) but tagged `actor: operator`; excluded from fleet cost and analytics; not claimable for steering.

Both are viewable; analytics separates them. The operator's ad-hoc work is visible to HOOP, never *owned* by HOOP.

### 5.4 Session tailer filtering

Each CLI's session directory is global on the host (one `~/.claude/projects/` serves all repos). Each project's session tailer must filter by cwd-match against the project's `path`. Claude Code already encodes the working directory in its per-session directory name (path with `/` → `-`), so the filter is a prefix match. Codex/OpenCode/Gemini have their own conventions; each `DiskAdapter` implements project-scoping.

**Important:** sessions from outside any registered project (e.g. operator runs `claude` in `/tmp/scratch/`) are surfaced in an "Unassigned" bucket, not silently dropped. The operator can assign or ignore.

---

## 6. Phased roadmap

### Phase 0 — Foundation (COMPLETE)

Repo created, docs scaffolded, notes seeded with prior-art research, plan published.

### Phase 1 — Single-host daemon, one workspace (v0.1)

**Goal.** HOOP runs on the EX44, reads one workspace's NEEDLE events, launches and supervises workers locally via tmux, serves a web UI. Read-only. Prove the end-to-end loop works before adding breadth.

**Deliverables.**

1. Rust binary with `serve`, `projects add`, `launch`, `stop`, `status`, `audit`.
2. Single-project registry entry; multi-project is phase 2.
3. Per-project runtime covering event tailer, session tailer, heartbeat monitor, worker supervisor.
4. Tag-join resolver using `[needle:<worker>:<bead-id>:<strand>]`.
5. Web UI: fleet map, bead list (tabular, no graph yet), worker timeline, conversation viewer with fleet/ad-hoc split, audit overlay, search palette (single project).
6. `~/.hoop/fleet.db` SQLite for launches, workers, actions audit.
7. Three NEEDLE hooks landed:
   - Prompt prefix tag
   - `events.jsonl` append helper
   - Heartbeat thread
8. Worker spawn verification: each worker writes `~/.hoop/workers/<name>.ack` within 10s; missing ack = failed spawn with a diagnostic.
9. Graceful shutdown: SIGTERM → grace window → SIGKILL. Claim released before any forced kill.
10. Startup audit: `br`, tmux, CLI binaries, project's `.beads/` accessibility. Blocking overlay on failure.

**Non-goals.** Multi-project. Steering. Dependency graph. Cost aggregation.

**Success criteria.**
- Spawn a 3-worker fleet from `fleet.yaml`; all three visible on the map within 10s.
- Kill a tmux session externally; HOOP reports dead within 30s and releases the claim.
- Restart `hoop serve`; reconnecting UI rebuilds state entirely from disk.
- Open any worker's transcript; see the bead id in the header (tag-join working).
- Zero silent drops: every unknown event appears in the UI's diagnostic panel.

### Phase 2 — Multi-project (v0.2)

**Goal.** One HOOP serves every project on the host. The primary bet of this plan.

**Deliverables.**

1. Project registry (`~/.hoop/projects.yaml`) with `add`, `remove`, `scan`, file-watch hot-reload.
2. Per-project runtime isolation — panic in one runtime doesn't touch others; restart-on-failure.
3. Fleet-of-fleets dashboard: one card per project, showing worker count, active beads, cost today, stuck count, last activity.
4. Project detail view: fleet map, bead graph (DAG, colored by state), strand timeline, conversation list, cost breakdown.
5. Cross-project dashboards:
   - Total spend today, this week, this month, bucketed by project and by adapter
   - Total workers running across all projects
   - Longest-running beads across projects
   - Most-active adapter today
6. Ad-hoc vs fleet classification on every conversation, with filter controls in the UI.
7. Unassigned-conversation bucket for sessions outside any registered project.
8. Search palette across all projects, results badged by project.
9. Collision detector: active workers touching the same file paths, across or within projects.
10. Stuck detector: no heartbeat transition for N minutes, or repeated failure on the same bead.
11. Cost panel:
    - Per-project, per-adapter, per-model, per-strand, per-day
    - Rate-limit window overlay for Claude (5h + 7d)
    - Cost-per-closed-bead (the real unit-economics number)
    - Ad-hoc cost separated from fleet cost
12. **Visual debug panel** (from unleashd's playbook) — per-bead step-through of the polecat's actual work: prompts sent, tool calls issued, state transitions, stderr. Scrubable timeline at the bead level, not just the conversation level. Answers "what did this polecat actually do with my $2.80?" in one view.
13. **Multi-account awareness.** An adapter config can declare multiple accounts (e.g. two Claude Max plans); HOOP tracks usage per account and surfaces which account is being drained toward a rate-limit window. No routing yet — just visibility.

**Success criteria.**
- EX44 with `hoop projects scan ~/` registers every workspace with `.beads/` in one command.
- Cost figures match `br`/provider summaries within ±2%.
- DAG renders 500+ beads interactively (<500ms interactions).
- Collision alert fires within 30s of two active workers touching the same file.
- Killing one project's runtime (e.g. delete its `.beads/`) shows an error card but leaves other projects unaffected.
- Visual debug panel reconstructs a full bead cycle from events + session transcript with no gaps.

### Phase 3 — Direct steering + multi-provider routing (v0.3)

**Goal.** HOOP gains write-side operations and the ability to route work across providers within a single swarm. Direct bead ops, tmux controls, per-swarm multi-provider routing, rate-limit-aware scheduling. All scoped by project; all audited. This phase is the **operator's** write-side; the Mayor comes in phase 4.

**Deliverables.**

1. **Direct bead-op buttons** (scoped per project) wired to `br`:
   - Boost / lower priority
   - Close bead with resolution (`closed-no-artifact`, `merged`, `parked`, `rejected`)
   - Create review bead as dep of target
   - Release stuck claim (only when worker demonstrably dead AND heartbeat stale)
   - Reassign bead to a specific worker or adapter
2. **tmux control actions** (SIGSTOP / SIGCONT / SIGTERM). TERM gate: release claim first, always.
3. **Multi-provider dynamic routing within one swarm** (unleashd's pattern). `fleet.yaml` can declare a mixed worker pool — e.g. two Claude Opus planners, three Codex mid-tier executors, one OpenCode executor — and beads are matched to workers by bead-level hints (`difficulty`, `strand`, `needs_review`). Rate-limit-aware: when one account's 5h window is 80% exhausted, new beads route to a different account/adapter rather than stalling.
4. **BYO-account orchestration.** Adapter configs reference credentials that persist in the CLI's native cache (HOOP never handles secrets); HOOP only tracks *which account is active per worker at spawn time* and rotates workers onto a fresh account when rate-limits approach. Opt-in per workspace.
5. **URL-scheme bridge for FABRIC:** `hoop://<project>/release/bead/<id>`, etc. FABRIC renders action links; HOOP opens a confirmation dialog.
6. **Permission snapshot at worker spawn** (env + `settings.json` copy) — no live inheritance.
7. `yolo: true` at project-level requires a `sandbox:` config stanza.

**Success criteria.**
- Every mutation has a corresponding audit row with the correct project name.
- Release-stuck-claim refuses when the PID is still alive OR heartbeat is fresh.
- FABRIC URL click opens the right project's HOOP dialog.
- A 10-bead batch distributed across a 5-worker mixed-provider pool completes without rate-limit hits when sufficient headroom exists.
- Rate-limit-aware routing drops a draining account cleanly and promotes a fresh one within one bead cycle.

### Phase 4 — The Mayor (v0.4)

**Goal.** Seat the Mayor — a persistent, cross-project coordinating agent — and give the operator a proper conversational handle on the city.

This is where HOOP stops being a dashboard and becomes a coordinator. The Mayor is the difference.

**Deliverables.**

1. **Mayor session** — a long-running Claude Code session (Opus by default; configurable) hosted by HOOP as a first-class resource. State persists across HOOP restarts via Claude Code's native session store; HOOP just tracks the session id.
2. **Mayor context ingestion.** Via MCP server or prompt injection, the Mayor has read access to:
   - Project registry and per-project status
   - Current fleet state (who's running, what they're claiming, heartbeat freshness)
   - Bead queue summaries per project (open / claimed / blocked / recently closed)
   - Cost roll-ups (per project, per adapter, per day, rate-limit headroom)
   - Recent audit log entries
   - Stuck / collision / cost-anomaly alerts
3. **Mayor tool belt** — every direct-steering operation from phase 3, plus:
   - `launch_fleet(project)` / `stop_fleet(project)` / `salvage_fleet(project)`
   - `create_bead(project, title, body, deps[], priority)` — goal-oriented prompts only; no method-dictating
   - `spawn_polecat(project, adapter, strands[])` — incremental worker add
   - `escalate(bead, reason)` — attach a HUMAN-blocked note and notify the operator
   - `summarize_day()` — generate a day-in-review for the operator
4. **Notification channel.** When a fleet closes a bead (or a convoy completes, or a worker hits a ceiling), the Mayor gets a structured event and decides whether to surface it to the operator. Matches Yegge's "Mayor kicks off most work and receives notifications when convoys finish."
5. **Operator ↔ Mayor chat pane** — primary UI surface of v0.4. Operator types "pick up the ibkr-mcp backlog, prioritize anything blocking production" and the Mayor decides what to do. Streams in real time. Cross-project by design.
6. **Hard ceilings still enforced.** The Mayor cannot create beads exceeding `max_attempts`, `max_review_rounds`, `cycle_budget` without explicit operator confirmation via a UI prompt (not via chat).
7. **Mayor audit trail.** Every Mayor-initiated mutation records `actor: hoop:mayor` in the audit log, with a link back to the prompt turn that produced it. Reviewable after the fact.
8. **Mayor-off switch.** HOOP runs fine without a Mayor (phases 1-3 use-case stays valid). Turning the Mayor on requires an Anthropic API key and explicit config.

**Success criteria.**
- Mayor session survives `systemctl restart hoop` with full context intact (via persistent session id).
- Operator asks "what did we do today?" and gets a coherent cross-project summary.
- Operator asks "everything on project X looks stuck — fix it" and the Mayor releases dead claims, identifies the blocker, and either unblocks autonomously or escalates with a clear reason.
- Mayor never exceeds ceilings without explicit operator approval.
- Mayor audit log lets the operator reconstruct any mutation chain end-to-end.

### Phase 5 — Operational polish (v0.5)

**Goal.** Make HOOP pleasant to run for the long haul on a server the operator doesn't want to babysit.

**Deliverables.**

1. **systemd user service template.** `hoop.service`, `hoop-ui.service`, standard `journalctl -u hoop` flow. Restart-on-failure; rate-limit on restart-loop.
2. **Config hot-reload.** Changes to `projects.yaml`, per-project `fleet.yaml`, and per-adapter config reload without restart. Schema validates before apply; bad config is rejected with a diff-style error.
3. **Log rotation.** HOOP's own logs (separate from NEEDLE's) rotate at 100MB or daily, keep 14 days.
4. **Health endpoints.** `/healthz` (liveness) and `/readyz` (all runtimes started). Exposed on a separate non-WS port for monitors.
5. **State backup.** `fleet.db` snapshotted daily to `~/.hoop/backups/`. Restore flow documented.
6. **Upgrade flow.** Drop-in binary replace, systemd restart, state-preserving. `hoop --version`. Breaking schema changes include a migration in-binary.
7. **Metrics.** Prometheus-compatible `/metrics`: fleet sizes, event rates, cost-per-minute, stuck-worker count, WS connections. Optional — off by default.
8. **Tailscale-aware auth.** Each WS connection identified by Tailscale identity (via Tailscale's whois). Operators appear in audit rows by email (`jed@...`) not IP.
9. **Performance budgets.** UI remains responsive with 20 projects × 5 workers each × 300 beads each. Measured on EX44-class hardware.
10. **Graceful degradation.** If a project's `.beads/` goes missing or its SQLite is corrupt, that project card shows a specific error (not a spinner, not a generic fail). Others keep working.
11. **Observer-mode second instance.** A second `hoop serve` on the same host can launch in read-only mode, attaching to the primary's event streams via the control socket. For long-tail debugging without disrupting the primary.

**Success criteria.**
- `systemctl --user restart hoop` resumes state in <5s.
- `projects.yaml` edit with syntax error rejected; old config still running.
- One month of normal operation produces <1GB in log+backup directories.
- Operator identity visible in audit log for every mutation.

### Phase 6 — Multi-operator (v1.0)

**Goal.** More than one person uses the same HOOP instance.

**Deliverables.**

1. Roles: **viewer** (read-only, sees everything) and **steerer** (read + write). Two levels only.
2. Role assignment per Tailscale identity (simple list in config).
3. Audit log carries real operator identity on every mutation.
4. Per-operator UI state: pinned projects, last-opened conversation, filter preferences.
5. "Who's looking at what" indicator (optional, privacy-toggleable): small presence dots on project cards.
6. Public README, user-facing docs, example configs.

**Success criteria.**
- Two operators on the same HOOP see consistent state at all times.
- Viewer-role attempts to mutate are refused at the schema boundary with a clear error.
- README enables a stranger to install HOOP against their own NEEDLE workspace in <30 minutes.

---

## 7. Technology decisions

| Layer | Choice | Why |
|---|---|---|
| Daemon language | Rust | Matches NEEDLE direction; strong async; single-binary distribution; no `node_modules` churn |
| HTTP / WS server | `axum` | Standard Rust HTTP stack; first-class WS; embedded static assets |
| File watching | `notify` | Cross-platform, reliable; battle-tested |
| UI | React + Vite + TypeScript + Jotai | Matches team skill; keyed atoms fit the streaming-vs-committed split |
| Schema | JSON Schema draft-07 + `typify` (Rust) + `json-schema-to-typescript` (TS) | One source of truth; codegen to both languages |
| Durable storage (HOOP) | SQLite via `rusqlite` | Fleet.db for launches/workers/actions/cost only; small, portable |
| Chat agent | Claude Haiku 4.5 (via Anthropic SDK with caching) | Cost-appropriate for tool-belt translation |
| Service supervisor | systemd (user-scope) | Standard; matches existing environment |
| Auth | Tailscale identity via whois | Matches environment; no separate auth system needed |

---

## 8. Non-goals

Explicitly out of scope. Where HOOP deliberately does not grow.

1. **Orchestrating work.** NEEDLE does this. HOOP never schedules, prioritizes, or assigns beads outside explicit operator action.
2. **Storing bead state.** `br` does this. HOOP never writes to bead SQLite directly.
3. **Replacing FABRIC.** FABRIC is read-only, deployable anywhere; HOOP is write-capable, local to the host. They link via URL bridge.
4. **Multi-host control.** One HOOP, one host. Growth in v1 is more projects on the same host, not more hosts. Multi-host is a someday concern (and arrives through the K8s sketch, not by federation of HOOPs).
5. **Cross-host bead queues.** A single bead queue lives with its workspace on one host. Cross-host coordination is out.
6. **RBAC beyond viewer / steerer.** If a team needs more granularity, run multiple HOOP instances.
7. **Secrets management.** HOOP reads credentials from each CLI's native cache; it never rotates, stores, or issues secrets.
8. **Browser-only execution.** HOOP requires a server — it spawns tmux and reads/writes filesystem.

---

## 9. Open questions

1. **Hot-reload granularity.** Should a `fleet.yaml` edit reload just that project's manifest, or restart the project runtime? Lean toward manifest-only reload with explicit "restart project" command for worker changes. Resolve in phase 2.
2. **Mayor: in-process session or subprocess?** The Mayor is a Claude Code session, not a generic agent; HOOP likely spawns/attaches to a real `claude` process with a known session id and MCP access to HOOP's APIs. This keeps HOOP optional (runnable without a Mayor) and keeps the Mayor's reasoning consistent with how the operator uses Claude elsewhere. Resolve in phase 4.
3. **Mayor's MCP surface.** Two designs: (a) expose HOOP's APIs via an MCP server the Mayor connects to, or (b) inject context via system prompt + tools via CLI MCP config. Lean (a) — cleaner, survives session restarts. Resolve in phase 4.
4. **Cost caps: per-project vs global?** Probably both, with per-project as primary and global as a safety cap. Resolve during phase 2 cost work.
5. **Multi-provider routing policy.** Static (assign adapter X to strand Y in manifest) vs dynamic (HOOP picks at claim time based on rate-limit headroom and bead hints). Lean static-with-dynamic-rate-limit-fallback. Resolve in phase 3.
6. **Conversation history expiry.** The host accumulates conversations indefinitely; at some point the session tailer's in-memory index becomes expensive. Lean toward LRU eviction with lazy reload on demand. Resolve if/when it becomes a problem.
7. **Session tailer scaling.** On a host with years of `~/.claude/projects/` accumulation, startup discovery can be slow. May need a cache of file mtimes + parsed-digests in `~/.hoop/session-cache.db`. Resolve if startup exceeds 10s.
8. **Worker supervisor API surface.** Does `hoop launch` return only after ack, or return immediately with an async status? Lean sync-with-timeout for CLI, async-with-progress-events for WS. Resolve in phase 1.
9. **Cross-project collision: alert or block?** If two active workers in different projects touch the same file, is that an alert or an auto-pause? Alert for now; auto-pause is phase 6+ after more data.
10. **Mayor-initiated cross-project operations.** Should the Mayor be allowed to start/stop fleets in projects the operator hasn't explicitly mentioned in the current conversation? Lean no — require an explicit project reference in the turn that triggered the mutation, or require UI confirmation. Resolve in phase 4.

---

## 10. Milestones

| Version | Target | Definition of done |
|---|---|---|
| v0.1 | +4 weeks | Single-project EX44 deployment, read-only, 3-worker fleet launchable from CLI |
| v0.2 | +10 weeks | Multi-project dashboard, cost aggregation, dependency graph, collision + stuck detection, visual debug panel, multi-account awareness |
| v0.3 | +14 weeks | Direct steering: bead ops + tmux control + multi-provider routing + BYO-account rotation + FABRIC URL bridge |
| v0.4 | +20 weeks | The Mayor: persistent cross-project coordinator with tool belt, notifications, operator chat pane |
| v0.5 | +24 weeks | Operational polish: systemd, hot-reload, backups, metrics, Tailscale identity |
| v1.0 | +28 weeks | Multi-operator with viewer/steerer roles; public README |

Dates are rough planning fiction; the important property is ordering. **Do not chase breadth before v0.1 is real.** A broken single-project loop doesn't get better by adding more projects to it. And do not seat the Mayor before direct steering works — the Mayor's tool belt is just the write-side ops with an agent in front of them; if the underlying operations are broken, wrapping an agent around them makes debugging harder, not easier.

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
                │  project: ardenone-cluster ─┐   │
                │  project: miroir ───────────┤   │── URL bridge ──► FABRIC
                │  project: ibkr-mcp ─────────┤   │
                │  project: vista ────────────┤   │
                │  project: spaxel ───────────┘   │
                │                                 │
                │  cross-project state            │
                │  audit log (fleet.db)           │
                └──┬──────────────────────────────┘
                   │
           tmux spawns ("needle-<project>-<worker>")
                   │
                   ▼
     ┌─────────────────────────────────────────────────────┐
     │ NEEDLE worker pool (per project, tmux-hosted)        │
     │                                                      │
     │  needle-ardenone-cluster-alpha   needle-miroir-alpha │
     │  needle-ardenone-cluster-bravo   needle-miroir-bravo │
     │  needle-ibkr-mcp-alpha          ...                  │
     │                                                      │
     │  each: br claim/close + YAML adapter + CLI child     │
     │  writes: .beads/events.jsonl  .beads/heartbeats.jsonl│
     └─────────────────────┬────────────────────────────────┘
                           │
                           ▼
              ┌──────────────────────────┐
              │ per-project .beads/       │
              │ (SQLite + JSONL, owned    │
              │  by `br`)                 │
              └──────────────────────────┘

    ┌────────────────────────────────────────────────────────┐
    │ Separately: operator's ad-hoc CLI sessions             │
    │ (claude, codex, opencode, gemini, aider) writing to    │
    │ ~/.claude/projects/, ~/.codex/sessions/, etc.          │
    │ HOOP observes, classifies as "operator" not "worker",  │
    │ includes in project view, excludes from fleet analytics│
    └────────────────────────────────────────────────────────┘
```

Each project is an island. HOOP stitches the islands into a view without merging them. Workers belong to islands; operators cross between them; the control plane is the ferry.

---

## 12. Appendix — Kubernetes worker deployment (someday, sketched)

Not a phase of this plan. Recorded here so the door isn't closed but the v1 design isn't warped chasing it.

If and when the single host runs out of room — CPU, memory, bandwidth to CLI APIs, or sheer session-management overhead — HOOP can extend to scheduling NEEDLE workers as Kubernetes pods rather than tmux sessions on the EX44. The shape:

- `hoop-agent` sidecar in each worker pod, forwarding events over a Tailscale-authenticated SSE endpoint to central HOOP. In Gas Town vocabulary this is still a polecat — the execution site changes, the role doesn't.
- Per-workspace PVC for the bead queue; pod-local for CLI session files (ephemeral) or a second PVC for persistent-replay use cases.
- Central HOOP stays on the EX44; clusters are leaf execution sites, not parallel control planes. The Mayor keeps one session, one cross-project view, and reaches into remote polecats through the same tool belt.
- Manifests in `jedarden/declarative-config` synced by ArgoCD; images built by Argo Workflows on iad-ci.
- Worker container is a pure NEEDLE container + adapter CLIs; the agent sidecar is a separate tiny Rust binary. Not headless HOOP.

Rackspace-spot-terraform automation was retired 2026-04-22; spot clusters are now manually provisioned. That makes cluster-creation a deliberate, human-in-the-loop step — fine for HOOP's deferred K8s work since cluster churn is no longer a background concern.

This requires phase 5's operational maturity and phase 3's steering write-path and phase 4's Mayor to exist first. The trigger is "I can't fit another fleet on this host," not "I want K8s on the resume." Design for it if and when the EX44 saturates; do not design the single-host daemon around it now.
