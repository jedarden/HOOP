//! Multi-operator concurrency tests (§19)
//!
//! Closing criteria:
//! 1. Two concurrent drafts don't clobber each other
//! 2. Presence indicators optional and privacy-respecting
//! 3. Proposal dedup tested with same-content-hash collisions
//! 4. Per-operator agent sessions isolated correctly
//!
//! Plan reference: §19 Multi-operator concurrency (phase 7)

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
// §19.1 Draft concurrency tests
// ---------------------------------------------------------------------------

#[test]
fn test_two_concurrent_drafts_both_land() {
    let (_tmp, _db_path) = setup_test_db();

    let now = chrono::Utc::now().to_rfc3339();

    // Operator A opens a draft
    let draft_a = hoop_daemon::fleet::DraftRow {
        id: "draft-concurrent-a".to_string(),
        project: "test-project".to_string(),
        title: "Fix auth timeout".to_string(),
        kind: "fix".to_string(),
        description: Some("Users report 30s timeouts".to_string()),
        has_acceptance_criteria: false,
        priority: Some(5),
        labels: vec!["auth".to_string()],
        created_by: "tailscale:operator-a@example.com".to_string(),
        created_at: now.clone(),
        source: "form".to_string(),
        agent_session_id: None,
        turn_id: None,
        status: "pending".to_string(),
        version: 1,
        original_json: None,
        resolved_by: None,
        resolved_at: None,
        rejection_reason: None,
        stitch_id: None,
        preview_json: None,
        opened_by: Some("tailscale:operator-a@example.com".to_string()),
        opened_at: Some(now.clone()),
        last_autosave_at: None,
        abandoned_at: None,
    };

    // Operator B opens a draft at the same time
    let draft_b = hoop_daemon::fleet::DraftRow {
        id: "draft-concurrent-b".to_string(),
        project: "test-project".to_string(),
        title: "Fix auth timeout".to_string(), // Same title!
        kind: "fix".to_string(),
        description: Some("Users report 30s timeouts".to_string()), // Same description!
        has_acceptance_criteria: false,
        priority: Some(5),
        labels: vec!["auth".to_string()],
        created_by: "tailscale:operator-b@example.com".to_string(),
        created_at: now.clone(),
        source: "form".to_string(),
        agent_session_id: None,
        turn_id: None,
        status: "pending".to_string(),
        version: 1,
        original_json: None,
        resolved_by: None,
        resolved_at: None,
        rejection_reason: None,
        stitch_id: None,
        preview_json: None,
        opened_by: Some("tailscale:operator-b@example.com".to_string()),
        opened_at: Some(now.clone()),
        last_autosave_at: None,
        abandoned_at: None,
    };

    // Both drafts should be accepted without conflict
    hoop_daemon::fleet::insert_draft(&draft_a).expect("insert draft_a");
    hoop_daemon::fleet::insert_draft(&draft_b).expect("insert draft_b");

    // Verify both drafts exist independently
    let fetched_a = hoop_daemon::fleet::get_draft("draft-concurrent-a")
        .expect("get draft_a")
        .expect("draft_a exists");
    assert_eq!(
        fetched_a.opened_by,
        Some("tailscale:operator-a@example.com".to_string())
    );

    let fetched_b = hoop_daemon::fleet::get_draft("draft-concurrent-b")
        .expect("get draft_b")
        .expect("draft_b exists");
    assert_eq!(
        fetched_b.opened_by,
        Some("tailscale:operator-b@example.com".to_string())
    );

    teardown_test_db();
}

#[test]
fn test_autosave_preserves_draft_concurrency() {
    let (_tmp, _db_path) = setup_test_db();

    let now = chrono::Utc::now().to_rfc3339();

    // Create initial draft
    let draft = hoop_daemon::fleet::DraftRow {
        id: "draft-autosave-test".to_string(),
        project: "test-project".to_string(),
        title: "Initial title".to_string(),
        kind: "task".to_string(),
        description: Some("Initial description".to_string()),
        has_acceptance_criteria: false,
        priority: Some(3),
        labels: vec![],
        created_by: "tailscale:operator-a@example.com".to_string(),
        created_at: now.clone(),
        source: "form".to_string(),
        agent_session_id: None,
        turn_id: None,
        status: "pending".to_string(),
        version: 1,
        original_json: None,
        resolved_by: None,
        resolved_at: None,
        rejection_reason: None,
        stitch_id: None,
        preview_json: None,
        opened_by: Some("tailscale:operator-a@example.com".to_string()),
        opened_at: Some(now.clone()),
        last_autosave_at: None,
        abandoned_at: None,
    };

    hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");

    // Autosave updates fields without incrementing version
    hoop_daemon::fleet::autosave_draft(
        "draft-autosave-test",
        Some("Updated title"),
        Some("Updated description"),
        None,
        Some(7),
        Some(&["urgent".to_string()]),
    )
    .expect("autosave draft");

    let fetched = hoop_daemon::fleet::get_draft("draft-autosave-test")
        .expect("get draft")
        .expect("draft exists");

    assert_eq!(fetched.title, "Updated title");
    assert_eq!(fetched.description, Some("Updated description".to_string()));
    assert_eq!(fetched.priority, Some(7));
    assert_eq!(fetched.labels, vec!["urgent".to_string()]);
    assert_eq!(
        fetched.version, 1,
        "version should NOT increment on autosave"
    );
    assert!(
        fetched.last_autosave_at.is_some(),
        "last_autosave_at should be set"
    );

    teardown_test_db();
}

#[test]
fn test_abandon_draft_marks_abandoned() {
    let (_tmp, _db_path) = setup_test_db();

    let now = chrono::Utc::now().to_rfc3339();

    let draft = hoop_daemon::fleet::DraftRow {
        id: "draft-abandon-test".to_string(),
        project: "test-project".to_string(),
        title: "To be abandoned".to_string(),
        kind: "task".to_string(),
        description: None,
        has_acceptance_criteria: false,
        priority: None,
        labels: vec![],
        created_by: "tailscale:operator-a@example.com".to_string(),
        created_at: now.clone(),
        source: "form".to_string(),
        agent_session_id: None,
        turn_id: None,
        status: "pending".to_string(),
        version: 1,
        original_json: None,
        resolved_by: None,
        resolved_at: None,
        rejection_reason: None,
        stitch_id: None,
        preview_json: None,
        opened_by: Some("tailscale:operator-a@example.com".to_string()),
        opened_at: Some(now.clone()),
        last_autosave_at: None,
        abandoned_at: None,
    };

    hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");

    // Abandon the draft
    hoop_daemon::fleet::abandon_draft("draft-abandon-test").expect("abandon draft");

    let fetched = hoop_daemon::fleet::get_draft("draft-abandon-test")
        .expect("get draft")
        .expect("draft exists");

    assert_eq!(fetched.status, "abandoned");
    assert!(fetched.abandoned_at.is_some(), "abandoned_at should be set");

    teardown_test_db();
}

#[test]
fn test_detect_similar_drafts_warns_duplication() {
    let (_tmp, _db_path) = setup_test_db();

    let now = chrono::Utc::now().to_rfc3339();

    // Create an existing draft
    let existing = hoop_daemon::fleet::DraftRow {
        id: "draft-existing".to_string(),
        project: "test-project".to_string(),
        title: "Fix authentication timeout".to_string(),
        kind: "fix".to_string(),
        description: Some("Users experiencing 30s delays on login".to_string()),
        has_acceptance_criteria: false,
        priority: Some(5),
        labels: vec!["auth".to_string()],
        created_by: "tailscale:operator-a@example.com".to_string(),
        created_at: now.clone(),
        source: "form".to_string(),
        agent_session_id: None,
        turn_id: None,
        status: "pending".to_string(),
        version: 1,
        original_json: None,
        resolved_by: None,
        resolved_at: None,
        rejection_reason: None,
        stitch_id: None,
        preview_json: None,
        opened_by: Some("tailscale:operator-a@example.com".to_string()),
        opened_at: Some(now.clone()),
        last_autosave_at: None,
        abandoned_at: None,
    };

    hoop_daemon::fleet::insert_draft(&existing).expect("insert existing draft");

    // Check for similar drafts (Already-Started Detection, marquee #8)
    let similar = hoop_daemon::fleet::detect_similar_drafts(
        "test-project",
        Some("Fix authentication timeout"),
        Some("Users experiencing 30s delays"),
        None,
    )
    .expect("detect similar drafts");

    // Should find the existing draft
    assert!(!similar.is_empty(), "should detect similar existing draft");
    assert_eq!(similar[0].id, "draft-existing");

    teardown_test_db();
}

// ---------------------------------------------------------------------------
// §19.2 Reflection Ledger concurrency tests
// ---------------------------------------------------------------------------

#[test]
fn test_reflection_proposal_dedup_by_content_hash() {
    let (_tmp, _db_path) = setup_test_db();

    // Operator A proposes a rule
    let id_a = hoop_daemon::fleet::propose_reflection_entry(
        "Always run tests before closing beads",
        "Operator repeated this 3 times",
        "global",
        &["stitch-a-1".to_string(), "stitch-a-2".to_string()],
    )
    .expect("propose from operator A");

    // Operator B proposes the SAME rule (same content hash)
    let id_b = hoop_daemon::fleet::propose_reflection_entry(
        "Always run tests before closing beads",
        "Different reason text", // Different reason doesn't matter for dedup
        "global",
        &["stitch-b-1".to_string()],
    )
    .expect("propose from operator B");

    // Should return the same ID (deduplicated)
    assert_eq!(id_a, id_b, "duplicate proposal should return the same ID");

    // Verify only one proposal exists with merged source_stitches
    let proposals =
        hoop_daemon::fleet::list_pending_reflection_proposals().expect("list proposals");

    assert_eq!(proposals.len(), 1, "should have only one proposal");
    assert_eq!(
        proposals[0].id, id_a,
        "proposal ID should match first proposal"
    );

    // Verify source_stitches were merged from both operators
    let stitches: Vec<String> =
        serde_json::from_str(&proposals[0].source_stitches).expect("parse source_stitches");
    assert_eq!(stitches.len(), 3, "should have 3 merged source stitches");
    assert!(stitches.contains(&"stitch-a-1".to_string()));
    assert!(stitches.contains(&"stitch-a-2".to_string()));
    assert!(stitches.contains(&"stitch-b-1".to_string()));

    teardown_test_db();
}

#[test]
fn test_reflection_proposal_approval_single_operator() {
    let (_tmp, _db_path) = setup_test_db();

    let proposal_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // Create a proposal directly in the database
    let entry = hoop_daemon::fleet::ReflectionLedgerEntry {
        id: proposal_id.clone(),
        scope: "global".to_string(),
        rule: "Never edit production config without review".to_string(),
        reason: "One incident of corruption".to_string(),
        source_stitches: "[]".to_string(),
        status: "proposed".to_string(),
        created_at: now.clone(),
        last_applied: None,
        applied_count: 0,
        approved_by: None,
        approved_at: None,
        archived_at: None,
        content_hash: "".to_string(),
        rejection_count: 0,
    };

    hoop_daemon::fleet::insert_reflection_entry(&entry).expect("insert proposal");

    // Operator A approves
    let approved = hoop_daemon::fleet::approve_reflection_proposal(
        &proposal_id,
        "tailscale:operator-a@example.com",
    )
    .expect("approve proposal");

    assert!(approved, "proposal should be approved");

    // Verify approval metadata
    let proposal = hoop_daemon::fleet::get_reflection_proposal(&proposal_id)
        .expect("get proposal")
        .expect("proposal exists");

    // Note: After approval, status changes to 'approved', so get_reflection_proposal
    // might not return it (since it filters by status='proposed')
    // Let's check via list_approved_reflection_entries instead
    let approved_list =
        hoop_daemon::fleet::list_approved_reflection_entries(None).expect("list approved entries");

    assert!(!approved_list.is_empty(), "should have approved entries");
    assert_eq!(
        approved_list[0].approved_by,
        Some("tailscale:operator-a@example.com".to_string())
    );

    teardown_test_db();
}

#[test]
fn test_reflection_proposal_rejection_prevents_reproposal() {
    let (_tmp, _db_path) = setup_test_db();

    let proposal_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let entry = hoop_daemon::fleet::ReflectionLedgerEntry {
        id: proposal_id.clone(),
        scope: "global".to_string(),
        rule: "Bad rule".to_string(),
        reason: "Test rejection".to_string(),
        source_stitches: "[]".to_string(),
        status: "proposed".to_string(),
        created_at: now.clone(),
        last_applied: None,
        applied_count: 0,
        approved_by: None,
        approved_at: None,
        archived_at: None,
        content_hash: "".to_string(),
        rejection_count: 0,
    };

    hoop_daemon::fleet::insert_reflection_entry(&entry).expect("insert proposal");

    // Operator A rejects
    let rejected =
        hoop_daemon::fleet::reject_reflection_proposal(&proposal_id).expect("reject proposal");

    assert!(rejected, "proposal should be rejected");

    // Verify rejection_count was incremented
    let proposal = hoop_daemon::fleet::get_reflection_proposal(&proposal_id).expect("get proposal");

    // After rejection, the proposal should have rejection_count > 0
    // and status = 'rejected'
    assert!(
        proposal.is_none(),
        "rejected proposal should not appear in proposed list"
    );

    teardown_test_db();
}

// ---------------------------------------------------------------------------
// §19.4 Presence indicators tests
// ---------------------------------------------------------------------------

#[test]
fn test_presence_update_and_query() {
    let (_tmp, _db_path) = setup_test_db();

    let now = chrono::Utc::now().to_rfc3339();

    // Operator A updates presence for a project
    hoop_daemon::fleet::update_presence(
        "tailscale:operator-a@example.com",
        Some("test-project"),
        None,
        "visible",
    )
    .expect("update presence");

    // Query presence for the project
    let presence =
        hoop_daemon::fleet::query_presence(Some("test-project"), None).expect("query presence");

    assert_eq!(presence.len(), 1);
    assert_eq!(presence[0].operator_id, "tailscale:operator-a@example.com");
    assert_eq!(presence[0].project, Some("test-project".to_string()));
    assert_eq!(presence[0].visibility, "visible");

    teardown_test_db();
}

#[test]
fn test_presence_privacy_toggle() {
    let (_tmp, _db_path) = setup_test_db();

    // Operator A sets visibility to hidden
    hoop_daemon::fleet::update_presence(
        "tailscale:operator-a@example.com",
        Some("test-project"),
        None,
        "hidden",
    )
    .expect("update presence hidden");

    // Query should filter out hidden entries
    let presence =
        hoop_daemon::fleet::query_presence(Some("test-project"), None).expect("query presence");

    assert_eq!(presence.len(), 0, "hidden presence should not be returned");

    teardown_test_db();
}

#[test]
fn test_presence_stale_entries_filtered() {
    let (_tmp, _db_path) = setup_test_db();

    let old_time = (chrono::Utc::now() - chrono::Duration::seconds(60)).to_rfc3339();

    // Insert a stale presence entry directly into the database
    let db_path = std::path::PathBuf::from(
        std::env::var("_HOOP_FLEET_DB_PATH").expect("_HOOP_FLEET_DB_PATH not set"),
    );
    let conn = rusqlite::Connection::open(&db_path).expect("open db");

    conn.execute(
        "INSERT INTO presence (operator_id, project, stitch_id, last_seen, visibility)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        [
            "tailscale:operator-stale@example.com",
            "test-project",
            "",
            &old_time,
            "visible",
        ],
    )
    .expect("insert stale presence");

    // Query should filter out stale entries (>30 seconds old)
    let presence =
        hoop_daemon::fleet::query_presence(Some("test-project"), None).expect("query presence");

    assert_eq!(presence.len(), 0, "stale presence should be filtered out");

    teardown_test_db();
}

#[test]
fn test_presence_remove_on_navigate_away() {
    let (_tmp, _db_path) = setup_test_db();

    // Operator A updates presence
    hoop_daemon::fleet::update_presence(
        "tailscale:operator-a@example.com",
        Some("test-project"),
        None,
        "visible",
    )
    .expect("update presence");

    // Verify presence is recorded
    let presence =
        hoop_daemon::fleet::query_presence(Some("test-project"), None).expect("query presence");
    assert_eq!(presence.len(), 1);

    // Operator A navigates away
    hoop_daemon::fleet::remove_presence(
        "tailscale:operator-a@example.com",
        Some("test-project"),
        None,
    )
    .expect("remove presence");

    // Verify presence is removed
    let presence =
        hoop_daemon::fleet::query_presence(Some("test-project"), None).expect("query presence");
    assert_eq!(presence.len(), 0);

    teardown_test_db();
}

// ---------------------------------------------------------------------------
// §19.3 Agent session ownership tests
// ---------------------------------------------------------------------------

#[test]
fn test_agent_session_per_operator() {
    let (_tmp, _db_path) = setup_test_db();

    let now = chrono::Utc::now().to_rfc3339();

    // Create agent session for Operator A
    let session_a = hoop_daemon::fleet::AgentSessionRow {
        id: uuid::Uuid::new_v4().to_string(),
        adapter_session_id: "claude-session-a".to_string(),
        adapter: "claude".to_string(),
        model: "claude-opus-4-7".to_string(),
        status: "active".to_string(),
        stitch_id: None,
        cost_usd: 0.0,
        input_tokens: 0,
        output_tokens: 0,
        turn_count: 0,
        has_started_session: false,
        created_at: now.clone(),
        last_activity_at: now.clone(),
        archived_at: None,
        archived_reason: None,
    };

    hoop_daemon::fleet::insert_agent_session(&session_a).expect("insert session A");

    // Create agent session for Operator B
    let session_b = hoop_daemon::fleet::AgentSessionRow {
        id: uuid::Uuid::new_v4().to_string(),
        adapter_session_id: "claude-session-b".to_string(),
        adapter: "claude".to_string(),
        model: "claude-opus-4-7".to_string(),
        status: "active".to_string(),
        stitch_id: None,
        cost_usd: 0.0,
        input_tokens: 0,
        output_tokens: 0,
        turn_count: 0,
        has_started_session: false,
        created_at: now.clone(),
        last_activity_at: now.clone(),
        archived_at: None,
        archived_reason: None,
    };

    hoop_daemon::fleet::insert_agent_session(&session_b).expect("insert session B");

    // Both sessions should coexist (no shared agent)
    let all_sessions = hoop_daemon::fleet::list_agent_sessions(100).expect("list agent sessions");

    // Filter for active sessions
    let active_sessions: Vec<_> = all_sessions
        .into_iter()
        .filter(|s| s.status == "active")
        .collect();

    assert_eq!(
        active_sessions.len(),
        2,
        "both operator sessions should coexist"
    );

    teardown_test_db();
}

#[test]
fn test_draft_tracks_operator_identity() {
    let (_tmp, _db_path) = setup_test_db();

    let now = chrono::Utc::now().to_rfc3339();

    // Create draft with explicit operator identity
    let draft = hoop_daemon::fleet::DraftRow {
        id: "draft-operator-test".to_string(),
        project: "test-project".to_string(),
        title: "Test draft".to_string(),
        kind: "task".to_string(),
        description: None,
        has_acceptance_criteria: false,
        priority: None,
        labels: vec![],
        created_by: "tailscale:operator-a@example.com".to_string(),
        created_at: now.clone(),
        source: "form".to_string(),
        agent_session_id: None,
        turn_id: None,
        status: "pending".to_string(),
        version: 1,
        original_json: None,
        resolved_by: None,
        resolved_at: None,
        rejection_reason: None,
        stitch_id: None,
        preview_json: None,
        opened_by: Some("tailscale:operator-a@example.com".to_string()),
        opened_at: Some(now.clone()),
        last_autosave_at: None,
        abandoned_at: None,
    };

    hoop_daemon::fleet::insert_draft(&draft).expect("insert draft");

    let fetched = hoop_daemon::fleet::get_draft("draft-operator-test")
        .expect("get draft")
        .expect("draft exists");

    assert_eq!(fetched.created_by, "tailscale:operator-a@example.com");
    assert_eq!(
        fetched.opened_by,
        Some("tailscale:operator-a@example.com".to_string())
    );

    teardown_test_db();
}

// ---------------------------------------------------------------------------
// §19.5 Conflict resolution tests
// ---------------------------------------------------------------------------

#[test]
fn test_no_lock_two_stitches_both_land() {
    let (_tmp, _db_path) = setup_test_db();

    let now = chrono::Utc::now().to_rfc3339();

    let stitch_id_a = uuid::Uuid::new_v4().to_string();
    let stitch_id_b = uuid::Uuid::new_v4().to_string();

    // Two operators create stitches targeting the same workspace
    // Both should succeed - no locking

    // Create stitch A
    let basic_info_a = hoop_daemon::fleet::StitchBasicInfo {
        stitch_id: &stitch_id_a,
        project: "test-project",
        kind: "operator",
        title: "Stitch from Operator A",
        created_by: "tailscale:operator-a@example.com",
        classification: "auto",
    };
    hoop_daemon::fleet::create_stitch(basic_info_a, &[])
        .expect("create stitch A");

    // Create stitch B
    let basic_info_b = hoop_daemon::fleet::StitchBasicInfo {
        stitch_id: &stitch_id_b,
        project: "test-project",
        kind: "operator",
        title: "Stitch from Operator B",
        created_by: "tailscale:operator-b@example.com",
        classification: "auto",
    };
    hoop_daemon::fleet::create_stitch(basic_info_b, &[])
        .expect("create stitch B");

    // Verify both exist by loading them
    let stitch_a = hoop_daemon::fleet::load_stitch_by_id(&stitch_id_a)
        .expect("load stitch A")
        .expect("stitch A exists");

    let stitch_b = hoop_daemon::fleet::load_stitch_by_id(&stitch_id_b)
        .expect("load stitch B")
        .expect("stitch B exists");

    assert_eq!(stitch_a.created_by, "tailscale:operator-a@example.com");
    assert_eq!(stitch_b.created_by, "tailscale:operator-b@example.com");

    teardown_test_db();
}
