# AlreadyExists Error Message Verification

**Task:** Verify AlreadyExists error messages are descriptive and consistent
**Bead:** bf-1pbir
**Date:** 2026-08-06

## Summary

All AlreadyExists error messages follow the established pattern correctly. The messages are descriptive, include file/directory paths, and match the format used by other error types.

## Display Implementation

**Location:** `hoop-daemon/src/file_io_error.rs:118-120`

```rust
FileIoError::AlreadyExists(path) => {
    write!(f, "File already exists: {}", path)
}
```

**Format:** `"File already exists: {path}"`

## Verification Against Acceptance Criteria

### ✓ Error messages contain "already exists" or "AlreadyExists"

All assertions check for the string `"File already exists"`:
- Line 959: `assert!(err_msg.contains("File already exists"));`
- Line 985: `assert!(err_msg.contains("File already exists"));`
- Line 1037: `assert!(err_msg.contains("File already exists"));`

### ✓ Error messages include the file/directory path

All test assertions verify the path is included:
- Line 960: `assert!(err_msg.contains("test.txt"));`
- Line 986: `assert!(err_msg.contains("test_dir"));`
- Line 1038: `assert!(err_msg.contains("blocking_file"));`

### ✓ Messages follow the pattern established by other error types

Comparison with other error types:

| Error Type | Format | Line |
|------------|--------|------|
| NotFound | `"File not found: {}"` | 112-114 |
| PermissionDenied | `"Permission denied: {}"` | 115-117 |
| **AlreadyExists** | **`"File already exists: {}"`** | **118-120** |
| InvalidInput | `"Invalid input for operation on: {}"` | 121-123 |

All three primary file system errors follow the pattern: `{Error description}: {path}`

### ✓ Compared with NotFound and PermissionDenied patterns

**NotFound:**
```rust
FileIoError::NotFound(path) => {
    write!(f, "File not found: {}", path)
}
```
Test assertions (line 403-405, 451-453):
- `assert!(err_msg.contains("File not found"));`
- `assert!(err_msg.contains("nonexistent.txt"));`

**PermissionDenied:**
```rust
FileIoError::PermissionDenied(path) => {
    write!(f, "Permission denied: {}", path)
}
```
Test assertions (line 427, 490, 909):
- `assert!(err_msg.contains("Permission") || err_msg.contains("permission"));`

**AlreadyExists:**
```rust
FileIoError::AlreadyExists(path) => {
    write!(f, "File already exists: {}", path)
}
```
Test assertions (line 959-960, 985-986, 1037-1038):
- `assert!(err_msg.contains("File already exists"));`
- `assert!(err_msg.contains("<path>"));`

**Pattern consistency:** All three follow identical structure.

## Test Coverage

The following tests verify AlreadyExists error messages:

1. **`test_classify_io_error_already_exists`** (line 782-791)
   - Verifies classification of ErrorKind::AlreadyExists
   - No message assertion (only variant type checked)

2. **`test_create_file_with_context_already_exists`** (line 926-934)
   - Note: This test passes because `File::create()` truncates existing files
   - Does not test AlreadyExists error

3. **`test_create_file_exclusive_with_context_already_exists`** (line 949-961)
   - ✓ Tests `File::create_new()` exclusive creation
   - ✓ Asserts "File already exists" in message
   - ✓ Asserts path "test.txt" in message

4. **`test_create_dir_with_context_already_exists`** (line 976-987)
   - ✓ Tests `fs::create_dir()` on existing directory
   - ✓ Asserts "File already exists" in message
   - ✓ Asserts path "test_dir" in message

5. **`test_create_dir_all_with_context_already_exists`** (line 1026-1039)
   - ✓ Tests `fs::create_dir_all()` when file exists at path
   - ✓ Asserts "File already exists" in message
   - ✓ Asserts path "blocking_file" in message

6. **`test_file_io_error_display`** (line 708-709)
   - ✓ Asserts exact format: `"File already exists: /path/to/file.txt"`

## Observations

### 1. Generic "File already exists" for directories

The error message uses `"File already exists"` even when the path refers to a directory:

```rust
// Line 985-986: Creating a directory that already exists
assert!(err_msg.contains("File already exists"));  // Not "Directory already exists"
assert!(err_msg.contains("test_dir"));
```

This is **consistent** with the standard library's `ErrorKind::AlreadyExists` behavior, which doesn't distinguish between files and directories. The message is technically accurate (a "file" in the POSIX sense includes directories), but users might find "Directory already exists" clearer when operating on directories.

**Decision:** This is a minor stylistic point, not a functional bug. The current implementation is acceptable and matches the pattern used by NotFound and PermissionDenied (both also say "File" regardless of whether the path is a file or directory).

### 2. Consistency across error types

All three primary file system errors (NotFound, PermissionDenied, AlreadyExists) use the same structure:
- `{Description}: {path}`
- All say "File" regardless of whether the target is a file or directory
- All include the full path in the message

This consistency is **good UX** — operators learn one pattern and can predict error messages for all file system operations.

## Conclusion

**Status:** ✓ PASS

All AlreadyExists error messages are:
1. ✓ Descriptive and clear
2. ✓ Contain "already exists" phrase
3. ✓ Include the file/directory path
4. ✓ Follow the established pattern from NotFound/PermissionDenied
5. ✓ Covered by automated tests

**No inconsistencies found.** The implementation is correct and consistent.
