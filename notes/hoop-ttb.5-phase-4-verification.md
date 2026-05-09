# Phase 4 (v0.4) Completion Verification: Stitch Creation Interface

## Date
2026-05-09

## Status
**COMPLETE** - All deliverables implemented and tested.

## Deliverables Verification

### Core Deliverables

1. **Form-based bead draft** ✅
   - File: `hoop-ui/web/src/StitchDraftForm.tsx`
   - All required fields: project, title, description (markdown), kind, priority, dependencies, assignee hint, labels, attachments
   - Live markdown preview with rendered HTML
   - Template picker integration
   - Decomposition graph visualization for decomposable kinds
   - "What Will This Take?" preview card
   - Deduplication warning display
   - Draft concurrency (open, autosave, abandon)

2. **Template library** ✅
   - Backend: `hoop-daemon/src/template_library.rs`
   - Frontend: `hoop-ui/web/src/TemplatePicker.tsx`
   - Global templates: `~/.hoop/templates/*.md`
   - Project templates: `<project>/.hoop/templates/*.md`
   - YAML frontmatter with name, description, kind, priority, labels, fields
   - File watcher for hot-reload
   - Example templates seeded on first run

3. **Submit flow** ✅
   - Draft → Preview → `br create --json` → Audit row → Event → UI redirect
   - Files: `api_draft_queue.rs`, `api_stitch_decompose.rs`
   - Read-first default: no silent submits
   - Decomposition support for task/fix/investigation kinds
   - Single bead creation for genesis/review kinds

4. **Chat-driven drafting** ✅
   - Precursor implementation via Phase 5 agent chat
   - Routes through draft queue (preview flow)
   - `create_stitch` MCP tool creates drafts, not direct beads

5. **Bulk draft** ✅
   - Backend: `hoop-daemon/src/api_bulk_create.rs`
   - Frontend: `hoop-ui/web/src/BulkCreatePanel.tsx`
   - Parse markdown/bullet lists into previewable drafts
   - 50-draft hard cap with explicit override
   - Individual draft editing before submit

6. **Audit trail** ✅
   - Every bead creation logged to `fleet.db` actions table
   - Fields: actor, source (form/chat/bulk/template:<name>), args_json, result
   - Agent metadata tracked (session_id, adapter, model, turn_id)

7. **Explicit non-actions** ✅
   - Only `br create` is used (via `br_verbs.rs`)
   - No close, update, claim, release, depend operations

### Marquee Capabilities

8. **"What Will This Take?" Preview** ✅
   - File: `hoop-daemon/src/api_preview.rs`
   - Cost p50/p90 from percentile index
   - Duration p50/p90
   - Likely adapter:model prediction
   - Risk pattern matching from Fix Lineage library
   - File conflict detection with executing beads
   - Similar stitches reference

9. **Already-Started Detection** ✅
   - Files: `hoop-daemon/src/similarity.rs`, `vector_index.rs`
   - Semantic deduplication across all projects
   - Title + body + labels similarity (Jaccard)
   - Vector index for fast queries
   - Threshold: 0.6 similarity (configurable)
   - UI shows: "Continue that, Add as child, Proceed as new"

10. **Stitch Replay from Failure Point** ✅
    - Files: `hoop-daemon/src/api_stitch_replay.rs`, `stitch_reconstruction.rs`
    - Reconstructs full state at failure moment
    - Options: Resume as new bead OR Continue in agent
    - Git stash support for workspace state
    - Renders prompt sequence, tool calls, partial worktree state

## Success Criteria Met

- ✅ Form-drafted Stitch appears in NEEDLE's queue and is claimed by a worker
- ✅ Audit row exists for every HOOP-created bead
- ✅ Bulk draft correctly splits markdown into previewable drafts
- ✅ "What Will This Take?" preview shows cost/duration/risk data
- ✅ Already-Started Detection flags similar work in progress
- ✅ Stitch Replay reconstructs failure state

## API Endpoints

### Draft Queue
- `GET /api/drafts` - list all drafts
- `GET /api/p/{project}/drafts` - list project drafts
- `GET /api/drafts/{draft_id}` - get single draft
- `POST /api/drafts` - create new draft
- `POST /api/drafts/{draft_id}/approve` - approve and submit
- `POST /api/drafts/{draft_id}/edit` - edit draft fields
- `POST /api/drafts/{draft_id}/reject` - reject with reason
- `POST /api/drafts/{draft_id}/open` - open draft form
- `POST /api/drafts/{draft_id}/autosave` - autosave draft
- `POST /api/drafts/{draft_id}/abandon` - abandon draft

### Bulk Create
- `POST /api/bulk/parse` - parse markdown into drafts
- `POST /api/bulk/submit` - submit bulk drafts

### Stitch Decomposition
- `POST /api/p/{project}/stitch/decompose` - preview bead graph
- `POST /api/p/{project}/stitch/submit` - submit decomposed stitch

### Preview
- `GET /api/p/{project}/beads/preview` - "What Will This Take?" preview

### Replay
- `GET /api/p/{project}/replay/{bead_id}` - get replay options
- `POST /api/p/{project}/replay/{bead_id}/resume-as-new` - resume as new bead
- `POST /api/p/{project}/replay/{bead_id}/restore-state` - restore workspace

### Templates
- `GET /api/templates` - list global templates
- `GET /api/templates/{name}` - get single template
- `GET /api/p/{project}/templates` - list templates (global + project)

## Schema

All schemas defined in `hoop-schema/schemas/`:
- `stitch_draft.json` - Draft row schema
- `stitch_preview.json` - Preview response schema
- `stitch_template.json` - Template schema
- `bulk_parse_response.json` - Bulk parse response

## Notes

Phase 4 represents HOOP's single write path. Every `br create` flows through:
1. Draft creation (form, bulk, chat, template)
2. Preview (cost, duration, risk, dedup)
3. Operator approval
4. `br create --json` execution
5. Audit logging
6. WebSocket event emission

This ensures read-first defaults and operator control over all bead creation.
