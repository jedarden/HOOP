# Interop with NEEDLE

How HOOP plugs into NEEDLE without duplicating state, breaking determinism, or requiring NEEDLE changes beyond four small hooks.

## The four hooks NEEDLE needs

HOOP is intentionally *additive* — NEEDLE runs standalone without HOOP, and HOOP doesn't replace any part of NEEDLE. The coupling is four small touchpoints:

### Hook 1: Dispatch prompt prefix tag

Every bead dispatch prepends the first user-message line with:

```
[needle:<worker-name>:<bead-id>:<strand>]
```

- `<worker-name>` — NATO id (`alpha`, `bravo`, …)
- `<bead-id>` — `br` bead id (`bd-abc123`)
- `<strand>` — one of `pluck | explore | mend | weave | unravel | pulse | knot`

This tag lives in the CLI's session file (Claude/Codex/OpenCode/Gemini all record the user message verbatim). HOOP's disk adapter extracts it during parse → joins transcripts back to beads without a second storage layer. Already the pattern used by the prior-art reference ADE and its swarm orchestrator.

**NEEDLE change:** one line in the prompt builder. Tag is opaque to the CLI and doesn't affect work.

### Hook 2: Event tap

Each NEEDLE worker appends one JSONL line per bead transition to a shared event log:

```jsonl
{"ts":"2026-04-21T18:42:10Z","worker":"alpha","bead":"bd-abc123","strand":"pluck","event":"claim"}
{"ts":"2026-04-21T18:47:33Z","worker":"alpha","bead":"bd-abc123","event":"dispatch","adapter":"claude","model":"opus"}
{"ts":"2026-04-21T18:52:01Z","worker":"alpha","bead":"bd-abc123","event":"complete","outcome":"success","duration_ms":287104,"exit_code":0}
```

**Where.** `$NEEDLE_EVENTS` env var, default `.beads/events.jsonl`. Shared across all workers in a workspace. Append-only.

**Why JSONL not the bead table.** Beads capture state; events capture transitions. Transitions are the analytics surface (duration, retry rate, strand mix). Pushing them into bead fields would either denormalize (many cycles per bead) or require a second SQLite table (adds write contention to the claim path). JSONL is free and tails cleanly.

**HOOP use.** Tail `events.jsonl` and broadcast `fleet_event` on the WS. Replay is trivial — just read the file.

**NEEDLE change:** one helper `emit_event(worker, bead, event, **fields)`; called at claim/dispatch/complete/fail/release/timeout/crash. Dozen lines max.

### Hook 3: Worker heartbeat

Each worker appends a heartbeat line every 10s (configurable):

```jsonl
{"ts":"...","worker":"alpha","state":"executing","bead":"bd-abc123","pid":12345,"adapter":"claude"}
{"ts":"...","worker":"alpha","state":"idle","last_strand":"pluck"}
{"ts":"...","worker":"alpha","state":"knot","reason":"strands exhausted"}
```

**Where.** Same `events.jsonl` with `event:"heartbeat"`, or a separate `.beads/heartbeats.jsonl` (simpler to tail at lower frequency).

**Why.** The `pid-is-alive` check works locally but not across machines. Heartbeats are the cross-machine liveness primitive.

**NEEDLE change:** one background thread per worker. Trivial.

### Hook 4: Stitch label inheritance

When a NEEDLE worker creates follow-up beads (via `br create`), any `stitch:*` labels from the claimed bead must be copied to the new bead. Without this, retries and cascading sub-beads lose Stitch lineage.

**HOOP side.** The REST API `POST /api/p/{project}/beads` supports a `parent_bead_id` field. When set, HOOP reads the parent bead's labels via `br get`, extracts all `stitch:*` labels, and appends them (deduplicated) to the new bead.

**NEEDLE side.** The worker's `br create` wrapper copies stitch labels:

```python
parent_labels = current_bead.get("labels", [])
stitch_labels = [l for l in parent_labels if l.startswith("stitch:")]
# Append stitch_labels to br create --labels
```

**Why.** Stitches decompose work into bead graphs. Each bead carries a `stitch:<id>` label so HOOP can group them. When a worker creates follow-up beads, losing the label breaks Stitch lineage.

**NEEDLE change:** one line in the `br create` wrapper.

That's it. No API surface, no RPC, no shared library. HOOP is a reader + a process manager.

## What HOOP provides to NEEDLE

Three things, each deliberately outside NEEDLE's scope:

### 1. Fleet launcher

Reads a `fleet.yaml`:

```yaml
workspace: /home/coding/myproject
workers:
  - name: alpha
    adapter: claude
    model: opus
    strands: [pluck, explore]
  - name: bravo
    adapter: codex
    model: gpt-5.3-codex
    reasoning: medium
    strands: [pluck]
    max_wait_for_tasks: 1800
  - name: charlie
    adapter: opencode
    model: kimi-k2.5-free
    strands: [pluck, mend]
```

Spawns named tmux sessions (`needle-<workspace>-<name>`), registers each in `~/.hoop/fleet.db`, tails the logs. Stop/restart/reload are HOOP operations. **Does not touch the bead queue.** NEEDLE workers still coordinate only through `br` claims.

### 2. Steering surface

Three ways to steer, in order of cost:

**a. Direct bead ops (no agent).** UI buttons map to `br` commands: boost priority, close/cancel, reassign, create review bead, release stuck claim. Every button is a known queue mutation. Audit trail via `actor:fabric-hoop:<user>` field.

**b. tmux controls (no agent).** Pause worker (SIGSTOP), resume (SIGCONT), kill, restart. Never kills mid-bead silently — always releases the claim first so the bead returns to queue in a consistent state.

**c. Chat steering (small agent).** Haiku-tier model with a tool belt of (a) + (b) + read-only NEEDLE/bead introspection. User chats natural language; agent translates intent to mutations. Optional — HOOP works without it.

### 3. Observability that NEEDLE doesn't provide

- **Fleet map** — every worker, current bead, last heartbeat, outcome distribution.
- **Bead graph** — dependency DAG colored by state (claimable / claimed / blocked / closed), with priority ladder overlay.
- **Strand timeline** — how much fleet time is spent in Pluck vs Explore vs Mend vs Knot.
- **Cost per bead / per strand / per adapter** — joining session-file usage counters with the event stream.
- **Collision detection** — two workers editing overlapping files (requires reading the CLI session tool-use events).
- **Stuck detection** — worker with no heartbeat transition for N minutes, or looping on the same bead across retries.

FABRIC already plans several of these; HOOP gains them by linking to FABRIC rather than reimplementing. See the separation of concerns below.

## Separation from FABRIC

| Layer | NEEDLE | FABRIC | HOOP |
|---|---|---|---|
| Do work | ✓ | | |
| Read events / logs | | ✓ | ✓ |
| Render observability | | ✓ | ✓ (links to FABRIC for deep views) |
| Detect stuck/loop/collision | | ✓ | |
| Spawn/kill workers | | | ✓ |
| Mutate bead queue | ✓ (via claim/close) | | ✓ (via `br` writes) |
| Signal tmux / kill pids | | | ✓ |
| Natural-language steering | | | ✓ |

FABRIC stays read-only and deployable anywhere. HOOP owns the write path. Both consume NEEDLE's three hooks.

**URL-scheme bridge.** FABRIC renders action links like `hoop://release/bead/<id>`; HOOP handles the action. Clicking a "stuck worker" alert in FABRIC deep-links to a HOOP confirmation dialog. No UI merging, no write path in FABRIC.

## Invariants HOOP must preserve

Things NEEDLE relies on that HOOP must not break:

1. **Atomic claim remains via `br update --claim`.** HOOP never pre-claims on behalf of a worker. Every claim goes through SQLite transaction isolation.
2. **Deterministic priority ordering.** HOOP may observe the order but never rewrite it outside explicit user action (priority-boost is a user action, not a HOOP policy).
3. **Workers remain independent.** No HOOP-driven cross-worker coordination. A dead HOOP must not stop beads from being processed.
4. **Event stream is append-only.** HOOP reads; NEEDLE writes. Never edits history.
5. **Bead schema changes go through NEEDLE.** HOOP never migrates bead state; it uses `br` CLI verbs.

If HOOP dies, workers keep running. If HOOP comes back, it re-reads the event stream from disk and reconstructs state. This is the "no central orchestrator for work" property made explicit.

## Fleet state store

HOOP's only durable state is `~/.hoop/fleet.db` (SQLite):

- `fleets(id, workspace, manifest_path, started_at, stopped_at, stop_reason)` — one row per launch
- `workers(fleet_id, name, adapter, model, tmux_session, pid, status, started_at, stopped_at)` — one row per worker
- `actions(ts, actor, kind, target, args, result)` — audit log of every write HOOP performed

Nothing about bead state goes here — that's NEEDLE's job. This DB is recreatable from `fleet.yaml` + tmux state; losing it is inconvenient, not catastrophic.

## Event flow end-to-end

```
        NEEDLE worker "alpha"                                  HOOP
        ─────────────────────                                  ────
         br claim bd-abc123
              │
              ├──► .beads/events.jsonl  ◄────── tail -F ──────► fleet WS broadcast
              │     {event: claim, ...}
              ▼
         dispatch via YAML adapter
         (prompt prefixed with tag)
              │
              ├──► .beads/events.jsonl  ◄────── tail -F ──────► fleet WS broadcast
              │     {event: dispatch, ...}
              ▼
         child CLI runs; writes to
         ~/.claude/projects/...jsonl
              │
              │                          ◄────── 5s poller ────► conversation WS broadcast
              │                                  (joined by tag
              │                                  to bead-id)
              ▼
         CLI exits → br close / fail
              │
              ├──► .beads/events.jsonl  ◄────── tail -F ──────► fleet WS broadcast
              │     {event: complete, ...}
              ▼
         back to claim loop

         heartbeat every 10s ───────────► .beads/heartbeats.jsonl ──► fleet WS (throttled)
```

Three independent flows — bead events, conversation transcripts, heartbeats — each with its own rate and staleness tolerance. HOOP joins them by worker name and bead id, not by timing.

## Adapter parser responsibilities

To surface conversation transcripts and tool-use details in HOOP's steering chat, each CLI needs a parser that emits the unified event union. Per-CLI quirks already known from prior art:

- **Claude.** Native session UUID. Resume: `--session-id` turn 1, `--resume` after. Tool uses and usage fields in JSONL. Max-permissions env var exists.
- **Codex.** `thread_id` captured from `thread.started`. Resume: `codex exec resume <thread-id>`. Token usage via `token_count` events.
- **OpenCode.** Session id captured from first run output. Resume: `--session <sid> --continue`. Separate storage dirs for `message`, `part`, `session`.
- **Gemini.** Has its own live-spawn path and session files in `~/.gemini/tmp/`.
- **Aider.** Simpler — no session concept beyond the git history; prompt via `--message`. HOOP's parser for Aider is closer to "run → exit → done" than a streaming event union.

## When HOOP doesn't help

HOOP is a control plane; it's not useful for:

- **Debugging a specific bead's failure.** Use `br get <id>` + the CLI's own session replay. HOOP can *link* to these but won't replace them.
- **Cross-workspace coordination of work.** That's a NEEDLE strand concern (Explore). HOOP surfaces it; doesn't coordinate it.
- **Schema migrations.** `br` owns schema.
- **Running as a daemon with no UI.** Today HOOP is UI-first. A `hoop launch --headless` mode could exist but isn't the point — use NEEDLE alone for unattended fleets; add HOOP when you want visibility and steering.
