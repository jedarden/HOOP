# Test Results: hoop-cli Unit Tests (Bead bf-1uygw)

## Date
2026-07-03

## Test Summary
All hoop-cli unit tests pass successfully.

## Test Execution
- Command: `cargo test` (from hoop-cli directory)
- Environment: Direct cargo test execution (nix-shell not required for tests)
- Duration: ~0.01s per test suite

## Test Results

### Test Suite 1: Library Tests (32 tests)
**Result: ok. 32 passed; 0 failed**

All project management tests passed:
- Project registration (add, remove, scan)
- Workspace discovery and validation
- Symlink handling and canonical path resolution
- Shorthand round-trip serialization
- Multi-workspace support

### Test Suite 2: Integration Tests (56 tests)
**Result: ok. 56 passed; 0 failed**

All integration tests passed:
- backup::tests::print_backup_metrics_* (2 tests)
- new::tests::parse_* (3 tests)
- restore::tests::* (17 tests)
- risk_patterns::tests::* (3 tests)
- projects::tests::* (32 tests - duplicated from lib tests)

### Total
- **88 tests passed**
- **0 tests failed**
- **0 tests ignored**
- **0 tests filtered out**

## Verification
- No test failures detected
- No regressions observed
- No lingering test subprocesses after completion
- All acceptance criteria met

## Notes
- Package name is `hoop` despite directory being `hoop-cli`
- Tests run successfully without nix-shell (no Node.js dependency for CLI tests)
