# Test Environment Verification (bf-5lnvz)

## Date: 2026-07-04

## Task Summary
Verify the test environment is properly set up and that the `epoch_sync_invariant` test exists in `hoop-daemon`.

## Findings

### ✅ Environment Setup
- **nix-shell**: Available at `/run/current-system/sw/bin/nix-shell`
- **Dependencies**: `pkg-config` and `openssl` available via nix-shell
- **Test file exists**: `hoop-daemon/tests/epoch_sync_invariant.rs` (11.9 KB)

### ✅ Test Definitions Found
The `epoch_sync_invariant.rs` file contains 5 integration tests:

1. `test_epoch_sync_init_event_carrying_subscriptions` - Validates init event includes subscriptions
2. `test_epoch_sync_initial_snapshots_after_init` - Verifies snapshot events follow init
3. `test_epoch_sync_reconnect_wipes_and_rebuilds` - Core invariant: reconnect wipes stale state
4. `test_epoch_sync_init_is_always_first_message` - Init must be first on every connection
5. `test_epoch_sync_concurrent_connections` - Multiple connections each receive their own init

### ❌ Compilation Errors Block Test Listing
**CRITICAL**: The test cannot be listed via `cargo test --list` due to compilation errors in its dependency `integration_harness.rs`:

#### Error 1: Line 602 - Field name mismatch
```rust
// Current (broken):
Ok((base_url, handle.shutdown_notify, handle._temp_dir))

// Should be:
Ok((base_url, handle.shutdown_notify, handle.temp_dir))
```
The `DaemonHandle` struct has a public `temp_dir` field, not `_temp_dir`.

#### Error 2: Line 269 - Missing required field
```rust
// Current (incomplete):
Bead {
    id: id.to_string(),
    title: title.to_string(),
    description: None,
    status,
    priority: 0,
    issue_type: BeadType::Task,
    created_at: Utc::now(),
    updated_at: Utc::now(),
    created_by: "test".to_string(),
    dependencies: vec![],
```
Missing `workspace: String` field (required as of 2026-07-04).

## Acceptance Criterion Status
**PARTIALLY MET**: 
- ✅ Test environment properly configured (nix-shell + dependencies)
- ✅ Test target exists in filesystem (`hoop-daemon/tests/epoch_sync_invariant.rs`)
- ✅ Test contains 5 properly defined test functions
- ❌ `cargo test --list` fails due to compilation errors in `integration_harness.rs`

The acceptance criterion command:
```bash
nix-shell -p pkg-config openssl --run 'cargo test -p hoop-daemon --list | grep epoch_sync_invariant'
```
fails at compilation stage, but the test file itself exists and is well-structured.

## Dependencies
- `integration_harness.rs` - Provides `spawn_test_daemon()` used by all epoch_sync tests
- `tokio-tungstenite` - WebSocket client library
- `futures_util` - Stream utilities for WebSocket handling

## Impact
- These 5 tests cannot run until compilation errors are fixed
- All tests depend on `integration_harness.rs`, so this affects multiple test files
- The test environment itself is working (nix-shell, dependencies), but the test code has bit-rot

## Recommendations
1. Fix the `_temp_dir` → `temp_dir` field name at integration_harness.rs:602
2. Add `workspace: "test".to_string()` to the Bead initialization at integration_harness.rs:269
3. Re-verify with `cargo test -p hoop-daemon --test epoch_sync_invariant -- --list` after fixes
