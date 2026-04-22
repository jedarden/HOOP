# HOOP implementation plan

**Status:** Draft v1 — 2026-04-22
**Scope:** Control plane for NEEDLE fleets, starting on a single Hetzner-class host and graduating to Kubernetes-deployed workers.

---

## 1. Vision

HOOP is the active frame for a NEEDLE fleet. It launches workers, holds the work under tension, shows what's happening, and provides the handle operators use to steer. It never does NEEDLE's job (processing beads) or FABRIC's job (passive observability) — it lives in the gap between them: process lifecycle + write-side operations + chat-driven steering.

The end-state deployment envelope:

- **Operator host** (this Hetzner EX44-class environment): runs HOOP itself, serves the web UI over Tailscale, holds the project registry, coordinates across workspaces. Long-lived, credentialed, the single pane of glass.
- **Worker execution plane**: starts as local tmux sessions on the same host; graduates to Kubernetes-scheduled pods in a shared cluster. Workers are commodity — they come and go.
- **Bead queues**: live with their workspaces (SQLite + JSONL via `br`), never inside HOOP's storage. HOOP reads bead state; `br` owns it.
- **CLI sessions**: live in each CLI's native directory (`~/.claude/projects/`, `~/.codex/sessions/`, etc.), read-only from HOOP's perspective. Joined to beads via the `[needle:<worker>:<bead-id>:<strand>]` prefix tag.

HOOP stays portable, workers stay close to the work. The design never asks either to be the other.

---

## 2. Principles

Locked in from day one. These come directly from the prior-art problems-and-solutions note and from NEEDLE's existing invariants.

1. **Events are authoritative; projections are derived.** HOOP writes event rows and reads projections. No `fleet_status.json` that goes stale.
2. **Liveness = process, never file.** `kill -0 pid && !stopped_record`.
3. **Server is the epoch.** Client rebuilds state from scratch on every reconnect.
4. **Dual-identity in schema.** Bead id (stable) + provider session id (derived). `session_bound` event at first join.
5. **JSON Schema as cross-repo contract.** Every record has `schema_version`. TS and Rust types codegen off one source.
6. **Atomic `.tmp` + rename for every write.** Line-buffered NDJSON reader for every read.
7. **Never silent-drop unknown events.** Log, emit progress, count.
8. **Never mutate bead state directly.** Use `br` CLI verbs. Every write audited with `actor: hoop:<user>`.
9. **Workers are independent.** If HOOP dies, workers keep working. HOOP reconstructs state from disk on restart.
10. **Read-only is the default.** Steering is opt-in per workspace.

---

## 3. Target environment

HOOP v1 targets a host shaped like this Hetzner EX44:

- Tailscale-only networking (no public ingress)
- `br` CLI installed, reachable in PATH
- One or more workspaces with `.beads/` directories
- Native CLIs installed and credentialed: Claude Code, Codex, OpenCode, Gemini, Aider
- tmux available; git 2.5+ for worktrees
- Long-lived (server, not a laptop); credentials already cached
- Existing operational conventions (kubectl proxies, ArgoCD read-only API, Argo Workflows CI, Docker Hub for images) available but not required for v1

Phase 4 extends target to a Rackspace Spot Kubernetes cluster for remote worker deployment.

---

## 4. Component architecture

### 4.1 Single-binary HOOP daemon

One Rust binary. Subcommands:

```
hoop serve                    # run the control-plane daemon (web UI + WS + REST)
hoop launch <project>         # spawn the fleet defined in <project>/fleet.yaml
hoop stop <project>           # stop workers for a project
hoop salvage <project>        # resume dead workers whose worktrees are still intact
hoop status                   # quick CLI view of active fleets
hoop projects add <path>      # register a workspace in ~/.hoop/projects.yaml
hoop projects list
hoop steer                    # open an interactive steering chat session (delegates to agent)
hoop audit                    # run startup binary/permission audit, print results
```

`hoop serve` is the long-lived process. Everything else is a client that speaks to it over a local Unix socket (`~/.hoop/control.sock`).

### 4.2 Daemon internals

```
┌────────────────────────────────────────────────────────────────┐
│ hoop serve                                                      │
│                                                                 │
│  ┌────────────────┐  ┌──────────────────┐  ┌────────────────┐ │
│  │ HTTP/WS server │  │ Project registry │  │ Control socket │ │
│  │ (axum, :7500)  │  │ (projects.yaml)  │  │ (UDS)          │ │
│  └───────┬────────┘  └────────┬─────────┘  └───────┬────────┘ │
│          │                     │                    │          │
│          │             ┌───────┴──────────────┐     │          │
│          │             │ Per-project runtime  │◄────┘          │
│          │             │ (one per workspace)  │                │
│          │             │                      │                │
│          │             │ - event tailer       │                │
│          │             │ - session tailer     │                │
│          │             │ - worker supervisor  │                │
│          │             │ - tag-join resolver  │                │
│          │             │ - heartbeat monitor  │                │
│          │             │ - cost aggregator    │                │
│          │             └──────────┬───────────┘                │
│          │                        │                            │
│          └────────────────────────┴────────────────────────────┤
│                             │                                   │
│  ┌──────────────────────────▼──────────────────────────────┐  │
│  │ Shared state (in-memory, rebuilt from disk on restart)  │  │
│  │                                                          │  │
│  │  - fleets: Map<project, FleetState>                     │  │
│  │  - workers: Map<id, WorkerState>                        │  │
│  │  - active_beads: Map<bead_id, ActiveBead>               │  │
│  │  - conversations: Map<id, Conversation>                 │  │
│  │  - streaming_content: Map<id, Rope>  // volatile        │  │
│  │  - session_alias: Map<session_id, bead_id>              │  │
│  └──────────────────────────────────────────────────────────┘  │
│                             │                                   │
│  ┌──────────────────────────▼──────────────────────────────┐  │
│  │ Durable state: ~/.hoop/fleet.db (SQLite)                │  │
│  │  - launches, workers, actions (audit), cost_daily       │  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────┘
```

### 4.3 Web client

- React + Vite + TypeScript, Jotai for state
- Single WS on `/ws`, REST on `/api/*`
- Served by the HOOP daemon itself (embedded static assets in release builds)
- Panels: project list, fleet map, bead graph, strand timeline, conversation viewer, cost panel, steering chat pane, audit overlay
- All streaming content lives in a separate reactive map (not in committed state) — per the in-flight isolation rule

### 4.4 Shared schema crate / package

- `hoop-schema/` with JSON Schema source of truth
- Rust types via `typify`; TS types via `json-schema-to-typescript`
- Used by daemon, web client, and any future agent sidecar
- Every record carries `schema_version: 1` from day one

---

## 5. Data flows

### 5.1 Local (phase 1) — tmux workers

```
NEEDLE worker (tmux)                       HOOP daemon
────────────────────                       ───────────
br claim bead-abc
   │
   ├─► .beads/events.jsonl  ◄── tail -F ──► event tailer
   │     {event: claim, ...}                     │
   │                                             ├─► fleet WS broadcast
   ▼                                             │
dispatch via YAML adapter                        │
(prompt prefixed with
 [needle:alpha:bead-abc:pluck])                  │
   │                                             │
   ├─► CLI writes to                             │
   │   ~/.claude/projects/<hash>/*.jsonl         │
   │                                             │
   │                                   ◄── 5s poll ── session tailer
   │                                             │     │
   │                                             │     ├─► tag-join resolver
   │                                             │     │      (tag → bead alias)
   │                                             │     │
   │                                             │     └─► conversation WS broadcast
   ▼
CLI exits → br close / fail
   │
   └─► .beads/events.jsonl  ◄── tail -F ────────► event tailer
         {event: complete, outcome, duration}          │
                                                       └─► cost aggregator
                                                              + fleet WS broadcast

heartbeat every 10s ──► .beads/heartbeats.jsonl ──► heartbeat monitor
                                                          │
                                                          └─► liveness projection
```

Three independent streams (events, sessions, heartbeats), each with its own rate and staleness budget. Joined in memory by worker id + bead id, never by wall-clock timing.

### 5.2 Remote (phase 4) — Kubernetes workers

```
Central HOOP (EX44)                       K8s worker pod
───────────────────                       ──────────────
                                          ┌─────────────────────────┐
                                          │ needle container         │
                                          │ - br                     │
                                          │ - CLI adapters           │
                                          │ - writes events.jsonl    │
                                          │   to /workspace/.beads/  │
                                          └────────┬────────────────┘
                                                   │
                                          ┌────────▼────────────────┐
                                          │ hoop-agent sidecar       │
                                          │ - tails .beads/*.jsonl   │
                                          │ - forwards events via    │
                                          │   SSE/gRPC to central    │
                                          │ - proxies br writes      │
                                          │   back from HOOP         │
                                          └────────┬────────────────┘
                                                   │
      ◄────── event stream (SSE over HTTPS/Tailscale) ──────
                                                   │
      ─────── steering commands (br proxy) ──────►
                                                   │
                                          PVC: /workspace
                                          (bead queue persists across
                                           pod restarts)
```

Same event model, network transport instead of `tail -F`. See §8 for the sidecar design rationale.

---

## 6. Phased roadmap

### Phase 0 — Foundation (COMPLETE)

- Repo created, docs scaffolding, notes seeded with prior-art research.
- Plan (this document) published.

### Phase 1 — Single-host daemon (v0.1)

**Goal:** HOOP runs on the EX44, reads one workspace's NEEDLE events, serves a web UI, launches and supervises workers locally via tmux. Read-only by default.

**Deliverables:**

1. Rust binary `hoop` with `serve`, `launch`, `status`, `audit` subcommands.
2. Project registry at `~/.hoop/projects.yaml`; `hoop projects add/list/remove`.
3. Single-workspace runtime:
   - Event tailer for `$WORKSPACE/.beads/events.jsonl`
   - Heartbeat monitor for `$WORKSPACE/.beads/heartbeats.jsonl`
   - Session tailer reading `~/.claude/projects/`, `~/.codex/sessions/`, etc., with bounded-concurrency progressive discovery
   - Tag-join resolver using the `[needle:<worker>:<bead-id>:<strand>]` prefix convention
   - Worker supervisor: spawns tmux sessions from `fleet.yaml`, records PIDs, verifies spawn-ack within grace window
4. Web UI panels: fleet map, bead list (no graph yet), per-worker timeline, conversation viewer, audit overlay, search palette
5. Durable storage: `~/.hoop/fleet.db` (SQLite) for launches, workers, actions audit
6. Three NEEDLE hooks landed:
   - Prompt prefix tag
   - `events.jsonl` append helper
   - Heartbeat thread
7. `tmux kill-session` grace + SIGKILL fallback on `hoop stop`
8. Startup audit: `br`, tmux, CLI binaries, `.beads/` accessibility — overlay on failure
9. Shared schema crate with `schema_version: 1` on every record

**Non-goals for v0.1:**
- Multi-project view (one workspace at a time)
- Steering (read-only)
- Dependency graph UI (list view only)
- Cost aggregation (defer to v0.3)
- K8s (defer to v0.4)

**Success criteria:**
- Spawn a fleet of 3 workers from `fleet.yaml`; see all three on the map within 10s
- Kill a tmux session externally; HOOP reports dead within 30s; claim released
- Restart `hoop serve`; reconnecting UI rebuilds state entirely from disk
- View any worker's transcript and see the bead id in the header (tag-join working)
- Zero silent-drops: every unknown event appears in the UI's "diagnostic" panel

### Phase 2 — Multi-project observability (v0.2)

**Goal:** One HOOP serves multiple projects on the EX44, cross-project dashboards, cost aggregation, dependency graph UI.

**Deliverables:**

1. Auto-discover workspaces with `.beads/` under a configured root (`hoop projects scan ~/ardenone-cluster/`)
2. Fleet-of-fleets dashboard: one card per project, aggregate worker count / active beads / cost-today
3. Bead dependency graph view (DAG rendering, SVG or canvas, colored by state)
4. Strand timeline: stacked-area chart showing fleet-seconds spent in Pluck / Explore / Mend / Knot per project, per day
5. Cost aggregator: per-project, per-adapter, per-bead, per-strand; rate-limit window overlay for Claude (5h + 7d)
6. Search palette across projects (substring over conversations, 50-result cap, cmd-K)
7. Collision detector: reads `touched_files` events from workers, alerts when active workers overlap
8. Stuck detector: worker with no heartbeat transition for configurable N minutes, or looping on the same bead across retries

**Success criteria:**
- EX44 discovers all workspaces in `~/` with `.beads/`; each appears in the dashboard
- Cost panel matches `br` summaries within ±2%
- Dependency graph renders 500+ beads interactively (<500ms interaction)
- Collision alert fires within 30s of two workers touching the same file

### Phase 3 — Steering surface (v0.3)

**Goal:** HOOP gains write-side operations. Direct bead ops, tmux controls, chat-driven steering. All audited.

**Deliverables:**

1. Direct bead-op buttons wired to `br` CLI:
   - Boost priority / lower priority
   - Close bead with resolution (`closed-no-artifact`, `merged`, `parked`)
   - Create review bead as dep of a target
   - Release stuck claim (only when worker demonstrably dead)
   - Reassign bead to a specific worker/adapter
2. tmux control actions (SIGSTOP / SIGCONT / SIGTERM), gated to "release claim first" for TERM
3. Chat steering pane:
   - Haiku-tier agent with tool belt: read-only NEEDLE introspection + write-side `br`/tmux operations
   - All chat-driven mutations emit `actions` audit rows
   - Hard ceilings: `max_attempts`, `max_review_rounds`, `cycle_budget` on every generated bead
4. URL-scheme bridge for FABRIC: `hoop://release/bead/<id>`, `hoop://boost/<id>`, etc. FABRIC renders action links; HOOP opens a confirmation dialog
5. Permission snapshot at worker spawn (env + settings.json copy) — no live inheritance
6. Per-workspace `yolo` opt-in plus required `sandbox:` config if enabled

**Success criteria:**
- Every mutation has a corresponding `actions` audit row
- A bead closed via HOOP shows the `actor: hoop:<user>` in `br` history
- Chat agent refuses to create beads exceeding ceiling without explicit operator override
- Release-stuck-claim only succeeds when the worker's PID is actually dead AND heartbeat is stale

### Phase 4 — Remote cluster deploy (v0.4)

**Goal:** HOOP can spawn NEEDLE workers into a Kubernetes cluster instead of local tmux. Central HOOP stays on the EX44; workers live in the cluster.

**Deliverables:**

1. New fleet manifest stanza: `runtime: k8s` with cluster selector, namespace, image, resources
2. Worker pod spec (see §8 for the sidecar decision):
   - `needle` container (NEEDLE binary + CLI adapters + `br`)
   - `hoop-agent` sidecar (event forwarder + `br` write proxy)
   - PVC mount for `/workspace` (bead queue persists across pod restarts)
   - Secret for CLI auth credentials (per-adapter)
3. Control plane pieces:
   - `k8s` worker supervisor using `kube-rs` — CRUD pods, watches for pod events
   - SSE receiver endpoint for agent sidecars to forward events
   - `br` proxy endpoint (authenticated, bounded, audit-logged) — HOOP sends a mutation; named pod executes it
4. Container build via Argo Workflows on iad-ci:
   - `hoop:latest` (server image, for dev/demo use)
   - `hoop-agent:latest` (sidecar)
   - `needle-worker:latest` (worker base image; adapter images can layer on top)
5. K8s manifests in `jedarden/declarative-config` under `k8s/<cluster>/hoop-workers/`, synced by ArgoCD
6. Tailscale Operator or ServiceLB for the SSE ingress path; no public exposure

**Success criteria:**
- `hoop launch <project> --cluster rs-manager --replicas 5` spawns 5 worker pods
- Events from pods reach the central HOOP within 2s (SSE latency)
- Pod killed externally → claim released by HOOP within 30s
- Surviving PVC + dead pod → `hoop salvage` resumes the bead in a new pod

### Phase 5 — Multi-operator / polish (v1.0)

**Goal:** More than one person uses the same HOOP instance; audit and UI reflect that.

**Deliverables:**

1. Per-operator identity (Tailscale identity or simple OIDC via Google SSO as ardenone already uses for Argo)
2. `actions` audit rows carry real operator identity
3. Per-operator UI state persistence
4. Role: read-only viewer vs steerer (two levels, not full RBAC)
5. Chat agent understands "my workers" vs fleet-wide queries
6. Docs, examples, public README

---

## 7. The worker-container question

**Question as posed:** In phase 4, should worker containers be headless HOOP instances, or a separate construct?

**Recommendation: separate construct — thin `hoop-agent` sidecar next to a pure `needle` container.** Not headless HOOP.

### Why not headless HOOP

1. **HOOP is a control plane; it reasons across workers.** A single worker has no need for that reasoning. Packaging it in every pod inflates container size, memory footprint, and attack surface.
2. **Bimodal binaries rot.** A binary with `hoop serve` and `hoop agent` subcommands slowly diverges: features that serve the server mode leak into the agent mode's config surface. Prior art on this pattern (every "runs in server or worker mode" binary I've seen) accumulates cruft.
3. **Upgrade lock-step.** If workers run HOOP, then upgrading HOOP means rolling every worker. Decoupling the sidecar from the server means HOOP can evolve faster than the agent (and vice versa).
4. **The sidecar's job is tiny.** Forward NDJSON lines over a stream, proxy a few `br` verbs, heartbeat. A purpose-built agent fits in a few hundred lines.

### Why not "just a plain NEEDLE container with no sidecar"

- Central HOOP needs *some* way to receive events from the pod. Options: shared PVC mount back to the EX44 (won't work cross-cluster), NATS/Kafka (overkill), or a sidecar that streams. Sidecar is the lightest.
- Steering (phase 3 write-side ops) needs a way to execute `br` inside the pod. Without a sidecar, HOOP would need `kubectl exec` on every mutation — slow, permission-heavy, audit-weak.

### The sidecar in detail

**`hoop-agent` responsibilities (and only these):**

1. Tail `/workspace/.beads/events.jsonl` and `/workspace/.beads/heartbeats.jsonl`; stream each line over SSE to the central HOOP's `/cluster/events/<pod-id>` endpoint.
2. Serve a small local HTTP endpoint for `br` proxy commands: HOOP posts `{op: "claim-release", bead: "bd-abc"}`, agent runs `br` locally, returns exit code + stdout.
3. Emit its own heartbeat to the central HOOP (proves sidecar alive; NEEDLE heartbeat proves worker alive; two are distinct).
4. Strip ANSI from any stdout it proxies.
5. Exit when the `needle` container exits (pod-level lifecycle).

**Agent non-responsibilities:**

- Does not make decisions (HOOP does)
- Does not reach outside the pod (no cross-pod or external network calls)
- Does not cache or buffer state beyond a short SSE replay window
- Does not authenticate users (HOOP does, using a pod identity token issued at pod-creation)

**Agent image:** Rust static binary, scratch base, <20MB. Same schema crate as HOOP so field names don't drift.

### Bead queue location in K8s

Per-workspace PVC: `workspace-<slug>-pvc` mounted at `/workspace`. Bead queue persists across pod restarts. Multiple worker pods in the same workspace mount the same PVC read-write (NEEDLE's SQLite claim model already handles this correctly).

Central HOOP does *not* own the bead queue. It reads event streams from sidecars, writes mutations back through sidecars. The queue is still local to the workspace — just on cluster storage instead of EX44 disk.

### Session files in K8s

Worker pods run the CLIs locally; the CLIs write session files inside the pod. Three options:

- **A. Ephemeral** — session files die with the pod. Transcripts are still forwarded live via events; historical replay reads from central HOOP's durable event log. This is the recommendation for v0.4.
- **B. PVC-mounted `~/.claude`** — session files persist; post-mortem replay from native CLI tools works. More complex, more storage cost.
- **C. Forwarded-on-complete** — sidecar uploads the session file to central HOOP on pod exit. Middle ground.

Default: A. Opt into B per workspace where native CLI replay is needed.

---

## 8. Technology decisions

| Layer | Choice | Why |
|---|---|---|
| Daemon language | Rust | Matches NEEDLE direction; strong async story; single-binary distribution; no `node_modules` maintenance |
| Web server | `axum` | Standard Rust HTTP stack; first-class WS; easy embedded static |
| K8s client | `kube-rs` | Standard Rust K8s SDK |
| UI | React + Vite + TypeScript + Jotai | Matches existing team skill; Jotai's keyed atoms fit the streaming-vs-committed split |
| Schema | JSON Schema draft-07 | Codegen to Rust (`typify`) and TS (`json-schema-to-typescript`); matches prior-art recommendation |
| Storage (HOOP) | SQLite | Fleet.db for launches/workers/actions only; small, portable |
| Event transport (local) | File tail (`notify` crate) | Same as prior art; cheapest reliable option |
| Event transport (K8s) | SSE over HTTPS (Tailscale) | Simpler than NATS; HOOP can authenticate via Tailscale identity |
| Container registry | Docker Hub (`ronaldraygun/hoop`, `ronaldraygun/hoop-agent`) | Matches existing convention |
| CI | Argo Workflows on iad-ci | No GitHub Actions per environment policy |
| K8s manifests | `jedarden/declarative-config`, synced by ArgoCD | Matches existing convention |
| Chat agent | Claude Haiku 4.5 | Cost-appropriate for the tool-belt translation task |

---

## 9. Non-goals

Explicitly out of scope. Surfaces where HOOP deliberately *does not* grow.

1. **Orchestrating work.** NEEDLE does this. HOOP never schedules, prioritizes, or assigns beads outside explicit operator action.
2. **Storing bead state.** `br` does this. HOOP never writes to SQLite directly.
3. **Replacing FABRIC.** FABRIC stays read-only, deployable anywhere; HOOP stays write-capable, local to the host. They link via URL bridge.
4. **Browser-only execution.** HOOP requires a server — it needs tmux and filesystem access.
5. **Running as a headless daemon with no UI.** That's just NEEDLE. Use NEEDLE alone if you don't want HOOP's UI.
6. **Multi-cluster control from one instance.** Each HOOP talks to at most one cluster. Multi-cluster is a v2.0 concern.
7. **RBAC beyond viewer/steerer.** Full role management is out. If you need more, operate multiple HOOP instances.
8. **Secrets management.** HOOP reads credentials; it doesn't rotate or store them. Secrets come from the host (phase 1–3) or Kubernetes Secrets (phase 4+).

---

## 10. Open questions

1. **Does the chat steering agent run in-process with HOOP or as a separate subprocess?** Leaning separate — keeps HOOP runnable without an Anthropic API key. Resolve before phase 3.
2. **Bead queue mount mode for K8s workers: ReadWriteMany vs ReadWriteOnce with scheduling constraints?** SQLite-over-ReadWriteMany has historical corruption risk. Lean RWO + pin all workers for a workspace to one node, or use a small SQLite proxy. Resolve during phase 4 design.
3. **How does cross-project steering interact with deterministic priority ordering?** If an operator boosts bead X in project A, does that affect what NEEDLE alpha picks in project B? No — beads are per-workspace. But a chat like "focus everything on project A" needs a clear mental model. Prototype in phase 3.
4. **Does FABRIC consume HOOP's event stream, or its own?** Both consume NEEDLE's `events.jsonl`. If FABRIC adds higher-level derived signals, HOOP might consume those too. Defer; revisit after FABRIC is further along.
5. **Should `hoop-agent` support running multiple NEEDLE workers in one pod, or strictly one-per-pod?** One-per-pod is simpler and matches cattle-not-pets. Multi-worker is cheaper for tiny beads. Start with one-per-pod; revisit if pod-startup cost dominates.

---

## 11. Milestones & success criteria

| Version | Target date | Definition of done |
|---|---|---|
| v0.1 | +4 weeks | Single-workspace EX44 deployment; read-only; 3-worker fleet launchable from CLI |
| v0.2 | +8 weeks | Multi-project dashboard; cost + dependency graph; collision detection |
| v0.3 | +12 weeks | Steering: direct ops + tmux control + chat agent; FABRIC URL bridge |
| v0.4 | +20 weeks | K8s worker deploy working against rs-manager; `hoop-agent` sidecar image published |
| v1.0 | +28 weeks | Multi-operator; viewer/steerer roles; public README |

Dates are rough planning fiction. The important ordering: don't reach for K8s before observability + steering are real on a single host. A broken local experience doesn't get better by adding a cluster underneath.

---

## 12. Relationship to the rest of the ecosystem

```
                    operator (browser)
                           │
                           │  HTTPS over Tailscale
                           ▼
                 ┌──────────────────┐
                 │  HOOP  (EX44)    │
                 │                   │────── reads ──────► FABRIC
                 │  - launch         │                     (URL bridge)
                 │  - steer          │
                 │  - observe        │
                 └──────────┬───────┘
                            │
                            │  tmux spawns (local)
                            │  k8s API (remote, v0.4+)
                            │
                            ▼
              ┌────────────────────────────┐
              │ NEEDLE workers             │
              │ (tmux locally / pods in K8s)│
              │                            │
              │ - br claim/close           │
              │ - CLI adapters             │
              │ - events.jsonl             │
              │ - heartbeats.jsonl         │
              └──────────────┬─────────────┘
                             │
                             ▼
                      ┌──────────────┐
                      │  br / .beads │
                      │  (SQLite +   │
                      │   JSONL)     │
                      └──────────────┘
```

HOOP is the handle; NEEDLE is the needle; `br` is the bead; FABRIC is the finished cloth. Each has one job and the dependency arrows only point one way.
