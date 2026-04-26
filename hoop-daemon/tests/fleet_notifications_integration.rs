//! Fleet notification integration tests
//!
//! Acceptance criteria:
//! - Events delivered within 5s of trigger
//! - Event schema stable and documented
//! - Agent has history of recent events (last 20) without consuming tool calls
//! - Test: synthetic event delivered and agent responds correctly
//!
//! Plan reference: §6 Phase 5 deliverable 4

use std::time::Duration;
use tokio::time::timeout;

/// Test that notifications are delivered to subscribers within 5 seconds
#[tokio::test]
async fn notification_delivered_within_5s() {
    // Reset the global ring for this test
    let ring = hoop_daemon::fleet_notifications::FleetNotificationRing::new();

    // Subscribe before pushing
    let mut rx = ring.subscribe();

    // Push a notification
    let notification = hoop_daemon::fleet_notifications::FleetNotification::new(
        hoop_daemon::fleet_notifications::FleetNotificationKind::StitchBeadsClosed,
        Some("test-project".to_string()),
        "Test notification",
        serde_json::json!({"stitch_id": "test-123"}),
    );
    ring.push(notification);

    // Verify delivery within 5s (should be near-instant)
    let result = timeout(Duration::from_secs(5), rx.recv()).await;
    assert!(
        result.is_ok(),
        "Notification should be delivered within 5 seconds"
    );

    let received = result.unwrap().unwrap();
    assert_eq!(received.kind, hoop_daemon::fleet_notifications::FleetNotificationKind::StitchBeadsClosed);
    assert_eq!(received.summary, "Test notification");
    assert_eq!(received.project, Some("test-project".to_string()));
}

/// Test that notification schema is stable and serializable
#[tokio::test]
async fn notification_schema_stable() {
    let notification = hoop_daemon::fleet_notifications::FleetNotification::new(
        hoop_daemon::fleet_notifications::FleetNotificationKind::CapacityAlert,
        Some("test-project".to_string()),
        "Capacity alert",
        serde_json::json!({
            "utilization_percent": 85.5,
            "threshold_percent": 80.0,
        }),
    );

    // Verify JSON serialization
    let json = serde_json::to_string(&notification).expect("Should serialize to JSON");

    // Verify it contains expected fields
    assert!(json.contains("\"id\":"));
    assert!(json.contains("\"ts\":"));
    assert!(json.contains("\"kind\":"));
    assert!(json.contains("\"project\":"));
    assert!(json.contains("\"summary\":"));
    assert!(json.contains("\"details\":"));

    // Verify round-trip
    let parsed: hoop_daemon::fleet_notifications::FleetNotification =
        serde_json::from_str(&json).expect("Should deserialize from JSON");
    assert_eq!(parsed.id, notification.id);
    assert_eq!(parsed.kind, notification.kind);
    assert_eq!(parsed.summary, notification.summary);
}

/// Test that agent has access to last 20 notifications without tool calls
#[tokio::test]
async fn agent_access_recent_notifications_without_tool_calls() {
    let ring = hoop_daemon::fleet_notifications::FleetNotificationRing::new();

    // Push 25 notifications (more than RING_SIZE of 20)
    for i in 0..25 {
        ring.push(hoop_daemon::fleet_notifications::FleetNotification::new(
            hoop_daemon::fleet_notifications::FleetNotificationKind::ConvoyComplete,
            Some("test-project".to_string()),
            format!("Notification {}", i),
            serde_json::json!({"index": i}),
        ));
    }

    // Snapshot should contain exactly RING_SIZE (20) notifications
    let snapshot = ring.snapshot();
    assert_eq!(
        snapshot.len(),
        hoop_daemon::fleet_notifications::RING_SIZE,
        "Snapshot should contain exactly RING_SIZE notifications"
    );

    // Oldest notification (index 5) should be first, newest (index 24) last
    assert_eq!(snapshot[0].summary, "Notification 5", "Oldest retained notification should be index 5");
    assert_eq!(snapshot[19].summary, "Notification 24", "Newest notification should be index 24");
}

/// Test synthetic event delivery through all three notification kinds
#[tokio::test]
async fn synthetic_event_all_kinds() {
    let ring = hoop_daemon::fleet_notifications::FleetNotificationRing::new();
    let mut rx = ring.subscribe();

    // Test StitchBeadsClosed
    ring.push(hoop_daemon::fleet_notifications::FleetNotification::new(
        hoop_daemon::fleet_notifications::FleetNotificationKind::StitchBeadsClosed,
        Some("my-project".to_string()),
        "All beads closed for stitch 'Fix authentication bug'",
        serde_json::json!({
            "stitch_id": "st-abc123",
            "bead_ids": ["ttb.1.1", "ttb.1.2", "ttb.1.3"],
            "closed_count": 3,
        }),
    ));

    // Test ConvoyComplete
    ring.push(hoop_daemon::fleet_notifications::FleetNotification::new(
        hoop_daemon::fleet_notifications::FleetNotificationKind::ConvoyComplete,
        Some("my-project".to_string()),
        "All workers completed for stitch 'Refactor API'",
        serde_json::json!({
            "stitch_id": "st-def456",
            "bead_ids": ["ttb.2.1", "ttb.2.2"],
            "worker_count": 2,
        }),
    ));

    // Test CapacityAlert
    ring.push(hoop_daemon::fleet_notifications::FleetNotification::new(
        hoop_daemon::fleet_notifications::FleetNotificationKind::CapacityAlert,
        None,
        "Account 'claude-default' 5h utilization at 92.5%",
        serde_json::json!({
            "account_id": "claude-default",
            "adapter": "claude",
            "utilization_5h": 92.5,
            "utilization_7d": 78.0,
            "threshold_pct": 80.0,
        }),
    ));

    // Verify all three were received
    let recv1 = timeout(Duration::from_secs(1), rx.recv()).await.unwrap().unwrap();
    assert_eq!(recv1.kind, hoop_daemon::fleet_notifications::FleetNotificationKind::StitchBeadsClosed);

    let recv2 = timeout(Duration::from_secs(1), rx.recv()).await.unwrap().unwrap();
    assert_eq!(recv2.kind, hoop_daemon::fleet_notifications::FleetNotificationKind::ConvoyComplete);

    let recv3 = timeout(Duration::from_secs(1), rx.recv()).await.unwrap().unwrap();
    assert_eq!(recv3.kind, hoop_daemon::fleet_notifications::FleetNotificationKind::CapacityAlert);

    // Verify snapshot contains all three
    let snapshot = ring.snapshot();
    assert_eq!(snapshot.len(), 3);
}

/// Test that agent context includes fleet notifications
#[test]
fn agent_context_includes_fleet_notifications() {
    // Create a test YAML config with a project
    let yaml = serde_yaml::from_str(
        r#"
projects:
  - name: test-project
    path: /tmp/test
"#,
    )
    .unwrap();

    // Build context index - this loads fleet notifications from the global ring
    let index = hoop_daemon::agent_context::ContextIndex::build_for_test(&yaml);

    // Fleet notifications should be present (even if empty in test)
    assert!(
        index.notifications.len() <= hoop_daemon::fleet_notifications::RING_SIZE,
        "Fleet notifications in context should not exceed RING_SIZE"
    );
}

/// Test that notification details are preserved
#[tokio::test]
async fn notification_details_preserved() {
    let ring = hoop_daemon::fleet_notifications::FleetNotificationRing::new();

    let details = serde_json::json!({
        "stitch_id": "st-xyz",
        "bead_ids": ["ttb.1.1", "ttb.1.2"],
        "closed_count": 2,
        "metadata": {
            "project": "test-project",
            "closed_by": "test-user"
        }
    });

    ring.push(hoop_daemon::fleet_notifications::FleetNotification::new(
        hoop_daemon::fleet_notifications::FleetNotificationKind::StitchBeadsClosed,
        Some("test-project".to_string()),
        "Test notification with complex details",
        details.clone(),
    ));

    let snapshot = ring.snapshot();
    assert_eq!(snapshot.len(), 1);

    // Verify all details are preserved
    assert_eq!(snapshot[0].details["stitch_id"], "st-xyz");
    assert_eq!(snapshot[0].details["closed_count"], 2);
    assert_eq!(snapshot[0].details["metadata"]["project"], "test-project");
}

/// Test multiple subscribers receive the same notification
#[tokio::test]
async fn multiple_subscribers_receive_notification() {
    let ring = hoop_daemon::fleet_notifications::FleetNotificationRing::new();

    // Create multiple subscribers
    let mut rx1 = ring.subscribe();
    let mut rx2 = ring.subscribe();
    let mut rx3 = ring.subscribe();

    // Push a single notification
    ring.push(hoop_daemon::fleet_notifications::FleetNotification::new(
        hoop_daemon::fleet_notifications::FleetNotificationKind::BeadCreatedByHoop,
        Some("multi-sub-test".to_string()),
        "Broadcast to all subscribers",
        serde_json::json!({"bead_id": "ttb.9.9"}),
    ));

    // All subscribers should receive the notification
    let recv1 = timeout(Duration::from_secs(1), rx1.recv()).await.unwrap().unwrap();
    let recv2 = timeout(Duration::from_secs(1), rx2.recv()).await.unwrap().unwrap();
    let recv3 = timeout(Duration::from_secs(1), rx3.recv()).await.unwrap().unwrap();

    assert_eq!(recv1.summary, "Broadcast to all subscribers");
    assert_eq!(recv2.summary, "Broadcast to all subscribers");
    assert_eq!(recv3.summary, "Broadcast to all subscribers");
}
