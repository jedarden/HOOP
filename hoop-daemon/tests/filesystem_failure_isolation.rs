//! Per-project runtime test: rm -rf .beads/ shows error card + siblings unaffected
//!
//! CI commands:
//!   cargo test -p hoop-daemon --test filesystem_failure_isolation
//!
//! This test verifies:
//! 1. Removing .beads/ for project A mid-run causes error state within 30s
//! 2. Projects B/C continue serving events normally
//! 3. /readyz reports degraded (A-listed)
//! 4. Restoring .beads/ recovers project A on next reload
//!
//! Plan reference: §6 Phase 2 success, §3.9

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use futures_util::stream::StreamExt;
use hoop_schema::ReadinessResponse;
use serde_json::Value as JsonValue;

mod integration_harness;
use integration_harness::setup_test_hoop_home;

/// Helper to create a minimal .beads directory
fn create_beads_dir(path: &std::path::Path) {
    let beads_dir = path.join(".beads");
    fs::create_dir_all(&beads_dir).expect("Failed to create .beads dir");
    let issues_path = beads_dir.join("issues.jsonl");
    fs::write(&issues_path, b"").expect("Failed to create issues.jsonl");
}

/// Set up a temporary HOOP home with multiple projects
fn setup_multi_project_home(
    project_a: &PathBuf,
    project_b: &PathBuf,
    project_c: &PathBuf,
) -> tempfile::TempDir {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let hoop_dir = temp_dir.path().join(".hoop");
    fs::create_dir_all(&hoop_dir).expect("Failed to create .hoop dir");

    // Create projects.yaml with all three projects
    let projects_yaml = format!(
        r#"projects:
  - name: project-a
    path: {}
    workspaces:
      - path: {}
        role: primary
  - name: project-b
    path: {}
    workspaces:
      - path: {}
        role: primary
  - name: project-c
    path: {}
    workspaces:
      - path: {}
        role: primary
"#,
        project_a.display(),
        project_a.display(),
        project_b.display(),
        project_b.display(),
        project_c.display(),
        project_c.display()
    );

    fs::write(hoop_dir.join("projects.yaml"), projects_yaml)
        .expect("Failed to write projects.yaml");

    // Create minimal config.yml
    let config_yaml = r#"schema_version: 1
agent:
  adapter: claude
  model: claude-sonnet-4-6
"#;

    fs::write(hoop_dir.join("config.yml"), config_yaml).expect("Failed to write config.yml");

    // Create data directory for fleet.db
    fs::create_dir_all(hoop_dir.join("data")).expect("Failed to create data dir");

    // Set environment variable to override home directory
    std::env::set_var("HOME", temp_dir.path());

    temp_dir
}

/// Get project status from /readyz endpoint
async fn get_readyz_status(base_url: &str) -> anyhow::Result<(u16, ReadinessResponse)> {
    let client = reqwest::Client::new();
    let resp = client
        .get(&format!("{}/readyz", base_url))
        .send()
        .await?;

    let status = resp.status().as_u16();
    let body = resp.json().await?;
    Ok((status, body))
}

#[tokio::test]
async fn test_beads_removal_shows_error_state_siblings_unaffected() {
    // Create temporary directories for three projects
    let project_a_dir = tempfile::tempdir().unwrap();
    let project_a_path = project_a_dir.path().to_path_buf();
    create_beads_dir(&project_a_path);

    let project_b_dir = tempfile::tempdir().unwrap();
    let project_b_path = project_b_dir.path().to_path_buf();
    create_beads_dir(&project_b_path);

    let project_c_dir = tempfile::tempdir().unwrap();
    let project_c_path = project_c_dir.path().to_path_buf();
    create_beads_dir(&project_c_path);

    // Set up a custom test HOOP home with all three projects
    let temp_dir = setup_multi_project_home(&project_a_path, &project_b_path, &project_c_path);

    // Bind to port 0 to get a random available port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to random port");
    let addr = listener.local_addr().expect("Failed to get local address");
    let base_url = format!("http://{}", addr);

    // Create the config struct
    let config = hoop_daemon::Config {
        bind_addr: addr,
        control_socket_path: temp_dir.path().join(".hoop").join("control.sock"),
        allow_br_mismatch: true,
        observer_mode: false,
        primary_addr: std::net::SocketAddr::from(([127, 0, 0, 1], 3000)),
    };

    let shutdown_notify = std::sync::Arc::new(tokio::sync::Notify::new());
    let notify_clone = shutdown_notify.clone();

    // Spawn the daemon in a background task
    tokio::spawn(async move {
        let result = hoop_daemon::serve(config).await;
        notify_clone.notify_one();
        if let Err(e) = result {
            eprintln!("Daemon error: {}", e);
        }
    });

    // Wait for the server to become ready
    let client = reqwest::Client::new();
    let start = std::time::Instant::now();

    while start.elapsed() < Duration::from_secs(10) {
        if let Ok(resp) = client
            .get(&format!("{}/healthz", base_url))
            .timeout(Duration::from_millis(200))
            .send()
            .await
        {
            if resp.status().is_success() {
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Initial state: all projects should be healthy
    tokio::time::sleep(Duration::from_millis(500)).await;

    let (status, readyz) = get_readyz_status(&base_url)
        .await
        .expect("Failed to get readyz status");

    assert_eq!(status, 200, "Initial readyz should return 200");
    assert_eq!(readyz.status, "ok", "Initial readyz status should be ok");
    assert!(
        readyz.degraded.is_empty(),
        "No projects should be degraded initially"
    );

    // Remove .beads/ from project A to simulate filesystem failure
    let beads_a_path = project_a_path.join(".beads");
    fs::remove_dir_all(&beads_a_path).expect("Failed to remove .beads from project A");

    // Trigger a config reload by touching projects.yaml
    let projects_path = temp_dir.path().join(".hoop").join("projects.yaml");
    let metadata = fs::metadata(&projects_path).expect("Failed to get projects.yaml metadata");
    let modified = metadata.modified().expect("Failed to get modified time");

    // Wait for the error to be detected (within 30 seconds per acceptance criteria)
    let start_detection = std::time::Instant::now();
    let mut degraded_detected = false;

    while start_detection.elapsed() < Duration::from_secs(30) {
        tokio::time::sleep(Duration::from_millis(500)).await;

        match get_readyz_status(&base_url).await {
            Ok((s, readyz_resp)) => {
                if s == 503 && readyz_resp.status == "degraded" {
                    // Check if project-a is in the degraded list
                    if readyz_resp.degraded.iter().any(|p| p.project == "project-a") {
                        degraded_detected = true;

                        // Verify projects B and C are NOT in the degraded list
                        assert!(
                            !readyz_resp.degraded.iter().any(|p| p.project == "project-b"),
                            "project-b should not be degraded"
                        );
                        assert!(
                            !readyz_resp.degraded.iter().any(|p| p.project == "project-c"),
                            "project-c should not be degraded"
                        );

                        // Verify project-a has an error message
                        let project_a_degraded = readyz_resp
                            .degraded
                            .iter()
                            .find(|p| p.project == "project-a")
                            .expect("project-a should be in degraded list");

                        assert!(
                            project_a_degraded.error.is_some(),
                            "project-a should have an error message"
                        );

                        // Verify the error mentions .beads
                        let error_msg = project_a_degraded.error.as_ref().unwrap();
                        assert!(
                            error_msg.contains(".beads") || error_msg.contains("beads"),
                            "Error message should mention .beads: {}",
                            error_msg
                        );

                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error getting readyz status: {}", e);
            }
        }
    }

    assert!(
        degraded_detected,
        "project-a should be detected as degraded within 30 seconds"
    );

    // Signal shutdown
    shutdown_notify.notify_one();
}

#[tokio::test]
async fn test_beads_removal_degraded_readyz_recovery() {
    // Create temporary directories for three projects
    let project_a_dir = tempfile::tempdir().unwrap();
    let project_a_path = project_a_dir.path().to_path_buf();
    create_beads_dir(&project_a_path);

    let project_b_dir = tempfile::tempdir().unwrap();
    let project_b_path = project_b_dir.path().to_path_buf();
    create_beads_dir(&project_b_path);

    let project_c_dir = tempfile::tempdir().unwrap();
    let project_c_path = project_c_dir.path().to_path_buf();
    create_beads_dir(&project_c_path);

    // Set up a custom test HOOP home with all three projects
    let temp_dir = setup_multi_project_home(&project_a_path, &project_b_path, &project_c_path);

    // Bind to port 0 to get a random available port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to random port");
    let addr = listener.local_addr().expect("Failed to get local address");
    let base_url = format!("http://{}", addr);

    // Create the config struct
    let config = hoop_daemon::Config {
        bind_addr: addr,
        control_socket_path: temp_dir.path().join(".hoop").join("control.sock"),
        allow_br_mismatch: true,
        observer_mode: false,
        primary_addr: std::net::SocketAddr::from(([127, 0, 0, 1], 3000)),
    };

    let shutdown_notify = std::sync::Arc::new(tokio::sync::Notify::new());
    let notify_clone = shutdown_notify.clone();

    // Spawn the daemon in a background task
    tokio::spawn(async move {
        let result = hoop_daemon::serve(config).await;
        notify_clone.notify_one();
        if let Err(e) = result {
            eprintln!("Daemon error: {}", e);
        }
    });

    // Wait for the server to become ready
    let client = reqwest::Client::new();
    let start = std::time::Instant::now();

    while start.elapsed() < Duration::from_secs(10) {
        if let Ok(resp) = client
            .get(&format!("{}/healthz", base_url))
            .timeout(Duration::from_millis(200))
            .send()
            .await
        {
            if resp.status().is_success() {
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Initial state: all projects should be healthy
    tokio::time::sleep(Duration::from_millis(500)).await;

    let (status, readyz) = get_readyz_status(&base_url)
        .await
        .expect("Failed to get readyz status");

    assert_eq!(status, 200, "Initial readyz should return 200");
    assert_eq!(readyz.status, "ok", "Initial readyz status should be ok");
    assert!(
        readyz.degraded.is_empty(),
        "No projects should be degraded initially"
    );

    // Remove .beads/ from project A to simulate filesystem failure
    let beads_a_path = project_a_path.join(".beads");
    fs::remove_dir_all(&beads_a_path).expect("Failed to remove .beads from project A");

    // Trigger a config reload by touching projects.yaml
    let projects_path = temp_dir.path().join(".hoop").join("projects.yaml");

    // Wait for the error to be detected (within 30 seconds per acceptance criteria)
    let start_detection = std::time::Instant::now();
    let mut degraded_detected = false;

    while start_detection.elapsed() < Duration::from_secs(30) {
        tokio::time::sleep(Duration::from_millis(500)).await;

        match get_readyz_status(&base_url).await {
            Ok((s, readyz_resp)) => {
                if s == 503 && readyz_resp.status == "degraded" {
                    if readyz_resp.degraded.iter().any(|p| p.project == "project-a") {
                        degraded_detected = true;
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error getting readyz status: {}", e);
            }
        }
    }

    assert!(
        degraded_detected,
        "project-a should be detected as degraded within 30 seconds"
    );

    // Restore .beads/ for project A
    create_beads_dir(&project_a_path);

    // Trigger a reload by touching the projects.yaml file
    // This simulates the operator restoring the directory and triggering a reload
    let projects_content = fs::read_to_string(&projects_path).expect("Failed to read projects.yaml");
    fs::write(&projects_path, projects_content).expect("Failed to write projects.yaml");

    // Wait for recovery (project-a should become healthy again)
    let start_recovery = std::time::Instant::now();
    let mut recovered = false;

    while start_recovery.elapsed() < Duration::from_secs(10) {
        tokio::time::sleep(Duration::from_millis(500)).await;

        match get_readyz_status(&base_url).await {
            Ok((s, readyz_resp)) => {
                if s == 200 && readyz_resp.status == "ok" {
                    // All projects should be healthy now
                    if readyz_resp.degraded.is_empty() {
                        recovered = true;
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error getting readyz status during recovery: {}", e);
            }
        }
    }

    assert!(
        recovered,
        "project-a should recover after .beads/ is restored"
    );

    // Signal shutdown
    shutdown_notify.notify_one();
}

#[tokio::test]
async fn test_sibling_projects_continue_during_degradation() {
    // This test specifically validates that sibling projects B and C
    // continue serving events normally while project A is degraded

    // Create temporary directories for three projects
    let project_a_dir = tempfile::tempdir().unwrap();
    let project_a_path = project_a_dir.path().to_path_buf();
    create_beads_dir(&project_a_path);

    let project_b_dir = tempfile::tempdir().unwrap();
    let project_b_path = project_b_dir.path().to_path_buf();
    create_beads_dir(&project_b_path);

    let project_c_dir = tempfile::tempdir().unwrap();
    let project_c_path = project_c_dir.path().to_path_buf();
    create_beads_dir(&project_c_path);

    // Set up a custom test HOOP home with all three projects
    let temp_dir = setup_multi_project_home(&project_a_path, &project_b_path, &project_c_path);

    // Bind to port 0 to get a random available port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to random port");
    let addr = listener.local_addr().expect("Failed to get local address");
    let base_url = format!("http://{}", addr);

    // Create the config struct
    let config = hoop_daemon::Config {
        bind_addr: addr,
        control_socket_path: temp_dir.path().join(".hoop").join("control.sock"),
        allow_br_mismatch: true,
        observer_mode: false,
        primary_addr: std::net::SocketAddr::from(([127, 0, 0, 1], 3000)),
    };

    let shutdown_notify = std::sync::Arc::new(tokio::sync::Notify::new());
    let notify_clone = shutdown_notify.clone();

    // Spawn the daemon in a background task
    tokio::spawn(async move {
        let result = hoop_daemon::serve(config).await;
        notify_clone.notify_one();
        if let Err(e) = result {
            eprintln!("Daemon error: {}", e);
        }
    });

    // Wait for the server to become ready
    let client = reqwest::Client::new();
    let start = std::time::Instant::now();

    while start.elapsed() < Duration::from_secs(10) {
        if let Ok(resp) = client
            .get(&format!("{}/healthz", base_url))
            .timeout(Duration::from_millis(200))
            .send()
            .await
        {
            if resp.status().is_success() {
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Initial state: all projects should be healthy
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Verify initial readyz is healthy
    let (status, readyz) = get_readyz_status(&base_url)
        .await
        .expect("Failed to get readyz status");
    assert_eq!(status, 200);
    assert_eq!(readyz.status, "ok");

    // Connect WebSocket to monitor events from sibling projects
    let ws_url = base_url.replace("http://", "ws://");
    let ws_url = format!("{}/ws", ws_url);
    let (ws_stream, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .expect("Failed to connect to WebSocket");
    let (_, mut ws_receiver) = ws_stream.split();

    // Remove .beads/ from project A
    let beads_a_path = project_a_path.join(".beads");
    fs::remove_dir_all(&beads_a_path).expect("Failed to remove .beads from project A");

    // Trigger a config reload by touching projects.yaml
    let projects_path = temp_dir.path().join(".hoop").join("projects.yaml");

    // Wait for project A to be marked degraded
    let start_wait = std::time::Instant::now();
    let mut project_a_degraded = false;

    while start_wait.elapsed() < Duration::from_secs(30) {
        tokio::time::sleep(Duration::from_millis(500)).await;

        if let Ok((s, readyz_resp)) = get_readyz_status(&base_url).await {
            if s == 503
                && readyz_resp.status == "degraded"
                && readyz_resp.degraded.iter().any(|p| p.project == "project-a")
            {
                project_a_degraded = true;
                break;
            }
        }
    }

    assert!(project_a_degraded, "project-a should be degraded");

    // Now monitor WebSocket for events from projects B and C
    // They should continue sending status updates despite A being degraded
    let mut received_b_status = false;
    let mut received_c_status = false;

    let start_monitor = std::time::Instant::now();

    while start_monitor.elapsed() < Duration::from_secs(5) {
        match tokio::time::timeout(Duration::from_millis(500), ws_receiver.next()).await {
            Ok(Some(Ok(msg))) => {
                if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
                    if let Ok(json) = serde_json::from_str::<JsonValue>(&text) {
                        if let Some(status) = json.get("status").and_then(|s| s.as_str()) {
                            if status == "project_runtime_status" {
                                if let Some(data) = json.get("data") {
                                    if let Some(name) = data.get("project_name").and_then(|n| n.as_str()) {
                                        if name == "project-b" {
                                            if let Some(state) = data.get("runtime_state").and_then(|s| s.as_str()) {
                                                // project-b should be healthy
                                                assert_eq!(
                                                    state, "healthy",
                                                    "project-b should remain healthy"
                                                );
                                                received_b_status = true;
                                            }
                                        } else if name == "project-c" {
                                            if let Some(state) = data.get("runtime_state").and_then(|s| s.as_str()) {
                                                assert_eq!(
                                                    state, "healthy",
                                                    "project-c should remain healthy"
                                                );
                                                received_c_status = true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Ok(None) => break,
            Err(_) => continue,
        }

        if received_b_status && received_c_status {
            break;
        }
    }

    assert!(
        received_b_status,
        "project-b should continue sending status events"
    );
    assert!(
        received_c_status,
        "project-c should continue sending status events"
    );

    // Signal shutdown
    shutdown_notify.notify_one();
}
