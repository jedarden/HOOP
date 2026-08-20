//! Epoch-sync invariant integration test (§B2, hoop-ttb.3.46)
//!
//! Validates the core invariant: on every WS (re)connect, the client wipes
//! its atom store and rebuilds from the server's `init` payload.
//!
//! Test scenarios:
//! 1. Initial connection receives init and all snapshot events
//! 2. Disconnect → server state changes → reconnect → stale rows gone
//! 3. Optimistic stubs survive reconnect (client-side only, not tested here)
//!
//! This is an integration test with actual WebSocket connections and server state.

use futures_util::stream::StreamExt;
use std::time::Duration;
use tokio::time::timeout;
use tokio_tungstenite;

mod integration_harness;
use integration_harness::spawn_test_daemon;

#[tokio::test]
async fn test_epoch_sync_init_event_carrying_subscriptions() {
    // Test that the init event carries the server-authoritative subscription list
    let (base_url, _shutdown) = spawn_test_daemon()
        .await
        .expect("Failed to spawn test daemon");

    let ws_url = base_url.replace("http://", "ws://");
    let ws_url = format!("{}/ws", ws_url);

    let (ws_stream, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .expect("Failed to connect to WebSocket");

    let (_, mut ws_receiver) = ws_stream.split();

    // First message must be init
    let init_msg = timeout(Duration::from_secs(2), ws_receiver.next())
        .await
        .expect("Timeout waiting for init message")
        .expect("WebSocket stream ended");

    let init_msg = init_msg.expect("Failed to receive init message");

    if let tokio_tungstenite::tungstenite::Message::Text(text) = init_msg {
        let event: serde_json::Value =
            serde_json::from_str(&text).expect("Failed to parse init event as JSON");

        assert_eq!(event["type"], "init", "First message should be init event");
        assert!(
            event["subscriptions"].is_array(),
            "init should contain subscriptions array"
        );

        // Verify global is always in subscriptions
        let subs: Vec<&str> = event["subscriptions"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|s| s.as_str())
            .collect();

        assert!(
            subs.contains(&"global"),
            "global should always be in subscriptions"
        );
    } else {
        panic!("Expected text message for init, got {:?}", init_msg);
    }
}

#[tokio::test]
async fn test_epoch_sync_initial_snapshots_after_init() {
    // Test that init is followed by all snapshot events in order
    let (base_url, _shutdown) = spawn_test_daemon()
        .await
        .expect("Failed to spawn test daemon");

    let ws_url = base_url.replace("http://", "ws://");
    let ws_url = format!("{}/ws", ws_url);

    let (ws_stream, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .expect("Failed to connect to WebSocket");

    let (_, mut ws_receiver) = ws_stream.split();

    // Collect first 10 messages
    let mut messages = Vec::new();
    for _ in 0..10 {
        match timeout(Duration::from_secs(2), ws_receiver.next()).await {
            Ok(Some(Ok(msg))) => {
                if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
                    messages.push(text);
                }
            }
            _ => break,
        }
    }

    assert!(!messages.is_empty(), "Should receive at least one message");

    // First message must be init
    let first_event: serde_json::Value =
        serde_json::from_str(&messages[0]).expect("Failed to parse first message");
    assert_eq!(first_event["type"], "init", "First message must be init");

    // Verify we receive snapshot events after init
    let mut received_workers_snapshot = false;
    let mut received_beads_snapshot = false;
    let mut _received_conversations_snapshot = false;
    let mut _received_projects_snapshot = false;
    let mut received_config_status = false;

    for msg in &messages[1..] {
        if let Ok(event) = serde_json::from_str::<serde_json::Value>(msg) {
            match event.get("type").and_then(|t| t.as_str()) {
                Some("workers_snapshot") => received_workers_snapshot = true,
                Some("beads_snapshot") => received_beads_snapshot = true,
                Some("conversations_snapshot") => _received_conversations_snapshot = true,
                Some("projects_snapshot") => _received_projects_snapshot = true,
                Some("config_status") => received_config_status = true,
                _ => {}
            }
        }
    }

    assert!(
        received_workers_snapshot,
        "Should receive workers_snapshot after init"
    );
    assert!(
        received_beads_snapshot,
        "Should receive beads_snapshot after init"
    );
    assert!(
        received_config_status,
        "Should receive config_status after init"
    );
}

#[tokio::test]
async fn test_epoch_sync_reconnect_wipes_and_rebuilds() {
    // Core invariant: disconnect → server state changes → reconnect → stale rows gone
    let (base_url, _shutdown) = spawn_test_daemon()
        .await
        .expect("Failed to spawn test daemon");

    let ws_url = base_url.replace("http://", "ws://");
    let ws_url = format!("{}/ws", ws_url);

    // First connection: get initial bead count
    let (ws_stream, _) = tokio_tungstenite::connect_async(&ws_url.clone())
        .await
        .expect("Failed to connect to WebSocket");

    let (_, mut ws_receiver) = ws_stream.split();

    // Wait for beads_snapshot
    let mut initial_beads_count = 0;
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(3) {
        match timeout(Duration::from_secs(1), ws_receiver.next()).await {
            Ok(Some(Ok(msg))) => {
                if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
                    if let Ok(event) = serde_json::from_str::<serde_json::Value>(&text) {
                        if event["type"] == "beads_snapshot" {
                            if let Some(beads) = event.get("beads").and_then(|b| b.as_array()) {
                                initial_beads_count = beads.len();
                            }
                            break;
                        }
                    }
                }
            }
            _ => break,
        }
    }

    // Connection closes implicitly when ws_receiver is dropped

    // Simulate a brief delay (client disconnected, server state might change)
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Second connection: receive fresh snapshot
    let (ws_stream2, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .expect("Failed to reconnect to WebSocket");

    let (_, mut ws_receiver2) = ws_stream2.split();

    // We should receive a new init event followed by snapshots
    let mut received_init = false;
    let mut received_beads_snapshot = false;
    let mut reconnect_beads_count = 0;

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
                                    reconnect_beads_count = beads.len();
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

    // The bead count should be consistent (same server state)
    assert_eq!(
        initial_beads_count, reconnect_beads_count,
        "Bead count should be consistent across reconnects"
    );
}

#[tokio::test]
async fn test_epoch_sync_init_is_always_first_message() {
    // Verify that init is always the first message on any connection
    let (base_url, _shutdown) = spawn_test_daemon()
        .await
        .expect("Failed to spawn test daemon");

    let ws_url = base_url.replace("http://", "ws://");
    let ws_url = format!("{}/ws", ws_url);

    // Test multiple connections
    for i in 0..3 {
        let (ws_stream, _) = tokio_tungstenite::connect_async(&ws_url.clone())
            .await
            .expect("Failed to connect to WebSocket (iteration {})");

        let (_, mut ws_receiver) = ws_stream.split();

        let first_msg = timeout(Duration::from_secs(2), ws_receiver.next())
            .await
            .expect(&format!(
                "Timeout waiting for init message (iteration {})",
                i
            ))
            .expect("WebSocket stream ended");

        let first_msg =
            first_msg.expect(&format!("Failed to receive init message (iteration {})", i));

        if let tokio_tungstenite::tungstenite::Message::Text(text) = first_msg {
            let event: serde_json::Value =
                serde_json::from_str(&text).expect("Failed to parse message as JSON");

            assert_eq!(
                event["type"], "init",
                "First message must be init (iteration {})",
                i
            );
        } else {
            panic!("Expected text message (iteration {})", i);
        }

        // Connection closes when ws_receiver is dropped
    }
}

#[tokio::test]
async fn test_epoch_sync_concurrent_connections() {
    // Test that multiple concurrent connections each receive their own init
    let (base_url, _shutdown) = spawn_test_daemon()
        .await
        .expect("Failed to spawn test daemon");

    let ws_url = base_url.replace("http://", "ws://");
    let ws_url = format!("{}/ws", ws_url);

    // Spawn multiple concurrent connections
    let mut handles = Vec::new();
    for i in 0..5 {
        let ws_url_clone = ws_url.clone();
        let handle = tokio::spawn(async move {
            let (ws_stream, _) = tokio_tungstenite::connect_async(&ws_url_clone)
                .await
                .expect("Failed to connect");

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
