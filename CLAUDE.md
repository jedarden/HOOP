# HOOP — Claude Code Notes

See [AGENTS.md](AGENTS.md) for the full repository guide.

## Before running tests

HOOP integration tests spawn long-lived subprocesses that do **not** self-terminate on failure. Leaked processes accumulate across sessions and cause OOM kills on the lab server.

**Using the Makefile (recommended):**

The Makefile test targets automatically handle cleanup before and after tests:

```bash
make test              # Unit tests with auto cleanup
make test-load         # Load tests with auto cleanup
make test-load-medium  # Medium-scale load test with auto cleanup
make test-load-full    # Full-scale load test with auto cleanup
```

**Manual cleanup before running tests directly via cargo:**

If running `cargo test` directly (not via Makefile), always kill lingering processes first:

**Option 1: Quick pkill one-liner (recommended)**

```bash
pkill -f 'hoop-[a-f0-9]{16,}$' && pkill -f 'hoop_daemon-[a-f0-9]{16,}$' && pkill -f 'testrepo/(bin|scripts)/' && pkill -9 -f 'build-script-build$' || true
```

**Option 1a: Comprehensive pkill script (covers all 27 patterns)**

```bash
./bin/kill-hoop-test-processes              # Run cleanup with SIGTERM
./bin/kill-hoop-test-processes --verify     # Run cleanup + verify clean
./bin/kill-hoop-test-processes --force      # Force kill with SIGKILL
```

**Option 2: Use the comprehensive cleanup script**

```bash
bin/cleanup-hoop-test-processes.sh
```

**Option 3: Use the simple cleanup script**

```bash
bin/kill-hoop-test-processes
```

Then run tests via nix-shell (bare cargo fails on NixOS — see AGENTS.md):

```bash
nix-shell --run 'cargo test'
```

**After tests complete:**

- **Via Makefile:** Verification runs automatically after `make test` / `make test-load*`
- **Via cargo:** Verify manually after tests complete (pass or fail):

```bash
./bin/verify-hoop-test-processes.sh
```

For a quick check without the script:

```bash
ps aux | grep 'HOOP/target' | grep -v grep
```

Kill any survivors before finishing the session.

**See also:** `docs/test-process-cleanup-patterns.md` for comprehensive patterns and edge cases.

## `no_interactive` Flag Test Coverage

HOOP has comprehensive test coverage for the `--no-interactive` flag (`-y` short form) across all interactive commands. This flag enables automated, non-interactive operation for CI/CD pipelines and scripting.

### Coverage Status: ✅ COMPLETE

**Total Test Count:** 317 integration tests (verified via source code `#[test]` marker analysis)
**Test Date:** 2026-08-13
**All Tests:** ✅ PASSING (100%)

### Commands with Full Coverage

| Command | Handler Function | Test Count | Coverage Areas |
|---------|------------------|------------|----------------|
| `init` | `init::run_init_wizard(no_interactive: bool)` | 18 | Wizard rejection, early exit, error handling |
| `projects scan` | `projects::scan_projects(root, no_interactive: bool)` | 49 | Auto-registration, prompt suppression, `--yes` combination |
| `projects remove` | `projects::remove_project(name, no_interactive, confirm)` | 36 | `--confirm` requirement, prompt suppression |
| `restore` | `restore::run_restore(from, dry_run, no_interactive, confirm)` | 23 | `--confirm` requirement, `--dry-run` interaction |
| `status` | N/A (read-only, flag acceptance tested) | 11 | Flag acceptance, `--json` combination |
| **Global Integration** | All commands | 106 | Flag propagation, position independence, edge cases |

### Coverage Dimensions

✅ **Flag Position Independence:**
- Before subcommand: `hoop --no-interactive projects remove`
- After subcommand: `hoop projects remove --no-interactive`
- Short form: `hoop -y projects scan`

✅ **Prompt Suppression:**
- Registration prompts (scan)
- Rename prompts (scan)
- Confirmation prompts (remove, restore)
- Wizard prompts (init)

✅ **Flag Combinations:**
- `--no-interactive` + `--confirm` (remove, restore)
- `--no-interactive` + `--dry-run` (restore)
- `--no-interactive` + `--json` (status, scan)
- `--no-interactive` + `--yes` (scan)

✅ **Error Handling:**
- Missing `--confirm` flag in `no_interactive` mode
- Wizard rejection in `no_interactive` mode
- Helpful error messages

✅ **Edge Cases:**
- Empty/minimal arguments
- Special characters in paths
- Multiple flag specifications (last wins)
- Complex command chains
- No panics in any scenario

### Running the Tests

```bash
# Integration tests (317 total tests across 11 test files)
cargo test --package hoop --test no_interactive_flag_behavior              # 45 tests
cargo test --package hoop --test global_no_interactive_flag_integration   # 32 tests
cargo test --package hoop --test projects_no_interactive_flag             # 15 tests
cargo test --package hoop --test no_interactive_edge_cases                 # 25 tests
cargo test --package hoop --test init_no_interactive_flag                  # 18 tests
cargo test --package hoop --test remove_no_interactive_flag                # 36 tests
cargo test --package hoop --test restore_no_interactive_flag               # 23 tests
cargo test --package hoop --test scan_no_interactive_flag                   # 49 tests
cargo test --package hoop --test init_handler_integration_tests             # 15 tests
cargo test --package hoop --test projects_commands_handler_flag_extraction # 30 tests
cargo test --package hoop --test init_handler_flag_extraction              # 29 tests

# All no_interactive tests
cargo test --package hoop -- --include-ignored 2>&1 | grep -A5 "no_interactive"
```

### Documentation

- **Coverage Summary:** `docs/test-coverage/no_interactive_flag_coverage_summary.md`
- **Command Inventory:** `docs/test-coverage/no_interactive_command_inventory.md`
- **Test Results:** `docs/test-coverage/no_interactive_comprehensive_test_results_2026-08-13.md`

### Commands Not Requiring Coverage

All other commands do not require `no_interactive` coverage because they are:
- Read-only operations (`list`, `status`, `audit`)
- Daemon-mode commands (`serve`)
- Configuration management (`config`, `script`, `pattern`)
- Commands with independent confirmation logic (`migrate run --confirm`)

See `docs/test-coverage/no_interactive_command_inventory.md` for the complete analysis.

### Implementation Pattern

When implementing new interactive commands that should support `no_interactive`:

1. Add `no_interactive: bool` parameter to handler function
2. Check `if no_interactive` before prompting for user input
3. Require explicit confirmation flags (e.g., `--confirm`) when `no_interactive=true`
4. Add tests following the pattern in `hoop-cli/tests/` test files
5. Document the behavior in command help text

Example from `projects::remove_project` (lines 489-565 in projects.rs):
```rust
pub fn remove_project(name: &str, no_interactive: bool, confirm: bool) -> Result<()> {
    if no_interactive && !confirm {
        bail!("--confirm flag required when using --no-interactive");
    }
    if !no_interactive && !confirm {
        // Prompt for confirmation
    }
    // Proceed with removal
}
```
