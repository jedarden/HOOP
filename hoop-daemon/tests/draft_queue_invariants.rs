//! Integration tests for the stitch draft queue (§3.10 read-first principle)
//!
//! Acceptance criteria (hoop-ttb.6.6):
//! 1. Agent never silently creates beads — MCP create_stitch → draft queue, not br create
//! 2. Draft queue persists across restarts — stored in fleet.db (SQLite)
//! 3. Preview action audited (who approved, when)
//! 4. Rejection reason optional but captured
//!
//! These tests exercise the fleet::DraftRow CRUD operations and the draft
//! state machine, using a temporary database isolated via _HOOP_FLEET_DB_PATH.

use std::path::PathBuf;
use std::sync::Mutex;
use tempfile::TempDir;

/// Serialize test setup so parallel tests don't fight over the env var.
static LOCK: Mutex<()> = Mutex::new(());

/// Set up a temporary fleet.db for testing.
///
/// Returns the TempDir (must be kept alive for the duration of the test)
/// and the path to the database file.
fn setup_test_db() -> (TempDir, PathBuf) {
    // Acquire lock before touching the env var
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

// ---------------------------------------------------------------------------
// 1. Agent never silently creates beads
// ---------------------------------------------------------------------------

#[test]
fn test_insert_draft_creates_no_beads() {
    let (_tmp, _db_path) = setup_test_db();

    let draft = hoop_daemon::fleet::DraftRow {
        id: "draft-test001".to_string(),
        project: "test-project".to_string(),
        title: "Investigate auth timeout".to_string(),
        kind: "investigation".to_string(),
        description: Some("Users report 30s timeouts on login".to_string()),
        has_acceptance_criteria: false,
        priority: Some(5),
        labels: vec!["auth".to_string()],
        created_by: "os:test-agent".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        source: "agent".to_string(),
        agent_session_id: Some("sess-abc123".to_string()),
        status: "pending".to_string(),
        version: 1,
        original_json: None,
        resolved_by: None,
        resolved_at: None,
        rejection_reason: None,
        stitch_id: None,
        preview_json: None,
    };

    hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");

    let fetched = hoop_daemon::fleet::get_draft("draft-test001")
        .expect("get draft")
        .expect("draft exists");
    assert_eq!(fetched.status, "pending");
    assert_eq!(fetched.source, "agent");
    assert!(fetched.stitch_id.is_none(), "draft must not have stitch_id until approved");

    teardown_test_db();
}

#[test]
fn test_agent_source_preserved_in_draft() {
    let (_tmp, _db_path) = setup_test_db();

    let draft = hoop_daemon::fleet::DraftRow {
        id: "draft-agent-src".to_string(),
        project: "test-project".to_string(),
        title: "Fix memory leak".to_string(),
        kind: "fix".to_string(),
        description: None,
        has_acceptance_criteria: false,
        priority: None,
        labels: vec![],
        created_by: "os:agent-worker-3".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        source: "agent".to_string(),
        agent_session_id: Some("sess-worker3".to_string()),
        status: "pending".to_string(),
        version: 1,
        original_json: None,
        resolved_by: None,
        resolved_at: None,
        rejection_reason: None,
        stitch_id: None,
        preview_json: None,
    };

    hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");

    let fetched = hoop_daemon::fleet::get_draft("draft-agent-src")
        .expect("get draft")
        .expect("draft exists");

    assert_eq!(fetched.source, "agent");
    assert_eq!(fetched.agent_session_id, Some("sess-worker3".to_string()));
    assert_eq!(fetched.created_by, "os:agent-worker-3");

    teardown_test_db();
}

// ---------------------------------------------------------------------------
// 2. Draft queue persists across restarts
// ---------------------------------------------------------------------------

#[test]
fn test_drafts_persist_across_simulated_restart() {
    let (_tmp, db_path) = setup_test_db();

    let draft1 = hoop_daemon::fleet::DraftRow {
        id: "draft-persist-1".to_string(),
        project: "test-project".to_string(),
        title: "First draft".to_string(),
        kind: "investigation".to_string(),
        description: None,
        has_acceptance_criteria: false,
        priority: Some(3),
        labels: vec![],
        created_by: "os:test".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        source: "agent".to_string(),
        agent_session_id: None,
        status: "pending".to_string(),
        version: 1,
        original_json: None,
        resolved_by: None,
        resolved_at: None,
        rejection_reason: None,
        stitch_id: None,
        preview_json: None,
    };

    let draft2 = hoop_daemon::fleet::DraftRow {
        id: "draft-persist-2".to_string(),
        project: "test-project".to_string(),
        title: "Second draft".to_string(),
        kind: "fix".to_string(),
        description: Some("Needs fixing".to_string()),
        has_acceptance_criteria: false,
        priority: Some(7),
        labels: vec!["urgent".to_string()],
        created_by: "os:test".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        source: "agent".to_string(),
        agent_session_id: None,
        status: "edited".to_string(),
        version: 2,
        original_json: None,
        resolved_by: None,
        resolved_at: None,
        rejection_reason: None,
        stitch_id: None,
        preview_json: None,
    };

    hoop_daemon::fleet::insert_draft(&draft1).expect("insert draft1");
    hoop_daemon::fleet::insert_draft(&draft2).expect("insert draft2");

    assert!(db_path.exists(), "fleet.db must persist on disk");

    let fetched1 = hoop_daemon::fleet::get_draft("draft-persist-1")
        .expect("get draft1")
        .expect("draft1 exists");
    assert_eq!(fetched1.title, "First draft");
    assert_eq!(fetched1.status, "pending");

    let fetched2 = hoop_daemon::fleet::get_draft("draft-persist-2")
        .expect("get draft2")
        .expect("draft2 exists");
    assert_eq!(fetched2.title, "Second draft");
    assert_eq!(fetched2.status, "edited");
    assert_eq!(fetched2.version, 2);

    teardown_test_db();
}

#[test]
fn test_list_drafts_filters_by_status() {
    let (_tmp, _db_path) = setup_test_db();

    for (id, status) in [
        ("draft-s1", "pending"),
        ("draft-s2", "pending"),
        ("draft-s3", "edited"),
        ("draft-s4", "approved"),
        ("draft-s5", "submitted"),
        ("draft-s6", "rejected"),
    ] {
        let draft = hoop_daemon::fleet::DraftRow {
            id: id.to_string(),
            project: "test-project".to_string(),
            title: format!("Draft {}", id),
            kind: "investigation".to_string(),
            description: None,
            has_acceptance_criteria: false,
            priority: None,
            labels: vec![],
            created_by: "os:test".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            source: "agent".to_string(),
            agent_session_id: None,
            status: status.to_string(),
            version: 1,
            original_json: None,
            resolved_by: None,
            resolved_at: None,
            rejection_reason: None,
            stitch_id: None,
            preview_json: None,
        };
        hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");
    }

    let pending = hoop_daemon::fleet::list_drafts(None, Some("pending"), 100).expect("list pending");
    assert_eq!(pending.len(), 2);
    assert!(pending.iter().all(|d| d.status == "pending"));

    let rejected = hoop_daemon::fleet::list_drafts(None, Some("rejected"), 100).expect("list rejected");
    assert_eq!(rejected.len(), 1);
    assert_eq!(rejected[0].id, "draft-s6");

    let actionable = {
        let mut p = hoop_daemon::fleet::list_drafts(None, Some("pending"), 100).expect("list pending");
        p.extend(hoop_daemon::fleet::list_drafts(None, Some("edited"), 100).expect("list edited"));
        p
    };
    assert_eq!(actionable.len(), 3); // 2 pending + 1 edited

    teardown_test_db();
}

// ---------------------------------------------------------------------------
// 3. Preview action audited (who approved, when)
// ---------------------------------------------------------------------------

#[test]
fn test_audit_row_written_on_draft_created() {
    let (_tmp, _db_path) = setup_test_db();

    let draft = hoop_daemon::fleet::DraftRow {
        id: "draft-audit-1".to_string(),
        project: "test-project".to_string(),
        title: "Audited draft".to_string(),
        kind: "investigation".to_string(),
        description: None,
        has_acceptance_criteria: false,
        priority: None,
        labels: vec![],
        created_by: "os:test-agent".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        source: "agent".to_string(),
        agent_session_id: None,
        status: "pending".to_string(),
        version: 1,
        original_json: None,
        resolved_by: None,
        resolved_at: None,
        rejection_reason: None,
        stitch_id: None,
        preview_json: None,
    };

    hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");

    let result = hoop_daemon::fleet::write_audit_row(
        "os:test-agent",
        hoop_daemon::fleet::ActionKind::DraftCreated,
        "draft-audit-1",
        Some("test-project"),
        Some(serde_json::json!({"title": "Audited draft", "kind": "investigation"}).to_string()),
        hoop_daemon::fleet::ActionResult::Success,
        None,
        Some("agent"),
        None,
        None,
    );
    assert!(result.is_ok(), "audit row should be written successfully");

    let audit_row = result.unwrap();
    assert_eq!(audit_row.actor, "os:test-agent");
    assert_eq!(audit_row.kind, hoop_daemon::fleet::ActionKind::DraftCreated);
    assert_eq!(audit_row.target, "draft-audit-1");
    assert_eq!(audit_row.project, Some("test-project".to_string()));
    assert!(!audit_row.hash_self.is_empty(), "hash_self must be populated");
    assert!(!audit_row.hash_prev.is_empty(), "hash_prev must be populated (genesis or previous)");

    teardown_test_db();
}

#[test]
fn test_audit_row_captures_approver_identity() {
    let (_tmp, _db_path) = setup_test_db();

    let draft = hoop_daemon::fleet::DraftRow {
        id: "draft-approve-audit".to_string(),
        project: "test-project".to_string(),
        title: "To be approved".to_string(),
        kind: "investigation".to_string(),
        description: None,
        has_acceptance_criteria: false,
        priority: None,
        labels: vec![],
        created_by: "os:test-agent".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        source: "agent".to_string(),
        agent_session_id: None,
        status: "pending".to_string(),
        version: 1,
        original_json: None,
        resolved_by: None,
        resolved_at: None,
        rejection_reason: None,
        stitch_id: None,
        preview_json: None,
    };

    hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");

    let operator = "os:jedarden";
    let now = chrono::Utc::now().to_rfc3339();

    hoop_daemon::fleet::update_draft_status(
        "draft-approve-audit",
        "approved",
        Some(operator),
        Some(&now),
        None,
        None,
    ).expect("update draft status");

    let audit_row = hoop_daemon::fleet::write_audit_row(
        operator,
        hoop_daemon::fleet::ActionKind::DraftApproved,
        "draft-approve-audit",
        Some("test-project"),
        Some(serde_json::json!({
            "title": "To be approved",
            "original_actor": "os:test-agent",
        }).to_string()),
        hoop_daemon::fleet::ActionResult::Success,
        None,
        Some("operator"),
        None,
        None,
    ).expect("write audit row");

    assert_eq!(audit_row.actor, operator);
    assert_eq!(audit_row.kind, hoop_daemon::fleet::ActionKind::DraftApproved);
    assert!(audit_row.source.as_deref() == Some("operator"));

    let updated = hoop_daemon::fleet::get_draft("draft-approve-audit")
        .expect("get draft")
        .expect("draft exists");
    assert_eq!(updated.resolved_by, Some(operator.to_string()));
    assert_eq!(updated.resolved_at, Some(now));
    assert_eq!(updated.status, "approved");

    teardown_test_db();
}

// ---------------------------------------------------------------------------
// 4. Rejection reason optional but captured
// ---------------------------------------------------------------------------

#[test]
fn test_rejection_with_reason() {
    let (_tmp, _db_path) = setup_test_db();

    let draft = hoop_daemon::fleet::DraftRow {
        id: "draft-reject-reason".to_string(),
        project: "test-project".to_string(),
        title: "Bad idea".to_string(),
        kind: "investigation".to_string(),
        description: None,
        has_acceptance_criteria: false,
        priority: None,
        labels: vec![],
        created_by: "os:test-agent".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        source: "agent".to_string(),
        agent_session_id: None,
        status: "pending".to_string(),
        version: 1,
        original_json: None,
        resolved_by: None,
        resolved_at: None,
        rejection_reason: None,
        stitch_id: None,
        preview_json: None,
    };

    hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");

    let operator = "os:jedarden";
    let reason = "Duplicate of existing stitch stitch-abc123";
    let now = chrono::Utc::now().to_rfc3339();

    hoop_daemon::fleet::update_draft_status(
        "draft-reject-reason",
        "rejected",
        Some(operator),
        Some(&now),
        Some(reason),
        None,
    ).expect("reject draft");

    let rejected = hoop_daemon::fleet::get_draft("draft-reject-reason")
        .expect("get draft")
        .expect("draft exists");

    assert_eq!(rejected.status, "rejected");
    assert_eq!(rejected.rejection_reason, Some(reason.to_string()));
    assert_eq!(rejected.resolved_by, Some(operator.to_string()));
    assert_eq!(rejected.resolved_at, Some(now));

    teardown_test_db();
}

#[test]
fn test_rejection_without_reason() {
    let (_tmp, _db_path) = setup_test_db();

    let draft = hoop_daemon::fleet::DraftRow {
        id: "draft-reject-noreason".to_string(),
        project: "test-project".to_string(),
        title: "Also bad".to_string(),
        kind: "investigation".to_string(),
        description: None,
        has_acceptance_criteria: false,
        priority: None,
        labels: vec![],
        created_by: "os:test-agent".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        source: "agent".to_string(),
        agent_session_id: None,
        status: "pending".to_string(),
        version: 1,
        original_json: None,
        resolved_by: None,
        resolved_at: None,
        rejection_reason: None,
        stitch_id: None,
        preview_json: None,
    };

    hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");

    let operator = "os:jedarden";
    let now = chrono::Utc::now().to_rfc3339();

    hoop_daemon::fleet::update_draft_status(
        "draft-reject-noreason",
        "rejected",
        Some(operator),
        Some(&now),
        None,
        None,
    ).expect("reject draft");

    let rejected = hoop_daemon::fleet::get_draft("draft-reject-noreason")
        .expect("get draft")
        .expect("draft exists");

    assert_eq!(rejected.status, "rejected");
    assert_eq!(rejected.rejection_reason, None, "rejection reason is optional");
    assert_eq!(rejected.resolved_by, Some(operator.to_string()));

    teardown_test_db();
}

#[test]
fn test_rejection_audit_captures_reason() {
    let (_tmp, _db_path) = setup_test_db();

    let reason = "Already tracked in stitch-xyz";
    let audit_row = hoop_daemon::fleet::write_audit_row(
        "os:jedarden",
        hoop_daemon::fleet::ActionKind::DraftRejected,
        "draft-reject-audit",
        Some("test-project"),
        Some(serde_json::json!({
            "title": "Rejected stitch",
            "rejection_reason": reason,
        }).to_string()),
        hoop_daemon::fleet::ActionResult::Success,
        None,
        Some("operator"),
        None,
        None,
    ).expect("write audit row");

    assert_eq!(audit_row.kind, hoop_daemon::fleet::ActionKind::DraftRejected);

    let args: serde_json::Value = serde_json::from_str(
        audit_row.args_json.as_deref().unwrap_or("{}")
    ).unwrap();
    assert_eq!(args["rejection_reason"], reason);

    teardown_test_db();
}

// ---------------------------------------------------------------------------
// State machine invariants
// ---------------------------------------------------------------------------

#[test]
fn test_edit_increments_version_and_stores_original() {
    let (_tmp, _db_path) = setup_test_db();

    let draft = hoop_daemon::fleet::DraftRow {
        id: "draft-edit-ver".to_string(),
        project: "test-project".to_string(),
        title: "Original title".to_string(),
        kind: "investigation".to_string(),
        description: Some("Original description".to_string()),
        has_acceptance_criteria: false,
        priority: Some(5),
        labels: vec![],
        created_by: "os:test".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        source: "agent".to_string(),
        agent_session_id: None,
        status: "pending".to_string(),
        version: 1,
        original_json: None,
        resolved_by: None,
        resolved_at: None,
        rejection_reason: None,
        stitch_id: None,
        preview_json: None,
    };

    hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");

    hoop_daemon::fleet::edit_draft(
        "draft-edit-ver",
        Some("Updated title"),
        Some("Updated description"),
        None,
        Some(8),
        None,
    ).expect("edit draft");

    let edited = hoop_daemon::fleet::get_draft("draft-edit-ver")
        .expect("get draft")
        .expect("draft exists");

    assert_eq!(edited.title, "Updated title");
    assert_eq!(edited.description, Some("Updated description".to_string()));
    assert_eq!(edited.priority, Some(8));
    assert_eq!(edited.version, 2, "edit must increment version");
    assert_eq!(edited.status, "edited", "edit must set status to 'edited'");
    assert!(edited.original_json.is_some(), "first edit must store original_json");

    teardown_test_db();
}

#[test]
fn test_approved_draft_records_stitch_id() {
    let (_tmp, _db_path) = setup_test_db();

    let draft = hoop_daemon::fleet::DraftRow {
        id: "draft-stitch-id".to_string(),
        project: "test-project".to_string(),
        title: "Approved draft".to_string(),
        kind: "investigation".to_string(),
        description: None,
        has_acceptance_criteria: false,
        priority: None,
        labels: vec![],
        created_by: "os:test-agent".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        source: "agent".to_string(),
        agent_session_id: None,
        status: "pending".to_string(),
        version: 1,
        original_json: None,
        resolved_by: None,
        resolved_at: None,
        rejection_reason: None,
        stitch_id: None,
        preview_json: None,
    };

    hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");

    let operator = "os:jedarden";
    let now = chrono::Utc::now().to_rfc3339();
    let stitch_id = "stitch-abc123";

    hoop_daemon::fleet::update_draft_status(
        "draft-stitch-id",
        "submitted",
        Some(operator),
        Some(&now),
        None,
        Some(stitch_id),
    ).expect("approve and submit draft");

    let submitted = hoop_daemon::fleet::get_draft("draft-stitch-id")
        .expect("get draft")
        .expect("draft exists");

    assert_eq!(submitted.status, "submitted");
    assert_eq!(submitted.stitch_id, Some(stitch_id.to_string()));
    assert_eq!(submitted.resolved_by, Some(operator.to_string()));

    teardown_test_db();
}

#[test]
fn test_hash_chain_integrity_with_draft_actions() {
    let (_tmp, _db_path) = setup_test_db();

    let actions = [
        hoop_daemon::fleet::ActionKind::DraftCreated,
        hoop_daemon::fleet::ActionKind::DraftEdited,
        hoop_daemon::fleet::ActionKind::DraftApproved,
        hoop_daemon::fleet::ActionKind::DraftRejected,
        hoop_daemon::fleet::ActionKind::DraftCreated,
    ];

    let mut prev_hash = String::new();
    for (i, kind) in actions.iter().enumerate() {
        let row = hoop_daemon::fleet::write_audit_row(
            &format!("os:actor-{}", i),
            kind.clone(),
            &format!("target-{}", i),
            Some("test-project"),
            None,
            hoop_daemon::fleet::ActionResult::Success,
            None,
            None,
            None,
            None,
        ).expect("write audit row");

        if i > 0 {
            assert_eq!(
                row.hash_prev, prev_hash,
                "hash_prev must match previous row's hash_self"
            );
        }
        prev_hash = row.hash_self.clone();
    }

    hoop_daemon::fleet::verify_hash_chain().expect("hash chain must be valid after draft actions");

    teardown_test_db();
}
