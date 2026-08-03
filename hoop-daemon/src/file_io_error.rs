//! File I/O error handling utilities
//!
//! Provides consistent error handling for file operations with explicit
//! error categorization and clear error messages that include file paths.
//!
//! # Error Types
//!
//! This module categorizes `std::io::Error` into specific error types:
//! - `NotFound` - File or directory not found
//! - `PermissionDenied` - Insufficient permissions
//! - `AlreadyExists` - File or directory already exists
//! - `InvalidInput` - Invalid input parameters
//! - `InvalidData` - Corrupted or invalid data
//! - `IsDirectory` - Path is a directory when file expected
//! - `NotDirectory` - Path is not a directory when directory expected
//! - `DirectoryNotEmpty` - Directory not empty when removal attempted
//! - `UnexpectedEof` - Unexpected end of file
//! - `WriteZero` - Write returned zero bytes
//! - `Interrupted` - Operation interrupted
//! - `TimedOut` - Operation timed out
//! - `BrokenPipe` - Broken pipe (connection closed)
//! - `WouldBlock` - Operation would block (non-blocking)
//! - `StorageFull` - Storage full
//! - `NetworkUnreachable` - Network unreachable
//! - `NetworkDown` - Network down
//! - `ConnectionAborted` - Connection aborted
//! - `ConnectionRefused` - Connection refused
//! - `ConnectionReset` - Connection reset
//! - `NotConnected` - Not connected
//! - `AddrInUse` - Address in use
//! - `AddrNotAvailable` - Address not available
//! - `OutOfMemory` - Out of memory
//! - `IoError` - Generic I/O error with message
//! - `Other` - Uncategorized error with kind and message
//!
//! # Usage
//!
//! ```ignore
//! use hoop_daemon::file_io_error::{read_file_with_context, write_file_with_context};
//!
//! // Read with clear error messages
//! let content = read_file_with_context(Path::new("/path/to/file"))?;
//!
//! // Write with clear error messages
//! write_file_with_context(Path::new("/path/to/file"), "content")?;
//! ```

use anyhow::{Context, Result};
use std::io::ErrorKind;
use std::path::Path;

/// File I/O error with clear categorization
#[derive(Debug, Clone)]
pub enum FileIoError {
    /// File not found at the specified path
    NotFound(String),
    /// Permission denied when accessing the file
    PermissionDenied(String),
    /// File already exists at the specified path
    AlreadyExists(String),
    /// Invalid input provided for the operation
    InvalidInput(String),
    /// Invalid data format or content
    InvalidData(String),
    /// Path is a directory when file expected
    IsDirectory(String),
    /// Path is not a directory when directory expected
    NotDirectory(String),
    /// Directory not empty when removal attempted
    DirectoryNotEmpty(String),
    /// Unexpected end of file
    UnexpectedEof(String),
    /// Write returned zero bytes
    WriteZero(String),
    /// Operation was interrupted
    Interrupted(String),
    /// Operation timed out
    TimedOut(String),
    /// Broken pipe (write end of pipe closed)
    BrokenPipe(String),
    /// Operation would block (on non-blocking operations)
    WouldBlock(String),
    /// Storage full
    StorageFull(String),
    /// Network unreachable
    NetworkUnreachable(String),
    /// Network down
    NetworkDown(String),
    /// Connection aborted
    ConnectionAborted(String),
    /// Connection refused
    ConnectionRefused(String),
    /// Connection reset
    ConnectionReset(String),
    /// Not connected
    NotConnected(String),
    /// Address in use
    AddrInUse(String),
    /// Address not available
    AddrNotAvailable(String),
    /// Out of memory
    OutOfMemory(String),
    /// I/O error (device error, connection error, etc.)
    IoError(String, String),
    /// Other error with context
    Other(String, String),
}

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
            FileIoError::InvalidInput(path) => {
                write!(f, "Invalid input for operation on: {}", path)
            }
            FileIoError::InvalidData(path) => {
                write!(f, "Invalid data in file: {}", path)
            }
            FileIoError::IsDirectory(path) => {
                write!(f, "Path is a directory, not a file: {}", path)
            }
            FileIoError::NotDirectory(path) => {
                write!(f, "Path is not a directory: {}", path)
            }
            FileIoError::DirectoryNotEmpty(path) => {
                write!(f, "Directory not empty: {}", path)
            }
            FileIoError::UnexpectedEof(path) => {
                write!(f, "Unexpected end of file: {}", path)
            }
            FileIoError::WriteZero(path) => {
                write!(f, "Write returned zero bytes: {}", path)
            }
            FileIoError::Interrupted(path) => {
                write!(f, "Operation interrupted on: {}", path)
            }
            FileIoError::TimedOut(path) => {
                write!(f, "Operation timed out on: {}", path)
            }
            FileIoError::BrokenPipe(path) => {
                write!(f, "Broken pipe (connection closed) for: {}", path)
            }
            FileIoError::WouldBlock(path) => {
                write!(f, "Operation would block (non-blocking) for: {}", path)
            }
            FileIoError::StorageFull(path) => {
                write!(f, "Storage full while accessing: {}", path)
            }
            FileIoError::NetworkUnreachable(path) => {
                write!(f, "Network unreachable while accessing: {}", path)
            }
            FileIoError::NetworkDown(path) => {
                write!(f, "Network down while accessing: {}", path)
            }
            FileIoError::ConnectionAborted(path) => {
                write!(f, "Connection aborted while accessing: {}", path)
            }
            FileIoError::ConnectionRefused(path) => {
                write!(f, "Connection refused while accessing: {}", path)
            }
            FileIoError::ConnectionReset(path) => {
                write!(f, "Connection reset while accessing: {}", path)
            }
            FileIoError::NotConnected(path) => {
                write!(f, "Not connected while accessing: {}", path)
            }
            FileIoError::AddrInUse(path) => {
                write!(f, "Address in use: {}", path)
            }
            FileIoError::AddrNotAvailable(path) => {
                write!(f, "Address not available: {}", path)
            }
            FileIoError::OutOfMemory(path) => {
                write!(f, "Out of memory while accessing: {}", path)
            }
            FileIoError::IoError(path, msg) => {
                write!(f, "I/O error accessing '{}': {}", path, msg)
            }
            FileIoError::Other(path, msg) => {
                write!(f, "Error accessing '{}': {}", path, msg)
            }
        }
    }
}

impl std::error::Error for FileIoError {}

/// Convert a std::io::Error to a FileIoError with path context
pub fn classify_io_error(io_err: &std::io::Error, path: &Path) -> FileIoError {
    let path_str = path.display().to_string();

    match io_err.kind() {
        ErrorKind::NotFound => FileIoError::NotFound(path_str),
        ErrorKind::PermissionDenied => FileIoError::PermissionDenied(path_str),
        ErrorKind::AlreadyExists => FileIoError::AlreadyExists(path_str),
        ErrorKind::InvalidInput => FileIoError::InvalidInput(path_str),
        ErrorKind::InvalidData => FileIoError::InvalidData(path_str),
        ErrorKind::UnexpectedEof => FileIoError::UnexpectedEof(path_str),
        ErrorKind::Interrupted => FileIoError::Interrupted(path_str),
        ErrorKind::TimedOut => FileIoError::TimedOut(path_str),
        ErrorKind::BrokenPipe => FileIoError::BrokenPipe(path_str),
        ErrorKind::WouldBlock => FileIoError::WouldBlock(path_str),
        ErrorKind::WriteZero => FileIoError::WriteZero(path_str),
        ErrorKind::StorageFull => FileIoError::StorageFull(path_str),
        ErrorKind::NetworkUnreachable => FileIoError::NetworkUnreachable(path_str),
        ErrorKind::NetworkDown => FileIoError::NetworkDown(path_str),
        ErrorKind::ConnectionAborted => FileIoError::ConnectionAborted(path_str),
        ErrorKind::ConnectionRefused => FileIoError::ConnectionRefused(path_str),
        ErrorKind::ConnectionReset => FileIoError::ConnectionReset(path_str),
        ErrorKind::NotConnected => FileIoError::NotConnected(path_str),
        ErrorKind::AddrInUse => FileIoError::AddrInUse(path_str),
        ErrorKind::AddrNotAvailable => FileIoError::AddrNotAvailable(path_str),
        ErrorKind::NotADirectory => FileIoError::NotDirectory(path_str),
        ErrorKind::IsADirectory => FileIoError::IsDirectory(path_str),
        ErrorKind::DirectoryNotEmpty => FileIoError::DirectoryNotEmpty(path_str),
        ErrorKind::Other => {
            // Check if the error message indicates it's a directory
            if io_err.to_string().to_lowercase().contains("is a directory") {
                FileIoError::IsDirectory(path_str)
            } else {
                FileIoError::IoError(path_str, io_err.to_string())
            }
        }
        // Handle uncategorized errors with explicit error kind
        _ => FileIoError::Other(path_str, format!("{}: {}", io_err.kind(), io_err)),
    }
}

/// Read a file to string with explicit error handling
///
/// Returns a clear error message that includes the file path and the
/// specific type of failure (not found, permission denied, etc.)
///
/// # Examples
///
/// ```ignore
/// use hoop_daemon::file_io_error::read_file_with_context;
///
/// let content = read_file_with_context(Path::new("/path/to/file"))?;
/// ```
pub fn read_file_with_context(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).map_err(|e| {
        let file_error = classify_io_error(&e, path);
        anyhow::anyhow!("{}", file_error)
    })
}

/// Open a file with explicit error handling
///
/// Returns a clear error message that includes the file path and the
/// specific type of failure (not found, permission denied, etc.)
pub fn open_file_with_context(path: &Path) -> Result<std::fs::File> {
    std::fs::File::open(path).map_err(|e| {
        let file_error = classify_io_error(&e, path);
        anyhow::anyhow!("{}", file_error)
    })
}

/// Read a file to string, returning None if not found
///
/// This is useful for optional configuration files where absence is not
/// an error condition.
///
/// # Examples
///
/// ```ignore
/// use hoop_daemon::file_io_error::read_file_optional;
///
/// if let Some(content) = read_file_optional(Path::new("/optional/config"))? {
///     // Process the file content
/// }
/// ```
pub fn read_file_optional(path: &Path) -> Result<Option<String>> {
    match std::fs::read_to_string(path) {
        Ok(content) => Ok(Some(content)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => {
            let file_error = classify_io_error(&e, path);
            Err(anyhow::anyhow!("{}", file_error))
        }
    }
}

/// Open a file, returning None if not found
///
/// This is useful for optional files where absence is not an error condition.
pub fn open_file_optional(path: &Path) -> Result<Option<std::fs::File>> {
    match std::fs::File::open(path) {
        Ok(file) => Ok(Some(file)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => {
            let file_error = classify_io_error(&e, path);
            Err(anyhow::anyhow!("{}", file_error))
        }
    }
}

/// Write content to a file with explicit error handling
///
/// Returns a clear error message that includes the file path and the
/// specific type of failure (permission denied, etc.)
///
/// # Examples
///
/// ```ignore
/// use hoop_daemon::file_io_error::write_file_with_context;
///
/// write_file_with_context(Path::new("/path/to/file"), "content")?;
/// ```
pub fn write_file_with_context(path: &Path, content: &str) -> Result<()> {
    std::fs::write(path, content).map_err(|e| {
        let file_error = classify_io_error(&e, path);
        anyhow::anyhow!("{}", file_error)
    })
}

/// Create a new file with explicit error handling
///
/// Returns a clear error message that includes the file path and the
/// specific type of failure (permission denied, already exists, etc.)
pub fn create_file_with_context(path: &Path) -> Result<std::fs::File> {
    std::fs::File::create(path).map_err(|e| {
        let file_error = classify_io_error(&e, path);
        anyhow::anyhow!("{}", file_error)
    })
}

/// Create a new file exclusively (fail if exists) with explicit error handling
///
/// Returns a clear error message that includes the file path and the
/// specific type of failure (permission denied, already exists, etc.)
///
/// # Examples
///
/// ```ignore
/// use hoop_daemon::file_io_error::create_file_exclusive_with_context;
///
/// // This will fail with AlreadyExists if the file exists
/// let file = create_file_exclusive_with_context(Path::new("/path/to/file"))?;
/// ```
pub fn create_file_exclusive_with_context(path: &Path) -> Result<std::fs::File> {
    std::fs::File::create_new(path).map_err(|e| {
        let file_error = classify_io_error(&e, path);
        anyhow::anyhow!("{}", file_error)
    })
}

/// Create a directory with explicit error handling
///
/// Returns a clear error message that includes the directory path and the
/// specific type of failure (permission denied, already exists, etc.)
pub fn create_dir_with_context(path: &Path) -> Result<()> {
    std::fs::create_dir(path).map_err(|e| {
        let file_error = classify_io_error(&e, path);
        anyhow::anyhow!("{}", file_error)
    })
}

/// Create a directory and all parent directories with explicit error handling
///
/// Returns a clear error message that includes the directory path and the
/// specific type of failure (permission denied, etc.)
pub fn create_dir_all_with_context(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path).map_err(|e| {
        let file_error = classify_io_error(&e, path);
        anyhow::anyhow!("{}", file_error)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_read_file_with_context_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        let result = read_file_with_context(&file_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test content");
    }

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

    #[test]
    fn test_open_file_with_context_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        let result = open_file_with_context(&file_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_open_file_with_context_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nonexistent.txt");

        let result = open_file_with_context(&file_path);
        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("File not found"));
        assert!(err_msg.contains("nonexistent.txt"));
    }

    #[test]
    fn test_read_file_optional_found() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        let result = read_file_optional(&file_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("test content".to_string()));
    }

    #[test]
    fn test_read_file_optional_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nonexistent.txt");

        let result = read_file_optional(&file_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_read_file_optional_permission_denied() {
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

        let result = read_file_optional(&file_path);
        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        #[cfg(unix)]
        assert!(err_msg.contains("Permission") || err_msg.contains("permission"));
    }

    #[test]
    fn test_open_file_optional_found() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        let result = open_file_optional(&file_path);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_open_file_optional_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nonexistent.txt");

        let result = open_file_optional(&file_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

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

    #[test]
    fn test_classify_io_error_write_zero() {
        let io_err = std::io::Error::new(ErrorKind::WriteZero, "write returned zero bytes");
        let path = Path::new("/test/path.txt");
        let file_error = classify_io_error(&io_err, path);

        match file_error {
            FileIoError::WriteZero(p) => assert_eq!(p, "/test/path.txt"),
            _ => panic!("Expected WriteZero error"),
        }
    }

    #[test]
    fn test_classify_io_error_storage_full() {
        let io_err = std::io::Error::new(ErrorKind::StorageFull, "storage full");
        let path = Path::new("/test/path.txt");
        let file_error = classify_io_error(&io_err, path);

        match file_error {
            FileIoError::StorageFull(p) => assert_eq!(p, "/test/path.txt"),
            _ => panic!("Expected StorageFull error"),
        }
    }

    #[test]
    fn test_classify_io_error_network_unreachable() {
        let io_err = std::io::Error::new(ErrorKind::NetworkUnreachable, "network unreachable");
        let path = Path::new("/test/path.txt");
        let file_error = classify_io_error(&io_err, path);

        match file_error {
            FileIoError::NetworkUnreachable(p) => assert_eq!(p, "/test/path.txt"),
            _ => panic!("Expected NetworkUnreachable error"),
        }
    }

    #[test]
    fn test_classify_io_error_network_down() {
        let io_err = std::io::Error::new(ErrorKind::NetworkDown, "network down");
        let path = Path::new("/test/path.txt");
        let file_error = classify_io_error(&io_err, path);

        match file_error {
            FileIoError::NetworkDown(p) => assert_eq!(p, "/test/path.txt"),
            _ => panic!("Expected NetworkDown error"),
        }
    }

    #[test]
    fn test_classify_io_error_connection_aborted() {
        let io_err = std::io::Error::new(ErrorKind::ConnectionAborted, "connection aborted");
        let path = Path::new("/test/path.txt");
        let file_error = classify_io_error(&io_err, path);

        match file_error {
            FileIoError::ConnectionAborted(p) => assert_eq!(p, "/test/path.txt"),
            _ => panic!("Expected ConnectionAborted error"),
        }
    }

    #[test]
    fn test_classify_io_error_connection_refused() {
        let io_err = std::io::Error::new(ErrorKind::ConnectionRefused, "connection refused");
        let path = Path::new("/test/path.txt");
        let file_error = classify_io_error(&io_err, path);

        match file_error {
            FileIoError::ConnectionRefused(p) => assert_eq!(p, "/test/path.txt"),
            _ => panic!("Expected ConnectionRefused error"),
        }
    }

    #[test]
    fn test_classify_io_error_connection_reset() {
        let io_err = std::io::Error::new(ErrorKind::ConnectionReset, "connection reset");
        let path = Path::new("/test/path.txt");
        let file_error = classify_io_error(&io_err, path);

        match file_error {
            FileIoError::ConnectionReset(p) => assert_eq!(p, "/test/path.txt"),
            _ => panic!("Expected ConnectionReset error"),
        }
    }

    #[test]
    fn test_classify_io_error_not_connected() {
        let io_err = std::io::Error::new(ErrorKind::NotConnected, "not connected");
        let path = Path::new("/test/path.txt");
        let file_error = classify_io_error(&io_err, path);

        match file_error {
            FileIoError::NotConnected(p) => assert_eq!(p, "/test/path.txt"),
            _ => panic!("Expected NotConnected error"),
        }
    }

    #[test]
    fn test_classify_io_error_addr_in_use() {
        let io_err = std::io::Error::new(ErrorKind::AddrInUse, "address in use");
        let path = Path::new("/test/path.txt");
        let file_error = classify_io_error(&io_err, path);

        match file_error {
            FileIoError::AddrInUse(p) => assert_eq!(p, "/test/path.txt"),
            _ => panic!("Expected AddrInUse error"),
        }
    }

    #[test]
    fn test_classify_io_error_addr_not_available() {
        let io_err = std::io::Error::new(ErrorKind::AddrNotAvailable, "address not available");
        let path = Path::new("/test/path.txt");
        let file_error = classify_io_error(&io_err, path);

        match file_error {
            FileIoError::AddrNotAvailable(p) => assert_eq!(p, "/test/path.txt"),
            _ => panic!("Expected AddrNotAvailable error"),
        }
    }

    #[test]
    fn test_classify_io_error_not_directory() {
        let io_err = std::io::Error::new(ErrorKind::NotADirectory, "not a directory");
        let path = Path::new("/test/path.txt");
        let file_error = classify_io_error(&io_err, path);

        match file_error {
            FileIoError::NotDirectory(p) => assert_eq!(p, "/test/path.txt"),
            _ => panic!("Expected NotDirectory error"),
        }
    }

    #[test]
    fn test_classify_io_error_is_directory() {
        let io_err = std::io::Error::new(ErrorKind::IsADirectory, "is a directory");
        let path = Path::new("/test/dir");
        let file_error = classify_io_error(&io_err, path);

        match file_error {
            FileIoError::IsDirectory(p) => assert_eq!(p, "/test/dir"),
            _ => panic!("Expected IsDirectory error"),
        }
    }

    #[test]
    fn test_classify_io_error_directory_not_empty() {
        let io_err = std::io::Error::new(ErrorKind::DirectoryNotEmpty, "directory not empty");
        let path = Path::new("/test/dir");
        let file_error = classify_io_error(&io_err, path);

        match file_error {
            FileIoError::DirectoryNotEmpty(p) => assert_eq!(p, "/test/dir"),
            _ => panic!("Expected DirectoryNotEmpty error"),
        }
    }

    #[test]
    fn test_file_io_error_display() {
        let err = FileIoError::NotFound("/path/to/file.txt".to_string());
        assert_eq!(err.to_string(), "File not found: /path/to/file.txt");

        let err = FileIoError::PermissionDenied("/path/to/file.txt".to_string());
        assert_eq!(err.to_string(), "Permission denied: /path/to/file.txt");

        let err = FileIoError::AlreadyExists("/path/to/file.txt".to_string());
        assert_eq!(err.to_string(), "File already exists: /path/to/file.txt");

        let err = FileIoError::InvalidInput("/path/to/file.txt".to_string());
        assert_eq!(err.to_string(), "Invalid input for operation on: /path/to/file.txt");

        let err = FileIoError::InvalidData("/path/to/file.txt".to_string());
        assert_eq!(err.to_string(), "Invalid data in file: /path/to/file.txt");

        let err = FileIoError::IsDirectory("/path/to/dir".to_string());
        assert_eq!(err.to_string(), "Path is a directory, not a file: /path/to/dir");

        let err = FileIoError::NotDirectory("/path/to/file.txt".to_string());
        assert_eq!(err.to_string(), "Path is not a directory: /path/to/file.txt");

        let err = FileIoError::DirectoryNotEmpty("/path/to/dir".to_string());
        assert_eq!(err.to_string(), "Directory not empty: /path/to/dir");

        let err = FileIoError::UnexpectedEof("/path/to/file.txt".to_string());
        assert_eq!(err.to_string(), "Unexpected end of file: /path/to/file.txt");

        let err = FileIoError::WriteZero("/path/to/file.txt".to_string());
        assert_eq!(err.to_string(), "Write returned zero bytes: /path/to/file.txt");

        let err = FileIoError::Interrupted("/path/to/file.txt".to_string());
        assert_eq!(err.to_string(), "Operation interrupted on: /path/to/file.txt");

        let err = FileIoError::TimedOut("/path/to/file.txt".to_string());
        assert_eq!(err.to_string(), "Operation timed out on: /path/to/file.txt");

        let err = FileIoError::BrokenPipe("/path/to/file.txt".to_string());
        assert_eq!(err.to_string(), "Broken pipe (connection closed) for: /path/to/file.txt");

        let err = FileIoError::WouldBlock("/path/to/file.txt".to_string());
        assert_eq!(err.to_string(), "Operation would block (non-blocking) for: /path/to/file.txt");

        let err = FileIoError::StorageFull("/path/to/file.txt".to_string());
        assert_eq!(err.to_string(), "Storage full while accessing: /path/to/file.txt");

        let err = FileIoError::NetworkUnreachable("/path/to/file.txt".to_string());
        assert_eq!(err.to_string(), "Network unreachable while accessing: /path/to/file.txt");

        let err = FileIoError::NetworkDown("/path/to/file.txt".to_string());
        assert_eq!(err.to_string(), "Network down while accessing: /path/to/file.txt");

        let err = FileIoError::ConnectionAborted("/path/to/file.txt".to_string());
        assert_eq!(err.to_string(), "Connection aborted while accessing: /path/to/file.txt");

        let err = FileIoError::ConnectionRefused("/path/to/file.txt".to_string());
        assert_eq!(err.to_string(), "Connection refused while accessing: /path/to/file.txt");

        let err = FileIoError::ConnectionReset("/path/to/file.txt".to_string());
        assert_eq!(err.to_string(), "Connection reset while accessing: /path/to/file.txt");

        let err = FileIoError::NotConnected("/path/to/file.txt".to_string());
        assert_eq!(err.to_string(), "Not connected while accessing: /path/to/file.txt");

        let err = FileIoError::AddrInUse("/path/to/file.txt".to_string());
        assert_eq!(err.to_string(), "Address in use: /path/to/file.txt");

        let err = FileIoError::AddrNotAvailable("/path/to/file.txt".to_string());
        assert_eq!(err.to_string(), "Address not available: /path/to/file.txt");

        let err = FileIoError::OutOfMemory("/path/to/file.txt".to_string());
        assert_eq!(err.to_string(), "Out of memory while accessing: /path/to/file.txt");

        let err = FileIoError::IoError("/path/to/file.txt".to_string(), "device error".to_string());
        assert_eq!(err.to_string(), "I/O error accessing '/path/to/file.txt': device error");

        let err = FileIoError::Other("/path/to/file.txt".to_string(), "unknown error".to_string());
        assert_eq!(err.to_string(), "Error accessing '/path/to/file.txt': unknown error");
    }

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

    #[test]
    fn test_classify_io_error_invalid_input() {
        let io_err = std::io::Error::new(ErrorKind::InvalidInput, "invalid parameter");
        let path = Path::new("/test/path.txt");
        let file_error = classify_io_error(&io_err, path);

        match file_error {
            FileIoError::InvalidInput(p) => assert_eq!(p, "/test/path.txt"),
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_classify_io_error_invalid_data() {
        let io_err = std::io::Error::new(ErrorKind::InvalidData, "corrupted data");
        let path = Path::new("/test/path.txt");
        let file_error = classify_io_error(&io_err, path);

        match file_error {
            FileIoError::InvalidData(p) => assert_eq!(p, "/test/path.txt"),
            _ => panic!("Expected InvalidData error"),
        }
    }

    #[test]
    fn test_classify_io_error_unexpected_eof() {
        let io_err = std::io::Error::new(ErrorKind::UnexpectedEof, "unexpected end of file");
        let path = Path::new("/test/path.txt");
        let file_error = classify_io_error(&io_err, path);

        match file_error {
            FileIoError::UnexpectedEof(p) => assert_eq!(p, "/test/path.txt"),
            _ => panic!("Expected UnexpectedEof error"),
        }
    }

    #[test]
    fn test_classify_io_error_interrupted() {
        let io_err = std::io::Error::new(ErrorKind::Interrupted, "operation interrupted");
        let path = Path::new("/test/path.txt");
        let file_error = classify_io_error(&io_err, path);

        match file_error {
            FileIoError::Interrupted(p) => assert_eq!(p, "/test/path.txt"),
            _ => panic!("Expected Interrupted error"),
        }
    }

    #[test]
    fn test_classify_io_error_timed_out() {
        let io_err = std::io::Error::new(ErrorKind::TimedOut, "operation timed out");
        let path = Path::new("/test/path.txt");
        let file_error = classify_io_error(&io_err, path);

        match file_error {
            FileIoError::TimedOut(p) => assert_eq!(p, "/test/path.txt"),
            _ => panic!("Expected TimedOut error"),
        }
    }

    #[test]
    fn test_classify_io_error_broken_pipe() {
        let io_err = std::io::Error::new(ErrorKind::BrokenPipe, "broken pipe");
        let path = Path::new("/test/path.txt");
        let file_error = classify_io_error(&io_err, path);

        match file_error {
            FileIoError::BrokenPipe(p) => assert_eq!(p, "/test/path.txt"),
            _ => panic!("Expected BrokenPipe error"),
        }
    }

    #[test]
    fn test_classify_io_error_would_block() {
        let io_err = std::io::Error::new(ErrorKind::WouldBlock, "operation would block");
        let path = Path::new("/test/path.txt");
        let file_error = classify_io_error(&io_err, path);

        match file_error {
            FileIoError::WouldBlock(p) => assert_eq!(p, "/test/path.txt"),
            _ => panic!("Expected WouldBlock error"),
        }
    }

    #[test]
    fn test_write_file_with_context_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let result = write_file_with_context(&file_path, "test content");
        assert!(result.is_ok());

        // Verify the content was written
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "test content");
    }

    #[test]
    fn test_write_file_with_context_permission_denied() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("no_write.txt");
        fs::write(&file_path, "content").unwrap();

        // Remove write permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&file_path).unwrap().permissions();
            perms.set_mode(0o000);
            fs::set_permissions(&file_path, perms).unwrap();
        }

        let result = write_file_with_context(&file_path, "new content");
        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        #[cfg(unix)]
        assert!(err_msg.contains("Permission") || err_msg.contains("permission"));
    }

    #[test]
    fn test_create_file_with_context_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let result = create_file_with_context(&file_path);
        assert!(result.is_ok());

        // Verify the file was created
        assert!(file_path.exists());
    }

    #[test]
    fn test_create_file_with_context_already_exists() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "existing content").unwrap();

        // Creating a file that already exists should succeed (truncates)
        let result = create_file_with_context(&file_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_file_exclusive_with_context_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let result = create_file_exclusive_with_context(&file_path);
        assert!(result.is_ok());

        // Verify the file was created
        assert!(file_path.exists());
    }

    #[test]
    fn test_create_file_exclusive_with_context_already_exists() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "existing content").unwrap();

        // Exclusive creation should fail when file exists
        let result = create_file_exclusive_with_context(&file_path);
        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("already exists") || err_msg.contains("AlreadyExists"));
        assert!(err_msg.contains("test.txt"));
    }

    #[test]
    fn test_create_dir_with_context_success() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("test_dir");

        let result = create_dir_with_context(&dir_path);
        assert!(result.is_ok());

        // Verify the directory was created
        assert!(dir_path.exists() && dir_path.is_dir());
    }

    #[test]
    fn test_create_dir_with_context_already_exists() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("test_dir");
        fs::create_dir(&dir_path).unwrap();

        let result = create_dir_with_context(&dir_path);
        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("already exists") || err_msg.contains("AlreadyExists"));
    }

    #[test]
    fn test_create_dir_all_with_context_success() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("parent").join("child").join("grandchild");

        let result = create_dir_all_with_context(&dir_path);
        assert!(result.is_ok());

        // Verify all directories were created
        assert!(dir_path.exists() && dir_path.is_dir());
    }

    #[test]
    fn test_create_dir_all_with_context_permission_denied() {
        let temp_dir = TempDir::new().unwrap();
        // Create a directory with no permissions
        let parent_dir = temp_dir.path().join("no_access");
        fs::create_dir(&parent_dir).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&parent_dir).unwrap().permissions();
            perms.set_mode(0o000);
            fs::set_permissions(&parent_dir, perms).unwrap();
        }

        let dir_path = parent_dir.join("child");
        let result = create_dir_all_with_context(&dir_path);
        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        #[cfg(unix)]
        assert!(err_msg.contains("Permission") || err_msg.contains("permission"));
    }

    #[test]
    fn test_create_dir_all_with_context_already_exists() {
        let temp_dir = TempDir::new().unwrap();
        // Create a file where we'll try to create a directory
        let file_path = temp_dir.path().join("blocking_file");
        fs::write(&file_path, "existing file").unwrap();

        // Try to create a directory at the same path (should fail with AlreadyExists)
        let result = create_dir_all_with_context(&file_path);
        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("already exists") || err_msg.contains("AlreadyExists"));
        assert!(err_msg.contains("blocking_file"));
    }
}
