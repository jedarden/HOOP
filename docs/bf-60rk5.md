# AlreadyExists Tests Inventory - Bead bf-60rk5

**Generated:** 2026-08-06  
**Status:** ✅ Complete - Read-only information gathering

## Task Scope
Find all AlreadyExists-related tests and capture their current error message output.

## Findings Summary

### Test File Location
- **File:** `hoop-daemon/src/file_io_error.rs`
- **Lines:** 708-1039 (test module)

### Complete AlreadyExists Test Inventory

**Total tests found:** 5 error-producing tests + 1 success test

| Test Name | Line Range | Error Message Pattern | Test Type |
|-----------|-----------|----------------------|-----------|
| test_file_io_error_display | 708-709 | `"File already exists: {path}"` | Display format |
| test_classify_io_error_already_exists | 782-791 | `"File already exists: {path}"` | Classification |
| test_create_file_exclusive_with_context_already_exists | 949-961 | `"File already exists: {path}"` | Integration |
| test_create_dir_with_context_already_exists | 976-987 | `"File already exists: {path}"` | Integration |
| test_create_dir_all_with_context_already_exists | 1026-1039 | `"File already exists: {path}"` | Integration |

### Unique Error Messages
**Count:** 1 unique error message format
- **Format:** `"File already exists: {path}"`
- **Implementation:** Display trait at line 118-120
- **Classification:** classify_io_error() at line 203

### Compilation Status
❌ **Tests do not currently compile**
- hoop-daemon lib has 37 compilation errors
- Static analysis only - runtime verification pending compilation fixes

## Methodology
1. Searched for AlreadyExists in test files
2. Located all test functions using the error type
3. Extracted expected error messages from assertions
4. Documented test locations and patterns
5. Cross-referenced with existing documentation

## Deliverables
- ✅ Located all test files containing AlreadyExists tests
- ✅ Identified expected error message output from test assertions
- ✅ Created list of unique error messages (1 format)
- ✅ Documented test file and line numbers for each message
- ✅ Saved findings to docs/bf-60rk5.md

## Notes
- Comprehensive documentation already exists in notes/alreadyexists_errors.log (from bead bf-2qzsz)
- All AlreadyExists errors follow consistent message format
- No variations or alternative message formats found in codebase
- Error message includes both static text ("File already exists: ") and dynamic path
