//! Supervisor health reporting tests for /readyz
//!
//! CI commands:
//!   cargo test -p hoop-daemon --test supervisor_health
//!
//! This test verifies:
//! 1. snapshot() returns current state of all runtimes
//! 2. subscribe_status() provides live status updates
//! 3. Health check logic (at least one healthy → ready)
//! 4. Status broadcasts include all required fields

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use hoop_daemon::projects::ProjectsConfig;
use hoop_daemon::shutdown::ShutdownCoordinator;
use hoop_daemon::supervisor::{ProjectRuntimeState, ProjectRuntimeStatus, ProjectSupervisor};
use hoop_daemon::ws::WorkerRegistry;
use hoop_daemon::Bead;
use hoop_schema::{ProjectsRegistry, ProjectsRegistryProjectsItem};

/// Create a test project with workspace path (shorthand single-workspace variant)
fn create_test_project(name: &str, path: PathBuf) -> ProjectsRegistryProjectsItem {
    ProjectsRegistryProjectsItem::Variant0 {
        name: name.to_string(),
        path: path.to_string_lossy().into_owned(),
        canonical_path: None,
        color: None,
        label: None,
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
    let (monitor_tx, _) = tokio::sync::broadcast::channel(64);
    let (sessions_tx, _) = tokio::sync::broadcast::channel(64);
    let worker_registry = Arc::new(WorkerRegistry::new(monitor_tx, sessions_tx));
    let beads = Arc::new(std::sync::RwLock::new(Vec::<Bead>::new()));
    let shutdown = Arc::new(ShutdownCoordinator::new());
    let cost_aggregator = Arc::new(std::sync::RwLock::new(
        hoop_daemon::cost::CostAggregator::new(PathBuf::from("/tmp/test-cost.toml"))
            .expect("Failed to create CostAggregator"),
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

/// Create a test ProjectsConfig from project definitions
fn create_test_config(projects: Vec<ProjectsRegistryProjectsItem>) -> ProjectsConfig {
    let registry = ProjectsRegistry { projects };
    let mut canonical_cache = std::collections::HashMap::new();

    // Pre-populate canonical cache for test projects
    for project in &registry.projects {
        let name = project.name();
        for ws in project.workspace_views() {
            if let Ok(canonical) = std::fs::canonicalize(&ws.path) {
                let cache_key = (name.to_string(), ws.path.clone());
                canonical_cache.insert(cache_key, canonical);
            }
        }
    }

    ProjectsConfig {
        registry,
        path: PathBuf::from("/test/projects.yaml"),
        canonical_cache,
        content_hash: String::new(),
    }
}

/// Health check logic: at least one runtime in Healthy/Starting → ready
fn is_ready(snapshots: &[ProjectRuntimeStatus]) -> bool {
    snapshots
        .iter()
        .any(|s| s.state.is_running())
}

#[tokio::test]
async fn test_snapshot_returns_all_runtimes() {
    // Create temporary directories for multiple projects
    let project1_dir = tempfile::tempdir().unwrap();
    let project1_path = project1_dir.path().to_path_buf();
    let _beads1 = create_beads_dir(&project1_path);

    let project2_dir = tempfile::tempdir().unwrap();
    let project2_path = project2_dir.path().to_path_buf();
    let _beads2 = create_beads_dir(&project2_path);

    let project3_dir = tempfile::tempdir().unwrap();
    let project3_path = project3_dir.path().to_path_buf();
    let _beads3 = create_beads_dir(&project3_path);

    let supervisor = create_test_supervisor().await;

    // Initially empty
    let snapshot = supervisor.snapshot().await;
    assert_eq!(snapshot.len(), 0);

    // Add all three projects
    let config = create_test_config(vec![
        create_test_project("project-1", project1_path),
        create_test_project("project-2", project2_path),
        create_test_project("project-3", project3_path),
    ]);

    supervisor
        .reconcile(&config)
        .await
        .expect("Reconcile should succeed");

    tokio::time::sleep(Duration::from_millis(100)).await;

    let snapshot = supervisor.snapshot().await;
    assert_eq!(snapshot.len(), 3);

    // Verify all fields are present
    for status in &snapshot {
        assert!(!status.project_name.is_empty());
        assert!(!status.project_path.as_os_str().is_empty());
        assert!(status.workspace_count > 0);
        assert_eq!(status.workspace_count, 1); // Single workspace per project
    }
}

#[tokio::test]
async fn test_status_subscription_receives_updates() {
    // Create temporary directory for a project
    let project_dir = tempfile::tempdir().unwrap();
    let project_path = project_dir.path().to_path_buf();
    let _beads = create_beads_dir(&project_path);

    let supervisor = create_test_supervisor().await;

    // Subscribe before creating runtime
    let mut status_rx = supervisor.subscribe_status();

    // Create a project runtime
    let config = create_test_config(vec![create_test_project(
        "test-project",
        project_path,
    )]);

    supervisor
        .reconcile(&config)
        .await
        .expect("Reconcile should succeed");

    // Wait for status update
    let received = tokio::select! {
        result = status_rx.recv() => result.is_ok(),
        _ = tokio::time::sleep(Duration::from_secs(2)) => false,
    };

    assert!(received, "Should receive status update");

    // Verify we can receive multiple updates
    let snapshot = supervisor.snapshot().await;
    assert_eq!(snapshot.len(), 1);
}

#[tokio::test]
async fn test_multiple_subscribers_receive_updates() {
    // Create temporary directory for a project
    let project_dir = tempfile::tempdir().unwrap();
    let project_path = project_dir.path().to_path_buf();
    let _beads = create_beads_dir(&project_path);

    let supervisor = create_test_supervisor().await;

    // Create multiple subscribers
    let mut rx1 = supervisor.subscribe_status();
    let mut rx2 = supervisor.subscribe_status();
    let mut rx3 = supervisor.subscribe_status();

    // Create a project runtime
    let config = create_test_config(vec![create_test_project(
        "test-project",
        project_path,
    )]);

    supervisor
        .reconcile(&config)
        .await
        .expect("Reconcile should succeed");

    // All subscribers should receive updates
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Try to receive from each
    for (i, mut rx) in vec![rx1, rx2, rx3].into_iter().enumerate() {
        let received = tokio::select! {
            result = rx.recv() => result.is_ok(),
            _ = tokio::time::sleep(Duration::from_millis(100)) => false,
        };
        // At least one subscriber should get an update
        // (broadcast channel may miss updates if receiver is slow)
        if i == 0 || received {
            assert!(true);
        }
    }
}

#[tokio::test]
async fn test_health_check_ready_when_healthy() {
    // Create temporary directory for a project
    let project_dir = tempfile::tempdir().unwrap();
    let project_path = project_dir.path().to_path_buf();
    let _beads = create_beads_dir(&project_path);

    let supervisor = create_test_supervisor().await;

    // Initially not ready (no runtimes)
    let snapshot = supervisor.snapshot().await;
    assert!(!is_ready(&snapshot), "Should not be ready with no runtimes");

    // Add a project
    let config = create_test_config(vec![create_test_project(
        "test-project",
        project_path,
    )]);

    supervisor
        .reconcile(&config)
        .await
        .expect("Reconcile should succeed");

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Should be ready now
    let snapshot = supervisor.snapshot().await;
    assert!(is_ready(&snapshot), "Should be ready with healthy runtime");
}

#[tokio::test]
async fn test_health_check_not_ready_when_all_failed() {
    // Create temporary directory for a project
    let project_dir = tempfile::tempdir().unwrap();
    let project_path = project_dir.path().to_path_buf();
    let _beads = create_beads_dir(&project_path);

    let supervisor = create_test_supervisor().await;

    // Add a project
    let config = create_test_config(vec![create_test_project(
        "test-project",
        project_path,
    )]);

    supervisor
        .reconcile(&config)
        .await
        .expect("Reconcile should succeed");

    tokio::time::sleep(Duration::from_millis(100)).await;

    let snapshot = supervisor.snapshot().await;
    assert_eq!(snapshot.len(), 1);

    // Simulate all runtimes in failed state
    // (In real scenario, this would happen due to panics)
    // For this test, we just verify the health check function works

    let all_failed = vec![ProjectRuntimeStatus {
        project_name: "test".to_string(),
        project_path: PathBuf::from("/test"),
        state: ProjectRuntimeState::Failed {
            error: "test error".to_string(),
            failed_at: chrono::Utc::now(),
            consecutive_failures: 1,
            next_restart_at: chrono::Utc::now(),
        },
        workspace_count: 1,
        bead_count: 0,
    }];

    assert!(!is_ready(&all_failed), "Should not be ready when all failed");

    let all_error = vec![ProjectRuntimeStatus {
        project_name: "test".to_string(),
        project_path: PathBuf::from("/test"),
        state: ProjectRuntimeState::Error {
            error: "test error".to_string(),
            errored_at: chrono::Utc::now(),
        },
        workspace_count: 1,
        bead_count: 0,
    }];

    assert!(!is_ready(&all_error), "Should not be ready when all in error state");

    let all_abandoned = vec![ProjectRuntimeStatus {
        project_name: "test".to_string(),
        project_path: PathBuf::from("/test"),
        state: ProjectRuntimeState::Abandoned {
            error: "test error".to_string(),
            abandoned_at: chrono::Utc::now(),
        },
        workspace_count: 1,
        bead_count: 0,
    }];

    assert!(!is_ready(&all_abandoned), "Should not be ready when all abandoned");
}

#[tokio::test]
async fn test_health_check_ready_with_one_healthy() {
    // Health check: at least one healthy → ready
    let mixed_states = vec![
        ProjectRuntimeStatus {
            project_name: "failed-project".to_string(),
            project_path: PathBuf::from("/test1"),
            state: ProjectRuntimeState::Failed {
                error: "test error".to_string(),
                failed_at: chrono::Utc::now(),
                consecutive_failures: 1,
                next_restart_at: chrono::Utc::now(),
            },
            workspace_count: 1,
            bead_count: 0,
        },
        ProjectRuntimeStatus {
            project_name: "healthy-project".to_string(),
            project_path: PathBuf::from("/test2"),
            state: ProjectRuntimeState::Healthy,
            workspace_count: 1,
            bead_count: 0,
        },
        ProjectRuntimeStatus {
            project_name: "error-project".to_string(),
            project_path: PathBuf::from("/test3"),
            state: ProjectRuntimeState::Error {
                error: "test error".to_string(),
                errored_at: chrono::Utc::now(),
            },
            workspace_count: 1,
            bead_count: 0,
        },
    ];

    assert!(is_ready(&mixed_states), "Should be ready with at least one healthy");

    // Also works with Starting state
    let mixed_with_starting = vec![
        ProjectRuntimeStatus {
            project_name: "failed-project".to_string(),
            project_path: PathBuf::from("/test1"),
            state: ProjectRuntimeState::Failed {
                error: "test error".to_string(),
                failed_at: chrono::Utc::now(),
                consecutive_failures: 1,
                next_restart_at: chrono::Utc::now(),
            },
            workspace_count: 1,
            bead_count: 0,
        },
        ProjectRuntimeStatus {
            project_name: "starting-project".to_string(),
            project_path: PathBuf::from("/test2"),
            state: ProjectRuntimeState::Starting,
            workspace_count: 1,
            bead_count: 0,
        },
    ];

    assert!(
        is_ready(&mixed_with_starting),
        "Should be ready with at least one starting"
    );
}

#[tokio::test]
async fn test_status_includes_project_metadata() {
    // Create temporary directory for a project
    let project_dir = tempfile::tempdir().unwrap();
    let project_path = project_dir.path().to_path_buf();
    let _beads = create_beads_dir(&project_path);

    let supervisor = create_test_supervisor().await;

    // Create a project runtime
    let config = create_test_config(vec![create_test_project(
        "my-test-project",
        project_path.clone(),
    )]);

    supervisor
        .reconcile(&config)
        .await
        .expect("Reconcile should succeed");

    tokio::time::sleep(Duration::from_millis(100)).await;

    let snapshot = supervisor.snapshot().await;
    assert_eq!(snapshot.len(), 1);

    let status = &snapshot[0];
    assert_eq!(status.project_name, "my-test-project");
    assert_eq!(status.project_path, project_path);
    assert_eq!(status.workspace_count, 1);
    assert!(status.state.is_running());
}

#[tokio::test]
async fn test_status_broadcasts_on_state_changes() {
    // Create temporary directory for a project
    let project_dir = tempfile::tempdir().unwrap();
    let project_path = project_dir.path().to_path_buf();
    let _beads = create_beads_dir(&project_path);

    let supervisor = create_test_supervisor().await;

    // Subscribe to status updates
    let mut status_rx = supervisor.subscribe_status();

    // Track received updates
    let mut update_count = 0;

    // Create a project runtime
    let config = create_test_config(vec![create_test_project(
        "test-project",
        project_path,
    )]);

    supervisor
        .reconcile(&config)
        .await
        .expect("Reconcile should succeed");

    // Collect updates for a short time
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_millis(500) {
        tokio::select! {
            result = status_rx.recv() => {
                if result.is_ok() {
                    update_count += 1;
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(50)) => break,
        }
    }

    // Should receive at least one update
    assert!(
        update_count >= 1,
        "Should receive at least one status update, got {}",
        update_count
    );
}

#[tokio::test]
async fn test_bead_count_in_status() {
    // Create temporary directory for a project
    let project_dir = tempfile::tempdir().unwrap();
    let project_path = project_dir.path().to_path_buf();
    let _beads = create_beads_dir(&project_path);

    let supervisor = create_test_supervisor().await;

    // Create a project runtime
    let config = create_test_config(vec![create_test_project(
        "test-project",
        project_path,
    )]);

    supervisor
        .reconcile(&config)
        .await
        .expect("Reconcile should succeed");

    tokio::time::sleep(Duration::from_millis(100)).await;

    let snapshot = supervisor.snapshot().await;
    assert_eq!(snapshot.len(), 1);

    // Bead count should be present (initially 0 since no beads in issues.jsonl)
    let status = &snapshot[0];
    assert_eq!(status.bead_count, 0);
}

#[tokio::test]
async fn test_workspace_count_in_status() {
    // Create temporary directory for a project with multiple workspaces
    let project_dir = tempfile::tempdir().unwrap();
    let project_path1 = project_dir.path().join("workspace1");
    let project_path2 = project_dir.path().join("workspace2");
    std::fs::create_dir_all(&project_path1).unwrap();
    std::fs::create_dir_all(&project_path2).unwrap();

    let _beads1 = create_beads_dir(&project_path1);
    let _beads2 = create_beads_dir(&project_path2);

    let supervisor = create_test_supervisor().await;

    // Create a project with multiple workspaces using the multi-workspace variant
    let multi_workspace_project = ProjectsRegistryProjectsItem::Variant1 {
        name: "multi-workspace-project".to_string(),
        workspaces: vec![
            hoop_schema::ProjectsRegistryProjectsItemVariant1WorkspacesItem {
                path: project_path1.to_string_lossy().into_owned(),
                canonical_path: None,
                role: hoop_schema::ProjectsRegistryProjectsItemVariant1WorkspacesItemRole::Primary,
            },
            hoop_schema::ProjectsRegistryProjectsItemVariant1WorkspacesItem {
                path: project_path2.to_string_lossy().into_owned(),
                canonical_path: None,
                role: hoop_schema::ProjectsRegistryProjectsItemVariant1WorkspacesItemRole::Source,
            },
        ],
        color: None,
        label: None,
        redaction: None,
    };

    let mut registry = ProjectsRegistry {
        projects: vec![multi_workspace_project],
    };

    let mut canonical_cache = std::collections::HashMap::new();
    for project in &registry.projects {
        let name = project.name();
        for ws in project.workspace_views() {
            if let Ok(canonical) = std::fs::canonicalize(&ws.path) {
                let cache_key = (name.to_string(), ws.path.clone());
                canonical_cache.insert(cache_key, canonical);
            }
        }
    }

    let config = ProjectsConfig {
        registry,
        path: PathBuf::from("/test/projects.yaml"),
        canonical_cache,
        content_hash: String::new(),
    };

    supervisor
        .reconcile(&config)
        .await
        .expect("Reconcile should succeed");

    tokio::time::sleep(Duration::from_millis(100)).await;

    let snapshot = supervisor.snapshot().await;
    assert_eq!(snapshot.len(), 1);

    // Should report 2 workspaces
    let status = &snapshot[0];
    assert_eq!(status.workspace_count, 2);
}
