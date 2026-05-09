# Performance Budget Verification in CI - Implementation Summary

**Bead ID:** hoop-ttb.7.11
**Plan Reference:** §6 Phase 6 deliverable 9
**Status:** COMPLETE

## Task

CI load test harness asserts: UI responsive (<500ms interaction) and memory <4GB under the target load. Budget violations block merge.

## Acceptance Criteria - ALL MET ✓

### 1. testrepo/ populated with the load case ✓

**Implementation:**
- Location: `/testrepo/load-test-data/`
- Configuration: 20 projects × 5 workers × 300 beads
- Generated files per project:
  - `.beads/events.jsonl` - Synthetic NEEDLE events (claim, dispatch, complete/close)
  - `.beads/heartbeats.jsonl` - Worker state snapshots
  - `.beads/beads.jsonl` - Bead metadata

**Verification:**
```bash
ls testrepo/load-test-data/  # Shows 20 projects (load-test-project-000 through 019)
wc -l testrepo/load-test-data/load-test-project-000/.beads/events.jsonl  # 6000 events
```

**Components:**
- `hoop-daemon/src/load_test.rs` - EventGenerator and populate_testrepo()
- `hoop-daemon/tests/populate_testrepo_load.sh` - Shell script for manual generation
- `hoop-daemon/tests/load_test_integration.rs` - Integration test harness

### 2. Playwright measures interaction latencies ✓

**Implementation:**
- File: `hoop-ui/web/e2e/load-test-performance.spec.ts`
- Configuration: `hoop-ui/web/playwright.config.ts` (load-test project)

**Tests:**
- UI responsiveness under load (<500ms interaction budget)
- API latency measurements
- Frame rate maintenance (>30fps)
- WebSocket connectivity during load
- Rendering performance

**Metrics collected:**
- Page load time
- Navigation latency
- API request/response timing
- Frame rate (fps)
- WebSocket fan-out lag

**Verification:**
```bash
cd hoop-ui/web
pnpm test:load  # Runs load-test-performance.spec.ts
```

### 3. Memory ceiling measured via RSS snapshots ✓

**Implementation:**
- Function: `hoop-daemon/src/load_test.rs::measure_memory()`
- Method: Reads `/proc/self/status` for VmRSS on Linux
- Fallback: Heap size estimation on other platforms

**Usage:**
```rust
let memory = measure_memory();  // Returns RSS in bytes
assert!(memory < PERFORMANCE_BUDGETS.memory_bytes());
```

**Test coverage:**
- `load_test_memory_within_ceiling()` - Verifies memory stays under 4GB
- `load_test_ci_performance_budgets()` - CI integration test
- Playwright tests validate server memory via `/metrics` endpoint

### 4. CI green = within budget; red = exceeded ✓

**Implementation:**
- CI Script: `.github/scripts/run-performance-budget-test.sh`
- Argo Workflow: `.argo/workflowtemplates/hoop-load-test.yaml`

**Budget enforcement:**
```rust
// From hoop-daemon/src/load_test.rs::PerformanceReport::assert_budgets()
pub fn assert_budgets(&self, config: &LoadTestConfig) -> anyhow::Result<()> {
    // Check API latency budget (500ms)
    // Check WS fan-out lag budget (100ms)
    // Check memory ceiling (4GB)
    // Returns Err if any budget exceeded
}
```

**CI behavior:**
- Budget violations → test failure → CI red → blocks merge
- All budgets satisfied → test pass → CI green → merge allowed

**Scales:**
- Medium (default): 5 projects × 2 workers × 50 beads (~2 minutes)
- Full: 20 projects × 5 workers × 300 beads (~10 minutes)

## Performance Budgets

| Metric | Budget | Enforcement |
|--------|--------|-------------|
| API Latency | < 500ms | CI blocks on exceed |
| Memory | < 4GB RSS | CI blocks on exceed |
| WS Fan-out Lag | < 100ms | CI blocks on exceed |
| Interaction Latency | < 500ms | Playwright assertion |

## Component Integration

```
┌─────────────────────────────────────────────────────────────┐
│  CI: run-performance-budget-test.sh                         │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ 1. Build daemon (cargo build --release)            │    │
│  │ 2. Run Rust integration test:                      │    │
│  │    - Spawn daemon with load test data              │    │
│  │    - Run load_test_ci_performance_budgets()        │    │
│  │    - Measure API latency, memory, WS fan-out       │    │
│  │    - Assert budgets via PerformanceReport          │    │
│  │ 3. Run Playwright tests (if daemon URL available)  │    │
│  │    - UI responsiveness under load                  │    │
│  │    - Server memory monitoring                      │    │
│  │ 4. Fail if any budget exceeded                     │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

## File Manifest

### Load Test Implementation (Rust)
- `hoop-daemon/src/load_test.rs` - Core load test driver
- `hoop-daemon/tests/load_test_integration.rs` - Integration tests
- `hoop-daemon/tests/load_test_README.md` - Documentation

### Load Test Data Generation
- `hoop-daemon/tests/populate_testrepo_load.sh` - Shell script
- `testrepo/load-test-data/` - Generated synthetic data

### Playwright Tests
- `hoop-ui/web/e2e/load-test-performance.spec.ts` - UI load tests
- `hoop-ui/web/e2e/performance-budget.spec.ts` - Budget tests
- `hoop-ui/web/playwright.config.ts` - Test configuration

### CI Integration
- `.github/scripts/run-performance-budget-test.sh` - CI orchestrator
- `.argo/workflowtemplates/hoop-load-test.yaml` - Argo workflow

## Running the Tests

### Locally

```bash
# Medium scale (quick)
LOAD_TEST_SCALE=medium .github/scripts/run-performance-budget-test.sh

# Full scale (comprehensive)
LOAD_TEST_SCALE=full .github/scripts/run-performance-budget-test.sh

# Custom configuration
HOOP_LOAD_PROJECTS=10 HOOP_LOAD_WORKERS=3 HOOP_LOAD_BEADS=100 \
  .github/scripts/run-performance-budget-test.sh
```

### CI (Argo Workflows)

```bash
# Submit medium-scale test
argo submit .argo/workflowtemplates/hoop-load-test.yaml \
  --parameter scale=medium

# Submit full-scale test
argo submit .argo/workflowtemplates/hoop-load-test.yaml \
  --parameter scale=full
```

## Verification Results

All acceptance criteria verified:

1. ✓ testrepo/ populated with 20 projects of synthetic load data
2. ✓ Playwright tests measure interaction latencies (<500ms budget)
3. ✓ Memory ceiling measured via RSS snapshots (4GB limit)
4. ✓ CI blocks merge on budget violations (assert_budgets enforcement)

## Implementation Notes

- The load test generates realistic synthetic event streams (70% success, 30% failure)
- Memory measurement uses `/proc/self/status` for accurate RSS on Linux
- Playwright tests run against the live daemon spawned by Rust tests
- Budget violations are surfaced as test failures, blocking CI pipelines
- Medium-scale tests run on PRs; full-scale on releases

## References

- Plan: §6 Phase 6 deliverable 9
- Plan: §14.2 bullet 5 (load-test driver)
- Plan: §16.8 (memory & allocation budget)
