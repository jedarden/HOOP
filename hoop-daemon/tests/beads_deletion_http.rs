//! Integration test: mid-run .beads/ deletion shows error card + /readyz reports degraded
//!
//! CI commands:
//!   cargo test -p hoop-daemon --test beads_deletion_http
//!
//! This test verifies §6 Phase 2 success criterion:
//! "Killing one project's runtime (delete `.beads/`) shows an error card;
//! other projects unaffected. /readyz reports degraded (A-listed)."
//!
//! Test scenario:
//! 1. Spawn daemon with 3 projects (A, B, C)
//! 2. Delete project A's .beads/ directory during runtime
//! 3. Assert:
//!    - Project A's card shows error state within 30s
//!    - Projects B/C continue serving events normally
//!    - /readyz reports degraded (A-listed)
//! 4. Restore .beads/ and verify recovery
//!
//! Plan reference: §6 Phase 2 success, §3.9

mod integration_harness;

use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;

use integration_harness::{spawn_test_daemon, spawn_test_daemon_with_config};
use hoop_daemon::Config;
use hoop_schema::ReadinessResponse;

/// Create a temporary project directory with .beads subdirectory
fn setup_project_dir(_name: &str) -> anyhow::Result<(TempDir, PathBuf)> {
    let project_dir = tempfile::tempdir()?;
    let project_path = project_dir.path().to_path_buf();

    // Create .beads directory with issues.jsonl
    let beads_dir = project_path.join(".beads");
    fs::create_dir_all(&beads_dir)?;

    let issues_path = beads_dir.join("issues.jsonl");
    fs::write(&issues_path, "")?;

    Ok((project_dir, project_path))
}

/// Create a projects.yaml with multiple projects
fn create_projects_yaml(project_paths: &[(&str, PathBuf)]) -> String {
    let projects: Vec<String> = project_paths
        .iter()
        .map(|(name, path)| {
            format!(
                r#"  - name: {}
    path: "{}""#,
                name,
                path.to_string_lossy().escape_default()
            )
        })
        .collect();

    format!(
        r#"---
projects:
{}
"#,
        projects.join("\n")
    )
}

/// Project status from /api/projects endpoint
#[derive(Debug, serde::Deserialize)]
struct ProjectStatus {
    name: String,
    state: String,
    #[serde(default)]
    error: Option<String>,
}

#[tokio::test]
async fn test_beads_deletion_readyz_degraded() {
    // Create temporary directories for three projects
    let (project_a_dir, project_a_path) = setup_project_dir("project-a").unwrap();
    let (project_b_dir, project_b_path) = setup_project_dir("project-b").unwrap();
    let (project_c_dir, project_c_path) = setup_project_dir("project-c").unwrap();

    // Store project paths as strings for use in the closure
    let project_a_str = project_a_path.to_string_lossy().to_string();
    let project_b_str = project_b_path.to_string_lossy().to_string();
    let project_c_str = project_c_path.to_string_lossy().to_string();

    // Spawn daemon with custom projects configuration
    let (base_url, _daemon) = spawn_test_daemon_with_config(Some(move |config: &mut Config| {
        // Create projects.yaml referencing all three projects
        let projects_yaml = format!(
            r#"---
projects:
  - name: project-a
    path: "{}"
  - name: project-b
    path: "{}"
  - name: project-c
    path: "{}"
"#,
            project_a_str, project_b_str, project_c_str
        );

        // Write custom projects.yaml to the test's .hoop directory
        let hoop_dir = config.control_socket_path.parent().unwrap();
        fs::write(hoop_dir.join("projects.yaml"), projects_yaml)
            .expect("Failed to write projects.yaml");
    }))
    .await
    .expect("Failed to spawn daemon");

    let client = reqwest::Client::new();

    // Wait for daemon to be ready
    let readyz_url = format!("{}/readyz", base_url);

    // Initially, readyz should return 200 (all projects healthy)
    let start = std::time::Instant::now();
    let mut initially_healthy = false;

    while start.elapsed() < Duration::from_secs(10) {
        match client.get(&readyz_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                let body: ReadinessResponse = resp.json().await.unwrap();
                if body.status == "ok" {
                    initially_healthy = true;
                    break;
                }
            }
            _ => {}
        }
        sleep(Duration::from_millis(200)).await;
    }

    assert!(
        initially_healthy,
        "Daemon should become healthy initially within 10s"
    );

    // Delete project A's .beads directory
    let beads_a_path = project_a_path.join(".beads");
    fs::remove_dir_all(&beads_a_path).unwrap();

    // Wait up to 30 seconds for /readyz to report degraded
    let start = std::time::Instant::now();
    let mut project_a_degraded = false;
    let mut degraded_response: Option<ReadinessResponse> = None;

    while start.elapsed() < Duration::from_secs(30) {
        match client.get(&readyz_url).send().await {
            Ok(resp) => {
                if resp.status() == 503 {
                    let body: ReadinessResponse = resp.json().await.unwrap();
                    if body.status == "degraded" {
                        // Check if project-a is in the degraded list
                        if body.degraded.iter().any(|d| d.project == "project-a") {
                            project_a_degraded = true;
                            degraded_response = Some(body);
                            break;
                        }
                    }
                }
            }
            Err(_) => {}
        }
        sleep(Duration::from_millis(500)).await;
    }

    assert!(
        project_a_degraded,
        "/readyz should report project-a as degraded within 30s"
    );

    // Verify the degraded response specifically mentions project-a
    let degraded = degraded_response.unwrap();
    let project_a_status = degraded
        .degraded
        .iter()
        .find(|d| d.project == "project-a")
        .expect("project-a should be in degraded list");

    assert!(
        project_a_status.state != "Healthy",
        "project-a state should not be Healthy, got: {}",
        project_a_status.state
    );

    // Verify projects B and C are NOT in the degraded list
    assert!(
        degraded.degraded.iter().all(|d| d.project != "project-b"),
        "project-b should not be in degraded list"
    );
    assert!(
        degraded.degraded.iter().all(|d| d.project != "project-c"),
        "project-c should not be in degraded list"
    );

    // Verify /api/projects shows the same state
    let projects_url = format!("{}/api/projects", base_url);
    let resp = client.get(&projects_url).send().await.unwrap();
    assert_eq!(resp.status(), 200);

    let projects: Vec<ProjectStatus> = resp.json().await.unwrap();

    let project_a_api = projects.iter().find(|p| p.name == "project-a").unwrap();
    assert_ne!(
        project_a_api.state, "Healthy",
        "project-a should not be Healthy via API"
    );

    let project_b_api = projects.iter().find(|p| p.name == "project-b").unwrap();
    assert!(
        project_b_api.state == "Healthy" || project_b_api.state == "Starting",
        "project-b should be Healthy or Starting, got: {}",
        project_b_api.state
    );

    let project_c_api = projects.iter().find(|p| p.name == "project-c").unwrap();
    assert!(
        project_c_api.state == "Healthy" || project_c_api.state == "Starting",
        "project-c should be Healthy or Starting, got: {}",
        project_c_api.state
    );

    // Test recovery: restore .beads directory
    fs::create_dir_all(&beads_a_path).unwrap();
    let issues_path = beads_a_path.join("issues.jsonl");
    fs::write(&issues_path, "").unwrap();

    // The supervisor should automatically detect the restored directory
    // and restart the runtime (via exponential backoff retry)

    // Wait for recovery (project-a should become healthy again)
    let start = std::time::Instant::now();
    let mut recovered = false;

    while start.elapsed() < Duration::from_secs(30) {
        match client.get(&readyz_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                let body: ReadinessResponse = resp.json().await.unwrap();
                if body.status == "ok" {
                    recovered = true;
                    break;
                }
            }
            _ => {}
        }
        sleep(Duration::from_millis(500)).await;
    }

    assert!(
        recovered,
        "Daemon should recover to healthy state after .beads/ restoration"
    );

    // Verify all projects are healthy again
    let resp = client.get(&projects_url).send().await.unwrap();
    let projects: Vec<ProjectStatus> = resp.json().await.unwrap();

    for project in &projects {
        assert!(
            project.state == "Healthy" || project.state == "Starting",
            "Project {} should be Healthy or Starting after recovery, got: {}",
            project.name,
            project.state
        );
    }

    // Cleanup happens automatically when daemon is dropped
    drop(project_a_dir);
    drop(project_b_dir);
    drop(project_c_dir);
}

#[tokio::test]
async fn test_beads_deletion_sibling_events_continue() {
    // This test verifies that sibling projects continue serving events
    // while one project is degraded

    // Create temporary directories for three projects
    let (project_a_dir, project_a_path) = setup_project_dir("project-a").unwrap();
    let (project_b_dir, project_b_path) = setup_project_dir("project-b").unwrap();
    let (project_c_dir, project_c_path) = setup_project_dir("project-c").unwrap();

    // Store project paths as strings for use in the closure
    let project_a_str = project_a_path.to_string_lossy().to_string();
    let project_b_str = project_b_path.to_string_lossy().to_string();
    let project_c_str = project_c_path.to_string_lossy().to_string();

    // Spawn daemon
    let (base_url, _daemon) = spawn_test_daemon_with_config(Some(move |config: &mut Config| {
        // Create projects.yaml referencing all three projects
        let projects_yaml = format!(
            r#"---
projects:
  - name: project-a
    path: "{}"
  - name: project-b
    path: "{}"
  - name: project-c
    path: "{}"
"#,
            project_a_str, project_b_str, project_c_str
        );

        // Write custom projects.yaml to the test's .hoop directory
        let hoop_dir = config.control_socket_path.parent().unwrap();
        fs::write(hoop_dir.join("projects.yaml"), projects_yaml)
            .expect("Failed to write projects.yaml");
    }))
    .await
    .expect("Failed to spawn daemon");

    let client = reqwest::Client::new();

    // Wait for initial health
    let readyz_url = format!("{}/readyz", base_url);
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(10) {
        if let Ok(resp) = client.get(&readyz_url).send().await {
            if resp.status().is_success() {
                break;
            }
        }
        sleep(Duration::from_millis(200)).await;
    }

    // Record baseline metrics for sibling projects
    let metrics_url = format!("{}/api/metrics", base_url);
    let resp_before = client.get(&metrics_url).send().await.unwrap();
    let _metrics_before = resp_before.text().await.unwrap();

    // Delete project A's .beads directory
    let beads_a_path = project_a_path.join(".beads");
    fs::remove_dir_all(&beads_a_path).unwrap();

    // Wait for project A to be reported as degraded
    let start = std::time::Instant::now();
    let mut degraded = false;
    while start.elapsed() < Duration::from_secs(30) {
        if let Ok(resp) = client.get(&readyz_url).send().await {
            if resp.status() == 503 {
                let body: ReadinessResponse = resp.json().await.unwrap();
                if body.degraded.iter().any(|d| d.project == "project-a") {
                    degraded = true;
                    break;
                }
            }
        }
        sleep(Duration::from_millis(500)).await;
    }
    assert!(degraded, "project-a should be degraded");

    // Verify sibling projects are still responding
    let projects_url = format!("{}/api/projects", base_url);
    let resp = client.get(&projects_url).send().await.unwrap();
    assert_eq!(resp.status(), 200, "API should still be accessible");

    let projects: Vec<ProjectStatus> = resp.json().await.unwrap();

    // Get metrics during degradation
    let resp_during = client.get(&metrics_url).send().await.unwrap();
    let metrics_during = resp_during.text().await.unwrap();

    // Verify metrics are still being collected
    assert!(
        !metrics_during.is_empty(),
        "Metrics should still be collected during degradation"
    );

    // Sibling projects should be in healthy state
    let project_b = projects.iter().find(|p| p.name == "project-b").unwrap();
    let project_c = projects.iter().find(|p| p.name == "project-c").unwrap();

    assert!(
        project_b.state == "Healthy" || project_b.state == "Starting",
        "project-b should be operational, got: {}",
        project_b.state
    );
    assert!(
        project_c.state == "Healthy" || project_c.state == "Starting",
        "project-c should be operational, got: {}",
        project_c.state
    );

    // Verify we can still query beads (API is functional)
    let beads_url = format!("{}/api/beads", base_url);
    let resp = client.get(&beads_url).send().await.unwrap();
    assert_eq!(
        resp.status(),
        200,
        "Should be able to query beads during degradation"
    );

    // Cleanup
    drop(project_a_dir);
    drop(project_b_dir);
    drop(project_c_dir);
}

#[tokio::test]
async fn test_readyz_response_format() {
    // Verify /readyz response format is correct
    let (base_url, _daemon) = spawn_test_daemon()
        .await
        .expect("Failed to spawn daemon");

    let client = reqwest::Client::new();
    let readyz_url = format!("{}/readyz", base_url);

    let resp = client.get(&readyz_url).send().await.unwrap();
    assert!(resp.status().is_success(), "Should be healthy initially");

    let body: ReadinessResponse = resp.json().await.unwrap();
    assert_eq!(body.status, "ok");
    assert!(body.degraded.is_empty());
}
