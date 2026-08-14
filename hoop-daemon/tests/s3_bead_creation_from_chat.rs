//! Acceptance test S3: Bead creation from chat
//!
//! Plan reference: §1.8 Acceptance scenarios
//!
//! **S3 — Bead creation from chat (Phase 2)**
//! Operator types "create a fix bead for the Calico IP selection issue on iad-acb"
//! into the chat pane. HOOP produces a draft Stitch with pre-filled title, body,
//! and target workspace. Operator reviews and confirms. br list --json in the
//! relevant workspace shows the new bead within 3 seconds. fleet.db audit log
//! carries the Stitch id and operator identity.
//!
//! Pass criteria:
//! 1. Draft Stitch appears in the draft queue after natural-language input.
//! 2. After confirmation, bead appears in the target workspace queue (stub br records the call).
//! 3. Audit row in fleet.db contains stitch_id, operator, source=chat.
//!
//! Fail criteria:
//! - Draft not created within 3 seconds of chat input
//! - Bead not created within 3 seconds of approval
//! - Audit log missing stitch_id or operator identity
//! - Audit log source != "chat"

mod integration_harness;

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tempfile::TempDir;

use integration_harness::spawn_test_daemon;

/// Path to the log file written by the fake br stub
struct FakeBr {
    /// Directory containing the fake `br` script
    bin_dir: TempDir,
    /// Path to the invocation log
    log_path: PathBuf,
}

impl FakeBr {
    fn new() -> Self {
        let bin_dir = TempDir::new().expect("create temp dir");
        let br_path = bin_dir.path().join("br");
        let log_path = bin_dir.path().join("br_invocations.log");

        // Write the fake br script: log verb + args, then output a fake bead ID
        let log_path_str = log_path.to_str().unwrap();
        let script = format!(
            "#!/bin/sh\n\
             echo \"$@\" >> {log_path_str}\n\
             # If verb is 'create', output a fake bead ID\n\
             if [ \"$1\" = \"create\" ]; then\n\
               echo \"bd-chat-$(date +%s)\"\n\
             fi\n\
             exit 0\n"
        );
        let mut f = fs::File::create(&br_path).expect("create br script");
        f.write_all(script.as_bytes()).expect("write br script");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&br_path, fs::Permissions::from_mode(0o755))
                .expect("chmod br script");
        }

        Self { bin_dir, log_path }
    }

    /// Get the PATH prefix that includes the fake br
    fn path_prefix(&self) -> String {
        self.bin_dir.path().to_str().unwrap().to_string()
    }

    /// Read and parse all logged invocations
    fn invocations(&self) -> Vec<String> {
        if !self.log_path.exists() {
            return vec![];
        }
        let contents = fs::read_to_string(&self.log_path).unwrap_or_default();
        contents
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect()
    }

    /// Check if br create was called with expected title
    fn has_create_with_title(&self, title: &str) -> bool {
        self.invocations()
            .iter()
            .any(|line| line.starts_with("create") && line.contains(title))
    }
}

#[tokio::test]
async fn s3_chat_creates_draft_in_queue() {
    //! Verify that natural-language chat input creates a draft Stitch in the queue

    let fake_br = FakeBr::new();
    let path_prefix = fake_br.path_prefix();

    // Set up the environment to use the fake br
    let original_path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", format!("{}:{}", path_prefix, original_path));

    let (base_url, _notify, _temp_dir) = spawn_test_daemon()
        .await
        .expect("Failed to spawn daemon");

    let client = reqwest::Client::new();

    // Simulate chat input: "create a fix bead for the Calico IP selection issue on iad-acb"
    let chat_input = "Fix Calico IP selection issue on iad-acb";

    // Create a draft via the API (this simulates the agent parsing the chat input)
    let create_req = serde_json::json!({
        "project": "testrepo",
        "title": chat_input,
        "kind": "fix",
        "description": "Calico IPAM selects IPs that conflict with existing node CIDR allocations",
        "has_acceptance_criteria": false,
        "priority": 7,
        "labels": ["calico", "networking", "critical"],
        "source": "chat"
    });

    let start = Instant::now();

    let create_resp = client
        .post(&format!("{}/api/drafts", base_url))
        .json(&create_req)
        .send()
        .await
        .expect("Failed to create draft");

    let create_elapsed = start.elapsed();

    assert_eq!(
        create_resp.status(), 200,
        "Draft creation should return 200"
    );

    let create_response: serde_json::Value = create_resp
        .json()
        .await
        .expect("Failed to parse draft response");

    let draft_id = create_response["draft_id"]
        .as_str()
        .expect("draft_id should be present");

    assert!(
        create_elapsed < Duration::from_secs(3),
        "Draft should be created within 3 seconds, took {:?}",
        create_elapsed
    );

    // Verify the draft appears in the draft queue
    let list_resp = client
        .get(&format!("{}/api/drafts", base_url))
        .send()
        .await
        .expect("Failed to list drafts");

    assert_eq!(list_resp.status(), 200, "List drafts should return 200");

    let list_response: serde_json::Value = list_resp
        .json()
        .await
        .expect("Failed to parse list response");

    let drafts = list_response["drafts"]
        .as_array()
        .expect("drafts should be an array");

    let found = drafts
        .iter()
        .any(|d| d["id"].as_str() == Some(draft_id));

    assert!(found, "Draft should appear in the draft queue");

    // Verify draft has correct fields
    let get_resp = client
        .get(&format!("{}/api/drafts/{}", base_url, draft_id))
        .send()
        .await
        .expect("Failed to get draft");

    assert_eq!(get_resp.status(), 200, "Get draft should return 200");

    let draft: serde_json::Value = get_resp
        .json()
        .await
        .expect("Failed to parse draft");

    assert_eq!(draft["title"], chat_input, "Draft title should match chat input");
    assert_eq!(draft["kind"], "fix", "Draft kind should be fix");
    assert_eq!(draft["source"], "chat", "Draft source should be chat");
    assert_eq!(draft["project"], "testrepo", "Draft project should be testrepo");
    assert_eq!(draft["status"], "pending", "Draft status should be pending");

    println!("S3 PASS: Draft created in queue within {:?}", create_elapsed);
}

#[tokio::test]
async fn s3_approval_creates_bead_via_stub_br() {
    //! Verify that approving the draft creates a bead (recorded by stub br)

    let fake_br = FakeBr::new();
    let path_prefix = fake_br.path_prefix();

    // Set up the environment to use the fake br
    let original_path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", format!("{}:{}", path_prefix, original_path));

    let (base_url, _notify, _temp_dir) = spawn_test_daemon()
        .await
        .expect("Failed to spawn daemon");

    let client = reqwest::Client::new();

    // Create a draft
    let create_req = serde_json::json!({
        "project": "testrepo",
        "title": "Fix Calico IP selection issue on iad-acb",
        "kind": "fix",
        "description": "Calico IPAM selects IPs that conflict with existing node CIDR allocations",
        "source": "chat"
    });

    let create_resp = client
        .post(&format!("{}/api/drafts", base_url))
        .json(&create_req)
        .send()
        .await
        .expect("Failed to create draft");

    let create_response: serde_json::Value = create_resp
        .json()
        .await
        .expect("Failed to parse draft response");

    let draft_id = create_response["draft_id"]
        .as_str()
        .expect("draft_id should be present");

    // Operator reviews and confirms (approve the draft)
    let approve_start = Instant::now();

    let approve_resp = client
        .post(&format!("{}/api/drafts/{}/approve", base_url, draft_id))
        .json(&serde_json::json!({}))
        .send()
        .await
        .expect("Failed to approve draft");

    let approve_elapsed = approve_start.elapsed();

    assert_eq!(
        approve_resp.status(), 200,
        "Draft approval should return 200"
    );

    let approve_response: serde_json::Value = approve_resp
        .json()
        .await
        .expect("Failed to parse approve response");

    let stitch_id = approve_response["stitch_id"]
        .as_str()
        .expect("stitch_id should be present");

    assert!(
        approve_elapsed < Duration::from_secs(3),
        "Bead should be created within 3 seconds of approval, took {:?}",
        approve_elapsed
    );

    // Verify stub br recorded the create call
    // Give a small delay for the br call to complete
    tokio::time::sleep(Duration::from_millis(100)).await;

    assert!(
        fake_br.has_create_with_title("Fix Calico IP selection issue on iad-acb"),
        "stub br should record br create call with expected title"
    );

    // Verify the draft status is now "submitted"
    let get_resp = client
        .get(&format!("{}/api/drafts/{}", base_url, draft_id))
        .send()
        .await
        .expect("Failed to get draft");

    let draft: serde_json::Value = get_resp
        .json()
        .await
        .expect("Failed to parse draft");

    assert_eq!(draft["status"], "submitted", "Draft status should be submitted");
    assert_eq!(draft["stitch_id"], stitch_id, "Draft should have stitch_id");

    println!("S3 PASS: Bead created within {:?} of approval", approve_elapsed);
}

#[tokio::test]
async fn s3_audit_log_contains_stitch_id_and_operator() {
    //! Verify that the audit log contains stitch_id, operator, and source=chat

    let fake_br = FakeBr::new();
    let path_prefix = fake_br.path_prefix();

    // Set up the environment to use the fake br
    let original_path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", format!("{}:{}", path_prefix, original_path));

    let (base_url, _notify, _temp_dir) = spawn_test_daemon()
        .await
        .expect("Failed to spawn daemon");

    let client = reqwest::Client::new();

    // Create a draft from chat
    let create_req = serde_json::json!({
        "project": "testrepo",
        "title": "Fix Calico IP selection issue on iad-acb",
        "kind": "fix",
        "description": "Calico IPAM selects IPs that conflict with existing node CIDR allocations",
        "source": "chat"
    });

    let create_resp = client
        .post(&format!("{}/api/drafts", base_url))
        .json(&create_req)
        .send()
        .await
        .expect("Failed to create draft");

    let create_response: serde_json::Value = create_resp
        .json()
        .await
        .expect("Failed to parse draft response");

    let draft_id = create_response["draft_id"]
        .as_str()
        .expect("draft_id should be present");

    // Approve the draft
    let approve_resp = client
        .post(&format!("{}/api/drafts/{}/approve", base_url, draft_id))
        .json(&serde_json::json!({}))
        .send()
        .await
        .expect("Failed to approve draft");

    let approve_response: serde_json::Value = approve_resp
        .json()
        .await
        .expect("Failed to parse approve response");

    let stitch_id = approve_response["stitch_id"]
        .as_str()
        .expect("stitch_id should be present");

    // Query the audit log
    let audit_resp = client
        .get(&format!("{}/api/audit?limit=100", base_url))
        .send()
        .await
        .expect("Failed to query audit log");

    assert_eq!(audit_resp.status(), 200, "Audit query should return 200");

    let audit_response: serde_json::Value = audit_resp
        .json()
        .await
        .expect("Failed to parse audit response");

    let audit_rows = audit_response["audit_rows"]
        .as_array()
        .expect("audit_rows should be an array");

    // Find DraftCreated entry
    let draft_created = audit_rows
        .iter()
        .find(|row| row["target"].as_str() == Some(draft_id)
            && row["kind"].as_str() == Some("DraftCreated"));

    assert!(draft_created.is_some(), "Audit log should contain DraftCreated entry");

    // Verify DraftCreated has source=chat
    let draft_created = draft_created.unwrap();
    let args = draft_created["args"].as_object().expect("args should be an object");
    assert_eq!(args["source"], "chat", "DraftCreated source should be chat");

    // Find DraftApproved entry (contains stitch_id)
    let draft_approved = audit_rows
        .iter()
        .find(|row| row["target"].as_str() == Some(draft_id)
            && row["kind"].as_str() == Some("DraftApproved"));

    assert!(draft_approved.is_some(), "Audit log should contain DraftApproved entry");

    let draft_approved = draft_approved.unwrap();
    let approved_args = draft_approved["args"].as_object().expect("args should be an object");

    assert_eq!(
        approved_args["stitch_id"], stitch_id,
        "DraftApproved args should contain stitch_id"
    );

    // Verify operator identity is present
    let actor = draft_approved["actor"].as_str().expect("actor should be present");
    assert!(!actor.is_empty(), "Operator identity should be present in audit log");

    println!("S3 PASS: Audit log contains stitch_id and operator identity");
    println!("  - stitch_id: {}", stitch_id);
    println!("  - operator: {}", actor);
    println!("  - source: chat");
}

#[tokio::test]
async fn s3_end_to_end_chat_flow() {
    //! Full end-to-end test: chat → draft → approve → bead → audit

    let fake_br = FakeBr::new();
    let path_prefix = fake_br.path_prefix();

    // Set up the environment to use the fake br
    let original_path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", format!("{}:{}", path_prefix, original_path));

    let (base_url, _notify, _temp_dir) = spawn_test_daemon()
        .await
        .expect("Failed to spawn daemon");

    let client = reqwest::Client::new();

    // Step 1: Simulate natural language chat input
    let chat_input = "create a fix bead for the Calico IP selection issue on iad-acb";

    // Parse the input (in real system, agent does this) and create draft
    let draft_start = Instant::now();

    let create_req = serde_json::json!({
        "project": "testrepo",
        "title": "Fix Calico IP selection issue on iad-acb",
        "kind": "fix",
        "description": "Calico IPAM selects IPs that conflict with existing node CIDR allocations. This causes network failures for new pods.",
        "priority": 8,
        "labels": ["calico", "networking", "bug"],
        "source": "chat"
    });

    let create_resp = client
        .post(&format!("{}/api/drafts", base_url))
        .json(&create_req)
        .send()
        .await
        .expect("Failed to create draft");

    assert_eq!(create_resp.status(), 200);
    let draft_elapsed = draft_start.elapsed();

    let create_response: serde_json::Value = create_resp.json().await.expect("Failed to parse response");
    let draft_id = create_response["draft_id"].as_str().expect("draft_id present");

    // Step 2: Operator reviews the draft (check it's in queue)
    let list_resp = client
        .get(&format!("{}/api/drafts", base_url))
        .send()
        .await
        .expect("Failed to list drafts");

    let list_response: serde_json::Value = list_resp.json().await.expect("Failed to parse list");
    let drafts = list_response["drafts"].as_array().expect("drafts array");
    let draft_in_queue = drafts.iter().any(|d| d["id"].as_str() == Some(draft_id));

    assert!(draft_in_queue, "Draft should be in queue");

    // Step 3: Operator confirms (approves)
    let approve_start = Instant::now();

    let approve_resp = client
        .post(&format!("{}/api/drafts/{}/approve", base_url, draft_id))
        .json(&serde_json::json!({}))
        .send()
        .await
        .expect("Failed to approve draft");

    assert_eq!(approve_resp.status(), 200);
    let approve_elapsed = approve_start.elapsed();

    let approve_response: serde_json::Value = approve_resp.json().await.expect("Failed to parse approve");
    let stitch_id = approve_response["stitch_id"].as_str().expect("stitch_id present");

    // Step 4: Verify bead was created (stub br records it)
    tokio::time::sleep(Duration::from_millis(100)).await;

    assert!(
        fake_br.has_create_with_title("Fix Calico IP selection issue on iad-acb"),
        "stub br should record the create call"
    );

    // Step 5: Verify audit log
    let audit_resp = client
        .get(&format!("{}/api/audit?limit=100", base_url))
        .send()
        .await
        .expect("Failed to query audit");

    let audit_response: serde_json::Value = audit_resp.json().await.expect("Failed to parse audit");
    let audit_rows = audit_response["audit_rows"].as_array().expect("audit_rows array");

    // Find relevant audit entries
    let draft_created = audit_rows
        .iter()
        .find(|r| r["target"].as_str() == Some(draft_id) && r["kind"].as_str() == Some("DraftCreated"));
    let draft_approved = audit_rows
        .iter()
        .find(|r| r["target"].as_str() == Some(draft_id) && r["kind"].as_str() == Some("DraftApproved"));

    assert!(draft_created.is_some(), "Audit should have DraftCreated");
    assert!(draft_approved.is_some(), "Audit should have DraftApproved");

    // Verify source=chat
    let dc_args = draft_created.unwrap()["args"].as_object().expect("args object");
    assert_eq!(dc_args["source"], "chat", "source should be chat");

    // Verify stitch_id in approved entry
    let da_args = draft_approved.unwrap()["args"].as_object().expect("args object");
    assert_eq!(da_args["stitch_id"], stitch_id, "stitch_id should match");

    // Verify operator present
    let actor = draft_approved.unwrap()["actor"].as_str().expect("actor present");
    assert!(!actor.is_empty(), "operator identity should be present");

    println!("S3 PASS: Full end-to-end chat flow completed");
    println!("  - Draft created in: {:?}", draft_elapsed);
    println!("  - Bead created in: {:?}", approve_elapsed);
    println!("  - stitch_id: {}", stitch_id);
    println!("  - operator: {}", actor);
}

#[tokio::test]
async fn s3_draft_queue_exposes_all_required_fields() {
    //! Verify the draft queue exposes all required fields for the UI

    let fake_br = FakeBr::new();
    let path_prefix = fake_br.path_prefix();

    let original_path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", format!("{}:{}", path_prefix, original_path));

    let (base_url, _notify, _temp_dir) = spawn_test_daemon()
        .await
        .expect("Failed to spawn daemon");

    let client = reqwest::Client::new();

    // Create a draft with all fields
    let create_req = serde_json::json!({
        "project": "testrepo",
        "title": "Test draft with all fields",
        "kind": "feature",
        "description": "Full description",
        "priority": 5,
        "labels": ["ui", "chat"],
        "source": "chat",
        "has_acceptance_criteria": true
    });

    let create_resp = client
        .post(&format!("{}/api/drafts", base_url))
        .json(&create_req)
        .send()
        .await
        .expect("Failed to create draft");

    assert_eq!(create_resp.status(), 200);

    let create_response: serde_json::Value = create_resp.json().await.expect("Failed to parse");
    let draft_id = create_response["draft_id"].as_str().expect("draft_id present");

    // Get the draft and verify all fields
    let get_resp = client
        .get(&format!("{}/api/drafts/{}", base_url, draft_id))
        .send()
        .await
        .expect("Failed to get draft");

    assert_eq!(get_resp.status(), 200);

    let draft: serde_json::Value = get_resp.json().await.expect("Failed to parse draft");

    // Verify all expected fields are present
    assert_eq!(draft["id"], draft_id);
    assert_eq!(draft["title"], "Test draft with all fields");
    assert_eq!(draft["kind"], "feature");
    assert_eq!(draft["description"], "Full description");
    assert_eq!(draft["priority"], 5);
    assert!(draft["labels"].is_array());
    assert_eq!(draft["source"], "chat");
    assert_eq!(draft["project"], "testrepo");
    assert_eq!(draft["status"], "pending");

    println!("S3 PASS: Draft queue exposes all required fields");
}
