# Phase 2 Exit Gate

## Overview

This gate enforces the plan §10 requirement: **"Phase 2 core deliverables (items 1–13) green before any marquee feature (14–17) is merged."**

The gate prevents marquee features from being merged until all 13 core Phase 2 deliverables have been verified with passing automated tests.

## The 13 Phase 2 Core Deliverables

1. **Project registry** — `projects.yaml` with add/remove/scan/hot-reload
2. **Per-project runtime isolation** — failure in one doesn't cascade
3. **Fleet-of-fleets dashboard** — project cards with worker count, active beads, cost today, stuck count, last activity
4. **Project detail view** — fleet map, bead graph (DAG), strand timeline, conversation list
5. **Cross-project dashboards** — total spend today/week, total workers running, longest-running beads
6. **Ad-hoc vs fleet classification** — filter controls
7. **Unassigned-conversation bucket** — sessions outside any project
8. **Search palette** — cross-project search with project badges
9. **Cost panel** — per-project, per-adapter, per-model, per-strand, per-day; rate-limit window overlay; cost-per-closed-bead
10. **Capacity visibility** — per-account 5h + 7d utilization meters, spend-based caps, burn-rate forecast, saturation alerts (no enforcement)
11. **Visual debug panel** — per-bead step-through: prompts, tool calls, results, stderr, state transitions; scrubable timeline
12. **Collision detector** — alerts when active workers touch overlapping files
13. **Stuck detector** — heartbeat-transition silence or repeated retries surfaced as alerts

## Running the Gate

### Locally

```bash
# Run the gate test
cargo test -p hoop-daemon --test phase2_exit_gate

# View the verification report
cat target/phase2-verification-report.json
```

### In CI

The gate test runs as part of the normal test suite:

```bash
cargo test --all
```

If the gate fails, CI will fail with a message like:

```
Phase 2 exit gate FAILED: 3 of 13 core deliverables lack passing tests.
Marquee features (14-17) cannot merge until all core deliverables are verified.
```

## Verification Report

The gate produces `target/phase2-verification-report.json`:

```json
{
  "total": 13,
  "verified": 13,
  "unverified": 0,
  "passed": true,
  "timestamp": "2026-05-27T12:34:56.789Z",
  "deliverables": [
    {
      "number": 1,
      "title": "Project registry with add/remove/scan/hot-reload",
      "description": "projects.yaml with add/remove/scan/hot-reload commands",
      "success_criteria": [
        "`hoop projects scan ~/` registers every workspace with `.beads/` in one command",
        "Hot-reload on projects.yaml changes within 5s"
      ],
      "test_files": [
        "hoop-daemon/tests/config_field_validation.rs",
        "hoop-daemon/tests/testrepo_harness_integration.rs"
      ],
      "test_names": [
        "test_projects_scan_registers_all_workspaces",
        "test_projects_yaml_hot_reload"
      ],
      "verified": true
    },
    ...
  ]
}
```

## Adding Tests for Deliverables

Each deliverable must have:

1. **Test file path** — Where the test lives (`.rs` or `.ts/.tsx`)
2. **Test function name** — The function or test case name
3. **Success criteria** — What the test validates

To add a test for a deliverable, edit `hoop-daemon/tests/phase2_exit_gate.rs`:

```rust
Phase2Deliverable {
    number: 1,
    title: "Your deliverable title",
    description: "Full description",
    success_criteria: vec![
        "Criterion 1".to_string(),
        "Criterion 2".to_string(),
    ],
    test_files: vec![
        "path/to/your/test.rs".to_string(),
    ],
    test_names: vec![
        "your_test_function_name".to_string(),
    ],
    verified: false,
}
```

## Success Criteria Validation

For each deliverable, the gate validates that:

1. The test file exists at the specified path
2. The test function name exists in that file
3. The test is properly decorated (Rust: `#[test]` or `#[tokio::test]`; TS: `test()` or `it()`)

## Marquee Features (Blocked by Gate)

These features cannot be merged until the gate passes:

- **14.** Stitch abstraction layer (foundational)
- **14b.** Pattern layer (foundational)
- **15.** Stitch-Provenance Code Archaeology
- **16.** Stitch Net-Diff Viewer
- **17.** Cost-Anomaly with Fix Lineage

## Plan Reference

- **Plan §6 Phase 2** — Core deliverables 1-13 and success criteria
- **Plan §10 Phase 2 → Phase 3 gate** — Entry criteria for Phase 3

## Troubleshooting

### Gate fails with "test NOT FOUND"

**Cause:** The test file or function name doesn't exist or doesn't match.

**Fix:**
1. Verify the test file exists at the specified path
2. Verify the function name matches exactly (Rust: `fn test_name(` or `async fn test_name(`)
3. For TypeScript tests, check that the test name in `test('name')` matches the snake_case version in the gate

### Gate fails but tests exist

**Cause:** The test exists but isn't properly decorated or named.

**Fix:**
- Rust tests: Add `#[test]` or `#[tokio::test]` before the function
- TypeScript tests: Ensure the test is defined with `test('name')` or `it('name')`

### Need to add a new test for a deliverable

**Process:**
1. Identify the deliverable number (1-13)
2. Create or add to the appropriate test file
3. Add the test function name to the deliverable's `test_names` array
4. Run the gate again to verify

## Related Documentation

- [Plan §6 Phase 2](../plan/plan.md#6-phased-roadmap) — Full Phase 2 deliverables and success criteria
- [Plan §10 Milestones](../plan/plan.md#10-milestones) — Phase entry/exit criteria
- [Testing Strategy](../plan/plan.md#14-testing-strategy) — Overall testing approach
