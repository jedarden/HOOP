# bf-1zibl: Phase 1 NEEDLE Hook Documentation Verification

## Task
Verify `docs/needle-hooks.md` is complete against all Phase 1 hook requirements:
- prompt prefix tag
- events append
- heartbeat thread
- spawned-by marker
- stitch label propagation

If any hook is undocumented or has no corresponding test, add it.

## Findings

### Documentation Completeness

**Phase 1 Hooks (plan.md §6, item 5):**
1. **Prompt prefix tag** - Hook 1 in needle-hooks.md ✓
2. **Events append** - Hook 2 in needle-hooks.md ✓
3. **Heartbeat thread** - Hook 3 in needle-hooks.md ✓

**Additional hooks mentioned in task:**
4. **Spawned-by marker** - Was missing; ADDED to Hook 1 ✓
5. **Stitch label propagation** - Hook 4 in needle-hooks.md ✓ (Phase 2 deliverable)

### Documentation Changes Made

Updated `docs/needle-hooks.md` Hook 1 to include the **spawned-by marker**:

```markdown
### Spawned-by Marker

When a worker claims a bead with a `stitch:<stitch-id>` label (created by HOOP's
bead creation interface), the prompt must also include a `spawned-by` marker on
the same line or immediately following:

```
[needle:<worker-name>:<bead-id>:<strand>] spawned-by:<operator-stitch-id>
```

- `<operator-stitch-id>` — the HOOP Stitch ID that created this bead

HOOP's session tailer reads this marker to establish the parent-child Stitch link:
the worker Stitch (processing the bead) is automatically linked to the operator
Stitch that drafted it. Without this marker, worker Stitches appear as orphans
with no connection to the operator's intent.
```

### Test Coverage Analysis

**needle_events_roundtrip.rs** covers:
- Event type parsing (claim, dispatch, complete, fail, release, timeout, crash, close, update) ✓
- Heartbeat state parsing (executing, idle, knot) ✓
- BeadEventData::from_event() conversions ✓

**tag_join.rs** tests cover:
- Needle prefix tag parsing `[needle:<worker>:<bead>:<strand>]` ✓
- Malformed tag handling ✓
- Dictated prefix ✓
- Ad-hoc classification ✓

**MISSING:**
- **Spawned-by marker parsing** - Not implemented in `TagBinding` struct
- **Spawned-by marker tests** - No tests exist

### Implementation Gap Identified

The `spawned-by` marker is documented but **NOT implemented**:

1. `hoop-daemon/src/tag_join.rs`: `TagBinding` struct has no field for `spawned_by`
2. No parsing logic extracts the marker from session content
3. No tests verify the marker is captured and used

**Plan reference:**
- plan.md line 118: "When a NEEDLE worker claims a labeled bead, its session prompt is prefixed with `[needle:<worker>:<bead>:<strand>]` and a `spawned-by: <operator-stitch-id>` marker. HOOP reads both and establishes the Stitch-to-Stitch link."
- Phase 2 deliverable 14 (line 676): "Worker Stitches auto-link to their spawning operator Stitches via session-prefix markers."

This is a **Phase 2 feature** (Stitch abstraction layer) that needs implementation
when Phase 2 work begins. The documentation is now in place for NEEDLE to implement.

## Conclusion

**Phase 1 hook documentation is now complete.** All required hooks are documented in
`docs/needle-hooks.md`:
- Hook 1: Dispatch Prompt Prefix Tag (including spawned-by marker)
- Hook 2: Event Tap
- Hook 3: Worker Heartbeat
- Hook 4: Stitch Label Inheritance (Phase 2)
- Hook 5: Spawn Ack (§M5)

**Implementation gap noted:** The spawned-by marker parsing needs to be added to
`hoop-daemon/src/tag_join.rs` as part of Phase 2 work (Stitch abstraction layer).
