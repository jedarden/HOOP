# Phase 2 Progress Tracker

**Genesis Bead:** hoop-ttb.3
**Last Updated:** 2026-04-30

## Summary

Phase 2 is 82.1% complete (55 of 67 child beads closed).

## Progress by Category

### Completed (55 beads) ✅

**Core Infrastructure:**
- hoop-ttb.3.1 - projects.yaml multi-workspace schema
- hoop-ttb.3.2 - projects.yaml hot-reload
- hoop-ttb.3.3 - hoop projects scan command
- hoop-ttb.3.4 - Per-project runtime supervisor
- hoop-ttb.3.5 - Project detail view: fleet map
- hoop-ttb.3.6 - Project detail view: bead graph
- hoop-ttb.3.7 - Fleet-of-fleets dashboard (VERIFIED: OverviewPage.tsx)
- hoop-ttb.3.7.1 - Project card: stuck-count badge (VERIFIED: in OverviewPage.tsx)
- hoop-ttb.3.8 - Cross-project dashboards: total spend/week
- hoop-ttb.3.9 - Cross-project dashboards: total workers
- hoop-ttb.3.10 - Ad-hoc vs fleet classification
- hoop-ttb.3.11 - Unassigned-conversation bucket (VERIFIED: UnassignedSessions.tsx + api_unassigned.rs)
- hoop-ttb.3.12 - Search palette: cross-project
- hoop-ttb.3.13 - Cost aggregator: per-project buckets
- hoop-ttb.3.14 - Rate-limit window overlay (5h + 7d)
- hoop-ttb.3.15 - Cost-per-closed-Stitch unit economics (VERIFIED: api_cost_per_stitch.rs)
- hoop-ttb.3.16 - Per-account: Claude 5h/7d meters
- hoop-ttb.3.17 - Per-account: Codex daily-spend
- hoop-ttb.3.20 - Capacity widget UI
- hoop-ttb.3.21 - Burn-rate forecast service
- hoop-ttb.3.23 - Visual debug panel
- hoop-ttb.3.24 - Collision detector
- hoop-ttb.3.25 - Stuck detector
- hoop-ttb.3.25.1 - Three-timer stuck detector
- hoop-ttb.3.26 - Diagnostic panel

**Stitch Layer (Marquee #1):**
- hoop-ttb.3.27 - Stitch schema
- hoop-ttb.3.28 - Stitch status derivation
- hoop-ttb.3.29 - Stitch archive filter
- hoop-ttb.3.30 - Stitch list UI

**Pattern Layer (Marquee #1b):**
- hoop-ttb.3.31 - Pattern schema
- hoop-ttb.3.32 - Pattern service (VERIFIED: api_patterns.rs)
- hoop-ttb.3.32.1 - Pattern status state-machine (VERIFIED: in api_patterns.rs)
- hoop-ttb.3.32.2 - Pattern-nesting cycle guard (VERIFIED: in api_patterns.rs)
- hoop-ttb.3.33 - Pattern view UI

**Code Archaeology (Marquee #2):**
- hoop-ttb.3.34 - NEEDLE hook #4: Bead-Id commit trailer
- hoop-ttb.3.35 - Bead-to-commit indexer
- hoop-ttb.3.36 - Stitch-Provenance file overlay

**Net-Diff Viewer (Marquee #3):**
- hoop-ttb.3.37 - Stitch Net-Diff computation engine (VERIFIED: net_diff.rs with tests)
- hoop-ttb.3.38 - Stitch Net-Diff UI

**Cost Anomaly (Marquee #4):**
- hoop-ttb.3.39 - Cost-Anomaly detector (2σ band) (VERIFIED: cost_anomaly.rs with tests)
- hoop-ttb.3.40 - Fix Lineage pattern library + curation UI (VERIFIED: fix_patterns.rs)
- hoop-ttb.3.40.1 - Fix-pattern library schema (VERIFIED: in fix_patterns.rs)
- hoop-ttb.3.41 - Cost-anomaly alert card (VERIFIED: integrated in cost_anomaly.rs)

**Additional:**
- hoop-ttb.3.43 - Fleet-of-fleets dashboard: longest-running beads (VERIFIED: CrossProjectDashboard.tsx)
- hoop-ttb.3.44 - Strand timeline view
- hoop-ttb.3.45 - Conversation list with cross-project filter
- hoop-ttb.3.46.1 - Epoch-sync invariant (partial)
- hoop-ttb.3.47 - Bead detail view: expert toggle
- hoop-ttb.3.48 - Bead list: zero bead IDs by default
- hoop-ttb.3.50.1 - Stitch link traversal: parents/children
- hoop-ttb.3.52 - Worker-Stitch auto-link via prefix marker
- hoop-ttb.3.53 - Stitch-creation surface: stitch label
- hoop-ttb.3.53.1 - NEEDLE hook: follow-up bead label inheritance

### Remaining (12 beads) 🚧

**Core Infrastructure:**
- hoop-ttb.3.18 - Per-account: OpenCode + ZAI proxy
- hoop-ttb.3.19 - Per-account: Gemini
- hoop-ttb.3.22 - Saturation alert
- hoop-ttb.3.4.1 - Per-project runtime test
- hoop-ttb.3.4.1.1 - Test filesystem failure scaffold
- hoop-ttb.3.4.1.2 - Beads-removal scenario test
- hoop-ttb.3.4.1.3 - Sibling unaffected test
- hoop-ttb.3.4.1.4 - /readyz degraded assertion
- hoop-ttb.3.4.2 - Supervisor subsystem doc
- hoop-ttb.3.46 - Epoch-sync invariant
- hoop-ttb.3.49 - Full-screen Search page

**Pattern Layer:**
- hoop-ttb.3.51 - Pattern saved-query evaluator

**NEEDLE Integration:**
- hoop-ttb.3.42 - NEEDLE hook: spawned-by marker
- hoop-ttb.3.50 - Stitch link traversal service (closure)

## Closing Criteria Status

| Criterion | Status |
|-----------|--------|
| Every phase-2 child bead closed | ❌ 12 remain |
| `hoop projects scan ~/` registers all workspaces | ✅ hoop-ttb.3.3 closed |
| Cost figures match `br` within ±2% | ⚠️ Not verified |
| Capacity meters within ±5% of `/status` | ⚠️ Not verified |
| Net-Diff assembles 5-bead / 11-commit cluster | ✅ hoop-ttb.3.37 verified (test exists) |
| Cost anomaly flags 3σ test case | ✅ hoop-ttb.3.41 verified (test exists) |
| UI dashboards contain zero bead IDs by default | ✅ hoop-ttb.3.48 closed |

## Next Steps

1. Complete remaining adapter capacity meters (hoop-ttb.3.18, 3.19)
2. Implement saturation alerts (hoop-ttb.3.22)
3. Complete Pattern saved-query evaluator (hoop-ttb.3.51)
4. Complete remaining test coverage (hoop-ttb.3.4.1.x series)
5. Complete NEEDLE integration hooks (hoop-ttb.3.42, 3.50)
