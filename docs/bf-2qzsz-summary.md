# Bead bf-2qzsz: AlreadyExists Error Message Extraction

## Task Completed

Extracted unique AlreadyExists error messages and documented with source locations.

## Findings

**Total unique AlreadyExists error messages: 1**

### Error Message Format
`"File already exists: {path}"`

Where `{path}` is the actual file or directory path at runtime.

## Source Locations

### Primary Definition
- **File**: `hoop-daemon/src/file_io_error.rs`
- **Line**: 119 (Display implementation)
- **Code**: `write!(f, "File already exists: {}", path)`

### Test Coverage
6 test functions verify this error message:

1. `test_classify_io_error_already_exists` (line 782)
2. `test_file_io_error_display` (line 708)
3. `test_create_file_exclusive_with_context_already_exists` (line 949)
4. `test_create_dir_with_context_already_exists` (line 976)
5. `test_create_dir_all_with_context_already_exists` (line 1026)
6. `test_create_file_with_context_already_exists` (line 926) - tests success case

## Deliverables

✅ **Output file**: `/tmp/alreadyexists_errors.log`
- Comprehensive documentation of all unique AlreadyExists error messages
- Complete source location mapping
- Test assertions and coverage details
- Usage patterns and cross-references

## Notes

- Single error message format used consistently across all AlreadyExists scenarios
- Tests verify both error classification and message content
- Message format applies to both file and directory operations
- Error produced by File::create_new(), create_dir(), and create_dir_all() (when blocked)
- File::create() does NOT produce this error (truncates existing files)

## Related Work

Previous bead `bf-1fm2w` identified these tests but couldn't run them due to Phase 1 compilation issues. This bead successfully completed the analysis through static code examination since the error format is clearly defined in the Display implementation.

## Dependencies

None - analysis completed through code inspection only.
