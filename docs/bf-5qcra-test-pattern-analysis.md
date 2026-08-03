# Test Pattern Analysis: NotFound and PermissionDenied

## Location
Tests are in `hoop-daemon/src/file_io_error.rs` in the `#[cfg(test)]` module starting at line 378.

## Test Pattern Structure

### Pattern 1: Unit Tests for `classify_io_error` Function
These tests directly verify the error classification logic without touching the filesystem.

**Template:**
```rust
#[test]
fn test_classify_io_error_<variant>() {
    let io_err = std::io::Error::new(ErrorKind::<Variant>, "description");
    let path = Path::new("/test/path.txt");
    let file_error = classify_io_error(&io_err, path);
    
    match file_error {
        FileIoError::<Variant>(p) => assert_eq!(p, "/test/path.txt"),
        _ => panic!("Expected <Variant> error"),
    }
}
```

**Examples:**
- `test_classify_io_error_not_found` (line 522)
- `test_classify_io_error_permission_denied` (line 534)
- `test_classify_io_error_already_exists` (line 783) - **Already exists**

**Key characteristics:**
- Creates a synthetic `std::io::Error` with a specific `ErrorKind`
- Tests the `classify_io_error` function directly
- Uses pattern matching to verify correct variant
- Asserts the path string is preserved

### Pattern 2: Integration Tests for File Operations
These test the actual file operations with real filesystem behavior.

**Template:**
```rust
#[test]
fn test_<operation>_<error_condition>() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    
    // Setup: Create conditions that will trigger the error
    // (e.g., remove permissions, create file before exclusive create, etc.)
    
    let result = <operation>(&file_path);
    assert!(result.is_err());
    
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("<expected error text>"));
    assert!(err_msg.contains("<filename>"));
}
```

**Examples:**
- `test_read_file_with_context_not_found` (line 397)
- `test_read_file_with_context_permission_denied` (line 410)
- `test_create_file_exclusive_with_context_already_exists` (line 950)
- `test_create_dir_with_context_already_exists` (line 977)

**Key characteristics:**
- Uses `tempfile::TempDir` for isolated test directories
- Triggers real error conditions (missing files, permission issues, etc.)
- Asserts operation returns `Err`
- Validates error message content includes expected text and filename

## Test Helpers and Fixtures

### 1. `tempfile::TempDir`
- **Import:** `use tempfile::TempDir;`
- **Usage:** Creates temporary directories that are automatically cleaned up
- **Pattern:** `let temp_dir = TempDir::new().unwrap();`
- **Path creation:** `temp_dir.path().join("filename.txt")`

### 2. Platform-specific permission handling
```rust
#[cfg(unix)]
{
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(&file_path).unwrap().permissions();
    perms.set_mode(0o000);
    fs::set_permissions(&file_path, perms).unwrap();
}
```

### 3. Error message assertions
```rust
assert!(err_msg.contains("File not found"));
assert!(err_msg.contains("nonexistent.txt"));
```

## Existing AlreadyExists Tests

Already has the following tests:
1. **Unit test:** `test_classify_io_error_already_exists` (line 783)
   - Tests `classify_io_error` with `ErrorKind::AlreadyExists`
   
2. **Integration test:** `test_create_file_exclusive_with_context_already_exists` (line 950)
   - Tests `create_file_exclusive_with_context` when file exists
   - Asserts error message contains "already exists" and filename

3. **Integration test:** `test_create_dir_with_context_already_exists` (line 977)
   - Tests `create_dir_with_context` when directory exists
   - Asserts error message contains "already exists"

## Where AlreadyExists Tests Should Be Added

**AlreadyExists already has good coverage:**
- Unit test for error classification ✓
- Integration test for exclusive file creation ✓
- Integration test for directory creation ✓

**Potential gaps (if additional coverage is needed):**
- Test `write_file_with_context` when directory is a file (path exists but is wrong type)
- Test for `create_dir_all_with_context` when some directories exist but others don't (partial path)

## Summary Table

| Test Type | NotFound | PermissionDenied | AlreadyExists |
|-----------|----------|------------------|----------------|
| Unit test (classify) | ✓ (line 522) | ✓ (line 534) | ✓ (line 783) |
| Integration (read) | ✓ (line 397) | ✓ (line 410) | N/A |
| Integration (open) | ✓ (line 444) | N/A | N/A |
| Integration (optional read) | ✓ (line 468) | ✓ (line 478) | N/A |
| Integration (write) | N/A | ✓ (line 892) | N/A |
| Integration (create exclusive) | N/A | N/A | ✓ (line 950) |
| Integration (create dir) | N/A | N/A | ✓ (line 977) |

**Note:** "N/A" means the error type doesn't apply to that operation (e.g., AlreadyExists doesn't apply to read operations).
