# bf-64zvy: tempfile Dependency Investigation Summary

## Current State

### hoop-daemon/Cargo.toml
```toml
[dependencies]
tempfile = "3"  # Line 67 - NOT optional, NOT gated

[features]
testing = []     # Line 11 - exists but not used for tempfile
```

### Other Workspace Crates

| Crate | Location | Status |
|-------|----------|--------|
| hoop-schema | `[dev-dependencies]` | ✅ Correct (only test usage) |
| hoop-mcp | `[dev-dependencies]` | ✅ Correct (no usage found) |
| hoop-cli | `[dependencies]` | ✅ Correct (production usage in `new.rs`) |

## tempfile Usage Analysis

### hoop-daemon (97 references across 24 files)

**All usage is in `#[cfg(test)]` or `#[cfg(any(test, feature = "testing"))]` modules:**

- `integration_harness.rs` - cfg-gated, uses `TempDir`
- `load_test.rs` - cfg-gated  
- 22 other files - all in `#[cfg(test)]` test modules

**Module gating in lib.rs (lines 82-87):**
```rust
#[cfg(any(test, feature = "testing"))]
pub mod integration_harness;
#[cfg(any(test, feature = "testing"))]
pub mod load_test;
```

### hoop-cli

**PRODUCTION usage found:**
- `new.rs:86-89` - Creates temporary draft file for `$EDITOR`
  - This is user-facing production code
  - Cannot move to dev-dependencies

**Test usage:**
- `restore.rs` - Multiple `#[test]` functions
- `projects.rs` - Multiple `#[test]` functions

## Testing Feature Usage

### Makefile
```makefile
cargo test --lib --features testing --verbose
```

The `testing` feature is used when running tests to enable `integration_harness` and `load_test` modules.

### Why Not Just Dev-Dependencies?

**Integration harness pattern:** The `integration_harness` module is:
- Compiled into lib.rs (not a separate test crate)
- Public so integration tests can use it: `use hoop_daemon::integration_harness`
- Cfg-gated behind `#[cfg(any(test, feature = "testing"))]`

This pattern means:
- Regular builds exclude the harness (feature not active)
- `cargo test` includes it (test cfg)
- External test harnesses need `--features testing` to access it

## Recommended Fix

**Move tempfile to dev-dependencies for hoop-daemon only:**

```toml
[dependencies]
# Remove tempfile from here

[dev-dependencies]
tempfile = "3"  # Add here
```

**Why this works:**
1. All hoop-daemon tempfile usage is in `#[cfg(test)]` code
2. Test code automatically gets access to dev-dependencies
3. Integration harness is already cfg-gated so it won't leak into production builds
4. No risk to production code (no production usage in hoop-daemon)

**Do NOT change hoop-cli:**
- `new.rs` uses tempfile in production code
- Must remain as regular dependency

## Files Using Tempfile in hoop-daemon

```
api_notes.rs              (#[cfg(test)])
api_prompts.rs            (#[cfg(test)])
api_skills.rs            (#[cfg(test)])
atomic_write.rs          (#[cfg(test)])
attachments.rs           (#[cfg(test)])
attachment_sync.rs       (#[cfg(test)])
backup_pipeline.rs        (#[cfg(test)])
capacity.rs              (#[cfg(test)])
config_backup.rs         (#[cfg(test)])
config_watcher.rs        (#[cfg(test)])
fleet.rs                 (#[cfg(test)])
integration_harness.rs   (#[cfg(any(test, feature = "testing"))])
load_test.rs             (#[cfg(any(test, feature = "testing"))])
net_diff.rs              (#[cfg(test)])
parse_jsonl_safe.rs      (#[cfg(test)])
path_security.rs        (#[cfg(test)])
projects.rs              (#[cfg(test)])
sessions.rs              (#[cfg(test)])
shutdown.rs              (#[cfg(test)])
stitch_reconstruction.rs (#[cfg(test)])
stitch_traversal.rs      (#[cfg(test)] - has #[cfg(test)] import)
template_library.rs      (#[cfg(test)])
uploads.rs               (#[cfg(test)])
worker_ack.rs            (#[cfg(test)])
```

## Verification

To verify the fix works:
```bash
# Build without testing feature - should not include tempfile
cargo build --release

# Run tests - should work with dev-dependency tempfile
cargo test

# Run integration harness with testing feature
cargo test --features testing
```
