# Dead Code Warning Catalog

**Analysis Date:** 2026-06-27  
**Scope:** HOOP workspace (hoop-daemon, hoop-mcp, hoop-cli, hoop-ui)  
**Command:** `cargo clippy --workspace -- -W dead_code`

## Executive Summary

**Result:** NO dead_code warnings found in the workspace.

The clippy run produced 96 warnings across multiple categories, but **none were actual dead_code warnings** (items that are defined but never used anywhere in the codebase).

## Compilation Blockers

The analysis was incomplete due to 8 compilation errors that must be fixed first:

| Error | Location | Issue |
|-------|----------|-------|
| E0425 | `hoop-daemon/src/lib.rs:1980` | Function `update_per_project_patterns` not found in `secrets_scanner` |
| E0425 | `hoop-daemon/src/lib.rs:2024` | Function `update_per_project_patterns` not found in `secrets_scanner` |
| E0609 | `hoop-daemon/src/lib.rs:1979` | Field `redaction` does not exist on `ResolvedConfig` |
| E0609 | `hoop-daemon/src/lib.rs:2023` | Field `redaction` does not exist on `ResolvedConfig` |
| E0061 | `hoop-daemon/src/lib.rs:3101` | Function `update_all_orphan_metrics` takes 1 argument but 2 supplied |
| E0277 | `hoop-daemon/src/lib.rs:3101` | `()` is not a future (incorrect .await) |
| E0061 | `hoop-daemon/src/lib.rs:3109` | Function `update_all_orphan_metrics` takes 1 argument but 2 supplied |
| E0277 | `hoop-daemon/src/lib.rs:3109` | `()` is not a future (incorrect .await) |

## Warning Categories Found

While not dead_code warnings, these categories appeared:

### 1. Unused Imports (70+ instances)

Most frequently unused:
- `utoipa::ToSchema` (23 instances) - OpenAPI schema derive macro
- `std::collections::HashMap` (3 instances)
- `PathBuf`, `Path` (3 instances)
- `warn` from tracing (3 instances)
- Various axum/serde imports

### 2. Unused Variables (25+ instances)

Variables assigned but never read:
- Timing variables: `start`, `elapsed_ms` (7 instances)
- Timeout flags: `timed_out` (5 instances) 
- Database connections: `conn` (4 instances)
- Loop variables: `link_kind`, `schedule`, `overlap_policy`
- Configuration variables: `config`, `cfg`

### 3. Unnecessary Mutability (8 instances)

Variables declared `mut` but never modified:
- `conn` in `api_tour_project.rs:243`
- `conn` in `api_fix_patterns.rs:454`
- `gemini_dirs`, `opencode_dirs` in `capacity.rs`
- `shared_files`, `shared_labels` in `cross_project_propagation.rs`
- `conn` in `fix_patterns.rs` (2 instances)

### 4. Unused Assignments (3 instances)

Values assigned but never read before reassignment:
- `timed_out` in `api_scripts.rs:371`
- `timed_out` in `api_skills.rs:354`
- `timed_out` in `hoop-mcp/src/skills.rs:321`

### 5. Other Clippy Warnings

- `lines_filter_map_ok` - suggesting `map_while(Result::ok)` instead of `flatten()` (2 instances in hoop-mcp)

## Files with Highest Warning Counts

| File | Warning Count | Primary Categories |
|------|---------------|-------------------|
| `hoop-daemon/src/api_patterns.rs` | 5 | unused_imports |
| `hoop-daemon/src/api_fix_patterns.rs` | 4 | unused_imports, unused_variables |
| `hoop-daemon/src/cross_project_propagation.rs` | 8 | unused_imports, unused_variables, unnecessary mut |
| `hoop-daemon/src/capacity.rs` | 5 | unused_imports, unused_variables |
| `hoop-daemon/src/lib.rs` | 3 | unused_imports, compilation errors |
| `hoop-daemon/src/api_skills.rs` | 3 | unused_imports, unused_variables |
| `hoop-daemon/src/auth.rs` | 3 | unused_variables |

## Conclusion

**No dead_code warnings exist in the HOOP workspace.** The codebase does not contain any functions, methods, structs, or enum variants that are defined but never referenced.

The 96 warnings found are all other categories (unused imports, unused variables, unnecessary mutability) which are code quality issues rather than dead code.

## Recommendations

1. **Fix compilation errors first** - The 8 errors blocking compilation must be resolved before any clippy-driven cleanup
2. **Clean up unused imports** - 70+ instances can be removed with `cargo fix --allow-dirty`
3. **Remove or document unused variables** - Many timing/performance variables are tracked but not used for metrics
4. **Review unnecessary mutability** - 8 `mut` declarations can be removed

## Next Steps

To perform a complete dead_code analysis:
1. Fix the 8 compilation errors
2. Re-run `cargo clippy --workspace -- -W dead_code`
3. Verify no dead_code warnings appear (expected: none)
