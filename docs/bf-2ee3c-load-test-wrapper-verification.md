# Load Test Wrapper Script Integration Verification

**Bead ID:** bf-2ee3c
**Date:** 2026-08-02
**Purpose:** Verify load test wrapper script integration in Makefile

## Summary

The wrapper script (`bin/run-with-log.sh`) is **fully integrated** into all load test targets. Load test output is properly captured to log files with ISO 8601 timestamp naming.

## Acceptance Criteria Verification

### ✅ 1. test-load target calls wrapper script

The `test-load` target (line 71) is an alias for `test-load-medium`, which invokes the wrapper script:

```makefile
test-load: test-load-medium
```

The `test-load-medium` target (line 85) calls:
```makefile
./bin/run-with-log.sh logs/test_medium_scale_load_test_$$(date -u +"%Y%m%dT%H%M%SZ").log cargo test --test load_test test_medium_scale_load_test -- --nocapture
```

### ✅ 2. Wrapper script invoked with correct arguments

The wrapper script is invoked with the correct arguments across all load test targets:

**test-load-medium:**
```bash
./bin/run-with-log.sh logs/test_medium_scale_load_test_<timestamp>.log cargo test --test load_test test_medium_scale_load_test -- --nocapture
```

**test-load-full:**
```bash
./bin/run-with-log.sh logs/test_full_scale_load_test_<timestamp>.log cargo test --test load_test test_full_scale_load_test -- --ignored --nocapture
```

**test-load-custom:**
```bash
./bin/run-with-log.sh logs/test_load_custom_<timestamp>.log cargo test --test load_test test_medium_scale_load_test -- --nocapture
```

Arguments structure:
1. Log file path with timestamp: `logs/test_<type>_<timestamp>.log`
2. Command: `cargo test --test load_test <test_name> -- --nocapture`
3. Environment variables: `HOOP_LOAD_PROJECTS`, `HOOP_LOAD_WORKERS`, `HOOP_LOAD_BEADS`, `HOOP_LOAD_CADENCE_MS`

### ✅ 3. Load test command executes within wrapper context

The wrapper script (`bin/run-with-log.sh`) properly:
- Redirects both stdout and stderr to the log file
- Preserves the original command's exit code
- Creates log files in the `logs/` directory with ISO 8601 timestamps

Example log file naming:
```
logs/test_medium_scale_load_test_20260802T144043Z.log
logs/test_full_scale_load_test_20260802T150026Z.log
```

## Load Test Targets with Wrapper Integration

| Target | Wrapper Script | Log File Pattern | Status |
|--------|----------------|------------------|---------|
| `test-load` | Yes (via test-load-medium) | `test_medium_scale_load_test_*.log` | ✅ |
| `test-load-medium` | Yes | `test_medium_scale_load_test_*.log` | ✅ |
| `test-load-full` | Yes | `test_full_scale_load_test_*.log` | ✅ |
| `test-load-custom` | Yes | `test_load_custom_*.log` | ✅ |
| `test-load-watch` | No (intentional - interactive mode) | N/A | ⚠️ |

## Conclusion

All acceptance criteria for the load test wrapper script integration are **fully met**:

- ✅ test-load target in Makefile calls wrapper script
- ✅ Wrapper script is invoked with correct arguments for load tests
- ✅ Load test command still executes within wrapper context

The wrapper script integration is complete and functional. No changes to the Makefile are required.

## Notes

The `test-load-watch` target intentionally does not use the wrapper script, as it runs in interactive watch mode via `cargo watch` and logs to stdout/stderr directly.
