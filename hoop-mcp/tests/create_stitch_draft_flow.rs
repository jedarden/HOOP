//! End-to-end MCP test for the create_stitch draft/approval boundary.
//!
//! The JSON-RPC call must create only a pending draft. The stitch row and
//! beads are created later, when an operator explicitly approves the draft.

use hoop_mcp::socket::SocketConfig;
use rusqlite::Connection;
use serde_json::Value;
use std::path::Path;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

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
fn create_stitch_jsonrpc_creates_retrievable_pending_draft() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .expect("build test runtime");
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        runtime.block_on(create_stitch_jsonrpc_creates_retrievable_pending_draft_async())
    }));
    runtime.shutdown_timeout(Duration::from_secs(1));
    if let Err(payload) = result {
        std::panic::resume_unwind(payload);
    }
}

async fn create_stitch_jsonrpc_creates_retrievable_pending_draft_async() {
    let (daemon_url, daemon) = hoop_daemon::integration_harness::spawn_test_daemon()
        .await
        .expect("spawn test daemon");
    let fleet_db = daemon.temp_dir.path().join(".hoop/fleet.db");
    std::env::set_var("_HOOP_FLEET_DB_PATH", &fleet_db);
    std::env::set_var("HOOP_DAEMON_URL", &daemon_url);

    let socket_dir = tempfile::tempdir().expect("create MCP socket directory");
    let socket_path = socket_dir.path().join("mcp.sock");
    let socket_task = tokio::spawn(hoop_mcp::socket::run_socket_server(SocketConfig {
        socket_path: socket_path.clone(),
        actor: "test-agent".to_string(),
    }));
    wait_for_socket(&socket_path).await;

    let title = format!("MCP draft approval boundary {}", uuid::Uuid::new_v4());
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "create_stitch",
            "project": "testrepo",
            "title": title.clone(),
            "description": "Must remain pending until operator approval",
            "kind": "investigation"
        }
    });

    let mut stream = UnixStream::connect(&socket_path)
        .await
        .expect("connect to MCP socket");
    stream
        .write_all(format!("{}\n", request).as_bytes())
        .await
        .expect("send JSON-RPC create_stitch request");
    stream.flush().await.expect("flush JSON-RPC request");

    let mut response_line = String::new();
    let mut reader = BufReader::new(stream);
    tokio::time::timeout(
        Duration::from_secs(10),
        reader.read_line(&mut response_line),
    )
    .await
    .expect("MCP response timed out")
    .expect("read JSON-RPC response");
    let response: Value = serde_json::from_str(&response_line).expect("valid JSON-RPC response");
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(
        response["error"].is_null(),
        "create_stitch failed: {response}"
    );

    let response_text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("create_stitch response should contain text JSON");
    let draft_response: Value = serde_json::from_str(response_text).expect("draft response JSON");
    let draft_id = draft_response["draft_id"]
        .as_str()
        .expect("response should include draft_id");
    assert_eq!(draft_response["status"], "pending");

    let client = reqwest::Client::new();
    let draft_response = tokio::time::timeout(
        Duration::from_secs(10),
        client
            .get(format!("{daemon_url}/api/drafts/{draft_id}"))
            .send(),
    )
    .await
    .expect("draft retrieval timed out")
    .expect("retrieve draft");
    let draft_status = draft_response.status();
    let draft_body = draft_response.text().await.expect("read retrieved draft");
    assert!(draft_status.is_success(), "draft retrieval should succeed");
    let draft: Value = serde_json::from_str(&draft_body).expect("parse retrieved draft");
    assert_eq!(draft["id"], draft_id);
    assert_eq!(draft["status"], "pending");
    assert!(
        draft["stitch_id"].is_null(),
        "draft must not have a stitch_id before approval"
    );

    let conn = Connection::open(fleet_db).expect("open fleet database");
    let draft_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM draft_queue WHERE id = ?1 AND status = 'pending'",
            [draft_id],
            |row| row.get(0),
        )
        .expect("query pending draft");
    assert_eq!(
        draft_count, 1,
        "draft should be committed to the draft queue"
    );

    let stitch_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM stitches WHERE title = ?1",
            [&title],
            |row| row.get(0),
        )
        .expect("query stitches");
    assert_eq!(
        stitch_count, 0,
        "create_stitch must not commit a stitch before approval"
    );

    let shutdown = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "shutdown",
        "params": {}
    });
    reader
        .get_mut()
        .write_all(format!("{}\n", shutdown).as_bytes())
        .await
        .expect("send JSON-RPC shutdown request");
    reader
        .get_mut()
        .flush()
        .await
        .expect("flush JSON-RPC shutdown request");
    let mut shutdown_response = String::new();
    tokio::time::timeout(
        Duration::from_secs(2),
        reader.read_line(&mut shutdown_response),
    )
    .await
    .expect("shutdown response timed out")
    .expect("read shutdown response");

    drop(reader);
    socket_task.abort();
    let _ = tokio::time::timeout(Duration::from_secs(1), socket_task).await;
    std::env::remove_var("_HOOP_FLEET_DB_PATH");
    std::env::remove_var("HOOP_DAEMON_URL");
    drop(daemon);
}
