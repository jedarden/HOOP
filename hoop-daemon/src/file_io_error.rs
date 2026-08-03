//! File I/O error handling utilities
//!
//! Provides consistent error handling for file operations with explicit
//! NotFound handling and clear error messages that include file paths.

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
    /// Directory instead of a file
    IsDirectory(String),
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
            FileIoError::IsDirectory(path) => {
                write!(f, "Path is a directory, not a file: {}", path)
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
        ErrorKind::Other => {
            // Check if the error message indicates it's a directory
            if io_err.to_string().to_lowercase().contains("is a directory") {
                FileIoError::IsDirectory(path_str)
            } else {
                FileIoError::IoError(path_str, io_err.to_string())
            }
        }
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
    fn test_classify_io_error_other() {
        let io_err = std::io::Error::new(ErrorKind::InvalidData, "invalid data");
        let path = Path::new("/test/path.txt");
        let file_error = classify_io_error(&io_err, path);

        match file_error {
            FileIoError::Other(p, _) => {
                assert_eq!(p, "/test/path.txt");
            }
            _ => panic!("Expected Other error"),
        }
    }

    #[test]
    fn test_file_io_error_display() {
        let err = FileIoError::NotFound("/path/to/file.txt".to_string());
        assert_eq!(err.to_string(), "File not found: /path/to/file.txt");

        let err = FileIoError::PermissionDenied("/path/to/file.txt".to_string());
        assert_eq!(err.to_string(), "Permission denied: /path/to/file.txt");

        let err = FileIoError::IsDirectory("/path/to/dir".to_string());
        assert_eq!(err.to_string(), "Path is a directory, not a file: /path/to/dir");

        let err = FileIoError::IoError("/path/to/file.txt".to_string(), "device error".to_string());
        assert_eq!(err.to_string(), "I/O error accessing '/path/to/file.txt': device error");
    }
}
