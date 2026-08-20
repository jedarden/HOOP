//! Real br-line deserialization regression test (bf-4hsqk — bf-315nx verification).
//!
//! br/bead-forge writes beads as JSONL lines to issues.jsonl. This test verifies
//! that HOOP can deserialize REAL captured br lines without error, covering:
//! - Each BeadStatus lowercase wire value (open, closed, blocked, completed, done)
//! - Each BeadType lowercase wire value (task, bug, chore, feature, test, docs, story, epic, genesis, review, fix)
//! - Missing created_by/dependencies keys (defaults to empty String/Vec)
//! - Unknown status/issue_type values (become Unknown variant instead of erroring)

use hoop_daemon::{Bead, BeadStatus, BeadType};

/// A REAL captured br line from bead-forge's issues.jsonl.
///
/// This is an actual line captured from a live br/bead-forge installation.
/// Deserializing it must succeed without error — a failure here means
/// production bead lines will be quarantined.
const REAL_BR_LINE: &str = r#"{
  "id":"bf-1na",
  "title":"Regression test: explore reaches workspace[N-1] when workspace[0] excluded via live exclusion set",
  "description":"Part of bf-47bfm acceptance criterion 3. Depends on bf-2x5.",
  "status":"closed",
  "priority":1,
  "issue_type":"task",
  "created_at":"2026-07-27T15:45:49.119046018Z",
  "updated_at":"2026-07-27T15:47:00.548801080Z",
  "closed_at":"2026-07-27T15:47:00.548801080Z",
  "close_reason":"Created in the wrong bead store by operator mistake",
  "closed_by_session":"cli",
  "source_repo":".",
  "compaction_level":0,
  "dependencies":[
    {"issue_id":"bf-1na","depends_on_id":"bf-2x5","type":"blocks","created_at":"2026-07-27T15:45:53.694907667Z","created_by":"cli","thread_id":""}
  ]
}"#;

/// Real captured br line deserializes into Bead without error.
#[test]
fn real_br_line_deserializes_successfully() {
    let bead: Bead =
        serde_json::from_str(REAL_BR_LINE).expect("Real br line must deserialize successfully");

    assert_eq!(bead.id, "bf-1na");
    assert_eq!(bead.status, BeadStatus::Closed);
    assert_eq!(bead.issue_type, BeadType::Task);
    assert_eq!(bead.priority, 1);
}

/// Minimal valid br line — only required keys, no created_by or dependencies.
#[test]
fn minimal_bead_line_with_defaults_deserializes() {
    let minimal = r#"{
        "id":"test-1",
        "title":"Test bead",
        "status":"open",
        "priority":0,
        "issue_type":"bug",
        "created_at":"2026-08-02T00:00:00Z",
        "updated_at":"2026-08-02T00:00:00Z"
    }"#;

    let bead: Bead = serde_json::from_str(minimal)
        .expect("Minimal bead line (without created_by/dependencies) must deserialize");

    assert_eq!(bead.id, "test-1");
    assert_eq!(bead.created_by, ""); // Default to empty string
    assert!(bead.dependencies.is_empty()); // Default to empty Vec
    assert_eq!(bead.status, BeadStatus::Open);
    assert_eq!(bead.issue_type, BeadType::Bug);
}

/// Each BeadStatus lowercase wire value deserializes correctly.
#[test]
fn all_bead_status_lowercase_values_deserialize() {
    let statuses = [
        ("open", BeadStatus::Open),
        ("closed", BeadStatus::Closed),
        ("blocked", BeadStatus::Blocked),
        ("completed", BeadStatus::Completed),
        ("done", BeadStatus::Done),
    ];

    for (wire_value, expected) in statuses {
        let json = format!(
            r#"{{"id":"test-{v}","title":"Test","status":"{v}","priority":0,"issue_type":"task","created_at":"2026-08-02T00:00:00Z","updated_at":"2026-08-02T00:00:00Z"}}"#,
            v = wire_value
        );

        let bead: Bead = serde_json::from_str(&json)
            .unwrap_or_else(|_| panic!("Status '{}' must deserialize", wire_value));

        assert_eq!(
            bead.status, expected,
            "Status '{}' should deserialize to {:?}",
            wire_value, expected
        );
    }
}

/// Each BeadType lowercase wire value deserializes correctly.
#[test]
fn all_bead_type_lowercase_values_deserialize() {
    let types = [
        ("task", BeadType::Task),
        ("bug", BeadType::Bug),
        ("chore", BeadType::Chore),
        ("feature", BeadType::Feature),
        ("test", BeadType::Test),
        ("docs", BeadType::Docs),
        ("story", BeadType::Story),
        ("epic", BeadType::Epic),
        ("genesis", BeadType::Genesis),
        ("review", BeadType::Review),
        ("fix", BeadType::Fix),
    ];

    for (wire_value, expected) in types {
        let json = format!(
            r#"{{"id":"test-{v}","title":"Test","status":"open","priority":0,"issue_type":"{v}","created_at":"2026-08-02T00:00:00Z","updated_at":"2026-08-02T00:00:00Z"}}"#,
            v = wire_value
        );

        let bead: Bead = serde_json::from_str(&json)
            .unwrap_or_else(|_| panic!("Issue type '{}' must deserialize", wire_value));

        assert_eq!(
            bead.issue_type, expected,
            "Issue type '{}' should deserialize to {:?}",
            wire_value, expected
        );
    }
}

/// Unrecognized BeadStatus values become Unknown instead of erroring.
#[test]
fn unrecognized_bead_status_becomes_unknown() {
    let unknown_statuses = ["cancelled", "in-progress", "pending-review", "archived"];

    for wire_value in unknown_statuses {
        let json = format!(
            r#"{{"id":"test-{v}","title":"Test","status":"{v}","priority":0,"issue_type":"task","created_at":"2026-08-02T00:00:00Z","updated_at":"2026-08-02T00:00:00Z"}}"#,
            v = wire_value
        );

        let bead: Bead = serde_json::from_str(&json).unwrap_or_else(|_| {
            panic!(
                "Unrecognized status '{}' must deserialize as Unknown",
                wire_value
            )
        });

        assert_eq!(
            bead.status,
            BeadStatus::Unknown,
            "Unrecognized status '{}' should become Unknown",
            wire_value
        );
    }
}

/// Unrecognized BeadType values become Unknown instead of erroring.
#[test]
fn unrecognized_bead_type_becomes_unknown() {
    let unknown_types = ["spike", "refactor", "hotfix", "incident"];

    for wire_value in unknown_types {
        let json = format!(
            r#"{{"id":"test-{v}","title":"Test","status":"open","priority":0,"issue_type":"{v}","created_at":"2026-08-02T00:00:00Z","updated_at":"2026-08-02T00:00:00Z"}}"#,
            v = wire_value
        );

        let bead: Bead = serde_json::from_str(&json).unwrap_or_else(|_| {
            panic!(
                "Unrecognized issue type '{}' must deserialize as Unknown",
                wire_value
            )
        });

        assert_eq!(
            bead.issue_type,
            BeadType::Unknown,
            "Unrecognized issue type '{}' should become Unknown",
            wire_value
        );
    }
}

/// Extra/unknown keys in br line are tolerated (serde default behavior).
#[test]
fn extra_keys_in_bead_line_are_tolerated() {
    let with_extra = r#"{
        "id":"test-extra",
        "title":"Test",
        "status":"open",
        "priority":0,
        "issue_type":"task",
        "created_at":"2026-08-02T00:00:00Z",
        "updated_at":"2026-08-02T00:00:00Z",
        "some_unknown_field":"ignored",
        "another_unknown_key":123,
        "nested_unknown":{"foo":"bar"}
    }"#;

    let bead: Bead = serde_json::from_str(with_extra)
        .expect("Bead line with extra unknown keys must deserialize");

    assert_eq!(bead.id, "test-extra");
    assert_eq!(bead.status, BeadStatus::Open);
    assert_eq!(bead.issue_type, BeadType::Task);
}

/// Bead line with null description (explicit null) deserializes with None.
#[test]
fn bead_line_with_null_description_deserializes() {
    let with_null_desc = r#"{
        "id":"test-null",
        "title":"Test",
        "description":null,
        "status":"open",
        "priority":0,
        "issue_type":"task",
        "created_at":"2026-08-02T00:00:00Z",
        "updated_at":"2026-08-02T00:00:00Z"
    }"#;

    let bead: Bead = serde_json::from_str(with_null_desc)
        .expect("Bead line with null description must deserialize");

    assert!(bead.description.is_none());
}
