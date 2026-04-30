# Phase 2 Completion Plan

**Date:** 2026-04-30
**Parent:** hoop-ttb.3 (Genesis: Phase 2 Implementation)
**Approach:** Systematic validation and batch closure by feature group

## Completion Strategy

Given that Phase 2 has 68 child beads and most implementations are complete, we'll use a **validation-driven batch closure** approach:

1. **Validate feature group** (integration test + manual verification)
2. **Close entire bead group** (all related beads in one pass)
3. **Document any gaps** (create new child beads for missing work)
4. **Update genesis progress** (track completion %)

## Batch Closure Groups

### Batch 1: Foundation & Project Registry (6 beads)
**Estimated effort:** 1 day

| Bead | Feature | Validation |
|------|---------|------------|
| hoop-ttb.3.1 | Multi-workspace schema | Test schema parsing, verify backward compat |
| hoop-ttb.3.2 | Hot-reload | Test config change, verify reload |
| hoop-ttb.3.3 | projects scan | Run scan on test host, verify discovery |
| hoop-ttb.3.4 | Runtime isolation | Kill one project, verify others unaffected |
| hoop-ttb.3.5 | Cross-project state | Verify fleet.db queries |
| hoop-ttb.3.6 | WS topic routing | Test per-project subscriptions |

### Batch 2: Dashboard & Views (7 beads)
**Estimated effort:** 1 day

| Bead | Feature | Validation |
|------|---------|------------|
| hoop-ttb.3.7 | Fleet-of-fleets dashboard | Verify project cards render |
| hoop-ttb.3.8 | Project detail view | Verify all panels load |
| hoop-ttb.3.9 | Cross-project dashboards | Verify totals accuracy |
| hoop-ttb.3.10 | Ad-hoc vs fleet | Test classification filter |
| hoop-ttb.3.11 | Unassigned bucket | Create orphan session, verify bucket |
| hoop-ttb.3.12 | Search palette | Test cross-project search |
| hoop-ttb.3.47 | Overview page | Verify home route |

### Batch 3: Cost & Capacity (10 beads)
**Estimated effort:** 2 days

| Bead | Feature | Validation |
|------|---------|------------|
| hoop-ttb.3.13 | Cost aggregator | Compare to `br` output |
| hoop-ttb.3.14 | Claude rate-limit | Verify overlay on chart |
| hoop-ttb.3.15 | Cost-per-Stitch | Check unit economics |
| hoop-ttb.3.16 | Claude utilization | Compare to `/status` (±5%) |
| hoop-ttb.3.17 | Codex utilization | Verify daily-spend |
| hoop-ttb.3.18 | OpenCode/ZAI | Verify limits |
| hoop-ttb.3.19 | Gemini | Verify JSONL estimation |
| hoop-ttb.3.20 | Capacity widget | Verify UI rendering |
| hoop-ttb.3.21 | Burn-rate forecast | Test prediction accuracy |
| hoop-ttb.3.22 | Saturation alert | Test alert threshold |

### Batch 4: Visual Debug & Diagnostics (4 beads)
**Estimated effort:** 1 day

| Bead | Feature | Validation |
|------|---------|------------|
| hoop-ttb.3.23 | Visual debug panel | Reconstruct known bead cycle |
| hoop-ttb.3.24 | Collision detector | Test on overlapping work |
| hoop-ttb.3.25 | Stuck detector | Create synthetic stuck worker |
| hoop-ttb.3.26 | Diagnostic panel | Verify unknown-event counter |

### Batch 5: Stitch Abstraction (4 beads)
**Estimated effort:** 1 day

| Bead | Feature | Validation |
|------|---------|------------|
| hoop-ttb.3.27 | Stitch schema | Verify round-trip |
| hoop-ttb.3.28 | Status derivation | Test all state transitions |
| hoop-ttb.3.29 | Archive filter | Test N-day quiet rule |
| hoop-ttb.3.30 | Stitch list UI | Verify ranking |

### Batch 6: Pattern Layer (3 beads)
**Estimated effort:** 1 day

| Bead | Feature | Validation |
|------|---------|------------|
| hoop-ttb.3.31 | Pattern schema | Verify round-trip |
| hoop-ttb.3.32 | Pattern service | Test state machine |
| hoop-ttb.3.33 | Pattern view UI | Test cross-project aggregate |

### Batch 7: Stitch-Provenance (3 beads)
**Estimated effort:** 2 days (includes NEEDLE coordination)

| Bead | Feature | Validation |
|------|---------|------------|
| hoop-ttb.3.34 | Bead-Id trailer | Submit NEEDLE PR |
| hoop-ttb.3.35 | Bead-to-commit index | Test git log walk |
| hoop-ttb.3.36 | File overlay attribution | Test hover on known commits |

### Batch 8: Net-Diff Viewer (2 beads)
**Estimated effort:** 1 day

| Bead | Feature | Validation |
|------|---------|------------|
| hoop-ttb.3.37 | Net-Diff computation | Test 5-bead/11-commit cluster |
| hoop-ttb.3.38 | Net-Diff UI | Verify unified PR view |

### Batch 9: Cost-Anomaly (3 beads)
**Estimated effort:** 1 day

| Bead | Feature | Validation |
|------|---------|------------|
| hoop-ttb.3.39 | Anomaly detector | Test synthetic 3σ case |
| hoop-ttb.3.40 | Fix pattern library | Test curation UI |
| hoop-ttb.3.41 | Anomaly alert card | Verify alert rendering |

### Batch 10: NEEDLE Hooks & Integration (4 beads)
**Estimated effort:** 2 days (includes NEEDLE coordination)

| Bead | Feature | Validation |
|------|---------|------------|
| hoop-ttb.3.42 | spawned-by marker | Submit NEEDLE PR |
| hoop-ttb.3.43 | stitch label propagation | Submit NEEDLE PR |
| hoop-ttb.3.44 | Orphan-bead detector | Create orphan, verify detection |
| hoop-ttb.3.45 | Reference linking | Test kind=references |

### Batch 11: WebSocket & UI Polish (4 beads)
**Estimated effort:** 1 day

| Bead | Feature | Validation |
|------|---------|------------|
| hoop-ttb.3.46 | Epoch-sync invariant | Run epochSync.test.ts |
| hoop-ttb.3.48 | Conversations view | Test filter behavior |
| hoop-ttb.3.49 | Full-screen search | Test >50 results |
| hoop-ttb.3.53 | WS broadcast events | Verify bead_created_by_hoop |

### Batch 12: Stitch Link Traversal (3 beads)
**Estimated effort:** 1 day

| Bead | Feature | Validation |
|------|---------|------------|
| hoop-ttb.3.50 | Link traversal | Test closure computation |
| hoop-ttb.3.51 | Query evaluator | Test auto-include |
| hoop-ttb.3.52 | Aggregated-read | Test /api/stitches/:id |

### Batch 13: Test Infrastructure (12 beads)
**Estimated effort:** 3 days

| Bead Range | Feature |
|------------|----------|
| hoop-ttb.3.4.1.x | Per-project runtime tests |
| hoop-ttb.3.32.1.x | Pattern state machine tests |
| hoop-ttb.3.46.1.x | WS event convention tests |
| hoop-ttb.3.50.1.x | Cross-workspace dependency tests |

## Progress Tracking

Current completion estimate: **75%** (implementation complete, validation pending)

| Batch | Beads | Status | Blocker |
|-------|-------|--------|---------|
| 1: Foundation | 6 | Ready for validation | None |
| 2: Dashboard | 7 | Ready for validation | None |
| 3: Cost/Capacity | 10 | Ready for validation | None |
| 4: Debug | 4 | Ready for validation | None |
| 5: Stitch | 4 | Ready for validation | None |
| 6: Pattern | 3 | Ready for validation | None |
| 7: Provenance | 3 | Blocked on NEEDLE | PR submission |
| 8: Net-Diff | 2 | Ready for validation | None |
| 9: Anomaly | 3 | Ready for validation | None |
| 10: Hooks | 4 | Blocked on NEEDLE | PR submission |
| 11: WS/UI | 4 | Ready for validation | None |
| 12: Traversal | 3 | Ready for validation | None |
| 13: Tests | 12 | Partial | Test coverage |

## Immediate Next Actions

1. **Start Batch 1 validation** - Run existing tests, document results
2. **Create validation script** - Automated test runner for all batches
3. **Submit NEEDLE PRs** - Unblock Batches 7 and 10
4. **Close Batch 1** - First set of beads closed

## Dependencies

- **NEEDLE PRs needed for:** hoop-ttb.3.34, hoop-ttb.3.42, hoop-ttb.3.43
- **Test coverage needed for:** All batches (current coverage ~60%)
- **Documentation updates needed:** README.md, operations.md

## Success Criteria

Phase 2 is complete when:
- [ ] All 68 child beads are closed
- [ ] All validation tests pass
- [ ] Cost figures match `br` within ±2%
- [ ] Capacity meters match `/status` within ±5%
- [ ] Net-Diff correctly assembles 5-bead/11-commit cluster
- [ ] Cost anomaly flags synthetic 3σ test case
- [ ] NEEDLE hooks submitted (or blocked on NEEDLE team)
- [ ] Dashboards contain zero bead IDs by default
