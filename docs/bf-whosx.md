# file_io_error Pattern Review (Bead bf-whosx)

## Summary

Research-only review of existing error handling patterns in the `hoop-daemon/src/file_io_error.rs` module, focusing on NotFound and PermissionDenied implementations.

## File Location

**Primary Module:** `/home/coding/HOOP/hoop-daemon/src/file_io_error.rs`

**Integration Point:** The module is exposed in `hoop-daemon/src/lib.rs` as `pub mod file_io_error;`

## Error Enum Structure

The module defines a comprehensive `FileIoError` enum with variants for all `std::io::ErrorKind` types plus additional categorizations:

```rust
pub enum FileIoError {
    NotFound(String),                    // Line 56
    PermissionDenied(String),            // Line 58
    AlreadyExists(String),               // Line 60
    InvalidInput(String),                // Line 62
    InvalidData(String),                 // Line 64
    IsDirectory(String),                 // Line 66
    NotDirectory(String),                // Line 68
    DirectoryNotEmpty(String),            // Line 70
    UnexpectedEof(String),               // Line 72
    WriteZero(String),                    // Line 74
    Interrupted(String),                  // Line 76
    TimedOut(String),                    // Line 78
    BrokenPipe(String),                  // Line 80
    WouldBlock(String),                  // Line 82
    StorageFull(String),                  // Line 84
    NetworkUnreachable(String),          // Line 86
    NetworkDown(String),                  // Line 88
    ConnectionAborted(String),           // Line 90
    ConnectionRefused(String),           // Line 92
    ConnectionReset(String),             // Line 94
    NotConnected(String),                // Line 96
    AddrInUse(String),                   // Line 98
    AddrNotAvailable(String),            // Line 100
    OutOfMemory(String),                 // Line 102
    IoError(String, String),             // Line 104 - Generic I/O with path + message
    Other(String, String),               // Line 106 - Uncategorized with kind + message
}
```

## FileIOError::from io::Error Implementation Pattern

### Core Conversion Function

**Location:** `classify_io_error()` function (lines 197-235)

**Pattern Structure:**
```rust
pub fn classify_io_error(io_err: &std::io::Error, path: &Path) -> FileIoError {
    let path_str = path.display().to_string();

    match io_err.kind() {
        ErrorKind::NotFound => FileIoError::NotFound(path_str),
        ErrorKind::PermissionDenied => FileIoError::PermissionDenied(path_str),
        ErrorKind::AlreadyExists => FileIoError::AlreadyExists(path_str),
        // ... other direct mappings ...
        
        ErrorKind::Other => {
            // Special handling for ambiguous errors
            if io_err.to_string().to_lowercase().contains("is a directory") {
                FileIoError::IsDirectory(path_str)
            } else {
                FileIoError::IoError(path_str, io_err.to_string())
            }
        }
        
        // Fallback for uncategorized errors
        _ => FileIoError::Other(path_str, format!("{}: {}", io_err.kind(), io_err)),
    }
}
```

### NotFound Match Arm Pattern

**Location:** Line 201 in `classify_io_error()`

```rust
ErrorKind::NotFound => FileIoError::NotFound(path_str),
```

**Display Implementation:** Lines 112-114
```rust
FileIoError::NotFound(path) => {
    write!(f, "File not found: {}", path)
}
```

**Error Message Format:** `"File not found: {path}"`

### PermissionDenied Match Arm Pattern

**Location:** Line 202 in `classify_io_error()`

```rust
ErrorKind::PermissionDenied => FileIoError::PermissionDenied(path_str),
```

**Display Implementation:** Lines 115-117
```rust
FileIoError::PermissionDenied(path) => {
    write!(f, "Permission denied: {}", path)
}
```

**Error Message Format:** `"Permission denied: {path}"`

## Usage Pattern in Wrapper Functions

All file operation wrappers follow the same pattern:

```rust
pub fn read_file_with_context(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).map_err(|e| {
        let file_error = classify_io_error(&e, path);
        anyhow::anyhow!("{}", file_error)
    })
}
```

**Key Pattern Elements:**
1. Call the underlying `std::fs` operation
2. Use `.map_err()` to transform errors
3. Call `classify_io_error()` with the error and path
4. Convert to `anyhow::Error` via display formatting

## Error Message Format Catalog

| Error Type | Display Format (from Display impl) |
|------------|-----------------------------------|
| NotFound | `"File not found: {path}"` |
| PermissionDenied | `"Permission denied: {path}"` |
| AlreadyExists | `"File already exists: {path}"` |
| InvalidInput | `"Invalid input for operation on: {path}"` |
| InvalidData | `"Invalid data in file: {path}"` |
| IsDirectory | `"Path is a directory, not a file: {path}"` |
| NotDirectory | `"Path is not a directory: {path}"` |
| DirectoryNotEmpty | `"Directory not empty: {path}"` |
| UnexpectedEof | `"Unexpected end of file: {path}"` |
| WriteZero | `"Write returned zero bytes: {path}"` |
| Interrupted | `"Operation interrupted on: {path}"` |
| TimedOut | `"Operation timed out on: {path}"` |
| BrokenPipe | `"Broken pipe (connection closed) for: {path}"` |
| WouldBlock | `"Operation would block (non-blocking) for: {path}"` |
| StorageFull | `"Storage full while accessing: {path}"` |
| NetworkUnreachable | `"Network unreachable while accessing: {path}"` |
| NetworkDown | `"Network down while accessing: {path}"` |
| ConnectionAborted | `"Connection aborted while accessing: {path}"` |
| ConnectionRefused | `"Connection refused while accessing: {path}"` |
| ConnectionReset | `"Connection reset while accessing: {path}"` |
| NotConnected | `"Not connected while accessing: {path}"` |
| AddrInUse | `"Address in use: {path}"` |
| AddrNotAvailable | `"Address not available: {path}"` |
| OutOfMemory | `"Out of memory while accessing: {path}"` |
| IoError | `"I/O error accessing '{path}': {message}"` |
| Other | `"Error accessing '{path}': {kind}: {message}"` |

## Test File Location

**Primary Test Location:** Tests are embedded inline in the same file at `/home/coding/HOOP/hoop-daemon/src/file_io_error.rs`

**Test Module:** Lines 358-977 (`#[cfg(test)] mod tests`)

### Test Patterns for Error Cases

#### NotFound Error Tests

**Direct Classification Test** (lines 502-511):
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

**Integration Test** (lines 377-387):
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

#### PermissionDenied Error Tests

**Direct Classification Test** (lines 514-523):
```rust
#[test]
fn test_classify_io_error_permission_denied() {
    let io_err = std::io::Error::new(ErrorKind::PermissionDenied, "permission denied");
    let path = Path::new("/test/path.txt");
    let file_error = classify_io_error(&io_err, path);

    match file_error {
        FileIoError::PermissionDenied(p) => assert_eq!(p, "/test/path.txt"),
        _ => panic!("Expected PermissionDenied error"),
    }
}
```

**Integration Test** (lines 390-411):
```rust
#[test]
fn test_read_file_with_context_permission_denied() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("no_permission.txt");
    fs::write(&file_path, "content").unwrap();

    // Remove read permissions
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
    // Permission denied behavior varies by system
    #[cfg(unix)]
    assert!(err_msg.contains("Permission") || err_msg.contains("permission"));
}
```

### Test Pattern Characteristics

1. **Two-layer testing:**
   - Direct `classify_io_error()` unit tests with synthetic errors
   - Integration tests using actual file operations with `tempfile`

2. **Path verification:** Tests assert that the path string is correctly preserved

3. **Message format verification:** Integration tests check for expected phrases in error messages

4. **Platform-aware:** Permission tests use `#[cfg(unix)]` for platform-specific behavior

5. **Display format coverage:** `test_file_io_error_display()` (lines 682-760) comprehensively tests all error message formats

## Available Public Functions

**Read Operations:**
- `read_file_with_context(path: &Path) -> Result<String>` (line 249)
- `read_file_optional(path: &Path) -> Result<Option<String>>` (line 281)
- `open_file_with_context(path: &Path) -> Result<std::fs::File>` (line 260)
- `open_file_optional(path: &Path) -> Result<Option<std::fs::File>>` (line 295)

**Write Operations:**
- `write_file_with_context(path: &Path, content: &str) -> Result<()>` (line 318)
- `create_file_with_context(path: &Path) -> Result<std::fs::File>` (line 329)

**Directory Operations:**
- `create_dir_with_context(path: &Path) -> Result<()>` (line 340)
- `create_dir_all_with_context(path: &Path) -> Result<()>` (line 351)

**Classification:**
- `classify_io_error(io_err: &std::io::Error, path: &Path) -> FileIoError` (line 197)

## Dependencies

- `anyhow` for error result types
- `std::io::ErrorKind` for error classification
- `std::path::Path` for path handling
- `tempfile` (dev-dependency) for test fixtures

## Integration Point

The module is used in `hoop-daemon/src/lib.rs`:

```rust
// Line showing integration
file_io_error::read_file_with_context(&abs_path)
```

## Key Design Patterns

1. **Single String Argument:** All error variants take a single `String` (path) except `IoError` and `Other` which take `(String, String)` for path + message

2. **Path Preservation:** The original path is always converted to string via `path.display().to_string()` and preserved in the error

3. **Display-based Error Messages:** The `Display` trait implementation (lines 109-192) is the single source of truth for error message formats

4. **Anyhow Integration:** All wrapper functions return `anyhow::Result<T>` and convert `FileIoError` to `anyhow::Error` via the Display trait

5. **Comprehensive Coverage:** The module handles all 27 `std::io::ErrorKind` variants plus custom categorization for ambiguous cases
