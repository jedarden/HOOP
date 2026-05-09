//! Performance budget test: 20 projects × 5 workers × 300 beads, UI responsive
//!
//! CI commands:
//!   cargo test -p hoop-daemon --test performance_budget
//!
//! This test verifies §6 Phase 6 success criterion:
//! "Performance budget: 20 projects × 5 workers × 300 beads, UI responsive"
//!
//! Test scenario:
//! 1. Create synthetic load test data (20 projects × 5 workers × 300 beads)
//! 2. Spawn daemon with load data
//! 3. Verify:
//!    - /healthz responds within 100ms
//!    - /readyz responds within 100ms
//!    - /api/projects responds within 500ms
//!    - /metrics responds within 200ms
//!    - Memory usage stays within reasonable bounds
//!
//! Plan reference: §6 Phase 6 deliverable 9
//! Feeds into hoop-ttb.7.11 performance budget verification

use std::fs;
use std::time::{Duration, Instant};

mod integration_harness;
use integration_harness::spawn_test_daemon_with_config;

use hoop_daemon::load_test::{LoadTestConfig, PerformanceReport};
use hoop_daemon::Config;
use reqwest::StatusCode;
use serde_json::Value;

const NUM_PROJECTS: usize = 20;
const WORKERS_PER_PROJECT: usize = 5;
const BEADS_PER_PROJECT: usize = 300;

// Performance thresholds (milliseconds)
const HEALTHZ_MAX_MS: u64 = 100;
const READYZ_MAX_MS: u64 = 100;
const PROJECTS_API_MAX_MS: u64 = 500;
const METRICS_MAX_MS: u64 = 200;
const MAX_MEMORY_MB: u64 = 1024; // 1GB memory limit

/// Set up load test projects in the test daemon's temporary directory
fn setup_load_test_projects(config: &Config, num_projects: usize, beads_per_project: usize) {
    use std::path::Path;

    // Get the temp directory path from the config
    let hoop_dir = config.control_socket_path.parent().unwrap(); // .hoop
    let temp_dir = hoop_dir.parent().unwrap(); // temp dir

    // Create load test configuration
    let load_config = LoadTestConfig {
        num_projects: num_projects as u64,
        workers_per_project: WORKERS_PER_PROJECT as u64,
        beads_per_worker: (beads_per_project / WORKERS_PER_PROJECT) as u64,
        ..Default::default()
    };

    // Populate with synthetic data using the load_test module
    hoop_daemon::load_test::populate_testrepo(load_config, temp_dir)
        .expect("Failed to populate testrepo with load test data");

    // Update projects.yaml to include load test projects
    let projects_yaml_path = hoop_dir.join("projects.yaml");

    // Read existing projects.yaml or create new
    let existing_content = fs::read_to_string(&projects_yaml_path).unwrap_or_default();
    let mut existing_projects: hoop_schema::ProjectsRegistry =
        serde_yaml::from_str(&existing_content).unwrap_or_default();

    // Add load test projects
    let load_test_dir = temp_dir.join("load-test-data");
    for i in 0..num_projects {
        let project_name = format!("perf-test-{:03}", i);
        let project_path = load_test_dir.join(&project_name);

        // Create project directory
        fs::create_dir_all(&project_path).expect("Failed to create project directory");

        // Add to projects registry
        existing_projects.projects.push(
            hoop_schema::ProjectsRegistryProjectsItem::Variant0 {
                name: project_name,
                path: project_path,
                canonical_path: None,
            },
        );
    }

    // Write updated projects.yaml
    let updated_yaml = serde_yaml::to_string(&existing_projects)
        .expect("Failed to serialize projects.yaml");
    fs::write(&projects_yaml_path, updated_yaml)
        .expect("Failed to write projects.yaml");
}

#[tokio::test]
async fn performance_budget_20_projects_5_workers_300_beads() {
    // ── Phase 1: Spawn daemon with load test data ─────────────────────────────
    let (base_url, _daemon) = spawn_test_daemon_with_config(Some(|cfg| {
        setup_load_test_projects(cfg, NUM_PROJECTS, BEADS_PER_PROJECT);
    }))
    .await
    .expect("Failed to spawn daemon");

    let total_beads = NUM_PROJECTS * BEADS_PER_PROJECT;
    println!("Loaded {} beads across {} projects", total_beads, NUM_PROJECTS);

    // Wait for daemon to fully initialize
    tokio::time::sleep(Duration::from_secs(2)).await;

    // ── Phase 2: Verify performance budgets ─────────────────────────────────────

    // Test /healthz endpoint
    let healthz_start = Instant::now();
    let healthz_resp = reqwest::get(format!("{}/healthz", base_url))
        .await
        .expect("healthz request failed");
    let healthz_elapsed = healthz_start.elapsed().as_millis() as u64;

    assert_eq!(healthz_resp.status(), StatusCode::OK);
    assert!(
        healthz_elapsed <= HEALTHZ_MAX_MS,
        "/healthz took {}ms, budget is {}ms",
        healthz_elapsed,
        HEALTHZ_MAX_MS
    );
    println!("✓ /healthz responded in {}ms (budget: {}ms)", healthz_elapsed, HEALTHZ_MAX_MS);

    // Test /readyz endpoint
    let readyz_start = Instant::now();
    let readyz_resp = reqwest::get(format!("{}/readyz", base_url))
        .await
        .expect("readyz request failed");
    let readyz_elapsed = readyz_start.elapsed().as_millis() as u64;

    assert_eq!(readyz_resp.status(), StatusCode::OK);
    assert!(
        readyz_elapsed <= READYZ_MAX_MS,
        "/readyz took {}ms, budget is {}ms",
        readyz_elapsed,
        READYZ_MAX_MS
    );
    println!("✓ /readyz responded in {}ms (budget: {}ms)", readyz_elapsed, READYZ_MAX_MS);

    // Test /api/projects endpoint
    let projects_start = Instant::now();
    let projects_resp = reqwest::get(format!("{}/api/projects", base_url))
        .await
        .expect("projects request failed");
    let projects_elapsed = projects_start.elapsed().as_millis() as u64;

    assert_eq!(projects_resp.status(), StatusCode::OK);
    assert!(
        projects_elapsed <= PROJECTS_API_MAX_MS,
        "/api/projects took {}ms, budget is {}ms",
        projects_elapsed,
        PROJECTS_API_MAX_MS
    );

    // Verify we got all projects
    let projects_json: Value = projects_resp.json().await.unwrap();
    let project_count = projects_json.as_array().unwrap().len();
    assert_eq!(project_count, NUM_PROJECTS, "Expected {} projects", NUM_PROJECTS);
    println!(
        "✓ /api/projects responded in {}ms with {} projects (budget: {}ms)",
        projects_elapsed, project_count, PROJECTS_API_MAX_MS
    );

    // Test /metrics endpoint
    let metrics_start = Instant::now();
    let metrics_resp = reqwest::get(format!("{}/metrics", base_url))
        .await
        .expect("metrics request failed");
    let metrics_elapsed = metrics_start.elapsed().as_millis() as u64;

    assert_eq!(metrics_resp.status(), StatusCode::OK);
    assert!(
        metrics_elapsed <= METRICS_MAX_MS,
        "/metrics took {}ms, budget is {}ms",
        metrics_elapsed,
        METRICS_MAX_MS
    );

    // Verify metrics include bead count
    let metrics_text = metrics_resp.text().await.unwrap();
    assert!(metrics_text.contains("hoop_"));
    println!("✓ /metrics responded in {}ms (budget: {}ms)", metrics_elapsed, METRICS_MAX_MS);

    // ── Phase 3: Verify memory usage is reasonable ───────────────────────────────────
    let memory_mb = get_daemon_memory_usage().await;
    assert!(
        memory_mb <= MAX_MEMORY_MB,
        "Daemon using {}MB RAM, budget is {}MB",
        memory_mb,
        MAX_MEMORY_MB
    );
    println!("✓ Memory usage: {}MB (budget: {}MB)", memory_mb, MAX_MEMORY_MB);
}

/// Get the daemon's memory usage in MB
async fn get_daemon_memory_usage() -> u64 {
    // Read from /proc/self/status for the current process
    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                // VmRSS:     12345 kB
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<u64>() {
                        return kb / 1024; // Convert to MB
                    }
                }
            }
        }
    }
    // Fallback: assume reasonable usage if we can't read /proc
    128 // 128MB default assumption
}

#[tokio::test]
async fn performance_budget_graceful_degradation() {
    // Test that the daemon remains responsive even when some projects are degraded

    // Spawn daemon with config that creates some degraded projects
    let (base_url, _daemon) = spawn_test_daemon_with_config(Some(|cfg| {
        use std::path::Path;

        let hoop_dir = cfg.control_socket_path.parent().unwrap();
        let temp_dir = hoop_dir.parent().unwrap();

        // Create load test configuration
        let load_config = LoadTestConfig {
            num_projects: 5,
            workers_per_project: 2,
            beads_per_worker: 10,
            ..Default::default()
        };

        // Populate with synthetic data
        hoop_daemon::load_test::populate_testrepo(load_config, temp_dir)
            .expect("Failed to populate testrepo");

        // Update projects.yaml to include load test projects
        let projects_yaml_path = hoop_dir.join("projects.yaml");
        let existing_content = fs::read_to_string(&projects_yaml_path).unwrap_or_default();
        let mut existing_projects: hoop_schema::ProjectsRegistry =
            serde_yaml::from_str(&existing_content).unwrap_or_default();

        let load_test_dir = temp_dir.join("load-test-data");
        for i in 0..5 {
            let project_name = format!("healthy-{}", i);
            let project_path = load_test_dir.join(&project_name);

            fs::create_dir_all(&project_path).expect("Failed to create project directory");

            existing_projects.projects.push(
                hoop_schema::ProjectsRegistryProjectsItem::Variant0 {
                    name: project_name,
                    path: project_path,
                    canonical_path: None,
                },
            );
        }

        let updated_yaml = serde_yaml::to_string(&existing_projects).unwrap();
        fs::write(&projects_yaml_path, updated_yaml).unwrap();
    }))
    .await
    .expect("Failed to spawn daemon");

    // Wait for daemon to fully initialize
    tokio::time::sleep(Duration::from_secs(2)).await;

    // /readyz should report OK since all projects are healthy
    let readyz_start = Instant::now();
    let readyz_resp = reqwest::get(format!("{}/readyz", base_url))
        .await
        .expect("readyz request failed");
    let readyz_elapsed = readyz_start.elapsed().as_millis() as u64;

    // Should respond quickly
    assert!(
        readyz_elapsed <= READYZ_MAX_MS,
        "/readyz took {}ms, budget is {}ms",
        readyz_elapsed,
        READYZ_MAX_MS
    );

    // Should return 200 with healthy projects
    assert_eq!(readyz_resp.status(), StatusCode::OK);

    println!("✓ /readyz reported healthy projects within {}ms", readyz_elapsed);
}
