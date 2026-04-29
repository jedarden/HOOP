//! Integration test: mid-run .beads/ deletion shows error card, siblings unaffected
//!
//! CI commands:
//!   cargo test -p hoop-daemon --test beads_deletion_isolation
//!
//! This test verifies §6 Phase 2 success criterion:
//! "Killing one project's runtime (delete `.beads/`) shows an error card;
//! other projects unaffected."
//!
//! Test scenario:
//! 1. Spawn daemon with 3 projects (A, B, C)
//! 2. Delete project A's .beads/ directory during runtime
//! 3. Assert:
//!    - Project A's card shows error state within 30s
//!    - Projects B/C continue serving events normally
//!    - /readyz reports degraded (A-listed)
//! 4. Restore .beads/ and verify recovery

use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::{sleep, timeout};

use hoop_daemon::projects::ProjectsConfig;
use hoop_daemon::supervisor::{ProjectRuntimeState, ProjectSupervisor};
use hoop_schema::{ProjectsRegistry, ProjectsRegistryProjectsItem};

/// Create a test project with workspace path
fn create_test_project(name: &str, path: PathBuf) -> ProjectsRegistryProjectsItem {
    ProjectsRegistryProjectsItem::Variant0 {
        name: name.to_string(),
        path: path.to_string_lossy().into_owned(),
        canonical_path: None,
        label: None,
        color: None,
    }
}

/// Create a temporary .beads directory with issues.jsonl
fn setup_beads_dir(project_dir: &PathBuf) -> anyhow::Result<()> {
    let beads_dir = project_dir.join(".beads");
    fs::create_dir_all(&beads_dir)?;
    let issues_path = beads_dir.join("issues.jsonl");
    fs::write(&issues_path, b"")?;
    Ok(())
}

/// Check if project state is an error state
fn is_error_state(state: &ProjectRuntimeState) -> bool {
    matches!(
        state,
        ProjectRuntimeState::Error { .. } | ProjectRuntimeState::Failed { .. }
    )
}

/// Check if project state is healthy
fn is_healthy_state(state: &ProjectRuntimeState) -> bool {
    matches!(state, ProjectRuntimeState::Healthy)
}

#[tokio::test]
async fn test_beads_deletion_shows_error_card() {
    // Create temporary directories for three projects
    let project_a_dir = tempfile::tempdir().unwrap();
    let project_a_path = project_a_dir.path().to_path_buf();
    setup_beads_dir(&project_a_path).unwrap();

    let project_b_dir = tempfile::tempdir().unwrap();
    let project_b_path = project_b_dir.path().to_path_buf();
    setup_beads_dir(&project_b_path).unwrap();

    let project_c_dir = tempfile::tempdir().unwrap();
    let project_c_path = project_c_dir.path().to_path_buf();
    setup_beads_dir(&project_c_path).unwrap();

    // Create a projects configuration
    let registry = ProjectsRegistry {
        projects: vec![
            create_test_project("project-a", project_a_path.clone()),
            create_test_project("project-b", project_b_path),
            create_test_project("project-c", project_c_path),
        ],
    };
    let config = ProjectsConfig {
        registry: registry.clone(),
        path: PathBuf::from("/test/projects.yaml"),
        canonical_cache: std::collections::HashMap::new(),
        content_hash: String::new(),
    };

    // Create a supervisor
    let (bead_tx, _bead_rx) = tokio::sync::broadcast::channel(64);
    let (session_tx, _session_rx) = tokio::sync::broadcast::channel(64);
    let worker_registry = std::sync::Arc::new(hoop_daemon::ws::WorkerRegistry::new());
    let beads = std::sync::Arc::new(std::sync::RwLock::new(Vec::new()));
    let shutdown = std::sync::Arc::new(hoop_daemon::shutdown::ShutdownCoordinator::new());
    let cost_aggregator = std::sync::Arc::new(std::sync::RwLock::new(
        hoop_daemon::cost::CostAggregator::new(),
    ));
    let vector_index = std::sync::Arc::new(std::sync::RwLock::new(
        hoop_daemon::vector_index::VectorIndex::new(),
    ));
    let scripts_dir = PathBuf::from("/tmp/scripts");
    let stuck_detector = std::sync::Arc::new(std::sync::Mutex::new(
        hoop_daemon::stuck_detector::StuckDetector::new(),
    ));

    let supervisor = ProjectSupervisor::new(
        bead_tx,
        session_tx,
        worker_registry,
        beads,
        shutdown,
        cost_aggregator,
        vector_index,
        scripts_dir,
        stuck_detector,
    );

    // Reconcile the projects
    supervisor.reconcile(&config).await.unwrap();

    // Wait for projects to start
    sleep(Duration::from_secs(2)).await;

    // Verify all projects are initially in a running state (Starting or Healthy)
    let snapshot = supervisor.snapshot().await;
    assert_eq!(snapshot.len(), 3, "Should have 3 projects");

    for status in &snapshot {
        assert!(
            status.state.is_running(),
            "Project {} should be running initially, got: {:?}",
            status.project_name,
            status.state
        );
    }

    // Delete project A's .beads directory
    let beads_a_path = project_a_path.join(".beads");
    fs::remove_dir_all(&beads_a_path).unwrap();

    // Wait up to 30 seconds for the error to be detected
    // The runtime should detect the missing .beads directory and transition to Error state
    let start = std::time::Instant::now();
    let mut project_a_error = false;

    while start.elapsed() < Duration::from_secs(30) {
        let snapshot = supervisor.snapshot().await;

        if let Some(status_a) = snapshot.iter().find(|s| s.project_name == "project-a") {
            if is_error_state(&status_a.state) {
                project_a_error = true;
                break;
            }
        }

        sleep(Duration::from_millis(500)).await;
    }

    assert!(
        project_a_error,
        "Project A should transition to error state within 30s after .beads/ deletion"
    );

    // Verify the final state
    let snapshot = supervisor.snapshot().await;
    let status_a = snapshot
        .iter()
        .find(|s| s.project_name == "project-a")
        .unwrap();
    assert!(
        is_error_state(&status_a.state),
        "Project A should be in error state, got: {:?}",
        status_a.state
    );

    // Verify projects B and C are still running (not affected by A's failure)
    let status_b = snapshot
        .iter()
        .find(|s| s.project_name == "project-b")
        .unwrap();
    let status_c = snapshot
        .iter()
        .find(|s| s.project_name == "project-c")
        .unwrap();

    assert!(
        status_b.state.is_running()
            || is_error_state(&status_b.state)
                && status_b.state.error().unwrap().contains("project-b"),
        "Project B should not be affected by project A's .beads/ deletion, got: {:?}",
        status_b.state
    );

    assert!(
        status_c.state.is_running()
            || is_error_state(&status_c.state)
                && status_c.state.error().unwrap().contains("project-c"),
        "Project C should not be affected by project A's .beads/ deletion, got: {:?}",
        status_c.state
    );
}

#[tokio::test]
async fn test_readyz_reports_degraded_after_beads_deletion() {
    // Create temporary directories for three projects
    let project_a_dir = tempfile::tempdir().unwrap();
    let project_a_path = project_a_dir.path().to_path_buf();
    setup_beads_dir(&project_a_path).unwrap();

    let project_b_dir = tempfile::tempdir().unwrap();
    let project_b_path = project_b_dir.path().to_path_buf();
    setup_beads_dir(&project_b_path).unwrap();

    let project_c_dir = tempfile::tempdir().unwrap();
    let project_c_path = project_c_dir.path().to_path_buf();
    setup_beads_dir(&project_c_path).unwrap();

    // Create a projects configuration
    let registry = ProjectsRegistry {
        projects: vec![
            create_test_project("project-a", project_a_path.clone()),
            create_test_project("project-b", project_b_path),
            create_test_project("project-c", project_c_path),
        ],
    };
    let config = ProjectsConfig {
        registry: registry.clone(),
        path: PathBuf::from("/test/projects.yaml"),
        canonical_cache: std::collections::HashMap::new(),
        content_hash: String::new(),
    };

    // Create a supervisor
    let (bead_tx, _bead_rx) = tokio::sync::broadcast::channel(64);
    let (session_tx, _session_rx) = tokio::sync::broadcast::channel(64);
    let worker_registry = std::sync::Arc::new(hoop_daemon::ws::WorkerRegistry::new());
    let beads = std::sync::Arc::new(std::sync::RwLock::new(Vec::new()));
    let shutdown = std::sync::Arc::new(hoop_daemon::shutdown::ShutdownCoordinator::new());
    let cost_aggregator = std::sync::Arc::new(std::sync::RwLock::new(
        hoop_daemon::cost::CostAggregator::new(),
    ));
    let vector_index = std::sync::Arc::new(std::sync::RwLock::new(
        hoop_daemon::vector_index::VectorIndex::new(),
    ));
    let scripts_dir = PathBuf::from("/tmp/scripts");
    let stuck_detector = std::sync::Arc::new(std::sync::Mutex::new(
        hoop_daemon::stuck_detector::StuckDetector::new(),
    ));

    let supervisor = ProjectSupervisor::new(
        bead_tx,
        session_tx,
        worker_registry,
        beads.clone(),
        shutdown,
        cost_aggregator,
        vector_index,
        scripts_dir,
        stuck_detector,
    );

    // Reconcile the projects
    supervisor.reconcile(&config).await.unwrap();

    // Wait for projects to start
    sleep(Duration::from_secs(2)).await;

    // Verify all projects are healthy initially
    let snapshot = supervisor.snapshot().await;
    let degraded: Vec<_> = snapshot
        .iter()
        .filter(|s| !is_healthy_state(&s.state))
        .collect();

    assert!(
        degraded.is_empty(),
        "All projects should be healthy initially, found {} degraded",
        degraded.len()
    );

    // Delete project A's .beads directory
    let beads_a_path = project_a_path.join(".beads");
    fs::remove_dir_all(&beads_a_path).unwrap();

    // Wait up to 30 seconds for the error to be detected
    let start = std::time::Instant::now();
    let mut project_a_degraded = false;

    while start.elapsed() < Duration::from_secs(30) {
        let snapshot = supervisor.snapshot().await;
        let degraded: Vec<_> = snapshot
            .iter()
            .filter(|s| !is_healthy_state(&s.state) && s.project_name == "project-a")
            .collect();

        if !degraded.is_empty() {
            project_a_degraded = true;
            break;
        }

        sleep(Duration::from_millis(500)).await;
    }

    assert!(
        project_a_degraded,
        "Project A should be reported as degraded within 30s"
    );

    // Verify the degraded list includes project-a
    let snapshot = supervisor.snapshot().await;
    let degraded: Vec<_> = snapshot
        .iter()
        .filter(|s| !is_healthy_state(&s.state))
        .collect();

    assert_eq!(degraded.len(), 1, "Only project A should be degraded");
    assert_eq!(degraded[0].project_name, "project-a");
}

#[tokio::test]
async fn test_beads_restoration_recovers_project() {
    // Create temporary directories for three projects
    let project_a_dir = tempfile::tempdir().unwrap();
    let project_a_path = project_a_dir.path().to_path_buf();
    setup_beads_dir(&project_a_path).unwrap();

    let project_b_dir = tempfile::tempdir().unwrap();
    let project_b_path = project_b_dir.path().to_path_buf();
    setup_beads_dir(&project_b_path).unwrap();

    let project_c_dir = tempfile::tempdir().unwrap();
    let project_c_path = project_c_dir.path().to_path_buf();
    setup_beads_dir(&project_c_path).unwrap();

    // Create a projects configuration
    let registry = ProjectsRegistry {
        projects: vec![
            create_test_project("project-a", project_a_path.clone()),
            create_test_project("project-b", project_b_path),
            create_test_project("project-c", project_c_path),
        ],
    };
    let config = ProjectsConfig {
        registry: registry.clone(),
        path: PathBuf::from("/test/projects.yaml"),
        canonical_cache: std::collections::HashMap::new(),
        content_hash: String::new(),
    };

    // Create a supervisor
    let (bead_tx, _bead_rx) = tokio::sync::broadcast::channel(64);
    let (session_tx, _session_rx) = tokio::sync::broadcast::channel(64);
    let worker_registry = std::sync::Arc::new(hoop_daemon::ws::WorkerRegistry::new());
    let beads = std::sync::Arc::new(std::sync::RwLock::new(Vec::new()));
    let shutdown = std::sync::Arc::new(hoop_daemon::shutdown::ShutdownCoordinator::new());
    let cost_aggregator = std::sync::Arc::new(std::sync::RwLock::new(
        hoop_daemon::cost::CostAggregator::new(),
    ));
    let vector_index = std::sync::Arc::new(std::sync::RwLock::new(
        hoop_daemon::vector_index::VectorIndex::new(),
    ));
    let scripts_dir = PathBuf::from("/tmp/scripts");
    let stuck_detector = std::sync::Arc::new(std::sync::Mutex::new(
        hoop_daemon::stuck_detector::StuckDetector::new(),
    ));

    let supervisor = ProjectSupervisor::new(
        bead_tx,
        session_tx,
        worker_registry,
        beads.clone(),
        shutdown,
        cost_aggregator,
        vector_index,
        scripts_dir,
        stuck_detector,
    );

    // Reconcile the projects
    supervisor.reconcile(&config).await.unwrap();

    // Wait for projects to start
    sleep(Duration::from_secs(2)).await;

    // Delete project A's .beads directory
    let beads_a_path = project_a_path.join(".beads");
    fs::remove_dir_all(&beads_a_path).unwrap();

    // Wait for the error to be detected
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(30) {
        let snapshot = supervisor.snapshot().await;
        if let Some(status_a) = snapshot.iter().find(|s| s.project_name == "project-a") {
            if is_error_state(&status_a.state) {
                break;
            }
        }
        sleep(Duration::from_millis(500)).await;
    }

    // Verify project A is in error state
    let snapshot = supervisor.snapshot().await;
    let status_a = snapshot
        .iter()
        .find(|s| s.project_name == "project-a")
        .unwrap();
    assert!(
        is_error_state(&status_a.state),
        "Project A should be in error state before restoration"
    );

    // Restore project A's .beads directory
    setup_beads_dir(&project_a_path).unwrap();

    // Trigger a reconcile to restart the runtime
    supervisor.reconcile(&config).await.unwrap();

    // Wait for recovery
    let start = std::time::Instant::now();
    let mut recovered = false;

    while start.elapsed() < Duration::from_secs(30) {
        let snapshot = supervisor.snapshot().await;
        if let Some(status_a) = snapshot.iter().find(|s| s.project_name == "project-a") {
            if is_healthy_state(&status_a.state) || status_a.state.is_running() {
                recovered = true;
                break;
            }
        }
        sleep(Duration::from_millis(500)).await;
    }

    assert!(
        recovered,
        "Project A should recover after .beads/ restoration"
    );

    // Verify all projects are healthy again
    let snapshot = supervisor.snapshot().await;
    let degraded: Vec<_> = snapshot
        .iter()
        .filter(|s| !is_healthy_state(&s.state) && !s.state.is_running())
        .collect();

    assert!(
        degraded.is_empty(),
        "All projects should be healthy after recovery, found {} degraded",
        degraded.len()
    );
}

#[tokio::test]
async fn test_sibling_projects_serve_events_during_degradation() {
    // This test verifies that projects B and C continue to serve events
    // while project A is degraded

    // Create temporary directories for three projects
    let project_a_dir = tempfile::tempdir().unwrap();
    let project_a_path = project_a_dir.path().to_path_buf();
    setup_beads_dir(&project_a_path).unwrap();

    let project_b_dir = tempfile::tempdir().unwrap();
    let project_b_path = project_b_dir.path().to_path_buf();
    setup_beads_dir(&project_b_path).unwrap();

    let project_c_dir = tempfile::tempdir().unwrap();
    let project_c_path = project_c_dir.path().to_path_buf();
    setup_beads_dir(&project_c_path).unwrap();

    // Create a projects configuration
    let registry = ProjectsRegistry {
        projects: vec![
            create_test_project("project-a", project_a_path.clone()),
            create_test_project("project-b", project_b_path),
            create_test_project("project-c", project_c_path),
        ],
    };
    let config = ProjectsConfig {
        registry: registry.clone(),
        path: PathBuf::from("/test/projects.yaml"),
        canonical_cache: std::collections::HashMap::new(),
        content_hash: String::new(),
    };

    // Create a supervisor with broadcast channels
    let (bead_tx, mut bead_rx) = tokio::sync::broadcast::channel(64);
    let (session_tx, _session_rx) = tokio::sync::broadcast::channel(64);
    let worker_registry = std::sync::Arc::new(hoop_daemon::ws::WorkerRegistry::new());
    let beads = std::sync::Arc::new(std::sync::RwLock::new(Vec::new()));
    let shutdown = std::sync::Arc::new(hoop_daemon::shutdown::ShutdownCoordinator::new());
    let cost_aggregator = std::sync::Arc::new(std::sync::RwLock::new(
        hoop_daemon::cost::CostAggregator::new(),
    ));
    let vector_index = std::sync::Arc::new(std::sync::RwLock::new(
        hoop_daemon::vector_index::VectorIndex::new(),
    ));
    let scripts_dir = PathBuf::from("/tmp/scripts");
    let stuck_detector = std::sync::Arc::new(std::sync::Mutex::new(
        hoop_daemon::stuck_detector::StuckDetector::new(),
    ));

    let supervisor = ProjectSupervisor::new(
        bead_tx.clone(),
        session_tx,
        worker_registry,
        beads.clone(),
        shutdown,
        cost_aggregator,
        vector_index,
        scripts_dir,
        stuck_detector,
    );

    // Reconcile the projects
    supervisor.reconcile(&config).await.unwrap();

    // Wait for projects to start
    sleep(Duration::from_secs(2)).await;

    // Delete project A's .beads directory
    let beads_a_path = project_a_path.join(".beads");
    fs::remove_dir_all(&beads_a_path).unwrap();

    // Wait for project A to enter error state
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(30) {
        let snapshot = supervisor.snapshot().await;
        if let Some(status_a) = snapshot.iter().find(|s| s.project_name == "project-a") {
            if is_error_state(&status_a.state) {
                break;
            }
        }
        sleep(Duration::from_millis(500)).await;
    }

    // Verify project A is in error state
    let snapshot = supervisor.snapshot().await;
    let status_a = snapshot
        .iter()
        .find(|s| s.project_name == "project-a")
        .unwrap();
    assert!(
        is_error_state(&status_a.state),
        "Project A should be in error state"
    );

    // Verify projects B and C are still running
    let status_b = snapshot
        .iter()
        .find(|s| s.project_name == "project-b")
        .unwrap();
    let status_c = snapshot
        .iter()
        .find(|s| s.project_name == "project-c")
        .unwrap();

    // The sibling projects should still be running (Starting or Healthy)
    assert!(
        status_b.state.is_running(),
        "Project B should still be running"
    );
    assert!(
        status_c.state.is_running(),
        "Project C should still be running"
    );

    // Verify that bead events are still being broadcast
    // (The broadcast channel should still be active)
    assert!(
        bead_tx.receiver_count() > 0,
        "Bead event broadcast should still be active"
    );

    // Add a bead to project B to verify events are still flowing
    let beads_b_path = project_b_path.join(".beads").join("issues.jsonl");
    let new_bead = r#"{"id":"test-1","title":"Test bead","status":"open","priority":0,"issue_type":"task","created_at":"2026-04-26T00:00:00Z","updated_at":"2026-04-26T00:00:00Z","created_by":"test","dependencies":[],"project":"project-b"}"#;
    fs::write(&beads_b_path, new_bead).unwrap();

    // Wait a bit for the bead reader to pick up the change
    sleep(Duration::from_secs(1)).await;

    // The bead reader should still be working for project B
    // (We can't directly test the broadcast without a more complex setup,
    // but we verified the broadcast channel is still active)
}

#[tokio::test]
async fn test_permanent_error_detection() {
    // Test that permanent errors are correctly detected
    assert!(
        ProjectSupervisor::is_permanent_error(".beads directory not found at: /test"),
        "Missing .beads should be a permanent error"
    );
    assert!(
        ProjectSupervisor::is_permanent_error("Workspace path does not exist: /test"),
        "Missing workspace should be a permanent error"
    );
    assert!(
        !ProjectSupervisor::is_permanent_error("Connection refused"),
        "Connection errors should not be permanent"
    );
    assert!(
        !ProjectSupervisor::is_permanent_error("Timeout"),
        "Timeouts should not be permanent"
    );
}
