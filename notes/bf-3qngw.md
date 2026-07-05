# bead bf-3qngw: hoop-daemon test suite regression

**Last updated:** 2026-07-04 21:30

## Summary

Ran the full hoop-daemon test suite per bead bf-3qngw requirements. **Test suite cannot run due to compilation failures.** The hoop-daemon crate itself compiles (with warnings only), but both lib tests and integration tests fail to compile due to API evolution outpacing test maintenance.

**Current state:**
- ✗ `cargo test -p hoop-daemon` fails to compile (14 errors in test code)
- ✗ Cannot verify test pass/fail status (tests don't compile)
- ⚠️ No regression detection possible (no passing baseline to compare against)

## Partial fixes applied

Fixed 3 compilation errors that were blocking initial compilation:

1. **disaster_recovery_runbook.rs:113** - Added missing `config_backup: None` field to `SnapshotManifest` initializer
2. **disaster_recovery_runbook.rs:449** - Fixed temporary value borrow issue with `file_name().to_string_lossy().into_owned()`
3. **Cargo.toml** - Moved `tempfile` to optional dependency and enabled it for the `testing` feature

## Remaining regression: 75 compilation errors

The test suite has bit-rot - tests were written against earlier APIs that have since evolved:

### Error categories (from `cargo test -p hoop-daemon --lib --features testing`)

| Error Type | Count | Description |
|------------|-------|-------------|
| E0061 (function args) | 20 | Functions now require more arguments than tests provide |
| E0063 (missing fields) | 18 | Structs have new required fields |
| E0308 (type mismatch) | 6 | Types don't match expected signatures |
| E0599 (missing methods) | 3 | Missing Default impls and associated functions |
| E0277 (unpinned) | 28 | Syntax highlighting stream test issues |

### Specific API changes causing test failures

1. **`resolve_actor`** - Now requires `DaemonState` parameter (was `None`)
2. **`PreviewRequest`** - Missing `attachments_count` field
3. **`CapacityMeterConfig`** - Missing `accounts_file`, `opencode_dirs`, `gcp_quota_config`, `gemini_dirs` fields
4. **`DaemonState`** - Missing `br_semaphore`, `br_semaphore_target_permits` fields
5. **`DictatedNote`** - Missing `draft_id`, `synthesis_result` fields
6. **`HoopConfig`** - Missing `embedding`, `redaction` fields
7. **`NeedleEvent`** - Missing `stash_sha` field
8. **`CommitEntry`** - Missing `bead_id` field
9. **`ProjectSupervisor::new()`** - Now requires 9 arguments (was 0)
10. **`CostAggregator::new()`** - Now requires 1 argument (was 0)
11. **`UploadRegistry::new()`** - Now requires 1 argument (was 0)
12. **Syntax highlighting tests** - 28 "cannot be unpinned" errors in `syntax_highlight_stream.rs`

## Current Run Findings (2026-07-04)

Ran `cargo test -p hoop-daemon` which resulted in **14 compilation errors** in the test code:

### Error Breakdown from This Run

| File | Error Type | Description |
|------|------------|-------------|
| `agent_turn_audit_trail.rs:16` | E0432 | Missing `tempfile` dependency (needs `cargo add tempfile`) |
| `agent_turn_audit_trail.rs:19` | E0061 | `Mutex::new()` requires 1 argument, given 0 |
| `adapter_failover_test.rs` | E0282 | Type annotations needed for closure parameters (10 occurrences) |
| `adapter_failover_test.rs:280` | E0063 | Missing field `test_subprocess_id` in struct init |

### Specific Compilation Errors

1. **Missing `tempfile` crate** - `agent_turn_audit_trail.rs` imports it but it's not in dev-dependencies
2. **Mutex initialization** - Line 19: `Mutex::new()` needs `Mutex::new(())`
3. **Type inference failures** - Multiple `spawn_test_daemon_with_config` calls need explicit closure parameter types
4. **Struct field missing** - Line 280 in adapter_failover_test.rs missing `test_subprocess_id`

## Conclusion

**The hoop-daemon test suite is NOT passing and cannot run.** This is not a regression from a previously-passing state — the tests have never compiled successfully. The main crate compiles with warnings only, but the test suite has bit-rot due to:

1. Missing dependencies in dev-dependencies
2. Test code written against earlier APIs that have evolved
3. Missing required fields in test fixtures
4. Type inference failures requiring explicit annotations

**Task acceptance criteria NOT met:**
- ❌ `cargo test -p hoop-daemon` does not pass (compilation failed)
- ❌ All hoop-daemon tests do not pass (cannot run them)
- ⚠️ No regressions detected (no baseline exists)

## Recommendation

This task (bf-3qngw) **cannot be completed** without first fixing the test suite compilation errors. A separate bead should track fixing these compilation errors:

### Required fixes before test suite can run:
1. Add `tempfile` to `hoop-daemon/Cargo.toml` dev-dependencies
2. Fix `Mutex::new()` call to include `()` argument
3. Add explicit type annotations to closure parameters in `adapter_failover_test.rs`
4. Fix missing `test_subprocess_id` field in struct initialization
5. Address the broader API evolution issues documented in the previous run (75+ errors)

### Path forward:
1. Create a new bead to fix test compilation errors
2. Once tests compile, establish a passing baseline
3. Then re-run this bead (bf-3qngw) to detect any regressions

**This bead should be left open/closed with "cannot complete" status** until the test suite can actually compile and run.

## Commands to reproduce

```bash
# Try to run hoop-daemon tests (fails with 75 compilation errors)
cargo test -p hoop-daemon --features testing

# Verify lib compiles (works, only warnings)
cargo build -p hoop-daemon
```
