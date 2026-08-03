# File IO Error Pattern Review (bf-whosx)

## Overview

This document summarizes the existing error handling patterns in the `file_io_error` module, specifically the `NotFound` and `PermissionDenied` implementations.

## Location and Structure

**File Location:** `/home/coding/HOOP/hoop-daemon/src/file_io_error.rs`

**Module Declaration:** `pub mod file_io_error;` in `hoop-daemon/src/lib.rs` (line 66)

### Core Error Type

```rust
pub enum FileIoError {
    NotFound(String),              // Line 56
    PermissionDenied(String),      // Line 58
    // ... 26 other error variants
}
```

### FileIOError::from io::Error Implementation Pattern

**NOT a direct `From` trait implementation** — instead uses a dedicated function:

```rust
pub fn classify_io_error(io_err: &std::io::Error, path: &Path) -> FileIoError
```

**Location:** Lines 197-235

**Pattern used by NotFound and PermissionDenied match arms:**

```rust
match io_err.kind() {
    ErrorKind::NotFound => FileIoError::NotFound(path_str),
    ErrorKind::PermissionDenied => FileIoError::PermissionDenied(path_str),
    // ... other match arms
}
```

The pattern extracts the path as a String (`path_str = path.display().to_string()`) and passes it to the variant constructor.

## Error Message Format

### Display Implementation (lines 109-192)

**NotFound format:**
```rust
FileIoError::NotFound(path) => {
    write!(f, "File not found: {}", path)
}
// Example: "File not found: /path/to/file.txt"
```

**PermissionDenied format:**
```rust
FileIoError::PermissionDenied(path) => {
    write!(f, "Permission denied: {}", path)
}
// Example: "Permission denied: /path/to/file.txt"
```

**Two-parameter variants (IoError, Other):**
```rust
FileIoError::IoError(path, msg) => {
    write!(f, "I/O error accessing '{}': {}", path, msg)
}
FileIoError::Other(path, msg) => {
    write!(f, "Error accessing '{}': {}", path, msg)
}
```

## Helper Functions

The module provides wrapper functions that use `classify_io_error`:

- `read_file_with_context(path: &Path) -> Result<String>` (lines 249-254)
- `open_file_with_context(path: &Path) -> Result<std::fs::File>` (lines 260-265)
- `read_file_optional(path: &Path) -> Result<Option<String>>` (lines 281-290)
- `open_file_optional(path: &Path) -> Result<Option<std::fs::File>>` (lines 295-304)
- `write_file_with_context(path: &Path, content: &str) -> Result<()>` (lines 318-323)
- `create_file_with_context(path: &Path) -> Result<std::fs::File>` (lines 329-334)
- `create_dir_with_context(path: &Path) -> Result<()>` (lines 340-345)
- `create_dir_all_with_context(path: &Path) -> Result<()>` (lines 351-356)

**Pattern used in all wrappers:**
```rust
std::fs::operation(path).map_err(|e| {
    let file_error = classify_io_error(&e, path);
    anyhow::anyhow!("{}", file_error)
})
```

## Test Location and Patterns

**Test file:** Tests are inline in the same module (`#[cfg(test)] mod tests` at line 358)

### Test Patterns for Error Cases

**1. Direct classification tests (unit tests for classify_io_error):**
```rust
#[test]
fn test_classify_io_error_not_found() {
    let io_err = std::io::Error::new(ErrorKind::NotFound, "file not found");
    let path = Path::new("/test/path.txt");
    let file_error = classify_io_error(&io_err, path);

    match file_error {
        FileIoError::NotFound(p) => assert_eq!(p, "/test/path.txt"),
        _ => panic!("Expected NotFound error"),
    }
}
```

Similar tests exist for:
- `test_classify_io_error_permission_denied` (lines 514-523)
- `test_classify_io_error_write_zero` (lines 526-535)
- `test_classify_io_error_storage_full` (lines 538-547)
- `test_classify_io_error_network_unreachable` (lines 550-559)
- `test_classify_io_error_network_down` (lines 562-571)
- `test_classify_io_error_connection_aborted` (lines 574-583)
- `test_classify_io_error_connection_refused` (lines 586-595)
- `test_classify_io_error_connection_reset` (lines 598-607)
- `test_classify_io_error_not_connected` (lines 610-619)
- `test_classify_io_error_addr_in_use` (lines 622-631)
- `test_classify_io_error_addr_not_available` (lines 634-643)
- `test_classify_io_error_not_directory` (lines 646-655)
- `test_classify_io_error_is_directory` (lines 658-667)
- `test_classify_io_error_directory_not_empty` (lines 670-679)
- `test_classify_io_error_already_exists` (lines 763-772)
- `test_classify_io_error_invalid_input` (lines 775-784)
- `test_classify_io_error_invalid_data` (lines 787-796)
- `test_classify_io_error_unexpected_eof` (lines 799-808)
- `test_classify_io_error_interrupted` (lines 811-820)
- `test_classify_io_error_timed_out` (lines 823-832)
- `test_classify_io_error_broken_pipe` (lines 835-844)
- `test_classify_io_error_would_block` (lines 847-856)

**2. Integration-style tests using TempDir:**
```rust
#[test]
fn test_read_file_with_context_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("nonexistent.txt");

    let result = read_file_with_context(&file_path);
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("File not found"));
    assert!(err_msg.contains("nonexistent.txt"));
}
```

**3. Permission denied tests (Unix-only with cfg attribute):**
```rust
#[test]
fn test_read_file_with_context_permission_denied() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("no_permission.txt");
    fs::write(&file_path, "content").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&file_path).unwrap().permissions();
        perms.set_mode(0o000);
        fs::set_permissions(&file_path, perms).unwrap();
    }

    let result = read_file_with_context(&file_path);
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    #[cfg(unix)]
    assert!(err_msg.contains("Permission") || err_msg.contains("permission"));
}
```

**4. Display format tests:**
```rust
#[test]
fn test_file_io_error_display() {
    let err = FileIoError::NotFound("/path/to/file.txt".to_string());
    assert_eq!(err.to_string(), "File not found: /path/to/file.txt");

    let err = FileIoError::PermissionDenied("/path/to/file.txt".to_string());
    assert_eq!(err.to_string(), "Permission denied: /path/to/file.txt");
    // ... tests for all 26 variants
}
```

## Usage in Codebase

**Current usage:** Only used in `hoop-daemon/src/lib.rs` at line 862:

```rust
file_io_error::read_file_with_context(&abs_path)
```

This is within the `get_file_content` async handler for the `/api/projects/:project/files/content` endpoint when `want_raw` is true.

## Key Design Decisions

1. **Path storage:** Uses `String` instead of `PathBuf` for simplicity in Display impl
2. **No `From` trait:** Uses explicit `classify_io_error` function instead of `impl From<io::Error>` to require explicit path context
3. **Comprehensive coverage:** Handles all 26 `std::io::ErrorKind` variants
4. **Consistent messaging:** All messages include the path for debugging
5. **anyhow integration:** Wraps errors in `anyhow::anyhow!()` for easy propagation with `?`
6. **Optional helpers:** Provides `*_optional` variants that return `Ok(None)` on NotFound

## Related Files

- Main implementation: `hoop-daemon/src/file_io_error.rs`
- Module declaration: `hoop-daemon/src/lib.rs:66`
- Usage site: `hoop-daemon/src/lib.rs:862`
