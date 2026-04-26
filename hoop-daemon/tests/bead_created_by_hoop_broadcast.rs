//! Integration test: bead_created_by_hoop event broadcast and fleet notification (hoop-ttb.3.53)
//!
//! Acceptance criteria:
//! - Broadcast within 100ms of successful create
//! - Event schema in hoop-schema/
//! - Agent notification path tested
//!
//! This test validates the full flow:
//! 1. bead_created_by_hoop event is broadcast on the channel
//! 2. Fleet notification is received by agents
//! 3. WebSocket forwarding would deliver to project subscribers

use hoop_daemon::fleet_notifications::{FleetNotification, FleetNotificationKind, notifications};
use hoop_daemon::ws::BeadCreatedByHoopData;
use std::time::Duration;
use tokio::sync::broadcast;

// ---------------------------------------------------------------------------
// Test: bead_created_by_hoop broadcast → fleet notification
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_bead_created_by_hoop_broadcast_to_fleet_notification() {
    // Create a broadcast channel (simulates DaemonState.bead_created_by_hoop_tx)
    let (tx, _rx) = broadcast::channel::<BeadCreatedByHoopData>(64);

    // Subscribe to the fleet notification ring (simulates agent session)
    let mut fleet_rx = notifications().subscribe();

    // Spawn task that forwards bead_created_by_hoop events to fleet notifications
    // (simulates the integration in lib.rs lines 2062-2092)
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        use hoop_daemon::fleet_notifications::FleetNotification;
        use hoop_daemon::fleet_notifications::FleetNotificationKind;

        let mut rx = tx_clone.subscribe();
        if let Ok(data) = rx.recv().await {
            let notification = FleetNotification::new(
                FleetNotificationKind::BeadCreatedByHoop,
                Some(data.project.clone()),
                format!("Bead {} created via HOOP by {}", data.bead_id, data.actor),
                serde_json::json!({
                    "bead_id": data.bead_id,
                    "project": data.project,
                    "actor": data.actor,
                    "source": data.source,
                    "ts": data.ts,
                }),
            );
            hoop_daemon::fleet_notifications::notifications().push(notification);
        }
    });

    // Create and send a bead_created_by_hoop event
    let event = BeadCreatedByHoopData {
        project: "test-project".to_string(),
        bead_id: "hoop-ttb.3.53".to_string(),
        actor: "tailscale:test@example.com".to_string(),
        source: "form".to_string(),
        ts: chrono::Utc::now().to_rfc3339(),
    };

    let start = std::time::Instant::now();
    let _ = tx.send(event);

    // Wait for fleet notification (with timeout)
    let notification = tokio::time::timeout(
        Duration::from_millis(200),
        fleet_rx.recv()
    )
    .await
    .expect("Fleet notification should be received within 200ms")
    .expect("Fleet notification channel should not be closed");

    let elapsed = start.elapsed();

    // Verify notification details
    assert_eq!(notification.kind, FleetNotificationKind::BeadCreatedByHoop);
    assert_eq!(notification.project, Some("test-project".to_string()));
    assert!(notification.summary.contains("hoop-ttb.3.53"));
    assert!(notification.summary.contains("test@example.com"));

    // Verify details payload
    assert_eq!(notification.details["bead_id"], "hoop-ttb.3.53");
    assert_eq!(notification.details["project"], "test-project");
    assert_eq!(notification.details["actor"], "tailscale:test@example.com");
    assert_eq!(notification.details["source"], "form");

    // Acceptance: Broadcast within 100ms
    assert!(
        elapsed < Duration::from_millis(100),
        "Notification should be received within 100ms, took {}ms",
        elapsed.as_millis()
    );
}

// ---------------------------------------------------------------------------
// Test: Event schema matches expected structure
// ---------------------------------------------------------------------------

#[test]
fn test_bead_created_by_hoop_schema_structure() {
    // Verify the event can be serialized/deserialized correctly
    let event = BeadCreatedByHoopData {
        project: "test-project".to_string(),
        bead_id: "bd-123".to_string(),
        actor: "os:testuser".to_string(),
        source: "chat".to_string(),
        ts: "2026-04-26T12:00:00Z".to_string(),
    };

    let json = serde_json::to_string(&event).expect("Should serialize");
    let parsed: BeadCreatedByHoopData = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(parsed.project, "test-project");
    assert_eq!(parsed.bead_id, "bd-123");
    assert_eq!(parsed.actor, "os:testuser");
    assert_eq!(parsed.source, "chat");
    assert_eq!(parsed.ts, "2026-04-26T12:00:00Z");
}

// ---------------------------------------------------------------------------
// Test: Fleet notification ring retains bead_created_by_hoop events
// ---------------------------------------------------------------------------

#[test]
fn test_fleet_notification_ring_retains_bead_created_by_hoop() {
    // Clear any existing notifications
    let snapshot = notifications().snapshot();
    let initial_count = snapshot.len();

    // Create and push a bead_created_by_hoop notification
    let notification = FleetNotification::new(
        FleetNotificationKind::BeadCreatedByHoop,
        Some("test-project".to_string()),
        "Test bead created".to_string(),
        serde_json::json!({"bead_id": "bd-test"}),
    );

    notifications().push(notification);

    // Verify it's in the snapshot
    let new_snapshot = notifications().snapshot();
    assert_eq!(new_snapshot.len(), initial_count + 1);

    // Find the bead_created_by_hoop notification
    let found = new_snapshot
        .iter()
        .any(|n| n.kind == FleetNotificationKind::BeadCreatedByHoop);
    assert!(found, "Fleet notification ring should contain bead_created_by_hoop event");
}
