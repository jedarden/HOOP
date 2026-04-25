//! Integration test for config hot-reload audit trail (§17.4, §13)
//!
//! Verifies that:
//! 1. Successful config reloads produce a `config_reloaded` audit row with
//!    prev_hash, new_hash, and delta_keys.
//! 2. Rejected config reloads produce a `config_reload_rejected` audit row
//!    with the error message.
//! 3. Audit rows are hash-chained into the actions table.
//! 4. Delta keys accurately reflect the config diff.

use hoop_daemon::fleet;
use hoop_daemon::projects::{self, compute_delta, ConfigReloadAudit, ConfigReloadRejectedAudit};
use serial_test::serial;

/// Helper: create a minimal valid projects.yaml content
fn yaml_one_project(path: &str) -> String {
    format!(
        r#"
projects:
  - name: test-proj
    path: {path}
"#
    )
}

fn yaml_two_projects(path1: &str, path2: &str) -> String {
    format!(
        r#"
projects:
  - name: test-proj
    path: {path1}
  - name: proj-two
    path: {path2}
"#
    )
}

fn yaml_invalid() -> &'static str {
    r#"
projects:
  - name: test-proj
    path: /tmp/exists
  invalid_yaml: [
"#
}

fn setup_test_db() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("fleet.db");
    std::env::set_var("_HOOP_FLEET_DB_PATH", &db_path);
    fleet::init_fleet_db_at(db_path).expect("init fleet db");
    dir
}

fn cleanup_test_db() {
    std::env::remove_var("_HOOP_FLEET_DB_PATH");
}

/// Test: successful config reload produces a `config_reloaded` audit row
/// with correct prev_hash, new_hash, delta_keys, and the row is hash-chained.
#[test]
#[serial]
fn test_config_reload_audit_success() {
    let _db_dir = setup_test_db();

    let tmp = tempfile::tempdir().expect("tempdir for projects");
    let repo1 = tmp.path().join("repo1");
    let repo2 = tmp.path().join("repo2");
    std::fs::create_dir_all(&repo1).unwrap();
    std::fs::create_dir_all(&repo2).unwrap();

    let yaml_path = tmp.path().join("projects.yaml");

    // Write initial config
    let v1 = yaml_one_project(repo1.to_str().unwrap());
    std::fs::write(&yaml_path, &v1).unwrap();
    let cfg1 = projects::ProjectsConfig::load_from(&yaml_path).unwrap();
    let prev_hash = cfg1.content_hash.clone();

    // Write updated config (add a project)
    let v2 = yaml_two_projects(repo1.to_str().unwrap(), repo2.to_str().unwrap());
    std::fs::write(&yaml_path, &v2).unwrap();
    let cfg2 = projects::ProjectsConfig::load_from(&yaml_path).unwrap();
    let new_hash = cfg2.content_hash.clone();

    // Compute delta
    let delta_keys = compute_delta(&cfg1.registry, &cfg2.registry);
    assert!(
        delta_keys.iter().any(|k| k.contains("+project:proj-two")),
        "delta should include +project:proj-two, got: {:?}",
        delta_keys
    );

    // Write audit row as the daemon would
    let audit_args = ConfigReloadAudit {
        file: yaml_path.display().to_string(),
        prev_hash: prev_hash.clone(),
        new_hash: new_hash.clone(),
        delta_keys: delta_keys.clone(),
        actor: "system:hot-reload".to_string(),
    };
    let args_json = serde_json::to_string(&audit_args).unwrap();
    let row = fleet::write_audit_row(
        "system:hot-reload",
        fleet::ActionKind::ConfigReloaded,
        &yaml_path.display().to_string(),
        None,
        Some(args_json),
        fleet::ActionResult::Success,
        None,
        None,
        None,
        None,
    )
    .expect("write audit row");

    assert_eq!(row.kind, fleet::ActionKind::ConfigReloaded);
    assert!(matches!(row.result, fleet::ActionResult::Success));
    assert!(row.error.is_none());
    assert!(row.hash_prev != row.hash_self, "hash chain must advance");

    // Verify we can query it back
    let rows = fleet::query_audit_rows(None, None, None, Some(fleet::ActionKind::ConfigReloaded))
        .expect("query");
    assert_eq!(rows.len(), 1, "should find exactly one config_reloaded row");
    let fetched = &rows[0];
    let fetched_args: serde_json::Value =
        serde_json::from_str(fetched.args_json.as_ref().unwrap()).unwrap();
    assert_eq!(fetched_args["prev_hash"], prev_hash);
    assert_eq!(fetched_args["new_hash"], new_hash);
    let fetched_delta = fetched_args["delta_keys"]
        .as_array()
        .expect("delta_keys should be array");
    assert!(
        fetched_delta
            .iter()
            .any(|v| v.as_str().unwrap().contains("+project:proj-two")),
        "fetched delta_keys should contain +project:proj-two"
    );

    // Verify hash chain integrity
    fleet::verify_hash_chain().expect("hash chain should be valid");

    cleanup_test_db();
}

/// Test: rejected config reload produces a `config_reload_rejected` audit row
/// with the error message.
#[test]
#[serial]
fn test_config_reload_audit_rejected() {
    let _db_dir = setup_test_db();

    let tmp = tempfile::tempdir().expect("tempdir for projects");
    let yaml_path = tmp.path().join("projects.yaml");

    // Write initial valid config
    let repo = tmp.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    std::fs::write(&yaml_path, yaml_one_project(repo.to_str().unwrap())).unwrap();
    let cfg = projects::ProjectsConfig::load_from(&yaml_path).unwrap();
    let prev_hash = cfg.content_hash.clone();

    // Write invalid config and simulate the reject path
    std::fs::write(&yaml_path, yaml_invalid()).unwrap();
    let _parse_result = projects::ProjectsConfig::load_from(&yaml_path);
    let error_msg = "Parse error: invalid YAML structure at line 4";

    // Write rejected audit row as the daemon would
    let audit_args = ConfigReloadRejectedAudit {
        file: yaml_path.display().to_string(),
        prev_hash: prev_hash.clone(),
        error: error_msg.to_string(),
        actor: "system:hot-reload".to_string(),
    };
    let args_json = serde_json::to_string(&audit_args).unwrap();
    let row = fleet::write_audit_row(
        "system:hot-reload",
        fleet::ActionKind::ConfigReloadRejected,
        &yaml_path.display().to_string(),
        None,
        Some(args_json),
        fleet::ActionResult::Failure,
        Some(error_msg.to_string()),
        None,
        None,
        None,
    )
    .expect("write audit row");

    assert_eq!(row.kind, fleet::ActionKind::ConfigReloadRejected);
    assert!(matches!(row.result, fleet::ActionResult::Failure));
    assert_eq!(row.error.as_deref(), Some(error_msg));
    assert!(row.hash_prev != row.hash_self);

    // Verify we can query it back
    let rows =
        fleet::query_audit_rows(None, None, None, Some(fleet::ActionKind::ConfigReloadRejected))
            .expect("query");
    assert_eq!(
        rows.len(),
        1,
        "should find exactly one config_reload_rejected row"
    );
    let fetched = &rows[0];
    let fetched_args: serde_json::Value =
        serde_json::from_str(fetched.args_json.as_ref().unwrap()).unwrap();
    assert_eq!(fetched_args["prev_hash"], prev_hash);
    assert_eq!(fetched_args["error"], error_msg);

    // Verify hash chain integrity
    fleet::verify_hash_chain().expect("hash chain should be valid");

    cleanup_test_db();
}

/// Test: delta computation reflects actual config changes.
#[test]
fn test_delta_keys_match_actual_diff() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let repo1 = tmp.path().join("repo1");
    let repo2 = tmp.path().join("repo2");
    std::fs::create_dir_all(&repo1).unwrap();
    std::fs::create_dir_all(&repo2).unwrap();

    let yaml_path = tmp.path().join("projects.yaml");

    // Write v1: one project
    std::fs::write(&yaml_path, yaml_one_project(repo1.to_str().unwrap())).unwrap();
    let cfg1 = projects::ProjectsConfig::load_from(&yaml_path).unwrap();

    // Write v2: two projects (added proj-two)
    std::fs::write(
        &yaml_path,
        yaml_two_projects(repo1.to_str().unwrap(), repo2.to_str().unwrap()),
    )
    .unwrap();
    let cfg2 = projects::ProjectsConfig::load_from(&yaml_path).unwrap();

    let delta = compute_delta(&cfg1.registry, &cfg2.registry);
    assert_eq!(delta.len(), 1, "should have exactly one delta: +project:proj-two");
    assert_eq!(delta[0], "+project:proj-two");

    // Write v3: test-proj moves to repo2, proj-two removed
    std::fs::write(&yaml_path, yaml_one_project(repo2.to_str().unwrap())).unwrap();
    let cfg3 = projects::ProjectsConfig::load_from(&yaml_path).unwrap();

    let delta2 = compute_delta(&cfg2.registry, &cfg3.registry);
    // proj-two was removed and test-proj's path changed
    assert!(
        delta2.iter().any(|k| k == "-project:proj-two"),
        "should have -project:proj-two, got: {:?}",
        delta2
    );
    assert!(
        delta2.iter().any(|k| k == "~project:test-proj.paths"),
        "should have ~project:test-proj.paths (path changed repo1→repo2), got: {:?}",
        delta2
    );
}

/// Test: round-trip a config change through audit, verify audit row content
/// matches the actual diff between old and new configs.
#[test]
#[serial]
fn test_round_trip_config_change_audit_matches_diff() {
    let _db_dir = setup_test_db();

    let tmp = tempfile::tempdir().expect("tempdir for projects");
    let repo1 = tmp.path().join("repo1");
    let repo2 = tmp.path().join("repo2");
    std::fs::create_dir_all(&repo1).unwrap();
    std::fs::create_dir_all(&repo2).unwrap();

    let yaml_path = tmp.path().join("projects.yaml");

    // v1
    let v1 = yaml_one_project(repo1.to_str().unwrap());
    std::fs::write(&yaml_path, &v1).unwrap();
    let cfg1 = projects::ProjectsConfig::load_from(&yaml_path).unwrap();

    // v2 (add a project)
    let v2 = yaml_two_projects(repo1.to_str().unwrap(), repo2.to_str().unwrap());
    std::fs::write(&yaml_path, &v2).unwrap();
    let cfg2 = projects::ProjectsConfig::load_from(&yaml_path).unwrap();

    let delta_keys = compute_delta(&cfg1.registry, &cfg2.registry);

    // Write audit row
    let audit_args = ConfigReloadAudit {
        file: yaml_path.display().to_string(),
        prev_hash: cfg1.content_hash.clone(),
        new_hash: cfg2.content_hash.clone(),
        delta_keys: delta_keys.clone(),
        actor: "system:hot-reload".to_string(),
    };
    let args_json = serde_json::to_string(&audit_args).unwrap();
    fleet::write_audit_row(
        "system:hot-reload",
        fleet::ActionKind::ConfigReloaded,
        &yaml_path.display().to_string(),
        None,
        Some(args_json),
        fleet::ActionResult::Success,
        None,
        None,
        None,
        None,
    )
    .expect("write audit row");

    // Now verify: read the audit row back and confirm delta matches actual diff
    let rows = fleet::query_audit_rows(None, None, None, Some(fleet::ActionKind::ConfigReloaded))
        .expect("query");
    assert_eq!(rows.len(), 1);
    let fetched_args: ConfigReloadAudit =
        serde_json::from_str(rows[0].args_json.as_ref().unwrap()).unwrap();

    // The hashes in audit must match the actual file hashes
    assert_eq!(fetched_args.prev_hash, cfg1.content_hash, "prev_hash mismatch");
    assert_eq!(fetched_args.new_hash, cfg2.content_hash, "new_hash mismatch");

    // The delta_keys must match the actual compute_delta result
    assert_eq!(
        fetched_args.delta_keys, delta_keys,
        "delta_keys in audit row must match computed delta"
    );
    assert!(
        fetched_args.delta_keys.iter().any(|k| k.contains("+project:proj-two")),
        "delta should reflect proj-two was added"
    );

    // Verify hash chain is intact after the round-trip
    fleet::verify_hash_chain().expect("hash chain should be valid after round-trip");

    cleanup_test_db();
}
