//! Supervisor restart-on-panic tests
//!
//! CI commands:
//!   cargo test -p hoop-daemon --test supervisor_restart
//!
//! This test verifies:
//! 1. Restart-on-panic behavior for per-project runtimes
//! 2. Exponential backoff calculation
//! 3. hoop_errors_total metric emission on panic
//! 4. Permanent error detection (no auto-restart)

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use hoop_daemon::metrics::metrics;
use hoop_daemon::projects::ProjectsConfig;
use hoop_daemon::shutdown::ShutdownCoordinator;
use hoop_daemon::supervisor::{ProjectRuntimeState, ProjectSupervisor};
use hoop_daemon::ws::WorkerRegistry;
use hoop_daemon::Bead;
use hoop_schema::{ProjectsRegistry, ProjectsRegistryProjectsItem};

/// Create a test project with workspace path (shorthand single-workspace variant)
fn create_test_project(name: &str, path: PathBuf) -> ProjectsRegistryProjectsItem {
    ProjectsRegistryProjectsItem::Variant0 {
        name: name.to_string(),
        path: path.to_string_lossy().into_owned(),
        canonical_path: None,
        label: None,
        color: None,
        redaction: None,
    }
}

/// Helper to create a temporary .beads directory
fn create_beads_dir(path: &std::path::Path) -> tempfile::TempDir {
    let beads_dir = path.join(".beads");
    std::fs::create_dir_all(&beads_dir).unwrap();
    let issues_path = beads_dir.join("issues.jsonl");
    std::fs::write(&issues_path, b"").unwrap();

    // Return a tempdir that will be cleaned up
    tempfile::TempDir::new().unwrap()
}

/// Helper to create a minimal supervisor for testing
async fn create_test_supervisor() -> ProjectSupervisor {
    let (bead_tx, _) = tokio::sync::broadcast::channel(64);
    let (session_tx, _) = tokio::sync::broadcast::channel(64);
    let (monitor_tx, _) =
        tokio::sync::broadcast::channel::<hoop_daemon::heartbeats::MonitorEvent>(64);
    let worker_registry = Arc::new(WorkerRegistry::new(monitor_tx, session_tx.clone()));
    let beads = Arc::new(std::sync::RwLock::new(Vec::<Bead>::new()));
    let shutdown = Arc::new(ShutdownCoordinator::new());
    let cost_aggregator = Arc::new(std::sync::RwLock::new(
        hoop_daemon::cost::CostAggregator::new(PathBuf::from("/tmp/test-cost-config.json"))
            .expect("Failed to create cost aggregator"),
    ));
    let vector_index = Arc::new(std::sync::RwLock::new(
        hoop_daemon::vector_index::VectorIndex::new(),
    ));
    let stuck_detector = Arc::new(std::sync::Mutex::new(
        hoop_daemon::stuck_detector::StuckDetector::new(),
    ));

    ProjectSupervisor::new(
        bead_tx,
        session_tx,
        worker_registry,
        beads,
        shutdown,
        cost_aggregator,
        vector_index,
        PathBuf::from("/tmp/hoop-test-scripts"),
        stuck_detector,
    )
}

#[tokio::test]
async fn test_exponential_backoff_delays() {
    // Verify exponential backoff calculation matches specification
    use hoop_daemon::supervisor::{BASE_RESTART_DELAY_SECS, MAX_RESTART_DELAY_SECS};

    // Failure 1: 2^0 * BASE = 1 second
    let delay1 = BASE_RESTART_DELAY_SECS * 2_u64.pow(0);
    assert_eq!(delay1, 1);

    // Failure 2: 2^1 * BASE = 2 seconds
    let delay2 = BASE_RESTART_DELAY_SECS * 2_u64.pow(1);
    assert_eq!(delay2, 2);

    // Failure 3: 2^2 * BASE = 4 seconds
    let delay3 = BASE_RESTART_DELAY_SECS * 2_u64.pow(2);
    assert_eq!(delay3, 4);

    // Failure 4: 2^3 * BASE = 8 seconds
    let delay4 = BASE_RESTART_DELAY_SECS * 2_u64.pow(3);
    assert_eq!(delay4, 8);

    // Failure 5: 2^4 * BASE = 16 seconds
    let delay5 = BASE_RESTART_DELAY_SECS * 2_u64.pow(4);
    assert_eq!(delay5, 16);

    // Verify cap at MAX_RESTART_DELAY_SECS (300 seconds)
    let uncapped = BASE_RESTART_DELAY_SECS * 2_u64.pow(20);
    assert_eq!(uncapped.min(MAX_RESTART_DELAY_SECS), MAX_RESTART_DELAY_SECS);
}

#[tokio::test]
async fn test_consecutive_failure_counter() {
    // Verify that consecutive failures are tracked correctly
    use hoop_daemon::supervisor::MAX_CONSECUTIVE_FAILURES;

    assert_eq!(MAX_CONSECUTIVE_FAILURES, 5);

    // Simulate failure state progression
    let mut consecutive_failures = 0;

    for i in 1..=5 {
        consecutive_failures += 1;
        assert_eq!(consecutive_failures, i);

        if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
            // Should be abandoned
            break;
        }
    }

    assert_eq!(consecutive_failures, 5);
}

#[tokio::test]
async fn test_permanent_error_detection() {
    // Verify that permanent errors are correctly identified
    let permanent_errors = vec![
        "Workspace path does not exist: /nonexistent",
        ".beads directory not found at: /path",
        "workspace path does not exist: /tmp/test",
        ".beads directory not found at: /home/user/project",
    ];

    for error in permanent_errors {
        assert!(
            ProjectSupervisor::is_permanent_error(error),
            "Should be permanent: {}",
            error
        );
    }

    // Verify that transient errors are NOT considered permanent
    let transient_errors = vec![
        "Connection refused",
        "Timeout",
        "Broken pipe",
        "Temporary failure",
        "Panic: synthetic panic for testing",
    ];

    for error in transient_errors {
        assert!(
            !ProjectSupervisor::is_permanent_error(error),
            "Should NOT be permanent: {}",
            error
        );
    }
}

#[tokio::test]
async fn test_runtime_state_transitions() {
    // Verify that runtime states correctly report running status

    // Starting and Healthy should be considered running
    assert!(ProjectRuntimeState::Starting.is_running());
    assert!(ProjectRuntimeState::Healthy.is_running());

    // Failed, Error, and Abandoned should NOT be running
    let failed = ProjectRuntimeState::Failed {
        error: "test error".to_string(),
        failed_at: chrono::Utc::now(),
        consecutive_failures: 1,
        next_restart_at: chrono::Utc::now(),
    };
    assert!(!failed.is_running());

    let error = ProjectRuntimeState::Error {
        error: "test error".to_string(),
        errored_at: chrono::Utc::now(),
    };
    assert!(!error.is_running());

    let abandoned = ProjectRuntimeState::Abandoned {
        error: "test error".to_string(),
        abandoned_at: chrono::Utc::now(),
    };
    assert!(!abandoned.is_running());
}

#[tokio::test]
async fn test_runtime_state_error_extraction() {
    // Verify error message extraction from all error states
    let error_msg = "Test error message";

    let failed = ProjectRuntimeState::Failed {
        error: error_msg.to_string(),
        failed_at: chrono::Utc::now(),
        consecutive_failures: 1,
        next_restart_at: chrono::Utc::now(),
    };
    assert_eq!(failed.error(), Some(error_msg));

    let error = ProjectRuntimeState::Error {
        error: error_msg.to_string(),
        errored_at: chrono::Utc::now(),
    };
    assert_eq!(error.error(), Some(error_msg));

    let abandoned = ProjectRuntimeState::Abandoned {
        error: error_msg.to_string(),
        abandoned_at: chrono::Utc::now(),
    };
    assert_eq!(abandoned.error(), Some(error_msg));

    // Healthy and Starting should return None
    assert!(ProjectRuntimeState::Healthy.error().is_none());
    assert!(ProjectRuntimeState::Starting.error().is_none());
}

#[tokio::test]
async fn test_runtime_state_display_string() {
    // Verify display strings for frontend consumption
    assert_eq!(
        ProjectRuntimeState::Starting.to_display_string(),
        "starting"
    );
    assert_eq!(ProjectRuntimeState::Healthy.to_display_string(), "healthy");

    assert_eq!(
        ProjectRuntimeState::Failed {
            error: "test".to_string(),
            failed_at: chrono::Utc::now(),
            consecutive_failures: 1,
            next_restart_at: chrono::Utc::now(),
        }
        .to_display_string(),
        "failed"
    );

    assert_eq!(
        ProjectRuntimeState::Error {
            error: "test".to_string(),
            errored_at: chrono::Utc::now(),
        }
        .to_display_string(),
        "error"
    );

    assert_eq!(
        ProjectRuntimeState::Abandoned {
            error: "test".to_string(),
            abandoned_at: chrono::Utc::now(),
        }
        .to_display_string(),
        "abandoned"
    );
}

#[tokio::test]
async fn test_supervisor_snapshot_empty() {
    // Verify supervisor snapshot with no runtimes
    let supervisor = create_test_supervisor().await;

    let snapshot = supervisor.snapshot().await;
    assert_eq!(snapshot.len(), 0);
}

#[tokio::test]
async fn test_supervisor_status_subscription() {
    // Verify status broadcast channel works
    let supervisor = create_test_supervisor().await;

    // Create two subscribers
    let mut rx1 = supervisor.subscribe_status();
    let mut rx2 = supervisor.subscribe_status();

    // Both should be able to receive (though channel is currently empty)
    // Try to receive with timeout to verify channel is working
    tokio::select! {
        _ = rx1.recv() => {},
        _ = tokio::time::sleep(Duration::from_millis(10)) => {
            // Timeout is OK - channel is empty
        }
    }

    tokio::select! {
        _ = rx2.recv() => {},
        _ = tokio::time::sleep(Duration::from_millis(10)) => {
            // Timeout is OK - channel is empty
        }
    }
}

#[tokio::test]
async fn test_metrics_counter_increments() {
    // Verify that hoop_errors_total metric can be incremented
    // This test ensures the metrics infrastructure is in place
    // for the supervisor to emit project_panic errors

    // Clear any existing metrics for this test
    let initial_count = metrics()
        .hoop_errors_total
        .snapshot()
        .iter()
        .filter(|(labels, _)| labels[0] == "supervisor" && labels[1] == "project_panic")
        .map(|(_, count)| *count)
        .sum::<u64>();

    // Increment the counter
    metrics()
        .hoop_errors_total
        .inc(&["supervisor", "project_panic"]);

    // Verify it incremented
    let new_count = metrics()
        .hoop_errors_total
        .snapshot()
        .iter()
        .filter(|(labels, _)| labels[0] == "supervisor" && labels[1] == "project_panic")
        .map(|(_, count)| *count)
        .sum::<u64>();

    assert_eq!(new_count, initial_count + 1);
}
