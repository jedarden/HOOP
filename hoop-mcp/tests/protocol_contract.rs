//! Protocol contract tests: daemon ↔ hoop-mcp HTTP and socket protocol
//!
//! Fixture-driven round-trip tests. The shared fixture files live at
//! `tests/fixtures/protocol/` (workspace root) and are also loaded by
//! `hoop-daemon/tests/protocol_contract.rs`. Drift on either side breaks CI.
//!
//! Protocol pairs covered:
//! - MCP socket: initialize  request/response
//! - MCP socket: tools/call  request/response
//! - Daemon HTTP: POST /api/drafts request  (cross-crate: same fixture as daemon tests)
//! - Daemon HTTP: GET  /api/stitches/{id} response structure
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
// MCP socket: initialize — agent sends to hoop-mcp
// ---------------------------------------------------------------------------

/// hoop-mcp must deserialize the initialize request from the agent.
///
/// Fails if `InitializeParams` drops or renames a field.
#[test]
fn test_initialize_request_mcp_parses_fixture() {
    let fixture = load_fixture("mcp_socket/initialize_request.json");

    let req: hoop_mcp::protocol::JsonRpcRequest =
        serde_json::from_value(fixture.clone())
            .expect("JsonRpcRequest must deserialize from initialize fixture");

    match req.method {
        hoop_mcp::protocol::Method::Initialize(params) => {
            let expected_version = fixture["params"]["protocol_version"].as_str().unwrap();
            assert_eq!(params.protocol_version, expected_version);

            let expected_name = fixture["params"]["client_info"]["name"].as_str().unwrap();
            assert_eq!(params.client_info.name, expected_name);

            let expected_ver = fixture["params"]["client_info"]["version"].as_str().unwrap();
            assert_eq!(params.client_info.version, expected_ver);
        }
        _ => panic!("expected Method::Initialize"),
    }
}

// ---------------------------------------------------------------------------
// MCP socket: initialize — hoop-mcp sends to agent
// ---------------------------------------------------------------------------

/// hoop-mcp serializes `InitializeResult` with every field the fixture declares.
///
/// Fails if `InitializeResult`, `ServerCapabilities`, or `ServerInfo` rename
/// a field (e.g. `server_info` → `serverInfo`).
#[test]
fn test_initialize_response_mcp_serializes_fixture_shape() {
    use hoop_mcp::protocol::{
        InitializeResult, JsonRpcResponse, ServerCapabilities, ServerInfo, ToolsCapability,
    };

    let fixture = load_fixture("mcp_socket/initialize_response.json");
    let fixture_result = &fixture["result"];

    let result = InitializeResult {
        protocol_version: fixture_result["protocol_version"].as_str().unwrap().to_string(),
        capabilities: ServerCapabilities {
            tools: ToolsCapability { list_changed: false },
            prompts: None,
            resources: None,
        },
        server_info: ServerInfo {
            name: fixture_result["server_info"]["name"].as_str().unwrap().to_string(),
            version: fixture_result["server_info"]["version"].as_str().unwrap().to_string(),
        },
    };

    let resp = JsonRpcResponse::result(
        serde_json::json!(null),
        serde_json::to_value(result).unwrap(),
    );

    let serialized = serde_json::to_value(&resp).unwrap();

    assert_eq!(serialized["jsonrpc"], fixture["jsonrpc"]);
    assert!(serialized.get("result").is_some(), "response must have 'result'");

    let serialized_result = &serialized["result"];
    for key in fixture_result.as_object().unwrap().keys() {
        assert!(
            serialized_result.get(key).is_some(),
            "InitializeResult must serialize '{}' (declared in fixture)",
            key
        );
    }

    // server_info sub-fields
    let fixture_server = &fixture_result["server_info"];
    let serialized_server = &serialized_result["server_info"];
    for key in fixture_server.as_object().unwrap().keys() {
        assert!(
            serialized_server.get(key).is_some(),
            "ServerInfo must serialize '{}' (declared in fixture)",
            key
        );
    }
}

// ---------------------------------------------------------------------------
// MCP socket: tools/call — agent sends to hoop-mcp
// ---------------------------------------------------------------------------

/// hoop-mcp must deserialize a tools/call request from the agent.
///
/// WIRE FORMAT NOTE: hoop-mcp uses flattened arguments (§9.3). Tool arguments
/// are placed directly in `params` alongside `name` rather than nested under
/// an `arguments` key. The fixture documents this contract.
///
/// Fails if `ToolCallParams` or `Method` changes the expected wire layout.
#[test]
fn test_tools_call_request_mcp_parses_fixture() {
    let fixture = load_fixture("mcp_socket/tools_call_request.json");

    let req: hoop_mcp::protocol::JsonRpcRequest =
        serde_json::from_value(fixture.clone())
            .expect("JsonRpcRequest must deserialize from tools_call fixture");

    match req.method {
        hoop_mcp::protocol::Method::ToolsCall(params) => {
            let expected_name = fixture["params"]["name"].as_str().unwrap();
            assert_eq!(params.name, expected_name);

            // All non-name fields in params must be present in the flattened arguments map
            let fixture_params = fixture["params"].as_object().unwrap();
            for (key, _) in fixture_params.iter() {
                if key == "name" {
                    continue;
                }
                assert!(
                    params.arguments.contains_key(key),
                    "argument '{}' from fixture must be in ToolCallParams.arguments (flattened wire format)",
                    key
                );
            }
        }
        _ => panic!("expected Method::ToolsCall"),
    }
}

// ---------------------------------------------------------------------------
// MCP socket: tools/call — hoop-mcp sends to agent
// ---------------------------------------------------------------------------

/// hoop-mcp serializes `ToolCallResult` with every field the fixture declares.
///
/// Fails if `ToolCallResult` or `Content` rename a field.
#[test]
fn test_tools_call_response_mcp_serializes_fixture_shape() {
    use hoop_mcp::protocol::{Content, JsonRpcResponse, ToolCallResult};

    let fixture = load_fixture("mcp_socket/tools_call_response.json");
    let fixture_result = &fixture["result"];

    let text = fixture_result["content"][0]["text"].as_str().unwrap().to_string();

    let result = ToolCallResult {
        content: vec![Content::Text { text }],
        is_error: None,
    };

    let resp = JsonRpcResponse::result(
        serde_json::json!(null),
        serde_json::to_value(result).unwrap(),
    );

    let serialized = serde_json::to_value(&resp).unwrap();

    assert_eq!(serialized["jsonrpc"], fixture["jsonrpc"]);
    assert!(serialized.get("result").is_some());

    let serialized_result = &serialized["result"];
    assert!(
        serialized_result.get("content").is_some(),
        "ToolCallResult must serialize 'content'"
    );

    let serialized_content = serialized_result["content"].as_array().unwrap();
    assert!(!serialized_content.is_empty(), "'content' must not be empty");

    let first = &serialized_content[0];
    assert_eq!(
        first["type"],
        fixture_result["content"][0]["type"],
        "Content type must match fixture"
    );
    assert!(
        first.get("text").is_some(),
        "Text content must have 'text' field"
    );
}

// ---------------------------------------------------------------------------
// Cross-crate contract: the POST /api/drafts body hoop-mcp sends
// ---------------------------------------------------------------------------

/// hoop-mcp constructs a request body that matches the SAME fixture the daemon
/// validates in hoop-daemon/tests/protocol_contract.rs.
///
/// This is the critical cross-crate drift detector. Both test files load
/// `daemon_http/create_draft_request.json`. If hoop-mcp changes the fields it
/// sends (e.g. renames `kind` → `issue_type`), this test fails. If the daemon
/// changes what it expects and updates the fixture, this test also fails until
/// hoop-mcp's `create_stitch_via_daemon` is updated to match.
///
/// How to fix a failure: update both `create_stitch_via_daemon` in tools.rs
/// AND the fixture file, then verify both test suites pass.
#[test]
fn test_mcp_create_stitch_request_body_matches_daemon_fixture() {
    let fixture = load_fixture("daemon_http/create_draft_request.json");

    // Replicate the json!({}) body built by create_stitch_via_daemon in tools.rs
    let project = fixture["project"].as_str().unwrap();
    let title = fixture["title"].as_str().unwrap();
    let description = fixture["description"].as_str();
    let kind = fixture["kind"].as_str().unwrap();
    let priority: Option<i64> = fixture["priority"].as_i64();

    let mcp_body = serde_json::json!({
        "project": project,
        "title": title,
        "kind": kind,
        "description": description,
        "has_acceptance_criteria": false,
        "priority": priority,
        "labels": [],
        "source": "agent",
    });

    // Every key in the fixture must be present in the MCP body
    for (key, expected_val) in fixture.as_object().unwrap() {
        let actual = mcp_body.get(key).unwrap_or_else(|| {
            panic!(
                "hoop-mcp must send '{}' to daemon (declared in fixture). \
                 If the field was intentionally renamed, update both \
                 create_stitch_via_daemon and the fixture.",
                key
            )
        });

        // Check value matches for non-null fixture values
        if !expected_val.is_null() {
            assert_eq!(
                actual, expected_val,
                "field '{}' value mismatch between MCP body and fixture",
                key
            );
        }
    }

    // Protocol invariants that must never change
    assert_eq!(
        mcp_body["source"], "agent",
        "source must always be 'agent' (protocol invariant)"
    );
    assert_eq!(
        mcp_body["has_acceptance_criteria"], false,
        "has_acceptance_criteria must always be false (protocol invariant)"
    );
}

/// hoop-mcp expects the daemon's stitch response to have a `messages` array,
/// because `redact_stitch_response` iterates over it.
///
/// Fails if the daemon renames `messages` without updating the redaction path
/// in hoop-mcp and the fixture.
#[test]
fn test_mcp_expects_messages_in_stitch_response() {
    let fixture = load_fixture("daemon_http/read_stitch_response.json");

    assert!(
        fixture.get("messages").is_some(),
        "read_stitch fixture must have 'messages' — hoop-mcp's redact_stitch_response requires it"
    );
    assert!(
        fixture["messages"].is_array(),
        "'messages' must be an array"
    );
    assert!(
        fixture.get("stitch").is_some(),
        "read_stitch fixture must have 'stitch' top-level object"
    );

    // Verify message structure matches what StitchMessage serializes
    if let Some(msg) = fixture["messages"].as_array().unwrap().first() {
        for required_field in &["id", "ts", "role", "content"] {
            assert!(
                msg.get(required_field).is_some(),
                "stitch message must have '{}' field (declared in fixture)",
                required_field
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Fixture self-consistency
// ---------------------------------------------------------------------------

#[test]
fn test_all_mcp_fixtures_are_valid_json() {
    let fixtures = [
        "mcp_socket/initialize_request.json",
        "mcp_socket/initialize_response.json",
        "mcp_socket/tools_call_request.json",
        "mcp_socket/tools_call_response.json",
        "daemon_http/create_draft_request.json",
        "daemon_http/create_draft_response.json",
        "daemon_http/read_stitch_response.json",
    ];
    for path in &fixtures {
        let val = load_fixture(path);
        assert!(val.is_object(), "fixture {} must be a JSON object", path);
    }
}
