# Orchestrator problems and solutions

Field survey of operational problems observed across multi-agent coding-swarm control planes — mined from commit histories, incident docs, and public issue trackers of several prior-art projects. Names stripped; lessons preserved.

For each problem: what goes wrong, the root cause, the fix observed in the wild, and how it maps to a HOOP-over-NEEDLE design where SQLite + JSONL is authoritative, workers are headless tmux-hosted CLIs (Claude Code / Codex / OpenCode / Gemini / Aider), and control-plane responsibilities are launch + steer + observe.

---

## A. Session discovery & persistence

### A1. Provider session paths drift across versions and sandboxes

**Observed.** A Gemini harness setting `GEMINI_CLI_HOME=~/.gemini-sandbox/accN` caused sessions to land at `$HOME/.gemini/tmp/` rather than where the code assumed. Same class of bug has hit every CLI at least once.

**Fix.** A registry of `(provider, sandbox-root, session-subpath)` tuples, reprobed on startup, not hardcoded. Emit a warning when an expected directory is absent but a sibling path looks plausible.

**HOOP.** Ship a `SessionAdapter` per provider that is allowed to *discover* where sessions live rather than hardcode it. Bind adapter version to CLI binary version; re-detect when either changes.

### A2. First-turn vs resume semantics differ per CLI

**Observed.** Claude CLI rejects `--session-id <uuid>` on turns after the first ("Session ID is already in use") — you must switch to `--resume <uuid>`. Codex has a distinct `codex exec resume <thread-id>` form. OpenCode uses `--session <sid> --continue`.

**Fix.** Per-provider `create` vs `resume` invocation configs, gated by a persisted flag (`has_started_session`). Flag persists in durable storage so tmux orphans can be re-parented without corrupting the provider's session store.

**HOOP.** Persist the flag on the bead, not in memory. A bead carries `provider_session_id` and `session_created_at`; every resume uses those fields. Never rely on "most recent session in cwd" (`-c/--continue`-style brittleness).

### A3. Rewriting JSONL while tailing it corrupts readers

**Observed.** Providers append and occasionally rewrite/compact their own session files. A reader tailing those files during rewrite sees truncated or reordered events.

**Fix — the rule.** *Write events. Read projections. Never write projections.* Events are written once via `.tmp` + atomic rename, never modified. Any "live status" file is a lie waiting to go stale.

**HOOP.** NEEDLE's JSONL is already append-only. HOOP must follow the same rule for every artifact it writes — no `worker_status.json`, no `fleet_state.json`. Derive state at read time from durable event logs + process liveness.

### A4. Ghost / stale projections outlive reality

**Observed.** Dashboards showed dead runs as "Running" for days because liveness was computed from a persisted `live-summary.json` that survived the process. One project had 51+ runs wedged in "running" state across five workspaces.

**Fix.** Liveness is a *process property*, never a file property. Record the worker PID in a `started` event; compute alive = `started && !stopped && kill -0 pid`. Optional heartbeat-with-TTL as a secondary signal.

**HOOP.** The fleet SQLite stores `worker(pid, started_at, stopped_at, last_heartbeat)`. Liveness view joins pid-alive + heartbeat-fresh. Worker-state strings (`executing`, `idle`, `knot`) are derived on read from bead-event history + last heartbeat, not stored.

---

## B. State reconciliation (dual-state recon)

### B1. UI id vs provider session id drift

**Observed.** UI mints a stable id (`d97e...`); the CLI creates its own session id (`019c...`). Merging local + server state on reconnect preserved stale UI ids that no longer matched any provider session. Same logical thread appeared twice.

**Fix.** Explicit dual-identity in the schema. `conversation.id` is UI-stable; `session_id` is provider-native; emit a `session_bound` event when they first meet. A `findBootstrapMatch` step ties newly-discovered session files back to in-memory ids by `(cwd, first-prompt-hash)` before they can register as duplicates.

**HOOP.** Bead id is UI-stable and permanent. Provider session id is captured when the CLI first reports it and stored as a bead field. Emit a `bead.session_bound { bead_id, provider_session_id, ts }` event when binding happens. Downstream analytics key off bead id only.

### B2. `init` merge preserves stale state

**Observed.** On WS reconnect, clients merged local state with server state. Stale rows survived forever.

**Fix — "aggressive epoch sync."** Server wins. Client does total wipe + replace on `init`, preserving only un-sent optimistic stubs.

**HOOP.** Same. On every WS (re)connect, the client empties its state map and rebuilds from the server's `init` payload.

### B3. Streaming buffer + committed history double-merge

**Observed.** UI stitched `text.delta` chunks into `messages[]`. Server crash mid-turn left the UI with partial content that disagreed with disk. Partial content reappeared on reload.

**Fix — "in-flight isolation."** Streaming deltas go to a *volatile buffer* (e.g. a separate reactive map). On `turn.complete`, the buffer is dropped and the UI re-reads committed messages from disk via a batched `conversations_updated` event.

**HOOP.** Live token stream from steering-chat turns goes to a `streamingContentAtom`, never into the canonical bead history. Bead transcripts render from the committed event log only.

---

## C. Worker lifecycle

### C1. Silence during tool execution looks like a hang

**Observed.** Some CLIs emit zero stdout during tool execution plus silent API-level retries. A naive idle-timer kills a healthy worker.

**Fix.** Three separate timers per worker, not one:
- `idle_timeout` — no events, any type, for N seconds
- `max_turn_runtime` — hard ceiling regardless of activity
- `content_seen_grace` — extended idle tolerance once the worker has produced real content (distinguishes "thinking" from "dead")

Heartbeat emitted by the harness wrapper, not the CLI — the CLI can't prove it's alive. Heartbeat is *capped* (~20 min) so a truly hung API call can't life-support the worker up to its max-runtime ceiling.

**HOOP.** All three timers are per-worker config with sensible defaults. Stall events include `saw_content: bool` so the distinction is preserved in telemetry.

### C2. Zombie processes & tmux orphans outlive the control plane

**Observed.** Detached/`unref`-ed child processes survived hot-reloads. Multiple dev watchers competed on the same port. Corrupted `.git/worktrees/` entries poisoned `git worktree add` for every other worker — one stale entry made a worker run 20 cycles doing nothing.

**Fix.** 
- Graceful shutdown: SIGTERM → grace window → SIGKILL.
- On startup, enumerate known worker PIDs, `kill -0` each, reconcile.
- `git worktree prune` unconditionally at startup (metadata only, never auto-destroy dirty worktrees — concurrent runs would be disrupted).
- Single-instance lockfile at workspace root; second instance refuses to start or runs read-only.

**HOOP.** On launch, HOOP reads its fleet DB, `kill -0` every recorded PID, reconciles. `hoop launch` takes a filesystem lock on `$WORKSPACE/.hoop.lock`; a second launch from the same workspace sees the lock and offers attach, not clobber.

### C3. Crash-loop from infinite retry on transient auth churn

**Observed.** Concurrent auth-token refreshes caused repeated process crashes; the server respawned aggressively and re-sent the same prompt, amplifying the problem.

**Fix.** Per-bead `attempts` counter; after N consecutive failures on the *same* bead, park the bead (don't re-queue). Consecutive-error counter with exponential backoff (`60 * 2^(n-1)` seconds) plus a cap (e.g. max 5 consecutive errors).

**HOOP.** Every claim records attempt count on the bead. Auto-park and escalate past the ceiling; raising the ceiling is an operator action, not automatic.

### C4. Multiple control-plane instances running at once

**Observed.** Multiple watchers on the same workspace produced amplified interrupts and confusing UI state.

**Fix.** Pidfile + advisory filesystem lock at workspace root. Detect-and-warn diagnostic.

---

## D. Claim races & atomicity

### D1. Leaked claims from crashed workers

**Observed.** Tasks moved `pending/ → current/` via filesystem `mv` stayed in `current/` forever when the worker crashed.

**Fix.** Claims carry `(bead_id, worker_id, claimed_at, lease_expires_at)`. Control plane periodically scans expired leases whose worker PID is dead; emits `claim_released`; bead becomes claimable. **Never auto-recycle an active worker's claim** — that causes two workers to race.

**HOOP.** `br` already provides atomic claims. HOOP adds a "lease scanner" as a periodic job that only touches claims whose worker is demonstrably dead (PID absent AND heartbeat stale > grace). Recycling is HOOP's responsibility; claiming remains NEEDLE's.

### D2. Worktree collisions

**Observed.** Worktree naming that used only the worker id caused re-runs in the same repo to stomp each other.

**Fix.** Worktree paths keyed by `(run_id, worker_id, attempt)`. Always `git worktree prune` at startup.

**HOOP.** Worktree template: `<repo>/.worktrees/<fleet-id>-<worker>-<attempt>`. Prune metadata on every `hoop launch`.

### D3. The "stale task loop" — claim + no-diff cycle

**Observed.** Worker claims a bead, investigates, decides the work is already done, signals completion with no code diff. Framework sees no diff, recycles the task to pending. Next worker picks it up. Loop forever.

**Fix.** Allow "completed without artifact" as a valid resolution. A bead closed with a justification string is closed. The closure reason is first-class.

**HOOP.** Bead close events carry a `resolution` field (`merged | closed-no-artifact | parked | rejected`). Steering agent and reviewer beads honor `closed-no-artifact` as terminal.

---

## E. Completion detection

### E1. "Done" has many definitions and they disagree

**Observed.** Codex JSONL sometimes ends on reasoning events with no `task_complete` marker. Process exit does not mean semantic completion. An explicit `{type:'result'}` marker is most reliable but not universal.

**Fix.** Completion is a union: `process-exit | final-event | explicit-token-marker`. Process exit is *lifecycle* canonical. Token markers (`BEAD_DONE(id, notes)`, etc.) drive *semantic* disposition: close, followup, park.

**HOOP.** NEEDLE already treats exit code 0 as the authority for lifecycle. HOOP's harness parser additionally scans stdout for semantic markers; these route the bead, they don't override exit code.

### E2. Externally imported sessions have no completion reason

**Observed.** Session files ingested from disk that weren't spawned by the control plane carry no reliable status.

**Fix.** Tag externally-discovered sessions `unknown_external`. Never attribute them to a bead.

**HOOP.** Session prefix tag (`[needle:<worker>:<bead-id>:...]`) is the attribution primitive. If the tag is missing, the conversation is surfaced but not joined.

---

## F. Streaming & message ordering

### F1. JSON parsing on split stdout chunks

**Observed.** Stream-JSON output chunked by the OS at non-boundary offsets. Naive `JSON.parse(line)` crashes.

**Fix.** Line-buffered NDJSON reader that carries partial lines across chunks.

**HOOP.** Every provider parser uses a shared NDJSON line buffer. This is cheap to do right and expensive to patch later.

### F2. Queue-while-running + interrupt race

**Observed.** User sends a message in the tiny window between "claim recorded" and "first stdout emitted." The `isRunning` flag is still false; message routes as a new turn instead of as a queue item.

**Fix.** Gate on a superset flag `hasActiveTurn = isRunning || isStreaming || sending`, not a single boolean.

### F3. Shared `/g` regex races across parsers

**Observed.** A regex with the `g` flag shared across async callers leaks `lastIndex`; parsing is non-deterministic under concurrency.

**Fix.** Lint rule or per-call regex construction. Never share stateful regexes.

### F4. ANSI/terminal control sequences leak into parsed text

**Observed.** Some providers emit ANSI codes that corrupt downstream UI.

**Fix.** Strip ANSI at the harness boundary.

---

## G. Session resume / fork

### G1. Duplicate session files per prompt

**Observed.** Some harness paths produced two JSONL session files for a single invocation.

**Fix.** Integration test per harness: one prompt → exactly one session artifact.

### G2. Fork emulation is hard to get right

**Observed.** Only some CLIs support real fork; others require resume-with-edited-history emulation. Emulation has subtle race conditions during the "opaque fork call."

**Fix.** Prefer native fork where supported; explicit race guard on emulated paths.

**HOOP.** NEEDLE doesn't need fork at the provider level. "Fork" in bead terms is "create N review beads as deps of the parent" — handled by the dependency graph, not the session layer.

### G3. `-c / --continue` is brittle

**Observed.** Resuming "the most recent session in this directory" surprises automation when other tools also write sessions in the same cwd.

**Fix.** Always pass explicit session ids. Never rely on implicit "most recent."

---

## H. Review / gating loops

### H1. Infinite review loops (reviewer bias)

**Observed.** Review prompts subtly biased ("Do NOT use REJECTED") caused reviewers to always return `NEEDS_CHANGES` → revision loop with no exit. Other prior art: `allow_delegation=True` producing infinite delegation loops; swarm agents failing to pick a handoff target with empty messages; termination conditions evaluating true but streams not stopping.

**Fix.**
- Neutral review prompts with `APPROVED` and `NEEDS_CHANGES` as equal options.
- Hard `max_review_rounds` per cycle (observed default: 7).
- Global `cycle_budget` per bead.
- `max_attempts` per bead with auto-park past the ceiling.

**HOOP.** Every bead carries `max_attempts`, `max_review_rounds`, `cycle_budget`. Ceilings are non-negotiable defaults, overridable only by explicit operator action.

### H2. Merge-conflict resolution: one-shot vs retry, and over-specified prompts

**Observed.** Single-shot "make this branch cleanly mergeable" often didn't actually merge. Over-specified prompts (`git merge main`, `git add` …) trapped the agent when step 3 failed — no recovery path.

**Fix.** Bounded retry (~5 attempts). Prompts are *goal-oriented*, not method-dictating: "Make branch clean against main, preserve your changes, don't stop until done."

**HOOP.** Prompt library ships goal-oriented variants. Review beads use them by default.

---

## I. Cost & token accounting

### I1. Provider-specific usage parsing and artifact drift

**Observed.** Each CLI surfaces token usage differently and field names drift between versions. Usage-limit errors get swallowed.

**Fix.** Generic usage types per provider adapter. Surface provider-level usage-limit errors in the UI, don't swallow.

**HOOP.** Usage events are parsed per-provider into a canonical shape and summed at the bead level (workers touch many beads; worker-level totals are derived views).

### I2. Budget-before-runaway

**Observed.** Hard spend ceilings (`--max-budget-usd`, `--max-turns`) are the cheapest safety rails to retrofit into a design. Retrofit is expensive.

**HOOP.** Per-bead spend cap enforced by killing the worker (releasing the claim) when exceeded. Fleet-level daily spend cap. Both surfaced in fleet config.

---

## J. Multi-tenant / multi-project

### J1. Global session dir leaks into every project view

**Observed.** Adapters ingesting `~/.codex/sessions/**` showed external runs from other repos as phantom entries in the current project's UI.

**Fix.** Filter *client-side* by active cwd (`workspaceConversationsAtom`). Do **not** filter server-side — that breaks "two projects open" views.

**HOOP.** Server always sends everything; client scopes by active workspace. Operators with multiple workspaces open get a unified view if they want one.

### J2. Workspace resolution

**Observed.** Path variation (`/Users/x` vs `/home/x`, symlinks) made the same workspace appear as two.

**Fix.** Canonicalize via `realpath` at ingestion; store both raw and canonical.

---

## K. Permissions & safety

### K1. Permission state doesn't propagate to concurrent children

**Observed.** Parent's permission state didn't propagate to parallel sub-agents. Mid-session UI-accepted permission grants didn't propagate either.

**Fix.** Freeze the permission surface at worker spawn — env vars + a copied `settings.json` snapshot. Don't inherit a live reference. Mid-run grants require an explicit broadcast event.

**HOOP.** Worker spawn materializes a permission snapshot into the tmux session's env. Permission grants during a run are a HOOP operation that rewrites and broadcasts to live workers.

### K2. Path traversal via unsanitized id fields

**Observed.** Wire-level ids landed in filesystem paths unchecked → path traversal risk.

**Fix.** Regex-validate every id used in a filesystem path at the WS/REST boundary. Zod/schema validation on every incoming message.

**HOOP.** Bead ids must match `^bd-[a-z0-9]+$`. Worker names must match `^[a-z]+$` (NATO alphabet). Both enforced at the schema boundary before any filesystem use.

### K3. `--yolo` / max-permissions as default

**Observed.** Running every worker with `--dangerously-skip-permissions` as a default is pervasive in prior art; it expands blast radius.

**Fix.** Yolo mode is opt-in per workspace and must be accompanied by a filesystem jail (sandbox-exec on macOS, bubblewrap/firejail on Linux, or a container). Every permission elevation is logged as a durable event.

**HOOP.** `fleet.yaml` requires `yolo: true` opt-in at workspace level plus a `sandbox:` stanza describing the jail. No yolo without a jail.

---

## L. Config & schema evolution

### L1. Schema migration across coupled repos

**Observed.** An EDN → JSON migration rippled across orchestrator + viewer repos; partial conversions produced silent gaps.

**Fix.** JSON Schema (draft-07) as the cross-repo contract. One source of truth, TypeScript/Python/Rust all codegen off it. `schema_version` field on every row; migration is a first-class operation.

**HOOP.** Bead schema lives in a `hoop-schema/` directory shared between HOOP, FABRIC, and any future consumers. Every record carries `schema_version`.

### L2. Config knob creep

**Observed.** Three overlapping counters (`max_cycle`, `max_iterations`, something else) → ambiguity → unified to two, with *hard errors* on deprecated keys during the transition.

**Fix.** Every knob has a cost. Collapse related counters aggressively. Hard-error on deprecated keys, don't warn silently.

### L3. Unguarded `JSON.parse` on disk artifacts

**Observed.** One corrupt JSONL line took down the control plane. Affected: session file parsers, localStorage readers, path autocomplete, review loaders.

**Fix.** Every disk-read JSON goes through try/catch + schema-validate, with per-file quarantine. A single bad line must not crash anything.

**HOOP.** Shared `parseJsonlSafe` helper used by every reader. Quarantine directory: `~/.hoop/quarantine/<ts>-<reason>/` with a daily report.

---

## M. Observability gaps

### M1. Silent drops in event parsers

**Observed.** Parsers silently dropped unknown event types. Watchdogs never saw a heartbeat → false stall kill.

**Fix.** Catchall in every event-type switch must log the raw event at `warn` and emit a `progress` event. Metric counter for unknown-event frequency per provider (spikes signal CLI version drift).

**HOOP.** No silent defaults in any parser switch. Unknown-event counters exposed on the health endpoint.

### M2. Fabricated timing data

**Observed.** Dashboards filled timing holes with interpolated/synthesized data. Misleading during post-mortem.

**Fix.** Never fabricate. Show gaps explicitly. Synthesize summaries only from actual data, never from assumed timings.

**HOOP.** Timeline renders gaps as gaps. Any synthesized field is labeled and traceable to its source events.

### M3. Missing diagnostics endpoint

**Observed.** Incident triage was expensive because no endpoint surfaced runtime identity aliases and the active process map.

**Fix.** Ship `/debug/state` on day one: every worker PID, claim, lease expiry, lingering tmux session, id alias table.

**HOOP.** `/debug/state` returns fleet roster + NEEDLE reconciliation (claimed beads, lease table) + session-id alias table. Gated by local-only access by default.

### M4. Collision detection between workers on the same files

**Observed.** Two workers in the same repo editing the same files corrupted diffs. Per-process logs + git lock files + re-check of added files used in prior art.

**Fix.** Worktree-per-worker (git-level isolation). Per-worker log path with `worker_id` in the filename. Emit a `touched_files` event so the control plane can detect overlap.

**HOOP.** Worktree isolation is baseline. Collision detector reads `touched_files` events across active workers; overlap surfaces as an alert with a suggested action (reassign, pause, or bless).

### M5. Silent tmux-orphan channels

**Observed.** tmux sockets created by automation (`-L <socket>`) weren't visible to operators who ran plain `tmux ls`. Parent reported "spawned successfully" while the agent never started because `send-keys` payloads exceeded ~255 bytes and were silently truncated.

**Fix.**
- Verify spawn success: worker must ack (write a known file or emit a known event) within N seconds; otherwise spawn is failed.
- Use file-based command delivery (write a temp script, `send-keys "source $f"`) so transmitted payloads stay < 200 bytes regardless of command length.

**HOOP.** `hoop launch` writes a short bootstrap script per worker and sends `source <path>`. Each worker writes `~/.hoop/workers/<name>.ack` within 10s of start; absence = failed spawn with exact diagnostic.

### M6. Stale worker status when parent dies without writing stopped

**Observed.** Orchestrator process died without `stopped.json`. Dashboards stayed "running" forever. Fix was to widen the force-done guard from "has stopped record" to "is not process-alive."

**Fix.** Always derive worker/run liveness from process-alive first. `stopped` records are optional evidence, not required.

### M7. Operators want salvage, not cleanup

**Observed.** `oompa salvage`-style commands: "discover surviving worktrees and orphan branches with unmerged changes, try to resume sessions, fall back to fresh sessions with partial-diff context, then run review + merge." Resume path bugs are the #1 complaint on some prior-art projects.

**Fix.** Build the salvage path in from day one. Surviving worktree + a claimed-but-unfinished bead is the *common* case. Default to attempt-resume, fall back to replay-with-context; never just recycle silently.

**HOOP.** `hoop salvage` command: scans for dead workers with active claims, inventories their worktrees, and produces a resume plan. Operator approves; HOOP executes via `br` claims and targeted provider resumes.

---

## Highest-leverage: prevent by design, not by patch

A dozen decisions that prior art paid to learn. Cheapest when baked in from day one:

1. **Events only, never projections.** One authority per fact; derive everything at read time.
2. **Dual-identity in schema from day one.** Bead id (stable) + provider session id (derived). Explicit `session_bound` event. Retrofitting this is where prior art bled for weeks.
3. **Server is the epoch.** Clients do total-replace on `init`, not merge.
4. **Liveness = process, never file.** `kill -0 pid && !stopped_record`.
5. **Multi-timer watchdogs with content-awareness.** idle / max-runtime / content-seen-grace. One unified timer is wrong.
6. **Never silent-drop unknown events.** Log, emit progress, count.
7. **Claim leases with TTL, not `mv` semantics.** Lease scanner releases only when worker is demonstrably dead.
8. **Atomic `.tmp` + rename; line-buffered NDJSON reader.** Cheap; blocks an entire bug class.
9. **JSON Schema as cross-repo contract.** Version field on every row. Migration is a first-class verb.
10. **Hard ceilings on retry / review rounds.** `max_attempts`, `max_review_rounds`, `cycle_budget` — per bead.
11. **Workspace canonicalization at ingestion.** `realpath` everything; store both raw and canonical.
12. **Verify spawn success.** tmux `send-keys` can silently truncate. Require worker-ack before believing spawn succeeded.

## How this maps onto NEEDLE's existing primitives

Several of these problems are already solved by NEEDLE's design and need nothing from HOOP except to not break them:

- Atomic claims (SQLite transaction) → D1
- Append-only JSONL → A3, A4
- Deterministic priority ordering → D1, parts of H1
- Explicit outcome paths (success / failure / timeout / crash) → C1, E1
- Strand escalation → H1's escape hatch when queue is empty

The remaining problems are where HOOP earns its keep — lifecycle (C*), observability (M*), schema evolution (L*), permissions (K*), steering/review ceilings (H*). Baked in from day one, they come for free. Retrofitted later, they bleed.
