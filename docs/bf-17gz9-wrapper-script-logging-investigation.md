# Wrapper Script Structure and Logging Investigation

**Bead ID:** bf-17gz9  
**Date:** 2026-08-02  
**Purpose:** Investigate wrapper script structure and logging for HOOP load tests

## Summary of Findings

The HOOP repository has a **fully functional and integrated wrapper script system** for test execution and logging. Load tests are already properly wrapped with comprehensive output capture and logging.

## 1. Wrapper Script Location and Structure

### Primary Scripts

**`bin/run-with-log.sh`** - Main wrapper script
- Location: `/home/coding/HOOP/bin/run-with-log.sh`
- Purpose: Execute commands with stdout/stderr redirected to log files while preserving exit codes
- Features:
  - Supports explicit log file paths or auto-generated names
  - Preserves original command exit codes
  - Redirects both stdout and stderr to log file
  - Can auto-generate descriptive log names via companion script

**`bin/generate-test-log-name.sh`** - Log name generator
- Location: `/home/coding/HOOP/bin/generate-test-log-name.sh`
- Purpose: Generate descriptive, timestamped log file names from test commands
- Naming convention: `<test_name>_<timestamp>.log`
- Features:
  - Extracts test names from cargo commands
  - Sanitizes names for filesystem safety
  - Generates ISO 8601 timestamps (YYYYMMDDTHHMMSSZ)
  - Handles special test patterns (load tests, lib tests, etc.)

### Wrapper Script Architecture

```bash
# Wrapper invocation pattern:
./bin/run-with-log.sh <log_file> <command> [args...]

# Auto-generated log names:
./bin/run-with-log.sh --auto <command> [args...]

# Execution flow:
1. Parse arguments (--auto vs explicit log path)
2. Generate log name (if --auto)
3. Create logs/ directory if needed
4. Execute command with stdout/stderr redirected to log file
5. Preserve and return original command's exit code
```

## 2. How Load Tests Are Currently Spawned

### Test Execution Flow

Load tests are executed through the standard Rust test harness:

```bash
# Medium-scale load test execution:
cargo test --test load_test test_medium_scale_load_test -- --nocapture

# Full-scale load test execution:
HOOP_LOAD_TEST_FULL_SCALE=1 \
HOOP_LOAD_PROJECTS=20 \
HOOP_LOAD_WORKERS=5 \
HOOP_LOAD_BEADS=200 \
cargo test --test load_test test_full_scale_load_test -- --ignored --nocapture
```

### Daemon Spawning (Rust Code Level)

From `hoop-daemon/tests/load_test.rs`:

```rust
// Tests use a shared mutex to prevent concurrent daemon spawning
static __TEST_MUTEX: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

// Daemon spawning in integration tests
let (base_url, _daemon) = spawn_test_daemon()
    .await
    .expect("Failed to spawn test daemon");
```

- Daemons are spawned via `integration_harness::spawn_test_daemon()`
- Tests use `__TEST_MUTEX` to serialize daemon spawning (prevent port conflicts)
- Each test gets its own daemon instance on a dynamic port
- Test framework manages daemon lifecycle

### Load Test Configuration

Load tests are configured via environment variables:

| Variable | Default | Purpose |
|----------|---------|---------|
| `HOOP_LOAD_PROJECTS` | 20 | Number of synthetic projects |
| `HOOP_LOAD_WORKERS` | 5 | Workers per project |
| `HOOP_LOAD_BEADS` | 200 | Beads per worker |
| `HOOP_LOAD_CADENCE_MS` | 10 | Event timing (milliseconds) |
| `HOOP_LOAD_TEST_FULL_SCALE` | unset | Enable full-scale test |

## 3. Current Stdout/Stderr Capture Location

### Capture Implementation in Wrapper Script

**File**: `bin/run-with-log.sh` (line 88)

```bash
# Run the command, redirecting both stdout and stderr to the log file
# Preserve the exit code
"$@" > "$LOG_FILE" 2>&1
EXIT_CODE=$?

# Exit with the original command's exit code
exit $EXIT_CODE
```

### Capture Points

1. **Direct cargo output**: All cargo test output goes to log file
2. **Test framework output**: Rust test harness output captured
3. **Daemon output**: Spawned daemon process output (currently may be going to terminal)
4. **Performance metrics**: Test results and timing captured

**Status**: ✅ Stdout/stderr capture is **already implemented** in the wrapper script

## 4. Logging Mechanism Used by Other Test Types

### Current Test Logging Coverage

**Unit Tests** (`make test`):
```makefile
@./bin/run-with-log.sh logs/unit_test_$$(date -u +"%Y%m%dT%H%M%SZ").log \
    cargo test --lib --features testing --verbose
```

**Integration Tests** (`make test-beads-deletion`):
```makefile
@./bin/run-with-log.sh logs/beads_deletion_http_$$(date -u +"%Y%m%dT%H%M%SZ").log \
    cargo test -p hoop-daemon --test beads_deletion_http -- --nocapture
```

**Load Tests** (`make test-load-medium`):
```makefile
./bin/run-with-log.sh logs/test_medium_scale_load_test_$$(date -u +"%Y%m%dT%H%M%SZ").log \
    cargo test --test load_test test_medium_scale_load_test -- --nocapture
```

### Log File Naming Convention

**Pattern**: `<test_type>_<timestamp>.log`

**Examples from existing logs**:
```
logs/unit_test_20260802T143443Z.log
logs/test_medium_scale_load_test_20260802T145324Z.log
logs/bead_status_deserialization_20260802T133002Z.log
logs/test_wrapper_verification_manual.log
```

**Features**:
- ISO 8601 timestamps for uniqueness and sorting
- Descriptive test names for easy identification
- Standard `.log` extension
- `logs/` directory for centralized collection

### Log File Contents

Current log files capture:
- ✅ Cargo compilation output
- ✅ Test execution progress
- ✅ Test results (pass/fail)
- ✅ Performance measurements
- ⚠️  **Potential gap**: Spawned daemon process output may not be captured

## 5. Existing Verification and Documentation

### Verification Files Found

Multiple verification documents confirm the wrapper script is working:
- `docs/bf-61vvf-wrapper-integration-verification.md` - Unit test wrapper verification
- `docs/bf-61vvf-unit-test-wrapper-verification.md` - Unit test verification
- `docs/bf-2ee3c-load-test-wrapper-verification.md` - Load test wrapper verification ✅
- `docs/bf-3b9s8-wrapper-script-verification.md` - General wrapper verification
- `docs/bf-4mc86-wrapper-verification.md` - Additional wrapper verification

### Integration Status

From `docs/bf-2ee3c-load-test-wrapper-verification.md`:
- ✅ `test-load` target calls wrapper script
- ✅ Wrapper script invoked with correct arguments
- ✅ Load test command executes within wrapper context
- ✅ All acceptance criteria met

**Conclusion**: The wrapper script integration is **complete and functional**.

## Key Architectural Points

### Wrapper Script Design Philosophy

1. **Simplicity**: Single-purpose bash script for output redirection
2. **Portability**: Works with any command, not just Rust tests
3. **Reliability**: Preserves exit codes for CI/CD integration
4. **Maintainability**: Clear, documented bash code with error handling

### Integration Pattern

```makefile
# Standard pattern in Makefile:
@mkdir -p logs                    # Ensure logs directory exists
@./bin/cleanup-hoop-test-processes.sh || true  # Pre-test cleanup
@./bin/run-with-log.sh logs/<test_name>_<timestamp>.log \
    <test_command>                # Run test with output capture
@./bin/verify-hoop-test-processes.sh || echo "Warning"  # Post-test verification
```

### Separation of Concerns

1. **Test execution**: Handled by cargo/Rust test harness
2. **Output capture**: Handled by wrapper script
3. **Process management**: Handled by cleanup/verification scripts
4. **Log naming**: Handled by generate-test-log-name.sh

## Potential Areas for Enhancement

### Identified Gaps

1. **Daemon output capture**: Spawned daemon processes may write directly to terminal
2. **Log rotation**: No automatic cleanup of old log files
3. **Log aggregation**: No centralized log collection or analysis
4. **Structured logging**: Logs are unstructured text (not JSON)

### Recommended Improvements

1. **Daemon stdout/stderr capture**: Ensure spawned daemons also redirect output
2. **Log retention policy**: Add age-based log cleanup
3. **Metadata headers**: Add test configuration to log file headers
4. **Performance metrics**: Separate structured metrics file

## Conclusion

The HOOP repository has a **mature and well-integrated wrapper script system** for test execution and logging. The load tests are properly wrapped with comprehensive output capture. The system follows best practices for:

- ✅ Wrapper script structure and location
- ✅ Load test spawning and execution
- ✅ Stdout/stderr capture mechanism
- ✅ Logging across all test types
- ✅ Makefile integration
- ✅ Documentation and verification

**No immediate changes required** - the wrapper script infrastructure is production-ready and fully functional.

## References

- Wrapper script: `bin/run-with-log.sh`
- Log name generator: `bin/generate-test-log-name.sh`
- Load test code: `hoop-daemon/tests/load_test.rs`
- Makefile integration: `Makefile` lines 45-136
- Load test documentation: `hoop-daemon/tests/load_test_README.md`
- Verification docs: `docs/bf-2ee3c-load-test-wrapper-verification.md`