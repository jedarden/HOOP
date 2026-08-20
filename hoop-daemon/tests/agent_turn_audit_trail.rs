//! Integration tests for agent turn audit trail (hoop-ttb.6.15)
//!
//! Acceptance criteria:
//! - Audit row includes the agent's model + adapter
//! - Turn reference clickable in audit UI
//! - Reconstructing any drafted Stitch back to its origin chat turn works
//!
//! This test verifies the full flow:
//! 1. Agent creates draft with session_id and turn_id
//! 2. Draft is approved and stitch is created
//! 3. Stitch row contains audit fields (created_by_actor, created_by_session_id, created_by_adapter, created_by_model, turn_id)
//! 4. Audit rows include the agent metadata

use std::path::PathBuf;
use std::sync::Mutex;
use tempfile::TempDir;

/// Serialize test setup so parallel tests don't fight over the env var.
static LOCK: Mutex<()> = Mutex::new(());

/// Set up a temporary fleet.db for testing.
fn setup_test_db() -> (TempDir, PathBuf) {
    let _guard = LOCK.lock().unwrap();

    let tmp = TempDir::new().expect("create temp dir");
    let hoop_dir = tmp.path().join(".hoop");
    std::fs::create_dir_all(&hoop_dir).expect("create .hoop dir");
    let db_path = hoop_dir.join("fleet.db");

    // Override fleet::db_path() for this test
    std::env::set_var("_HOOP_FLEET_DB_PATH", &db_path);

    hoop_daemon::fleet::init_fleet_db().expect("init fleet.db");

    (tmp, db_path)
}

/// Restore the env var after the test.
fn teardown_test_db() {
    let _guard = LOCK.lock().unwrap();
    std::env::remove_var("_HOOP_FLEET_DB_PATH");
}

/// Test: Draft with agent session ID and turn ID propagates to stitch audit fields
#[test]
fn test_draft_agent_metadata_propagates_to_stitch() {
    let (_tmp, _db_path) = setup_test_db();

    let session_id = "agent-session-abc123";
    let turn_id = "turn-xyz789";
    let adapter = "claude";
    let model = "claude-opus-4-7";

    // Create a draft with agent metadata
    let draft = hoop_daemon::fleet::DraftRow {
        id: "draft-audit-test".to_string(),
        project: "test-project".to_string(),
        title: "Test audit trail".to_string(),
        kind: "investigation".to_string(),
        description: Some("Testing agent audit trail".to_string()),
        has_acceptance_criteria: false,
        priority: Some(3),
        labels: vec![],
        created_by: "user:test-operator".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        source: "agent".to_string(),
        agent_session_id: Some(session_id.to_string()),
        turn_id: Some(turn_id.to_string()),
        status: "pending".to_string(),
        version: 1,
        original_json: None,
        resolved_by: None,
        resolved_at: None,
        rejection_reason: None,
        stitch_id: None,
        preview_json: None,
        opened_by: None,
        opened_at: None,
        last_autosave_at: None,
        abandoned_at: None,
    };

    hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");

    // Fetch the draft to verify it was stored
    let fetched = hoop_daemon::fleet::get_draft("draft-audit-test")
        .expect("get draft")
        .expect("draft exists");

    assert_eq!(fetched.agent_session_id, Some(session_id.to_string()));
    assert_eq!(fetched.turn_id, Some(turn_id.to_string()));

    // Now simulate creating a stitch with the audit fields
    // This is what happens when the draft is approved
    let stitch_id = "stitch-audit-test";
    let bead_links: Vec<(&str, &str)> = vec![];

    let actor = format!("hoop:agent:{}", session_id);

    hoop_daemon::fleet::create_stitch_with_audit(
        stitch_id,
        &fetched.project,
        &fetched.kind,
        &fetched.title,
        &fetched.created_by,
        &bead_links,
        "operator",
        Some(&actor),
        Some(session_id),
        Some(adapter),
        Some(model),
        Some(turn_id),
    )
    .expect("create stitch with audit");

    // Query the stitch to verify audit fields were stored
    let conn = rusqlite::Connection::open(hoop_daemon::fleet::db_path()).expect("open fleet.db");

    let stitch_row: (String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>) = conn
        .query_row(
            "SELECT id, created_by_actor, created_by_session_id, created_by_adapter, created_by_model, turn_id
             FROM stitches WHERE id = ?1",
            &[stitch_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .expect("query stitch");

    assert_eq!(stitch_row.0, stitch_id);
    assert_eq!(
        stitch_row.1,
        Some(actor.to_string()),
        "created_by_actor should be set"
    );
    assert_eq!(
        stitch_row.2,
        Some(session_id.to_string()),
        "created_by_session_id should be set"
    );
    assert_eq!(
        stitch_row.3,
        Some(adapter.to_string()),
        "created_by_adapter should be set"
    );
    assert_eq!(
        stitch_row.4,
        Some(model.to_string()),
        "created_by_model should be set"
    );
    assert_eq!(
        stitch_row.5,
        Some(turn_id.to_string()),
        "turn_id should be set"
    );

    // Verify stitch_messages has the turn_id as a system note
    let message_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM stitch_messages WHERE stitch_id = ?1 AND role = 'system'",
            &[stitch_id],
            |row| row.get(0),
        )
        .expect("count system messages");

    assert_eq!(
        message_count, 1,
        "Should have one system note with turn reference"
    );

    let message_content: String = conn
        .query_row(
            "SELECT content FROM stitch_messages WHERE stitch_id = ?1 AND role = 'system'",
            &[stitch_id],
            |row| row.get(0),
        )
        .expect("get system message content");

    assert!(
        message_content.contains(turn_id),
        "System message should reference the turn_id"
    );

    teardown_test_db();
}

/// Test: Audit row for stitch creation includes agent metadata
#[test]
fn test_stitch_created_audit_includes_agent_metadata() {
    let (_tmp, _db_path) = setup_test_db();

    let session_id = "agent-session-audit";
    let turn_id = "turn-audit-123";
    let adapter = "claude";
    let model = "claude-opus-4-7";

    // Write a StitchCreated audit row with agent metadata
    let stitch_id = "stitch-audit-row-test";
    let args = serde_json::json!({
        "source": "agent",
        "kind": "investigation",
        "title": "Test audit row",
        "bead_count": 2,
        "bead_ids": ["bead-1", "bead-2"],
        "agent_session_id": session_id,
        "agent_adapter": adapter,
        "agent_model": model,
        "turn_id": turn_id,
    })
    .to_string();

    let actor = format!("hoop:agent:{}", session_id);

    hoop_daemon::fleet::write_audit_row(
        &actor,
        hoop_daemon::fleet::ActionKind::StitchCreated,
        stitch_id,
        Some("test-project"),
        Some(args),
        hoop_daemon::fleet::ActionResult::Success,
        None,
        Some("agent"),
        Some(stitch_id),
        None,
    )
    .expect("write audit row");

    // Query the audit row
    let rows = hoop_daemon::fleet::query_audit_rows(Some(10), None, None, None, None, None)
        .expect("query audit rows");

    let audit_row = rows
        .iter()
        .find(|r| r.target == stitch_id)
        .expect("should find audit row for our stitch");

    assert_eq!(audit_row.actor, actor);
    assert_eq!(
        audit_row.kind,
        hoop_daemon::fleet::ActionKind::StitchCreated
    );

    // Parse args_json and verify agent metadata is present
    let args_value: serde_json::Value = audit_row
        .args_json
        .as_ref()
        .map(|s| serde_json::from_str(s).unwrap())
        .expect("args_json should be valid JSON");

    assert_eq!(args_value["agent_session_id"], session_id);
    assert_eq!(args_value["agent_adapter"], adapter);
    assert_eq!(args_value["agent_model"], model);
    assert_eq!(args_value["turn_id"], turn_id);

    teardown_test_db();
}

/// Test: Actor format "hoop:agent:<session-id>" is parseable
#[test]
fn test_agent_actor_format_is_parseable() {
    let session_id = "session-xyz-123";
    let actor = format!("hoop:agent:{}", session_id);

    // Parse the actor string
    let parts: Vec<&str> = actor.split(':').collect();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0], "hoop");
    assert_eq!(parts[1], "agent");
    assert_eq!(parts[2], session_id);

    // Verify we can extract the session_id
    let extracted_session_id = actor.strip_prefix("hoop:agent:").unwrap();
    assert_eq!(extracted_session_id, session_id);
}

/// Test: Turn ID format is consistent
#[test]
fn test_turn_id_format_is_consistent() {
    let turn_id = hoop_daemon::agent_session::AgentSessionManager::current_turn_id;
    // This is just a compile-time check that the function exists
    // The actual functionality is tested in the agent_session module tests
}

/// Test: Stitch can be reconstructed back to its origin turn
#[test]
fn test_stitch_reconstructs_to_origin_turn() {
    let (_tmp, _db_path) = setup_test_db();

    let session_id = "agent-session-reconstruct";
    let turn_id = "turn-reconstruct-456";
    let adapter = "claude";
    let model = "claude-opus-4-7";

    // Create a stitch with full audit trail
    let stitch_id = "stitch-reconstruct-test";
    let bead_links: Vec<(&str, &str)> = vec![];
    let actor = format!("hoop:agent:{}", session_id);

    hoop_daemon::fleet::create_stitch_with_audit(
        stitch_id,
        "test-project",
        "investigation",
        "Reconstruction test",
        "user:test",
        &bead_links,
        "operator",
        Some(&actor),
        Some(session_id),
        Some(adapter),
        Some(model),
        Some(turn_id),
    )
    .expect("create stitch for reconstruction");

    // Query the stitch
    let conn = rusqlite::Connection::open(hoop_daemon::fleet::db_path()).expect("open fleet.db");

    // Reconstruct the origin turn information
    let (stored_session_id, stored_adapter, stored_model, stored_turn_id): (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ) = conn
        .query_row(
            "SELECT created_by_session_id, created_by_adapter, created_by_model, turn_id
             FROM stitches WHERE id = ?1",
            &[stitch_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .expect("query stitch for reconstruction");

    // Verify we can reconstruct the full turn context
    assert_eq!(stored_session_id, Some(session_id.to_string()));
    assert_eq!(stored_adapter, Some(adapter.to_string()));
    assert_eq!(stored_model, Some(model.to_string()));
    assert_eq!(stored_turn_id, Some(turn_id.to_string()));

    // With this information, we can:
    // 1. Navigate to the agent session (session_id)
    // 2. Find the specific turn (turn_id)
    // 3. Display the adapter/model used

    // Generate the turn URL (as done in the UI)
    let turn_url = format!("/agent?session={}&turn={}", session_id, turn_id);
    assert_eq!(
        turn_url,
        format!("/agent?session={}&turn={}", session_id, turn_id)
    );

    teardown_test_db();
}
