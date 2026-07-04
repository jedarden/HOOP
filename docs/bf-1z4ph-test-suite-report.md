# Bead bf-1z4ph: Run full hoop-daemon test suite

## Task Completion Status: FAILED - Code does not compile

### Attempted Actions

1. ✅ Cleaned up lingering test processes before running tests
2. ❌ **Cannot run `cargo test -p hoop-daemon` - compilation errors prevent test execution**
3. ❌ **Cannot verify all tests pass - tests cannot run**
4. ✅ **Documenting failures (this file)**

### Compilation Errors Blocking Test Suite

The hoop-daemon crate does not compile. Multiple compilation errors across several test files:

#### 1. Missing `Arc` import in test fixtures (`api_stitch_decompose.rs`)
- Lines 1197-1223: 12+ instances of `Arc::new()` calls without `Arc` in scope
- Tests use `std::sync::Arc` but the import is missing from test module
- Error: `cannot find type Arc in this scope`

#### 2. Missing struct fields in test fixtures (`capacity.rs`)
- Lines 3226, 3266, 3310, 3354, 3398, 3442, 3486
- `CapacityMeterConfig` initialization missing required fields:
  - `accounts_file`
  - `opencode_dirs`
- Error: `missing fields accounts_file and opencode_dirs in initializer of capacity::CapacityMeterConfig`

#### 3. Missing function arguments in tests (`config_watcher.rs`)
- Lines 591, 617, 642, 679
- `ConfigWatcher::reload_config()` called with 4 arguments, requires 5
- Missing: `agent_config_changed_tx: Arc<Mutex<Option<broadcast::Sender<AgentConfigChanged>>>>`
- Error: `this function takes 5 arguments but 4 arguments were supplied`

#### 4. Unused import warning (`prompt_substitute.rs`)
- Line 15: `json` imported from `serde_json` but never used
- Warning: `unused import: json`

### Context

Per AGENTS.md, this is expected:
> **ACTUAL STATE (as of 2026-06-28): Phase 0 complete. Phase 1 in progress. `cargo build` FAILS (36 compilation errors).**

The repository is in a known broken state. The test suite cannot be run until the compilation errors are fixed.

### Recommendation

This bead (`bf-1z4ph`) should be blocked by a compilation fix bead. The test suite cannot verify "no regressions" because the code does not yet compile. The sequence should be:

1. Fix compilation errors (new bead or existing one)
2. Run `cargo test -p hoop-daemon` to verify baseline
3. Then use this bead to check for regressions

### Verification After Fix

Once compilation errors are fixed, run:

```bash
# Before running tests (per CLAUDE.md)
pkill -f 'hoop-[a-f0-9]{16,}$' && pkill -f 'hoop_daemon-[a-f0-9]{16,}$' && pkill -f 'testrepo/(bin|scripts)/' && pkill -9 -f 'build-script-build$' || true

# Run test suite
nix-shell --run 'cargo test -p hoop-daemon'

# Verify no leaked processes after tests
./bin/verify-hoop-test-processes.sh
```

### Conclusion

**Task cannot be completed.** The hoop-daemon test suite cannot run because the crate does not compile. This is a known state documented in AGENTS.md. The fix for test_add_pattern (the recent work) cannot be verified for regressions because the test environment is not functional.
