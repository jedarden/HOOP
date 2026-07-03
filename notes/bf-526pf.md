# bf-526pf: Clippy Verification - 75 Errors Remaining

## Summary

Ran `cargo clippy --workspace -- -D warnings` and found **75 errors** (80 compilation errors total, including 5 trait bound errors for `EnableTourRequest`).

## Verification Command
```bash
nix-shell -p pkg-config openssl --run "cargo clippy --workspace -- -D warnings 2>&1 | grep '^error:' | wc -l"
# Current output: 75
# Target: 0
```

## Error Categories

### 1. Missing `utoipa::ToSchema` derive (5 compilation errors - BLOCKS BUILD)
- **File:** `hoop-daemon/src/api_tour_project.rs:34`
- **Type:** `EnableTourRequest`
- **Issue:** Missing `#[derive(utoipa::ToSchema)]` on the struct
- **Impact:** This is a compilation error, not just a warning - it blocks the build

### 2. Unused Imports (~34 errors)
Multiple files have unused imports including:
- `PathBuf` in `accounts_config.rs`, `atomic_write.rs`
- `warn` in `accounts_config.rs`, `config_backup.rs`
- `State`, `Connection`, `params`, `Deserialize` in `api_bead_files.rs`
- `get`, `delete`, `put` in various API modules
- `RecommendedWatcher` in `api_skills.rs`
- `anyhow`, `bail`, `json` in various modules
- And many more...

### 3. Unused Variables (~30 errors)
Variables assigned but never used:
- Timing variables: `start`, `elapsed_ms` (should be `_start`, `_elapsed_ms`)
- `remote_addr`, `required_role` in `auth.rs`
- `timed_out` in `api_scripts.rs`, `api_skills.rs`
- `schedule`, `overlap_policy` in `script_scheduler.rs`
- `config`, `conn`, `workspace` parameters
- And many more...

### 4. Unnecessary `mut` (~6 errors)
Variables marked `mut` but never mutated:
- `conn` in `api_tour_project.rs:239`, `api_fix_patterns.rs:454`
- `shutdown_rx` in `lib.rs:3446`
- `gemini_dirs`, `opencode_dirs` in `capacity.rs`
- `shared_files`, `shared_labels` in `cross_project_propagation.rs`

## Action Required

1. **Fix the compilation error first:** Add `#[derive(utoipa::ToSchema)]` to `EnableTourRequest` in `hoop-daemon/src/api_tour_project.rs`
2. **Run auto-fix:** `cargo clippy --workspace --fix --allow-dirty --allow-staged`
3. **Manual fixes may be needed** for cases where clippy's auto-fix isn't appropriate

## Next Steps

This bead (`bf-526pf`) was to **verify** zero warnings remain. The verification failed - 75 errors remain.
A follow-up bead should be created to fix these errors systematically, starting with the compilation-blocking `utoipa::ToSchema` issue.
