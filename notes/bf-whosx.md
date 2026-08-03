# file_io_error Pattern Review (Bead bf-whosx)

## Summary

Research completed on the existing `FileIOError` implementation in `/home/coding/HOOP/hoop-daemon/src/file_io_error.rs`. The module provides comprehensive error handling for file I/O operations with explicit categorization.

## Module Location

**File:** `/home/coding/HOOP/hoop-daemon/src/file_io_error.rs`

## Key Findings

### 1. FileIOError::from std::io::Error Implementation

**Location:** Lines 196-235 (function `classify_io_error`)

**Pattern Structure:**
```rust
pub fn classify_io_error(io_err: &std::io::Error, path: &Path) -> FileIoError {
    let path_str = path.display().to_string();

    match io_err.kind() {
        ErrorKind::NotFound => FileIoError::NotFound(path_str),
        ErrorKind::PermissionDenied => FileIoError::PermissionDenied(path_str),
        // ... other error kinds
        ErrorKind::Other => {
            // Special handling for "is a directory" detection
            if io_err.to_string().to_lowercase().contains("is a directory") {
                FileIoError::IsDirectory(path_str)
            } else {
                FileIoError::IoError(path_str, io_err.to_string())
            }
        }
        _ => FileIoError::Other(path_str, format!("{}: {}", io_err.kind(), io_err)),
    }
}
```

### 2. NotFound and PermissionDenied Match Arms

**Exact Match Patterns (lines 201-202):**
```rust
ErrorKind::NotFound => FileIoError::NotFound(path_str),
ErrorKind::PermissionDenied => FileIoError::PermissionDenied(path_str),
```

**Both use the same structure:**
- Match on `ErrorKind::NotFound` or `ErrorKind::PermissionDenied`
- Create variant with `path_str` (converted via `path.display().to_string()`)
- Store as single `String` field

### 3. Error Message Format

**Display Implementation (lines 109-192):**

**NotFound format (lines 112-114):**
```rust
FileIoError::NotFound(path) => {
    write!(f, "File not found: {}", path)
}
```
**Exact message:** `"File not found: {path}"`

**PermissionDenied format (lines 115-117):**
```rust
FileIoError::PermissionDenied(path) => {
    write!(f, "Permission denied: {}", path)
}
```
**Exact message:** `"Permission denied: {path}"`

**Pattern consistency:**
- All single-argument errors use: `"{Error description}: {path}"`
- Two-argument errors (IoError, Other) use: `"Error accessing '{path}': {message}"`

### 4. Test File Location

**Location:** Same file `/home/coding/HOOP/hoop-daemon/src/file_io_error.rs` (lines 358-977)

**Test Module Structure:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;
    // ... test cases
}
```

**Existing Test Patterns for Error Cases:**

1. **NotFound tests (lines 376-387, 423-434):**
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

2. **PermissionDenied tests (lines 389-411, 457-478):**
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

3. **Classification tests (lines 501-856):**
   - Direct classification of `std::io::Error` by kind
   - Pattern: Create `std::io::Error` → call `classify_io_error` → match variant → assert path

### 5. Module Structure

**Public Functions:**
- `read_file_with_context(path: &Path) -> Result<String>` (lines 249-254)
- `open_file_with_context(path: &Path) -> Result<std::fs::File>` (lines 260-265)
- `read_file_optional(path: &Path) -> Result<Option<String>>` (lines 281-290)
- `open_file_optional(path: &Path) -> Result<Option<std::fs::File>>` (lines 295-304)
- `write_file_with_context(path: &Path, content: &str) -> Result<()>` (lines 318-323)
- `create_file_with_context(path: &Path) -> Result<std::fs::File>` (lines 329-334)
- `create_dir_with_context(path: &Path) -> Result<()>` (lines 340-345)
- `create_dir_all_with_context(path: &Path) -> Result<()>` (lines 351-356)
- `classify_io_error(io_err: &std::io::Error, path: &Path) -> FileIoError` (lines 197-235)

### 6. Error Variants

The `FileIoError` enum has 27 variants covering all `std::io::ErrorKind` values:
- NotFound, PermissionDenied, AlreadyExists, InvalidInput, InvalidData
- IsDirectory, NotDirectory, DirectoryNotEmpty
- UnexpectedEof, WriteZero, Interrupted, TimedOut
- BrokenPipe, WouldBlock, StorageFull
- NetworkUnreachable, NetworkDown
- ConnectionAborted, ConnectionRefused, ConnectionReset, NotConnected
- AddrInUse, AddrNotAvailable, OutOfMemory
- IoError(String, String) - generic I/O error with message
- Other(String, String) - uncategorized with kind and message

### 7. Wrapper Function Pattern

**All wrapper functions follow the same pattern (e.g., lines 250-253):**
```rust
pub fn read_file_with_context(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).map_err(|e| {
        let file_error = classify_io_error(&e, path);
        anyhow::anyhow!("{}", file_error)
    })
}
```

**Key elements:**
1. Call the appropriate `std::fs` function
2. Use `.map_err()` to transform the error
3. Call `classify_io_error()` to convert to `FileIoError`
4. Wrap in `anyhow::anyhow!("{}", file_error)` to convert to `anyhow::Error`

**Optional variants** (`read_file_optional`, `open_file_optional`):
```rust
match std::fs::read_to_string(path) {
    Ok(content) => Ok(Some(content)),
    Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
    Err(e) => { /* classify and wrap */ }
}
```

### 8. Usage Notes

**Current Status:** The module is defined but **not yet actively used** in the codebase:
- No usage found in other modules beyond the file_io_error.rs itself
- Not exported in lib.rs (lines 1-200 checked)
- Ready for adoption but waiting for implementation

**Note:** There is NO `From<std::io::Error>` implementation. The module uses `classify_io_error()` function instead, which requires both an error and a path parameter.

## Acceptance Criteria Checklist

✅ **Identify exact location and structure of FileIOError::from io::Error implementation**
- Found: `classify_io_error` function at lines 196-235

✅ **Document pattern used by NotFound and PermissionDenied match arms**
- Both use `ErrorKind::X => FileIoError::X(path_str)` pattern
- Path converted via `path.display().to_string()`

✅ **Note exact error message format used by existing error types**
- NotFound: `"File not found: {path}"`
- PermissionDenied: `"Permission denied: {path}"`
- General pattern: `"{Error description}: {path}"`

✅ **Identify test file location and existing test patterns**
- Tests in same file: lines 358-977
- Patterns use `tempfile::TempDir` for fixtures
- Unix-specific permission tests use `#[cfg(unix)]`
- Message assertions use `.contains()` for flexibility
