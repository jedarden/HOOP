# AlreadyExists Test Failure Analysis (Bead bf-4mtbo)

## Executive Summary

**Root Cause:** The AlreadyExists tests in `hoop-daemon/src/file_io_error.rs` **cannot run** due to broader compilation errors in the HOOP codebase. These are **compilation failures**, not runtime test failures.

**Failure Type:** COMPILATION ERRORS (preventing test execution)

**Affected Tests:**
1. `test_classify_io_error_already_exists` (line 783)
2. `test_create_file_with_context_already_exists` (line 927)
3. `test_create_file_exclusive_with_context_already_exists` (line 950)
4. `test_create_dir_with_context_already_exists` (line 977)
5. `test_create_dir_all_with_context_already_exists` (line 1026)

## Detailed Analysis

### Test Code Quality

The AlreadyExists tests themselves are **correctly written** with no syntax errors:
- Proper test structure
- Correct assertions for AlreadyExists behavior
- Appropriate use of error messages
- Good coverage of AlreadyExists scenarios

### Compilation Blockers

The tests cannot compile due to these broader issues:

#### 1. Missing Module Declarations (api_beads.rs:1162-1166)
```
error[E0433]: cannot find module or crate `template_library` in this scope
error[E0433]: cannot find module or crate `api_prompts` in this scope
error[E0433]: cannot find module or crate `api_notes` in this scope
error[E0433]: cannot find module or crate `api_skills` in this scope
error[E0433]: cannot find module or crate `api_scripts` in this scope
```

**Impact:** Blocks compilation of the entire hoop-daemon lib, including file_io_error tests.

**Fix Required:** Add module declarations to `hoop-daemon/src/lib.rs`:
```rust
mod template_library;
mod api_prompts;
mod api_notes;
mod api_skills;
mod api_scripts;
```

#### 2. Missing Macro Import (prompt_substitute.rs:521, 542)
```
error: cannot find macro `json` in this scope
```

**Impact:** Blocks compilation of prompt_substitute module.

**Fix Required:** Add to `hoop-daemon/src/lib.rs` or file:
```rust
use serde_json::json;
```

#### 3. Struct Initialization Errors

**DaemonState missing fields (api_stitch_decompose.rs:1200):**
```
error[E0063]: missing fields `br_semaphore` and `br_semaphore_target_permits` in initializer of `DaemonState`
```

**CapacityMeterConfig missing fields (capacity.rs:3137, 3229, 3253, 3293):**
```
error[E0063]: missing fields `accounts_file`, `gcp_quota_config` and `opencode_dirs` in initializer of `capacity::CapacityMeterConfig`
```

**DictatedNote missing fields (dictated_notes.rs:774):**
```
error[E0063]: missing fields `draft_id` and `synthesis_result` in initializer of `dictated_notes::DictatedNote`
```

**NeedleEvent missing field (load_test.rs:182):**
```
error[E0063]: missing field `stash_sha` in initializer of `events::NeedleEvent`
```

#### 4. Missing Default Implementation (api_stitch_decompose.rs:1234)
```
error[E0599]: no function or associated item named `default` found for struct `redaction_policy::RedactionPolicyState`
```

**Impact:** Test fixture initialization fails.

**Fix Required:** Implement `Default` trait for `RedactionPolicyState` or use `::new()` with proper parameters.

#### 5. Type Mismatches (heartbeats.rs:935, 1089)
```
error[E0308]: mismatched types
   --> hoop-daemon/src/heartbeats.rs:935:13
    |
935 |             Ok(())
    |             ^^^^^^ expected `()`, found `Result<(), _>`
```

**Impact:** Property test return type mismatch.

## Test-by-Test Breakdown

### 1. test_classify_io_error_already_exists (line 783)
**Status:** Cannot run (blocked by compilation)
**Expected behavior:** Classify `ErrorKind::AlreadyExists` correctly
**Code quality:** Test is correctly written

### 2. test_create_file_with_context_already_exists (line 927)
**Status:** Cannot run (blocked by compilation)
**Expected behavior:** `File::create()` succeeds even if file exists (truncates)
**Code quality:** Test is correctly written

### 3. test_create_file_exclusive_with_context_already_exists (line 950)
**Status:** Cannot run (blocked by compilation)
**Expected behavior:** `File::create_new()` fails with AlreadyExists when file exists
**Code quality:** Test is correctly written

### 4. test_create_dir_with_context_already_exists (line 977)
**Status:** Cannot run (blocked by compilation)
**Expected behavior:** `create_dir()` fails with AlreadyExists when directory exists
**Code quality:** Test is correctly written

### 5. test_create_dir_all_with_context_already_exists (line 1026)
**Status:** Cannot run (blocked by compilation)
**Expected behavior:** `create_dir_all()` fails with AlreadyExists when file exists at path
**Code quality:** Test is correctly written

## Root Cause Summary

**Primary Issue:** Stale test fixtures in multiple modules across the codebase. Production structs gained new fields that were never added to test initializers.

**Secondary Issue:** Missing module declarations and imports prevent compilation.

**NOT an AlreadyExists-specific issue:** The AlreadyExists tests themselves are correctly written and would likely pass if the compilation blockers were resolved.

## Next Steps

To fix this, the following work is needed (in order):

1. **Fix missing module declarations** in `lib.rs`
2. **Add missing macro imports** (`json!` macro)
3. **Update all test fixtures** to include new struct fields:
   - `DaemonState`: Add `br_semaphore`, `br_semaphore_target_permits`
   - `CapacityMeterConfig`: Add `accounts_file`, `gcp_quota_config`, `opencode_dirs`
   - `DictatedNote`: Add `draft_id`, `synthesis_result`
   - `NeedleEvent`: Add `stash_sha`
4. **Fix type mismatches** in property tests
5. **Implement Default trait** for `RedactionPolicyState` or update test to use `::new()`

Once the codebase compiles, the AlreadyExists tests should run and (based on code inspection) are expected to pass.

## Context from Repository State

From `AGENTS.md`:
> **ACTUAL STATE (as of 2026-07-26): Phase 0 complete. Phase 1 in progress.** `cargo test --workspace` does **not** compile: 31 errors in the `hoop-daemon` `lib test` target (stale test fixtures — production structs such as `CapacityMeterConfig`, `DaemonState`, `HoopConfig` gained fields that the test initializers were never updated for).

This confirms the root cause analysis: **stale test fixtures** across the codebase prevent compilation and test execution.

---

**Analysis Date:** 2026-08-06  
**Bead ID:** bf-4mtbo  
**Analyst:** Claude (GLM-4.7)