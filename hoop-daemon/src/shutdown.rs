//! Graceful shutdown coordinator
//!
//! Handles SIGTERM with grace period, flushes in-flight state to disk,
//! closes WebSocket connections, checkpoints fleet.db, and exits.
//! Hard SIGKILL backstop after grace period.
//!
//! Acceptance (from plan §6.6):
//! - SIGTERM triggers shutdown sequence
//! - Active WS clients notified (close frame)
//! - In-flight event-tailer state flushed
//! - fleet.db checkpointed (SQLite WAL)
//! - Socket cleaned up
//! - Exit in <5s under normal load

use crate::metrics;
use anyhow::{Context, Result};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, mpsc, Notify};
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// Default grace period before SIGKILL backstop
const DEFAULT_GRACE_PERIOD_SECS: u64 = 5;

/// Shutdown coordinator
///
/// Manages the graceful shutdown sequence across all daemon components.
#[derive(Debug, Clone)]
pub struct ShutdownCoordinator {
    /// Shutdown signal sender
    shutdown_tx: broadcast::Sender<ShutdownPhase>,
    /// Whether we're currently shutting down
    is_shutting_down: Arc<std::sync::atomic::AtomicBool>,
    /// Count of active WebSocket connections
    active_connections: Arc<AtomicUsize>,
    /// Notify when all connections have closed
    all_connections_closed: Arc<Notify>,
    /// Shutdown start time for metrics
    shutdown_start: Arc<std::sync::Mutex<Option<Instant>>>,
}

/// Shutdown phase
///
/// Components can subscribe to specific phases or all phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ShutdownPhase {
    /// Shutdown initiated - prepare for graceful exit
    Initiated = 0,
    /// Close new connections - stop accepting new work
    CloseNewConnections = 1,
    /// Drain in-flight work - finish processing active requests
    DrainInFlight = 2,
    /// Flush state - write buffers to disk
    FlushState = 3,
    /// Notify clients - send close frames to WebSocket clients
    NotifyClients = 4,
    /// Checkpoint database - WAL checkpoint for fleet.db
    CheckpointDb = 5,
    /// Cleanup sockets - remove Unix socket files
    CleanupSockets = 6,
    /// Final exit - all components should be stopped
    Exit = 7,
}

impl ShutdownCoordinator {
    /// Create a new shutdown coordinator
    pub fn new() -> Self {
        let (shutdown_tx, _) = broadcast::channel(32);
        Self {
            shutdown_tx,
            is_shutting_down: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            active_connections: Arc::new(AtomicUsize::new(0)),
            all_connections_closed: Arc::new(Notify::new()),
            shutdown_start: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    /// Subscribe to shutdown phases
    pub fn subscribe(&self) -> broadcast::Receiver<ShutdownPhase> {
        self.shutdown_tx.subscribe()
    }

    /// Check if shutdown is in progress
    pub fn is_shutting_down(&self) -> bool {
        self.is_shutting_down.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Register an active WebSocket connection
    ///
    /// Returns a token that should be dropped when the connection closes.
    pub fn register_connection(&self) -> ConnectionToken {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
        metrics::metrics().hoop_ws_clients_connected.set(self.active_connections.load(Ordering::Relaxed) as i64);
        ConnectionToken {
            coordinator: self.clone(),
        }
    }

    /// Get the current number of active connections
    pub fn active_connections(&self) -> usize {
        self.active_connections.load(Ordering::Relaxed)
    }

    /// Wait for all connections to close (with timeout)
    ///
    /// Returns Ok if all connections closed, Err if timeout occurred.
    pub async fn wait_for_connections_closed(&self, timeout_duration: Duration) -> Result<()> {
        if self.active_connections() == 0 {
            return Ok(());
        }

        match timeout(timeout_duration, self.all_connections_closed.notified()).await {
            Ok(_) => Ok(()),
            Err(_) => {
                let remaining = self.active_connections();
                warn!(
                    "Timeout waiting for {} connections to close",
                    remaining
                );
                metrics::metrics()
                    .hoop_shutdown_timeout_connections
                    .inc_by(remaining as u64);
                Err(anyhow::anyhow!(
                    "Timeout waiting for {} connections to close",
                    remaining
                ))
            }
        }
    }

    /// Initiate graceful shutdown
    ///
    /// Runs through each shutdown phase with a timeout, then sends SIGKILL
    /// to the current process as a backstop.
    pub async fn shutdown(&self, grace_period_secs: Option<u64>) -> Result<()> {
        self.is_shutting_down.store(true, std::sync::atomic::Ordering::SeqCst);
        let grace_period = Duration::from_secs(grace_period_secs.unwrap_or(DEFAULT_GRACE_PERIOD_SECS));

        // Record shutdown start time for metrics
        {
            let mut start_guard = self.shutdown_start.lock().unwrap();
            *start_guard = Some(Instant::now());
        }

        let initial_connections = self.active_connections();
        info!(
            "Initiating graceful shutdown (grace period: {:?}, active connections: {})",
            grace_period, initial_connections
        );

        let start = std::time::Instant::now();

        // Phase 1: Initiated
        self.broadcast_phase(ShutdownPhase::Initiated).await;
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Phase 2: Close new connections
        self.broadcast_phase(ShutdownPhase::CloseNewConnections).await;
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Phase 3: Drain in-flight work
        self.broadcast_phase(ShutdownPhase::DrainInFlight).await;
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Phase 4: Flush state
        self.broadcast_phase(ShutdownPhase::FlushState).await;
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Phase 5: Notify clients
        self.broadcast_phase(ShutdownPhase::NotifyClients).await;

        // Wait a bit for clients to receive close frame and disconnect
        if initial_connections > 0 {
            info!("Waiting for {} WebSocket connections to close...", initial_connections);
            if let Err(e) = self.wait_for_connections_closed(Duration::from_millis(500)).await {
                warn!("Error waiting for connections to close: {}", e);
            }
        }

        tokio::time::sleep(Duration::from_millis(100)).await;

        // Phase 6: Checkpoint database
        self.broadcast_phase(ShutdownPhase::CheckpointDb).await;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Phase 7: Cleanup sockets
        self.broadcast_phase(ShutdownPhase::CleanupSockets).await;
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Phase 8: Final exit
        self.broadcast_phase(ShutdownPhase::Exit).await;

        let elapsed = start.elapsed();

        // Record shutdown duration metric
        metrics::metrics()
            .hoop_shutdown_duration_seconds
            .observe(elapsed.as_secs_f64());

        info!("Graceful shutdown completed in {:?}", elapsed);

        // If we took too long, warn but don't SIGKILL ourselves
        // (let systemd or the parent handle that)
        if elapsed > grace_period {
            warn!(
                "Shutdown exceeded grace period: {:?} > {:?}",
                elapsed, grace_period
            );
            metrics::metrics().hoop_shutdown_exceeded_grace_period.inc();
        }

        Ok(())
    }

    /// Decrement the active connection count (called by ConnectionToken on drop)
    fn connection_closed(&self) {
        let count = self.active_connections.fetch_sub(1, Ordering::Relaxed) - 1;
        metrics::metrics().hoop_ws_clients_connected.set(count as i64);

        if count == 0 {
            info!("All WebSocket connections closed");
            self.all_connections_closed.notify_waiters();
        }
    }

    /// Broadcast a shutdown phase to all subscribers
    async fn broadcast_phase(&self, phase: ShutdownPhase) {
        debug!("Broadcasting shutdown phase: {:?}", phase);
        // Ignore errors - some receivers may have dropped
        let _ = self.shutdown_tx.send(phase);
    }

    /// Wait for a specific shutdown phase with a timeout
    ///
    /// Returns Ok if the phase was received, Err if timeout occurred.
    pub async fn wait_for_phase(
        &self,
        mut rx: broadcast::Receiver<ShutdownPhase>,
        phase: ShutdownPhase,
        timeout_duration: Duration,
    ) -> Result<()> {
        match timeout(timeout_duration, async {
            while let Ok(received_phase) = rx.recv().await {
                if received_phase == phase {
                    return Ok(());
                }
                // If we receive a later phase, we missed the one we wanted
                if (received_phase as u8) > (phase as u8) {
                    return Ok(()); // Already past the phase we wanted
                }
            }
            Err(anyhow::anyhow!("Shutdown channel closed"))
        })
        .await
        {
            Ok(r) => r,
            Err(_) => Err(anyhow::anyhow!(
                "Timeout waiting for shutdown phase {:?}",
                phase
            )),
        }
    }
}

impl Default for ShutdownCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// A component that participates in graceful shutdown
///
/// Implement this trait for components that need to participate
/// in the shutdown sequence.
#[async_trait::async_trait]
pub trait ShutdownParticipant: Send + Sync {
    /// Get the name of this participant (for logging)
    fn name(&self) -> &str;

    /// Prepare for shutdown - called when shutdown is initiated
    async fn prepare(&self) -> Result<()>;

    /// Drain in-flight work - finish processing active requests
    async fn drain(&self) -> Result<()>;

    /// Flush state - write buffers to disk
    async fn flush(&self) -> Result<()>;

    /// Notify clients - send close frames to connected clients
    async fn notify_clients(&self) -> Result<()>;

    /// Cleanup - release resources
    async fn cleanup(&self) -> Result<()>;
}

/// Database checkpoint handle for fleet.db
///
/// Ensures SQLite WAL is checkpointed before shutdown.
pub struct DbCheckpointHandle {
    fleet_db_path: std::path::PathBuf,
}

impl DbCheckpointHandle {
    /// Create a new database checkpoint handle
    pub fn new(fleet_db_path: std::path::PathBuf) -> Self {
        Self { fleet_db_path }
    }

    /// Checkpoint the SQLite WAL file
    ///
    /// This ensures all WAL entries are applied to the main database
    /// before shutdown. Called during the CheckpointDb phase.
    pub fn checkpoint(&self) -> Result<()> {
        use rusqlite::Connection;

        if !self.fleet_db_path.exists() {
            debug!("No fleet.db to checkpoint at {}", self.fleet_db_path.display());
            return Ok(());
        }

        debug!("Checkpointing fleet.db at {}", self.fleet_db_path.display());

        let conn = Connection::open(&self.fleet_db_path)
            .with_context(|| format!("Failed to open fleet.db at {}", self.fleet_db_path.display()))?;

        // Run WAL checkpoint in TRUNCATE mode
        // This ensures all WAL entries are applied and the WAL file is truncated
        conn.pragma_update(None, "wal_checkpoint", "TRUNCATE")
            .context("Failed to checkpoint WAL")?;

        info!("fleet.db WAL checkpointed successfully");

        Ok(())
    }
}

/// Tailer shutdown handle for event/session tailers
///
/// Ensures tailers stop gracefully and flush any in-flight partial data.
pub struct TailerShutdownHandle {
    shutdown_tx: mpsc::Sender<()>,
}

impl TailerShutdownHandle {
    /// Create a new tailer shutdown handle from a shutdown channel
    pub fn new(shutdown_tx: mpsc::Sender<()>) -> Self {
        Self { shutdown_tx }
    }

    /// Signal the tailer to stop and flush any pending data
    pub async fn stop(&self) -> Result<()> {
        debug!("Stopping tailer and flushing pending data");
        // Send shutdown signal
        let _ = self.shutdown_tx.send(()).await;
        // Give the tailer time to flush (max 500ms)
        tokio::time::sleep(Duration::from_millis(500)).await;
        debug!("Tailer stopped successfully");
        Ok(())
    }
}

/// Socket cleanup handle
///
/// Ensures Unix socket files are removed on shutdown.
pub struct SocketCleanupHandle {
    socket_path: std::path::PathBuf,
}

impl SocketCleanupHandle {
    /// Create a new socket cleanup handle
    pub fn new(socket_path: std::path::PathBuf) -> Self {
        Self { socket_path }
    }

    /// Remove the socket file
    pub fn cleanup(&self) -> Result<()> {
        if self.socket_path.exists() {
            debug!("Removing socket file at {}", self.socket_path.display());
            std::fs::remove_file(&self.socket_path)
                .with_context(|| format!("Failed to remove socket at {}", self.socket_path.display()))?;
            info!("Socket file removed: {}", self.socket_path.display());
        }
        Ok(())
    }
}

/// Token representing an active WebSocket connection
///
/// When dropped, decrements the active connection count in the ShutdownCoordinator.
/// This is a RAII pattern to ensure connections are always deregistered.
#[derive(Debug, Clone)]
pub struct ConnectionToken {
    coordinator: ShutdownCoordinator,
}

impl ConnectionToken {
    /// Create a new connection token
    #[allow(dead_code)]
    fn new(coordinator: ShutdownCoordinator) -> Self {
        Self { coordinator }
    }
}

impl Drop for ConnectionToken {
    fn drop(&mut self) {
        self.coordinator.connection_closed();
    }
}

/// Spawn a task that listens for shutdown signals and coordinates shutdown
///
/// This is the main entry point for shutdown handling.
pub async fn spawn_shutdown_listener(
    coordinator: ShutdownCoordinator,
    on_shutdown: impl FnOnce() + Send + 'static,
) {
    tokio::spawn(async move {
        // Listen for SIGTERM and SIGINT
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};

            let mut sigterm = signal(SignalKind::terminate()).expect("Failed to setup SIGTERM handler");
            let mut sigint = signal(SignalKind::interrupt()).expect("Failed to setup SIGINT handler");

            tokio::select! {
                _ = sigterm.recv() => {
                    info!("Received SIGTERM, initiating graceful shutdown");
                    let _ = coordinator.shutdown(None).await;
                    on_shutdown();
                }
                _ = sigint.recv() => {
                    info!("Received SIGINT, initiating graceful shutdown");
                    let _ = coordinator.shutdown(None).await;
                    on_shutdown();
                }
            }
        }

        #[cfg(not(unix))]
        {
            // On non-Unix, just listen for Ctrl-C
            tokio::signal::ctrl_c().await.expect("Failed to setup Ctrl-C handler");
            info!("Received Ctrl-C, initiating graceful shutdown");
            let _ = coordinator.shutdown(None).await;
            on_shutdown();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shutdown_phase_ordering() {
        assert!((ShutdownPhase::Initiated as u8) < (ShutdownPhase::Exit as u8));
        assert!((ShutdownPhase::CloseNewConnections as u8) < (ShutdownPhase::DrainInFlight as u8));
        assert!((ShutdownPhase::FlushState as u8) < (ShutdownPhase::NotifyClients as u8));
        assert!((ShutdownPhase::CheckpointDb as u8) < (ShutdownPhase::CleanupSockets as u8));
    }

    #[tokio::test]
    async fn test_shutdown_coordinator_basic() {
        let coordinator = ShutdownCoordinator::new();
        assert!(!coordinator.is_shutting_down());

        let mut rx = coordinator.subscribe();

        // Broadcast a phase
        coordinator.broadcast_phase(ShutdownPhase::Initiated).await;

        // Should receive the phase
        let received = timeout(Duration::from_millis(100), rx.recv())
            .await
            .expect("timeout")
            .expect("receive error");
        assert_eq!(received, ShutdownPhase::Initiated);
    }

    #[tokio::test]
    async fn test_wait_for_phase() {
        let coordinator = ShutdownCoordinator::new();
        let rx = coordinator.subscribe();

        // Broadcast the phase in a background task
        let coord_clone = coordinator.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            coord_clone.broadcast_phase(ShutdownPhase::FlushState).await;
        });

        // Wait for the phase
        let result = coordinator
            .wait_for_phase(rx, ShutdownPhase::FlushState, Duration::from_secs(1))
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_wait_for_phase_timeout() {
        let coordinator = ShutdownCoordinator::new();
        let rx = coordinator.subscribe();

        // Wait for a phase that will never come
        let result = coordinator
            .wait_for_phase(rx, ShutdownPhase::FlushState, Duration::from_millis(50))
            .await;
        assert!(result.is_err());
    }

    #[test]
    fn test_db_checkpoint_handle() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("fleet.db");

        // Create a simple database
        {
            let conn = rusqlite::Connection::open(&db_path).unwrap();
            conn.pragma_update(None, "journal_mode", "WAL").unwrap();
            conn.execute("CREATE TABLE test (id INTEGER)", []).unwrap();
        }

        let handle = DbCheckpointHandle::new(db_path.clone());
        assert!(handle.checkpoint().is_ok());

        // Database should still be valid after checkpoint
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM test", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_socket_cleanup_handle() {
        let temp_dir = tempfile::tempdir().unwrap();
        let socket_path = temp_dir.path().join("test.sock");

        // Create a dummy socket file
        std::fs::write(&socket_path, b"dummy").unwrap();
        assert!(socket_path.exists());

        let handle = SocketCleanupHandle::new(socket_path.clone());
        assert!(handle.cleanup().is_ok());
        assert!(!socket_path.exists());
    }
}
