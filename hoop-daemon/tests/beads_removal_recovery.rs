//! Integration test: per-project runtime test — rm -rf .beads shows error card + siblings unaffected
//!
//! CI commands:
//!   cargo test -p hoop-daemon --test beads_removal_recovery
//!
//! This test verifies:
//! 1. Project A's card shows error state within 30s after .beads/ removal
//! 2. Projects B/C continue serving events normally during degradation
//! 3. /readyz reports degraded (with project A listed)
//! 4. Restoring .beads/ recovers project A on next reload
//!
//! Plan reference: §6 Phase 2 success, §3.9

mod integration_harness;

use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::timeout;

use hoop_schema::{DegradedProject, ReadinessResponse};
use integration_harness::spawn_test_daemon_with_config;

/// Create a test project directory with .beads/ structure
fn create_test_project(name: &str) -> (tempfile::TempDir, PathBuf) {
    let project_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let project_path = project_dir.path().to_path_buf();

    // Create .beads directory with minimal structure
    let beads_dir = project_path.join(".beads");
    fs::create_dir_all(&beads_dir).expect("Failed to create .beads dir");

    // Create empty issues.jsonl (required for bead reader)
    let issues_path = beads_dir.join("issues.jsonl");
    fs::write(&issues_path, b"").expect("Failed to create issues.jsonl");

    // Create events.jsonl for session tailer
    let events_path = beads_dir.join("events.jsonl");
    fs::write(&events_path, b"").expect("Failed to create events.jsonl");

    (project_dir, project_path)
}

/// Remove .beads/ directory for a project
fn remove_beads_dir(project_path: &PathBuf) {
    let beads_dir = project_path.join(".beads");
    fs::remove_dir_all(&beads_dir).expect("Failed to remove .beads dir");
}

/// Restore .beads/ directory for a project
fn restore_beads_dir(project_path: &PathBuf) {
    let beads_dir = project_path.join(".beads");
    fs::create_dir_all(&beads_dir).expect("Failed to recreate .beads dir");

    let issues_path = beads_dir.join("issues.jsonl");
    fs::write(&issues_path, b"").expect("Failed to recreate issues.jsonl");

    let events_path = beads_dir.join("events.jsonl");
    fs::write(&events_path, b"").expect("Failed to recreate events.jsonl");
}

#[tokio::test]
async fn test_beads_removal_shows_error_state() {
    // Create three projects
    let (_project_a_dir, project_a_path) = create_test_project("project_a");
    let (_project_b_dir, project_b_path) = create_test_project("project_b");
    let (_project_c_dir, project_c_path) = create_test_project("project_c");

    let project_a_path_clone = project_a_path.clone();
    let project_b_path_clone = project_b_path.clone();
    let project_c_path_clone = project_c_path.clone();

    // Spawn daemon with custom config pointing to all three projects
    let (base_url, _daemon) =
        spawn_test_daemon_with_config(Some(move |config: &mut hoop_daemon::Config| {
            // Customize projects.yaml to include all three projects
            let projects_yaml = format!(
                r#"projects:
  - name: project_a
    path: {}
    workspaces:
      - path: {}
        role: primary
  - name: project_b
    path: {}
    workspaces:
      - path: {}
        role: primary
  - name: project_c
    path: {}
    workspaces:
      - path: {}
        role: primary
"#,
                project_a_path_clone.display(),
                project_a_path_clone.display(),
                project_b_path_clone.display(),
                project_b_path_clone.display(),
                project_c_path_clone.display(),
                project_c_path_clone.display()
            );

            let hoop_dir = config.control_socket_path.parent().unwrap();
            fs::write(hoop_dir.join("projects.yaml"), projects_yaml)
                .expect("Failed to write projects.yaml");
        }))
        .await
        .expect("Failed to spawn test daemon");

    let client = reqwest::Client::new();

    // Wait for all projects to become healthy initially
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(10) {
        let resp = client
            .get(&format!("{}/api/projects", base_url))
            .send()
            .await
            .expect("Failed to GET /api/projects");

        if resp.status().is_success() {
            let projects: serde_json::Value = resp
                .json()
                .await
                .expect("Failed to parse projects response");

            let all_healthy = projects.as_array().unwrap().iter().all(|p| {
                p.get("state")
                    .and_then(|s| s.as_str())
                    .map(|s| s == "healthy")
                    .unwrap_or(false)
            });

            if all_healthy {
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    // Verify all projects are healthy initially
    let resp = client
        .get(&format!("{}/readyz", base_url))
        .send()
        .await
        .expect("Failed to GET /readyz");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "All projects should be healthy initially"
    );

    // Remove .beads/ from project A
    remove_beads_dir(&project_a_path);

    // Wait for project A to show error state (within 30s)
    let start = std::time::Instant::now();
    let mut project_a_error = None;

    while start.elapsed() < Duration::from_secs(30) {
        let resp = client
            .get(&format!("{}/api/projects", base_url))
            .send()
            .await
            .expect("Failed to GET /api/projects");

        if resp.status().is_success() {
            let projects: serde_json::Value = resp
                .json()
                .await
                .expect("Failed to parse projects response");

            for project in projects.as_array().unwrap() {
                let name = project.get("name").and_then(|n| n.as_str()).unwrap();
                let state = project.get("state").and_then(|s| s.as_str()).unwrap();

                if name == "project_a" && state != "healthy" {
                    project_a_error = Some(state.to_string());
                    break;
                }
            }

            if project_a_error.is_some() {
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    assert!(
        project_a_error.is_some(),
        "Project A should show error state within 30s after .beads/ removal"
    );

    // Verify projects B and C are still healthy
    let resp = client
        .get(&format!("{}/api/projects", base_url))
        .send()
        .await
        .expect("Failed to GET /api/projects");

    let projects: serde_json::Value = resp
        .json()
        .await
        .expect("Failed to parse projects response");

    let project_b_healthy = projects
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p.get("name").and_then(|n| n.as_str()) == Some("project_b"))
        .all(|p| p.get("state").and_then(|s| s.as_str()) == Some("healthy"));

    let project_c_healthy = projects
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p.get("name").and_then(|n| n.as_str()) == Some("project_c"))
        .all(|p| p.get("state").and_then(|s| s.as_str()) == Some("healthy"));

    assert!(
        project_b_healthy,
        "Project B should remain healthy during project A's degradation"
    );

    assert!(
        project_c_healthy,
        "Project C should remain healthy during project A's degradation"
    );

    // Verify /readyz reports degraded with project A listed
    let resp = client
        .get(&format!("{}/readyz", base_url))
        .send()
        .await
        .expect("Failed to GET /readyz");

    assert_eq!(
        resp.status().as_u16(),
        503,
        "/readyz should return 503 when any project is degraded"
    );

    let readiness: ReadinessResponse = resp
        .json()
        .await
        .expect("Failed to parse readiness response");

    assert_eq!(
        readiness.status, "degraded",
        "Readiness status should be degraded"
    );

    assert!(
        readiness.degraded.iter().any(|p| p.project == "project_a"),
        "/readyz should list project_a as degraded"
    );

    // Verify project B and C are NOT in the degraded list
    assert!(
        !readiness.degraded.iter().any(|p| p.project == "project_b"),
        "/readyz should NOT list project_b as degraded"
    );

    assert!(
        !readiness.degraded.iter().any(|p| p.project == "project_c"),
        "/readyz should NOT list project_c as degraded"
    );

    // Restore .beads/ for project A
    restore_beads_dir(&project_a_path);

    // Trigger a config reload to recover project A
    let resp = client
        .post(&format!("{}/api/config/reload", base_url))
        .send()
        .await
        .expect("Failed to POST /api/config/reload");

    assert!(resp.status().is_success(), "Config reload should succeed");

    // Wait for project A to recover (within 10s)
    let start = std::time::Instant::now();
    let mut project_a_recovered = false;

    while start.elapsed() < Duration::from_secs(10) {
        let resp = client
            .get(&format!("{}/readyz", base_url))
            .send()
            .await
            .expect("Failed to GET /readyz");

        if resp.status().as_u16() == 200 {
            project_a_recovered = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    assert!(
        project_a_recovered,
        "Project A should recover after .beads/ is restored and config is reloaded"
    );

    // Final verification: all projects healthy
    let resp = client
        .get(&format!("{}/readyz", base_url))
        .send()
        .await
        .expect("Failed to GET /readyz");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "All projects should be healthy after recovery"
    );
}

#[tokio::test]
async fn test_multiple_beads_removal_isolated() {
    // Test that removing .beads/ from multiple projects only affects those projects
    let (_project_a_dir, project_a_path) = create_test_project("project_a");
    let (_project_b_dir, project_b_path) = create_test_project("project_b");
    let (_project_c_dir, project_c_path) = create_test_project("project_c");

    let project_a_path_clone = project_a_path.clone();
    let project_b_path_clone = project_b_path.clone();
    let project_c_path_clone = project_c_path.clone();

    let (base_url, _daemon) =
        spawn_test_daemon_with_config(Some(move |config: &mut hoop_daemon::Config| {
            let projects_yaml = format!(
                r#"projects:
  - name: project_a
    path: {}
    workspaces:
      - path: {}
        role: primary
  - name: project_b
    path: {}
    workspaces:
      - path: {}
        role: primary
  - name: project_c
    path: {}
    workspaces:
      - path: {}
        role: primary
"#,
                project_a_path_clone.display(),
                project_a_path_clone.display(),
                project_b_path_clone.display(),
                project_b_path_clone.display(),
                project_c_path_clone.display(),
                project_c_path_clone.display()
            );

            let hoop_dir = config.control_socket_path.parent().unwrap();
            fs::write(hoop_dir.join("projects.yaml"), projects_yaml)
                .expect("Failed to write projects.yaml");
        }))
        .await
        .expect("Failed to spawn test daemon");

    let client = reqwest::Client::new();

    // Wait for all projects to become healthy
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(10) {
        let resp = client
            .get(&format!("{}/readyz", base_url))
            .send()
            .await
            .expect("Failed to GET /readyz");
        if resp.status().as_u16() == 200 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    // Remove .beads/ from project A and B
    remove_beads_dir(&project_a_path);
    remove_beads_dir(&project_b_path);

    // Wait for degradation to be detected
    let start = std::time::Instant::now();
    let mut degraded_count = 0;

    while start.elapsed() < Duration::from_secs(30) {
        let resp = client
            .get(&format!("{}/readyz", base_url))
            .send()
            .await
            .expect("Failed to GET /readyz");

        if resp.status().as_u16() == 503 {
            let readiness: ReadinessResponse = resp
                .json()
                .await
                .expect("Failed to parse readiness response");

            degraded_count = readiness.degraded.len();
            if degraded_count >= 2 {
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    assert!(
        degraded_count >= 2,
        "At least 2 projects should be degraded"
    );

    // Verify project C is still healthy
    let resp = client
        .get(&format!("{}/api/projects", base_url))
        .send()
        .await
        .expect("Failed to GET /api/projects");

    let projects: serde_json::Value = resp
        .json()
        .await
        .expect("Failed to parse projects response");

    let project_c_healthy = projects
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p.get("name").and_then(|n| n.as_str()) == Some("project_c"))
        .all(|p| p.get("state").and_then(|s| s.as_str()) == Some("healthy"));

    assert!(
        project_c_healthy,
        "Project C should remain healthy even when A and B are degraded"
    );
}
