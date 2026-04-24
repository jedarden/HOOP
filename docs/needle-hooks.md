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
{"ts":"2026-04-21T18:52:01Z","worker":"alpha","bead":"bd-abc123","event":"complete","outcome":"success","duration_ms":287104}
```

**NEEDLE change:** one helper `emit_event(worker, bead, event, **fields)` at each
state transition (claim/dispatch/complete/fail/release/timeout/crash).

## Hook 3: Worker Heartbeat

Each worker appends a heartbeat line every 10s to `.beads/heartbeats.jsonl`.

```jsonl
{"ts":"...","worker":"alpha","state":"executing","bead":"bd-abc123","pid":12345}
```

**NEEDLE change:** one background thread per worker.

## Hook 4: Stitch Label Inheritance

When a NEEDLE worker creates follow-up beads (via `br create`), any `stitch:*`
labels from the claimed bead must be copied to the new bead. Without this,
retries and cascading sub-beads lose Stitch lineage.

### HOOP Side

HOOP's REST API supports `parent_bead_id` in the `POST /api/p/{project}/beads`
request. When provided, HOOP reads the parent bead's labels via `br get`,
extracts all `stitch:*` labels using `extract_stitch_labels()`, and appends
them to the new bead (deduplicating against any already-present labels).

```json
{
  "title": "Follow-up: fix edge case",
  "parent_bead_id": "bd-parent123",
  "issue_type": "task"
}
```

The `extract_stitch_labels()` function in `br_verbs.rs` filters a label list
to return only labels matching the `stitch:*` prefix. It handles single labels,
multiple labels, empty lists, and avoids false positives on non-`stitch:` prefixes.

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

- `test_extract_stitch_labels_single` — one stitch label among others
- `test_extract_stitch_labels_multiple` — multiple stitch labels all extracted
- `test_extract_stitch_labels_none` — non-stitch labels produce empty result
- `test_extract_stitch_labels_empty` — empty input produces empty result
- `test_extract_stitch_labels_no_false_positives` — similar prefixes not matched
- `test_stitch_label_inheritance_single_label` — single stitch label inherited
- `test_stitch_label_inheritance_multiple_labels` — all stitch labels inherited
- `test_stitch_label_inheritance_no_duplicates` — duplicate labels not added twice
- `test_stitch_label_inheritance_no_stitch_labels` — non-stitch parent leaves labels unchanged

## Plan Reference

- §1.6: NEEDLE tag format and dispatch hooks
- §6 Phase 2 deliverable 14: stitch label propagation on worker create
