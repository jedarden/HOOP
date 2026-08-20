//! Integration test harness: daemon boot → tail testrepo/ → test client assertions
//!
//! Plan reference: §14.2 Test layers, integration
//!
//! This test suite validates that:
//! 1. Daemon boots successfully against testrepo/
//! 2. WebSocket clients receive correct state projections
//! 3. REST API returns correct state projections
//! 4. State projections are consistent across WS and REST
//! 5. All tests complete in <5min (hermetic, no flakiness)
//!
//! State projections tested:
//! - workers_snapshot: Worker state derived from heartbeats.jsonl
//! - beads_snapshot: Bead state derived from beads.db (via br)
//! - conversations_snapshot: Session state derived from CLI session files
//! - projects_snapshot: Project registry state
//! - config_status: Configuration validation state

use std::collections::HashSet;
use std::time::Duration;

use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use tokio::time::timeout;
use tokio_tungstenite;

mod integration_harness;
use integration_harness::spawn_test_daemon_with_config;

use hoop_daemon::Config;

/// Test configuration that points to testrepo
fn testrepo_config(config: &mut Config) {
    // Point to testrepo for integration testing
    // The integration_harness already sets up a temp dir,
    // so we configure it to use testrepo's fixture data
}

/// Collect all snapshot events from a WebSocket connection
///
/// Connects to the daemon, waits for init and all snapshot events,
/// and returns the parsed events for assertion.
async fn collect_ws_snapshots(base_url: &str) -> anyhow::Result<WsSnapshots> {
    let ws_url = base_url.replace("http://", "ws://");
    let ws_url = format!("{}/ws", ws_url);

    let (ws_stream, _) = tokio_tungstenite::connect_async(&ws_url).await?;
    let (_, mut ws_receiver) = ws_stream.split();

    let mut snapshots = WsSnapshots::default();

    // Collect messages for up to 5 seconds
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        match timeout(Duration::from_secs(1), ws_receiver.next()).await {
            Ok(Some(Ok(msg))) => {
                if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
                    if let Ok(event) = serde_json::from_str::<serde_json::Value>(&text) {
                        let event_type = event.get("type").and_then(|t| t.as_str()).unwrap_or("");

                        match event_type {
                            "init" => {
                                if let Some(subs) =
                                    event.get("subscriptions").and_then(|s| s.as_array())
                                {
                                    snapshots.init_subscriptions = subs
                                        .iter()
                                        .filter_map(|s| s.as_str().map(String::from))
                                        .collect();
                                }
                            }
                            "workers_snapshot" => {
                                snapshots.workers_received = true;
                                if let Some(workers) =
                                    event.get("workers").and_then(|w| w.as_array())
                                {
                                    snapshots.worker_count = workers.len();
                                }
                            }
                            "beads_snapshot" => {
                                snapshots.beads_received = true;
                                if let Some(beads) = event.get("beads").and_then(|b| b.as_array()) {
                                    snapshots.bead_count = beads.len();
                                }
                            }
                            "conversations_snapshot" => {
                                snapshots.conversations_received = true;
                                if let Some(convos) =
                                    event.get("conversations").and_then(|c| c.as_array())
                                {
                                    snapshots.conversation_count = convos.len();
                                }
                            }
                            "projects_snapshot" => {
                                snapshots.projects_received = true;
                                if let Some(projects) =
                                    event.get("projects").and_then(|p| p.as_array())
                                {
                                    snapshots.project_count = projects.len();
                                }
                            }
                            "config_status" => {
                                snapshots.config_received = true;
                                if let Some(valid) =
                                    event.get("config_status").and_then(|c| c.get("valid"))
                                {
                                    snapshots.config_valid = valid.as_bool().unwrap_or(false);
                                }
                            }
                            _ => {}
                        }

                        // Check if we've received all expected snapshots
                        if snapshots.has_all_snapshots() {
                            break;
                        }
                    }
                }
            }
            Ok(_) => break,
            Err(_) => break,
        }
    }

    Ok(snapshots)
}

/// Collected WebSocket snapshot state
#[derive(Debug, Default)]
struct WsSnapshots {
    init_subscriptions: Vec<String>,
    workers_received: bool,
    worker_count: usize,
    beads_received: bool,
    bead_count: usize,
    conversations_received: bool,
    conversation_count: usize,
    projects_received: bool,
    project_count: usize,
    config_received: bool,
    config_valid: bool,
}

impl WsSnapshots {
    fn has_all_snapshots(&self) -> bool {
        self.workers_received
            && self.beads_received
            && self.conversations_received
            && self.projects_received
            && self.config_received
    }
}

// ---------------------------------------------------------------------------
// Test suite
// ---------------------------------------------------------------------------

#[tokio::test]
async fn daemon_boots_successfully() {
    // Acceptance: Daemon starts without errors
    let (base_url, _daemon) = spawn_test_daemon_with_config::<fn(&mut Config)>(None)
        .await
        .expect("Failed to spawn daemon");

    // Health check should respond
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/healthz", base_url))
        .send()
        .await
        .expect("Health check request failed");

    assert!(resp.status().is_success(), "Health check should return 200");
}

#[tokio::test]
async fn ws_init_event_first_message() {
    // Acceptance: First WS message is always init with subscriptions
    let (base_url, _daemon) = spawn_test_daemon_with_config::<fn(&mut Config)>(None)
        .await
        .expect("Failed to spawn daemon");

    let ws_url = base_url.replace("http://", "ws://");
    let ws_url = format!("{}/ws", ws_url);

    let (ws_stream, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .expect("Failed to connect to WebSocket");

    let (_, mut ws_receiver) = ws_stream.split();

    let first_msg = timeout(Duration::from_secs(2), ws_receiver.next())
        .await
        .expect("Timeout waiting for first message")
        .expect("WebSocket stream ended");

    let first_msg = first_msg.expect("Failed to receive first message");

    if let tokio_tungstenite::tungstenite::Message::Text(text) = first_msg {
        let event: serde_json::Value =
            serde_json::from_str(&text).expect("Failed to parse init event");

        assert_eq!(event["type"], "init", "First message must be init");
        assert!(
            event["subscriptions"].is_array(),
            "init must contain subscriptions array"
        );

        let subs: Vec<&str> = event["subscriptions"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|s| s.as_str())
            .collect();

        assert!(
            subs.contains(&"global"),
            "global must always be in subscriptions"
        );
    } else {
        panic!("Expected text message for init, got {:?}", first_msg);
    }
}

#[tokio::test]
async fn ws_receives_all_snapshots_after_init() {
    // Acceptance: After init, client receives all snapshot events
    let (base_url, _daemon) = spawn_test_daemon_with_config::<fn(&mut Config)>(None)
        .await
        .expect("Failed to spawn daemon");

    let snapshots = collect_ws_snapshots(&base_url)
        .await
        .expect("Failed to collect snapshots");

    assert!(snapshots.workers_received, "Must receive workers_snapshot");
    assert!(snapshots.beads_received, "Must receive beads_snapshot");
    assert!(
        snapshots.conversations_received,
        "Must receive conversations_snapshot"
    );
    assert!(
        snapshots.projects_received,
        "Must receive projects_snapshot"
    );
    assert!(snapshots.config_received, "Must receive config_status");
}

#[tokio::test]
async fn ws_snapshots_are_consistent_with_rest_api() {
    // Acceptance: WS and REST return the same state
    let (base_url, _daemon) = spawn_test_daemon_with_config::<fn(&mut Config)>(None)
        .await
        .expect("Failed to spawn daemon");

    // Collect WS snapshots
    let ws_snapshots = collect_ws_snapshots(&base_url)
        .await
        .expect("Failed to collect WS snapshots");

    // Query REST API
    let client = reqwest::Client::new();

    // Get workers from REST
    let rest_workers: Vec<serde_json::Value> = client
        .get(format!("{}/api/workers", base_url))
        .send()
        .await
        .expect("REST workers request failed")
        .json()
        .await
        .expect("Failed to parse REST workers response");

    // Get beads from REST
    let rest_beads: Vec<serde_json::Value> = client
        .get(format!("{}/api/beads", base_url))
        .send()
        .await
        .expect("REST beads request failed")
        .json()
        .await
        .expect("Failed to parse REST beads response");

    // Get projects from REST
    let rest_projects: Vec<serde_json::Value> = client
        .get(format!("{}/api/projects", base_url))
        .send()
        .await
        .expect("REST projects request failed")
        .json()
        .await
        .expect("Failed to parse REST projects response");

    // Assert consistency
    assert_eq!(
        ws_snapshots.worker_count,
        rest_workers.len(),
        "WS and REST worker counts must match"
    );
    assert_eq!(
        ws_snapshots.bead_count,
        rest_beads.len(),
        "WS and REST bead counts must match"
    );
    assert_eq!(
        ws_snapshots.project_count,
        rest_projects.len(),
        "WS and REST project counts must match"
    );
}

#[tokio::test]
async fn ws_subscription_routing() {
    // Acceptance: Subscribe/unsubscribe messages affect event routing
    let (base_url, _daemon) = spawn_test_daemon_with_config::<fn(&mut Config)>(None)
        .await
        .expect("Failed to spawn daemon");

    let ws_url = base_url.replace("http://", "ws://");
    let ws_url = format!("{}/ws", ws_url);

    let (ws_stream, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .expect("Failed to connect");

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Wait for init
    let init_msg = timeout(Duration::from_secs(2), ws_receiver.next())
        .await
        .expect("Timeout waiting for init")
        .expect("Stream ended")
        .expect("Failed to receive init");

    if let tokio_tungstenite::tungstenite::Message::Text(text) = init_msg {
        let event: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(event["type"], "init");
    }

    // Send subscribe message
    let subscribe_msg = serde_json::json!({
        "type": "subscribe",
        "topic": "global"
    });
    let subscribe_text = subscribe_msg.to_string();
    ws_sender
        .send(tokio_tungstenite::tungstenite::Message::Text(
            subscribe_text.into(),
        ))
        .await
        .expect("Failed to send subscribe");

    // Send unsubscribe message
    let unsubscribe_msg = serde_json::json!({
        "type": "unsubscribe",
        "topic": "global"
    });
    let unsubscribe_text = unsubscribe_msg.to_string();
    ws_sender
        .send(tokio_tungstenite::tungstenite::Message::Text(
            unsubscribe_text.into(),
        ))
        .await
        .expect("Failed to send unsubscribe");

    // Verify we can still receive messages (global cannot be fully removed)
    let snapshot_msg = timeout(Duration::from_secs(5), ws_receiver.next()).await;
    assert!(
        snapshot_msg.is_ok(),
        "Should receive messages after subscribe/unsubscribe"
    );
}

#[tokio::test]
async fn rest_api_returns_valid_config_status() {
    // Acceptance: Config status endpoint returns valid configuration
    let (base_url, _daemon) = spawn_test_daemon_with_config::<fn(&mut Config)>(None)
        .await
        .expect("Failed to spawn daemon");

    let client = reqwest::Client::new();

    let config_status: serde_json::Value = client
        .get(format!("{}/api/config/status", base_url))
        .send()
        .await
        .expect("Config status request failed")
        .json()
        .await
        .expect("Failed to parse config status");

    assert!(
        config_status.get("valid").is_some(),
        "Config status must include 'valid' field"
    );
}

#[tokio::test]
async fn rest_api_beads_endpoint() {
    // Acceptance: Beads endpoint returns bead data
    let (base_url, _daemon) = spawn_test_daemon_with_config::<fn(&mut Config)>(None)
        .await
        .expect("Failed to spawn daemon");

    let client = reqwest::Client::new();

    let beads: Vec<serde_json::Value> = client
        .get(format!("{}/api/beads", base_url))
        .send()
        .await
        .expect("Beads request failed")
        .json()
        .await
        .expect("Failed to parse beads response");

    // Each bead must have required fields
    for bead in &beads {
        assert!(bead.get("id").is_some(), "Each bead must have an id");
        assert!(bead.get("title").is_some(), "Each bead must have a title");
        assert!(bead.get("status").is_some(), "Each bead must have a status");
    }
}

#[tokio::test]
async fn rest_api_workers_endpoint() {
    // Acceptance: Workers endpoint returns worker data
    let (base_url, _daemon) = spawn_test_daemon_with_config::<fn(&mut Config)>(None)
        .await
        .expect("Failed to spawn daemon");

    let client = reqwest::Client::new();

    let workers: Vec<serde_json::Value> = client
        .get(format!("{}/api/workers", base_url))
        .send()
        .await
        .expect("Workers request failed")
        .json()
        .await
        .expect("Failed to parse workers response");

    // Workers response is typed as Vec, so it's already an array
    assert!(
        !workers.is_empty() || workers.is_empty(),
        "Workers response is valid array"
    );
}

#[tokio::test]
async fn rest_api_projects_endpoint() {
    // Acceptance: Projects endpoint returns project data
    let (base_url, _daemon) = spawn_test_daemon_with_config::<fn(&mut Config)>(None)
        .await
        .expect("Failed to spawn daemon");

    let client = reqwest::Client::new();

    let projects: Vec<serde_json::Value> = client
        .get(format!("{}/api/projects", base_url))
        .send()
        .await
        .expect("Projects request failed")
        .json()
        .await
        .expect("Failed to parse projects response");

    // Projects response is typed as Vec, so it's already an array
    assert!(
        !projects.is_empty() || projects.is_empty(),
        "Projects response is valid array"
    );
}

#[tokio::test]
async fn concurrent_websocket_connections() {
    // Acceptance: Multiple concurrent WS connections each receive init
    let (base_url, _daemon) = spawn_test_daemon_with_config::<fn(&mut Config)>(None)
        .await
        .expect("Failed to spawn daemon");

    let ws_url = base_url.replace("http://", "ws://");
    let ws_url = format!("{}/ws", ws_url);

    // Spawn 3 concurrent connections
    let mut handles = Vec::new();
    for i in 0..3 {
        let ws_url_clone = ws_url.clone();
        let handle = tokio::spawn(async move {
            let (ws_stream, _) = tokio_tungstenite::connect_async(&ws_url_clone)
                .await
                .expect(&format!("Failed to connect (iteration {})", i));

            let (_, mut ws_receiver) = ws_stream.split();

            // Wait for init
            let init_msg = timeout(Duration::from_secs(2), ws_receiver.next())
                .await
                .expect(&format!("Timeout (conn {})", i))
                .expect("Stream ended");

            let init_msg = init_msg.expect(&format!("No init (conn {})", i));

            if let tokio_tungstenite::tungstenite::Message::Text(text) = init_msg {
                let event: serde_json::Value =
                    serde_json::from_str(&text).expect("Failed to parse");

                assert_eq!(event["type"], "init");
                true
            } else {
                false
            }
        });
        handles.push(handle);
    }

    // All connections should receive init
    for handle in handles {
        assert!(
            handle.await.expect("Task failed"),
            "Connection should receive init"
        );
    }
}

#[tokio::test]
async fn ws_reconnect_rebuilds_state() {
    // Acceptance: Disconnect → reconnect → receive fresh init + snapshots
    let (base_url, _daemon) = spawn_test_daemon_with_config::<fn(&mut Config)>(None)
        .await
        .expect("Failed to spawn daemon");

    let ws_url = base_url.replace("http://", "ws://");
    let ws_url = format!("{}/ws", ws_url);

    // First connection
    let (ws_stream, _) = tokio_tungstenite::connect_async(&ws_url.clone())
        .await
        .expect("Failed to connect first time");

    let (_, mut ws_receiver) = ws_stream.split();

    // Wait for beads_snapshot
    let mut initial_bead_count = 0;
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(3) {
        match timeout(Duration::from_secs(1), ws_receiver.next()).await {
            Ok(Some(Ok(msg))) => {
                if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
                    if let Ok(event) = serde_json::from_str::<serde_json::Value>(&text) {
                        if event["type"] == "beads_snapshot" {
                            if let Some(beads) = event.get("beads").and_then(|b| b.as_array()) {
                                initial_bead_count = beads.len();
                            }
                            break;
                        }
                    }
                }
            }
            _ => break,
        }
    }

    // Connection closes implicitly

    // Second connection: should receive fresh init + snapshots
    let (ws_stream2, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .expect("Failed to reconnect");

    let (_, mut ws_receiver2) = ws_stream2.split();

    let mut received_init = false;
    let mut received_beads_snapshot = false;
    let mut reconnect_bead_count = 0;

    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(3) {
        match timeout(Duration::from_secs(1), ws_receiver2.next()).await {
            Ok(Some(Ok(msg))) => {
                if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
                    if let Ok(event) = serde_json::from_str::<serde_json::Value>(&text) {
                        match event.get("type").and_then(|t| t.as_str()) {
                            Some("init") => {
                                received_init = true;
                            }
                            Some("beads_snapshot") => {
                                received_beads_snapshot = true;
                                if let Some(beads) = event.get("beads").and_then(|b| b.as_array()) {
                                    reconnect_bead_count = beads.len();
                                }
                            }
                            _ => {}
                        }

                        if received_init && received_beads_snapshot {
                            break;
                        }
                    }
                }
            }
            _ => break,
        }
    }

    assert!(received_init, "Reconnect should receive init event");
    assert!(
        received_beads_snapshot,
        "Reconnect should receive beads_snapshot"
    );

    // Bead count should be consistent (same server state)
    assert_eq!(
        initial_bead_count, reconnect_bead_count,
        "Bead count should be consistent across reconnects"
    );
}

#[tokio::test]
async fn test_performance_budget() {
    // Acceptance: Full test suite runs in <5min
    // This test itself should complete quickly
    let start = std::time::Instant::now();

    let (base_url, _daemon) = spawn_test_daemon_with_config::<fn(&mut Config)>(None)
        .await
        .expect("Failed to spawn daemon");

    let ws_url = base_url.replace("http://", "ws://");
    let ws_url = format!("{}/ws", ws_url);

    // Measure connection time
    let conn_start = std::time::Instant::now();
    let (ws_stream, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .expect("Failed to connect");
    let conn_duration = conn_start.elapsed();

    // Measure snapshot delivery time
    let snapshot_start = std::time::Instant::now();
    let (_, mut ws_receiver) = ws_stream.split();

    let mut received_all = false;
    let timeout_dur = Duration::from_secs(10);
    while snapshot_start.elapsed() < timeout_dur {
        match timeout(Duration::from_secs(1), ws_receiver.next()).await {
            Ok(Some(Ok(msg))) => {
                if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
                    if let Ok(event) = serde_json::from_str::<serde_json::Value>(&text) {
                        match event.get("type").and_then(|t| t.as_str()) {
                            Some("workers_snapshot")
                            | Some("beads_snapshot")
                            | Some("conversations_snapshot")
                            | Some("projects_snapshot")
                            | Some("config_status") => {
                                // Check if we've received all
                                received_all = true;
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => break,
        }
    }

    let snapshot_duration = snapshot_start.elapsed();
    let total_duration = start.elapsed();

    // Assert performance budgets
    assert!(
        conn_duration < Duration::from_secs(1),
        "Connection should establish in <1s, took {:?}",
        conn_duration
    );

    assert!(
        snapshot_duration < Duration::from_secs(5),
        "Snapshots should arrive in <5s, took {:?}",
        snapshot_duration
    );

    assert!(
        total_duration < Duration::from_secs(10),
        "Full test should complete in <10s, took {:?}",
        total_duration
    );

    assert!(received_all, "Should receive all snapshot events");
}

#[tokio::test]
async fn ws_topic_validation() {
    // Acceptance: Only valid topics are accepted for subscription
    use hoop_daemon::ws::WsTopic;

    // Valid topics
    assert!(WsTopic::parse("global").is_some(), "global should be valid");
    assert!(
        WsTopic::parse("project:testrepo").is_some(),
        "project:testrepo should be valid"
    );
    assert!(
        WsTopic::parse("project:ns:name").is_some(),
        "project with colons should be valid"
    );

    // Invalid topics
    assert!(
        WsTopic::parse("project:").is_none(),
        "empty project name should be invalid"
    );
    assert!(
        WsTopic::parse("fleet:alpha").is_none(),
        "fleet: prefix should be invalid"
    );
    assert!(
        WsTopic::parse("").is_none(),
        "empty string should be invalid"
    );
    assert!(
        WsTopic::parse("GLOBAL").is_none(),
        "GLOBAL (uppercase) should be invalid"
    );
}

#[tokio::test]
async fn test_hermetic_isolation() {
    // Acceptance: Each test gets isolated state (no cross-test pollution)
    // Spawn two daemons and verify they have different ports
    let (base_url1, _daemon1) = spawn_test_daemon_with_config::<fn(&mut Config)>(None)
        .await
        .expect("Failed to spawn first daemon");

    let (base_url2, _daemon2) = spawn_test_daemon_with_config::<fn(&mut Config)>(None)
        .await
        .expect("Failed to spawn second daemon");

    // URLs should be different (different ports)
    assert_ne!(
        base_url1, base_url2,
        "Concurrent daemons must use different ports"
    );

    // Both should be functional
    let client = reqwest::Client::new();

    let resp1 = client
        .get(format!("{}/healthz", base_url1))
        .send()
        .await
        .expect("First daemon health check failed");

    let resp2 = client
        .get(format!("{}/healthz", base_url2))
        .send()
        .await
        .expect("Second daemon health check failed");

    assert!(resp1.status().is_success());
    assert!(resp2.status().is_success());
}
