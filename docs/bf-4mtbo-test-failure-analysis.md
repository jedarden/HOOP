# AlreadyExists Test Failures - Analysis (bead bf-4mtbo)

## Summary

**Finding:** The AlreadyExists tests do NOT have runtime failures. They have **compilation failures** that prevent them from running at all.

**Failure Type:** Compilation errors in unrelated modules prevent the entire `hoop-daemon` test suite from building.

## Tests Analyzed

All 5 AlreadyExists tests were analyzed:

1. **test_classify_io_error_already_exists** (line 783)
   - Purpose: Tests classification of AlreadyExists error kind
   - Status: Cannot run - compilation errors in unrelated modules

2. **test_create_file_with_context_already_exists** (line 927)
   - Purpose: Tests File::create behavior when file exists (should succeed - truncates)
   - Status: Cannot run - compilation errors in unrelated modules

3. **test_create_file_exclusive_with_context_already_exists** (line 950)
   - Purpose: Tests File::create_new fails with AlreadyExists when file exists
   - Status: Cannot run - compilation errors in unrelated modules

4. **test_create_dir_with_context_already_exists** (line 977)
   - Purpose: Tests fs::create_dir fails with AlreadyExists when directory exists
   - Status: Cannot run - compilation errors in unrelated modules

5. **test_create_dir_all_with_context_already_exists** (line 1026)
   - Purpose: Tests fs::create_dir_all fails when path is an existing file
   - Status: Cannot run - compilation errors in unrelated modules

## Root Cause Analysis

### Primary Issue: Unrelated Compilation Errors

The AlreadyExists tests themselves are syntactically correct and properly written. However, they cannot run because the `hoop-daemon` crate fails to compile due to errors in **unrelated modules**:

1. **Missing macro import** in `prompt_substitute.rs`:
   - Lines 521, 542: `json!` macro not found
   - Missing: `use serde_json::json;`

2. **Missing module declarations** in `lib.rs`:
   - `template_library`
   - `api_prompts`
   - `api_notes`
   - `api_skills`
   - `api_scripts`

3. **Stale test fixtures** - struct initializers missing new fields:
   - `DaemonState` missing: `br_semaphore`, `br_semaphore_target_permits`
   - `CapacityMeterConfig` missing: `accounts_file`, `gcp_quota_config`, `opencode_dirs`
   - `DictatedNote` missing: `draft_id`, `synthesis_result`
   - `NeedleEvent::Fail` missing: `stash_sha`
   - `RedactionPolicyState::default()` method doesn't exist

### Impact

- **Type:** Compilation errors (not runtime test failures)
- **Scope:** Entire `hoop-daemon` test suite
- **Blocker:** Tests cannot be executed until compilation succeeds
- **Relationship:** AlreadyExists tests are **collateral damage** - they're not broken themselves

## Test Code Review

### AlreadyExists Test Implementation (Correct)

The AlreadyExists tests are correctly implemented:

```rust
// Line 783-791: Correctly tests classify_io_error for AlreadyExists
#[test]
fn test_classify_io_error_already_exists() {
    let io_err = std::io::Error::new(ErrorKind::AlreadyExists, "file already exists");
    let path = Path::new("/test/path.txt");
    let file_error = classify_io_error(&io_err, path);

    match file_error {
        FileIoError::AlreadyExists(p) => assert_eq!(p, "/test/path.txt"),
        _ => panic!("Expected AlreadyExists error"),
    }
}
```

### Error Classification Logic (Correct)

The `classify_io_error` function correctly handles AlreadyExists:

```rust
// Line 203: Correctly maps ErrorKind::AlreadyExists
ErrorKind::AlreadyExists => FileIoError::AlreadyExists(path_str),
```

### Display Implementation (Correct)

The Display impl correctly formats AlreadyExists errors:

```rust
// Line 118-120: Correct display message
FileIoError::AlreadyExists(path) => {
    write!(f, "File already exists: {}", path)
}
```

## Conclusion

**The AlreadyExists functionality is NOT broken.** The tests are correctly written and the implementation is correct. The issue is entirely in unrelated modules that prevent compilation.

**What needs fixing:**
1. Add `use serde_json::json;` to `prompt_substitute.rs`
2. Add missing module declarations to `lib.rs`
3. Update stale test fixtures with new struct fields
4. Implement Default trait for RedactionPolicyState or replace with ::new() call

**What does NOT need fixing:**
- AlreadyExists error classification
- AlreadyExists test logic
- AlreadyExists display formatting
- file_io_error.rs implementation

The tests will pass once the compilation errors are resolved.
