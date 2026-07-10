# Task bf-glos2: Enable tempfile for test builds

## Status: ✅ ALREADY COMPLETED

This task was already completed in commit `26dd265 deps(bf-i849k): Add tempfile dependency for test infrastructure`.

## Verification Results

### 1. tempfile is in dev-dependencies ✅
`hoop-daemon/Cargo.toml` line 75:
```toml
[dev-dependencies]
tempfile = "3"
```

### 2. lib-test build has ZERO tempfile errors ✅
Ran `nix-shell --run 'cargo test --lib --no-run'` and verified:
- No E0433 errors ("cannot find crate tempfile")
- No E0432 errors (unresolved import tempfile)
- The 97-error cluster mentioned in the task is **eliminated**

### 3. Production build doesn't include tempfile ✅
Verified that `tempfile` is NOT in `[dependencies]` section:
```bash
$ grep -A1 "^\[dependencies\]" hoop-daemon/Cargo.toml | grep tempfile
# (no output - tempfile not in main dependencies)
```

### 4. No new warnings introduced ✅
The build output shows only pre-existing warnings (unused imports, unused mut), all unrelated to the tempfile dependency change.

## Root Cause (from task description)
The original issue was that `tempfile = { version = "3", optional = true }` was gated behind a `testing` feature, but `cargo test --lib` didn't enable that feature, causing 97 compilation errors.

## Solution Applied
Moved `tempfile` to `[dev-dependencies]` (non-optional) so it's automatically available to all test builds.

## Test Code Usage
tempfile is used extensively in test code across multiple files:
- `hoop-daemon/src/template_library.rs` (5 uses)
- `hoop-daemon/src/path_security.rs`
- `hoop-daemon/src/load_test.rs` (2 uses)
- `hoop-daemon/src/stitch_traversal.rs`
- `hoop-daemon/src/attachments.rs`

All of these now compile successfully without tempfile-related errors.
