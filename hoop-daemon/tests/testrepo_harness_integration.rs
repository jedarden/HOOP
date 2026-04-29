//! Integration test harness: daemon boot → tail testrepo/ → test client assertions
//!
//! Acceptance criteria from hoop-ttb.11.3:
//! - Daemon boots successfully against testrepo/
//! - WebSocket clients receive correct state projections
//! - REST API returns correct state projections
//! - State projections are consistent across WS and REST
//! - Tests are hermetic (no flakiness)
//! - Full suite runs in <5min
//!
//! Plan reference: §14.2 Test layers, integration
//!
//! State projections tested:
//! - workers_snapshot: Worker state derived from heartbeats.jsonl
//! - beads_snapshot: Bead state derived from beads.db (via br)
//! - conversations_snapshot: Session state derived from CLI session files
//! - projects_snapshot: Project registry state
//! - config_status: Configuration validation state

use std::time::Duration;
use tokio::time::timeout;

use hoop_daemon::integration_harness::spawn_test_daemon;

// ---------------------------------------------------------------------------
// Test client for driving REST + WebSocket interactions
// ---------------------------------------------------------------------------

/// Test client that combines REST and WebSocket interactions
struct TestClient {
    base_url: String,
    http_client: reqwest::Client,
}

impl TestClient {
    /// Create a new test client connected to the daemon
    async fn new(base_url: String) -> anyhow::Result<Self> {
        let http_client = reqwest::Client::new();
        let start = std::time::Instant::now();

        while start.elapsed() < Duration::from_secs(10) {
            if let Ok(resp) = http_client
                .get(&format!("{}/healthz", &base_url))
                .timeout(Duration::from_millis(200))
                .send()
                .await
            {
                if resp.status().is_success() {
                    return Ok(Self { base_url, http_client });
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        anyhow::bail!("Daemon did not become ready");
    }

    /// Connect to WebSocket and collect all snapshot events
    async fn collect_ws_snapshots(&self) -> anyhow::Result<WsSnapshots> {
        let ws_url = self.base_url.replace("http://", "ws://");
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
                                    if let Some(subs) = event.get("subscriptions").and_then(|s| s.as_array()) {
                                        snapshots.init_subscriptions = subs
                                            .iter()
                                            .filter_map(|s| s.as_str().map(String::from))
                                            .collect();
                                    }
                                }
                                "workers_snapshot" => {
                                    snapshots.workers_received = true;
                                    snapshots.workers_data = event.get("workers").cloned();
                                    if let Some(workers) = event.get("workers").and_then(|w| w.as_array()) {
                                        snapshots.worker_count = workers.len();
                                    }
                                }
                                "beads_snapshot" => {
                                    snapshots.beads_received = true;
                                    snapshots.beads_data = event.get("beads").cloned();
                                    if let Some(beads) = event.get("beads").and_then(|b| b.as_array()) {
                                        snapshots.bead_count = beads.len();
                                    }
                                }
                                "conversations_snapshot" => {
                                    snapshots.conversations_received = true;
                                    if let Some(convos) = event.get("conversations").and_then(|c| c.as_array()) {
                                        snapshots.conversation_count = convos.len();
                                    }
                                }
                                "projects_snapshot" => {
                                    snapshots.projects_received = true;
                                    snapshots.projects_data = event.get("projects").cloned();
                                    if let Some(projects) = event.get("projects").and_then(|p| p.as_array()) {
                                        snapshots.project_count = projects.len();
                                    }
                                }
                                "config_status" => {
                                    snapshots.config_received = true;
                                    snapshots.config_data = event.get("config_status").cloned();
                                    if let Some(valid) = event.get("config_status").and_then(|c| c.get("valid")) {
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

    /// GET /healthz
    async fn healthz(&self) -> anyhow::Result<serde_json::Value> {
        let resp = self.http_client.get(&format!("{}/healthz", self.base_url)).send().await?;
        Ok(resp.json().await?)
    }

    /// GET /readyz
    async fn readyz(&self) -> anyhow::Result<serde_json::Value> {
        let resp = self.http_client.get(&format!("{}/readyz", self.base_url)).send().await?;
        Ok(resp.json().await?)
    }

    /// GET /api/beads
    async fn get_beads(&self) -> anyhow::Result<Vec<serde_json::Value>> {
        let resp = self.http_client.get(&format!("{}/api/beads", self.base_url)).send().await?;
        Ok(resp.json().await?)
    }

    /// GET /api/workers/timeline
    async fn get_workers_timeline(&self) -> anyhow::Result<Vec<serde_json::Value>> {
        let resp = self
            .http_client
            .get(&format!("{}/api/workers/timeline", self.base_url))
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    /// GET /api/conversations
    async fn get_conversations(&self) -> anyhow::Result<Vec<serde_json::Value>> {
        let resp = self
            .http_client
            .get(&format!("{}/api/conversations", self.base_url))
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    /// GET /api/projects
    async fn get_projects(&self) -> anyhow::Result<Vec<serde_json::Value>> {
        let resp = self
            .http_client
            .get(&format!("{}/api/projects", self.base_url))
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    /// GET /api/config/status
    async fn get_config_status(&self) -> anyhow::Result<serde_json::Value> {
        let resp = self
            .http_client
            .get(&format!("{}/api/config/status", self.base_url))
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    /// GET /api/capacity
    async fn get_capacity(&self) -> anyhow::Result<serde_json::Value> {
        let resp = self
            .http_client
            .get(&format!("{}/api/capacity", self.base_url))
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    /// GET /metrics
    async fn get_metrics(&self) -> anyhow::Result<String> {
        let resp = self
            .http_client
            .get(&format!("{}/metrics", self.base_url))
            .send()
            .await?;
        Ok(resp.text().await?)
    }
}

/// Collected WebSocket snapshot state
#[derive(Debug, Default)]
struct WsSnapshots {
    init_subscriptions: Vec<String>,
    workers_received: bool,
    worker_count: usize,
    workers_data: Option<serde_json::Value>,
    beads_received: bool,
    bead_count: usize,
    beads_data: Option<serde_json::Value>,
    conversations_received: bool,
    conversation_count: usize,
    projects_received: bool,
    project_count: usize,
    projects_data: Option<serde_json::Value>,
    config_received: bool,
    config_valid: bool,
    config_data: Option<serde_json::Value>,
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
    let (base_url, _daemon) = spawn_test_daemon()
        .await
        .expect("Failed to spawn daemon");

    let client = TestClient::new(base_url).await.expect("Failed to create test client");

    // Health check should respond
    let health = client.healthz().await.expect("Health check failed");
    assert_eq!(health["status"], "ok", "Health check should return ok");

    // Ready check should respond
    let ready = client.readyz().await.expect("Ready check failed");
    assert_eq!(ready["status"], "ok", "Ready check should return ok");
}

#[tokio::test]
async fn ws_init_event_is_first_message() {
    // Acceptance: First WS message is always init with subscriptions
    let (base_url, _daemon) = spawn_test_daemon()
        .await
        .expect("Failed to spawn daemon");

    let client = TestClient::new(base_url).await.expect("Failed to create test client");

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
        let event: serde_json::Value = serde_json::from_str(&text)
            .expect("Failed to parse init event");

        assert_eq!(event["type"], "init", "First message must be init");
        assert!(
            event["subscriptions"].is_array(),
            "init must contain subscriptions array"
        );

        // Global subscription should always be present
        let subs: Vec<&str> = event["subscriptions"]
            .as_array()
            .expect("subscriptions should be array")
            .iter()
            .filter_map(|s| s.as_str())
            .collect();

        assert!(
            subs.contains(&"global"),
            "Global subscription should be present"
        );
    } else {
        panic!("First message must be text, got {:?}", first_msg);
    }
}

#[tokio::test]
async fn ws_receives_all_snapshot_events() {
    // Acceptance: WS client receives all expected snapshot events
    let (base_url, _daemon) = spawn_test_daemon()
        .await
        .expect("Failed to spawn daemon");

    let client = TestClient::new(base_url).await.expect("Failed to create test client");

    let snapshots = client.collect_ws_snapshots().await.expect("Failed to collect snapshots");

    // Verify all snapshots were received
    assert!(
        snapshots.workers_received,
        "workers_snapshot should be received"
    );
    assert!(
        snapshots.beads_received,
        "beads_snapshot should be received"
    );
    assert!(
        snapshots.conversations_received,
        "conversations_snapshot should be received"
    );
    assert!(
        snapshots.projects_received,
        "projects_snapshot should be received"
    );
    assert!(
        snapshots.config_received,
        "config_status should be received"
    );
}

#[tokio::test]
async fn rest_api_endpoints_return_valid_state() {
    // Acceptance: REST API returns correct state projections
    let (base_url, _daemon) = spawn_test_daemon()
        .await
        .expect("Failed to spawn daemon");

    let client = TestClient::new(base_url).await.expect("Failed to create test client");

    // Test beads endpoint
    let beads = client.get_beads().await.expect("Failed to fetch beads");
    assert!(beads.is_array(), "Beads response should be an array");

    // Test workers timeline endpoint
    let workers = client.get_workers_timeline().await.expect("Failed to fetch workers");
    assert!(workers.is_array(), "Workers response should be an array");

    // Test conversations endpoint
    let conversations = client.get_conversations().await.expect("Failed to fetch conversations");
    assert!(conversations.is_array(), "Conversations response should be an array");

    // Test projects endpoint
    let projects = client.get_projects().await.expect("Failed to fetch projects");
    assert!(projects.is_array(), "Projects response should be an array");

    // Test config status endpoint
    let config = client.get_config_status().await.expect("Failed to fetch config status");
    assert!(config.get("valid").is_some(), "Config status must include 'valid' field");

    // Test capacity endpoint
    let capacity = client.get_capacity().await.expect("Failed to fetch capacity");
    assert!(capacity.is_object() || capacity.is_array(), "Capacity should be object or array");
}

#[tokio::test]
async fn metrics_endpoint_exposes_expected_metrics() {
    // Acceptance: /metrics returns Prometheus-style metrics
    let (base_url, _daemon) = spawn_test_daemon()
        .await
        .expect("Failed to spawn daemon");

    let client = TestClient::new(base_url).await.expect("Failed to create test client");

    let metrics = client.get_metrics().await.expect("Failed to fetch metrics");

    // Verify metrics contain expected prefixes
    assert!(
        metrics.contains("hoop_") || metrics.lines().count() > 0,
        "Metrics should contain hoop_ prefixed metrics or be non-empty"
    );

    // Verify each line is valid Prometheus format
    for (i, line) in metrics.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue; // Skip empty lines and comments
        }

        // Each metric line should have: metric_name{labels} value
        // or: metric_name value
        assert!(
            trimmed.contains(' ') || trimmed.contains('\t'),
            "Metric line {} should have whitespace separator: {}",
            i + 1,
            trimmed
        );
    }
}

#[tokio::test]
async fn ws_subscribe_unsubscribe_works() {
    // Acceptance: Subscribe/unsubscribe messages are processed
    let (base_url, _daemon) = spawn_test_daemon()
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
    ws_sender.send(tokio_tungstenite::tungstenite::Message::Text(subscribe_msg.to_string()))
        .await
        .expect("Failed to send subscribe");

    // Send unsubscribe message
    let unsubscribe_msg = serde_json::json!({
        "type": "unsubscribe",
        "topic": "global"
    });
    ws_sender.send(tokio_tungstenite::tungstenite::Message::Text(unsubscribe_msg.to_string()))
        .await
        .expect("Failed to send unsubscribe");

    // Verify we can still receive messages (global cannot be fully removed)
    let snapshot_msg = timeout(Duration::from_secs(5), ws_receiver.next())
        .await;
    assert!(snapshot_msg.is_ok(), "Should receive messages after subscribe/unsubscribe");
}

#[tokio::test]
async fn concurrent_websocket_connections() {
    // Acceptance: Multiple concurrent WS connections each receive init
    let (base_url, _daemon) = spawn_test_daemon()
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
                let event: serde_json::Value = serde_json::from_str(&text)
                    .expect("Failed to parse");

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
        assert!(handle.await.expect("Task failed"), "Connection should receive init");
    }
}

#[tokio::test]
async fn ws_reconnect_rebuilds_state() {
    // Acceptance: Disconnect → reconnect → receive fresh init + snapshots
    let (base_url, _daemon) = spawn_test_daemon()
        .await
        .expect("Failed to spawn daemon");

    let ws_url = base_url.replace("http://", "ws://");
    let ws_url = format!("{}/ws", ws_url);

    // First connection
    {
        let (ws_stream, _) = tokio_tungstenite::connect_async(&ws_url)
            .await
            .expect("Failed to connect first time");

        let (_, mut ws_receiver) = ws_stream.split();

        // Should receive init
        let init_msg = timeout(Duration::from_secs(2), ws_receiver.next())
            .await
            .expect("Timeout on first connection")
            .expect("Stream ended")
            .expect("No init on first connection");

        if let tokio_tungstenite::tungstenite::Message::Text(text) = init_msg {
            let event: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert_eq!(event["type"], "init");
        }
    } // Drop first connection

    // Reconnect
    let (ws_stream, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .expect("Failed to reconnect");

    let (_, mut ws_receiver) = ws_stream.split();

    // Should receive init again
    let init_msg = timeout(Duration::from_secs(2), ws_receiver.next())
        .await
        .expect("Timeout on reconnection")
        .expect("Stream ended")
        .expect("No init on reconnection");

    if let tokio_tungstenite::tungstenite::Message::Text(text) = init_msg {
        let event: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(event["type"], "init");
    }

    // Should receive snapshots again
    let snapshots_msg = timeout(Duration::from_secs(5), ws_receiver.next())
        .await
        .expect("Timeout waiting for snapshots after reconnect")
        .expect("Stream ended")
        .expect("No snapshots after reconnect");

    if let tokio_tungstenite::tungstenite::Message::Text(text) = snapshots_msg {
        let event: serde_json::Value = serde_json::from_str(&text).unwrap();
        // Should be one of the snapshot events
        let event_type = event["type"].as_str().unwrap_or("");
        assert!(
            event_type.ends_with("_snapshot") || event_type == "config_status",
            "After reconnect, should receive snapshot events, got {}",
            event_type
        );
    }
}

#[tokio::test]
async fn test_state_projections_contain_required_fields() {
    // Acceptance: State projections contain all required fields
    let (base_url, _daemon) = spawn_test_daemon()
        .await
        .expect("Failed to spawn daemon");

    let client = TestClient::new(base_url).await.expect("Failed to create test client");

    // Verify beads projection
    let beads = client.get_beads().await.expect("Failed to fetch beads");
    for bead in &beads {
        assert!(
            bead.get("id").is_some(),
            "Each bead must have an 'id' field"
        );
        assert!(
            bead.get("title").is_some(),
            "Each bead must have a 'title' field"
        );
        assert!(
            bead.get("status").is_some(),
            "Each bead must have a 'status' field"
        );
    }

    // Verify workers projection
    let workers = client.get_workers_timeline().await.expect("Failed to fetch workers");
    for worker in &workers {
        assert!(
            worker.get("name").is_some(),
            "Each worker must have a 'name' field"
        );
        // Worker state may be optional in some cases
    }

    // Verify projects projection
    let projects = client.get_projects().await.expect("Failed to fetch projects");
    for project in &projects {
        assert!(
            project.get("name").is_some(),
            "Each project must have a 'name' field"
        );
        assert!(
            project.get("path").is_some(),
            "Each project must have a 'path' field"
        );
    }
}
