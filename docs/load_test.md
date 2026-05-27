# HOOP Load Test Infrastructure

## Overview

The load test driver generates concurrent synthetic event streams (20 projects × 5 workers × 300 beads) and drives the daemon. It asserts:

- **UI responsiveness budget**: <500ms API response time
- **Memory ceiling**: <400MB RSS (per plan §16.8)
- **WS fan-out lag**: <100ms broadcast to all clients

## Architecture

### Components

1. **LoadTestConfig** (`hoop-daemon/src/load_test.rs`)
   - Configurable via environment variables
   - Default: 20 projects × 5 workers × 300 beads

2. **EventGenerator** (`hoop-daemon/src/load_test.rs`)
   - Generates synthetic NEEDLE event streams
   - Realistic claim → dispatch → complete/close flow
   - 70% success, 30% failure rate

3. **PerformanceReport** (`hoop-daemon/src/load_test.rs`)
   - Tracks API latencies, WS fan-out lag, memory usage
   - Asserts budgets are satisfied

4. **run_load_test()** (`hoop-daemon/src/load_test.rs`)
   - Main test runner
   - Spawns daemon, makes API calls, measures performance

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `HOOP_LOAD_PROJECTS` | 20 | Number of synthetic projects |
| `HOOP_LOAD_WORKERS` | 5 | Workers per project |
| `HOOP_LOAD_BEADS` | 300 | Beads per worker |
| `HOOP_LOAD_CADENCE_MS` | 10 | Delay between events (ms) |
| `HOOP_LOAD_TEST_FULL_SCALE` | 0 | Set to 1 to enable full-scale test |

### Performance Budgets

| Metric | Budget | Description |
|--------|--------|-------------|
| API Latency | 500ms | Maximum API response time |
| WS Fan-out Lag | 100ms | Maximum WebSocket broadcast lag |
| Memory Ceiling | 400MB | Maximum RSS memory usage under canonical load (per plan §16.8) |

## Running Load Tests

### Local Development

```bash
# Quick smoke test (1×1×2)
cargo test --test load_test test_load_test_with_daemon -- --nocapture

# Medium-scale test (5×2×50)
cargo test --test load_test test_medium_scale_load_test -- --nocapture

# Full-scale test (20×5×300)
HOOP_LOAD_TEST_FULL_SCALE=1 cargo test --test load_test test_full_scale_load_test -- --ignored --nocapture
```

### Argo Workflows (CI/CD)

The load test runs via Argo Workflows in the `iad-ci` cluster.

#### Submitting a workflow

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

# Full-scale (release validation)
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

#### Checking workflow status

```bash
# List recent workflows
kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig \
  get workflows -n argo-workflows --sort-by=.metadata.creationTimestamp | tail -20

# Get workflow logs
kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig \
  logs -n argo-workflows <pod-name> -c main
```

## CI Integration

### GitHub Actions (Deprecated)

GitHub Actions workflows exist at `.github/workflows/load-test.yml` but are **disabled** per project instructions. All CI/CD runs via Argo Workflows.

### Argo Workflows

The `hoop-load-test` WorkflowTemplate is defined in `.argo/workflowtemplates/hoop-load-test.yaml`.

**Usage tiers:**

| Tier | Trigger | Scale | Blocking |
|------|---------|-------|----------|
| PR | Pull request | Medium (5×2×50) | No (`continue-on-error: true`) |
| Main | Push to main | Full (20×5×300) | No (`continue-on-error: true`) |
| Release | Tag push (`v*`) | Full (20×5×300) | **Yes** (required for release) |

## Test Files

| File | Purpose |
|------|---------|
| `hoop-daemon/src/load_test.rs` | Load test library (config, generator, runner) |
| `hoop-daemon/tests/load_test.rs` | Integration tests |
| `.github/workflows/load-test.yml` | GitHub Actions (deprecated, reference only) |
| `.argo/workflowtemplates/hoop-load-test.yaml` | Argo WorkflowTemplate |

## Implementation Details

### Event Generation

The `EventGenerator` creates realistic event streams:

1. **Claim**: Worker claims a bead
2. **Dispatch**: Worker starts processing (with adapter/model info)
3. **Complete/Fail**: Worker finishes (70/30 split)
4. **Close/Release**: Bead closed or released back to pool

### Performance Measurement

- **API Latency**: Measured via HTTP request timing
- **WS Fan-out Lag**: Measured via multiple WS clients receiving broadcast events
- **Memory**: Measured via `/proc/self/status` (VmRSS)

### Budget Assertion

The `PerformanceReport::assert_budgets()` method checks:

1. Max API latency ≤ 500ms
2. Max WS fan-out lag ≤ 100ms
3. Max memory ≤ 400MB

If any budget is exceeded, the test fails with detailed error messages.

## Expected Runtime and Resource Requirements

### Medium Scale (5×2×50 = 500 beads)

- **Expected runtime**: 3-5 minutes
- **CPU requests**: 1000m (1 core)
- **CPU limits**: 4000m (4 cores)
- **Memory requests**: 2Gi
- **Memory limits**: 8Gi
- **Disk usage**: ~50MB for synthetic test data

### Full Scale (20×5×300 = 30,000 beads)

- **Expected runtime**: 10-15 minutes
- **CPU requests**: 1000m (1 core)
- **CPU limits**: 4000m (4 cores)
- **Memory requests**: 2Gi
- **Memory limits**: 8Gi
- **Disk usage**: ~3GB for synthetic test data

### Runtime Breakdown

The load test runtime consists of:

1. **Build phase** (2-3 min): `cargo build --release --bin hoop-daemon`
2. **Data generation** (1-2 min): Synthetic events/beads written to temp directory
3. **Daemon startup** (10-30s): Daemon boots with load test data
4. **Load test execution** (2-5 min for medium, 5-8 min for full): API calls, WS fan-out, memory sampling
5. **Playwright tests** (1-2 min): UI responsiveness validation (if available)

### CI Resource Limits

The Argo WorkflowTemplate configures the following resource limits:

```yaml
resources:
  requests:
    memory: "2Gi"
    cpu: "1000m"
  limits:
    memory: "8Gi"
    cpu: "4000m"
```

These limits are sufficient for the full-scale test. The daemon should stay well under the 400MB RSS ceiling even at full scale.

## CI Job Exit Behavior

The load test CI job **fails** when any of the following conditions are met:

1. **API latency exceeds 500ms**: Any API call takes longer than the responsiveness budget
2. **Memory exceeds 400MB**: Daemon RSS exceeds the memory ceiling
3. **WS fan-out lag exceeds 100ms**: WebSocket broadcast to all clients takes too long

The failure is enforced at two levels:

1. **Rust integration test**: `load_test_ci_performance_budgets` in `hoop-daemon/tests/load_test_integration.rs` calls `PerformanceReport::assert_budgets()` which returns an error if budgets are exceeded
2. **Shell script**: `.github/scripts/run-performance-budget-test.sh` exits with code 1 when the Rust test fails, causing the Argo workflow to fail

This ensures that performance budget violations block merge per the Phase 2 exit gate requirement.

## Plan Reference

- **§10 Phase 2 exit gate**: Load test: 20 projects × 5 workers × 300 beads synthetic run completes within responsiveness budget
- **§14.2 bullet 5**: Load-test driver for performance budget verification

Feeds into hoop-ttb.7.11 performance budget verification.
