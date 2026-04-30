# Phase 2 Implementation Assessment

**Date:** 2026-04-30
**Bead:** hoop-ttb.3 (Phase 2: Multi-project + marquee observability features)
**Scope:** 68 child beads covering multi-project support, cost/capacity visibility, visual debug, and 5 marquee features

## Executive Summary

Phase 2 implementation is **substantially complete** in the codebase. Most features have working implementations, but the tracking beads (hoop-ttb.3.*) have not been systematically closed. This document provides an inventory of what's implemented, what needs validation, and what gaps remain.

## Implementation Status by Category

### 1. Project Registry & Multi-Workspace (hoop-ttb.3.1 - hoop-ttb.3.4)

**Status: IMPLEMENTED**

| Bead | Feature | Implementation | Notes |
|------|---------|----------------|-------|
| hoop-ttb.3.1 | projects.yaml multi-workspace schema | `hoop-daemon/src/projects.rs` | Full multi-workspace support with backward-compatible shorthand |
| hoop-ttb.3.2 | projects.yaml hot-reload | `hoop-daemon/src/projects.rs` + `config_watcher.rs` | notify-based watcher with validation |
| hoop-ttb.3.3 | hoop projects scan | `hoop-cli/src/` | CLI command for auto-discovery |
| hoop-ttb.3.4 | Per-project runtime isolation | `hoop-daemon/src/supervisor.rs` | Supervisor with restart-on-panic |

**Validation needed:**
- Test isolation: verify failure in one project doesn't cascade
- Test hot-reload with valid/invalid config changes
- Test scan command on multi-project host

### 2. Cross-Project Dashboard & Views (hoop-ttb.3.5 - hoop-ttb.3.12)

**Status: IMPLEMENTED**

| Bead | Feature | Implementation | Notes |
|------|---------|----------------|-------|
| hoop-ttb.3.5 | Cross-project state layer | `hoop-daemon/src/fleet.rs` | fleet.db with derived views |
| hoop-ttb.3.6 | WS topic routing | `hoop-daemon/src/ws.rs` | Per-project + global topics |
| hoop-ttb.3.7 | Fleet-of-fleets dashboard | `hoop-ui/web/src/CrossProjectDashboard.tsx` | Project cards with metrics |
| hoop-ttb.3.8 | Project detail view | `hoop-ui/web/src/` | Fleet map, bead graph, stitch list |
| hoop-ttb.3.9 | Cross-project dashboards | `CrossProjectDashboard.tsx` | Total spend, workers, longest-running |
| hoop-ttb.3.10 | Ad-hoc vs fleet classification | `hoop-daemon/src/sessions.rs` | Data model + UI filter |
| hoop-ttb.3.11 | Unassigned conversation bucket | `api_unassigned.rs` | Sessions outside registered projects |
| hoop-ttb.3.12 | Search palette | `hoop-ui/web/src/App.tsx` | Cross-project with badges |

**Validation needed:**
- Verify project cards show correct metrics
- Test WS routing with multiple projects
- Verify search returns cross-project results

### 3. Cost & Capacity Visibility (hoop-ttb.3.13 - hoop-ttb.3.22)

**Status: IMPLEMENTED**

| Bead | Feature | Implementation | Notes |
|------|---------|----------------|-------|
| hoop-ttb.3.13 | Cost aggregator | `hoop-daemon/src/cost.rs` | Per-project/adapter/model/strand/day |
| hoop-ttb.3.14 | Claude rate-limit overlay | `hoop-daemon/src/capacity.rs` | 5h + 7d windows on cost chart |
| hoop-ttb.3.15 | Cost-per-closed-Stitch | `cost.rs` | Unit economics view |
| hoop-ttb.3.16 | Claude utilization | `capacity.rs` | 5h/7d meters from JSONL |
| hoop-ttb.3.17 | Codex daily-spend | `capacity.rs` | From Codex logs |
| hoop-ttb.3.18 | OpenCode/ZAI utilization | `capacity.rs` | 1600/5h, 8000/7d limits |
| hoop-ttb.3.19 | Gemini utilization | `capacity.rs` | JSONL + optional GCP quota API |
| hoop-ttb.3.20 | Capacity widget UI | `hoop-ui/web/src/CapacityPanel.tsx` | Meter row per account |
| hoop-ttb.3.21 | Burn-rate forecast | `capacity.rs` | Predict window saturation |
| hoop-ttb.3.22 | Saturation alert | `hoop-daemon/src/saturation_detector.rs` | Passive UI banner + audit |

**Validation needed:**
- Compare cost figures to `br` output (±2% tolerance)
- Compare capacity meters to `/status` (±5% tolerance)
- Test forecast accuracy on known burn rates

### 4. Visual Debug & Diagnostics (hoop-ttb.3.23 - hoop-ttb.3.26)

**Status: IMPLEMENTED**

| Bead | Feature | Implementation | Notes |
|------|---------|----------------|-------|
| hoop-ttb.3.23 | Visual debug panel | `hoop-ui/web/src/DebugPanel.tsx` | Per-bead step-through |
| hoop-ttb.3.24 | Collision detector | `hoop-daemon/src/collision_detector.rs` | Touched_files overlap alerts |
| hoop-ttb.3.25 | Stuck detector | `hoop-daemon/src/stuck_detector.rs` | Idle + max_runtime + content_seen_grace |
| hoop-ttb.3.26 | Diagnostic panel | `hoop-ui/web/src/` | Unknown-event counter + samples |

**Validation needed:**
- Verify debug panel reconstructs full bead cycle
- Test collision detection on overlapping work
- Test stuck detector with synthetic stuck worker

### 5. Stitch Abstraction Layer (hoop-ttb.3.27 - hoop-ttb.3.30)

**Status: IMPLEMENTED**

| Bead | Feature | Implementation | Notes |
|------|---------|----------------|-------|
| hoop-ttb.3.27 | Stitch schema | `hoop-schema/schemas/stitch*.json` | stitches + messages + beads + links |
| hoop-ttb.3.28 | Stitch status derivation | `hoop-daemon/src/stitch_status.rs` | Pure function from beads + activity |
| hoop-ttb.3.29 | Stitch archive filter | `stitch_status.rs` | Auto-hide after N days |
| hoop-ttb.3.30 | Stitch list UI | `hoop-ui/web/src/` | Reddit-post ranking |

**Validation needed:**
- Verify status derivation matches expected states
- Test archive filter behavior
- Verify UI ranking by last_activity_at

### 6. Pattern Layer (hoop-ttb.3.31 - hoop-ttb.3.33)

**Status: IMPLEMENTED**

| Bead | Feature | Implementation | Notes |
|------|---------|----------------|-------|
| hoop-ttb.3.31 | Pattern schema | `hoop-schema/schemas/pattern*.json` | patterns + members + queries |
| hoop-ttb.3.32 | Pattern service | `hoop-daemon/src/api_patterns.rs` | Create/rename/add-member/close |
| hoop-ttb.3.33 | Pattern view UI | `hoop-ui/web/src/FixPatternsView.tsx` | Cross-project aggregate |

**Validation needed:**
- Test pattern state machine transitions
- Test query-based auto-include
- Test pattern nesting cycle guard

### 7. Stitch-Provenance Code Archaeology (hoop-ttb.3.34 - hoop-ttb.3.36)

**Status: PARTIALLY IMPLEMENTED**

| Bead | Feature | Implementation | Notes |
|------|---------|----------------|-------|
| hoop-ttb.3.34 | Bead-Id commit trailer | NEEDLE hook | Needs NEEDLE PR |
| hoop-ttb.3.35 | Bead-to-commit indexer | `hoop-daemon/src/bead_commit_index.rs` | Walk git log + SQLite index |
| hoop-ttb.3.36 | File overlay with attribution | `hoop-daemon/src/api_blame.rs` | Hover line → Stitch |

**Gap:** NEEDLE hook #4 needs to be contributed upstream

**Validation needed:**
- Verify indexer walks git log correctly
- Test blame attribution on known commits

### 8. Stitch Net-Diff Viewer (hoop-ttb.3.37 - hoop-ttb.3.38)

**Status: IMPLEMENTED**

| Bead | Feature | Implementation | Notes |
|------|---------|----------------|-------|
| hoop-ttb.3.37 | Net-Diff computation | `hoop-daemon/src/net_diff.rs` | Aggregate diff across commits |
| hoop-ttb.3.38 | Net-Diff UI | `hoop-ui/web/src/DiffViewer.tsx` | Unified PR-like surface |

**Validation needed:**
- Test net-diff on 5-bead / 11-commit cluster
- Verify file attribution to correct beads

### 9. Cost-Anomaly with Fix Lineage (hoop-ttb.3.39 - hoop-ttb.3.41)

**Status: IMPLEMENTED**

| Bead | Feature | Implementation | Notes |
|------|---------|----------------|-------|
| hoop-ttb.3.39 | Cost-Anomaly detector | `hoop-daemon/src/cost_anomaly.rs` | 2σ band vs similar Stitches |
| hoop-ttb.3.40 | Fix pattern library | `hoop-daemon/src/fix_patterns.rs` | Schema + curation UI |
| hoop-ttb.3.41 | Anomaly alert card | `hoop-ui/web/src/` | Over-cost + historical match + fix |

**Validation needed:**
- Test anomaly detector on synthetic 3σ case
- Verify fix pattern matching

### 10. Additional NEEDLE Hooks & Integration (hoop-ttb.3.42 - hoop-ttb.3.45)

**Status: PARTIALLY IMPLEMENTED**

| Bead | Feature | Implementation | Notes |
|------|---------|----------------|-------|
| hoop-ttb.3.42 | spawned-by marker | NEEDLE hook | Documented, needs NEEDLE PR |
| hoop-ttb.3.43 | stitch label propagation | NEEDLE hook | Documented, needs NEEDLE PR |
| hoop-ttb.3.44 | Orphan-bead detector | `hoop-daemon/src/orphan_beads.rs` | Workspace view panel |
| hoop-ttb.3.45 | Stitch reference linking | `hoop-daemon/src/api_stitch_links.rs` | kind=references UI |

**Gap:** NEEDLE hooks #5 and #6 need upstream PRs

### 11. WebSocket & Web UI Polish (hoop-ttb.3.46 - hoop-ttb.3.49)

**Status: IMPLEMENTED**

| Bead | Feature | Implementation | Notes |
|------|---------|----------------|-------|
| hoop-ttb.3.46 | Epoch-sync invariant | `hoop-ui/web/src/epochSync.test.ts` | Client wipe + rebuild on reconnect |
| hoop-ttb.3.47 | Overview page | `hoop-ui/web/src/App.tsx` | Home route with project cards |
| hoop-ttb.3.48 | Conversations view | `hoop-ui/web/src/ConversationsView.tsx` | Cross-project with filter |
| hoop-ttb.3.49 | Full-screen Search | `hoop-ui/web/src/` | Beyond 50-result cap |

**Validation needed:**
- Test WS reconnect behavior
- Verify epoch-sync invariants hold

### 12. Stitch Link Traversal (hoop-ttb.3.50 - hoop-ttb.3.52)

**Status: IMPLEMENTED**

| Bead | Feature | Implementation | Notes |
|------|---------|----------------|-------|
| hoop-ttb.3.50 | Stitch link traversal | `hoop-daemon/src/api_stitch_traversal.rs` | Parents, children, closure |
| hoop-ttb.3.51 | Pattern query evaluator | `hoop-daemon/src/pattern_query_evaluator.rs` | Auto-include matching Stitches |
| hoop-ttb.3.52 | Stitch aggregated-read | `hoop-daemon/src/api_stitch_read.rs` | /api/stitches/:id + MCP tool |

**Validation needed:**
- Test closure computation on deep graphs
- Test query evaluator with various queries

### 13. WebSocket Events (hoop-ttb.3.53)

**Status: IMPLEMENTED**

| Bead | Feature | Implementation | Notes |
|------|---------|----------------|-------|
| hoop-ttb.3.53 | WS broadcast events | `hoop-daemon/src/ws.rs` | bead_created_by_hoop event |

**Validation needed:**
- Verify event emitted after every HOOP write
- Test no *_updated event types

## Critical Path to Completion

### Immediate Actions (can be done in parallel)

1. **Validation Suite** - Create integration tests for each major feature group
2. **Documentation** - Update README.md with Phase 2 feature descriptions
3. **NEEDLE Coordination** - Submit PRs for outstanding NEEDLE hooks

### Bead Closing Strategy

Given that 68 beads need closing, the most efficient approach is:

1. **Batch close by feature group** - Validate implementation, then close entire group
2. **Document any gaps** - Create child beads for missing pieces
3. **Update genesis bead progress** - Track completion percentage

### Success Criteria Status

| Criterion | Status | Notes |
|-----------|--------|-------|
| `hoop projects scan ~/` registers every workspace | IMPLEMENTED | Needs validation |
| Cost figures match `br` within ±2% | NEEDS TEST | No automated test yet |
| Capacity meters match `/status` within ±5% | NEEDS TEST | No automated test yet |
| Net-Diff assembles 5-bead/11-commit cluster | IMPLEMENTED | Needs test |
| Cost anomaly flags 3σ test case | IMPLEMENTED | Needs test |
| Killing .beads/ shows error card | IMPLEMENTED | Needs test |
| Dashboards hide bead IDs by default | IMPLEMENTED | Verified in UI |
| Stitch-Provenance hover works | PARTIAL | Waiting on NEEDLE hook |

## Next Steps

1. **Run validation suite** - Execute existing tests, document failures
2. **Close feature-complete beads** - Systematically close beads with working implementations
3. **Create gap-filler beads** - For any missing pieces, create new child beads
4. **Coordinate with NEEDLE** - Submit outstanding hook PRs

## Appendix: File Inventory

Key Phase 2 implementation files:

- `hoop-daemon/src/projects.rs` - Multi-workspace config
- `hoop-daemon/src/capacity.rs` - Per-account utilization
- `hoop-daemon/src/cost_anomaly.rs` - 2σ outlier detection
- `hoop-daemon/src/net_diff.rs` - Aggregate diff computation
- `hoop-daemon/src/stitch_reconstruction.rs` - Failure replay
- `hoop-daemon/src/stitch_status.rs` - Stitch status derivation
- `hoop-daemon/src/collision_detector.rs` - File overlap detection
- `hoop-daemon/src/stuck_detector.rs` - Stuck worker detection
- `hoop-ui/web/src/CapacityPanel.tsx` - Capacity visualization
- `hoop-ui/web/src/CostPanel.tsx` - Cost breakdown
- `hoop-ui/web/src/CrossProjectDashboard.tsx` - Fleet overview
- `hoop-ui/web/src/DebugPanel.tsx` - Per-bead step-through
- `hoop-ui/web/src/DiffViewer.tsx` - Net-diff review
