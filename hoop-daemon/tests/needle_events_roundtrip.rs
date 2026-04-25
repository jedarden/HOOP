//! Integration test: NEEDLE event and heartbeat round-trips against testrepo fixtures.
//!
//! Validates acceptance criteria from hoop-ttb.2.23:
//! - Event shapes match HOOP's parser expectations (documented schema, §Hook2)
//! - All event types parse correctly: claim, dispatch, complete, fail, release,
//!   timeout, crash, close, update
//! - All heartbeat states parse correctly: executing, idle, knot
//! - BeadEventData::from_event() produces correct output for all event types
//!
//! The fixture files at testrepo/.beads/ serve as the canonical reference
//! for what NEEDLE workers must emit.

use hoop_daemon::events::{BeadEventData, NeedleEvent};
use hoop_daemon::heartbeats::HeartbeatMonitor;
use hoop_daemon::parse_jsonl_safe::LineSource;
use std::fs;
use std::path::PathBuf;

// ── Fixture paths ────────────────────────────────────────────────────────────

fn testrepo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root is parent of hoop-daemon/")
        .join("testrepo")
}

fn events_fixture_path() -> PathBuf {
    testrepo_root().join(".beads/events.jsonl")
}

fn heartbeats_fixture_path() -> PathBuf {
    testrepo_root().join(".beads/heartbeats.jsonl")
}

// ── Helper: parse an event line ───────────────────────────────────────────────

fn parse_event(line: &str) -> NeedleEvent {
    serde_json::from_str(line)
        .unwrap_or_else(|e| panic!("Failed to parse event line: {e}\n  Line: {line}"))
}

// ── Fixture sanity check ─────────────────────────────────────────────────────

#[test]
fn fixture_files_exist() {
    assert!(
        events_fixture_path().exists(),
        "testrepo/.beads/events.jsonl must exist — it is the canonical NEEDLE event schema reference"
    );
    assert!(
        heartbeats_fixture_path().exists(),
        "testrepo/.beads/heartbeats.jsonl must exist — it is the canonical NEEDLE heartbeat schema reference"
    );
}

// ── Event-type coverage ──────────────────────────────────────────────────────

/// All expected event types must appear in the fixture file.
#[test]
fn events_fixture_covers_all_event_types() {
    let content = fs::read_to_string(events_fixture_path())
        .expect("testrepo/.beads/events.jsonl must be readable");

    let required = [
        "claim", "dispatch", "complete", "fail", "release",
        "timeout", "crash", "close", "update",
    ];

    for event_type in required {
        let found = content.lines().any(|l| {
            l.contains(&format!(r#""event":"{event_type}""#))
        });
        assert!(
            found,
            "fixture must contain at least one '{event_type}' event — add one to testrepo/.beads/events.jsonl"
        );
    }
}

/// All heartbeat states must appear in the fixture file.
#[test]
fn heartbeats_fixture_covers_all_states() {
    let content = fs::read_to_string(heartbeats_fixture_path())
        .expect("testrepo/.beads/heartbeats.jsonl must be readable");

    for state in ["idle", "executing", "knot"] {
        let found = content.lines().any(|l| {
            l.contains(&format!(r#""state":"{state}""#))
        });
        assert!(
            found,
            "fixture must contain at least one '{state}' heartbeat — add one to testrepo/.beads/heartbeats.jsonl"
        );
    }
}

// ── Event-shape validation ───────────────────────────────────────────────────

#[test]
fn claim_event_parses_with_strand() {
    let content = fs::read_to_string(events_fixture_path()).unwrap();
    let line = content.lines()
        .find(|l| l.contains(r#""event":"claim""#))
        .expect("fixture must have a claim event");

    match parse_event(line) {
        NeedleEvent::Claim { worker, bead, strand, .. } => {
            assert!(!worker.is_empty(), "claim: worker must be non-empty");
            assert!(bead.starts_with("bd-"), "claim: bead must start with 'bd-'");
            assert!(strand.is_some(), "claim in fixture should include strand field");
        }
        other => panic!("Expected Claim, got {other:?}"),
    }
}

#[test]
fn dispatch_event_parses_adapter_and_model() {
    let content = fs::read_to_string(events_fixture_path()).unwrap();
    let line = content.lines()
        .find(|l| l.contains(r#""event":"dispatch""#))
        .expect("fixture must have a dispatch event");

    match parse_event(line) {
        NeedleEvent::Dispatch { worker, bead, adapter, model, .. } => {
            assert!(!worker.is_empty(), "dispatch: worker must be non-empty");
            assert!(bead.starts_with("bd-"), "dispatch: bead must start with 'bd-'");
            assert!(adapter.is_some(), "dispatch in fixture should include adapter");
            assert!(model.is_some(), "dispatch in fixture should include model");
        }
        other => panic!("Expected Dispatch, got {other:?}"),
    }
}

#[test]
fn complete_event_parses_outcome_and_duration() {
    let content = fs::read_to_string(events_fixture_path()).unwrap();
    let line = content.lines()
        .find(|l| l.contains(r#""event":"complete""#))
        .expect("fixture must have a complete event");

    match parse_event(line) {
        NeedleEvent::Complete { worker, bead, outcome, duration_ms, exit_code, .. } => {
            assert!(!worker.is_empty(), "complete: worker must be non-empty");
            assert!(bead.starts_with("bd-"), "complete: bead must start with 'bd-'");
            assert!(outcome.is_some(), "complete in fixture should include outcome");
            assert!(duration_ms.is_some(), "complete in fixture should include duration_ms");
            assert!(exit_code.is_some(), "complete in fixture should include exit_code");
        }
        other => panic!("Expected Complete, got {other:?}"),
    }
}

#[test]
fn fail_event_parses_error_and_duration() {
    let content = fs::read_to_string(events_fixture_path()).unwrap();
    let line = content.lines()
        .find(|l| l.contains(r#""event":"fail""#))
        .expect("fixture must have a fail event");

    match parse_event(line) {
        NeedleEvent::Fail { worker, bead, error, duration_ms, .. } => {
            assert!(!worker.is_empty(), "fail: worker must be non-empty");
            assert!(bead.starts_with("bd-"), "fail: bead must start with 'bd-'");
            assert!(error.is_some(), "fail in fixture should include error");
            assert!(duration_ms.is_some(), "fail in fixture should include duration_ms");
        }
        other => panic!("Expected Fail, got {other:?}"),
    }
}

#[test]
fn release_event_parses() {
    let content = fs::read_to_string(events_fixture_path()).unwrap();
    let line = content.lines()
        .find(|l| l.contains(r#""event":"release""#))
        .expect("fixture must have a release event");

    match parse_event(line) {
        NeedleEvent::Release { worker, bead, .. } => {
            assert!(!worker.is_empty(), "release: worker must be non-empty");
            assert!(bead.starts_with("bd-"), "release: bead must start with 'bd-'");
        }
        other => panic!("Expected Release, got {other:?}"),
    }
}

#[test]
fn timeout_event_parses() {
    let content = fs::read_to_string(events_fixture_path()).unwrap();
    let line = content.lines()
        .find(|l| l.contains(r#""event":"timeout""#))
        .expect("fixture must have a timeout event");

    match parse_event(line) {
        NeedleEvent::Timeout { worker, bead, .. } => {
            assert!(!worker.is_empty(), "timeout: worker must be non-empty");
            assert!(bead.starts_with("bd-"), "timeout: bead must start with 'bd-'");
        }
        other => panic!("Expected Timeout, got {other:?}"),
    }
}

#[test]
fn crash_event_parses_exit_code() {
    let content = fs::read_to_string(events_fixture_path()).unwrap();
    let line = content.lines()
        .find(|l| l.contains(r#""event":"crash""#))
        .expect("fixture must have a crash event");

    match parse_event(line) {
        NeedleEvent::Crash { worker, bead, exit_code, .. } => {
            assert!(!worker.is_empty(), "crash: worker must be non-empty");
            assert!(bead.starts_with("bd-"), "crash: bead must start with 'bd-'");
            assert!(exit_code.is_some(), "crash in fixture should include exit_code");
        }
        other => panic!("Expected Crash, got {other:?}"),
    }
}

/// Every line in the fixture must parse without returning Unknown.
/// Unknown means the event type was not recognized by HOOP's parser.
#[test]
fn all_fixture_events_are_recognized() {
    let content = fs::read_to_string(events_fixture_path()).unwrap();
    for (i, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let event = parse_event(line);
        assert!(
            !matches!(event, NeedleEvent::Unknown),
            "line {} parsed as Unknown — add the event type to NeedleEvent: {line}",
            i + 1
        );
    }
}

// ── BeadEventData::from_event() round-trip ───────────────────────────────────

/// Every recognized event in the fixture must produce a Some(BeadEventData).
#[test]
fn from_event_produces_some_for_all_fixture_events() {
    let content = fs::read_to_string(events_fixture_path()).unwrap();
    for (i, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let event = parse_event(line);
        if matches!(event, NeedleEvent::Unknown) {
            continue;
        }
        let data = BeadEventData::from_event(&event);
        assert!(
            data.is_some(),
            "from_event returned None for recognized event on line {}: {line}",
            i + 1
        );
        let data = data.unwrap();
        assert!(!data.bead_id.is_empty(), "BeadEventData must have bead_id (line {})", i + 1);
        assert!(!data.worker.is_empty(), "BeadEventData must have worker (line {})", i + 1);
        assert!(!data.event_type.is_empty(), "BeadEventData must have event_type (line {})", i + 1);
    }
}

/// claim → BeadEventData carries the strand field.
#[test]
fn from_event_claim_carries_strand() {
    let event = NeedleEvent::Claim {
        ts: "2026-04-21T18:42:10Z".to_string(),
        worker: "alpha".to_string(),
        bead: "bd-abc123".to_string(),
        strand: Some("pluck".to_string()),
    };
    let data = BeadEventData::from_event(&event).unwrap();
    assert_eq!(data.event_type, "claim");
    assert_eq!(data.strand, Some("pluck".to_string()));
    assert_eq!(data.bead_id, "bd-abc123");
}

/// dispatch → BeadEventData carries adapter + model.
#[test]
fn from_event_dispatch_carries_adapter_model() {
    let event = NeedleEvent::Dispatch {
        ts: "2026-04-21T18:47:33Z".to_string(),
        worker: "alpha".to_string(),
        bead: "bd-abc123".to_string(),
        adapter: Some("claude".to_string()),
        model: Some("claude-opus-4-6".to_string()),
    };
    let data = BeadEventData::from_event(&event).unwrap();
    assert_eq!(data.event_type, "dispatch");
    assert_eq!(data.adapter, Some("claude".to_string()));
    assert_eq!(data.model, Some("claude-opus-4-6".to_string()));
}

/// complete → BeadEventData carries outcome, duration_ms, exit_code.
#[test]
fn from_event_complete_carries_timing_fields() {
    let event = NeedleEvent::Complete {
        ts: "2026-04-21T18:52:01Z".to_string(),
        worker: "alpha".to_string(),
        bead: "bd-abc123".to_string(),
        outcome: Some("success".to_string()),
        duration_ms: Some(287104),
        exit_code: Some(0),
    };
    let data = BeadEventData::from_event(&event).unwrap();
    assert_eq!(data.event_type, "complete");
    assert_eq!(data.outcome, Some("success".to_string()));
    assert_eq!(data.duration_ms, Some(287104));
    assert_eq!(data.exit_code, Some(0));
}

/// fail → BeadEventData carries error + duration_ms.
#[test]
fn from_event_fail_carries_error() {
    let event = NeedleEvent::Fail {
        ts: "2026-04-21T18:53:00Z".to_string(),
        worker: "bravo".to_string(),
        bead: "bd-def456".to_string(),
        error: Some("context limit exceeded".to_string()),
        duration_ms: Some(90000),
    };
    let data = BeadEventData::from_event(&event).unwrap();
    assert_eq!(data.event_type, "fail");
    assert_eq!(data.error, Some("context limit exceeded".to_string()));
    assert_eq!(data.duration_ms, Some(90000));
}

// ── Heartbeat-shape validation ───────────────────────────────────────────────

fn heartbeat_source(line_number: usize) -> LineSource {
    LineSource {
        tag: "heartbeats_test",
        file_path: heartbeats_fixture_path(),
        line_number,
    }
}

#[test]
fn heartbeat_executing_state_parses() {
    let content = fs::read_to_string(heartbeats_fixture_path()).unwrap();
    let line = content.lines()
        .find(|l| l.contains(r#""state":"executing""#))
        .expect("fixture must have an executing heartbeat");

    let hb = HeartbeatMonitor::parse_heartbeat_line(line, &heartbeat_source(1))
        .expect("executing heartbeat must parse successfully");

    assert!(!hb.worker.is_empty(), "heartbeat: worker must be non-empty");
    match hb.state {
        hoop_daemon::WorkerState::Executing { bead, pid, adapter } => {
            assert!(bead.starts_with("bd-"), "executing: bead must start with 'bd-'");
            assert!(pid > 0, "executing: pid must be positive");
            assert!(!adapter.is_empty(), "executing: adapter must be non-empty");
        }
        other => panic!("Expected Executing state, got {other:?}"),
    }
}

#[test]
fn heartbeat_idle_state_parses() {
    let content = fs::read_to_string(heartbeats_fixture_path()).unwrap();
    let line = content.lines()
        .find(|l| l.contains(r#""state":"idle""#))
        .expect("fixture must have an idle heartbeat");

    let hb = HeartbeatMonitor::parse_heartbeat_line(line, &heartbeat_source(1))
        .expect("idle heartbeat must parse successfully");

    assert!(!hb.worker.is_empty(), "heartbeat: worker must be non-empty");
    assert!(matches!(hb.state, hoop_daemon::WorkerState::Idle { .. }));
}

#[test]
fn heartbeat_knot_state_parses() {
    let content = fs::read_to_string(heartbeats_fixture_path()).unwrap();
    let line = content.lines()
        .find(|l| l.contains(r#""state":"knot""#))
        .expect("fixture must have a knot heartbeat");

    let hb = HeartbeatMonitor::parse_heartbeat_line(line, &heartbeat_source(1))
        .expect("knot heartbeat must parse successfully");

    assert!(!hb.worker.is_empty(), "heartbeat: worker must be non-empty");
    match hb.state {
        hoop_daemon::WorkerState::Knot { reason } => {
            assert!(!reason.is_empty(), "knot: reason must be non-empty");
        }
        other => panic!("Expected Knot state, got {other:?}"),
    }
}

/// Every line in the heartbeats fixture must parse successfully.
#[test]
fn all_fixture_heartbeats_parse() {
    let content = fs::read_to_string(heartbeats_fixture_path()).unwrap();
    for (i, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let source = heartbeat_source(i + 1);
        let hb = HeartbeatMonitor::parse_heartbeat_line(line, &source);
        assert!(
            hb.is_some(),
            "heartbeat line {} failed to parse: {line}",
            i + 1
        );
    }
}

/// Heartbeat timestamps must parse as valid RFC3339.
#[test]
fn heartbeat_timestamps_are_valid_rfc3339() {
    let content = fs::read_to_string(heartbeats_fixture_path()).unwrap();
    for (i, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let source = heartbeat_source(i + 1);
        let hb = HeartbeatMonitor::parse_heartbeat_line(line, &source);
        if let Some(hb) = hb {
            // ts field is chrono::DateTime<Utc> — if it parsed, it's valid
            assert!(
                hb.ts.timestamp() > 0,
                "heartbeat line {} has invalid/epoch timestamp",
                i + 1
            );
        }
    }
}

// ── Heartbeat interval contract ───────────────────────────────────────────────

/// Consecutive heartbeats for the same worker+bead must be ~10s apart.
///
/// This validates the NEEDLE heartbeat interval contract (§Hook3):
/// each worker emits a heartbeat every ~10s while executing.
/// We only check heartbeats for the same bead to avoid false positives
/// across bead boundaries (where gaps are expected and unlimited).
#[test]
fn consecutive_heartbeats_per_bead_are_roughly_10s_apart() {
    let content = fs::read_to_string(heartbeats_fixture_path()).unwrap();

    // Collect all executing heartbeats, keyed by (worker, bead)
    let mut per_bead: std::collections::HashMap<(String, String), Vec<chrono::DateTime<chrono::Utc>>> =
        std::collections::HashMap::new();

    for (i, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let source = heartbeat_source(i + 1);
        if let Some(hb) = HeartbeatMonitor::parse_heartbeat_line(line, &source) {
            if let hoop_daemon::WorkerState::Executing { ref bead, .. } = hb.state {
                per_bead
                    .entry((hb.worker.clone(), bead.clone()))
                    .or_default()
                    .push(hb.ts);
            }
        }
    }

    // Check each (worker, bead) pair that has >= 2 heartbeats
    for ((worker, bead), mut timestamps) in per_bead {
        if timestamps.len() < 2 {
            continue;
        }
        timestamps.sort();
        for window in timestamps.windows(2) {
            let gap = window[1].signed_duration_since(window[0]).num_seconds();
            assert!(
                gap >= 5 && gap <= 30,
                "worker={worker}, bead={bead}: consecutive heartbeats must be ~10s apart (got {gap}s)"
            );
        }
    }
}
