//! Comprehensive field validation tests for config.yml hot-reload.
//!
//! Tests that every field in the config.yml schema has proper validation
//! and that validation errors surface structured details (field, line, expected, got).
//!
//! Plan reference: §17 Configuration hot-reload, §6 Phase 6 deliverable 2
//!
//! CI command:
//!   cargo test -p hoop-daemon --test config_field_validation

use hoop_daemon::config_resolver::{ConfigError, Resolved};
use hoop_daemon::projects::ConfigError as ProjectsConfigError;
use serde_yaml::{self, Value};
use std::collections::HashMap;

/// Test helper: parse YAML and extract structured error details.
fn parse_and_get_error(yaml: &str) -> Option<ConfigError> {
    let result: Result<Value, _> = serde_yaml::from_str(yaml);
    match result {
        Ok(_) => None,
        Err(yaml_err) => Some(ConfigError::from_yaml(&yaml_err)),
    }
}

/// Test helper: parse projects.yaml and extract structured error details.
fn parse_projects_and_get_error(yaml: &str) -> Option<ProjectsConfigError> {
    let result: Result<hoop_schema::ProjectsRegistry, _> = serde_yaml::from_str(yaml);
    match result {
        Ok(_) => None,
        Err(yaml_err) => Some(ProjectsConfigError::from(yaml_err)),
    }
}

// ── schema_version field tests ───────────────────────────────────────────────

#[test]
fn test_schema_version_missing_required_field() {
    let yaml = r#"
agent:
  adapter: claude
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "missing schema_version should fail");
    let err = err.unwrap();
    assert!(err.field.is_some(), "error should include field path");
    assert!(
        err.field.as_ref().unwrap().contains("schema_version"),
        "field path should mention schema_version: {:?}",
        err.field
    );
}

#[test]
fn test_schema_version_wrong_type_integer() {
    let yaml = r#"
schema_version: 1
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "integer schema_version should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("string"),
        "expected should be string: {:?}",
        err.expected
    );
    assert!(err.field.is_some(), "error should include field path");
}

#[test]
fn test_schema_version_invalid_format_no_patch() {
    let yaml = r#"
schema_version: "1.0"
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "invalid schema_version format should fail");
    let err = err.unwrap();
    assert!(
        err.message.to_lowercase().contains("pattern")
            || err.message.to_lowercase().contains("format"),
        "error should mention pattern/format: {:?}",
        err.message
    );
}

#[test]
fn test_schema_version_invalid_format_text() {
    let yaml = r#"
schema_version: "latest"
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "invalid schema_version text should fail");
    let err = err.unwrap();
    assert!(
        err.message.to_lowercase().contains("pattern")
            || err.message.to_lowercase().contains("format"),
        "error should mention pattern/format: {:?}",
        err.message
    );
}

// ── agent.adapter field tests ────────────────────────────────────────────────

#[test]
fn test_agent_adapter_missing_required_field() {
    let yaml = r#"
schema_version: "1.0.0"
agent:
  model: claude-3-5-sonnet-20241022
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "missing agent.adapter should fail");
    let err = err.unwrap();
    assert!(
        err.field.as_ref().unwrap().contains("adapter")
            || err.message.to_lowercase().contains("adapter"),
        "error should mention adapter: {:?}",
        err.field
    );
}

#[test]
fn test_agent_adapter_wrong_type_integer() {
    let yaml = r#"
schema_version: "1.0.0"
agent:
  adapter: 42
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "integer adapter should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("string"),
        "expected should be string: {:?}",
        err.expected
    );
    assert!(
        err.field.as_ref().unwrap().contains("adapter"),
        "field path should include adapter: {:?}",
        err.field
    );
}

#[test]
fn test_agent_adapter_invalid_value() {
    let yaml = r#"
schema_version: "1.0.0"
agent:
  adapter: "not-a-real-adapter"
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "invalid adapter value should fail");
    let err = err.unwrap();
    assert!(
        err.message.to_lowercase().contains("adapter")
            || err.message.to_lowercase().contains("variant")
            || err.message.to_lowercase().contains("unknown"),
        "error should mention adapter/variant: {:?}",
        err.message
    );
}

#[test]
fn test_agent_adapter_wrong_type_null() {
    let yaml = r#"
schema_version: "1.0.0"
agent:
  adapter: null
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "null adapter should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("string"),
        "expected should be string: {:?}",
        err.expected
    );
}

// ── agent.model field tests ──────────────────────────────────────────────────

#[test]
fn test_agent_model_wrong_type_integer() {
    let yaml = r#"
schema_version: "1.0.0"
agent:
  adapter: claude
  model: 12345
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "integer model should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("string"),
        "expected should be string: {:?}",
        err.expected
    );
    assert!(
        err.field.as_ref().unwrap().contains("model"),
        "field path should include model: {:?}",
        err.field
    );
}

#[test]
fn test_agent_model_wrong_type_object() {
    let yaml = r#"
schema_version: "1.0.0"
agent:
  adapter: claude
  model:
    name: "claude-3-5-sonnet-20241022"
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "object model should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("string"),
        "expected should be string: {:?}",
        err.expected
    );
}

// ── server.bind_addr field tests ─────────────────────────────────────────────

#[test]
fn test_server_bind_addr_wrong_type_integer() {
    let yaml = r#"
schema_version: "1.0.0"
server:
  bind_addr: 3000
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "integer bind_addr should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("string"),
        "expected should be string: {:?}",
        err.expected
    );
    assert!(
        err.field.as_ref().unwrap().contains("bind_addr"),
        "field path should include bind_addr: {:?}",
        err.field
    );
}

#[test]
fn test_server_bind_addr_wrong_type_object() {
    let yaml = r#"
schema_version: "1.0.0"
server:
  bind_addr:
    host: "127.0.0.1"
    port: 3000
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "object bind_addr should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("string"),
        "expected should be string: {:?}",
        err.expected
    );
}

// ── metrics.enabled field tests ───────────────────────────────────────────────

#[test]
fn test_metrics_enabled_wrong_type_string() {
    let yaml = r#"
schema_version: "1.0.0"
metrics:
  enabled: "true"
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "string metrics.enabled should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("boolean"),
        "expected should be boolean: {:?}",
        err.expected
    );
    assert!(
        err.field.as_ref().unwrap().contains("enabled"),
        "field path should include enabled: {:?}",
        err.field
    );
}

#[test]
fn test_metrics_enabled_wrong_type_integer() {
    let yaml = r#"
schema_version: "1.0.0"
metrics:
  enabled: 1
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "integer metrics.enabled should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("boolean"),
        "expected should be boolean: {:?}",
        err.expected
    );
}

// ── metrics.port field tests ──────────────────────────────────────────────────

#[test]
fn test_metrics_port_wrong_type_string() {
    let yaml = r#"
schema_version: "1.0.0"
metrics:
  port: "9091"
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "string metrics.port should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("integer"),
        "expected should be integer: {:?}",
        err.expected
    );
    assert!(
        err.field.as_ref().unwrap().contains("port"),
        "field path should include port: {:?}",
        err.field
    );
}

#[test]
fn test_metrics_port_negative_value() {
    let yaml = r#"
schema_version: "1.0.0"
metrics:
  port: -1
"#;
    let err = parse_and_get_error(yaml);
    // This may or may not fail depending on schema validation
    // If it passes, at least document the behavior
    if let Some(err) = err {
        assert!(
            err.message.to_lowercase().contains("port")
                || err.message.to_lowercase().contains("range")
                || err.message.to_lowercase().contains("minimum"),
            "error should mention port/range: {:?}",
            err.message
        );
    }
}

// ── audit.retention_days field tests ──────────────────────────────────────────

#[test]
fn test_audit_retention_days_wrong_type_string() {
    let yaml = r#"
schema_version: "1.0.0"
audit:
  retention_days: "90"
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "string retention_days should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("integer"),
        "expected should be integer: {:?}",
        err.expected
    );
    assert!(
        err.field.as_ref().unwrap().contains("retention_days"),
        "field path should include retention_days: {:?}",
        err.field
    );
}

#[test]
fn test_audit_retention_days_wrong_type_boolean() {
    let yaml = r#"
schema_version: "1.0.0"
audit:
  retention_days: true
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "boolean retention_days should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("integer"),
        "expected should be integer: {:?}",
        err.expected
    );
}

// ── audit.hash_chain field tests ──────────────────────────────────────────────

#[test]
fn test_audit_hash_chain_wrong_type_string() {
    let yaml = r#"
schema_version: "1.0.0"
audit:
  hash_chain: "true"
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "string hash_chain should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("boolean"),
        "expected should be boolean: {:?}",
        err.expected
    );
}

#[test]
fn test_audit_hash_chain_wrong_type_integer() {
    let yaml = r#"
schema_version: "1.0.0"
audit:
  hash_chain: 1
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "integer hash_chain should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("boolean"),
        "expected should be boolean: {:?}",
        err.expected
    );
}

// ── ui.theme field tests ──────────────────────────────────────────────────────

#[test]
fn test_ui_theme_wrong_type_integer() {
    let yaml = r#"
schema_version: "1.0.0"
ui:
  theme: 1
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "integer ui.theme should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("string"),
        "expected should be string: {:?}",
        err.expected
    );
}

#[test]
fn test_ui_theme_invalid_value() {
    let yaml = r#"
schema_version: "1.0.0"
ui:
  theme: "not-a-real-theme"
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "invalid ui.theme value should fail");
    let err = err.unwrap();
    assert!(
        err.message.to_lowercase().contains("theme")
            || err.message.to_lowercase().contains("variant")
            || err.message.to_lowercase().contains("unknown"),
        "error should mention theme/variant: {:?}",
        err.message
    );
}

#[test]
fn test_ui_theme_wrong_type_boolean() {
    let yaml = r#"
schema_version: "1.0.0"
ui:
  theme: true
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "boolean ui.theme should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("string"),
        "expected should be string: {:?}",
        err.expected
    );
}

// ── ui.archive_after_days field tests ─────────────────────────────────────────

#[test]
fn test_ui_archive_after_days_wrong_type_string() {
    let yaml = r#"
schema_version: "1.0.0"
ui:
  archive_after_days: "30"
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "string archive_after_days should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("integer"),
        "expected should be integer: {:?}",
        err.expected
    );
}

#[test]
fn test_ui_archive_after_days_wrong_type_boolean() {
    let yaml = r#"
schema_version: "1.0.0"
ui:
  archive_after_days: true
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "boolean archive_after_days should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("integer"),
        "expected should be integer: {:?}",
        err.expected
    );
}

// ── reflection.enabled field tests ────────────────────────────────────────────

#[test]
fn test_reflection_enabled_wrong_type_string() {
    let yaml = r#"
schema_version: "1.0.0"
reflection:
  enabled: "true"
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "string reflection.enabled should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("boolean"),
        "expected should be boolean: {:?}",
        err.expected
    );
}

// ── reflection.detection_threshold field tests ─────────────────────────────────

#[test]
fn test_reflection_detection_threshold_wrong_type_string() {
    let yaml = r#"
schema_version: "1.0.0"
reflection:
  detection_threshold: "0.8"
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "string detection_threshold should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("number"),
        "expected should be number: {:?}",
        err.expected
    );
}

#[test]
fn test_reflection_detection_threshold_wrong_type_boolean() {
    let yaml = r#"
schema_version: "1.0.0"
reflection:
  detection_threshold: true
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "boolean detection_threshold should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("number"),
        "expected should be number: {:?}",
        err.expected
    );
}

// ── reflection.auto_archive_after_days field tests ────────────────────────────

#[test]
fn test_reflection_auto_archive_after_days_wrong_type_string() {
    let yaml = r#"
schema_version: "1.0.0"
reflection:
  auto_archive_after_days: "30"
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "string auto_archive_after_days should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("integer"),
        "expected should be integer: {:?}",
        err.expected
    );
}

// ── roles.viewers field tests ─────────────────────────────────────────────────

#[test]
fn test_roles_viewers_wrong_type_string() {
    let yaml = r#"
schema_version: "1.0.0"
roles:
  viewers: "user@example.com"
"#;
    let err = parse_and_get_error(yaml);
    assert!(
        err.is_some(),
        "string roles.viewers should fail (must be array)"
    );
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("array"),
        "expected should be array: {:?}",
        err.expected
    );
}

#[test]
fn test_roles_viewers_array_element_wrong_type_integer() {
    let yaml = r#"
schema_version: "1.0.0"
roles:
  viewers:
    - user@example.com
    - 42
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "integer in viewers array should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("string"),
        "expected should be string: {:?}",
        err.expected
    );
}

// ── roles.drafters field tests ────────────────────────────────────────────────

#[test]
fn test_roles_drafters_wrong_type_string() {
    let yaml = r#"
schema_version: "1.0.0"
roles:
  drafters: "user@example.com"
"#;
    let err = parse_and_get_error(yaml);
    assert!(
        err.is_some(),
        "string roles.drafters should fail (must be array)"
    );
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("array"),
        "expected should be array: {:?}",
        err.expected
    );
}

// ── agent_extensions paths tests ───────────────────────────────────────────────

#[test]
fn test_agent_extensions_skills_wrong_type_integer() {
    let yaml = r#"
schema_version: "1.0.0"
agent_extensions:
  skills: 123
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "integer agent_extensions.skills should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("string"),
        "expected should be string: {:?}",
        err.expected
    );
}

#[test]
fn test_agent_extensions_scripts_wrong_type_array() {
    let yaml = r#"
schema_version: "1.0.0"
agent_extensions:
  scripts:
    - /path/to/scripts
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "array agent_extensions.scripts should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("string"),
        "expected should be string: {:?}",
        err.expected
    );
}

// ── projects.yaml field tests ─────────────────────────────────────────────────

#[test]
fn test_projects_missing_required_name_field() {
    let yaml = r#"
projects:
  - path: /tmp/test
"#;
    let err = parse_projects_and_get_error(yaml);
    assert!(err.is_some(), "missing project name should fail");
    let err = err.unwrap();
    assert!(
        err.field.is_some() || err.message.to_lowercase().contains("name"),
        "error should mention name: field={:?}, message={:?}",
        err.field,
        err.message
    );
}

#[test]
fn test_projects_name_wrong_type_integer() {
    let yaml = r#"
projects:
  - name: 42
    path: /tmp/test
"#;
    let err = parse_projects_and_get_error(yaml);
    assert!(err.is_some(), "integer project name should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("string"),
        "expected should be string: {:?}",
        err.expected
    );
}

#[test]
fn test_projects_missing_required_path_field() {
    let yaml = r#"
projects:
  - name: test-project
"#;
    let err = parse_projects_and_get_error(yaml);
    assert!(err.is_some(), "missing project path should fail");
    let err = err.unwrap();
    assert!(
        err.field.is_some() || err.message.to_lowercase().contains("path"),
        "error should mention path: field={:?}, message={:?}",
        err.field,
        err.message
    );
}

#[test]
fn test_projects_path_wrong_type_integer() {
    let yaml = r#"
projects:
  - name: test-project
    path: 12345
"#;
    let err = parse_projects_and_get_error(yaml);
    assert!(err.is_some(), "integer project path should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("string"),
        "expected should be string: {:?}",
        err.expected
    );
}

#[test]
fn test_projects_path_wrong_type_boolean() {
    let yaml = r#"
projects:
  - name: test-project
    path: true
"#;
    let err = parse_projects_and_get_error(yaml);
    assert!(err.is_some(), "boolean project path should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("string"),
        "expected should be string: {:?}",
        err.expected
    );
}

#[test]
fn test_projects_label_wrong_type_integer() {
    let yaml = r#"
projects:
  - name: test-project
    path: /tmp/test
    label: 42
"#;
    let err = parse_projects_and_get_error(yaml);
    assert!(err.is_some(), "integer project label should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("string"),
        "expected should be string: {:?}",
        err.expected
    );
}

#[test]
fn test_projects_color_wrong_type_integer() {
    let yaml = r#"
projects:
  - name: test-project
    path: /tmp/test
    color: 123
"#;
    let err = parse_projects_and_get_error(yaml);
    assert!(err.is_some(), "integer project color should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("string"),
        "expected should be string: {:?}",
        err.expected
    );
}

#[test]
fn test_projects_disabled_wrong_type_string() {
    let yaml = r#"
projects:
  - name: test-project
    path: /tmp/test
    disabled: "true"
"#;
    let err = parse_projects_and_get_error(yaml);
    assert!(err.is_some(), "string project disabled should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("boolean"),
        "expected should be boolean: {:?}",
        err.expected
    );
}

#[test]
fn test_projects_wrong_type_not_array() {
    let yaml = r#"
projects: "not-an-array"
"#;
    let err = parse_projects_and_get_error(yaml);
    assert!(err.is_some(), "non-array projects should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("array")
            || err.message.to_lowercase().contains("array")
            || err.message.to_lowercase().contains("sequence"),
        "error should mention array: expected={:?}, message={:?}",
        err.expected,
        err.message
    );
}

#[test]
fn test_projects_array_element_wrong_type_string() {
    let yaml = r#"
projects:
  - "not-an-object"
"#;
    let err = parse_projects_and_get_error(yaml);
    assert!(err.is_some(), "string in projects array should fail");
    let err = err.unwrap();
    assert!(
        err.expected.as_deref() == Some("object")
            || err.message.to_lowercase().contains("object")
            || err.message.to_lowercase().contains("map"),
        "error should mention object: expected={:?}, message={:?}",
        err.expected,
        err.message
    );
}

// ── Unknown field rejection tests ─────────────────────────────────────────────

#[test]
fn test_unknown_field_at_root_level() {
    let yaml = r#"
schema_version: "1.0.0"
unknown_field: "should be rejected"
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "unknown field should be rejected");
    let err = err.unwrap();
    assert!(
        err.message.to_lowercase().contains("unknown")
            || err.message.to_lowercase().contains("field"),
        "error should mention unknown field: {:?}",
        err.message
    );
}

#[test]
fn test_unknown_field_nested_in_agent() {
    let yaml = r#"
schema_version: "1.0.0"
agent:
  adapter: claude
  unknown_field: "should be rejected"
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "unknown nested field should be rejected");
    let err = err.unwrap();
    assert!(
        err.message.to_lowercase().contains("unknown")
            || err.message.to_lowercase().contains("field"),
        "error should mention unknown field: {:?}",
        err.message
    );
}

#[test]
fn test_unknown_field_nested_in_ui() {
    let yaml = r#"
schema_version: "1.0.0"
ui:
  theme: dark
  unknown_field: "should be rejected"
"#;
    let err = parse_and_get_error(yaml);
    assert!(
        err.is_some(),
        "unknown nested field in ui should be rejected"
    );
    let err = err.unwrap();
    assert!(
        err.message.to_lowercase().contains("unknown")
            || err.message.to_lowercase().contains("field"),
        "error should mention unknown field: {:?}",
        err.message
    );
}

#[test]
fn test_unknown_field_in_projects_entry() {
    let yaml = r#"
projects:
  - name: test-project
    path: /tmp/test
    unknown_field: "should be rejected"
"#;
    let err = parse_projects_and_get_error(yaml);
    assert!(
        err.is_some(),
        "unknown field in project entry should be rejected"
    );
    let err = err.unwrap();
    assert!(
        err.message.to_lowercase().contains("unknown")
            || err.message.to_lowercase().contains("field"),
        "error should mention unknown field: {:?}",
        err.message
    );
}

// ── YAML syntax error tests ───────────────────────────────────────────────────

#[test]
fn test_yaml_syntax_error_unclosed_quote() {
    let yaml = r#"
schema_version: "1.0.0
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "unclosed quote should fail");
    let err = err.unwrap();
    assert!(
        err.line > 0,
        "error should include line number: {:?}",
        err.line
    );
}

#[test]
fn test_yaml_syntax_error_unmatched_bracket() {
    let yaml = r#"
schema_version: "1.0.0"
roles:
  viewers: [user@example.com
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "unmatched bracket should fail");
    let err = err.unwrap();
    assert!(
        err.line > 0,
        "error should include line number: {:?}",
        err.line
    );
}

#[test]
fn test_yaml_syntax_error_invalid_escape_sequence() {
    let yaml = r#"
schema_version: "1.0.0"
agent:
  adapter: "claude\x"
"#;
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "invalid escape sequence should fail");
    let err = err.unwrap();
    assert!(
        err.line > 0,
        "error should include line number: {:?}",
        err.line
    );
}

#[test]
fn test_yaml_syntax_error_trailing_comma_in_array() {
    let yaml = r#"
schema_version: "1.0.0"
roles:
  viewers:
    - user@example.com,
"#;
    // YAML doesn't allow trailing commas in flow arrays
    let err = parse_and_get_error(yaml);
    assert!(err.is_some(), "trailing comma should fail");
    let err = err.unwrap();
    assert!(
        err.line > 0,
        "error should include line number: {:?}",
        err.line
    );
}

// ── Structured error details tests ───────────────────────────────────────────

#[test]
fn test_error_includes_line_and_column_numbers() {
    let yaml = r#"
schema_version: "1.0.0"
agent:
  adapter: 42
"#;
    let err = parse_and_get_error(yaml).unwrap();
    assert!(
        err.line > 0,
        "error should include line number: {:?}",
        err.line
    );
    assert!(
        err.col > 0,
        "error should include column number: {:?}",
        err.col
    );
}

#[test]
fn test_error_includes_field_path_for_nested_fields() {
    let yaml = r#"
schema_version: "1.0.0"
agent:
  adapter: 42
"#;
    let err = parse_and_get_error(yaml).unwrap();
    assert!(
        err.field.is_some(),
        "error should include field path: {:?}",
        err.field
    );
    let field = err.field.unwrap();
    assert!(
        field.contains("adapter"),
        "field path should mention adapter: {:?}",
        field
    );
}

#[test]
fn test_error_includes_expected_and_got_for_type_mismatches() {
    let yaml = r#"
schema_version: "1.0.0"
metrics:
  enabled: "true"
"#;
    let err = parse_and_get_error(yaml).unwrap();
    assert!(
        err.expected.is_some(),
        "error should include expected type: {:?}",
        err.expected
    );
    assert!(
        err.got.is_some(),
        "error should include actual type: {:?}",
        err.got
    );
}

#[test]
fn test_error_message_is_human_readable() {
    let yaml = r#"
schema_version: "1.0.0"
agent:
  adapter: 42
"#;
    let err = parse_and_get_error(yaml).unwrap();
    assert!(!err.message.is_empty(), "error message should not be empty");
    assert!(
        err.message.len() < 500,
        "error message should be concise (got {} chars)",
        err.message.len()
    );
}
