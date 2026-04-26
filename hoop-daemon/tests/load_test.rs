//! Load-test driver: synthetic event stream generator vs daemon
//!
//! This test module re-exports the load_test library for use in integration tests.
//!
//! Plan reference: §14.2 bullet 5
//! Feeds into hoop-ttb.7.11 performance budget verification

// Re-export the library module for tests
pub use hoop_daemon::load_test::*;

#[test]
fn test_load_test_config_defaults() {
    let config = LoadTestConfig::default();
    assert_eq!(config.num_projects, 20);
    assert_eq!(config.workers_per_project, 5);
    assert_eq!(config.beads_per_worker, 200);
    assert_eq!(config.total_beads(), 20 * 5 * 200);
    assert_eq!(config.total_workers(), 20 * 5);
}

#[test]
fn test_load_test_config_env_override() {
    std::env::set_var("HOOP_LOAD_PROJECTS", "5");
    std::env::set_var("HOOP_LOAD_WORKERS", "3");
    std::env::set_var("HOOP_LOAD_BEADS", "100");

    let config = LoadTestConfig::default();
    assert_eq!(config.num_projects, 5);
    assert_eq!(config.workers_per_project, 3);
    assert_eq!(config.beads_per_worker, 100);
    assert_eq!(config.total_beads(), 5 * 3 * 100);

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
    // Small-scale integration test with a real daemon
    // Note: This test requires a running daemon or can use the integration_harness
    // For now, we'll just verify the config is valid
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

    // TODO: Add daemon spawn and full load test execution
    // The integration_harness module provides spawn_test_daemon()
    // but integration tests cannot see each other directly.
}
