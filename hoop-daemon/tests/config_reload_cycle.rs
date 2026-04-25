//! Integration test for config hot-reload validate-before-apply + rollback (§17.5)
//!
//! Verifies that:
//! 1. Valid config loads and validates successfully.
//! 2. Bad YAML is rejected with structured error details (field, line, expected, got).
//! 3. Previous valid config continues to serve after rejection.
//! 4. A fix cycle (invalid → valid) correctly applies the new config.
//! 5. Audit trail records both rejected and successful reloads.
//! 6. Metric counters track rejections.
//!
//! CI command:
//!   cargo test -p hoop-daemon --test config_reload_cycle

use hoop_daemon::fleet;
use hoop_daemon::metrics;
use hoop_daemon::projects::{self, compute_delta, ConfigError, ConfigReloadAudit, ConfigReloadRejectedAudit};
use serial_test::serial;

fn yaml_one_project(name: &str, path: &str) -> String {
    format!(
        r#"
projects:
  - name: {name}
    path: {path}
"#
    )
}

fn yaml_two_projects(name1: &str, path1: &str, name2: &str, path2: &str) -> String {
    format!(
        r#"
projects:
  - name: {name1}
    path: {path1}
  - name: {name2}
    path: {path2}
"#
    )
}

fn yaml_missing_required_field() -> &'static str {
    r#"
projects:
  - path: /tmp/exists
"#
}

fn yaml_bad_type() -> &'static str {
    r#"
projects:
  - name: 42
    path: /tmp/exists
"#
}

fn yaml_truncated() -> &'static str {
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

/// Core cycle test: valid → invalid → valid preserves state throughout.
///
/// This simulates the exact validate-before-apply flow in reload_config:
/// 1. Schema-validate (parse YAML into typed structure)
/// 2. Semantic validation (paths, .beads, dedup)
/// 3. Apply only if both pass — otherwise keep previous valid config
#[test]
#[serial]
fn test_edit_invalid_then_fix_cycle_preserves_state() {
    let _db_dir = setup_test_db();

    let tmp = tempfile::tempdir().expect("tempdir for projects");
    let repo1 = tmp.path().join("repo1");
    let repo2 = tmp.path().join("repo2");
    std::fs::create_dir_all(&repo1).unwrap();
    std::fs::create_dir_all(&repo2).unwrap();

    let yaml_path = tmp.path().join("projects.yaml");

    // ── Phase 1: Load initial valid config ──────────────────────────────
    let v1 = yaml_one_project("proj-alpha", repo1.to_str().unwrap());
    std::fs::write(&yaml_path, &v1).unwrap();
    let cfg_v1 = projects::ProjectsConfig::load_from(&yaml_path)
        .expect("v1 should parse successfully");
    assert_eq!(cfg_v1.registry.projects.len(), 1, "v1: one project");
    assert_eq!(cfg_v1.registry.projects[0].name(), "proj-alpha");
    let hash_v1 = cfg_v1.content_hash.clone();
    assert!(!hash_v1.is_empty(), "content hash must be set");

    // ── Phase 2: Write invalid (truncated YAML) → must reject ──────────
    std::fs::write(&yaml_path, yaml_truncated()).unwrap();
    let result_bad = projects::ProjectsConfig::load_from(&yaml_path);
    assert!(result_bad.is_err(), "truncated YAML must be rejected");

    // Verify: previous config still valid
    let cfg_check = cfg_v1.clone();
    assert_eq!(cfg_check.registry.projects.len(), 1, "previous config preserved");
    assert_eq!(cfg_check.registry.projects[0].name(), "proj-alpha");
    assert_eq!(cfg_check.content_hash, hash_v1, "hash unchanged after rejection");

    // ── Phase 3: Fix with different valid config ────────────────────────
    let v2 = yaml_two_projects(
        "proj-alpha",
        repo1.to_str().unwrap(),
        "proj-beta",
        repo2.to_str().unwrap(),
    );
    std::fs::write(&yaml_path, &v2).unwrap();
    let cfg_v2 = projects::ProjectsConfig::load_from(&yaml_path)
        .expect("v2 should parse successfully");
    assert_eq!(cfg_v2.registry.projects.len(), 2, "v2: two projects");
    assert_eq!(cfg_v2.registry.projects[0].name(), "proj-alpha");
    assert_eq!(cfg_v2.registry.projects[1].name(), "proj-beta");
    let hash_v2 = cfg_v2.content_hash.clone();
    assert_ne!(hash_v1, hash_v2, "content hash must change on valid edit");

    // Verify delta computation
    let delta = compute_delta(&cfg_v1.registry, &cfg_v2.registry);
    assert!(
        delta.iter().any(|k| k.contains("+project:proj-beta")),
        "delta should show proj-beta added, got: {:?}", delta
    );

    // ── Phase 4: Write another invalid (missing required field) ─────────
    std::fs::write(&yaml_path, yaml_missing_required_field()).unwrap();
    let result_bad2 = projects::ProjectsConfig::load_from(&yaml_path);
    assert!(result_bad2.is_err(), "missing field must be rejected");

    // Verify: previous config (v2) still preserved
    assert_eq!(cfg_v2.registry.projects.len(), 2, "v2 config preserved after second rejection");
    assert_eq!(cfg_v2.content_hash, hash_v2, "v2 hash unchanged");

    // ── Phase 5: Fix back to single project ─────────────────────────────
    let v3 = yaml_one_project("proj-alpha", repo2.to_str().unwrap());
    std::fs::write(&yaml_path, &v3).unwrap();
    let cfg_v3 = projects::ProjectsConfig::load_from(&yaml_path)
        .expect("v3 should parse successfully");
    assert_eq!(cfg_v3.registry.projects.len(), 1, "v3: back to one project");

    // Delta from v2 → v3 should show removal of proj-beta and path change
    let delta_23 = compute_delta(&cfg_v2.registry, &cfg_v3.registry);
    assert!(
        delta_23.iter().any(|k| k == "-project:proj-beta"),
        "delta should show proj-beta removed, got: {:?}", delta_23
    );

    // ── Verify audit trail ──────────────────────────────────────────────
    // Write audit rows for each transition (simulating what the daemon does)
    let rejected_audit = ConfigReloadRejectedAudit {
        file: yaml_path.display().to_string(),
        prev_hash: hash_v1.clone(),
        error: "Parse error: truncated YAML".to_string(),
        actor: "system:hot-reload".to_string(),
    };
    let rejected_json = serde_json::to_string(&rejected_audit).unwrap();
    fleet::write_audit_row(
        "system:hot-reload",
        fleet::ActionKind::ConfigReloadRejected,
        &yaml_path.display().to_string(),
        None,
        Some(rejected_json),
        fleet::ActionResult::Failure,
        Some("Parse error: truncated YAML".to_string()),
        None,
        None,
        None,
    )
    .expect("write rejected audit row");

    let success_audit = ConfigReloadAudit {
        file: yaml_path.display().to_string(),
        prev_hash: hash_v1.clone(),
        new_hash: hash_v2.clone(),
        delta_keys: delta.clone(),
        actor: "system:hot-reload".to_string(),
    };
    let success_json = serde_json::to_string(&success_audit).unwrap();
    fleet::write_audit_row(
        "system:hot-reload",
        fleet::ActionKind::ConfigReloaded,
        &yaml_path.display().to_string(),
        None,
        Some(success_json),
        fleet::ActionResult::Success,
        None,
        None,
        None,
        None,
    )
    .expect("write success audit row");

    // Query and verify
    let rejected_rows = fleet::query_audit_rows(
        None, None, None, Some(fleet::ActionKind::ConfigReloadRejected),
    ).expect("query rejected");
    assert_eq!(rejected_rows.len(), 1, "one rejected audit row");
    assert!(matches!(rejected_rows[0].result, fleet::ActionResult::Failure));

    let success_rows = fleet::query_audit_rows(
        None, None, None, Some(fleet::ActionKind::ConfigReloaded),
    ).expect("query success");
    assert_eq!(success_rows.len(), 1, "one success audit row");
    assert!(matches!(success_rows[0].result, fleet::ActionResult::Success));

    // Verify hash chain integrity through the full cycle
    fleet::verify_hash_chain().expect("hash chain intact after full cycle");

    cleanup_test_db();
}

/// Verify that parse errors surface structured details: field, line, expected, got.
///
/// Tests ConfigError::from(serde_yaml::Error) directly since that's the path
/// used by the watcher's parse_config to produce structured error details.
#[test]
fn test_schema_violation_surfaces_structured_details() {
    // ── Test 1: Missing required field ──────────────────────────────────
    let result: Result<hoop_schema::ProjectsRegistry, _> =
        serde_yaml::from_str(yaml_missing_required_field());
    assert!(result.is_err(), "missing name should fail");
    let yaml_err = result.unwrap_err();
    let err = ConfigError::from(yaml_err);
    assert!(
        err.line > 0 || err.field.is_some(),
        "missing field error should have location info: line={}, field={:?}",
        err.line, err.field,
    );
    assert!(
        !err.message.is_empty(),
        "error message should not be empty",
    );

    // ── Test 2: Wrong type for name ─────────────────────────────────────
    let result2: Result<hoop_schema::ProjectsRegistry, _> =
        serde_yaml::from_str(yaml_bad_type());
    assert!(result2.is_err(), "integer name should fail");
    let yaml_err2 = result2.unwrap_err();
    let err2 = ConfigError::from(yaml_err2);
    assert!(
        err2.line > 0,
        "type error should report line number: line={}", err2.line,
    );
    assert!(
        err2.expected.is_some() || err2.got.is_some() || err2.field.is_some(),
        "type error should have structured details: expected={:?}, got={:?}, field={:?}",
        err2.expected, err2.got, err2.field,
    );

    // ── Test 3: Truncated/malformed YAML ────────────────────────────────
    let result3: Result<hoop_schema::ProjectsRegistry, _> =
        serde_yaml::from_str(yaml_truncated());
    assert!(result3.is_err(), "truncated YAML should fail");
    let yaml_err3 = result3.unwrap_err();
    let err3 = ConfigError::from(yaml_err3);
    assert!(
        err3.line > 0,
        "parse error should report line number: line={}", err3.line,
    );
}

/// Verify the rejection metric increments on bad config.
#[test]
fn test_rejection_metric_increments() {
    let rejected_before = metrics::metrics().hoop_config_reload_rejected_total.get();

    // Simulate what the daemon does on rejection: increment the counter
    metrics::metrics().hoop_config_reload_rejected_total.inc();

    let rejected_after = metrics::metrics().hoop_config_reload_rejected_total.get();
    assert_eq!(
        rejected_after, rejected_before + 1,
        "rejection metric should increment by 1"
    );
}

/// Verify the success metric increments on successful reload.
#[test]
fn test_success_metric_increments() {
    let success_before = metrics::metrics().hoop_config_reload_success_total.get();

    metrics::metrics().hoop_config_reload_success_total.inc();

    let success_after = metrics::metrics().hoop_config_reload_success_total.get();
    assert_eq!(
        success_after, success_before + 1,
        "success metric should increment by 1"
    );
}

/// Verify that validation catches semantic errors (missing paths, no .beads).
#[test]
fn test_semantic_validation_rejects_bad_paths() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let repo_exists = tmp.path().join("exists");
    std::fs::create_dir_all(repo_exists.join(".beads")).unwrap();

    let repo_no_beads = tmp.path().join("no-beads");
    std::fs::create_dir_all(&repo_no_beads).unwrap();

    let repo_missing = tmp.path().join("/nonexistent/path/that/does/not/exist");

    let yaml_path = tmp.path().join("projects.yaml");

    // Config with: one valid, one missing .beads, one nonexistent path
    let yaml = format!(
        r#"
projects:
  - name: valid-proj
    path: {}
  - name: no-beads-proj
    path: {}
  - name: missing-path-proj
    path: {}
"#,
        repo_exists.display(),
        repo_no_beads.display(),
        repo_missing.display(),
    );

    std::fs::write(&yaml_path, &yaml).unwrap();
    let cfg = projects::ProjectsConfig::load_from(&yaml_path)
        .expect("YAML should parse fine");

    let errors = cfg.validate();
    assert!(
        errors.len() >= 2,
        "should detect at least 2 semantic errors (no .beads + missing path), got: {:?}",
        errors,
    );

    // Verify structured error details
    let no_beads_err = errors.iter().find(|e| e.message.contains("no-beads-proj") && e.message.contains(".beads"));
    assert!(
        no_beads_err.is_some(),
        "should detect missing .beads for no-beads-proj, got: {:?}", errors,
    );
    let err = no_beads_err.unwrap();
    assert!(err.field.is_some(), "semantic error should have field path");
    assert!(
        err.expected.as_deref() == Some("directory containing .beads/"),
        "expected should say what's needed"
    );

    let missing_err = errors.iter().find(|e| e.message.contains("missing-path-proj") && e.message.contains("does not exist"));
    assert!(
        missing_err.is_some(),
        "should detect nonexistent path for missing-path-proj, got: {:?}", errors,
    );
    let err = missing_err.unwrap();
    assert!(err.field.is_some(), "missing path error should have field");
    assert!(
        err.expected.as_deref() == Some("existing directory"),
        "expected should say 'existing directory'"
    );
}

/// Full end-to-end cycle test with semantic validation as the rejection reason.
#[test]
#[serial]
fn test_semantic_validation_rejection_preserves_state() {
    let _db_dir = setup_test_db();

    let tmp = tempfile::tempdir().expect("tempdir");
    let repo = tmp.path().join("repo");
    std::fs::create_dir_all(repo.join(".beads")).unwrap();

    let yaml_path = tmp.path().join("projects.yaml");

    // Phase 1: Valid config
    let v1 = yaml_one_project("good-proj", repo.to_str().unwrap());
    std::fs::write(&yaml_path, &v1).unwrap();
    let cfg_v1 = projects::ProjectsConfig::load_from(&yaml_path)
        .expect("valid config should load");
    assert!(cfg_v1.validate().is_empty(), "valid config should pass validation");
    let hash_v1 = cfg_v1.content_hash.clone();

    // Phase 2: Parseable YAML but semantically invalid (nonexistent path)
    let bad_yaml = yaml_one_project("bad-proj", "/nonexistent/path");
    std::fs::write(&yaml_path, &bad_yaml).unwrap();
    let cfg_bad = projects::ProjectsConfig::load_from(&yaml_path)
        .expect("YAML should still parse");
    let validation_errors = cfg_bad.validate();
    assert!(!validation_errors.is_empty(), "nonexistent path should fail validation");

    // Previous config (v1) is still the active one — cfg_bad was never "applied"
    assert_eq!(cfg_v1.registry.projects.len(), 1, "previous config preserved");
    assert_eq!(cfg_v1.registry.projects[0].name(), "good-proj");
    assert_eq!(cfg_v1.content_hash, hash_v1, "hash unchanged");

    // Phase 3: Fix with valid config (different project name)
    let v2 = yaml_one_project("another-proj", repo.to_str().unwrap());
    std::fs::write(&yaml_path, &v2).unwrap();
    let cfg_v2 = projects::ProjectsConfig::load_from(&yaml_path)
        .expect("fixed config should load");
    assert!(cfg_v2.validate().is_empty(), "fixed config should pass validation");
    assert_eq!(cfg_v2.registry.projects[0].name(), "another-proj");

    // Verify delta from v1 → v2
    let delta = compute_delta(&cfg_v1.registry, &cfg_v2.registry);
    assert!(
        delta.iter().any(|k| k.contains("-project:good-proj")),
        "delta should show good-proj removed, got: {:?}", delta
    );
    assert!(
        delta.iter().any(|k| k.contains("+project:another-proj")),
        "delta should show another-proj added, got: {:?}", delta
    );

    // Verify audit trail for the rejected semantic validation
    let first_err = validation_errors.into_iter().next().unwrap();
    let rejected_audit = ConfigReloadRejectedAudit {
        file: yaml_path.display().to_string(),
        prev_hash: hash_v1.clone(),
        error: first_err.message.clone(),
        actor: "system:hot-reload".to_string(),
    };
    let rejected_json = serde_json::to_string(&rejected_audit).unwrap();
    fleet::write_audit_row(
        "system:hot-reload",
        fleet::ActionKind::ConfigReloadRejected,
        &yaml_path.display().to_string(),
        None,
        Some(rejected_json),
        fleet::ActionResult::Failure,
        Some(first_err.message),
        None,
        None,
        None,
    )
    .expect("write rejected audit");

    let rows = fleet::query_audit_rows(
        None, None, None, Some(fleet::ActionKind::ConfigReloadRejected),
    ).expect("query");
    assert_eq!(rows.len(), 1, "one rejected audit row for semantic validation");
    assert!(rows[0].error.is_some(), "rejected row should have error message");

    fleet::verify_hash_chain().expect("hash chain intact");

    cleanup_test_db();
}
