# Performance Budget Verification in CI (hoop-ttb.7.11)

## Acceptance Criteria - ALL MET ✓

### 1. testrepo/ populated with the load case ✓
- **20 projects** created: `load-test-project-000` through `load-test-project-019`
- **5 workers per project**
- **300 beads per worker** (30,000 total beads)
- **4 events per bead** (Claim, Dispatch, Complete/Close or Fail/Release)
- **6,000 events per project** (120,000 total events)
- Location: `testrepo/load-test-data/`

### 2. Playwright measures interaction latencies ✓
- File: `hoop-ui/web/e2e/load-test-performance.spec.ts`
- Tests measure:
  - Connection time to daemon
  - Bead list render time
  - Bead interaction latency
  - Rapid navigation latency
  - API request latency
  - Concurrent request handling
- All tests assert `< 500ms` response time budget
- Run via: `pnpm test:load` (uses `load-test` project in playwright.config.ts)

### 3. Memory ceiling measured via rss snapshots ✓
- File: `hoop-daemon/src/load_test.rs`
- Function: `measure_memory()` reads `/proc/self/status` for VmRSS
- Memory samples collected during load test
- `< 4GB` ceiling enforced in `PerformanceReport::assert_budgets()`
- Memory sampled before, during, and after load generation

### 4. CI green = within budget; red = exceeded ✓
- **Argo Workflow**: `.argo/workflowtemplates/hoop-load-test.yaml`
  - Runs on medium scale (5×2×50) for PRs
  - Runs on full scale (20×5×300) for releases
  - Budget violations cause workflow failure

- **CI Script**: `.github/scripts/run-performance-budget-test.sh`
  - Builds daemon
  - Runs Rust integration test (`load_test_ci_performance_budgets`)
  - Runs Playwright UI performance tests
  - Exits with non-zero on budget violations

- **Rust Test**: `hoop-daemon/tests/load_test_integration.rs::load_test_ci_performance_budgets`
  - Calls `report.assert_budgets()` which fails test if any budget exceeded

## Performance Budgets

| Metric | Budget | Implementation |
|--------|--------|----------------|
| API Latency | < 500ms | `PERFORMANCE_BUDGETS.api_latency_ms` |
| Memory | < 4GB | `PERFORMANCE_BUDGETS.memory_gb` |
| WS Fan-out Lag | < 100ms | `PERFORMANCE_BUDGETS.ws_fanout_lag_ms` |

## Running Locally

```bash
# Medium scale (quick test)
cargo test --package hoop-daemon --test load_test_integration load_test_ci_performance_budgets -- --nocapture

# Full scale (requires explicit enable)
HOOP_LOAD_TEST_FULL_SCALE=1 cargo test --package hoop-daemon --test load_test_integration load_test_ci_performance_budgets -- --ignored --nocapture

# With Playwright UI tests
pnpm test:load
```

## Files Modified/Created for This Implementation

1. `hoop-daemon/src/load_test.rs` - Core load test implementation
2. `hoop-daemon/tests/load_test_integration.rs` - Integration tests with performance assertions
3. `hoop-ui/web/e2e/load-test-performance.spec.ts` - Playwright UI performance tests
4. `.github/scripts/run-performance-budget-test.sh` - CI test runner script
5. `.argo/workflowtemplates/hoop-load-test.yaml` - Argo workflow template
6. `hoop-daemon/tests/populate_testrepo_load.sh` - Shell script to populate testrepo
7. `scripts/populate-load-test-data.py` - Python script to populate testrepo
8. `testrepo/load-test-data/` - 20 projects with synthetic load data

## Plan Reference
§6 Phase 6 deliverable 9
