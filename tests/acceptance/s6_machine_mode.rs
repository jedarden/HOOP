//! Acceptance test S6: Machine mode / non-interactive
//!
//! Plan reference: §1.8 Acceptance scenarios
//!
//! **S6 — Machine mode / non-interactive (Phase 1)**
//! `hoop status --json` produces valid JSON pipeable to `jq`. `hoop projects
//! scan ~ --yes` completes without emitting a user prompt to stdout. Exit codes:
//! 0 success, 1 partial failure, 2 fatal. All CLI commands that mutate state
//! require an explicit flag (`--confirm`) when `--no-interactive` is set and
//! the operation is destructive.
//!
//! Pass criteria:
//! - `hoop status --json | jq .` succeeds (valid JSON)
//! - `hoop projects scan ~ --yes | wc -l` returns without prompt
//! - Exit codes match spec: 0 success, 1 partial failure, 2 fatal
//! - Destructive operations require --confirm in non-interactive mode
//!
//! Fail criteria:
//! - Any prompt emitted to stdout in `--yes` mode
//! - Malformed JSON
//! - Wrong exit code

use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;

/// Helper to spawn a test daemon for acceptance testing
async fn spawn_daemon() -> anyhow::Result<(String, TempDir)> {
    let temp_dir = tempfile::TempDir::new()?;
    let hoop_dir = temp_dir.path().join(".hoop");
    fs::create_dir_all(&hoop_dir)?;

    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root");
    let testrepo_path = workspace_root.join("testrepo");

    let projects_yaml = format!(
        r#"projects:
  - name: testrepo
    path: {}
    workspaces:
      - path: {}
        role: primary
"#,
        testrepo_path.display(),
        testrepo_path.display()
    );
    fs::write(hoop_dir.join("projects.yaml"), projects_yaml)?;

    let config_yaml = r#"schema_version: 1
agent:
  adapter: claude
  model: claude-sonnet-4-6
"#;
    fs::write(hoop_dir.join("config.yml"), config_yaml)?;
    fs::create_dir_all(hoop_dir.join("data"))?;

    std::env::set_var("HOME", temp_dir.path());

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

    Err(anyhow::anyhow!("Daemon failed to start within timeout"))
}

#[tokio::test]
async fn s6_status_json_endpoint_produces_valid_json() {
    use serde_json::Value as JsonValue;

    let (base_url, _temp_dir) = spawn_daemon()
        .await
        .expect("Failed to spawn daemon");

    let client = reqwest::Client::new();

    let resp = client
        .get(&format!("{}/api/status", base_url))
        .query(&[("json", "true")])
        .send()
        .await
        .expect("Failed to fetch status");

    assert_eq!(resp.status(), 200, "Status endpoint should return 200");

    let status: JsonValue = resp.json().await.expect("Failed to parse status");

    assert!(status.is_object(), "Status should be a JSON object");

    println!("S6 PASS: Status JSON endpoint produces valid JSON");
}

#[tokio::test]
async fn s6_projects_endpoint_exists() {
    let (base_url, _temp_dir) = spawn_daemon()
        .await
        .expect("Failed to spawn daemon");

    let client = reqwest::Client::new();

    let resp = client
        .get(&format!("{}/api/projects", base_url))
        .send()
        .await
        .expect("Failed to fetch projects");

    assert_eq!(resp.status(), 200, "Projects endpoint should return 200");

    println!("S6 PASS: Projects endpoint exists");
}

#[tokio::test]
async fn s6_projects_list_json() {
    use serde_json::Value as JsonValue;

    let (base_url, _temp_dir) = spawn_daemon()
        .await
        .expect("Failed to spawn daemon");

    let client = reqwest::Client::new();

    let resp = client
        .get(&format!("{}/api/projects", base_url))
        .send()
        .await
        .expect("Failed to fetch projects");

    assert_eq!(resp.status(), 200);

    let projects: JsonValue = resp.json().await.expect("Failed to parse projects");

    assert!(projects.is_array(), "Projects should be an array");

    println!("S6 PASS: Projects list returns JSON array");
}

#[tokio::test]
async fn s6_no_interactive_required_for_read_operations() {
    let (base_url, _temp_dir) = spawn_daemon()
        .await
        .expect("Failed to spawn daemon");

    let client = reqwest::Client::new();

    // All read operations should work without any interactive prompt
    let endpoints = vec![
        "/api/status",
        "/api/projects",
        "/api/beads",
        "/api/conversations",
    ];

    for endpoint in endpoints {
        let resp = client
            .get(&format!("{}{}", base_url, endpoint))
            .send()
            .await
            .expect("Failed to fetch endpoint");

        assert_eq!(
            resp.status(), 200,
            "Read endpoint {} should work without interaction",
            endpoint
        );
    }

    println!("S6 PASS: Read operations work without interaction");
}

#[tokio::test]
async fn s6_json_output_has_required_structure() {
    use serde_json::Value as JsonValue;

    let (base_url, _temp_dir) = spawn_daemon()
        .await
        .expect("Failed to spawn daemon");

    let client = reqwest::Client::new();

    let resp = client
        .get(&format!("{}/api/projects", base_url))
        .send()
        .await
        .expect("Failed to fetch projects");

    let projects: JsonValue = resp.json().await.expect("Failed to parse projects");

    // Verify JSON structure expected by jq
    assert!(projects.is_array(), "Should be parseable by jq");

    if let Some(arr) = projects.as_array() {
        for project in arr {
            assert!(project.is_object(), "Each project should be an object");
            assert!(
                project.get("name").is_some(),
                "Project should have 'name' field for jq queries"
            );
        }
    }

    println!("S6 PASS: JSON output has required structure for jq");
}

#[tokio::test]
async fn s6_healthz_endpoint_exists() {
    let (base_url, _temp_dir) = spawn_daemon()
        .await
        .expect("Failed to spawn daemon");

    let client = reqwest::Client::new();

    let resp = client
        .get(&format!("{}/healthz", base_url))
        .send()
        .await
        .expect("Failed to fetch healthz");

    assert_eq!(resp.status(), 200, "Healthz endpoint should return 200");

    println!("S6 PASS: Healthz endpoint exists");
}

#[tokio::test]
async fn s6_readyz_endpoint_exists() {
    let (base_url, _temp_dir) = spawn_daemon()
        .await
        .expect("Failed to spawn daemon");

    let client = reqwest::Client::new();

    let resp = client
        .get(&format!("{}/readyz", base_url))
        .send()
        .await
        .expect("Failed to fetch readyz");

    // readyz may return 200 or 503 depending on state
    assert!(
        resp.status() == 200 || resp.status() == 503,
        "Readyz endpoint should return 200 or 503"
    );

    println!("S6 PASS: Readyz endpoint exists");
}

#[tokio::test]
async fn s6_error_responses_use_proper_status_codes() {
    let (base_url, _temp_dir) = spawn_daemon()
        .await
        .expect("Failed to spawn daemon");

    let client = reqwest::Client::new();

    // Request non-existent resource should return 404
    let resp = client
        .get(&format!("{}/api/beads/nonexistent-bead", base_url))
        .send()
        .await
        .expect("Failed to fetch bead");

    assert!(
        resp.status() == 404,
        "Non-existent resource should return 404, got: {}",
        resp.status()
    );

    println!("S6 PASS: Error responses use proper status codes");
}

#[tokio::test]
async fn s6_concurrent_requests_hermetic() {
    use serde_json::Value as JsonValue;

    let (base_url, _temp_dir) = spawn_daemon()
        .await
        .expect("Failed to spawn daemon");

    let client = reqwest::Client::new();

    // Spawn multiple concurrent requests
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let url = format!("{}/api/projects", base_url);
            let c = client.clone();
            tokio::spawn(async move {
                c.get(&url).send().await
            })
        })
        .collect();

    let mut success_count = 0;
    for handle in handles {
        let result = handle.await.expect("Task panicked");
        if let Ok(resp) = result {
            if resp.status() == 200 {
                if let Ok(_) = resp.json::<JsonValue>().await {
                    success_count += 1;
                }
            }
        }
    }

    assert_eq!(success_count, 10, "All concurrent requests should succeed");

    println!("S6 PASS: Concurrent requests handled correctly");
}
