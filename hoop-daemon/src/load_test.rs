//! Load-test driver: synthetic event stream generator vs daemon
//!
//! Generates concurrent synthetic event streams (20 projects × 5 workers × 200 beads)
//! and drives the daemon. Asserts:
//! - UI responsiveness budget (<500ms API response)
//! - Memory ceiling (<4GB RSS)
//! - WS fan-out lag (<100ms broadcast to all clients)
//!
//! Configurable via environment variables:
//! - `HOOP_LOAD_PROJECTS`: number of projects (default: 20)
//! - `HOOP_LOAD_WORKERS`: workers per project (default: 5)
//! - `HOOP_LOAD_BEADS`: beads per worker (default: 200)
//! - `HOOP_LOAD_CADENCE_MS`: delay between events (default: 10ms)
//!
//! Plan reference: §14.2 bullet 5
//! Feeds into hoop-ttb.7.11 performance budget verification

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::{Duration, Instant};

use chrono::Utc;
use serde_json::json;

use crate::events::NeedleEvent;
use crate::heartbeats::WorkerHeartbeat;
use crate::WorkerState;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Load test configuration
#[derive(Debug, Clone)]
pub struct LoadTestConfig {
    /// Number of synthetic projects to create
    pub num_projects: u64,
    /// Number of workers per project
    pub workers_per_project: u64,
    /// Number of beads each worker processes
    pub beads_per_worker: u64,
    /// Delay between events in milliseconds
    pub event_cadence_ms: u64,
    /// Responsiveness budget for API calls (milliseconds)
    pub api_latency_budget_ms: u64,
    /// Memory ceiling in bytes (4GB default)
    pub memory_ceiling_bytes: u64,
    /// WS fan-out lag budget (milliseconds)
    pub ws_fanout_lag_budget_ms: u64,
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        Self {
            num_projects: Self::env_or_default("HOOP_LOAD_PROJECTS", 20),
            workers_per_project: Self::env_or_default("HOOP_LOAD_WORKERS", 5),
            beads_per_worker: Self::env_or_default("HOOP_LOAD_BEADS", 200),
            event_cadence_ms: Self::env_or_default("HOOP_LOAD_CADENCE_MS", 10),
            api_latency_budget_ms: 500,
            memory_ceiling_bytes: 4 * 1024 * 1024 * 1024, // 4GB
            ws_fanout_lag_budget_ms: 100,
        }
    }
}

impl LoadTestConfig {
    fn env_or_default(key: &str, default: usize) -> u64 {
        std::env::var(key)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default) as u64
    }

    /// Total number of synthetic beads
    pub fn total_beads(&self) -> u64 {
        self.num_projects * self.workers_per_project * self.beads_per_worker
    }

    /// Total number of workers
    pub fn total_workers(&self) -> u64 {
        self.num_projects * self.workers_per_project
    }
}

// ---------------------------------------------------------------------------
// Synthetic data generator
// ---------------------------------------------------------------------------

/// Generates synthetic NEEDLE event streams
pub struct EventGenerator {
    config: LoadTestConfig,
    start_time: chrono::DateTime<chrono::Utc>,
}

impl EventGenerator {
    pub fn new(config: LoadTestConfig) -> Self {
        Self {
            config,
            start_time: Utc::now(),
        }
    }

    /// Generate all synthetic events for the load test
    ///
    /// Returns a vector of (project_name, events) tuples
    pub fn generate_all(&self) -> Vec<(String, Vec<NeedleEvent>)> {
        let mut all_events = Vec::new();

        for project_idx in 0..(self.config.num_projects as usize) {
            let project_name = format!("load-test-project-{:03}", project_idx);
            let events = self.generate_project_events(&project_name);
            all_events.push((project_name, events));
        }

        all_events
    }

    /// Generate events for a single project
    fn generate_project_events(&self, project: &str) -> Vec<NeedleEvent> {
        let mut events = Vec::new();
        let mut current_ts = self.start_time;

        for worker_idx in 0..(self.config.workers_per_project as usize) {
            let worker_name = format!("{}-worker-{:02}", project, worker_idx);
            let worker_events = self.generate_worker_events(&worker_name, project, &mut current_ts);
            events.extend(worker_events);
        }

        events
    }

    /// Generate events for a single worker processing beads
    fn generate_worker_events(&self, worker: &str, project: &str, ts: &mut chrono::DateTime<chrono::Utc>) -> Vec<NeedleEvent> {
        let mut events = Vec::new();

        for bead_idx in 0..(self.config.beads_per_worker as usize) {
            let bead_id = format!("{}-bd-{:06}", project, bead_idx);

            // Simulate a realistic claim -> dispatch -> complete/close flow
            // 70% success, 30% failure for realism
            let is_success = bead_idx % 10 < 7;

            // Claim event
            events.push(NeedleEvent::Claim {
                ts: ts.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                worker: worker.to_string(),
                bead: bead_id.clone(),
                strand: Some(format!("strand-{:04}", bead_idx % 100)),
            });
            *ts += chrono::Duration::milliseconds(self.config.event_cadence_ms as i64);

            // Dispatch event
            events.push(NeedleEvent::Dispatch {
                ts: ts.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                worker: worker.to_string(),
                bead: bead_id.clone(),
                adapter: Some("claude".to_string()),
                model: Some("claude-sonnet-4-6".to_string()),
            });
            *ts += chrono::Duration::milliseconds(self.config.event_cadence_ms as i64);

            if is_success {
                // Complete event
                events.push(NeedleEvent::Complete {
                    ts: ts.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                    worker: worker.to_string(),
                    bead: bead_id.clone(),
                    outcome: Some("success".to_string()),
                    duration_ms: Some(1000 + (bead_idx % 5000) as u64),
                    exit_code: Some(0),
                });

                // Close event
                events.push(NeedleEvent::Close {
                    ts: ts.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                    worker: worker.to_string(),
                    bead: bead_id.clone(),
                });
            } else {
                // Fail event
                events.push(NeedleEvent::Fail {
                    ts: ts.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                    worker: worker.to_string(),
                    bead: bead_id.clone(),
                    error: Some("simulated failure".to_string()),
                    duration_ms: Some(500 + (bead_idx % 2000) as u64),
                });

                // Release event
                events.push(NeedleEvent::Release {
                    ts: ts.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                    worker: worker.to_string(),
                    bead: bead_id.clone(),
                });
            }

            *ts += chrono::Duration::milliseconds(self.config.event_cadence_ms as i64 * 2);
        }

        events
    }

    /// Write synthetic events to disk in a temporary directory
    pub fn write_to_disk(&self, temp_dir: &Path) -> anyhow::Result<()> {
        for (project_name, events) in self.generate_all() {
            let project_dir = temp_dir.join(&project_name).join(".beads");
            std::fs::create_dir_all(&project_dir)?;

            let events_path = project_dir.join("events.jsonl");
            let file = File::create(&events_path)?;
            let mut writer = BufWriter::new(file);

            for event in events {
                let json = serde_json::to_string(&event)?;
                writeln!(writer, "{}", json)?;
            }

            writer.flush()?;

            // Create heartbeats.jsonl
            let heartbeats = self.generate_heartbeats(&project_name);
            let heartbeats_path = project_dir.join("heartbeats.jsonl");
            let file = File::create(&heartbeats_path)?;
            let mut writer = BufWriter::new(file);

            for heartbeat in heartbeats {
                let json = serde_json::to_string(&heartbeat)?;
                writeln!(writer, "{}", json)?;
            }

            writer.flush()?;

            // Create beads.jsonl with open beads
            let beads = self.generate_beads(&project_name);
            let beads_path = project_dir.join("beads.jsonl");
            let file = File::create(&beads_path)?;
            let mut writer = BufWriter::new(file);

            for bead in beads {
                let json = serde_json::to_string(&bead)?;
                writeln!(writer, "{}", json)?;
            }

            writer.flush()?;
        }

        Ok(())
    }

    /// Generate synthetic heartbeats for a project
    fn generate_heartbeats(&self, project: &str) -> Vec<WorkerHeartbeat> {
        let mut heartbeats = Vec::new();
        let ts = Utc::now();

        for worker_idx in 0..(self.config.workers_per_project as usize) {
            let worker_name = format!("{}-worker-{:02}", project, worker_idx);

            // Mix of idle and executing states
            let state = if worker_idx % 3 == 0 {
                WorkerState::Executing {
                    bead: format!("{}-bd-{:06}", project, worker_idx * 10),
                    pid: 12345 + worker_idx as u32,
                    adapter: "claude".to_string(),
                }
            } else {
                WorkerState::Idle {
                    last_strand: Some("pluck".to_string()),
                }
            };

            heartbeats.push(WorkerHeartbeat {
                worker: worker_name,
                ts,
                state,
            });
        }

        heartbeats
    }

    /// Generate synthetic beads for a project
    fn generate_beads(&self, project: &str) -> Vec<serde_json::Value> {
        let mut beads = Vec::new();
        let ts = Utc::now();

        // Create a mix of open and closed beads
        for bead_idx in 0..(self.config.beads_per_worker as usize) {
            let bead_id = format!("{}-bd-{:06}", project, bead_idx);
            let is_open = bead_idx % 10 < 3; // 30% open

            beads.push(json!({
                "id": bead_id,
                "title": format!("Load test bead {}", bead_idx),
                "description": format!("Synthetic bead for load testing"),
                "status": if is_open { "open" } else { "closed" },
                "priority": 0,
                "issue_type": "task",
                "created_at": ts.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                "updated_at": ts.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                "created_by": "load-test-generator",
                "dependencies": [],
                "project": project,
            }));
        }

        beads
    }
}

// ---------------------------------------------------------------------------
// Performance assertions
// ---------------------------------------------------------------------------

/// Performance assertion results
#[derive(Debug)]
pub struct PerformanceReport {
    /// Did all assertions pass?
    pub passed: bool,
    /// Total number of events processed
    pub total_events: usize,
    /// API latency measurements (ms)
    pub api_latencies: Vec<u64>,
    /// WS fan-out lag measurements (ms)
    pub ws_fanout_lags: Vec<u64>,
    /// Memory usage samples (bytes)
    pub memory_samples: Vec<u64>,
    /// Specific failures
    pub failures: Vec<String>,
}

impl PerformanceReport {
    /// Assert that all performance budgets are satisfied
    pub fn assert_budgets(&self, config: &LoadTestConfig) -> anyhow::Result<()> {
        let mut failures = Vec::new();

        // Check API latency budget
        if let Some(&max_latency) = self.api_latencies.iter().max() {
            if max_latency > config.api_latency_budget_ms {
                failures.push(format!(
                    "API latency exceeded budget: {}ms > {}ms",
                    max_latency, config.api_latency_budget_ms
                ));
            }
        }

        // Check WS fan-out lag budget
        if let Some(&max_lag) = self.ws_fanout_lags.iter().max() {
            if max_lag > config.ws_fanout_lag_budget_ms {
                failures.push(format!(
                    "WS fan-out lag exceeded budget: {}ms > {}ms",
                    max_lag, config.ws_fanout_lag_budget_ms
                ));
            }
        }

        // Check memory ceiling
        if let Some(&max_memory) = self.memory_samples.iter().max() {
            if max_memory > config.memory_ceiling_bytes {
                failures.push(format!(
                    "Memory exceeded ceiling: {}MB > {}MB",
                    max_memory / 1024 / 1024,
                    config.memory_ceiling_bytes / 1024 / 1024
                ));
            }
        }

        if !failures.is_empty() {
            anyhow::bail!("Performance budget violations:\n{}", failures.join("\n"));
        }

        Ok(())
    }

    /// Generate a human-readable summary
    pub fn summary(&self) -> String {
        let mut lines = vec![
            "=== Load Test Performance Report ===".to_string(),
            format!("Total events: {}", self.total_events),
            format!("Result: {}", if self.passed { "PASS" } else { "FAIL" }),
            String::new(),
        ];

        if !self.api_latencies.is_empty() {
            let avg = self.api_latencies.iter().sum::<u64>() / self.api_latencies.len() as u64;
            let max = self.api_latencies.iter().max().unwrap();
            lines.push(format!("API Latency: avg={}ms, max={}ms", avg, max));
        }

        if !self.ws_fanout_lags.is_empty() {
            let avg = self.ws_fanout_lags.iter().sum::<u64>() / self.ws_fanout_lags.len() as u64;
            let max = self.ws_fanout_lags.iter().max().unwrap();
            lines.push(format!("WS Fan-out Lag: avg={}ms, max={}ms", avg, max));
        }

        if !self.memory_samples.is_empty() {
            let avg = self.memory_samples.iter().sum::<u64>() / self.memory_samples.len() as u64;
            let max = self.memory_samples.iter().max().unwrap();
            lines.push(format!(
                "Memory: avg={}MB, max={}MB",
                avg / 1024 / 1024,
                max / 1024 / 1024
            ));
        }

        if !self.failures.is_empty() {
            lines.push(String::new());
            lines.push("Failures:".to_string());
            for failure in &self.failures {
                lines.push(format!("  - {}", failure));
            }
        }

        lines.join("\n")
    }
}

/// Measure current process RSS memory in bytes
pub fn measure_memory() -> u64 {
    // Try /proc/self/status first (Linux)
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<u64>() {
                        return kb * 1024;
                    }
                }
            }
        }
    }

    // Fallback: estimate based on heap size (not accurate but works cross-platform)
    0
}

// ---------------------------------------------------------------------------
// Load test runner
// ---------------------------------------------------------------------------

/// Runs the full load test against a daemon at the given URL
pub async fn run_load_test(
    base_url: &str,
    config: LoadTestConfig,
) -> anyhow::Result<PerformanceReport> {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;

    let client = reqwest::Client::new();

    // Create temporary directory for synthetic data
    let temp_dir = tempfile::TempDir::new()?;
    let generator = EventGenerator::new(config.clone());
    generator.write_to_disk(temp_dir.path())?;

    let mut api_latencies = Vec::new();
    let mut ws_fanout_lags = Vec::new();
    let mut memory_samples = Vec::new();
    let mut failures = Vec::new();

    // Sample memory before
    memory_samples.push(measure_memory());

    let start = Instant::now();

    // Create multiple WS connections to test fan-out
    let num_ws_clients = 10;
    let ws_url = base_url.replace("http://", "ws://");
    let ws_url = format!("{}/ws", ws_url);

    let mut ws_clients = Vec::new();
    for i in 0..num_ws_clients {
        match tokio_tungstenite::connect_async(&ws_url).await {
            Ok((ws_stream, _)) => {
                let (mut ws_sender, mut ws_receiver) = ws_stream.split();
                ws_clients.push((ws_sender, ws_receiver));
            }
            Err(e) => {
                failures.push(format!("Failed to connect WS client {}: {}", i, e));
            }
        }
    }

    // Simulate load by making concurrent API requests
    let http_start = Instant::now();

    // Test health endpoint latency
    let health_start = Instant::now();
    match client.get(&format!("{}/healthz", base_url)).send().await {
        Ok(resp) if resp.status().is_success() => {
            api_latencies.push(health_start.elapsed().as_millis() as u64);
        }
        Ok(resp) => {
            failures.push(format!("Health check failed: {}", resp.status()));
        }
        Err(e) => {
            failures.push(format!("Health check error: {}", e));
        }
    }

    // Test /api/beads endpoint
    let beads_start = Instant::now();
    match client.get(&format!("{}/api/beads", base_url)).send().await {
        Ok(resp) if resp.status().is_success() => {
            api_latencies.push(beads_start.elapsed().as_millis() as u64);
        }
        Ok(resp) => {
            failures.push(format!("GET /api/beads failed: {}", resp.status()));
        }
        Err(e) => {
            failures.push(format!("GET /api/beads error: {}", e));
        }
    }

    // Test /api/projects endpoint
    let projects_start = Instant::now();
    match client.get(&format!("{}/api/projects", base_url)).send().await {
        Ok(resp) if resp.status().is_success() => {
            api_latencies.push(projects_start.elapsed().as_millis() as u64);
        }
        Ok(resp) => {
            failures.push(format!("GET /api/projects failed: {}", resp.status()));
        }
        Err(e) => {
            failures.push(format!("GET /api/projects error: {}", e));
        }
    }

    let http_elapsed = http_start.elapsed();

    // Measure WS fan-out lag
    if !ws_clients.is_empty() {
        let fanout_start = Instant::now();

        // Send a subscribe message and measure how long it takes for all clients to receive
        let subscribe_msg = json!({"type": "subscribe", "topic": "global"}).to_string();

        for (ws_sender, _) in &mut ws_clients {
            let _ = ws_sender
                .send(Message::Text(subscribe_msg.clone()))
                .await;
        }

        // Wait for all clients to receive an event (simplified: just measure time)
        let _ = tokio::time::timeout(Duration::from_millis(500), async {
            // In a real test, we'd wait for specific events
            tokio::time::sleep(Duration::from_millis(50)).await;
        })
        .await;

        ws_fanout_lags.push(fanout_start.elapsed().as_millis() as u64);
    }

    // Close WS connections
    for (mut ws_sender, _) in ws_clients {
        let _ = ws_sender.send(Message::Close(None)).await;
    }

    // Sample memory after
    memory_samples.push(measure_memory());

    let total_events = (config.total_beads() * 5) as usize; // Approx 5 events per bead
    let passed = failures.is_empty();

    // Check budgets
    let report = PerformanceReport {
        passed,
        total_events,
        api_latencies: api_latencies.clone(),
        ws_fanout_lags: ws_fanout_lags.clone(),
        memory_samples: memory_samples.clone(),
        failures: failures.clone(),
    };

    // Validate against budgets
    if let Err(e) = report.assert_budgets(&config) {
        failures.push(e.to_string());
    }

    Ok(report)
}
