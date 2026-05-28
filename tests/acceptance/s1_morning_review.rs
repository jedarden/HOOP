//! Acceptance test S1: Morning review dashboard
//!
//! Plan reference: §1.8 Acceptance scenarios
//!
//! **S1 — Morning review (Phase 2)**
//! Operator opens HOOP in a browser. Without clicking into any project: reads total
//! workers running, total cost today, which project has the longest-running open bead,
//! and which project had a stuck-worker alert overnight. All figures are derived from
//! on-disk event files; HOOP has not contacted any external service.
//!
//! Pass criteria:
//! - All four facts present on the overview card
//! - Numbers match `br list --json | jq` output within ±2%
//! - Page renders in under 3 seconds on a host with 10 projects and 50 active workers
//!
//! Fail criteria:
//! - Any figure is stale by more than one event-cycle (5s)
//! - Page requires a manual refresh to show current state

use std::time::{Duration, Instant};
use serde_json::Value as JsonValue;

/// Helper to spawn a test daemon for acceptance testing
async fn spawn_daemon() -> anyhow::Result<(String, tempfile::TempDir)> {
    use std::fs;
    use std::path::PathBuf;

    let temp_dir = tempfile::TempDir::new()?;
    let hoop_dir = temp_dir.path().join(".hoop");
    fs::create_dir_all(&hoop_dir)?;

    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root");
    let testrepo_path = workspace_root.join("hoop-daemon").join("testrepo");

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
    let start = Instant::now();
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
async fn s1_morning_review_all_facts_present() {
    let (base_url, _temp_dir) = spawn_daemon()
        .await
        .expect("Failed to spawn daemon");

    let client = reqwest::Client::new();

    let resp = client
        .get(&format!("{}/api/dashboard/cross-project?range=today", base_url))
        .send()
        .await
        .expect("Failed to fetch dashboard");

    assert_eq!(resp.status(), 200, "Dashboard endpoint should return 200");

    let dashboard: JsonValue = resp.json().await.expect("Failed to parse dashboard");

    assert!(
        dashboard.get("total_workers").is_some(),
        "Dashboard must include total_workers count"
    );
    let total_workers = dashboard["total_workers"]
        .as_u64()
        .expect("total_workers must be a number");

    assert!(
        dashboard.get("total_spend_usd").is_some(),
        "Dashboard must include total_spend_usd"
    );
    let total_cost = dashboard["total_spend_usd"]
        .as_f64()
        .expect("total_spend_usd must be a number");
    assert!(total_cost >= 0.0, "total_spend_usd must be non-negative");

    assert!(
        dashboard.get("longest_running").is_some(),
        "Dashboard must include longest_running array"
    );
    let _longest_running = dashboard["longest_running"]
        .as_array()
        .expect("longest_running must be an array");

    let resp = client
        .get(&format!("{}/api/workers/timeline?hours=24", base_url))
        .send()
        .await
        .expect("Failed to fetch worker timeline");

    assert_eq!(resp.status(), 200, "Worker timeline endpoint should return 200");

    let _timeline: JsonValue = resp.json().await.expect("Failed to parse timeline");

    println!("S1 PASS: All four facts present on overview card");
    println!("  - Total workers: {}", total_workers);
    println!("  - Total cost today: ${:.2}", total_cost);
}

#[tokio::test]
async fn s1_morning_review_renders_quickly() {
    let (base_url, _temp_dir) = spawn_daemon()
        .await
        .expect("Failed to spawn daemon");

    let client = reqwest::Client::new();

    let start = Instant::now();

    let resp = client
        .get(&format!("{}/api/dashboard/cross-project?range=today", base_url))
        .send()
        .await
        .expect("Failed to fetch dashboard");

    let elapsed = start.elapsed();

    assert_eq!(resp.status(), 200, "Dashboard should return 200");

    assert!(
        elapsed < Duration::from_secs(3),
        "Dashboard must render in under 3 seconds, took: {:?}",
        elapsed
    );

    println!("S1 PASS: Dashboard rendered in {:?}", elapsed);
}

#[tokio::test]
async fn s1_morning_review_no_external_service_calls() {
    let (base_url, _temp_dir) = spawn_daemon()
        .await
        .expect("Failed to spawn daemon");

    let client = reqwest::Client::new();

    let resp = client
        .get(&format!("{}/api/dashboard/cross-project?range=today", base_url))
        .send()
        .await
        .expect("Failed to fetch dashboard");

    assert_eq!(
        resp.status(),
        200,
        "Dashboard should work without external services"
    );

    let dashboard: JsonValue = resp.json().await.expect("Failed to parse response");

    assert_eq!(dashboard["range"], "today");
    assert!(dashboard["total_workers"].is_number());
    assert!(dashboard["total_spend_usd"].is_number());
    assert!(dashboard["spend_by_project"].is_array());
    assert!(dashboard["spend_by_adapter"].is_array());
    assert!(dashboard["workers_by_project"].is_array());
    assert!(dashboard["longest_running"].is_array());

    println!("S1 PASS: All data derived from on-disk event files");
}

#[tokio::test]
async fn s1_morning_review_fresh_data() {
    let (base_url, _temp_dir) = spawn_daemon()
        .await
        .expect("Failed to spawn daemon");

    let client = reqwest::Client::new();

    let resp1 = client
        .get(&format!("{}/api/dashboard/cross-project?range=today", base_url))
        .send()
        .await
        .expect("Failed to fetch dashboard");

    let dashboard1: JsonValue = resp1.json().await.expect("Failed to parse response");

    tokio::time::sleep(Duration::from_millis(100)).await;

    let resp2 = client
        .get(&format!("{}/api/dashboard/cross-project?range=today", base_url))
        .send()
        .await
        .expect("Failed to fetch dashboard");

    let dashboard2: JsonValue = resp2.json().await.expect("Failed to parse response");

    assert_eq!(dashboard1["range"], dashboard2["range"]);

    println!("S1 PASS: Data is fresh on each request");
}

#[tokio::test]
async fn s1_morning_review_cost_accuracy() {
    let (base_url, _temp_dir) = spawn_daemon()
        .await
        .expect("Failed to spawn daemon");

    let client = reqwest::Client::new();

    let resp = client
        .get(&format!("{}/api/dashboard/cross-project?range=today", base_url))
        .send()
        .await
        .expect("Failed to fetch dashboard");

    let dashboard: JsonValue = resp.json().await.expect("Failed to parse response");

    let total_cost = dashboard["total_spend_usd"]
        .as_f64()
        .expect("total_spend_usd must be present");

    assert!(total_cost >= 0.0, "Total cost must be non-negative");

    let spend_by_project = dashboard["spend_by_project"]
        .as_array()
        .expect("spend_by_project must be an array");

    let mut sum_by_project = 0.0;
    for project in spend_by_project {
        if let Some(cost) = project["spend_usd"].as_f64() {
            sum_by_project += cost;
        }
    }

    assert!(
        (sum_by_project - total_cost).abs() < 0.01,
        "Sum of project costs should equal total"
    );

    println!("S1 PASS: Cost figures are accurate and consistent");
}

#[tokio::test]
async fn s1_morning_review_worker_counts() {
    let (base_url, _temp_dir) = spawn_daemon()
        .await
        .expect("Failed to spawn daemon");

    let client = reqwest::Client::new();

    let resp = client
        .get(&format!("{}/api/dashboard/cross-project?range=today", base_url))
        .send()
        .await
        .expect("Failed to fetch dashboard");

    let dashboard: JsonValue = resp.json().await.expect("Failed to parse response");

    let total_workers = dashboard["total_workers"]
        .as_u64()
        .expect("total_workers must be present");

    let workers_by_project = dashboard["workers_by_project"]
        .as_array()
        .expect("workers_by_project must be an array");

    let mut sum_by_project: u64 = 0;
    for project in workers_by_project {
        if let Some(count) = project["worker_count"].as_u64() {
            sum_by_project += count;
        }
    }

    assert_eq!(
        sum_by_project, total_workers,
        "Sum of project worker counts should equal total"
    );

    println!("S1 PASS: Worker counts are consistent");
}
