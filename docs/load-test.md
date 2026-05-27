# Load Testing

## Overview

HOOP includes a synthetic load test driver that generates concurrent event streams (20 projects × 5 workers × 300 beads at canonical load) and validates the daemon against performance budgets.

## Performance Budgets

The load test validates the following performance budgets:

| Metric | Budget | Description |
|--------|--------|-------------|
| API Latency | < 500ms | HTTP API response time |
| WS Fan-out Lag | < 100ms | WebSocket broadcast round-trip time |
| Memory Ceiling | < 400MB | Process RSS memory under canonical load (per plan §16.8) |

## Configuration

Load tests are configured via environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `HOOP_LOAD_PROJECTS` | 20 | Number of synthetic projects |
| `HOOP_LOAD_WORKERS` | 5 | Workers per project |
| `HOOP_LOAD_BEADS` | 300 | Beads per worker |
| `HOOP_LOAD_CADENCE_MS` | 10 | Delay between events (ms) |

## Runtime & Resource Requirements

### Expected Runtime

| Scale | Configuration | Expected Runtime | Description |
|-------|---------------|------------------|-------------|
| Smoke | 1×1×2 | ~30 seconds | Quick validation during development |
| Medium | 5×2×50 | ~2-3 minutes | PR validation, fast CI feedback |
| Full | 20×5×300 | ~15-20 minutes | Release validation, Phase 2 exit gate |

### Resource Requirements

The load test requires the following resources:

**Local Development:**
- **CPU**: 2-4 cores recommended
- **Memory**: 4-8 GB RAM (daemon uses ~200-400MB under load)
- **Disk**: ~500 MB temporary space for synthetic event data

**CI Environment (Argo Workflows):**
- **CPU Request**: 1000m (1 core)
- **CPU Limit**: 4000m (4 cores)
- **Memory Request**: 2 Gi
- **Memory Limit**: 8 Gi

### Performance Budget Thresholds

The load test will fail if any of these thresholds are exceeded:

| Metric | Budget | Rationale |
|--------|--------|-----------|
| API Latency (max) | 500ms | UI responsiveness requirement (plan §16.8) |
| API Latency (p95) | 400ms | 95th percentile should stay well under max |
| Memory Ceiling | 400 MB RSS | Canonical load memory budget (plan §16.8) |
| WS Fan-out Lag | 100ms | Real-time event broadcast requirement |

## Running Load Tests

### Via Makefile

```bash
# Medium-scale test (5x2x50, ~2 minutes)
make test-load

# Full-scale test (20x5x300, ~15 minutes)
make test-load-full

# Custom configuration
HOOP_LOAD_PROJECTS=10 HOOP_LOAD_WORKERS=3 make test-load-custom
```

### Via Cargo

```bash
# Medium-scale test
HOOP_LOAD_PROJECTS=5 HOOP_LOAD_WORKERS=2 HOOP_LOAD_BEADS=50 \
  cargo test --test load_test test_medium_scale_load_test -- --nocapture

# Full-scale test (requires explicit enable)
HOOP_LOAD_TEST_FULL_SCALE=1 \
  cargo test --test load_test test_full_scale_load_test -- --ignored --nocapture
```

### Via Example Binary

```bash
# Build and run the load test example
cargo run --example load-test-runner -- --url http://localhost:3000

# With custom URL
cargo run --example load-test-runner -- --url http://localhost:8080 --verbose
```

## CI Integration

### Phase 2 Exit Gate

The load test is a **Phase 2 exit gate** (plan §10). A full-scale run (20×5×300) must complete within the performance budgets for Phase 2 to be declared complete.

**Phase 2 Exit Criteria:**
- Load test: 20 projects × 5 workers × 300 beads synthetic run completes within responsiveness budget

### Argo Workflows (CI/CD)

The `hoop-load-test` WorkflowTemplate (`.argo/workflowtemplates/hoop-load-test.yaml`) runs in the `iad-ci` cluster.

**What it does:**
1. Clones the HOOP repository
2. Builds the daemon (`cargo build --release --bin hoop-daemon`)
3. Populates testrepo with synthetic load data
4. Runs Rust integration tests (`load_test_ci_performance_budgets`)
5. Measures API latency, memory usage, and WS fan-out lag
6. Asserts all performance budgets are satisfied
7. Fails the workflow if any budget is exceeded

### Workflow Scales

Load tests run in CI with different scales based on context:

| Context | Scale | Required |
|---------|-------|----------|
| Pull Request | Medium (5x2x50) | Optional (doesn't block) |
| Main Branch Push | Medium (5x2x50) | Runs but doesn't block |
| Release Tag | Full (20x5x300) | Required (blocks release) |
| Phase 2 Exit Gate | Full (20x5x300) | Required (blocks phase completion) |
| Manual Dispatch | User selected | Optional |

### Manual CI Trigger

**Via Argo Workflows (iad-ci cluster):**

```bash
# Medium-scale (quick validation)
kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig create -f - <<EOF
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  generateName: hoop-load-test-medium-
  namespace: argo-workflows
spec:
  workflowTemplateRef:
    name: hoop-load-test
  arguments:
    parameters:
      - name: scale
        value: medium
EOF

# Full-scale (Phase 2 exit gate validation)
kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig create -f - <<EOF
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  generateName: hoop-load-test-full-
  namespace: argo-workflows
spec:
  workflowTemplateRef:
    name: hoop-load-test
  arguments:
    parameters:
      - name: scale
        value: full
EOF
```

**Via Argo UI:**
1. Navigate to `https://argo-ci.ardenone.com` (Tailscale VPN required)
2. Go to Workflow Templates → hoop-load-test
3. Click "Submit Workflow"
4. Select scale (medium/full)
5. Click "Submit"

## Test Output

A successful load test produces output like:

```
=== Medium-Scale Load Test ===
Total beads: 10
Total workers: 10
=== Load Test Performance Report ===
Total events: 1000
Result: PASS

API Latency: avg=45ms, max=120ms
WS Fan-out Lag: avg=8ms, max=25ms
Memory: avg=150MB, max=200MB

✓ All performance budgets satisfied
```

## Architecture

### Event Generation

The load test generates synthetic NEEDLE event streams that simulate realistic activity:

1. **Claim** - Worker claims a bead
2. **Dispatch** - Worker starts processing
3. **Complete** (70%) or **Fail** (30%) - Worker finishes
4. **Close** (on success) or **Release** (on failure) - Bead resolved

Events are written to temporary `.beads/` directories with:
- `events.jsonl` - Event stream
- `heartbeats.jsonl` - Worker state snapshots
- `beads.jsonl` - Bead metadata

### Measurement

The load test measures:

1. **API Latency** - Time to complete HTTP requests to `/healthz`, `/api/beads`, `/api/projects`, `/api/metrics`
2. **WS Fan-out Lag** - Time for WebSocket broadcast to reach all clients (measured via subscribe latency)
3. **Memory** - Process RSS sampled before and after the test

### Assertions

After running, the test asserts:
- P95 API latency ≤ budget
- Max WS fan-out lag ≤ budget
- Max memory ≤ ceiling

Violations cause the test to fail with detailed failure messages.

## Integration with hoop-ttb.7.11

This load test driver feeds into the performance budget verification (hoop-ttb.7.11). The performance budgets validated here are the same budgets that must be satisfied for releases.

## Troubleshooting

### Test Timeouts

If tests timeout, try:
- Reducing `HOOP_LOAD_BEADS`
- Increasing `HOOP_LOAD_CADENCE_MS`
- Running the medium-scale test instead of full-scale

### High Memory Usage

If memory exceeds the budget:
- Check for memory leaks in the daemon
- Reduce the number of concurrent projects/workers
- Profile with `valgrind` or `heaptrack`

### WS Fan-out Lag Exceeded

If WebSocket lag is high:
- Check the broadcast channel implementation
- Verify no blocking operations in the WS forwarder
- Check for slow clients

### CI Failures

If load tests fail in CI:
1. Check if it's a flaky test (re-run)
2. Compare with local results
3. Check CI resource limits
4. Review recent changes for performance regressions

## Development

### Adding New Metrics

To add a new performance metric:

1. Add the metric to `LoadTestConfig` in `src/load_test.rs`
2. Collect measurements in `run_load_test()`
3. Add assertion in `PerformanceReport::assert_budgets()`
4. Update this documentation

### Test Data

The load test uses synthetic data that doesn't require external fixtures. For integration tests that need realistic data, see `testrepo/` and the integration harness.

## References

- **Plan references:** §10 (Phase 2 exit gate), §14.2 bullet 5 (load-test driver), §16.8 (memory budget)
- **Feeds into:** hoop-ttb.7.11 (Performance budget verification in CI)
- **Implementation:** `hoop-daemon/src/load_test.rs`
- **Integration tests:** `hoop-daemon/tests/load_test_integration.rs`
- **CI workflow:** `.argo/workflowtemplates/hoop-load-test.yaml`
- **CI script:** `.github/scripts/run-performance-budget-test.sh`
- **Example:** `hoop-daemon/examples/load-test-runner.rs`
