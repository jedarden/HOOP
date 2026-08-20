//! Integration tests for FleetNotification → operator-script delivery.

use std::os::unix::fs::PermissionsExt;
use std::time::Duration;

use tempfile::TempDir;

fn install_capture_script(dir: &TempDir, event: &str, output: &std::path::Path) {
    let script_path = dir.path().join("capture-event");
    let manifest_path = dir.path().join("capture-event.yml");
    let output_path = output.display().to_string().replace('\'', "'\\''");

    hoop_daemon::atomic_write::atomic_write_file_str(
        &script_path,
        &format!("#!/bin/sh\ncat > '{output_path}'\n"),
    )
    .expect("write capture script");

    let mut permissions = std::fs::metadata(&script_path)
        .expect("read capture script metadata")
        .permissions();
    permissions.set_mode(permissions.mode() | 0o111);
    std::fs::set_permissions(&script_path, permissions).expect("make capture script executable");

    let manifest = format!("name: capture-event\ntimeout_secs: 5\non:\n  - event: {event}\n");
    hoop_daemon::atomic_write::atomic_write_file_str(&manifest_path, &manifest)
        .expect("write capture manifest");
}

async fn wait_for_file(path: &std::path::Path) -> String {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    loop {
        if let Ok(contents) = std::fs::read_to_string(path) {
            return contents;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "timed out waiting for {}",
            path.display()
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

#[tokio::test]
async fn capacity_alert_push_triggers_subscribed_script() {
    let scripts_dir = TempDir::new().expect("create scripts directory");
    let output_path = scripts_dir.path().join("payload.json");
    install_capture_script(&scripts_dir, "capacity_alert", &output_path);

    let ring = hoop_daemon::fleet_notifications::FleetNotificationRing::new();
    let notification = hoop_daemon::fleet_notifications::FleetNotification::new(
        hoop_daemon::fleet_notifications::FleetNotificationKind::CapacityAlert,
        None,
        "Account utilization is high",
        serde_json::json!({
            "account_id": "claude-default",
            "utilization_5h": 85.5,
        }),
    );

    ring.push(notification.clone());
    hoop_daemon::script_trigger::spawn_fleet_notification_script_trigger(
        scripts_dir.path().to_path_buf(),
        notification,
    );

    let payload: serde_json::Value =
        serde_json::from_str(&wait_for_file(&output_path).await).expect("valid script payload");
    assert_eq!(payload["kind"], "capacity_alert");
    assert_eq!(payload["summary"], "Account utilization is high");
    assert_eq!(payload["details"]["utilization_5h"], 85.5);
}

#[tokio::test]
async fn needle_event_script_trigger_regression() {
    let scripts_dir = TempDir::new().expect("create scripts directory");
    let output_path = scripts_dir.path().join("payload.json");
    install_capture_script(&scripts_dir, "fail", &output_path);

    let raw = r#"{"event":"fail","ts":"2026-04-21T18:42:10Z","worker":"alpha","bead":"bd-abc123","error":"task failed"}"#;
    let event: hoop_daemon::events::NeedleEvent =
        serde_json::from_str(raw).expect("parse raw NeedleEvent");
    let ctx = hoop_daemon::script_trigger::EventContext::from_event(&event, raw);
    let results =
        hoop_daemon::script_trigger::trigger_matching_scripts(scripts_dir.path(), &ctx).await;

    assert_eq!(results.len(), 1);
    assert!(results[0].succeeded);
    let payload: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&output_path).expect("read script payload"))
            .expect("valid raw event payload");
    assert_eq!(payload["event"], "fail");
    assert_eq!(payload["bead"], "bd-abc123");
}
