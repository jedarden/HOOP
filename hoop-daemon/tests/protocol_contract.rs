//! Protocol contract tests: daemon ↔ hoop-mcp HTTP and socket protocol
//!
//! Fixture-driven round-trip tests. The shared fixture files live at
//! `tests/fixtures/protocol/` (workspace root) and are also loaded by
//! `hoop-mcp/tests/protocol_contract.rs`. Drift on either side breaks CI.
//!
//! Protocol pairs covered:
//! - POST /api/drafts request  (daemon receives from hoop-mcp)
//! - POST /api/drafts response (daemon sends to hoop-mcp)
//! - GET  /api/stitches/{id}   (daemon sends to hoop-mcp)
//! - ControlRequest / ControlResponse (daemon ↔ hoop-cli over control.sock)
//!
//! §9.3 / §13 MCP socket protocol contract.

use std::{fs, path::Path};

/// Load a fixture JSON file relative to the workspace `tests/fixtures/protocol/` directory.
fn load_fixture(relative: &str) -> serde_json::Value {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(manifest_dir)
        .parent()
        .expect("workspace root")
        .join("tests/fixtures/protocol")
        .join(relative);
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("fixture file missing: {}", path.display()));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("invalid JSON in fixture {}: {}", path.display(), e))
}

// ---------------------------------------------------------------------------
// POST /api/drafts — request the daemon receives from hoop-mcp
// ---------------------------------------------------------------------------

/// Daemon must deserialize the exact JSON body hoop-mcp sends.
///
/// Fails if `CreateDraftRequest` drops or renames a field without updating the
/// fixture. The matching test in hoop-mcp validates the other direction.
#[test]
fn test_create_draft_request_daemon_parses_fixture() {
    let fixture = load_fixture("daemon_http/create_draft_request.json");

    let req: hoop_daemon::api_draft_queue::CreateDraftRequest =
        serde_json::from_value(fixture.clone())
            .expect("CreateDraftRequest must deserialize from fixture (daemon side)");

    assert_eq!(req.project, fixture["project"].as_str().unwrap());
    assert_eq!(req.title, fixture["title"].as_str().unwrap());
    assert_eq!(req.kind, fixture["kind"].as_str().unwrap());
    assert_eq!(req.source, fixture["source"].as_str().unwrap());
    assert_eq!(
        req.description,
        fixture["description"].as_str().map(|s| s.to_string()),
    );
    assert_eq!(req.priority, fixture["priority"].as_i64());
}

// ---------------------------------------------------------------------------
// POST /api/drafts — response the daemon sends to hoop-mcp
// ---------------------------------------------------------------------------

/// Daemon serializes `CreateDraftResponse` with exactly the field names the
/// fixture declares.
///
/// Fails if the daemon renames a field (e.g. `draft_id` → `id`) without
/// updating the fixture. hoop-mcp reads these field names by key.
#[test]
fn test_create_draft_response_daemon_serializes_fixture_shape() {
    let fixture = load_fixture("daemon_http/create_draft_response.json");

    let resp = hoop_daemon::api_draft_queue::CreateDraftResponse {
        draft_id: fixture["draft_id"].as_str().unwrap().to_string(),
        status: fixture["status"].as_str().unwrap().to_string(),
    };

    let serialized = serde_json::to_value(&resp).unwrap();

    for (key, expected) in fixture.as_object().unwrap() {
        let actual = serialized
            .get(key)
            .unwrap_or_else(|| panic!("CreateDraftResponse missing field '{}' (fixture declares it)", key));
        assert_eq!(actual, expected, "field '{}' value mismatch", key);
    }
}

// ---------------------------------------------------------------------------
// GET /api/stitches/{id} — response the daemon sends to hoop-mcp
// ---------------------------------------------------------------------------

/// Daemon serializes `AggregatedStitchResponse` with every top-level field
/// the fixture declares.
///
/// Fails if the daemon renames or drops a field that hoop-mcp accesses (e.g.
/// `messages` is read by the redaction pass; `stitch.id` is read for display).
#[test]
fn test_read_stitch_response_daemon_serializes_fixture_shape() {
    let fixture = load_fixture("daemon_http/read_stitch_response.json");
    let fixture_stitch = &fixture["stitch"];
    let fixture_msg = &fixture["messages"][0];
    let fixture_cost = &fixture["cost_duration"];

    let resp = hoop_daemon::api_stitch_read::AggregatedStitchResponse {
        stitch: hoop_daemon::api_stitch_read::StitchRow {
            id: fixture_stitch["id"].as_str().unwrap().to_string(),
            project: fixture_stitch["project"].as_str().unwrap().to_string(),
            kind: fixture_stitch["kind"].as_str().unwrap().to_string(),
            title: fixture_stitch["title"].as_str().unwrap().to_string(),
            created_by: fixture_stitch["created_by"].as_str().unwrap().to_string(),
            created_at: fixture_stitch["created_at"].as_str().unwrap().to_string(),
            last_activity_at: fixture_stitch["last_activity_at"].as_str().unwrap().to_string(),
            participants: fixture_stitch["participants"].clone(),
        },
        messages: vec![hoop_daemon::api_stitch_read::StitchMessage {
            id: fixture_msg["id"].as_str().unwrap().to_string(),
            ts: fixture_msg["ts"].as_str().unwrap().to_string(),
            role: fixture_msg["role"].as_str().unwrap().to_string(),
            content: fixture_msg["content"].as_str().unwrap().to_string(),
            tokens: fixture_msg["tokens"].as_i64(),
        }],
        linked_beads: vec![],
        touched_files: vec![],
        cost_duration: hoop_daemon::api_stitch_read::CostDuration {
            total_tokens: fixture_cost["total_tokens"].as_i64().unwrap(),
            message_count: fixture_cost["message_count"].as_u64().unwrap() as usize,
            wall_clock: fixture_cost["wall_clock"].as_str().unwrap().to_string(),
            first_message_ts: fixture_cost["first_message_ts"].as_str().map(|s| s.to_string()),
            last_message_ts: fixture_cost["last_message_ts"].as_str().map(|s| s.to_string()),
        },
        link_graph: hoop_daemon::api_stitch_read::LinkGraph {
            outgoing: vec![],
            incoming: vec![],
        },
        elapsed_ms: None,
    };

    let serialized = serde_json::to_value(&resp).unwrap();

    // All top-level fixture keys must appear in the serialized output
    for key in fixture.as_object().unwrap().keys() {
        assert!(
            serialized.get(key).is_some(),
            "AggregatedStitchResponse must serialize '{}' (declared in fixture)",
            key
        );
    }

    // StitchRow sub-fields
    let serialized_stitch = &serialized["stitch"];
    for key in fixture_stitch.as_object().unwrap().keys() {
        assert!(
            serialized_stitch.get(key).is_some(),
            "StitchRow must serialize '{}' (declared in fixture)",
            key
        );
    }

    // StitchMessage sub-fields
    let serialized_msg = &serialized["messages"][0];
    for key in fixture_msg.as_object().unwrap().keys() {
        assert!(
            serialized_msg.get(key).is_some(),
            "StitchMessage must serialize '{}' (declared in fixture)",
            key
        );
    }

    // CostDuration sub-fields (skip null-valued fixture fields)
    let serialized_cost = &serialized["cost_duration"];
    for key in fixture_cost.as_object().unwrap().keys() {
        if fixture_cost[key].is_null() {
            continue; // skip_serializing_if may omit nulls
        }
        assert!(
            serialized_cost.get(key).is_some(),
            "CostDuration must serialize '{}' (declared in fixture)",
            key
        );
    }
}

// ---------------------------------------------------------------------------
// Control socket: ControlRequest / ControlResponse (daemon ↔ hoop-cli)
// ---------------------------------------------------------------------------

/// ControlRequest round-trips through JSON without data loss.
#[test]
fn test_control_request_status_round_trip() {
    use hoop_daemon::ControlRequest;

    let req = ControlRequest::Status {
        project: Some("test-project".to_string()),
    };
    let serialized = serde_json::to_string(&req).unwrap();
    let parsed: ControlRequest = serde_json::from_str(&serialized).unwrap();

    match parsed {
        ControlRequest::Status { project } => {
            assert_eq!(project, Some("test-project".to_string()));
        }
    }
}

/// ControlRequest without project filter round-trips.
#[test]
fn test_control_request_status_no_project_round_trip() {
    use hoop_daemon::ControlRequest;

    let req = ControlRequest::Status { project: None };
    let serialized = serde_json::to_string(&req).unwrap();
    let parsed: ControlRequest = serde_json::from_str(&serialized).unwrap();

    match parsed {
        ControlRequest::Status { project } => {
            assert!(project.is_none());
        }
    }
}

/// ControlResponse::Status round-trips through JSON without data loss.
#[test]
fn test_control_response_status_round_trip() {
    use hoop_daemon::{ControlResponse, ProjectStatus, StatusResponse};

    let resp = ControlResponse::Status(StatusResponse {
        daemon_running: true,
        uptime_secs: 3600,
        projects: vec![ProjectStatus {
            name: "test-project".to_string(),
            path: "/home/test/project".to_string(),
            active_beads: 3,
            workers: 2,
            runtime_state: Some("running".to_string()),
            runtime_error: None,
        }],
    });

    let serialized = serde_json::to_string(&resp).unwrap();
    let parsed: ControlResponse = serde_json::from_str(&serialized).unwrap();

    match parsed {
        ControlResponse::Status(status) => {
            assert!(status.daemon_running);
            assert_eq!(status.uptime_secs, 3600);
            assert_eq!(status.projects.len(), 1);
            assert_eq!(status.projects[0].name, "test-project");
            assert_eq!(status.projects[0].active_beads, 3);
            assert_eq!(status.projects[0].workers, 2);
            assert_eq!(
                status.projects[0].runtime_state,
                Some("running".to_string())
            );
            assert!(status.projects[0].runtime_error.is_none());
        }
        _ => panic!("expected ControlResponse::Status"),
    }
}

/// ControlResponse::Error round-trips through JSON.
#[test]
fn test_control_response_error_round_trip() {
    use hoop_daemon::ControlResponse;

    let resp = ControlResponse::Error {
        message: "daemon not running".to_string(),
    };

    let serialized = serde_json::to_string(&resp).unwrap();
    let parsed: ControlResponse = serde_json::from_str(&serialized).unwrap();

    match parsed {
        ControlResponse::Error { message } => {
            assert_eq!(message, "daemon not running");
        }
        _ => panic!("expected ControlResponse::Error"),
    }
}

// ---------------------------------------------------------------------------
// Fixture self-consistency: fixture files must be valid JSON and non-empty
// ---------------------------------------------------------------------------

#[test]
fn test_all_daemon_fixtures_are_valid_json() {
    let fixtures = [
        "daemon_http/create_draft_request.json",
        "daemon_http/create_draft_response.json",
        "daemon_http/read_stitch_response.json",
    ];
    for path in &fixtures {
        let val = load_fixture(path);
        assert!(
            val.is_object(),
            "fixture {} must be a JSON object",
            path
        );
    }
}
