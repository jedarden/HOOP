//! Fleet notification ring – structured events from the fleet to the agent.
//!
//! ## Overview
//!
//! Three trigger conditions emit a [`FleetNotification`] into the global ring:
//!
//! | Kind | Trigger |
//! |------|---------|
//! | [`FleetNotificationKind::StitchBeadsClosed`] | All beads linked to a Stitch closed |
//! | [`FleetNotificationKind::ConvoyComplete`] | All NEEDLE workers for a Stitch completed |
//! | [`FleetNotificationKind::CapacityAlert`] | 5-hour utilisation exceeded threshold |
//!
//! ## Agent delivery (≤5 s SLO)
//!
//! The ring is a global in-memory singleton backed by a `tokio::sync::broadcast`
//! channel. Subscribers (e.g. `AgentSessionManager`) receive notifications the
//! instant they are pushed and can inject them into a proactive agent turn —
//! no tool call required.
//!
//! ## History
//!
//! The last [`RING_SIZE`] (20) notifications are always available via
//! [`FleetNotificationRing::snapshot()`]. They are rendered into the system
//! prompt by `ContextIndex::to_system_prompt()` so the agent sees them on
//! every new session.
//!
//! ## Schema stability
//!
//! `FleetNotification` is forward-compatible: new optional fields must carry
//! `#[serde(default)]`. Existing fields are never removed or renamed.
//!
//! ## `escalate_to_operator`
//!
//! The agent may call the `escalate_to_operator` tool (defined in
//! `AgentAdapter`) to surface a notification to the operator via WebSocket.
//! Silently noting an event is the default — the agent only escalates when
//! the situation warrants human attention.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{OnceLock, RwLock};
use tokio::sync::broadcast;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum notifications retained in the ring.
pub const RING_SIZE: usize = 20;

/// Capacity-alert firing threshold — 5-hour sliding-window utilisation percent.
pub const CAPACITY_ALERT_THRESHOLD_PCT: f64 = 80.0;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Kinds of fleet notifications delivered to the agent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FleetNotificationKind {
    /// All beads linked to a Stitch (via `stitch_beads`) are now Closed.
    StitchBeadsClosed,
    /// All NEEDLE workers for a Stitch reached a terminal state
    /// (Complete / Close / Fail / Timeout / Crash / Release).
    ConvoyComplete,
    /// Account 5-hour utilisation exceeded [`CAPACITY_ALERT_THRESHOLD_PCT`].
    CapacityAlert,
}

/// A structured event emitted by the fleet for agent consumption.
///
/// **Schema contract**: new optional fields may be added with `#[serde(default)]`.
/// Existing fields are never removed or renamed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetNotification {
    /// Stable UUID identifying this notification.
    pub id: String,
    /// ISO-8601 wall-clock timestamp of the triggering event.
    pub ts: String,
    /// Event kind.
    pub kind: FleetNotificationKind,
    /// Project name, if the event is scoped to a single project.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    /// Human-readable summary (≤120 chars).
    pub summary: String,
    /// Kind-specific structured payload.
    pub details: serde_json::Value,
}

impl FleetNotification {
    /// Construct and timestamp a new notification.
    pub fn new(
        kind: FleetNotificationKind,
        project: Option<String>,
        summary: impl Into<String>,
        details: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            ts: Utc::now().to_rfc3339(),
            kind,
            project,
            summary: summary.into(),
            details,
        }
    }
}

// ---------------------------------------------------------------------------
// Ring buffer
// ---------------------------------------------------------------------------

/// Thread-safe ring buffer + broadcast channel for fleet notifications.
pub struct FleetNotificationRing {
    inner: RwLock<VecDeque<FleetNotification>>,
    tx: broadcast::Sender<FleetNotification>,
}

impl FleetNotificationRing {
    fn new() -> Self {
        let (tx, _) = broadcast::channel(64);
        Self {
            inner: RwLock::new(VecDeque::with_capacity(RING_SIZE)),
            tx,
        }
    }

    /// Push a notification into the ring, evicting the oldest entry when full.
    ///
    /// Broadcasts to all subscribers immediately (satisfies the ≤5 s SLO).
    pub fn push(&self, n: FleetNotification) {
        {
            let mut ring = self.inner.write().unwrap();
            if ring.len() >= RING_SIZE {
                ring.pop_front();
            }
            ring.push_back(n.clone());
        }
        // SendError only if no subscribers; that is fine.
        let _ = self.tx.send(n);
    }

    /// Return a snapshot of all retained notifications, oldest first.
    pub fn snapshot(&self) -> Vec<FleetNotification> {
        self.inner.read().unwrap().iter().cloned().collect()
    }

    /// Subscribe to future notifications (real-time, non-polling).
    pub fn subscribe(&self) -> broadcast::Receiver<FleetNotification> {
        self.tx.subscribe()
    }

    /// Number of notifications currently in the ring.
    pub fn len(&self) -> usize {
        self.inner.read().unwrap().len()
    }

    /// True if the ring is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// ---------------------------------------------------------------------------
// Global singleton
// ---------------------------------------------------------------------------

static RING: OnceLock<FleetNotificationRing> = OnceLock::new();

/// Access the global fleet-notification ring.
pub fn notifications() -> &'static FleetNotificationRing {
    RING.get_or_init(FleetNotificationRing::new)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_notification(kind: FleetNotificationKind) -> FleetNotification {
        FleetNotification::new(
            kind,
            Some("test-project".to_string()),
            "test summary",
            serde_json::json!({"bead": "ttb.1.2"}),
        )
    }

    #[test]
    fn ring_capacity_evicts_oldest() {
        let ring = FleetNotificationRing::new();
        for i in 0..(RING_SIZE + 5) {
            ring.push(FleetNotification {
                id: i.to_string(),
                ts: "2026-01-01T00:00:00Z".to_string(),
                kind: FleetNotificationKind::StitchBeadsClosed,
                project: None,
                summary: format!("notification {i}"),
                details: serde_json::Value::Null,
            });
        }
        let snap = ring.snapshot();
        assert_eq!(snap.len(), RING_SIZE);
        // Oldest should be evicted; first retained entry is index 5.
        assert_eq!(snap[0].id, "5");
    }

    #[test]
    fn notification_serialization_stable() {
        let n = make_notification(FleetNotificationKind::CapacityAlert);
        let json = serde_json::to_string(&n).unwrap();
        let back: FleetNotification = serde_json::from_str(&json).unwrap();
        assert_eq!(back.kind, FleetNotificationKind::CapacityAlert);
        assert_eq!(back.summary, "test summary");
        assert_eq!(back.project, Some("test-project".to_string()));
    }

    #[test]
    fn kind_serializes_as_snake_case() {
        assert_eq!(
            serde_json::to_value(FleetNotificationKind::StitchBeadsClosed)
                .unwrap()
                .as_str()
                .unwrap(),
            "stitch_beads_closed"
        );
        assert_eq!(
            serde_json::to_value(FleetNotificationKind::ConvoyComplete)
                .unwrap()
                .as_str()
                .unwrap(),
            "convoy_complete"
        );
        assert_eq!(
            serde_json::to_value(FleetNotificationKind::CapacityAlert)
                .unwrap()
                .as_str()
                .unwrap(),
            "capacity_alert"
        );
    }

    #[test]
    fn snapshot_empty_ring() {
        let ring = FleetNotificationRing::new();
        assert!(ring.snapshot().is_empty());
        assert!(ring.is_empty());
    }

    #[test]
    fn snapshot_ordering_oldest_first() {
        let ring = FleetNotificationRing::new();
        ring.push(FleetNotification {
            id: "first".to_string(),
            ts: "2026-01-01T00:00:00Z".to_string(),
            kind: FleetNotificationKind::ConvoyComplete,
            project: None,
            summary: "first".to_string(),
            details: serde_json::Value::Null,
        });
        ring.push(FleetNotification {
            id: "second".to_string(),
            ts: "2026-01-01T00:00:01Z".to_string(),
            kind: FleetNotificationKind::CapacityAlert,
            project: None,
            summary: "second".to_string(),
            details: serde_json::Value::Null,
        });
        let snap = ring.snapshot();
        assert_eq!(snap[0].id, "first");
        assert_eq!(snap[1].id, "second");
    }

    #[test]
    fn notification_details_preserved() {
        let ring = FleetNotificationRing::new();
        ring.push(FleetNotification::new(
            FleetNotificationKind::StitchBeadsClosed,
            Some("proj".to_string()),
            "all beads closed",
            serde_json::json!({
                "stitch_id": "st-abc",
                "bead_ids": ["ttb.1.1", "ttb.1.2"],
                "closed_count": 2
            }),
        ));
        let snap = ring.snapshot();
        assert_eq!(snap[0].details["stitch_id"], "st-abc");
        assert_eq!(snap[0].details["closed_count"], 2);
    }

    #[test]
    fn capacity_threshold_constant_is_sane() {
        assert!(CAPACITY_ALERT_THRESHOLD_PCT > 0.0);
        assert!(CAPACITY_ALERT_THRESHOLD_PCT < 100.0);
    }
}
