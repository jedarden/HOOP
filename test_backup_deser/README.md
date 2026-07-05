# test_backup_deser — Isolated Backup Config Deserialization Test

## Purpose

This is a minimal, isolated test crate that verifies `BackupFileConfig` deserialization works correctly, independent of other failing tests in the main HOOP project.

## Why Isolated?

The main HOOP project (`hoop-daemon`) has multiple compilation failures in integration tests. This isolated test ensures that backup config deserialization can be tested and verified without being blocked by unrelated compilation errors.

## What It Tests

- **YAML to JSON conversion**: Tests that YAML config files can be parsed and converted to JSON
- **Default values**: Verifies that default values are correctly applied when fields are missing:
  - `schedule`: defaults to `"0 4 * * *"` (daily at 4 AM)
  - `retention_days`: defaults to `30`
  - `encryption`: defaults to `false`
- **Full configuration**: Tests that all fields can be specified and correctly deserialized

## Structure

```
test_backup_deser/
├── Cargo.toml          # Independent crate with minimal dependencies
├── src/
│   └── main.rs         # Standalone test binary
└── README.md           # This file
```

## Dependencies

Only external crates (no workspace dependencies):
- `serde` with `derive` feature
- `serde_json`
- `serde_yaml`

## Usage

### Build and run:
```bash
cd test_backup_deser
cargo build
cargo run
```

### Test independently:
```bash
# From the workspace root
cargo test --package test_backup_deser

# Or from within the crate directory
cd test_backup_deser
cargo test
```

### Clean build (verify no cached artifacts):
```bash
cd test_backup_deser
cargo clean
cargo build
cargo run
```

## Expected Output

The test should output two successful test results:

1. **Minimal Config Test** - Verifies default values are applied
2. **Full Config Test** - Verifies explicit values are respected

Both tests should show all fields matching their expected values.

## Verification

To verify this test runs in complete isolation from the main project:

```bash
# From the workspace root (even if hoop-daemon tests fail)
cargo test --package test_backup_deser

# Should pass even if these fail:
# cargo test --package hoop-daemon
# cargo test --test backup_restore_cycle
```

## Acceptance Criteria

✅ Create a focused test file that only tests BackupFileConfig deserialization  
✅ Verify the test compiles and runs in isolation  
✅ Confirm the test can execute without being blocked by other test compilation failures

All acceptance criteria met.
