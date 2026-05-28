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

mod integration_harness;

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
        let actual = serialized.get(key).unwrap_or_else(|| {
            panic!(
                "CreateDraftResponse missing field '{}' (fixture declares it)",
                key
            )
        });
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
            last_activity_at: fixture_stitch["last_activity_at"]
                .as_str()
                .unwrap()
                .to_string(),
            participants: fixture_stitch["participants"].clone(),
            total_cost_usd: None,
            total_tokens: None,
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
            first_message_ts: fixture_cost["first_message_ts"]
                .as_str()
                .map(|s| s.to_string()),
            last_message_ts: fixture_cost["last_message_ts"]
                .as_str()
                .map(|s| s.to_string()),
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
// WebSocket events — daemon serializes events matching fixture schema
// ---------------------------------------------------------------------------

/// Daemon serializes `init` event with subscriptions array.
///
/// Fails if the init event structure changes.
#[test]
fn test_ws_event_init_serializes_fixture_shape() {
    use hoop_daemon::ws::WsEvent;

    let fixture = load_fixture("ws_events/init.json");

    let subs = vec!["global".to_string(), "project:test-project".to_string()];
    let event = WsEvent::init(subs);

    let serialized = serde_json::to_value(&event).unwrap();

    assert_eq!(serialized["type"], fixture["type"]);
    assert!(
        serialized.get("subscriptions").is_some(),
        "init event must have 'subscriptions'"
    );
    assert!(
        serialized["subscriptions"].is_array(),
        "subscriptions must be an array"
    );
}

/// Daemon serializes `worker_update` event matching fixture.
#[test]
fn test_ws_event_worker_update_serializes_fixture_shape() {
    use hoop_daemon::ws::{WorkerData, WorkerDisplayState, WsEvent};
    use hoop_daemon::heartbeats::WorkerLiveness;

    let fixture = load_fixture("ws_events/worker_update.json");

    let worker = WorkerData {
        worker: "test-worker".to_string(),
        state: WorkerDisplayState::Executing {
            bead: "bead-123".to_string(),
            adapter: "claude-opus-4-7".to_string(),
            model: None,
        },
        liveness: WorkerLiveness::Live,
        last_heartbeat: fixture["worker"]["last_heartbeat"].as_str().unwrap().parse().unwrap(),
        heartbeat_age_secs: 0,
    };

    let event = WsEvent {
        event_type: "worker_update".to_string(),
        worker: Some(worker),
        workers: None,
        beads: None,
        conversations: None,
        conversation: None,
        streaming: None,
        projects: None,
        config_status: None,
        capacity: None,
        bead_event: None,
        bead_events: None,
        bead_created_by_hoop: None,
        stitch_created: None,
        draft_update: None,
        collision_alert: None,
        morning_brief: None,
        stuck_alert: None,
        agent_session: None,
        spawn_ack_alert: None,
        pattern_saved_query_synced: None,
        saturation_alert: None,
        cost_anomaly_alert: None,
        subscriptions: None,
    };
    let serialized = serde_json::to_value(&event).unwrap();

    assert_eq!(serialized["type"], fixture["type"]);
    assert!(
        serialized.get("worker").is_some(),
        "worker_update must have 'worker'"
    );
    assert_eq!(
        serialized["worker"]["worker"],
        fixture["worker"]["worker"]
    );
}

/// Daemon serializes `workers_snapshot` event matching fixture.
#[test]
fn test_ws_event_workers_snapshot_serializes_fixture_shape() {
    use hoop_daemon::ws::{WorkerData, WorkerDisplayState, WsEvent};
    use hoop_daemon::heartbeats::WorkerLiveness;

    let fixture = load_fixture("ws_events/workers_snapshot.json");

    let workers = vec![
        WorkerData {
            worker: "test-worker".to_string(),
            state: WorkerDisplayState::Executing {
                bead: "bead-123".to_string(),
                adapter: "claude-opus-4-7".to_string(),
                model: None,
            },
            liveness: WorkerLiveness::Live,
            last_heartbeat: "2026-04-26T10:00:00Z".parse().unwrap(),
            heartbeat_age_secs: 0,
        },
    ];

    let event = WsEvent::workers_snapshot(workers);
    let serialized = serde_json::to_value(&event).unwrap();

    assert_eq!(serialized["type"], fixture["type"]);
    assert!(
        serialized.get("workers").is_some(),
        "workers_snapshot must have 'workers'"
    );
}

/// Daemon serializes `beads_snapshot` event matching fixture.
#[test]
fn test_ws_event_beads_snapshot_serializes_fixture_shape() {
    use hoop_daemon::ws::{BeadData, WsEvent};

    let fixture = load_fixture("ws_events/beads_snapshot.json");

    let beads = vec![BeadData {
        id: "bead-123".to_string(),
        title: "Fix authentication timeout".to_string(),
        status: "open".to_string(),
        priority: 5,
        issue_type: "task".to_string(),
        created_at: "2026-04-26T09:00:00Z".to_string(),
        updated_at: "2026-04-26T10:00:00Z".to_string(),
        created_by: "os:jedarden".to_string(),
        dependencies: vec![],
    }];

    let event = WsEvent::beads_snapshot(beads);
    let serialized = serde_json::to_value(&event).unwrap();

    assert_eq!(serialized["type"], fixture["type"]);
    assert!(
        serialized.get("beads").is_some(),
        "beads_snapshot must have 'beads'"
    );
}

/// Daemon serializes `config_status` event matching fixture.
#[test]
fn test_ws_event_config_status_serializes_fixture_shape() {
    use hoop_daemon::ws::{ConfigStatusData, WsEvent};

    let fixture = load_fixture("ws_events/config_status.json");

    let status = ConfigStatusData {
        valid: true,
        error: None,
        restart_required: None,
    };

    let event = WsEvent::config_status(status);
    let serialized = serde_json::to_value(&event).unwrap();

    assert_eq!(serialized["type"], fixture["type"]);
    assert!(
        serialized.get("config_status").is_some(),
        "config_status must have 'config_status'"
    );
}

/// Daemon serializes `stitch_created` event matching fixture.
#[test]
fn test_ws_event_stitch_created_serializes_fixture_shape() {
    use hoop_daemon::ws::{StitchCreatedData, WsEvent};

    let fixture = load_fixture("ws_events/stitch_created.json");

    let data = StitchCreatedData {
        bead_id: "bead-123".to_string(),
        title: "Fix authentication timeout".to_string(),
        project: "test-project".to_string(),
        stitch_id: Some("stitch-abc456".to_string()),
        source: "operator".to_string(),
        actor: "os:jedarden".to_string(),
        created_at: "2026-04-26T10:00:00Z".to_string(),
    };

    let event = WsEvent::stitch_created(data);
    let serialized = serde_json::to_value(&event).unwrap();

    assert_eq!(serialized["type"], fixture["type"]);
    assert!(
        serialized.get("stitch_created").is_some(),
        "stitch_created must have 'stitch_created'"
    );
}

/// Daemon serializes `bead_created_by_hoop` event matching fixture.
#[test]
fn test_ws_event_bead_created_by_hoop_serializes_fixture_shape() {
    use hoop_daemon::ws::{BeadCreatedByHoopData, WsEvent};

    let fixture = load_fixture("ws_events/bead_created_by_hoop.json");

    let data = BeadCreatedByHoopData {
        project: "test-project".to_string(),
        bead_id: "bead-789".to_string(),
        actor: "hoop".to_string(),
        source: "hoop".to_string(),
        ts: "2026-04-26T10:00:00Z".to_string(),
    };

    let event = WsEvent::bead_created_by_hoop(data);
    let serialized = serde_json::to_value(&event).unwrap();

    assert_eq!(serialized["type"], fixture["type"]);
    assert!(
        serialized.get("bead_created_by_hoop").is_some(),
        "bead_created_by_hoop must have 'bead_created_by_hoop'"
    );
}

/// Daemon serializes `draft_update` event matching fixture.
#[test]
fn test_ws_event_draft_update_serializes_fixture_shape() {
    use hoop_daemon::ws::{DraftUpdateData, WsEvent};

    let fixture = load_fixture("ws_events/draft_update.json");

    let data = DraftUpdateData {
        draft_id: "draft-abc123".to_string(),
        project: "test-project".to_string(),
        title: "Investigate API latency".to_string(),
        kind: "task".to_string(),
        status: "pending".to_string(),
        action: "created".to_string(),
        actor: "agent".to_string(),
        created_by: "claude-code".to_string(),
        version: 1,
        rejection_reason: None,
    };

    let event = WsEvent::draft_update(data);
    let serialized = serde_json::to_value(&event).unwrap();

    assert_eq!(serialized["type"], fixture["type"]);
    assert!(
        serialized.get("draft_update").is_some(),
        "draft_update must have 'draft_update'"
    );
}

/// Daemon serializes `collision_alert` event matching fixture.
#[test]
fn test_ws_event_collision_alert_serializes_fixture_shape() {
    use hoop_daemon::ws::{CollisionAlertData, WsEvent};

    let fixture = load_fixture("ws_events/collision_alert.json");

    let data = CollisionAlertData {
        alert_id: "collision-001".to_string(),
        detected_at: "2026-04-26T10:00:00Z".to_string(),
        worker_a: "worker-alpha".to_string(),
        bead_a: "bead-001".to_string(),
        worker_b: "worker-bravo".to_string(),
        bead_b: "bead-002".to_string(),
        overlapping_files: vec!["src/auth.rs".to_string(), "src/auth/mod.rs".to_string()],
    };

    let event = WsEvent::collision_alert_event(data);
    let serialized = serde_json::to_value(&event).unwrap();

    assert_eq!(serialized["type"], fixture["type"]);
    assert!(
        serialized.get("collision_alert").is_some(),
        "collision_alert must have 'collision_alert'"
    );
}

/// Daemon serializes `morning_brief` event matching fixture.
#[test]
fn test_ws_event_morning_brief_serializes_fixture_shape() {
    use hoop_daemon::ws::{MorningBriefData, WsEvent};

    let fixture = load_fixture("ws_events/morning_brief.json");

    let data = MorningBriefData {
        id: "brief-001".to_string(),
        headline: "3 high-priority drafts awaiting review".to_string(),
        generated_at: "2026-04-26T08:00:00Z".to_string(),
        draft_count: 3,
        status: "active".to_string(),
    };

    let event = WsEvent::morning_brief(data);
    let serialized = serde_json::to_value(&event).unwrap();

    assert_eq!(serialized["type"], fixture["type"]);
    assert!(
        serialized.get("morning_brief").is_some(),
        "morning_brief must have 'morning_brief'"
    );
}

/// Daemon serializes `projects_snapshot` event matching fixture.
#[test]
fn test_ws_event_projects_snapshot_serializes_fixture_shape() {
    use hoop_daemon::ws::{ProjectCardData, WsEvent};

    let fixture = load_fixture("ws_events/projects_snapshot.json");

    let projects = vec![ProjectCardData {
        name: "test-project".to_string(),
        label: "Test Project".to_string(),
        color: "#3B82F6".to_string(),
        path: "/home/coding/test-project".to_string(),
        degraded: false,
        runtime_state: Some("healthy".to_string()),
        runtime_error: None,
        bead_count: 5,
        worker_count: 2,
        active_stitch_count: 1,
        cost_today: 0.45,
        stuck_count: 0,
        last_activity: Some("2026-04-26T10:00:00Z".to_string()),
    }];

    let event = WsEvent::projects_snapshot(projects);
    let serialized = serde_json::to_value(&event).unwrap();

    assert_eq!(serialized["type"], fixture["type"]);
    assert!(
        serialized.get("projects").is_some(),
        "projects_snapshot must have 'projects'"
    );
}

/// Daemon round-trips WsEvent through JSON (fixture validates deserialization).
#[test]
fn test_ws_event_round_trip_from_fixture() {
    use hoop_daemon::ws::WsEvent;

    let fixtures = [
        "ws_events/init.json",
        "ws_events/worker_update.json",
        "ws_events/beads_snapshot.json",
        "ws_events/config_status.json",
        "ws_events/stitch_created.json",
        "ws_events/bead_created_by_hoop.json",
        "ws_events/draft_update.json",
        "ws_events/collision_alert.json",
        "ws_events/morning_brief.json",
        "ws_events/projects_snapshot.json",
    ];

    for path in &fixtures {
        let fixture = load_fixture(path);

        // Verify fixture can be deserialized as WsEvent
        let event: WsEvent = serde_json::from_value(fixture.clone())
            .unwrap_or_else(|e| panic!("fixture {} must deserialize as WsEvent: {}", path, e));

        // Verify re-serialized event matches fixture structure
        let serialized = serde_json::to_value(&event).unwrap();
        assert_eq!(
            serialized["type"], fixture["type"],
            "fixture {} event_type must round-trip",
            path
        );
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
        "ws_events/init.json",
        "ws_events/worker_update.json",
        "ws_events/workers_snapshot.json",
        "ws_events/beads_snapshot.json",
        "ws_events/conversations_snapshot.json",
        "ws_events/config_status.json",
        "ws_events/config_status_invalid.json",
        "ws_events/stitch_created.json",
        "ws_events/bead_created_by_hoop.json",
        "ws_events/draft_update.json",
        "ws_events/collision_alert.json",
        "ws_events/morning_brief.json",
        "ws_events/projects_snapshot.json",
        "ws_events/capacity_snapshot.json",
        "ws_events/agent_session.json",
        "ws_events/stuck_alert.json",
        "ws_events/bead_event.json",
        "ws_events/bead_events.json",
        "ws_events/spawn_ack_alert.json",
    ];
    for path in &fixtures {
        let val = load_fixture(path);
        assert!(val.is_object(), "fixture {} must be a JSON object", path);
    }
}
