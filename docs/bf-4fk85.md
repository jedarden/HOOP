# Error Type Pattern Comparison Analysis (bf-4fk85)

## Overview

This analysis compares error message patterns across three error types in `hoop-daemon/src/file_io_error.rs`:
- `NotFound`
- `PermissionDenied` 
- `AlreadyExists`

## Display Implementation Pattern (lines 109-120)

### NotFound (line 112-114)
```rust
FileIoError::NotFound(path) => {
    write!(f, "File not found: {}", path)
}
```
**Pattern:** `"File not found: {path}"`

### PermissionDenied (line 115-117)
```rust
FileIoError::PermissionDenied(path) => {
    write!(f, "Permission denied: {}", path)
}
```
**Pattern:** `"Permission denied: {path}"`

### AlreadyExists (line 118-120)
```rust
FileIoError::AlreadyExists(path) => {
    write!(f, "File already exists: {}", path)
}
```
**Pattern:** `"File already exists: {path}"`

## Common Pattern Characteristics

All three error types follow the **same structural pattern**:

1. **Wording structure:** `[Capitalized error description]: {path}`
   - Error description is concise and human-readable
   - Colon separator between description and path
   - Path follows as context

2. **Path inclusion:** 
   - All three include the full path as a string parameter
   - Path is interpolated directly with `{}` formatting
   - No additional context beyond the path

3. **Capitalization:**
   - Error descriptions use **Title Case** for the first letter of each major word
   - NotFound: "File not found"
   - PermissionDenied: "Permission denied" 
   - AlreadyExists: "File already exists"

4. **Message conciseness:**
   - All messages are 3-4 words (excluding the path)
   - No additional explanatory text
   - Direct, actionable error descriptions

## Test Coverage Verification

### NotFound tests:
- Line 404-406: `test_read_file_with_context_not_found` verifies:
  - `err_msg.contains("File not found")`
  - `err_msg.contains("nonexistent.txt")`
  
- Line 451-453: `test_open_file_with_context_not_found` verifies:
  - `err_msg.contains("File not found")`
  - `err_msg.contains("nonexistent.txt")`

- Line 702-703: `test_file_io_error_display` verifies exact format:
  - `"File not found: /path/to/file.txt"`

### PermissionDenied tests:
- Line 426-429: `test_read_file_with_context_permission_denied` verifies:
  - Contains "Permission" or "permission" (case-insensitive check)
  
- Line 705-706: `test_file_io_error_display` verifies exact format:
  - `"Permission denied: /path/to/file.txt"`

### AlreadyExists tests:
- Line 708-709: `test_file_io_error_display` verifies exact format:
  - `"File already exists: /path/to/file.txt"`

- Line 959-961: `test_create_file_exclusive_with_context_already_exists` verifies:
  - `err_msg.contains("File already exists")`
  - `err_msg.contains("test.txt")`

- Line 985-987: `test_create_dir_with_context_already_exists` verifies:
  - `err_msg.contains("File already exists")`
  - `err_msg.contains("test_dir")`

- Line 1037-1039: `test_create_dir_all_with_context_already_exists` verifies:
  - `err_msg.contains("File already exists")`
  - `err_msg.contains("blocking_file")`

## Pattern Consistency Analysis

### ✅ AlreadyExists DOES follow the same conventions

The AlreadyExists error type is **fully consistent** with NotFound and PermissionDenied:

1. **Structure:** Uses the same `[Error description]: {path}` pattern
2. **Capitalization:** "File already exists" follows Title Case convention
3. **Path inclusion:** Includes path with same formatting `{}` 
4. **Message length:** "File already exists" is 3 words, same as NotFound and PermissionDenied
5. **Test coverage:** Has equivalent test coverage with same assertion patterns

### Minor Pattern Differences (within acceptable variation)

**No significant differences found.** All three error types follow an identical pattern in their Display implementation and test verification.

The only variation worth noting is that PermissionDenied tests use case-insensitive matching (`"Permission" || "permission"`), while NotFound and AlreadyExists use exact string matching. However, this is a test implementation detail, not a pattern difference in the actual error messages.

## Conclusion

The AlreadyExists error type follows the **exact same pattern** as NotFound and PermissionDenied for:
- Wording structure (`[Description]: {path}`)
- Path inclusion (always present, formatted with `{}`)
- Capitalization (Title Case descriptions)
- Test verification (same assertion patterns)

**No pattern inconsistencies identified.** The implementation is consistent across all three error types.
