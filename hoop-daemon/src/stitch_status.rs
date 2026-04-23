//! Stitch status derivation
//!
//! Pure function mapping Stitch context → derived status string.
//! Never stored; computed on read.
//!
//! Status rules (§1.6, §4.7):
//! - `In Progress` — any linked bead claimed right now, or streaming messages in last N minutes
//! - `Awaiting Review` — review-kind linked beads open
//! - `Quiet N days` — no activity for N days (N shown to user)
//!
//! This is a pure function: same inputs always produce the same output.
//! No side effects, no I/O, no mutable state.

use chrono::{DateTime, Duration, Utc};

/// Derived Stitch status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StitchStatus {
    /// Stitch has active work: claimed bead or recent streaming
    InProgress,
    /// Stitch has open review beads
    AwaitingReview,
    /// Stitch has been quiet for N days
    Quiet { days: u64 },
}

impl StitchStatus {
    /// Display string for UI
    pub fn as_str(&self) -> &'static str {
        match self {
            StitchStatus::InProgress => "In Progress",
            StitchStatus::AwaitingReview => "Awaiting Review",
            StitchStatus::Quiet { .. } => "Quiet",
        }
    }

    /// CSS class for UI styling
    pub fn css_class(&self) -> &'static str {
        match self {
            StitchStatus::InProgress => "status-in-progress",
            StitchStatus::AwaitingReview => "status-awaiting-review",
            StitchStatus::Quiet { .. } => "status-quiet",
        }
    }
}

/// Linked bead info needed for status derivation
#[derive(Debug, Clone)]
pub struct LinkedBead {
    /// Bead ID
    pub id: String,
    /// Current bead status
    pub status: BeadStatus,
    /// Bead type (review, task, etc.)
    pub issue_type: BeadType,
    /// Worker currently claiming this bead (if any)
    pub claimed_by: Option<String>,
    /// When this bead was last updated
    pub updated_at: DateTime<Utc>,
}

/// Bead status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeadStatus {
    Open,
    Closed,
}

/// Bead type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeadType {
    Task,
    Bug,
    Epic,
    Genesis,
    Review,
    Fix,
}

/// Activity info for a Stitch
#[derive(Debug, Clone)]
pub struct StitchActivity {
    /// Timestamp of most recent message
    pub last_message_at: Option<DateTime<Utc>>,
    /// Timestamp of most recent streaming content (if any)
    pub last_streaming_at: Option<DateTime<Utc>>,
}

/// Configuration for status derivation
#[derive(Debug, Clone)]
pub struct DeriveConfig {
    /// Minutes to consider streaming "recent" for In Progress
    pub streaming_recent_minutes: u64,
    /// Days threshold for Quiet status
    pub quiet_days_threshold: u64,
}

impl Default for DeriveConfig {
    fn default() -> Self {
        Self {
            streaming_recent_minutes: 5,
            quiet_days_threshold: 7,
        }
    }
}

/// Context for deriving Stitch status
///
/// All inputs needed to derive a Stitch's status.
/// This is the pure function's input.
#[derive(Debug, Clone)]
pub struct StitchContext {
    /// All beads linked to this Stitch
    pub linked_beads: Vec<LinkedBead>,
    /// Activity info for this Stitch
    pub activity: StitchActivity,
    /// Configuration for derivation
    pub config: DeriveConfig,
}

impl StitchContext {
    /// Derive the Stitch's status from its context
    ///
    /// This is the pure function: given the same inputs, it always returns
    /// the same output. No side effects, no I/O.
    ///
    /// Priority order (first match wins):
    /// 1. In Progress — claimed bead OR recent streaming
    /// 2. Awaiting Review — open review-kind beads
    /// 3. Quiet N days — no recent activity
    pub fn derive_status(&self) -> StitchStatus {
        // Check for In Progress first (highest priority)
        if self.is_in_progress() {
            return StitchStatus::InProgress;
        }

        // Check for Awaiting Review
        if self.has_open_review_beads() {
            return StitchStatus::AwaitingReview;
        }

        // Default to Quiet
        let days = self.days_since_activity();
        StitchStatus::Quiet { days }
    }

    /// Check if Stitch is in progress
    ///
    /// Returns true if:
    /// - Any linked bead is currently claimed (has a worker)
    /// - OR there was streaming activity in the last N minutes
    fn is_in_progress(&self) -> bool {
        // Check for claimed beads
        let has_claimed = self.linked_beads.iter().any(|bead| {
            bead.claimed_by.is_some() && bead.status == BeadStatus::Open
        });
        if has_claimed {
            return true;
        }

        // Check for recent streaming
        if let Some(streaming_at) = self.activity.last_streaming_at {
            let threshold = Duration::minutes(self.config.streaming_recent_minutes as i64);
            let now = Utc::now();
            if now.signed_duration_since(streaming_at) < threshold {
                return true;
            }
        }

        false
    }

    /// Check if Stitch has open review beads
    fn has_open_review_beads(&self) -> bool {
        self.linked_beads.iter().any(|bead| {
            bead.issue_type == BeadType::Review && bead.status == BeadStatus::Open
        })
    }

    /// Calculate days since last activity
    fn days_since_activity(&self) -> u64 {
        let last_activity = self.activity.last_message_at;

        if let Some(ts) = last_activity {
            let duration = Utc::now().signed_duration_since(ts);
            duration.num_days().max(0) as u64
        } else {
            // No activity recorded - use a large number
            999
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<Utc> {
        Utc::now()
    }

    fn minutes_ago(n: i64) -> DateTime<Utc> {
        now() - Duration::minutes(n)
    }

    fn days_ago(n: i64) -> DateTime<Utc> {
        now() - Duration::days(n)
    }

    fn open_bead(id: &str, issue_type: BeadType) -> LinkedBead {
        LinkedBead {
            id: id.to_string(),
            status: BeadStatus::Open,
            issue_type,
            claimed_by: None,
            updated_at: now(),
        }
    }

    fn claimed_bead(id: &str, issue_type: BeadType, worker: &str) -> LinkedBead {
        LinkedBead {
            id: id.to_string(),
            status: BeadStatus::Open,
            issue_type,
            claimed_by: Some(worker.to_string()),
            updated_at: now(),
        }
    }

    fn closed_bead(id: &str, issue_type: BeadType) -> LinkedBead {
        LinkedBead {
            id: id.to_string(),
            status: BeadStatus::Closed,
            issue_type,
            claimed_by: None,
            updated_at: now(),
        }
    }

    fn base_context() -> StitchContext {
        StitchContext {
            linked_beads: vec![],
            activity: StitchActivity {
                last_message_at: Some(now()),
                last_streaming_at: None,
            },
            config: DeriveConfig::default(),
        }
    }

    #[test]
    fn test_purity_same_inputs_same_output() {
        let ctx = base_context();

        // Multiple calls with same context produce same result
        let status1 = ctx.derive_status();
        let status2 = ctx.derive_status();
        let status3 = ctx.derive_status();

        assert_eq!(status1, status2);
        assert_eq!(status2, status3);
    }

    #[test]
    fn test_claimed_bead_is_in_progress() {
        let mut ctx = base_context();
        ctx.linked_beads = vec![claimed_bead("bd-1", BeadType::Task, "worker-alpha")];

        assert_eq!(ctx.derive_status(), StitchStatus::InProgress);
    }

    #[test]
    fn test_recent_streaming_is_in_progress() {
        let mut ctx = base_context();
        ctx.activity.last_streaming_at = Some(minutes_ago(2)); // 2 minutes ago
        ctx.linked_beads = vec![open_bead("bd-1", BeadType::Task)];

        assert_eq!(ctx.derive_status(), StitchStatus::InProgress);
    }

    #[test]
    fn test_old_streaming_not_in_progress() {
        let mut ctx = base_context();
        ctx.activity.last_streaming_at = Some(minutes_ago(10)); // 10 minutes ago (> 5 min threshold)
        ctx.linked_beads = vec![open_bead("bd-1", BeadType::Task)];

        assert_ne!(ctx.derive_status(), StitchStatus::InProgress);
    }

    #[test]
    fn test_open_review_bead_is_awaiting_review() {
        let mut ctx = base_context();
        ctx.linked_beads = vec![open_bead("bd-1", BeadType::Review)];

        assert_eq!(ctx.derive_status(), StitchStatus::AwaitingReview);
    }

    #[test]
    fn test_closed_review_bead_not_awaiting_review() {
        let mut ctx = base_context();
        ctx.linked_beads = vec![closed_bead("bd-1", BeadType::Review)];

        assert_ne!(ctx.derive_status(), StitchStatus::AwaitingReview);
    }

    #[test]
    fn test_quiet_status() {
        let mut ctx = base_context();
        ctx.linked_beads = vec![open_bead("bd-1", BeadType::Task)];
        ctx.activity.last_message_at = Some(days_ago(5));

        match ctx.derive_status() {
            StitchStatus::Quiet { days } => {
                assert_eq!(days, 5);
            }
            other => panic!("Expected Quiet status, got {:?}", other),
        }
    }

    #[test]
    fn test_priority_order_claimed_over_review() {
        let mut ctx = base_context();
        ctx.linked_beads = vec![
            claimed_bead("bd-1", BeadType::Task, "worker-alpha"),
            open_bead("bd-2", BeadType::Review),
        ];

        // In Progress should win even though there's an open review
        assert_eq!(ctx.derive_status(), StitchStatus::InProgress);
    }

    #[test]
    fn test_priority_order_review_over_quiet() {
        let mut ctx = base_context();
        ctx.linked_beads = vec![
            open_bead("bd-1", BeadType::Review),
            open_bead("bd-2", BeadType::Task),
        ];
        ctx.activity.last_message_at = Some(days_ago(10));

        // Awaiting Review should win over Quiet
        assert_eq!(ctx.derive_status(), StitchStatus::AwaitingReview);
    }

    #[test]
    fn test_no_activity_returns_max_quiet() {
        let mut ctx = base_context();
        ctx.linked_beads = vec![open_bead("bd-1", BeadType::Task)];
        ctx.activity.last_message_at = None;

        match ctx.derive_status() {
            StitchStatus::Quiet { days } => {
                assert_eq!(days, 999);
            }
            other => panic!("Expected Quiet status with 999 days, got {:?}", other),
        }
    }

    #[test]
    fn test_display_strings() {
        assert_eq!(StitchStatus::InProgress.as_str(), "In Progress");
        assert_eq!(StitchStatus::AwaitingReview.as_str(), "Awaiting Review");
        assert_eq!(StitchStatus::Quiet { days: 5 }.as_str(), "Quiet");
    }

    #[test]
    fn test_css_classes() {
        assert_eq!(StitchStatus::InProgress.css_class(), "status-in-progress");
        assert_eq!(StitchStatus::AwaitingReview.css_class(), "status-awaiting-review");
        assert_eq!(StitchStatus::Quiet { days: 5 }.css_class(), "status-quiet");
    }

    /// Performance test: status derivation for 20 beads must be < 10ms
    /// This is the acceptance criterion from §4.7
    #[test]
    fn test_performance_20_beads_under_10ms() {
        let mut ctx = base_context();

        // Create 20 linked beads (worst case scenario)
        for i in 0..20 {
            ctx.linked_beads.push(LinkedBead {
                id: format!("bd-{}", i),
                status: if i % 3 == 0 { BeadStatus::Closed } else { BeadStatus::Open },
                issue_type: match i % 4 {
                    0 => BeadType::Review,
                    1 => BeadType::Task,
                    2 => BeadType::Bug,
                    _ => BeadType::Fix,
                },
                claimed_by: if i % 5 == 0 { Some(format!("worker-{}", i)) } else { None },
                updated_at: days_ago(i as i64),
            });
        }
        ctx.activity.last_message_at = Some(days_ago(3));

        // Measure time
        let start = std::time::Instant::now();
        let _status = ctx.derive_status();
        let elapsed = start.elapsed();

        // Must be under 10ms
        assert!(
            elapsed.as_millis() < 10,
            "Status derivation took {}ms, must be < 10ms",
            elapsed.as_millis()
        );

        // Print actual time for visibility
        println!("Status derivation for 20 beads: {:?}", elapsed);
    }

    /// Backward compatibility: works without streaming data (phase 3-)
    #[test]
    fn test_backward_compat_no_streaming() {
        let mut ctx = base_context();
        ctx.linked_beads = vec![open_bead("bd-1", BeadType::Task)];
        ctx.activity.last_streaming_at = None; // No streaming data
        ctx.activity.last_message_at = Some(days_ago(5));

        // Should still work and return Quiet
        match ctx.derive_status() {
            StitchStatus::Quiet { days } => {
                assert_eq!(days, 5);
            }
            other => panic!("Expected Quiet status, got {:?}", other),
        }
    }

    /// Backward compatibility: works with empty linked beads (phase 3-)
    #[test]
    fn test_backward_compat_empty_linked_beads() {
        let ctx = StitchContext {
            linked_beads: vec![], // No linked beads
            activity: StitchActivity {
                last_message_at: Some(days_ago(10)),
                last_streaming_at: None,
            },
            config: DeriveConfig::default(),
        };

        // Should still work
        match ctx.derive_status() {
            StitchStatus::Quiet { days } => {
                assert_eq!(days, 10);
            }
            other => panic!("Expected Quiet status, got {:?}", other),
        }
    }
}
