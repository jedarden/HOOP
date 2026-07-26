//! BeadStatus wire-format deserialization (bf-364c3 / bf-315nx mismatch #3).
//!
//! br/bead-forge writes `status` as an always-lowercase snake_case string
//! (open, closed, blocked, completed, done). HOOP's `BeadStatus` enum must
//! deserialize every one of those wire values, and fall back to `Unknown`
//! for any unrecognized status rather than erroring — an error here would
//! quarantine 100% of captured bead lines.

use hoop_daemon::BeadStatus;

/// Each observed br/bead-forge wire value deserializes to the matching variant.
#[test]
fn bead_status_deserializes_known_lowercase_wire_values() {
    assert_eq!(
        serde_json::from_str::<BeadStatus>("\"open\"").unwrap(),
        BeadStatus::Open
    );
    assert_eq!(
        serde_json::from_str::<BeadStatus>("\"closed\"").unwrap(),
        BeadStatus::Closed
    );
    assert_eq!(
        serde_json::from_str::<BeadStatus>("\"blocked\"").unwrap(),
        BeadStatus::Blocked
    );
    assert_eq!(
        serde_json::from_str::<BeadStatus>("\"completed\"").unwrap(),
        BeadStatus::Completed
    );
    assert_eq!(
        serde_json::from_str::<BeadStatus>("\"done\"").unwrap(),
        BeadStatus::Done
    );
}

/// An unrecognized status string deserializes to `Unknown` instead of erroring,
/// so future/unrecognized statuses never cause a quarantine.
#[test]
fn bead_status_unrecognized_status_becomes_unknown() {
    assert_eq!(
        serde_json::from_str::<BeadStatus>("\"cancelled\"").unwrap(),
        BeadStatus::Unknown
    );
    assert_eq!(
        serde_json::from_str::<BeadStatus>("\"in-progress\"").unwrap(),
        BeadStatus::Unknown
    );
}
