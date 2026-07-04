# Epoch Sync Test Failure Investigation (Bead bf-5jpx7)

## Summary

The epoch sync invariant test (`epoch_sync_invariant.rs`) **does not fail at runtime** — it **fails to compile**. The test was never executed due to compilation errors in the test harness API.

## Root Cause: API Mismatch

The test harness API changed but the tests were not updated to match:

### Actual Function Signature
```rust
// In integration_harness.rs
pub async fn spawn_test_daemon() -> anyhow::Result<(String, DaemonHandle)>
```

### Test Code Expectation (OUTDATED)
```rust
// In epoch_sync_invariant.rs
let (base_url, _shutdown, _temp_dir) = spawn_test_daemon()
    .await
    .expect("Failed to spawn test daemon");
```

The test expects a **3-tuple** `(base_url, shutdown, temp_dir)` but the function returns a **2-tuple** `(String, DaemonHandle)`. The `DaemonHandle` struct encapsulates both the shutdown notification and temp_dir cleanup via its `Drop` impl.

## Specific Compilation Errors

### Error 1: Type Mismatch (E0308)
```
error[E0308]: mismatched types
  --> hoop-daemon/tests/epoch_sync_invariant.rs:25:9
   |
25 |     let (base_url, _shutdown, _temp_dir) = spawn_test_daemon()
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^--- this expression has type `(std::string::String, DaemonHandle)`
   |         |
   |         expected a tuple with 2 elements, found one with 3 elements
   |
   | expected tuple `(String, DaemonHandle)`
   | found tuple `(_, _, _)`
```

This occurs at **5 locations** in `epoch_sync_invariant.rs`:
- Line 25: `test_epoch_sync_init_event_carrying_subscriptions`
- Line 76: `test_epoch_sync_init_event_authoritative_subscriptions`
- Line 146: `test_epoch_sync_reconnect_resets_client`
- Line 243: `test_epoch_sync_event_ordering`
- Line 291: `test_epoch_id_monotonicity`

### Error 2: Double-Await Bug (E0277)
```
error[E0277]: `(std::string::String, DaemonHandle)` is not a future
   --> hoop-daemon/tests/integration_harness.rs:709:10
    |
709 |         .await
    |          ^^^^^ `(std::string::String, DaemonHandle)` is not a future
```

The function `spawn_test_daemon()` is already `async` and returns `Result<(String, DaemonHandle)>` when awaited. The correct pattern is:
```rust
let (base_url, _handle) = spawn_test_daemon().await?;
```

But some tests incorrectly do:
```rust
let (base_url, _handle) = spawn_test_daemon().await?.await // WRONG: double-await
```

## Test Harness Architecture

The current `DaemonHandle` encapsulates cleanup:
```rust
struct DaemonHandle {
    shutdown_notify: Arc<tokio::sync::Notify>,
    temp_dir: tempfile::TempDir,
}

impl Drop for DaemonHandle {
    fn drop(&mut self) {
        // Cleanup temp_dir when handle dropped
    }
}
```

## Impact

- **5 integration tests** in `epoch_sync_invariant.rs` cannot run
- **12+ other tests** in `integration_harness.rs` have the double-await bug
- The epoch sync invariants have **never been verified** because the tests never compiled

## Fix Required

Update all test call sites from:
```rust
let (base_url, _shutdown, _temp_dir) = spawn_test_daemon()
    .await
    .expect("Failed to spawn test daemon");
```

To:
```rust
let (base_url, _handle) = spawn_test_daemon()
    .await
    .expect("Failed to spawn test daemon");
```

And fix the double-await bugs in `integration_harness.rs` by removing the redundant `.await`.

## Test Output

```
error: could not compile `hoop-daemon` (test "epoch_sync_invariant") due to 43 previous errors; 2 warnings emitted
```

**No runtime panic, assertion, or timeout occurred — the test executable was never built.**
