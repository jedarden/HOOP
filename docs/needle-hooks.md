# NEEDLE Hooks Reference

Hooks are small touchpoints between HOOP and NEEDLE. HOOP is intentionally
additive — NEEDLE runs standalone without HOOP. The coupling is these hooks.

## Hook 1: Dispatch Prompt Prefix Tag

Every bead dispatch prepends the first user-message line with:

```
[needle:<worker-name>:<bead-id>:<strand>]
```

- `<worker-name>` — NATO id (`alpha`, `bravo`, ...)
- `<bead-id>` — `br` bead id (`bd-abc123`)
- `<strand>` — one of `pluck | explore | mend | weave | unravel | pulse | knot`

HOOP's disk adapter extracts this tag from CLI session files to join transcripts
back to beads without a second storage layer.

**NEEDLE change:** one line in the prompt builder. Tag is opaque to the CLI.

## Hook 2: Event Tap

Each NEEDLE worker appends one JSONL line per bead transition to `.beads/events.jsonl`.
HOOP tails this file and broadcasts events on the WebSocket.

```jsonl
{"ts":"2026-04-21T18:42:10Z","worker":"alpha","bead":"bd-abc123","strand":"pluck","event":"claim"}
{"ts":"2026-04-21T18:47:33Z","worker":"alpha","bead":"bd-abc123","event":"dispatch","adapter":"claude","model":"claude-opus-4-6"}
{"ts":"2026-04-21T18:52:01Z","worker":"alpha","bead":"bd-abc123","event":"complete","outcome":"success","duration_ms":287104,"exit_code":0}
{"ts":"2026-04-21T18:57:00Z","worker":"bravo","bead":"bd-def456","event":"fail","error":"context limit exceeded","duration_ms":90000}
{"ts":"2026-04-21T19:00:00Z","worker":"bravo","bead":"bd-def456","event":"release"}
{"ts":"2026-04-21T19:10:00Z","worker":"charlie","bead":"bd-ghi789","event":"timeout"}
{"ts":"2026-04-21T19:17:00Z","worker":"delta","bead":"bd-jkl012","event":"crash","exit_code":139}
{"ts":"2026-04-21T19:25:01Z","worker":"alpha","bead":"bd-mno345","event":"close"}
{"ts":"2026-04-21T19:26:00Z","worker":"alpha","bead":"bd-pqr678","event":"update"}
```

Fields by event type:

| event | required | optional |
|---|---|---|
| `claim` | — | `strand` |
| `dispatch` | — | `adapter`, `model` |
| `complete` | — | `outcome`, `duration_ms`, `exit_code` |
| `fail` | — | `error`, `duration_ms` |
| `release` | — | — |
| `timeout` | — | — |
| `crash` | — | `exit_code` |
| `close` | — | — |
| `update` | — | — |

All events carry `ts` (RFC 3339 UTC), `worker` (NATO id), `bead` (`bd-*`), and `event`.

**NEEDLE change:** one helper `emit_event(worker, bead, event, **fields)` at each
state transition (claim/dispatch/complete/fail/release/timeout/crash/close/update).

## Hook 3: Worker Heartbeat

Each worker appends a heartbeat line every 10s to `.beads/heartbeats.jsonl`.

Three states — one line per tick:

```jsonl
{"ts":"2026-04-21T18:42:10Z","worker":"alpha","state":"executing","bead":"bd-abc123","pid":12345,"adapter":"claude"}
{"ts":"2026-04-21T18:52:05Z","worker":"alpha","state":"idle","last_strand":"pluck"}
{"ts":"2026-04-21T18:57:05Z","worker":"bravo","state":"knot","reason":"strands exhausted"}
```

Fields by state:

| state | required | optional |
|---|---|---|
| `executing` | `bead`, `pid`, `adapter` | — |
| `idle` | — | `last_strand` |
| `knot` | `reason` | — |

All three states carry `ts` (RFC 3339 UTC) and `worker` (NATO id).

HOOP liveness rules: `Live` = PID alive + heartbeat ≤ 20s old; `Hung` = PID alive but stale; `Dead` = PID gone.

**NEEDLE change:** one background thread per worker that emits every 10s.

## Hook 4: Stitch Label Inheritance

When a NEEDLE worker creates follow-up beads (via `br create`), any `stitch:*`
labels from the claimed bead must be copied to the new bead. Without this,
retries and cascading sub-beads lose Stitch lineage.

### HOOP Side

HOOP provides two paths for Hook 4 label propagation:

#### Daemon REST API

The `POST /api/p/{project}/beads` endpoint supports `parent_bead_id` in the
request body. When provided, HOOP reads the parent bead's labels via `br get`,
extracts all `stitch:*` labels using `extract_stitch_labels()`, and appends
them to the new bead (deduplicating against any already-present labels).

```json
{
  "title": "Follow-up: fix edge case",
  "parent_bead_id": "bd-parent123",
  "issue_type": "task"
}
```

#### MCP `create_bead` Tool

The MCP server exposes a `create_bead` tool that workers can use to create
follow-up beads. When `parent_bead_id` is provided, the tool reads the parent
bead's labels via `br get --json`, propagates `stitch:*` labels using
`propagate_stitch_labels()`, and passes the combined labels to `br create`.

```json
{
  "project": "my-project",
  "title": "Follow-up: retry edge case",
  "parent_bead_id": "bd-parent123",
  "issue_type": "task"
}
```

The `propagate_stitch_labels()` function in `hoop-mcp/src/br_verbs.rs` encapsulates
the one-line label copy: extract `stitch:*` labels from parent, append deduplicated.

### NEEDLE Side

When a worker claims a bead with `stitch:*` labels, the NEEDLE dispatch should
copy those labels to any `br create` invocation the worker makes in its session.
This is a one-line copy in the worker's `br create` wrapper:

```python
# In the worker's br create wrapper:
parent_labels = current_bead.get("labels", [])
stitch_labels = [l for l in parent_labels if l.startswith("stitch:")]
# Append stitch_labels to br create --labels
```

### Why

Stitches decompose work into bead graphs. Each bead in a Stitch carries a
`stitch:<id>` label so HOOP can group them. When a worker creates follow-up
beads (e.g., a retry bead, a cascading sub-task), losing the label breaks the
Stitch lineage — the new bead becomes orphaned from its Stitch context.

### Test Coverage

Both `hoop-daemon` and `hoop-mcp` carry identical Hook 4 tests:

- `test_extract_stitch_labels_single` — one stitch label among others
- `test_extract_stitch_labels_multiple` — multiple stitch labels all extracted
- `test_extract_stitch_labels_none` — non-stitch labels produce empty result
- `test_extract_stitch_labels_empty` — empty input produces empty result
- `test_extract_stitch_labels_no_false_positives` — similar prefixes not matched
- `test_stitch_label_inheritance_single_label` — single stitch label inherited
- `test_stitch_label_inheritance_multiple_labels` — all stitch labels inherited

MCP-specific Hook 4 tests (`hoop-mcp`):

- `test_propagate_stitch_labels_worker_followup_single` — worker-created follow-up bead inherits parent's stitch label
- `test_propagate_stitch_labels_multiple_stitch_labels` — multiple stitch labels all propagate

Daemon-only tests (REST API integration):

- `test_stitch_label_inheritance_no_duplicates` — duplicate labels not added twice
- `test_stitch_label_inheritance_no_stitch_labels` — non-stitch parent leaves labels unchanged

## Hook 5: Spawn Ack (§M5)

Every NEEDLE worker must write a spawn-ack file at boot to prove it started
successfully.  HOOP verifies the ack within a 10-second grace window; absence
triggers a `MissingAck` alert surfaced on the WebSocket and in metrics.

### Why

`tmux send-keys` can silently truncate payloads longer than ~255 bytes.  The
spawn command appears to succeed while the worker never actually starts.
Heartbeats alone cannot catch this: a non-started worker emits no heartbeats,
so HOOP would simply see silence — with no way to distinguish "not yet started"
from "spawn failed".  The ack provides a positive confirmation of startup.

### Ack file

- **Location:** `~/.hoop/workers/<worker-name>.ack`
- **Format:** single JSON object, one line:

```json
{"worker":"alpha","ts":"2026-04-24T10:00:00Z","pid":12345}
```

### NEEDLE boot hook

Run this at the very start of each worker's boot sequence (before the heartbeat
loop begins):

```sh
mkdir -p ~/.hoop/workers
printf '{"worker":"%s","ts":"%s","pid":%d}\n' \
    "$NEEDLE_WORKER" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$$" \
    > ~/.hoop/workers/${NEEDLE_WORKER}.ack.tmp
mv ~/.hoop/workers/${NEEDLE_WORKER}.ack.tmp \
   ~/.hoop/workers/${NEEDLE_WORKER}.ack
```

The `.tmp` + rename pattern ensures HOOP never reads a partial file.

### HOOP side

`WorkerAckMonitor` (`hoop-daemon/src/worker_ack.rs`) watches
`~/.hoop/workers/` for `.ack` files using the `notify` crate.

- **Pre-existing acks** are loaded on daemon startup (handles daemon restarts
  where workers are already running).
- **Grace window:** 10 seconds from the worker's first observed heartbeat.
- **Missing-ack alert:** if a worker heartbeats for ≥ 10 s with no ack file,
  HOOP emits a `spawn_ack_alert` WebSocket event and increments
  `hoop_worker_spawn_missing_ack_total`.
- **Alert message:** `"Worker 'X' has been heartbeating for Ns but has no
  spawn ack at ~/.hoop/workers/X.ack — boot hook may be missing (§M5)"`

### Validation

HOOP validates that the worker name in the JSON matches the filename stem.
Files that fail to parse or have a name mismatch are skipped with a `WARN` log.

### Test coverage

Unit tests live in `hoop-daemon/src/worker_ack.rs`:

- `test_is_ack_file_positive` / `_negative` — extension filtering
- `test_parse_ack_valid` — happy path parse
- `test_parse_ack_trailing_newline` — tolerates trailing newline
- `test_parse_ack_name_mismatch` — rejects mismatched worker name
- `test_parse_ack_invalid_json` — rejects malformed JSON
- `test_on_heartbeat_records_first_seen` — first heartbeat timestamp recorded
- `test_scan_existing_loads_ack` — pre-existing ack loaded on startup
- `test_get_all_acks_empty` — empty state
- `test_ack_received_clears_alert` — ack received after heartbeat updates state

## Plan Reference

- §1.6: NEEDLE tag format and dispatch hooks
- §6 Phase 2 deliverable 14: stitch label propagation on worker create
- §M5: Silent tmux-orphan channels — spawn verification via ack file
