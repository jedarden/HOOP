# Reference feature inventory

Distilled from a prior-art multi-agent ADE (Agent Development Environment) that coordinates swarms across Claude Code, Codex, Gemini, and OpenCode. Names and branding stripped — the interest is in what capabilities a mature control plane actually ships, so HOOP can pick, adapt, or reject each on its merits.

## 1. Cross-agent conversation aggregation

**What it does.** Single UI lists every conversation from every installed headless CLI, grouped by working directory.

**How.** A `DiskAdapter { provider, discoverFiles, parseFile }` interface per CLI. Server loads in two phases: (1) stat every session file, sort by mtime; (2) parse in parallel with bounded concurrency (~16) and stream batches to the client as they complete. A background poller (5s) re-scans for external edits (user ran the CLI directly in another terminal).

**Disk locations** — same for HOOP:
- Claude Code — `~/.claude/projects/`
- Codex — `~/.codex/sessions/YYYY/MM/DD/`
- Gemini — `~/.gemini/tmp/`
- OpenCode — `~/.local/share/opencode/storage/{message,part,session}/`

**HOOP relevance.** Essential. Bead dispatches prefix prompts with `[needle:<worker>:<bead-id>]` — the disk adapter extracts that tag on parse and joins session → worker → bead → outcome without any new storage layer.

## 2. Live conversation mode

**What it does.** Launch or continue a conversation against any supported CLI from the UI; stream tokens live.

**How.** Per-turn child process (no long-running worker). Session continuity is provider-specific:
- Claude: `--session-id <uuid>` turn 1, `--resume <uuid>` after
- Codex: `codex exec --json -C` turn 1 (prompt via stdin), `codex exec resume <thread-id> --json -` after
- OpenCode: `opencode run --format json`, then `--session <sid> --continue`
- Gemini: native live-spawn path

A thin adapter layer (`harness` + `parser` pair per CLI) normalizes argv construction and stdout parsing into one unified event union:

```
session.started | turn.started | text.delta | tool.use |
progress | stderr | error | out_of_tokens | turn.complete
```

**HOOP relevance.** NEEDLE workers are already headless one-shots; HOOP's "live conversation" role is not for NEEDLE workers — it's for **ad-hoc steering conversations** (user chats with the fleet steering agent, or fires a one-off diagnostic prompt against an adapter). Reuse the same unified event union so session playback works identically whether the conversation came from a bead or from an ad-hoc chat.

## 3. Server-owned message queue

**What it does.** User can enqueue more turns while a conversation is mid-stream; queue survives refresh and disconnect.

**How.** `Conversation.queue: QueuedMessage[]` held server-side (explicitly moved off the client after a design doc revision). WS ops: `queue_message`, `interrupt_and_send`, `cancel_queued_message`, `clear_queue`. Server broadcasts `queue_updated` so multiple tabs mirror state.

**HOOP relevance.** Applies to the steering chat pane, not to NEEDLE beads. NEEDLE already has the authoritative queue; don't duplicate it.

## 4. Sub-agent panel

**What it does.** When the agent uses `Task` / thread-spawn / collab tools, UI shows each sub-agent with independent status, token usage, current action.

**How.** Sub-agent tool detection module; parent conversation tracks `subAgents[]` with discriminated `statusSource: 'native' | 'inferred_parent_completion' | 'recovered_from_disk'`. Codex sub-agents are linked via `session_meta.payload.source.subagent.thread_spawn.parent_thread_id` → parent conversation id.

**HOOP relevance.** Useful inside the steering chat. For NEEDLE fleet view, the analogue is the **bead dependency graph** — a claimed bead that blocks other beads is the same shape as a parent with sub-agents.

## 5. Swarm orchestration

**What it does.** Long-running autonomous pool of workers cycling through work → review → merge/reject. Configured via a single JSON file.

**Config shape.**
```json
{
  "planner":  { "model": "codex:gpt-5.3-codex:high", "prompt": [...], "max_pending": 18 },
  "reviewer": { "model": "claude:opus", "prompt": [...] },
  "workers": [
    { "model": "codex:gpt-5.3-codex:medium", "prompt": [...], "max_cycle": 8, "count": 3, "can_plan": false, "max_wait_for_tasks": 1800 },
    { "model": "claude:opus", "prompt": [...], "max_cycle": 4, "count": 1, "wait_between": 20 }
  ]
}
```

**Roles.** Planner emits tasks → workers claim + execute in git worktrees → reviewer verdicts `approved | needs-changes | rejected`. Completion flows move JSON files `pending/` → `current/` → `done/`.

**On-disk run artifacts** (per run, under `<project>/runs/<run-id>/`):
- `started.json` — `{ run-id, started-at, pid, config-file, workers[], planner, reviewer }`
- `stopped.json` — `{ stopped-at, reason: completed|interrupted|error, error? }`
- `summary.json` — written on clean exit; synthesized by server if missing
- `cycles/{worker-id}-c{N}.json` — `{ worker-id, cycle, outcome, timestamp, duration-ms, claimed-task-ids, recycled-tasks, review-rounds, error-snippet }`
  - outcomes: `merged | rejected | error | done | executor-done | no-changes | working | claimed | sync-failed | merge-failed | interrupted`
- `reviews/{worker-id}-c{N}-r{round}.json` — `{ verdict, timestamp, output, diff-files }`

**Liveness.** Server checks `stopped.json` absence, then pings the `pid` from `started.json`.

**HOOP relevance.** This is the feature NEEDLE most benefits from. Direct mapping:

| Reference artifact | NEEDLE equivalent |
|---|---|
| `started.json` / `stopped.json` | Fleet roster bead (genesis) with open/closed state |
| `cycles/<wid>-c<N>.json` | Cycle-type bead closed with outcome fields |
| `reviews/<wid>-c<N>-r<R>.json` | Review-type bead blocking the cycle bead |
| `tasks/pending/*.json` | Open claimable beads |
| `tasks/done/*.json` | Closed beads |
| Planner role | Bead-generator (see `SPINDLE` concept) |
| Reviewer role | Review-type bead, claimable by any adapter |
| Worker `max_cycle`, `max_wait_for_tasks`, `wait_between` | Per-worker manifest fields in `fleet.yaml` |

The reference implementation keeps all this in scattered JSON files because it has no authoritative queue. HOOP + NEEDLE collapse it: the bead queue *is* the run state. No `runs/` directory needed.

## 6. Merge-conversations feature (fan-out review)

**What it does.** Fork a live conversation N ways; each child receives a hardcoded review prompt and writes a review doc; parent aggregates.

**How.** Parent conversation stores `mergeParentMeta = { children: [{sourceConversationId, childConversationId, reviewUuid, childWorkingDirectory}], prefixInjected }`. Native `--fork` where supported; cp+resume emulation otherwise. Children write to `merge_review_docs/REVIEW_DOC_<uuid>.txt`.

**HOOP relevance.** In NEEDLE terms this is "N review beads spawned as dependents of one implementation bead, each assigned to a different adapter/model, results aggregated by a synthesis bead." Same outcome, no fork-emulation dance — the dependency graph already expresses fan-out.

## 7. Fork / resume threads

`resumedFromConversationId` on the conversation record. Fork source session id passed as a spawn arg. Useful for "rewind to message N and branch."

**HOOP relevance.** Analogue: create a bead with `forked_from: <bead-id>` and a cursor pointing at a message index in the parent's transcript. Steering action, not a core NEEDLE feature.

## 8. Search palette

Client-side substring search over all conversations. Case-insensitive `includes`, 150ms debounce, cap at 50 results, 60-char snippets around matches. Cmd+K.

**HOOP relevance.** Same. Operates over the disk-adapter cache. No server roundtrip.

## 9. Audit overlay

On server startup, enumerate every configured headless CLI binary and report availability. If any missing, UI overlays a blocker prompting install. Keeps the "why isn't it working" loop to seconds, not debugging.

**HOOP relevance.** Essential. HOOP adds NEEDLE-specific checks: `br` available, `.beads/` accessible, tmux server running, bead schema version compatible.

## 10. Usage / cost panel

`GET /api/usage?days=N` aggregates tokens + cost:
- Claude: JSONL `message.usage` with input/output/cache-read/cache-write
- Codex: `event_msg.payload.type=token_count` events
- OpenCode: per-message `tokens.{input,output,cache.{read,write}}` + `cost`

Pricing hardcoded per provider. Per-session + per-day breakdown, top sessions, rate-limit windows (5h + 7d for Claude).

**HOOP relevance.** Essential for a fleet — cost/day per adapter is a primary metric. Pull the same accounting logic into the fleet event tap; cross-reference with bead events to get cost-per-bead, cost-per-strand, cost-per-workspace.

## 11. Palette generator (theme)

`POST /api/generate-palette?provider=...` runs the adapter in single-shot mode with a theme-extraction prompt, parses returned JSON into a 14-slot palette, caches under `~/.agent-viewer/palettes/palette_N.json`.

**HOOP relevance.** Novelty feature. Skip for v1.

## 12. Saved prompts library

Client-side library of reusable prompts (`PromptPalette` + `useSavedPrompts`).

**HOOP relevance.** Maps onto NEEDLE prompt includes — bead prompts already resolve from a library in the workspace. UI can surface the same library for ad-hoc/chat use.

## 13. Per-conversation reasoning effort

Pass-through string enforced at the WS boundary:
- Claude: `low | medium | high | xhigh | max`
- Codex: `minimal | low | medium | high | xhigh`

Server validates via `isEffortValidForProvider()` — on reject, emits `error` and rebroadcasts authoritative state so optimistic UI rolls back.

**HOOP relevance.** Same. Field on the adapter config + per-worker override. Bead can also carry an effort field to override for specific work items.

## 14. UI state server sync

`~/.agent-viewer/ui-state.json` persists active conversation, expanded projects, done conversations, promoted workers, last-seen message index per conversation, sidebar view mode. Sent in the WS `init` payload.

**HOOP relevance.** Same pattern. Persist at `~/.hoop/ui-state.json`.

## 15. Local-domain auto-setup

On macOS launch, a setup script edits `/etc/hosts` once (prompts sudo) so `http://<name>.localhost` resolves. Dev/prod port split: Vite on one port, API on another, single process in prod.

**HOOP relevance.** Optional polish. If HOOP serves over Tailscale (which is this environment's norm), skip local-domain routing and bind to the Tailscale hostname instead.

## 16. WS event schema hygiene

One quirk worth copying: the reference implementation has **no `conversation_updated` event type**. Setters reuse `conversation_created` for both create and mutate. On reject, server emits `error` + authoritative `conversation_created` rebroadcast, letting the client roll back optimistic writes cheaply.

**HOOP relevance.** Apply the same principle to fleet/bead events. Use `bead_state` for both create and update; use `worker_state` for both register and update. Fewer event types, fewer bugs.

## 17. Dual-state reconciliation engine

Two sources of truth race: live WS stream and the 5s disk poller. Rules the reference documents:

1. UI `id` is permanent; CLI `sessionId` is provider-native. Server emits `session_bound` to bind them.
2. Server wins on reconnect. Client wipes its conversation map on `init`, preserving only un-sent optimistic stubs.
3. Bootstrap interceptor aliases newly-discovered JSONL files back to existing in-memory UUIDs to prevent poller duplicates.
4. Streaming text writes to a separate `streamingContent` map, not to committed messages. Committed messages come from the poller on a batched `conversations_updated` event.

**HOOP relevance.** Half of this problem evaporates when the queue is authoritative (bead state is SQLite + JSONL, not scattered CLI files). But the **other half survives** — HOOP still reads native CLI session files for conversation playback, and those are written behind its back. Keep the interceptor pattern for that source specifically. See `architecture-patterns.md`.

## 18. Extension surfaces (stable-ish)

Five points where the reference is explicitly extensible:

1. **Provider adapter triad** — `harness` (argv builder), `parser` (unified-event emitter), `provider` (name + model list).
2. **DiskAdapter interface** — `{ provider, discoverFiles, parseFile }`.
3. **Unified runtime event union** — `session.started | turn.started | text.delta | tool.use | progress | stderr | error | out_of_tokens | turn.complete`. Stable cross-harness API.
4. **Swarm config schema** — one JSON file with planner/reviewer/workers[].
5. **Run-artifact JSON schemas** — generated types from a single schema source.

**HOOP relevance.** Adopt (3) wholesale. For (1) and (2), NEEDLE's YAML adapter already does the harness job; HOOP just needs a parser module per CLI. Fleet config (4) replaces the swarm config, expressed as `fleet.yaml`. Run-artifacts (5) are replaced by bead schema.
