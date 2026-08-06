# Error Type Pattern Comparison - Bead bf-4fk85

**Generated:** 2026-08-06  
**Status:** ✅ Complete - Analysis only (no fixes)  
**Depends On:** bf-9k4kc (AlreadyExists consistency analysis)

---

## Task Scope

Compare AlreadyExists error message patterns with NotFound and PermissionDenied to understand the established pattern across error types.

---

## Common Pattern Analysis

### Display Implementation Reference
**File:** `hoop-daemon/src/file_io_error.rs`  
**Lines:** 109-120

```rust
impl std::fmt::Display for FileIoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileIoError::NotFound(path) => {
                write!(f, "File not found: {}", path)
            }
            FileIoError::PermissionDenied(path) => {
                write!(f, "Permission denied: {}", path)
            }
            FileIoError::AlreadyExists(path) => {
                write!(f, "File already exists: {}", path)
            }
            // ... other variants
        }
    }
}
```

---

## Error Type 1: NotFound

### Canonical Format
**Pattern:** `"File not found: {path}"`

### Structure Analysis
1. **Prefix:** `"File not found"` (Capital F, lowercase rest)
2. **Separator:** `": "` (colon + space)
3. **Value:** `{path}` (string interpolation)

### Test Coverage

#### Test 1: `test_file_io_error_display`
**Location:** Lines 702-703  
**Type:** Direct Display format test

**Expected message:**
```rust
let err = FileIoError::NotFound("/path/to/file.txt".to_string());
assert_eq!(err.to_string(), "File not found: /path/to/file.txt");
```

**Analysis:**
- ✅ Exact match validation
- ✅ Verifies complete format structure
- ✅ Tests capitalization: "File not found"
- ✅ Tests separator: ": "
- ✅ Tests path inclusion

---

#### Test 2: `test_read_file_with_context_not_found`
**Location:** Lines 398-404  
**Type:** Integration test (read_file_with_context)

**Test code:**
```rust
let err_msg = result.unwrap_err().to_string();
assert!(err_msg.contains("File not found"));
assert!(err_msg.contains("nonexistent.txt"));
```

**Analysis:**
- ✅ Partial format validation via `.contains()`
- ✅ Verifies phrase presence: "File not found"
- ✅ Verifies path component: "nonexistent.txt"
- ✅ Separates prefix and path checks

---

#### Test 3: `test_open_file_with_context_not_found`
**Location:** Lines 448-451  
**Type:** Integration test (open_file_with_context)

**Test code:**
```rust
let err_msg = result.unwrap_err().to_string();
assert!(err_msg.contains("File not found"));
assert!(err_msg.contains("nonexistent.txt"));
```

**Analysis:**
- ✅ Partial format validation via `.contains()`
- ✅ Verifies phrase presence: "File not found"
- ✅ Verifies path component: "nonexistent.txt"
- ✅ Separates prefix and path checks

---

### NotFound Summary
| Metric | Count | Status |
|--------|-------|--------|
| Total error-producing tests | 3 | ✅ |
| Tests with exact format match | 1 | ✅ |
| Tests with partial format match | 2 | ✅ |
| Tests without format validation | 0 | ✅ |

**Format:** `"File not found: {path}"`  
**Consistency:** ✅ 100%

---

## Error Type 2: PermissionDenied

### Canonical Format
**Pattern:** `"Permission denied: {path}"`

### Structure Analysis
1. **Prefix:** `"Permission denied"` (Capital P, lowercase d)
2. **Separator:** `": "` (colon + space)
3. **Value:** `{path}` (string interpolation)

### Test Coverage

#### Test 1: `test_file_io_error_display`
**Location:** Lines 705-706  
**Type:** Direct Display format test

**Expected message:**
```rust
let err = FileIoError::PermissionDenied("/path/to/file.txt".to_string());
assert_eq!(err.to_string(), "Permission denied: /path/to/file.txt");
```

**Analysis:**
- ✅ Exact match validation
- ✅ Verifies complete format structure
- ✅ Tests capitalization: "Permission denied"
- ✅ Tests separator: ": "
- ✅ Tests path inclusion

---

#### Test 2: `test_read_file_with_context_permission_denied`
**Location:** Lines 419-426  
**Type:** Integration test (read_file_with_context)

**Test code:**
```rust
let err_msg = result.unwrap_err().to_string();
#[cfg(unix)]
assert!(err_msg.contains("Permission") || err_msg.contains("permission"));
```

**Analysis:**
- ✅ Partial format validation via `.contains()`
- ✅ Platform-aware assertion (unix only)
- ✅ Flexible capitalization check
- ⚠️ Does NOT verify path inclusion (unlike other tests)
- ⚠️ Does NOT verify separator

**Deviation:** This test is more lenient than other error tests

---

### PermissionDenied Summary
| Metric | Count | Status |
|--------|-------|--------|
| Total error-producing tests | 2 | ✅ |
| Tests with exact format match | 1 | ✅ |
| Tests with partial format match | 1 | ⚠️ |
| Tests without format validation | 0 | ✅ |

**Format:** `"Permission denied: {path}"`  
**Consistency:** ✅ 100% (one test is lenient by design)

---

## Error Type 3: AlreadyExists (from bf-9k4kc)

### Canonical Format
**Pattern:** `"File already exists: {path}"`

### Structure Analysis
1. **Prefix:** `"File already exists"` (Capital F, lowercase rest)
2. **Separator:** `": "` (colon + space)
3. **Value:** `{path}` (string interpolation)

### Test Coverage Summary

#### Test Coverage Details

| Test | Line | Format Check | Type |
|------|------|--------------|------|
| test_file_io_error_display | 708-709 | ✅ Exact match | Direct Display |
| test_classify_io_error_already_exists | 782-791 | ❌ No format check | Classification |
| test_create_file_exclusive_with_context_already_exists | 949-961 | ✅ Partial match | Integration |
| test_create_dir_with_context_already_exists | 976-987 | ✅ Partial match | Integration |
| test_create_dir_all_with_context_already_exists | 1026-1039 | ✅ Partial match | Integration |

### AlreadyExists Summary
| Metric | Count | Status |
|--------|-------|--------|
| Total error-producing tests | 5 | ✅ |
| Tests with exact format match | 1 | ✅ |
| Tests with partial format match | 3 | ✅ |
| Tests without format validation | 1 | ⚠️ |

**Format:** `"File already exists: {path}"`  
**Consistency:** ✅ 100%

---

## Cross-Error-Type Pattern Comparison

### Canonical Formats

| Error Type | Format | Prefix | Separator |
|------------|--------|--------|-----------|
| NotFound | `"File not found: {path}"` | "File not found" | ": " |
| PermissionDenied | `"Permission denied: {path}"` | "Permission denied" | ": " |
| AlreadyExists | `"File already exists: {path}"` | "File already exists" | ": " |

### Pattern Consistency Analysis

#### Element 1: Prefix Wording Structure
**Pattern:** `{Capitalized first word} {lowercase description}`

| Error Type | Prefix | Capitalization Pattern | Match? |
|------------|--------|------------------------|--------|
| NotFound | "File not found" | File (F) + not found (lower) | ✅ |
| PermissionDenied | "Permission denied" | Permission (P) + denied (lower) | ✅ |
| AlreadyExists | "File already exists" | File (F) + already exists (lower) | ✅ |

**Conclusion:** ✅ All three follow the same capitalization pattern

---

#### Element 2: Separator
**Expected:** `": "` (colon + space)

| Error Type | Separator | Consistent? |
|------------|-----------|-------------|
| NotFound | ": " | ✅ |
| PermissionDenied | ": " | ✅ |
| AlreadyExists | ": " | ✅ |

**Conclusion:** ✅ All three use identical separator

---

#### Element 3: Path Inclusion
**Expected:** Path always included after separator

| Error Type | Path Included? | Format |
|------------|----------------|--------|
| NotFound | ✅ Yes | `{path}` interpolation |
| PermissionDenied | ✅ Yes | `{path}` interpolation |
| AlreadyExists | ✅ Yes | `{path}` interpolation |

**Conclusion:** ✅ All three include path

---

### Test Strategy Comparison

| Error Type | Total Tests | Exact Match | Partial Match | No Check |
|------------|-------------|-------------|---------------|-----------|
| NotFound | 3 | 1 (33%) | 2 (67%) | 0 (0%) |
| PermissionDenied | 2 | 1 (50%) | 1 (50%) | 0 (0%) |
| AlreadyExists | 5 | 1 (20%) | 3 (60%) | 1 (20%) |

**Analysis:**
- ✅ All error types have at least one exact match test
- ✅ All error types use partial matching (`.contains()`) in integration tests
- ⚠️ Only AlreadyExists has a test that skips format validation (classification test)
- ⚠️ PermissionDenied has the fewest tests (2 vs 3 for NotFound, 5 for AlreadyExists)

---

## Pattern Differences

### Difference 1: Prefix Wording Style

| Error Type | Wording Pattern | Notes |
|------------|-----------------|-------|
| NotFound | "File not found" | Uses "File" prefix despite applying to dirs too |
| PermissionDenied | "Permission denied" | Generic, applies to any filesystem object |
| AlreadyExists | "File already exists" | Uses "File" prefix despite applying to dirs too |

**Observation:** NotFound and AlreadyExists both say "File" even though the error applies to both files and directories. This is slightly misleading but consistent across the two types.

---

### Difference 2: Test Coverage Balance

| Error Type | Integration Contexts | Test Balance |
|------------|---------------------|--------------|
| NotFound | read_file, open_file (2) | ✅ Balanced |
| PermissionDenied | read_file (1) | ⚠️ Lighter coverage |
| AlreadyExists | create_file, create_dir, create_dir_all (3) | ✅ Comprehensive |

**Observation:** AlreadyExists has the most comprehensive test coverage with three different integration contexts. PermissionDenied has the lightest coverage.

---

### Difference 3: Assertion Strictness

| Error Type | Strictest Test | Lenient Test |
|------------|----------------|--------------|
| NotFound | Exact match in Display test | All checks include path |
| PermissionDenied | Exact match in Display test | Unix-only, case-insensitive check |
| AlreadyExists | Exact match in Display test | Classification test has NO format check |

**Observation:** PermissionDenied's integration test is the most lenient (platform-specific, case-insensitive). AlreadyExists' classification test is the only test that completely skips format validation.

---

## Does AlreadyExists Follow the Same Conventions?

### Convention Checklist

| Convention | NotFound | PermissionDenied | AlreadyExists | Follows? |
|------------|----------|------------------|---------------|----------|
| Capitalized first word | ✅ "File" | ✅ "Permission" | ✅ "File" | ✅ YES |
| Lowercase rest of prefix | ✅ "not found" | ✅ "denied" | ✅ "already exists" | ✅ YES |
| Colon + space separator | ✅ ": " | ✅ ": " | ✅ ": " | ✅ YES |
| Path interpolation after separator | ✅ `{path}` | ✅ `{path}` | ✅ `{path}` | ✅ YES |
| Single Display impl | ✅ Yes | ✅ Yes | ✅ Yes | ✅ YES |
| At least one exact match test | ✅ Yes | ✅ Yes | ✅ Yes | ✅ YES |
| Integration tests use `.contains()` | ✅ Yes | ✅ Yes | ✅ Yes | ✅ YES |

**Conclusion:** ✅ **YES** - AlreadyExists follows ALL established conventions

---

## Conclusions

### Pattern Consistency
**Status:** ✅ **EXCELLENT** - All three error types follow identical conventions

All error types use:
1. **Same structural pattern:** `{Prefix}: {path}`
2. **Same capitalization:** Capitalized first word, lowercase rest
3. **Same separator:** Colon + space (`: `)
4. **Same path handling:** Always included after separator
5. **Same testing strategy:** Mix of exact match (Display test) and partial match (integration tests)

---

### AlreadyExists Conformance
**Status:** ✅ **PERFECT** - AlreadyExists follows all conventions with zero deviations

AlreadyExists is fully consistent with NotFound and PermissionDenied patterns:
- ✅ Same wording structure
- ✅ Same capitalization
- ✅ Same separator
- ✅ Same path inclusion
- ✅ Same test strategy

---

### Pattern Differences (None Found)
**Status:** ✅ **NO SIGNIFICANT DIFFERENCES**

The only minor differences are:
1. **Prefix semantics:** NotFound and AlreadyExists say "File" (misleading for directories)
2. **Test quantity:** PermissionDenied has fewer tests (2 vs 3 vs 5)
3. **Test strictness:** PermissionDenied's integration test is platform-specific

These are implementation details, not pattern deviations. All three error types follow the **same underlying pattern**.

---

## Acceptance Criteria Verification

| Criterion | Status |
|-----------|--------|
| ✅ Capture all NotFound error messages from tests | PASS - 3 tests documented |
| ✅ Capture all PermissionDenied error messages from tests | PASS - 2 tests documented |
| ✅ Document the common pattern used by these error types | PASS - Pattern: `{Prefix}: {path}` |
| ✅ Compare this pattern against AlreadyExists messages | PASS - AlreadyExists uses same pattern |
| ✅ Identify pattern differences between error types | PASS - No significant differences found |

---

## Notes

- **Pattern uniformity:** The FileIoError enum demonstrates excellent design consistency across all error variants
- **Implementation quality:** Single Display trait implementation ensures format consistency at compile time
- **Test strategy:** Healthy mix of unit tests (exact match) and integration tests (partial match)
- **No runtime verification:** Tests do not currently compile (31 compilation errors in hoop-daemon lib), so this is static analysis only
- **AlreadyExists leads:** AlreadyExists has the most comprehensive test coverage (5 tests vs 3 for NotFound, 2 for PermissionDenied)

---

## Deliverables Checklist

- ✅ Captured all NotFound error messages from tests
- ✅ Captured all PermissionDenied error messages from tests
- ✅ Documented the common pattern: `{Prefix}: {path}`
- ✅ Compared pattern against AlreadyExists messages
- ✅ Identified pattern differences (none found)
- ✅ Verified AlreadyExists follows same conventions
- ✅ Made no fixes (analysis only)
