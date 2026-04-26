# Load Testing

## Overview

HOOP includes a synthetic load test driver that generates concurrent event streams (20 projects × 5 workers × 200 beads) and validates the daemon against performance budgets.

## Performance Budgets

The load test validates the following performance budgets:

| Metric | Budget | Description |
|--------|--------|-------------|
| API Latency | < 500ms | HTTP API response time |
| WS Fan-out Lag | < 100ms | WebSocket broadcast round-trip time |
| Memory Ceiling | < 4GB | Process RSS memory |

## Configuration

Load tests are configured via environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `HOOP_LOAD_PROJECTS` | 20 | Number of synthetic projects |
| `HOOP_LOAD_WORKERS` | 5 | Workers per project |
| `HOOP_LOAD_BEADS` | 200 | Beads per worker |
| `HOOP_LOAD_CADENCE_MS` | 10 | Delay between events (ms) |

## Running Load Tests

### Via Makefile

```bash
# Medium-scale test (5x2x50, ~2 minutes)
make test-load

# Full-scale test (20x5x200, ~10 minutes)
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

Load tests run in CI with different scales based on context:

| Context | Scale | Required |
|---------|-------|----------|
| Pull Request | Medium (5x2x50) | Optional (doesn't block) |
| Main Branch Push | Medium (5x2x50) | Runs but doesn't block |
| Release Tag | Full (20x5x200) | Required (blocks release) |
| Manual Dispatch | User selected | Optional |

### Manual CI Trigger

You can trigger load tests manually via the GitHub Actions UI:

1. Go to Actions → Load Tests
2. Click "Run workflow"
3. Select scale (medium/full)
4. Click "Run workflow"

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

- Plan reference: §14.2 bullet 5
- Feeds into: hoop-ttb.7.11 (Performance budget verification in CI)
- Implementation: `hoop-daemon/src/load_test.rs`
- Tests: `hoop-daemon/tests/load_test.rs`
- Example: `hoop-daemon/examples/load-test-runner.rs`
