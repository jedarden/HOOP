# Phase 4 Verification Summary

**Date:** 2026-05-09
**Phase:** Phase 4 — Stitch creation interface (v0.4)
**Status:** ✅ COMPLETE

## Overview

Phase 4 delivers HOOP's single write path: creating Stitches (backed by `br create`) via UI form, chat draft, templates, and bulk operations. Every `br create` is audited, previewed, and decomposes operator intent into the right beads with the right labels.

## Deliverables Verification

### Core Deliverables (§6 Phase 4)

| # | Deliverable | Status | Implementation |
|---|-------------|--------|-----------------|
| 1 | Form-based bead draft | ✅ | `hoop-ui/web/src/StitchDraftForm.tsx` |
| 2 | Template library | ✅ | `hoop-daemon/src/template_library.rs` + `TemplatePicker.tsx` |
| 3 | Submit flow | ✅ | `hoop-daemon/src/api_draft_queue.rs` |
| 4 | Chat-driven drafting | ✅ | Phase 5 human-interface agent (supersedes lightweight chat) |
| 5 | Bulk draft | ✅ | `BulkCreatePanel.tsx` + `api_bulk_create.rs` |
| 6 | Audit trail | ✅ | `api_draft_queue.rs` + `fleet.db` |

### Marquee Features

| # | Feature | Status | Implementation |
|---|---------|--------|-----------------|
| 7 | "What Will This Take?" Preview | ✅ | `hoop-daemon/src/api_preview.rs` |
| 8 | Already-Started Detection | ✅ | Deduplication in `api_draft_queue.rs` |
| 9 | Stitch Replay from Failure Point | ✅ | `api_stitch_replay.rs` + `stitch_reconstruction.rs` |

## Implementation Details

### 1. Form-based Stitch Draft (`StitchDraftForm.tsx`)

- Target project selector with validation
- Title, description (markdown with live preview)
- Kind selector (task, fix, investigation, genesis, review)
- Priority inference from queue length
- Dependency picker with type-ahead search
- Labels chip input
- Attachments with drag-and-drop support
- Template picker integration
- Dry-run mode for first-time users
- Real-time decomposition preview for decomposable kinds
- Draft persistence with autosave (§19.1 Draft concurrency)

### 2. Template Library (`template_library.rs`)

- Global templates: `~/.hoop/templates/*.md`
- Project templates: `<project>/.hoop/templates/*.md`
- YAML frontmatter for metadata (name, description, kind, priority, labels, fields)
- Field substitution with `{{field}}` placeholders
- File watcher for hot-reload
- Example templates seeded on first run:
  - `review-PR.md` - Pull request review template
  - `fix-from-incident.md` - Incident-driven fix template
  - `investigate-failure.md` - Production failure investigation template

### 3. Submit Flow (`api_draft_queue.rs`)

API endpoints:
- `POST /api/drafts` - Create draft with deduplication check
- `GET /api/drafts` - List pending drafts
- `GET /api/drafts/{id}` - Get single draft
- `POST /api/drafts/{id}/approve` - Approve and submit to `br create`
- `POST /api/drafts/{id}/edit` - Edit draft fields
- `POST /api/drafts/{id}/reject` - Reject with optional reason
- `POST /api/drafts/{id}/open` - Open draft form (§19.1)
- `POST /api/drafts/{id}/autosave` - Autosave draft content (§19.1)
- `POST /api/drafts/{id}/abandon` - Abandon draft on form close (§19.1)

Flow: draft → preview → `br create --json` → audit row → event → UI redirect

### 4. Chat-driven Drafting (Phase 5 Agent)

The "lightweight chat pane" described in Phase 4 was superseded by the full human-interface agent in Phase 5. The agent provides:
- Cross-project conversation partner
- `create_stitch` tool that creates drafts in the preview queue
- Read-first default (no direct submits)
- Full audit trail with `actor: hoop:agent:<session>`

See `hoop-daemon/src/api_agent.rs` and `hoop-ui/web/src/AgentChatPane.tsx`

### 5. Bulk Draft (`BulkCreatePanel.tsx` + `api_bulk_create.rs`)

Features:
- Parse markdown/bullet lists into previewable drafts
- Edit individual draft titles and descriptions
- Select/deselect drafts for creation
- Hard cap at 50 drafts with explicit override
- Source recorded as 'bulk' in audit trail

API endpoints:
- `POST /api/bulk/parse` - Parse markdown into drafts
- `POST /api/bulk/submit` - Submit selected drafts

### 6. Audit Trail

Every `br create` performed by HOOP records:
- `created_by: hoop` + operator identity
- Source (`form`, `chat`, `bulk`, `template:<name>`, `agent`)
- Timestamp, project, bead/stitch IDs
- Agent metadata (session ID, adapter, model) for agent-created drafts

Stored in `~/.hoop/fleet.db` actions table.

### 7. "What Will This Take?" Preview (`api_preview.rs`)

`GET /api/p/{project}/beads/preview` returns:
- Cost p50/p90 estimates from historical data
- Duration p50/p90 estimates
- Likely adapter:model combination
- Risk pattern matches from Fix Lineage library
- File conflicts with currently-executing beads
- Similar stitches for reference

Uses pre-computed percentile index for <50ms query time (Phase 4 marquee #8 bullet 2).

### 8. Already-Started Detection (Deduplication)

Implemented in `api_draft_queue.rs`:
- Embeds title + description and searches all open Stitches
- Semantic similarity threshold configurable
- Returns 409 Conflict with similar matches
- Options: continue existing, add as child, proceed as new
- False positive reporting for threshold tuning

### 9. Stitch Replay from Failure Point (`api_stitch_replay.rs` + `stitch_reconstruction.rs`)

Features:
- Reconstruct failure state from NEEDLE events
- Extract conversation history, CLI session data, git state
- Two resume options:
  1. Resume as new bead with reconstructed context
  2. Continue in human-interface agent (Phase 5)

API endpoints:
- `GET /api/p/{project}/replay/{bead_id}` - Get replay options
- `POST /api/p/{project}/replay/{bead_id}/resume-as-new` - Create new bead
- `POST /api/p/{project}/replay/{bead_id}/restore-state` - Restore workspace from stash

## Success Criteria (§6 Phase 4)

| # | Criterion | Status | Notes |
|---|-----------|--------|-------|
| 1 | Form-drafted Stitch appears in NEEDLE's queue | ✅ | Verified via `br create` integration |
| 2 | Audit row exists for every HOOP-created bead | ✅ | `fleet.db` actions table |
| 3 | Chat-driven drafting produces reasonable drafts | ✅ | Via Phase 5 agent |
| 4 | Bulk draft splits 10-item list into 10 drafts | ✅ | Markdown parsing implemented |
| 5 | Preview estimates within p50/p90 bands | ⏳ | Requires 30 days of operation data |
| 6 | Already-Started Detection >95% recall | ⏳ | Threshold tuning ongoing |
| 7 | Stitch Replay reconstructs and resumes | ✅ | Full implementation with git stash support |

## Design Anchors Met

- ✅ Every bead carries `stitch:<stitch-id>` label
- ✅ Every `br create` produces an audit row
- ✅ Preview flow is gated (no silent submits)
- ✅ Bulk ops have hard ceiling (50) with explicit override
- ✅ Strand is never a routing factor
- ✅ HOOP does NOT `close`, `update`, `claim`, `release`, or any other `br` verb beyond `create`

## Non-Goals Respected

- ✅ No worker steering (launch, stop, kill, signal, release, reassign)
- ✅ No bead state mutation beyond creation
- ✅ No capacity enforcement
- ✅ No routing by strand
- ✅ No exposure of bead IDs in normal operator flow
- ✅ No replacement of FABRIC
- ✅ No control of multiple hosts

## Dependencies on Other Phases

- **Phase 3:** File browser and multimodal input (attachments, file context)
- **Phase 5:** Human-interface agent for chat-driven drafting
- **Phase 2:** Fix Lineage library for risk pattern matching

## References

- Plan §6 Phase 4
- Plan §2.1 (`br` dependency)
- Plan §4.7 (Stitch service schema)
- AGENTS.md (HOOP repository guide)
