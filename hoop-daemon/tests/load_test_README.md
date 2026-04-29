# Load Test Driver

Synthetic event stream generator for load testing the HOOP daemon.

## Overview

The load test driver generates concurrent synthetic event streams (20 projects × 5 workers × 300 beads by default) and drives the daemon to assert performance budgets:

- **UI responsiveness budget**: API latency < 500ms
- **Memory ceiling**: RSS < 4GB
- **WS fan-out lag**: Broadcast latency < 100ms

## Configuration

Configure via environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `HOOP_LOAD_PROJECTS` | 20 | Number of synthetic projects |
| `HOOP_LOAD_WORKERS` | 5 | Workers per project |
| `HOOP_LOAD_BEADS` | 200 | Beads per worker |
| `HOOP_LOAD_CADENCE_MS` | 10 | Delay between events (ms) |

## Running Load Tests

### Quick Smoke Test (Small Scale)

```bash
cargo test --test load_test test_load_test_with_daemon -- --nocapture
```

### Medium Scale (5×2×50, ~2 minutes)

```bash
cargo test --test load_test test_medium_scale_load_test -- --nocapture
```

### Full Scale (20×5×300, requires explicit enable)

```bash
HOOP_LOAD_TEST_FULL_SCALE=1 cargo test --test load_test test_full_scale_load_test -- --ignored --nocapture
```

### Custom Configuration

```bash
HOOP_LOAD_PROJECTS=10 HOOP_LOAD_WORKERS=3 HOOP_LOAD_BEADS=100 \
  cargo test --test load_test test_medium_scale_load_test -- --nocapture
```

## CI Integration

The load test runs in CI via `.github/workflows/load-test.yml`:

- **PRs**: Medium scale, blocks merge if configured as required check in branch protection
- **Main branch**: Full scale, blocks merge on budget violations
- **Releases**: Full scale, **required** for release
- **Manual**: Trigger via GitHub Actions UI with scale selection

### Configuring PR Load Test as Required

To make the PR load test block merges, configure it as a required check in GitHub branch protection rules:

1. Go to repository Settings → Branches
2. Edit the branch protection rule for `main`
3. Under "Protect matching branches", enable "Require status checks to pass before merging"
4. Add `Load Test (Medium Scale)` and `UI Performance Budget (Playwright)` as required checks

This ensures performance budgets are enforced before any PR can merge.

## Performance Budgets

The load test asserts these performance budgets (hoop-ttb.7.11):

| Metric | Budget | Rationale |
|--------|--------|-----------|
| API Latency | < 500ms p95 | UI responsiveness |
| Memory | < 4GB RSS | Production resource limits |
| WS Fan-out Lag | < 100ms | Real-time collaboration |

## Implementation Details

### Event Generation

The `EventGenerator` creates synthetic NEEDLE events:

1. **Claim events**: Worker claims a bead
2. **Dispatch events**: Bead dispatched to agent
3. **Complete/Close events** (70%): Successful completion
4. **Fail/Release events** (30%): Simulated failures

Each worker processes its configured bead count with realistic timing.

### Performance Measurement

- **API Latency**: Measured via HTTP request timing
- **WS Fan-out**: Measured by tracking broadcast delivery to multiple clients
- **Memory**: Sampled via `/proc/self/status` (Linux) or estimated

## Plan Reference

§14.2 bullet 5 - Load-test driver for synthetic event generation

## Feeds Into

hoop-ttb.7.11 - Performance budget verification
