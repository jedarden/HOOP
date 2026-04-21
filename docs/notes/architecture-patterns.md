# Architecture patterns worth absorbing

Design decisions observed in mature multi-agent control planes that HOOP should adopt, adapt, or explicitly reject. Grounded in a prior-art TypeScript reference implementation; generalized here so the decisions stand on their own.

## Process model: single server + per-turn child processes

**The pattern.** One long-lived Node/server process. Per conversation turn, spawn a short-lived child that runs the headless CLI, streams events, exits. Session continuity is the CLI's responsibility (`--resume`, `--session-id`, `thread_id`). The server never keeps a persistent child process per conversation.

**Why it works.** No process-leak surface area. A crashed child can't hang the server. Session resume via the CLI's native flags is deterministic — the CLI's own session file is the source of truth for continuity. Scales to many conversations because idle conversations cost nothing.

**HOOP adaptation.** HOOP inherits this for its own steering chat. NEEDLE workers are *already* per-bead one-shots following the same model — the tmux session is just a supervisor loop that spawns the CLI child per bead. No change needed there.

## Communication: one WebSocket + REST sidecar

**The pattern.** All live state goes over a single WS on `/`. REST endpoints (`/api/*`) are used only for: (a) point-in-time reads that don't need to be live, (b) operations that take a query-string filter, (c) things that predate the WS channel and weren't migrated.

**Why.** One connection, one authoritative broadcast path. Reconnect logic lives in one place. REST is escape hatch, not primary API.

**HOOP adaptation.** Same. Single WS for fleet state, bead events, worker heartbeats, steering chat stream. REST for: historical reports (cost by day, bead outcomes by strand), bulk exports, health probes.

## Schema: Zod on both sides, discriminated unions for message types

**The pattern.** `shared/` package with Zod schemas for every message. Both client and server `safeParse` before acting. Message unions discriminate on a `type` field. Reject path is explicit: log, emit `error`, rebroadcast authoritative state.

**Why.** Every malformed input is caught at the boundary with a precise error. Type narrowing inside handlers comes for free. Evolving a message type is additive and visible.

**HOOP adaptation.** Same. One `shared/` package (or `hoop-schema` crate if Rust) with every WS/REST message typed.

## State: server-authoritative, client-optimistic, rollback by rebroadcast

**The pattern.** Client may apply optimistic updates. On reject, server doesn't send a "reject" message — it emits a generic `error` + the authoritative state. Client's state update path is the same whether the update was accepted or rejected, which removes an entire class of reject-path bugs.

**Why.** Reject paths are the least-tested code. Reusing the happy path keeps them honest.

**HOOP adaptation.** Apply to every steering mutation: pause worker, boost bead, release claim, etc. User clicks → client shows pending → server either broadcasts the new authoritative state or broadcasts the old state (with an error toast). Same rendering pipeline either way.

## Reconciliation: two clocks, two rules

When two sources of truth update the same entity (live WS stream + 5s disk poller), you need explicit rules for who wins and how duplicates are prevented. The reference documents four:

1. **UI id is permanent.** Provider session-id is native and may appear later. Emit a `bind` event when they first meet.
2. **Server wins on reconnect.** Client wipes its state on `init`, preserving only truly local un-sent drafts.
3. **Bootstrap interceptor on the poller.** When the poller finds a new file, check if its session-id is already aliased to a live in-memory id. If so, merge; don't create a duplicate.
4. **Streaming is a different map than committed messages.** Token deltas go to `streamingContent`; committed messages come from the poller. A message is "real" only when the poller has seen it.

**HOOP adaptation.** HOOP has *fewer* reconciliation problems than the reference because the bead queue is authoritative — NEEDLE owns that path, nobody else writes to it. But:

- Native CLI session files are still written behind HOOP's back (every Claude/Codex/OpenCode/Gemini invocation writes its own session log). The interceptor pattern applies to *that* source.
- The steering chat is live-streaming + resumable. It needs the same streaming-vs-committed split.
- Fleet roster (who's in `fleet.yaml` vs who's actually in tmux) is a two-clock problem. Use rule 2: manifest is declarative truth; actual tmux state reconciles on next poll.

## Adapter layering: harness + parser + disk-adapter

**The pattern.** Each headless CLI gets three files:
- `harness/<cli>.ts` — builds argv, handles flags, constructs stdin payloads
- `parser/<cli>.ts` — consumes CLI stdout, emits unified events
- `adapters/<cli>-disk-adapter.ts` — walks the CLI's session directory, parses session files into a canonical shape

All three output into one unified contract. Adding a new CLI is three small files plus one enum entry.

**Why.** Each CLI's JSON/argv format is different and changes on its own cadence. Isolating that drift per file means a breaking CLI upgrade only touches that CLI's adapter.

**HOOP adaptation.** NEEDLE's YAML adapter covers the harness role. HOOP needs parser + disk-adapter per CLI, both consuming the same unified event union defined once in `hoop-shared`.

## Unified runtime event union

```
session.started    — CLI reports its session id (bind event)
turn.started       — a new assistant turn begins
text.delta         — streaming token
tool.use           — tool invocation with args (and later result)
progress           — CLI-emitted status marker (thinking, indexing, etc.)
stderr             — captured stderr line
error              — terminal error
out_of_tokens      — rate/budget exhausted
turn.complete      — turn finished (with reason: success|error|killed|...)
```

**Why it matters.** Parsers diverge wildly; consumers shouldn't know. Every UI component, analytics reducer, and event tap speaks one language.

**HOOP adaptation.** Adopt exactly this union. Add one NEEDLE-specific wrapper event `bead.transition { bead_id, from, to, worker, outcome }` that lifts the CLI event stream into bead-queue semantics without polluting the base union.

## Storage: JSONL files + in-memory maps, no SQLite

**The reference's choice.** No database. Conversations are read-only mirrors of each CLI's session file. Swarm runs are JSON files. UI state is one JSON file. Everything else is in-memory + browser localStorage.

**Why it works for them.** The CLIs already persist. The server has no durable state to store — it's a view layer.

**HOOP's choice diverges.** NEEDLE has authoritative SQLite + JSONL via `br`. HOOP doesn't need to add storage — it reads from NEEDLE's bead queue and tails the CLI session files. This is actually simpler than the reference: HOOP has one more structured data source, not fewer. Keep browser localStorage for purely-client concerns (UI preferences, drafts).

## Streaming separation (critical)

**The pattern.** Streaming token deltas never trigger structural re-renders. They're written to a separate `streamingContentAtom` / map keyed by conversation id. The UI merges streaming content with committed messages at render time. Only on `turn.complete` does the message promote into the committed list.

**Why.** Streaming at 50+ tokens/sec would re-render the entire conversation list on every token. Jotai (or Zustand, or any reactive store) doesn't care about the keyed streaming map; only the focused conversation re-renders.

**HOOP adaptation.** Same. Whether streaming is from a steering chat turn or from a live worker transcript, keep it in a separate reactive atom.

## Discovery: progressive, bounded-concurrency, with a poller

**The pattern.** Startup: two-phase file discovery (stat all, then parse in batches of 100 with concurrency 16). Progressive streaming of parsed batches to the client means first results appear in <1s even when there are 10k+ sessions on disk. Background poller runs every 5s to detect external edits; active sessions are skipped to avoid self-triggering.

**Why.** With cross-CLI aggregation, session counts grow linearly with user usage. Naive eager-load blocks the UI for seconds. Progressive + bounded-concurrency is table stakes.

**HOOP adaptation.** Same. Bead queue itself is small enough to load eagerly, but session transcripts are not. Apply the pattern to transcript loading specifically.

## Audit on boot

On startup, resolve every configured binary and report availability. If anything is missing, block the UI with an overlay that tells the user exactly what to install.

**Why.** 90% of "doesn't work" reports are missing CLIs. Surface it once, at boot, with the exact command the user needs.

**HOOP adaptation.** Broaden: audit `br`, tmux server, `.beads/` accessibility, bead schema version, NEEDLE binary, each configured adapter binary. One overlay, one list of actionable fixes.

## Conventions worth stealing wholesale

- **No `X_updated` event types.** `X_created` does double duty for create and mutate. Fewer types, fewer bugs.
- **Reject = error + rebroadcast.** Single rendering path.
- **Provider triad per CLI.** Harness + parser + disk-adapter, all three or none.
- **One unified event union.** Everything downstream speaks it.
- **Streaming in a separate map.** Never re-render the world on a token delta.
- **Binary audit at boot.** Fail visible, not at first use.
- **Bounded-concurrency progressive discovery.** Never eager-load all session transcripts.

## Patterns to explicitly reject

- **Storing run state as scattered JSON files.** NEEDLE's bead queue replaces this. Don't introduce `runs/<id>/cycles/*.json` — that's a workaround for not having an authoritative queue.
- **Fork emulation via cp+resume.** The dependency graph expresses fan-out; no need to forge parallel sessions.
- **Hardcoded provider pricing.** The reference hardcodes Claude/Codex prices inline. Put pricing in a config file that can be updated without a release.
- **Server-persisted UI state for shared preferences.** User-specific UI state is fine server-side (as one JSON file); anything collaborative belongs in the bead queue or not at all.
