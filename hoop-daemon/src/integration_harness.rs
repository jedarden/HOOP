//! Integration test harness for spawning test daemon instances
//!
//! Provides utilities for integration tests to spawn temporary daemon instances
//! with isolated state (temporary directories, random ports, etc.).
//!
//! ## Example
//!
//! ```rust
//! use hoop_daemon::integration_harness::spawn_test_daemon;
//!
//! #[tokio::test]
//! async fn my_test() {
//!     let (base_url, _shutdown, _temp_dir) = spawn_test_daemon()
//!         .await
//!         .expect("Failed to spawn daemon");
//!
//!     // Make API calls to base_url
//!     let resp = reqwest::get(format!("{}/healthz", base_url)).await;
//!     assert!(resp.is_ok());
//! }
//! ```

use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use tokio::task::JoinHandle;
use tracing::debug;

use crate::Config;

/// Test daemon handle
///
/// Contains the base URL for API calls and a shutdown handle.
pub struct TestDaemon {
    /// Base URL for API calls (e.g., "http://127.0.0.1:54321")
    pub base_url: String,
    /// Join handle for the daemon task
    pub handle: JoinHandle<anyhow::Result<()>>,
    /// Temporary directory (kept alive for test inspection)
    pub temp_dir: TempDir,
    /// Bind address the daemon is listening on
    pub bind_addr: SocketAddr,
}

/// Shutdown handle for terminating the test daemon
///
/// When dropped, sends a shutdown signal to the daemon.
impl Drop for TestDaemon {
    fn drop(&mut self) {
        debug!("Shutting down test daemon at {}", self.base_url);
        self.handle.abort();
    }
}

/// Spawn a test daemon instance for integration testing
///
/// Creates a temporary directory structure with:
/// - `.hoop/` directory
/// - `.hoop/projects.yaml` (minimal configuration)
/// - `.hoop/fleet.db` (initialized)
///
/// The daemon is started on a random port to avoid conflicts.
///
/// # Returns
///
/// A tuple of (base_url, TestDaemon). The TestDaemon handle will
/// automatically shut down the daemon when dropped.
///
/// # Example
///
/// ```rust
/// let (base_url, daemon) = spawn_test_daemon().await?;
/// // daemon shuts down automatically when dropped
/// ```
pub async fn spawn_test_daemon() -> anyhow::Result<(String, TestDaemon)> {
    spawn_test_daemon_with_config::<fn(&mut Config)>(None).await
}

/// Spawn a test daemon with a custom configuration
///
/// # Arguments
///
/// * `config_fn` - Optional function to customize the daemon config before startup
///
/// # Returns
///
/// A tuple of (base_url, TestDaemon)
pub async fn spawn_test_daemon_with_config<F>(
    config_fn: Option<F>,
) -> anyhow::Result<(String, TestDaemon)>
where
    F: FnOnce(&mut Config) + Send + 'static,
{
    // Create temporary directory for test state
    let temp_dir = tempfile::TempDir::new()?;
    let hoop_dir = temp_dir.path().join(".hoop");
    fs::create_dir_all(&hoop_dir)?;

    // Create minimal projects.yaml
    let projects_path = hoop_dir.join("projects.yaml");
    let projects_yaml = r#"---
projects: []
"#;
    fs::write(&projects_path, projects_yaml)?;

    // Set environment variable to point to temp directory
    // This ensures the daemon uses our test config
    let home_override = temp_dir.path().to_path_buf();
    std::env::set_var("HOME", home_override.as_os_str());

    // Find an available port
    let bind_addr = find_available_port()?;
    let control_socket_path = hoop_dir.join("control.sock");

    // Create daemon config
    let mut config = Config {
        bind_addr,
        control_socket_path,
        allow_br_mismatch: true, // Allow for tests
        observer_mode: false,
        primary_addr: SocketAddr::from(([127, 0, 0, 1], 3000)),
    };

    // Apply custom config if provided
    if let Some(f) = config_fn {
        f(&mut config);
    }

    let base_url = format!("http://{}", bind_addr);

    debug!("Spawning test daemon at {}", base_url);

    // Spawn the daemon in a background task
    let handle = tokio::spawn(async move {
        crate::serve(config).await
    });

    // Wait for daemon to be ready
    wait_for_daemon_ready(&base_url).await?;

    let daemon = TestDaemon {
        base_url: base_url.clone(),
        handle,
        temp_dir,
        bind_addr,
    };

    Ok((base_url, daemon))
}

/// Find an available port on localhost
///
/// Tries to bind to ports in the 50000-60000 range to find an available one.
fn find_available_port() -> anyhow::Result<SocketAddr> {
    use std::net::TcpListener;

    // Try a few random ports in the test range
    for _ in 0..100 {
        let port = 50000 + (rand::random::<u16>() % 10000);
        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        if TcpListener::bind(addr).is_ok() {
            return Ok(addr);
        }
    }

    // Fallback to OS-assigned port
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;
    Ok(addr)
}

/// Wait for the daemon to be ready
///
/// Polls the health endpoint until it returns success or times out.
async fn wait_for_daemon_ready(base_url: &str) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let health_url = format!("{}/healthz", base_url);
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(30);

    while start.elapsed() < timeout {
        match client.get(&health_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                debug!("Daemon ready at {}", base_url);
                return Ok(());
            }
            Ok(_) => {
                // Not ready yet, wait a bit
            }
            Err(_) => {
                // Connection error, daemon not up yet
            }
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    anyhow::bail!("Daemon did not become ready within {:?} at {}", timeout, base_url);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_spawn_and_shutdown() {
        let (base_url, daemon) = spawn_test_daemon()
            .await
            .expect("Failed to spawn daemon");

        // Verify health endpoint works
        let client = reqwest::Client::new();
        let resp = client
            .get(format!("{}/healthz", base_url))
            .send()
            .await
            .expect("Health check failed");

        assert!(resp.status().is_success());

        // Daemon shuts down when daemon is dropped
        drop(daemon);
    }

    #[tokio::test]
    async fn test_spawn_with_custom_config() {
        let (base_url, _daemon) = spawn_test_daemon_with_config::<fn(&mut Config)>(Some(|config| {
            // Customize the config
            config.allow_br_mismatch = true;
        }))
        .await
        .expect("Failed to spawn daemon");

        // Verify daemon is accessible
        let client = reqwest::Client::new();
        let resp = client
            .get(format!("{}/healthz", base_url))
            .send()
            .await
            .expect("Health check failed");

        assert!(resp.status().is_success());
    }
}
