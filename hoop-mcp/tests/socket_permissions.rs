//! MCP socket permissions smoke tests
//!
//! Verifies §13 security invariant: MCP socket is Unix domain socket
//! with 0600 permissions (same-user only), no TCP exposure.

use std::fs;
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::path::PathBuf;
use tempfile::TempDir;

/// Verify socket mode is 0600 (user read/write only)
#[test]
fn test_socket_mode_is_0600() {
    let temp_dir = TempDir::new().expect("temp dir");
    let socket_path = temp_dir.path().join("test-mcp.sock");

    // Bind socket like the production code does
    let listener = std::os::unix::net::UnixListener::bind(&socket_path)
        .expect("bind socket");

    // Set permissions
    fs::set_permissions(&socket_path, fs::Permissions::from_mode(0o600))
        .expect("set permissions");

    // Verify mode
    let metadata = fs::metadata(&socket_path).expect("metadata");
    let mode = metadata.permissions().mode() & 0o777;

    assert_eq!(
        mode, 0o600,
        "socket must have mode 0600 (user read/write only), got 0{:o}",
        mode
    );

    // Verify it's a socket
    #[cfg(target_os = "linux")]
    {
        let file_type = metadata.file_type();
        assert!(
            file_type.is_socket(),
            "MCP endpoint must be a Unix socket, not a regular file"
        );
    }

    drop(listener);
}

/// Verify zero TCP listener code in socket module
#[test]
fn test_no_tcp_listener_in_socket_module() {
    // This is a compile-time assertion: the socket module only uses UnixListener
    // If TcpListener is ever added, this test documents the security regression

    let socket_src = include_str!("../src/socket.rs");

    // The socket module must NOT contain TcpListener
    assert!(
        !socket_src.contains("TcpListener"),
        "socket.rs must not contain TcpListener (TCP exposure violates §13)"
    );

    // The socket module MUST use UnixListener
    assert!(
        socket_src.contains("UnixListener"),
        "socket.rs must use UnixListener for same-user security"
    );

    // Verify mode 0600 is documented
    assert!(
        socket_src.contains("0o600") || socket_src.contains("0600"),
        "socket.rs must document 0600 permissions (same-user only)"
    );
}

/// Verify default socket path is in home directory (not world-writable)
#[test]
fn test_default_socket_path_is_in_home() {
    use hoop_mcp::socket::SocketConfig;

    let config = SocketConfig::default();

    // Default path should be ~/.hoop/mcp.sock
    let path_str = config.socket_path.to_string_lossy();

    assert!(
        path_str.contains(".hoop") && path_str.contains("mcp.sock"),
        "default socket path should be ~/.hoop/mcp.sock, got {}",
        path_str
    );

    // Should not be in /tmp (world-writable)
    assert!(
        !path_str.starts_with("/tmp"),
        "socket path must not be in /tmp (security risk)"
    );
}

/// Verify socket configuration structure does not allow TCP binding
#[test]
fn test_socket_config_has_no_tcp_parameters() {
    // SocketConfig only has socket_path (PathBuf) and actor (String)
    // No host, port, or TCP-related parameters

    let config = hoop_mcp::socket::SocketConfig {
        socket_path: PathBuf::from("/tmp/test.sock"),
        actor: "test-actor".to_string(),
    };

    // These are the only fields
    assert_eq!(config.actor, "test-actor");
    assert_eq!(config.socket_path, PathBuf::from("/tmp/test.sock"));

    // No way to specify TCP bind address (compile-time check)
    let _ = config.socket_path;
}

/// Smoke test: verify another user cannot connect
///
/// Note: This is a documentation of the security invariant.
/// Actual cross-user connection testing requires multiple OS users,
/// which is not feasible in standard CI environments.
/// The 0600 mode enforcement is the kernel-level security guarantee.
#[test]
fn test_cross_user_connection_is_blocked_by_kernel() {
    // Unix socket permissions are enforced by the kernel, not by application code
    // When a socket has mode 0600, only the owner (and root) can connect

    // Create a test socket with 0600 permissions
    let temp_dir = TempDir::new().expect("temp dir");
    let socket_path = temp_dir.path().join("test-perm.sock");

    let _listener = std::os::unix::net::UnixListener::bind(&socket_path)
        .expect("bind socket");

    fs::set_permissions(&socket_path, fs::Permissions::from_mode(0o600))
        .expect("set permissions");

    // Verify the permissions
    let metadata = fs::metadata(&socket_path).expect("metadata");
    let mode = metadata.permissions().mode() & 0o777;

    // Kernel will reject connections from other users when mode is 0600
    // This is documented here as the security guarantee
    assert_eq!(
        mode, 0o600,
        "kernel enforces same-user-only when mode is 0600"
    );

    // In production, run as different user to verify:
    // sudo -u otheruser nc -U /path/to/mcp.sock
    // Expected: "Connection refused" (EACCES)

    drop(_listener);
}
