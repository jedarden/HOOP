# Load Test CI Verification Summary

## Overview

This document verifies that the load test CI job (`hoop-load-test.yaml`) properly implements the Phase 2 exit gate requirement: "Load test: 20 projects × 5 workers × 300 beads synthetic run completes within responsiveness budget."

## Verification Checklist

### ✅ 1. Full-Scale Configuration

**Location:** `.argo/workflowtemplates/hoop-load-test.yaml`

The workflow template supports both medium and full scale:
- **Medium:** 5 projects × 2 workers × 50 beads (PR validation, fast CI feedback)
- **Full:** 20 projects × 5 workers × 300 beads (Release validation, Phase 2 exit gate)

**Evidence:**
```yaml
- name: beads
  script:
    image: alpine:3.19
    command: [sh, -c]
    source: |
      if [ "{{workflow.parameters.scale}}" = "full" ]; then
        echo "300"
      else
        echo "{{workflow.parameters.num_beads}}"
      fi
```

### ✅ 2. Performance Budget Assertions

**Location:** `hoop-daemon/src/load_test.rs` and `hoop-daemon/tests/load_test_integration.rs`

The load test validates the following performance budgets:

| Metric | Budget | Enforcement |
|--------|--------|-------------|
| API Latency | < 500ms | `PerformanceReport::assert_budgets()` fails test if exceeded |
| Memory Ceiling | < 400MB RSS | `PerformanceReport::assert_budgets()` fails test if exceeded |
| WS Fan-out Lag | < 100ms | `PerformanceReport::assert_budgets()` fails test if exceeded |

**Evidence:**
```rust
pub fn assert_budgets(&self, config: &LoadTestConfig) -> anyhow::Result<()> {
    let mut failures = Vec::new();

    // Check API latency budget
    if let Some(&max_latency) = self.api_latencies.iter().max() {
        if max_latency > config.api_latency_budget_ms {
            failures.push(format!(
                "API latency exceeded budget: {}ms > {}ms",
                max_latency, config.api_latency_budget_ms
            ));
        }
    }
    // ... similar checks for memory and WS fan-out
}
```

### ✅ 3. CI Job Exit Behavior

**Location:** `.github/scripts/run-performance-budget-test.sh`

The CI job properly fails when budgets are exceeded:

**Evidence:**
```bash
if RUST_LOG=info cargo test --package hoop-daemon --test load_test_integration load_test_ci_performance_budgets -- --nocapture; then
  RUST_TEST_RESULT=0
  echo -e "${GREEN}✓ Rust load test integration passed${NC}"
else
  RUST_TEST_RESULT=1
  echo -e "${RED}✗ Rust load test integration failed${NC}"
  OVERALL_RESULT=1
  echo ""
  echo "Rust test failure indicates performance budget violation."
  echo "This blocks merge per hoop-ttb.7.11."
  rm -f "$DAEMON_URL_FILE"
  exit 1  # <-- Exits with code 1, failing the Argo workflow
fi
```

### ✅ 4. Phase 2 Exit Gate Wiring

**Location:** `docs/plan/plan.md` line 1002

The plan §10 table includes the load test as a Phase 2 exit gate:

```markdown
| Load test: 20 projects × 5 workers × 300 beads synthetic run completes within responsiveness budget | Phase 2 exit (and each subsequent phase) |
```

### ✅ 5. Runtime and Resource Requirements

**Location:** `docs/load-test.md` and `docs/load_test.md`

Both documentation files include:
- Expected runtime for smoke (~30s), medium (~2-3 min), and full (~15-20 min) scales
- CI resource requirements (CPU: 1-4 cores, Memory: 2-8 Gi)
- Performance budget thresholds
- Troubleshooting guidance

**Evidence:**
| Scale | Configuration | Expected Runtime | Description |
|-------|---------------|------------------|-------------|
| Smoke | 1×1×2 | ~30 seconds | Quick validation during development |
| Medium | 5×2×50 | ~2-3 minutes | PR validation, fast CI feedback |
| Full | 20×5×300 | ~15-20 minutes | Release validation, Phase 2 exit gate |

### ✅ 6. Playwright UI Performance Tests

**Location:** `hoop-ui/web/e2e/load-test-performance.spec.ts`

The Playwright test suite validates UI responsiveness under load:
- Page load time < 2s
- Time to Interactive < 3s
- Interaction latency < 500ms
- Memory ceiling enforcement via `/metrics` endpoint

## Conclusion

The load test CI infrastructure is **complete and properly wired** for the Phase 2 exit gate:

1. ✅ `hoop-load-test.yaml` runs the full 20×5×300 fixture
2. ✅ Budget assertions are enforced (API latency <500ms, RSS <400MB, WS lag <100ms)
3. ✅ CI job fails when budgets exceeded (exits with code 1)
4. ✅ Phase 2 exit gate is documented in plan §10
5. ✅ Runtime and resource requirements are documented
6. ✅ Playwright UI performance tests are integrated

## Changes Made

- Updated `docs/plan/plan.md` line 1002 to reflect correct bead count (300 instead of 200)
- Verified all budget assertions are properly enforced
- Confirmed CI job exit behavior blocks merge on budget violations

## References

- **Plan:** §10 (Phase 2 exit gate), §14.2 bullet 5 (load-test driver), §16.8 (memory budget)
- **Implementation:** `hoop-daemon/src/load_test.rs`
- **Integration tests:** `hoop-daemon/tests/load_test_integration.rs`
- **CI workflow:** `.argo/workflowtemplates/hoop-load-test.yaml`
- **CI script:** `.github/scripts/run-performance-budget-test.sh`
- **Documentation:** `docs/load-test.md`, `docs/load_test.md`
