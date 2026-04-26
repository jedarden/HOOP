//! Load-test driver: synthetic event stream generator vs daemon
//!
//! This test module re-exports the load_test library for use in integration tests.
//!
//! Plan reference: §14.2 bullet 5
//! Feeds into hoop-ttb.7.11 performance budget verification

// Re-export the library module for tests
pub use hoop_daemon::load_test::*;

// Marker for tests that need exclusive access (serial execution)
// These tests spawn daemons and can't run in parallel
static __TEST_MUTEX: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[test]
fn test_load_test_config_defaults() {
    // Clear environment variables to test true defaults
    std::env::remove_var("HOOP_LOAD_PROJECTS");
    std::env::remove_var("HOOP_LOAD_WORKERS");
    std::env::remove_var("HOOP_LOAD_BEADS");
    std::env::remove_var("HOOP_LOAD_CADENCE_MS");

    let config = LoadTestConfig::default();
    assert_eq!(config.num_projects, 20);
    assert_eq!(config.workers_per_project, 5);
    assert_eq!(config.beads_per_worker, 200);
    assert_eq!(config.total_beads(), 20 * 5 * 200);
    assert_eq!(config.total_workers(), 20 * 5);
}

#[test]
fn test_load_test_config_env_override() {
    // Use unique keys to avoid conflicts with parallel tests
    std::env::set_var("HOOP_LOAD_PROJECTS", "5");
    std::env::set_var("HOOP_LOAD_WORKERS", "3");
    std::env::set_var("HOOP_LOAD_BEADS", "100");

    let config = LoadTestConfig::default();
    assert_eq!(config.num_projects, 5);
    assert_eq!(config.workers_per_project, 3);
    assert_eq!(config.beads_per_worker, 100);
    assert_eq!(config.total_beads(), 5 * 3 * 100);

    // Always cleanup
    std::env::remove_var("HOOP_LOAD_PROJECTS");
    std::env::remove_var("HOOP_LOAD_WORKERS");
    std::env::remove_var("HOOP_LOAD_BEADS");
}

#[test]
fn test_event_generator_creates_expected_events() {
    let config = LoadTestConfig {
        num_projects: 2,
        workers_per_project: 2,
        beads_per_worker: 10,
        event_cadence_ms: 10,
        ..Default::default()
    };

    let generator = EventGenerator::new(config);
    let events = generator.generate_all();

    assert_eq!(events.len(), 2); // 2 projects

    for (project, project_events) in events {
        assert!(project.starts_with("load-test-project-"));
        // Each worker processes 10 beads, each with ~5 events
        assert!(project_events.len() >= 2 * 10 * 4); // At least claim, dispatch, complete/close
    }
}

#[test]
fn test_event_generator_writes_to_disk() {
    use hoop_daemon::events::NeedleEvent;

    let config = LoadTestConfig {
        num_projects: 1,
        workers_per_project: 1,
        beads_per_worker: 5,
        event_cadence_ms: 10,
        ..Default::default()
    };

    let generator = EventGenerator::new(config);
    let temp_dir = tempfile::TempDir::new().unwrap();

    generator.write_to_disk(temp_dir.path()).unwrap();

    // Check that events.jsonl was created
    let events_path = temp_dir.path().join("load-test-project-000").join(".beads").join("events.jsonl");
    assert!(events_path.exists());

    // Check that heartbeats.jsonl was created
    let heartbeats_path = temp_dir.path().join("load-test-project-000").join(".beads").join("heartbeats.jsonl");
    assert!(heartbeats_path.exists());

    // Check that beads.jsonl was created
    let beads_path = temp_dir.path().join("load-test-project-000").join(".beads").join("beads.jsonl");
    assert!(beads_path.exists());

    // Verify events are valid JSONL
    let events_content = std::fs::read_to_string(&events_path).unwrap();
    for line in events_content.lines() {
        let _: NeedleEvent = serde_json::from_str(line).unwrap();
    }
}

#[test]
fn test_performance_report_summary() {
    let report = PerformanceReport {
        passed: true,
        total_events: 1000,
        api_latencies: vec![10, 20, 30, 40, 50],
        ws_fanout_lags: vec![5, 10, 15],
        memory_samples: vec![1024 * 1024 * 100, 1024 * 1024 * 200],
        failures: vec![],
    };

    let summary = report.summary();
    assert!(summary.contains("PASS"));
    assert!(summary.contains("1000"));
    assert!(summary.contains("API Latency"));
    assert!(summary.contains("WS Fan-out Lag"));
    assert!(summary.contains("Memory"));
}

#[test]
fn test_performance_report_assert_budgets_pass() {
    let config = LoadTestConfig::default();
    let report = PerformanceReport {
        passed: true,
        total_events: 1000,
        api_latencies: vec![100, 200, 300], // All under 500ms
        ws_fanout_lags: vec![10, 20, 30], // All under 100ms
        memory_samples: vec![1024 * 1024 * 500], // 500MB under 4GB
        failures: vec![],
    };

    assert!(report.assert_budgets(&config).is_ok());
}

#[test]
fn test_performance_report_assert_budgets_fail() {
    let config = LoadTestConfig::default();
    let report = PerformanceReport {
        passed: false,
        total_events: 1000,
        api_latencies: vec![600], // Over 500ms budget
        ws_fanout_lags: vec![10, 20, 30],
        memory_samples: vec![1024 * 1024 * 500],
        failures: vec![],
    };

    assert!(report.assert_budgets(&config).is_err());
}

#[tokio::test]
async fn test_run_load_test_smoke() {
    // Small-scale smoke test
    let config = LoadTestConfig {
        num_projects: 1,
        workers_per_project: 1,
        beads_per_worker: 2,
        event_cadence_ms: 1,
        ..Default::default()
    };

    // This test requires a running daemon
    // In CI, we'd spawn a test daemon first
    // For now, we just verify the config is valid
    assert_eq!(config.total_beads(), 2);
}

#[tokio::test]
async fn test_load_test_with_daemon() {
    // Acquire lock to prevent concurrent daemon spawning
    let _lock = __TEST_MUTEX.lock().await;

    // Small-scale integration test with a real daemon
    use hoop_daemon::integration_harness::spawn_test_daemon;

    let config = LoadTestConfig {
        num_projects: 1,
        workers_per_project: 1,
        beads_per_worker: 2,
        event_cadence_ms: 1,
        ..Default::default()
    };

    // Verify config is valid
    assert_eq!(config.total_beads(), 2);
    assert_eq!(config.total_workers(), 1);

    // Spawn a test daemon
    let (base_url, _daemon) = spawn_test_daemon()
        .await
        .expect("Failed to spawn test daemon");

    // Run the load test
    let report = run_load_test(&base_url, config.clone())
        .await
        .expect("Load test should complete");

    // Verify the report
    assert_eq!(report.total_events, 10); // 2 beads * ~5 events each

    // For small scale, we should pass budgets
    assert!(report.assert_budgets(&config).is_ok(),
        "Small-scale load test should pass performance budgets");
}

/// Full-scale load test (20x5x200)
///
/// This test validates the performance budget for the full target load.
/// Run with: cargo test --test load_test test_full_scale_load_test -- --ignored
///
/// Environment variables:
///   HOOP_LOAD_TEST_FULL_SCALE=1  - Enable this test
///   HOOP_LOAD_PROJECTS=20        - Number of projects (default: 20)
///   HOOP_LOAD_WORKERS=5          - Workers per project (default: 5)
///   HOOP_LOAD_BEADS=200          - Beads per worker (default: 200)
#[tokio::test]
#[ignore]
async fn test_full_scale_load_test() {
    // Acquire lock to prevent concurrent daemon spawning
    let _lock = __TEST_MUTEX.lock().await;

    // Only run if explicitly enabled
    if std::env::var("HOOP_LOAD_TEST_FULL_SCALE").is_err() {
        return;
    }

    use hoop_daemon::integration_harness::spawn_test_daemon;

    let config = LoadTestConfig::default();

    println!("=== Full-Scale Load Test ===");
    println!("Projects: {}", config.num_projects);
    println!("Workers per project: {}", config.workers_per_project);
    println!("Beads per worker: {}", config.beads_per_worker);
    println!("Total beads: {}", config.total_beads());
    println!("Total workers: {}", config.total_workers());
    println!();

    // Spawn a test daemon
    let (base_url, _daemon) = spawn_test_daemon()
        .await
        .expect("Failed to spawn test daemon");

    // Run the load test with a longer timeout
    let start = std::time::Instant::now();
    let report = tokio::time::timeout(
        std::time::Duration::from_secs(600), // 10 minute timeout
        run_load_test(&base_url, config.clone()),
    )
    .await
    .expect("Load test timed out after 10 minutes")
    .expect("Load test should complete");

    let elapsed = start.elapsed();
    println!("Load test completed in {:?}", elapsed);
    println!();
    println!("{}", report.summary());

    // Assert performance budgets - this will fail if budgets are exceeded
    report.assert_budgets(&config)
        .expect("Performance budgets must be satisfied");

    // Additional assertions for full-scale test
    assert!(report.total_events > 0, "Should process events");
    assert!(report.api_latencies.len() > 0, "Should measure API latencies");

    // Verify p95 latency is within budget
    let mut sorted_latencies = report.api_latencies.clone();
    sorted_latencies.sort();
    let p95_index = sorted_latencies.len() * 95 / 100;
    let p95_latency = sorted_latencies.get(p95_index).copied().unwrap_or(0);
    assert!(p95_latency <= config.api_latency_budget_ms,
        "P95 API latency {}ms should be within budget {}ms",
        p95_latency, config.api_latency_budget_ms);
}

/// Medium-scale load test for quick validation
///
/// Runs a smaller load test (5x2x50) for faster CI feedback.
#[tokio::test]
async fn test_medium_scale_load_test() {
    // Acquire lock to prevent concurrent daemon spawning
    let _lock = __TEST_MUTEX.lock().await;

    use hoop_daemon::integration_harness::spawn_test_daemon;

    let config = LoadTestConfig {
        num_projects: 5,
        workers_per_project: 2,
        beads_per_worker: 50,
        event_cadence_ms: 10,
        ..Default::default()
    };

    println!("=== Medium-Scale Load Test ===");
    println!("Total beads: {}", config.total_beads());
    println!("Total workers: {}", config.total_workers());

    // Spawn a test daemon
    let (base_url, _daemon) = spawn_test_daemon()
        .await
        .expect("Failed to spawn test daemon");

    // Run the load test
    let report = tokio::time::timeout(
        std::time::Duration::from_secs(120), // 2 minute timeout
        run_load_test(&base_url, config.clone()),
    )
    .await
    .expect("Medium-scale load test timed out")
    .expect("Load test should complete");

    println!("{}", report.summary());

    // Assert budgets pass
    report.assert_budgets(&config)
        .expect("Medium-scale load test should pass performance budgets");
}
