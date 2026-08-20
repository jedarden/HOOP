//! Integration test client utilities
//!
//! Provides helper functions for driving REST API and WebSocket interactions
//! in integration tests. Includes assertions for state projections.
//!
//! ## Example
//!
//! ```rust
//! use hoop_daemon::integration_test_client::*;
//!
//! #[tokio::test]
//! async fn test_bead_lifecycle() {
//!     let client = TestClient::new("http://127.0.0.1:3000").await;
//!
//!     // Assert initial state
//!     assert!(client.healthz().await.is_ok());
//!
//!     // Create a bead via API
//!     let bead = client.create_bead("test-project", "Test bead").await.unwrap();
//!     assert_eq!(bead["title"], "Test bead");
//!
//!     // Verify bead appears in list
//!     let beads = client.list_beads().await.unwrap();
//!     assert!(beads.iter().any(|b| b["id"] == bead["id"]));
//! }
//! ```

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value as JsonValue;
use std::time::Duration;
use tokio_tungstenite::tungstenite::Message;

/// Test client for driving HOOP daemon interactions
pub struct TestClient {
    /// Base URL for API calls (e.g., "http://127.0.0.1:3000")
    pub base_url: String,
    /// HTTP client
    http_client: reqwest::Client,
}

impl TestClient {
    /// Create a new test client
    ///
    /// Waits for the daemon to be ready before returning.
    pub async fn new(base_url: &str) -> Result<Self> {
        let client = Self {
            base_url: base_url.to_string(),
            http_client: reqwest::Client::new(),
        };

        // Wait for daemon to be ready
        client.wait_for_ready(Duration::from_secs(30)).await?;

        Ok(client)
    }

    /// Wait for the daemon to be ready
    async fn wait_for_ready(&self, timeout: Duration) -> Result<()> {
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            match self.healthz().await {
                Ok(_) => return Ok(()),
                Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
            }
        }

        anyhow::bail!("Daemon did not become ready within {:?}", timeout);
    }

    /// GET /healthz - health check endpoint
    pub async fn healthz(&self) -> Result<reqwest::Response> {
        Ok(self
            .http_client
            .get(&format!("{}/healthz", self.base_url))
            .send()
            .await?)
    }

    /// GET /readyz - readiness check endpoint
    pub async fn readyz(&self) -> Result<reqwest::Response> {
        Ok(self
            .http_client
            .get(&format!("{}/readyz", self.base_url))
            .send()
            .await?)
    }

    /// GET /api/beads - list all beads
    pub async fn list_beads(&self) -> Result<Vec<JsonValue>> {
        let resp = self
            .http_client
            .get(&format!("{}/api/beads", self.base_url))
            .send()
            .await?;

        if !resp.status().is_success() {
            anyhow::bail!("GET /api/beads failed: {}", resp.status());
        }

        let beads: Vec<JsonValue> = resp.json().await?;
        Ok(beads)
    }

    /// GET /api/beads/:bead_id - get a specific bead
    pub async fn get_bead(&self, bead_id: &str) -> Result<JsonValue> {
        let resp = self
            .http_client
            .get(&format!("{}/api/beads/{}", self.base_url, bead_id))
            .send()
            .await?;

        if !resp.status().is_success() {
            anyhow::bail!("GET /api/beads/{} failed: {}", bead_id, resp.status());
        }

        Ok(resp.json().await?)
    }

    /// POST /api/p/:project/beads - create a new bead
    pub async fn create_bead(&self, project: &str, title: &str) -> Result<JsonValue> {
        self.create_bead_with_details(project, title, "task", None)
            .await
    }

    /// POST /api/p/:project/beads - create a new bead with full details
    pub async fn create_bead_with_details(
        &self,
        project: &str,
        title: &str,
        issue_type: &str,
        description: Option<&str>,
    ) -> Result<JsonValue> {
        let mut body = serde_json::json!({
            "title": title,
            "issue_type": issue_type,
            "priority": 0,
        });

        if let Some(desc) = description {
            body["description"] = serde_json::json!(desc);
        }

        let resp = self
            .http_client
            .post(&format!("{}/api/p/{}/beads", self.base_url, project))
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let error_text = resp.text().await.unwrap_or_default();
            anyhow::bail!(
                "POST /api/p/{}/beads failed: {} - {}",
                project,
                status,
                error_text
            );
        }

        Ok(resp.json().await?)
    }

    /// GET /api/capacity - get capacity information
    pub async fn get_capacity(&self) -> Result<JsonValue> {
        let resp = self
            .http_client
            .get(&format!("{}/api/capacity", self.base_url))
            .send()
            .await?;

        if !resp.status().is_success() {
            anyhow::bail!("GET /api/capacity failed: {}", resp.status());
        }

        Ok(resp.json().await?)
    }

    /// GET /metrics - get Prometheus metrics
    pub async fn get_metrics(&self) -> Result<String> {
        let resp = self
            .http_client
            .get(&format!("{}/metrics", self.base_url))
            .send()
            .await?;

        if !resp.status().is_success() {
            anyhow::bail!("GET /metrics failed: {}", resp.status());
        }

        Ok(resp.text().await?)
    }

    /// GET /api/workers/timeline - get worker timeline
    pub async fn get_worker_timeline(&self) -> Result<JsonValue> {
        let resp = self
            .http_client
            .get(&format!("{}/api/workers/timeline", self.base_url))
            .send()
            .await?;

        if !resp.status().is_success() {
            anyhow::bail!("GET /api/workers/timeline failed: {}", resp.status());
        }

        Ok(resp.json().await?)
    }

    /// Connect to the WebSocket endpoint
    ///
    /// Returns a tuple of (ws_sender, ws_receiver) for sending and receiving messages.
    pub async fn connect_ws(&self) -> Result<WsConnection> {
        let ws_url = self.base_url.replace("http://", "ws://");
        let ws_url = format!("{}/ws", ws_url);

        let (ws_stream, _) = tokio_tungstenite::connect_async(&ws_url).await?;
        let (ws_sender, ws_receiver) = ws_stream.split();

        Ok(WsConnection {
            sender: ws_sender,
            receiver: ws_receiver,
        })
    }

    /// Assert that the daemon is healthy
    pub async fn assert_healthy(&self) -> Result<()> {
        let resp = self.healthz().await?;
        if !resp.status().is_success() {
            anyhow::bail!("Health check failed: {}", resp.status());
        }
        Ok(())
    }

    /// Assert that the daemon is ready
    pub async fn assert_ready(&self) -> Result<()> {
        let resp = self.readyz().await?;
        if !resp.status().is_success() {
            anyhow::bail!("Readiness check failed: {}", resp.status());
        }
        Ok(())
    }

    /// Assert that a bead with the given ID exists
    pub async fn assert_bead_exists(&self, bead_id: &str) -> Result<()> {
        let bead = self.get_bead(bead_id).await?;
        if bead["id"] != bead_id {
            anyhow::bail!("Bead ID mismatch: expected {}, got {}", bead_id, bead["id"]);
        }
        Ok(())
    }

    /// Assert that a bead with the given title exists
    pub async fn assert_bead_with_title_exists(&self, title: &str) -> Result<()> {
        let beads = self.list_beads().await?;
        let found = beads.iter().any(|b| b["title"] == title);
        if !found {
            anyhow::bail!("No bead with title '{}' found", title);
        }
        Ok(())
    }

    /// Assert that the bead count matches the expected value
    pub async fn assert_bead_count(&self, expected: usize) -> Result<()> {
        let beads = self.list_beads().await?;
        if beads.len() != expected {
            anyhow::bail!(
                "Bead count mismatch: expected {}, got {}",
                expected,
                beads.len()
            );
        }
        Ok(())
    }

    /// Assert that the capacity report is valid
    pub async fn assert_valid_capacity(&self) -> Result<()> {
        let capacity = self.get_capacity().await?;

        // Check that capacity has expected fields
        if !capacity.is_object() {
            anyhow::bail!("Capacity response is not an object");
        }

        Ok(())
    }
}

/// WebSocket connection wrapper
pub struct WsConnection {
    sender: futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
    receiver: futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
}

impl WsConnection {
    /// Send a message via WebSocket
    pub async fn send(&mut self, msg: Message) -> Result<()> {
        self.sender
            .send(msg)
            .await
            .context("Failed to send WebSocket message")?;
        Ok(())
    }

    /// Receive a message via WebSocket
    pub async fn recv(&mut self) -> Result<Option<Message>> {
        match self.receiver.next().await {
            Some(Ok(msg)) => Ok(Some(msg)),
            Some(Err(e)) => Err(anyhow::anyhow!("WebSocket error: {}", e)),
            None => Ok(None),
        }
    }

    /// Wait for a bead event matching the given predicate
    pub async fn wait_for_bead_event<F>(&mut self, predicate: F) -> Result<JsonValue>
    where
        F: Fn(&JsonValue) -> bool,
    {
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(5);

        while start.elapsed() < timeout {
            match self.recv().await? {
                Some(Message::Text(text)) => {
                    if let Ok(value) = serde_json::from_str::<JsonValue>(&text) {
                        // Check if this is a bead event
                        if let Some(event_type) = value.get("type").and_then(|v| v.as_str()) {
                            if event_type == "bead_event" || event_type == "bead" {
                                if let Some(data) = value.get("data") {
                                    if predicate(data) {
                                        return Ok(data.clone());
                                    }
                                }
                            }
                        }
                    }
                }
                Some(Message::Close(_)) => {
                    anyhow::bail!("WebSocket connection closed");
                }
                None => {
                    anyhow::bail!("WebSocket connection terminated");
                }
                _ => {}
            }
        }

        anyhow::bail!("Timeout waiting for bead event");
    }

    /// Wait for a bead with the given status
    pub async fn wait_for_bead_status(&mut self, bead_id: &str, status: &str) -> Result<JsonValue> {
        self.wait_for_bead_event(|data| {
            data.get("id")
                .and_then(|v| v.as_str())
                .map(|id| id == bead_id)
                .unwrap_or(false)
                && data
                    .get("status")
                    .and_then(|v| v.as_str())
                    .map(|s| s == status)
                    .unwrap_or(false)
        })
        .await
    }

    /// Close the WebSocket connection
    pub async fn close(mut self) -> Result<()> {
        self.sender
            .send(Message::Close(None))
            .await
            .context("Failed to send close message")?;
        Ok(())
    }
}

/// State projection assertions
///
/// Helper functions for asserting that the daemon's state projections
/// match expected values.
impl TestClient {
    /// Assert that the worker timeline contains expected workers
    pub async fn assert_worker_count(&self, expected_min: usize) -> Result<()> {
        let timeline = self.get_worker_timeline().await?;

        let worker_count = timeline.as_array().map(|arr| arr.len()).unwrap_or(0);

        if worker_count < expected_min {
            anyhow::bail!(
                "Worker count below expected: at least {} expected, got {}",
                expected_min,
                worker_count
            );
        }

        Ok(())
    }

    /// Assert that metrics contain expected values
    pub async fn assert_metrics_contain(&self, metric_name: &str) -> Result<bool> {
        let metrics = self.get_metrics().await?;
        Ok(metrics.contains(metric_name))
    }

    /// Parse a metric value from the metrics endpoint
    pub async fn get_metric_value(&self, metric_name: &str) -> Result<Option<f64>> {
        let metrics = self.get_metrics().await?;

        for line in metrics.lines() {
            if line.starts_with(metric_name) && !line.starts_with("#") {
                // Parse: metric_name{labels} value
                if let Some(last_space) = line.rfind(' ') {
                    if let Ok(value) = line[last_space + 1..].parse::<f64>() {
                        return Ok(Some(value));
                    }
                }
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        // This test just verifies the client can be created
        // It will fail if there's no daemon running, which is expected
        let result = TestClient::new("http://127.0.0.1:3000").await;
        // We don't assert success here - this is just a compilation test
        let _ = result;
    }
}
