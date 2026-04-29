# HOOP Load Test Infrastructure

## Overview

The load test driver generates concurrent synthetic event streams (20 projects × 5 workers × 200 beads) and drives the daemon. It asserts:

- **UI responsiveness budget**: <500ms API response time
- **Memory ceiling**: <4GB RSS
- **WS fan-out lag**: <100ms broadcast to all clients

## Architecture

### Components

1. **LoadTestConfig** (`hoop-daemon/src/load_test.rs`)
   - Configurable via environment variables
   - Default: 20 projects × 5 workers × 200 beads

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
| `HOOP_LOAD_BEADS` | 200 | Beads per worker |
| `HOOP_LOAD_CADENCE_MS` | 10 | Delay between events (ms) |
| `HOOP_LOAD_TEST_FULL_SCALE` | 0 | Set to 1 to enable full-scale test |

### Performance Budgets

| Metric | Budget | Description |
|--------|--------|-------------|
| API Latency | 500ms | Maximum API response time |
| WS Fan-out Lag | 100ms | Maximum WebSocket broadcast lag |
| Memory Ceiling | 4GB | Maximum RSS memory usage |

## Running Load Tests

### Local Development

```bash
# Quick smoke test (1×1×2)
cargo test --test load_test test_load_test_with_daemon -- --nocapture

# Medium-scale test (5×2×50)
cargo test --test load_test test_medium_scale_load_test -- --nocapture

# Full-scale test (20×5×200)
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
| Main | Push to main | Full (20×5×200) | No (`continue-on-error: true`) |
| Release | Tag push (`v*`) | Full (20×5×200) | **Yes** (required for release) |

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
3. Max memory ≤ 4GB

If any budget is exceeded, the test fails with detailed error messages.

## Plan Reference

§14.2 bullet 5 - Load-test driver for performance budget verification

Feeds into hoop-ttb.7.11 performance budget verification.
