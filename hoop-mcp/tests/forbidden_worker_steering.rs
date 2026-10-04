//! Runtime guard tests for forbidden worker-steering verbs
//!
//! Verifies that the MCP server rejects worker-steering actions with clear
//! error messages, as specified in plan §6 Phase 5 deliverable 3 and §8.2.

use hoop_mcp::tools::{
    forbidden_worker_steering_error, is_forbidden_worker_steering_verb,
    FORBIDDEN_WORKER_STEERING_VERBS,
};
use serde_json::Value;
use std::path::Path;
use std::time::Duration;

async fn wait_for_socket(path: &Path) {
    for _ in 0..100 {
        if path.exists() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("MCP socket did not appear at {}", path.display());
}

#[test]
fn test_forbidden_list_contains_all_required_verbs() {
    let expected = [
        "launch_fleet",
        "stop_fleet",
        "release_claim",
        "boost_priority",
        "close_stitch",
        "close_bead",
        "signal_worker",
        "kill_worker",
        "pause_worker",
    ];

    assert_eq!(
        FORBIDDEN_WORKER_STEERING_VERBS.len(),
        expected.len(),
        "FORBIDDEN_WORKER_STEERING_VERBS has {} entries, expected {}",
        FORBIDDEN_WORKER_STEERING_VERBS.len(),
        expected.len()
    );

    for verb in &expected {
        assert!(
            FORBIDDEN_WORKER_STEERING_VERBS.contains(verb),
            "'{}' missing from FORBIDDEN_WORKER_STEERING_VERBS",
            verb
        );
        assert!(
            is_forbidden_worker_steering_verb(verb),
            "'{}' not detected as forbidden",
            verb
        );
    }
}

#[test]
fn test_is_forbidden_worker_steering_verb() {
    // All forbidden verbs should be detected
    for verb in FORBIDDEN_WORKER_STEERING_VERBS {
        assert!(
            is_forbidden_worker_steering_verb(verb),
            "'{}' should be detected as forbidden",
            verb
        );
    }

    // Legitimate tools should not be flagged
    let legitimate_tools = [
        "find_stitches",
        "read_stitch",
        "find_beads",
        "read_bead",
        "read_file",
        "grep",
        "search_conversations",
        "summarize_project",
        "summarize_day",
        "create_stitch",
        "escalate_to_operator",
    ];

    for tool in &legitimate_tools {
        assert!(
            !is_forbidden_worker_steering_verb(tool),
            "'{}' should NOT be detected as forbidden",
            tool
        );
    }
}

#[test]
fn test_forbidden_worker_steering_error_message() {
    let tool_name = "launch_fleet";
    let error = forbidden_worker_steering_error(tool_name);

    assert!(error.contains("HOOP cannot perform worker-steering actions"));
    assert!(error.contains(tool_name));
    assert!(error.contains("br close"));
    assert!(error.contains("NEEDLE's tooling"));
}

#[test]
fn test_each_forbidden_verb_has_distinct_error() {
    for verb in FORBIDDEN_WORKER_STEERING_VERBS {
        let error = forbidden_worker_steering_error(verb);
        assert!(
            error.contains(verb),
            "Error message for '{}' should mention the tool name",
            verb
        );
        assert!(
            error.contains("worker-steering"),
            "Error should mention 'worker-steering'"
        );
    }
}

#[test]
fn test_runtime_guard_rejects_all_forbidden_verbs() {
    use hoop_mcp::tools::McpServerState;
    use serde_json::Map;

    // Create a minimal server state (audit log path may not exist, but we only test call_tool)
    let state = McpServerState::new("test-actor".to_string())
        .expect("Failed to create McpServerState for test");

    for verb in FORBIDDEN_WORKER_STEERING_VERBS {
        let result = state.call_tool(verb, &Map::new());
        assert!(
            result.is_err(),
            "call_tool should reject forbidden verb '{}'",
            verb
        );

        let error_msg = result.unwrap_err();
        assert!(
            error_msg.contains("worker-steering"),
            "Error for '{}' should mention 'worker-steering', got: {}",
            verb,
            error_msg
        );
        assert!(
            error_msg.contains(verb),
            "Error for '{}' should mention the tool name, got: {}",
            verb,
            error_msg
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn forbidden_worker_steering_jsonrpc_calls_return_hard_errors() {
    use hoop_mcp::socket::{run_socket_server, SocketConfig};
    use tempfile::tempdir;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixStream;

    let socket_dir = tempdir().expect("create MCP socket directory");
    let socket_path = socket_dir.path().join("mcp.sock");
    let socket_task = tokio::spawn(run_socket_server(SocketConfig {
        socket_path: socket_path.clone(),
        actor: "forbidden-worker-steering-test".to_string(),
    }));
    wait_for_socket(&socket_path).await;

    let stream = UnixStream::connect(&socket_path)
        .await
        .expect("connect to MCP socket");
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    for (id, tool_name) in [(1, "launch_fleet"), (2, "kill_worker")] {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/call",
            "params": { "name": tool_name }
        });
        writer
            .write_all(format!("{request}\n").as_bytes())
            .await
            .expect("send forbidden JSON-RPC request");
        writer
            .flush()
            .await
            .expect("flush forbidden JSON-RPC request");

        let mut response_line = String::new();
        tokio::time::timeout(Duration::from_secs(2), reader.read_line(&mut response_line))
            .await
            .expect("forbidden JSON-RPC response timed out")
            .expect("read forbidden JSON-RPC response");
        let response: Value =
            serde_json::from_str(&response_line).expect("valid forbidden JSON-RPC response");

        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], id);
        assert!(
            response.get("result").is_none(),
            "forbidden tool {tool_name} must not return a result: {response}"
        );
        let error = response
            .get("error")
            .and_then(Value::as_object)
            .unwrap_or_else(|| {
                panic!("forbidden tool {tool_name} must return an error: {response}")
            });
        assert_eq!(error["code"], -32603);
        let message = error["message"]
            .as_str()
            .expect("JSON-RPC error should include a message");
        assert!(
            message.contains(tool_name),
            "error should name forbidden tool {tool_name}: {message}"
        );
        assert!(
            message.contains("cannot perform worker-steering actions"),
            "error should explain that worker-steering is forbidden: {message}"
        );
    }

    drop(reader);
    drop(writer);
    socket_task.abort();
    let _ = socket_task.await;
}

#[test]
fn test_runtime_guard_allows_legitimate_tools() {
    use hoop_mcp::tools::McpServerState;
    use serde_json::{json, Map};

    // Create a minimal server state
    let state = McpServerState::new("test-actor".to_string())
        .expect("Failed to create McpServerState for test");

    // escalate_to_operator doesn't require project context
    let args: Map<String, serde_json::Value> =
        json!({"message": "test"}).as_object().unwrap().clone();
    let result = state.call_tool("escalate_to_operator", &args);
    // This should fail because escalate_to_operator writes to a file, but NOT because of the forbidden guard
    // The key is that it shouldn't return the "worker-steering" error
    if let Err(e) = result {
        assert!(
            !e.contains("worker-steering"),
            "Legitimate tool should not trigger worker-steering error, got: {}",
            e
        );
    }
}

#[test]
fn test_unknown_tool_not_confused_with_forbidden() {
    use hoop_mcp::tools::{forbidden_worker_steering_error, is_forbidden_worker_steering_verb};

    let unknown_tool = "some_random_tool_that_does_not_exist";

    assert!(
        !is_forbidden_worker_steering_verb(unknown_tool),
        "Unknown tool should not be classified as worker-steering verb"
    );

    let error = forbidden_worker_steering_error(unknown_tool);
    assert!(error.contains(unknown_tool));
}
