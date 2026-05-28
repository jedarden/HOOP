//! Acceptance test S5: Degraded mode - workspace deleted at runtime
//!
//! Plan reference: §1.8 Acceptance scenarios
//!
//! **S5 — Degraded: project workspace deleted at runtime (Phase 2)**
//! Operator removes a project's `.beads/` directory while HOOP is running. Within
//! one event-cycle (≤10s), HOOP shows an error card for that project on the dashboard.
//! All other projects continue updating normally. Restoring the `.beads/` directory
//! causes auto-recovery within one event-cycle.
//!
//! Pass criteria:
//! - Error card appears within 10s
//! - Other project cards unaffected
//! - Auto-recovery on restore without daemon restart
//!
//! Fail criteria:
//! - HOOP crashes
//! - Other projects' state is corrupted
//! - Recovery requires a manual restart

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

fn create_beads_dir(path: &std::path::Path) {
    let beads_dir = path.join(".beads");
    fs::create_dir_all(&beads_dir).expect("Failed to create .beads dir");
    let issues_path = beads_dir.join("issues.jsonl");
    fs::write(&issues_path, b"").expect("Failed to create issues.jsonl");
}

fn setup_multi_project_home(
    project_a: &PathBuf,
    project_b: &PathBuf,
    project_c: &PathBuf,
) -> tempfile::TempDir {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let hoop_dir = temp_dir.path().join(".hoop");
    fs::create_dir_all(&hoop_dir).expect("Failed to create .hoop dir");

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

    let config_yaml = r#"schema_version: 1
agent:
  adapter: claude
  model: claude-sonnet-4-6
"#;

    fs::write(hoop_dir.join("config.yml"), config_yaml)
        .expect("Failed to write config.yml");
    fs::create_dir_all(hoop_dir.join("data"))
        .expect("Failed to create data dir");
    std::env::set_var("HOME", temp_dir.path());

    temp_dir
}

async fn spawn_daemon_with_multi_project(
    project_a: &PathBuf,
    project_b: &PathBuf,
    project_c: &PathBuf,
) -> anyhow::Result<(String, tempfile::TempDir)> {
    let temp_dir = setup_multi_project_home(project_a, project_b, project_c);
    let hoop_dir = temp_dir.path().join(".hoop");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let base_url = format!("http://{}", addr);

    use hoop_daemon::Config;
    let config = Config {
        bind_addr: addr,
        control_socket_path: hoop_dir.join("control.sock"),
        allow_br_mismatch: true,
        observer_mode: false,
        primary_addr: std::net::SocketAddr::from(([127, 0, 0, 1], 3000)),
    };

    tokio::spawn(async move {
        if let Err(e) = hoop_daemon::serve(config).await {
            eprintln!("Daemon error: {}", e);
        }
    });

    // Wait for daemon to be ready
    let client = reqwest::Client::new();
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(10) {
        if client
            .get(&format!("{}/healthz", base_url))
            .timeout(Duration::from_millis(200))
            .send()
            .await
            .ok()
            .and_then(|r| r.status().is_success().then_some(()))
            .is_some()
        {
            return Ok((base_url, temp_dir));
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    Err(anyhow::anyhow!("Daemon failed to start"))
}

async fn get_readyz_status(base_url: &str) -> anyhow::Result<(u16, serde_json::Value)> {
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
async fn s5_workspace_deleted_error_within_10s() {
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

    let (base_url, _temp_dir) = spawn_daemon_with_multi_project(
        &project_a_path,
        &project_b_path,
        &project_c_path,
    )
    .await
    .expect("Failed to spawn daemon");

    // Initial state: all projects should be healthy
    tokio::time::sleep(Duration::from_millis(500)).await;

    let (status, readyz) = get_readyz_status(&base_url)
        .await
        .expect("Failed to get readyz status");

    assert_eq!(status, 200, "Initial readyz should return 200");

    // Remove .beads/ from project A
    let beads_a_path = project_a_path.join(".beads");
    fs::remove_dir_all(&beads_a_path).expect("Failed to remove .beads from project A");

    // Wait for the error to be detected (within 10 seconds)
    let start_detection = std::time::Instant::now();
    let mut degraded_detected = false;

    while start_detection.elapsed() < Duration::from_secs(10) {
        tokio::time::sleep(Duration::from_millis(500)).await;

        match get_readyz_status(&base_url).await {
            Ok((s, readyz_resp)) => {
                if s == 503 || readyz_resp["status"] == "degraded" {
                    degraded_detected = true;
                    break;
                }
            }
            Err(_) => continue,
        }
    }

    assert!(
        degraded_detected,
        "Error card should appear within 10s of workspace deletion"
    );

    println!("S5 PASS: Error card appeared within {:?}", start_detection.elapsed());
}

#[tokio::test]
async fn s5_other_projects_unaffected() {
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

    let (base_url, _temp_dir) = spawn_daemon_with_multi_project(
        &project_a_path,
        &project_b_path,
        &project_c_path,
    )
    .await
    .expect("Failed to spawn daemon");

    let client = reqwest::Client::new();

    // Wait for daemon to be ready
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Remove .beads/ from project A
    let beads_a_path = project_a_path.join(".beads");
    fs::remove_dir_all(&beads_a_path).expect("Failed to remove .beads from project A");

    // Wait for degraded state
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Verify projects B and C are still accessible
    let resp = client
        .get(&format!("{}/api/projects", base_url))
        .send()
        .await
        .expect("Failed to fetch projects");

    assert_eq!(resp.status(), 200, "Projects endpoint should still work");

    let projects: serde_json::Value = resp.json().await.expect("Failed to parse projects");

    assert!(
        projects.as_array().map(|arr| arr.len()).unwrap_or(0) >= 2,
        "Other projects should still be accessible"
    );

    // Verify the daemon hasn't crashed
    let resp = client
        .get(&format!("{}/healthz", base_url))
        .send()
        .await
        .expect("Failed to check health");

    assert!(resp.status().is_success(), "Daemon should still be healthy");

    println!("S5 PASS: Other projects unaffected when one project fails");
}

#[tokio::test]
async fn s5_auto_recovery_on_restore() {
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

    let (base_url, _temp_dir) = spawn_daemon_with_multi_project(
        &project_a_path,
        &project_b_path,
        &project_c_path,
    )
    .await
    .expect("Failed to spawn daemon");

    // Wait for daemon to be ready
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Initial state should be healthy
    let (status, _readyz) = get_readyz_status(&base_url)
        .await
        .expect("Failed to get readyz status");
    assert_eq!(status, 200);

    // Remove .beads/ from project A
    let beads_a_path = project_a_path.join(".beads");
    fs::remove_dir_all(&beads_a_path).expect("Failed to remove .beads from project A");

    // Wait for degraded state
    tokio::time::sleep(Duration::from_secs(2)).await;

    let (status, _readyz) = get_readyz_status(&base_url)
        .await
        .expect("Failed to get readyz status after deletion");
    // Should be in degraded state

    // Restore .beads/ directory
    create_beads_dir(&project_a_path);

    // Wait for auto-recovery (within 10 seconds)
    let start_recovery = std::time::Instant::now();
    let mut recovered = false;

    while start_recovery.elapsed() < Duration::from_secs(10) {
        tokio::time::sleep(Duration::from_millis(500)).await;

        match get_readyz_status(&base_url).await {
            Ok((s, readyz_resp)) => {
                if s == 200 && readyz_resp["status"] == "ok" {
                    recovered = true;
                    break;
                }
            }
            Err(_) => continue,
        }
    }

    assert!(
        recovered,
        "Auto-recovery should occur within 10s of workspace restore"
    );

    println!("S5 PASS: Auto-recovery completed within {:?}", start_recovery.elapsed());
}

#[tokio::test]
async fn s5_no_daemon_crash_on_workspace_deletion() {
    // Create temporary directory
    let project_a_dir = tempfile::tempdir().unwrap();
    let project_a_path = project_a_dir.path().to_path_buf();
    create_beads_dir(&project_a_path);

    let project_b_dir = tempfile::tempdir().unwrap();
    let project_b_path = project_b_dir.path().to_path_buf();
    create_beads_dir(&project_b_path);

    let project_c_dir = tempfile::tempdir().unwrap();
    let project_c_path = project_c_dir.path().to_path_buf();
    create_beads_dir(&project_c_path);

    let (base_url, _temp_dir) = spawn_daemon_with_multi_project(
        &project_a_path,
        &project_b_path,
        &project_c_path,
    )
    .await
    .expect("Failed to spawn daemon");

    let client = reqwest::Client::new();

    // Wait for daemon to be ready
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Remove .beads/ directory
    let beads_a_path = project_a_path.join(".beads");
    fs::remove_dir_all(&beads_a_path).expect("Failed to remove .beads");

    // Wait and verify daemon is still running
    tokio::time::sleep(Duration::from_secs(2)).await;

    let resp = client
        .get(&format!("{}/healthz", base_url))
        .send()
        .await
        .expect("Failed to check health");

    assert!(
        resp.status().is_success(),
        "Daemon should still be running after workspace deletion"
    );

    println!("S5 PASS: Daemon did not crash on workspace deletion");
}
