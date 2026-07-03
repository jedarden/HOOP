# Epoch Sync Test Failure Investigation (bf-4j4ys)

## Executive Summary

The epoch sync invariant test (`epoch_sync_invariant`) is **not running at all** due to **compilation errors** in the test harness. The test code fails to compile, so no runtime test execution occurs.

## Root Cause: Compilation Failures

### 1. Missing `workspace` field in Bead struct initialization

**Location:** `hoop-daemon/tests/integration_harness.rs:269`

**Error:**
```
error[E0063]: missing field `workspace` in initializer of `Bead`
   --> hoop-daemon/tests/integration_harness.rs:269:5
    |
269 |     Bead {
    |     ^^^^ missing `workspace`
```

**Explanation:**
The `Bead` struct in `hoop-daemon/src/lib.rs` was updated to include a `workspace` field (line 188):
```rust
/// Workspace path assigned by HOOP at load time — not stored in issues.jsonl
#[serde(skip_deserializing, default)]
pub workspace: String,
```

However, the test helper function `create_mock_bead()` in `integration_harness.rs` (lines 269-281) doesn't include this field when creating mock beads, causing compilation to fail.

**Fix needed:**
Add `workspace: String::default()` (or similar) to the Bead initialization in `create_mock_bead()`.

---

### 2. Incorrect field name `handle._temp_dir`

**Location:** `hoop-daemon/tests/integration_harness.rs:602`

**Error:**
```
error[E0609]: no field `_temp_dir` on type `DaemonHandle`
   --> hoop-daemon/tests/integration_harness.rs:602:50
    |
602 |     Ok((base_url, handle.shutdown_notify, handle._temp_dir))
    |                                                  ^^^^^^^^^ unknown field
    |
help: a field with a similar name exists
    |
602 -     Ok((base_url, handle.shutdown_notify, handle._temp_dir))
602 +     Ok((base_url, handle.shutdown_notify, handle.temp_dir))
```

**Explanation:**
The `DaemonHandle` struct (line 582-585) has a public field named `temp_dir`, not `_temp_dir`. The test code tries to access `_temp_dir` which doesn't exist.

**Current struct definition:**
```rust
pub struct DaemonHandle {
    shutdown_notify: Arc<tokio::sync::Notify>,
    pub temp_dir: TempDir,
}
```

**Fix needed:**
Change `handle._temp_dir` to `handle.temp_dir` on line 602.

---

### 3. Tungstenite API change: `Message::Text` expects `Utf8Bytes` instead of `String`

**Locations:** 
- `hoop-daemon/tests/integration_harness.rs:862`
- `hoop-daemon/tests/integration_harness.rs:1105`
- `hoop-daemon/tests/integration_harness.rs:1202`
- `hoop-daemon/tests/integration_harness.rs:1214`
- `hoop-daemon/tests/integration_harness.rs:1222`

**Error pattern:**
```
error[E0308]: mismatched types
   --> hoop-daemon/tests/integration_harness.rs:862:13
    |
861 |         .send(tokio_tungstenite::tungstenite::Message::Text(
    |               --------------------------------------------- arguments to this enum variant are incorrect
862 |             subscribe_msg.to_string(),
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^ expected `Utf8Bytes`, found `String`
     |
note: tuple variant defined here
   --> /home/coding/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tungstenite-0.26.2/src/protocol/message.rs:160:5
    |
160 |     Text(Utf8Bytes),
    |     ^^^^
help: call `Into::into` on this expression to convert `std::string::String` into `Utf8Bytes`
    |
862 |             subscribe_msg.to_string().into(),
    |                                      +++++++
```

**Explanation:**
The `tokio_tungstenite` crate upgraded to tungstenite 0.26.2, which changed the `Message::Text` variant to use `Utf8Bytes` instead of `String`. This is a breaking change in the WebSocket API.

**Fix needed:**
Add `.into()` to convert `String` to `Utf8Bytes` at all 5 locations:
- Line 862: `subscribe_msg.to_string().into()`
- Line 1105: `subscribe_msg.to_string().into()`
- Line 1202: `"{invalid json}".to_string().into()`
- Line 1214: `unknown_msg.to_string().into()`
- Line 1222: `"".to_string().into()`

---

## Impact

**None of the epoch sync invariant tests can run** until these compilation errors are fixed. The test binary cannot be built, so no runtime behavior can be verified.

## Next Steps

1. Fix the 7 compilation errors listed above
2. Rebuild the test binary
3. Run the test again to see if there are any *runtime* failures
4. Document any additional issues found at runtime

## Test Command

```bash
nix-shell -p pkg-config openssl --run 'cargo test -p hoop-daemon --test epoch_sync_invariant -- --nocapture'
```

---

**Investigated:** 2026-07-03  
**Bead ID:** bf-4j4ys  
**Conclusion:** Test cannot compile due to API drift and missing fields in test fixtures.
