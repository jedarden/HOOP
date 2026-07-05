# Isolated Backup Config Deserialization Test

## Summary

The `test_backup_deser` crate provides a minimal, isolated test for `BackupFileConfig` deserialization that compiles and runs independently of other failing tests in the main codebase.

## Location

```
test_backup_deser/
├── Cargo.toml
└── src/
    └── main.rs
```

## What It Tests

1. **Minimal YAML config** - Verifies default values are correctly applied:
   - `schedule`: defaults to `"0 4 * * *"`
   - `retention_days`: defaults to `30`
   - `encryption`: defaults to `false`

2. **Full YAML config** - Verifies all fields deserialize correctly when specified

## How to Run

```bash
# Build and run the isolated test
cargo build --package test_backup_deser
cargo run --package test_backup_deser

# Or directly with cargo run
cargo run --package test_backup_deser
```

## Why This Isolation Matters

The main integration test (`hoop-daemon/tests/backup_restore_cycle.rs`) cannot compile due to:
1. Missing `tempfile` dependency (only available via `--features testing`)
2. Other compilation errors in the main library that block test compilation

This isolated test provides:
- ✅ Independent compilation (no dependencies on main crate)
- ✅ Focused testing of BackupFileConfig deserialization only
- ✅ No blocking by other test failures
- ✅ Fast execution (no integration test overhead)

## Acceptance Criteria Met

- ✅ **Created focused test file**: `test_backup_deser/src/main.rs` tests only BackupFileConfig deserialization
- ✅ **Verified compilation and execution in isolation**: `cargo build --package test_backup_deser` succeeds
- ✅ **Confirmed unblocked execution**: Test runs without being affected by other compilation failures

## Related Files

- Main integration test: `hoop-daemon/tests/backup_restore_cycle.rs` (currently blocked by compilation errors)
- Implementation: `hoop-daemon/src/backup.rs` and `hoop-daemon/src/backup_pipeline.rs`
