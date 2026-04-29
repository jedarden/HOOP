//! Skill invocation integration tests — manifest schema and JSON-Schema validation (§22.2)
//!
//! Validates acceptance criteria:
//! - Manifest parsing with args_schema
//! - Skill invocation with valid args succeeds
//! - Skill invocation with invalid args fails fast with readable error
//! - Agent tool-belt augmented with skill description

use hoop_daemon::api_skills;
use serde_json::json;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_skill_manifest_parsing() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let skill_dir = temp_dir.path().join("test-skill");
    fs::create_dir(&skill_dir).expect("Failed to create skill dir");

    let manifest_content = r#"
name: test-skill
description: A test skill for manifest parsing
summary: Test summary
scope: global
args_schema:
  type: object
  properties:
    url:
      type: string
      format: uri
    count:
      type: number
      minimum: 1
      maximum: 100
  required:
    - url
timeout_secs: 60
"#;

    fs::write(skill_dir.join("manifest.yml"), manifest_content)
        .expect("Failed to write manifest");

    let skills = api_skills::discover_skills(temp_dir.path());
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].name, "test-skill");
    assert_eq!(skills[0].manifest.description, "A test skill for manifest parsing");
    assert_eq!(skills[0].manifest.timeout_secs, 60);
    assert!(!skills[0].executable); // No run file
}

#[test]
fn test_skill_invocation_with_valid_args() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let skill_dir = temp_dir.path().join("echo-skill");
    fs::create_dir(&skill_dir).expect("Failed to create skill dir");

    let manifest_content = r#"
name: echo-skill
description: Echo input args
summary: Echo skill
scope: global
args_schema:
  type: object
  properties:
    message:
      type: string
  required:
    - message
timeout_secs: 30
"#;

    fs::write(skill_dir.join("manifest.yml"), manifest_content)
        .expect("Failed to write manifest");

    // Create executable run script
    let run_content = r#"#!/usr/bin/env python3
import sys, json
args = json.loads(sys.stdin.read())
print(json.dumps({"received": args}))
"#;
    fs::write(skill_dir.join("run"), run_content)
        .expect("Failed to write run script");

    // Make executable
    let mut perms = fs::metadata(skill_dir.join("run"))
        .expect("Failed to get metadata")
        .permissions();
    perms.set_mode(perms.mode() | 0o111);
    fs::set_permissions(skill_dir.join("run"), perms)
        .expect("Failed to set permissions");

    let skills = api_skills::discover_skills(temp_dir.path());
    assert_eq!(skills.len(), 1);
    assert!(skills[0].executable);

    // Test valid args
    let args = json!({"message": "hello world"});
    let result = api_skills::execute_skill(
        &skills[0],
        &args,
    );

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.exit_code, Some(0));
    assert!(!response.timed_out);
    assert!(response.stdout.contains("hello world"));
}

#[test]
fn test_skill_invocation_rejects_missing_required_arg() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let skill_dir = temp_dir.path().join("validate-skill");
    fs::create_dir(&skill_dir).expect("Failed to create skill dir");

    let manifest_content = r#"
name: validate-skill
description: Validation test skill
summary: Test validation
scope: global
args_schema:
  type: object
  properties:
    url:
      type: string
    count:
      type: number
  required:
    - url
timeout_secs: 30
"#;

    fs::write(skill_dir.join("manifest.yml"), manifest_content)
        .expect("Failed to write manifest");

    let skills = api_skills::discover_skills(temp_dir.path());
    assert_eq!(skills.len(), 1);

    // Test missing required 'url' argument
    let args = json!({"count": 42}); // Missing 'url'
    let result = api_skills::validate_args_against_schema(
        &args,
        &skills[0].manifest.args_schema,
    );

    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(!errors.is_empty());
    // Error should mention the missing required property
    let error_msg = errors.iter().map(|e| e.message.as_str()).collect::<Vec<_>>().join(" ");
    assert!(error_msg.contains("url") || error_msg.contains("required"));
}

#[test]
fn test_skill_invocation_rejects_wrong_type() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let skill_dir = temp_dir.path().join("typecheck-skill");
    fs::create_dir(&skill_dir).expect("Failed to create skill dir");

    let manifest_content = r#"
name: typecheck-skill
description: Type checking test skill
summary: Test type validation
scope: global
args_schema:
  type: object
  properties:
    count:
      type: number
      minimum: 1
    url:
      type: string
      format: uri
  required:
    - count
timeout_secs: 30
"#;

    fs::write(skill_dir.join("manifest.yml"), manifest_content)
        .expect("Failed to write manifest");

    let skills = api_skills::discover_skills(temp_dir.path());
    assert_eq!(skills.len(), 1);

    // Test wrong type for 'count' (string instead of number)
    let args = json!({"count": "not a number"});
    let result = api_skills::validate_args_against_schema(
        &args,
        &skills[0].manifest.args_schema,
    );

    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(!errors.is_empty());
    // Error should mention the type mismatch
    let error_msg = errors[0].message.to_lowercase();
    assert!(
        error_msg.contains("type") || error_msg.contains("number") || error_msg.contains("integer")
    );
}

#[test]
fn test_skill_invocation_rejects_invalid_uri_format() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let skill_dir = temp_dir.path().join("uri-skill");
    fs::create_dir(&skill_dir).expect("Failed to create skill dir");

    let manifest_content = r#"
name: uri-skill
description: URI validation test skill
summary: Test URI format validation
scope: global
args_schema:
  type: object
  properties:
    url:
      type: string
      format: uri
  required:
    - url
timeout_secs: 30
"#;

    fs::write(skill_dir.join("manifest.yml"), manifest_content)
        .expect("Failed to write manifest");

    let skills = api_skills::discover_skills(temp_dir.path());
    assert_eq!(skills.len(), 1);

    // Test invalid URI format
    let args = json!({"url": "not-a-valid-uri"});
    let result = api_skills::validate_args_against_schema(
        &args,
        &skills[0].manifest.args_schema,
    );

    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(!errors.is_empty());
}

#[test]
fn test_skill_invocation_rejects_out_of_range() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let skill_dir = temp_dir.path().join("range-skill");
    fs::create_dir(&skill_dir).expect("Failed to create skill dir");

    let manifest_content = r#"
name: range-skill
description: Range validation test skill
summary: Test range validation
scope: global
args_schema:
  type: object
  properties:
    count:
      type: number
      minimum: 1
      maximum: 100
  required:
    - count
timeout_secs: 30
"#;

    fs::write(skill_dir.join("manifest.yml"), manifest_content)
        .expect("Failed to write manifest");

    let skills = api_skills::discover_skills(temp_dir.path());
    assert_eq!(skills.len(), 1);

    // Test value below minimum
    let args = json!({"count": 0});
    let result = api_skills::validate_args_against_schema(
        &args,
        &skills[0].manifest.args_schema,
    );
    assert!(result.is_err());

    // Test value above maximum
    let args = json!({"count": 101});
    let result = api_skills::validate_args_against_schema(
        &args,
        &skills[0].manifest.args_schema,
    );
    assert!(result.is_err());
}

#[test]
fn test_skill_to_mcp_tool_conversion() {
    let skills = vec![
        api_skills::SkillEntry {
            name: "fetch".to_string(),
            path: PathBuf::from("/skills/fetch"),
            run_path: PathBuf::from("/skills/fetch/run"),
            manifest: api_skills::SkillManifest {
                name: "fetch".to_string(),
                description: "Fetch a URL".to_string(),
                summary: "URL fetcher".to_string(),
                scope: api_skills::SkillScope::Global,
                projects: Vec::new(),
                pattern: None,
                args_schema: json!({
                    "type": "object",
                    "properties": {
                        "url": {"type": "string"}
                    },
                    "required": ["url"]
                }),
                timeout_secs: 60,
            },
            executable: true,
        },
        api_skills::SkillEntry {
            name: "incomplete".to_string(),
            path: PathBuf::from("/skills/incomplete"),
            run_path: PathBuf::from("/skills/incomplete/run"),
            manifest: api_skills::SkillManifest {
                name: "incomplete".to_string(),
                description: "Incomplete skill".to_string(),
                summary: "Summary".to_string(),
                scope: api_skills::SkillScope::Global,
                projects: Vec::new(),
                pattern: None,
                args_schema: json!({}),
                timeout_secs: 300,
            },
            executable: false,
        },
    ];

    let tools = api_skills::skills_to_mcp_tools(&skills);
    assert_eq!(tools.len(), 1); // Only executable skills
    assert_eq!(tools[0]["name"], "skill_fetch");
    assert_eq!(tools[0]["description"], "Fetch a URL");
    assert_eq!(tools[0]["inputSchema"]["type"], "object");
    assert_eq!(tools[0]["inputSchema"]["required"], json!(["url"]));
}

#[test]
fn test_skill_scope_project() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let skill_dir = temp_dir.path().join("project-skill");
    fs::create_dir(&skill_dir).expect("Failed to create skill dir");

    let manifest_content = r#"
name: project-skill
description: Project-scoped skill
summary: Test project scope
scope: project
projects:
  - project-a
  - project-b
args_schema:
  type: object
timeout_secs: 30
"#;

    fs::write(skill_dir.join("manifest.yml"), manifest_content)
        .expect("Failed to write manifest");

    let skills = api_skills::discover_skills(temp_dir.path());
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].manifest.scope, api_skills::SkillScope::Project);
    assert_eq!(skills[0].manifest.projects, vec!["project-a", "project-b"]);
}

#[test]
fn test_skill_scope_pattern() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let skill_dir = temp_dir.path().join("pattern-skill");
    fs::create_dir(&skill_dir).expect("Failed to create skill dir");

    let manifest_content = r#"
name: pattern-skill
description: Pattern-scoped skill
summary: Test pattern scope
scope: pattern
pattern: "fix-*"
args_schema:
  type: object
timeout_secs: 30
"#;

    fs::write(skill_dir.join("manifest.yml"), manifest_content)
        .expect("Failed to write manifest");

    let skills = api_skills::discover_skills(temp_dir.path());
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].manifest.scope, api_skills::SkillScope::Pattern);
    assert_eq!(skills[0].manifest.pattern, Some("fix-*".to_string()));
}

#[test]
fn test_skill_manifest_name_mismatch() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let skill_dir = temp_dir.path().join("actual-name");
    fs::create_dir(&skill_dir).expect("Failed to create skill dir");

    // Manifest has different name
    let manifest_content = r#"
name: different-name
description: Test
summary: Test
scope: global
args_schema:
  type: object
"#;

    fs::write(skill_dir.join("manifest.yml"), manifest_content)
        .expect("Failed to write manifest");

    let skills = api_skills::discover_skills(temp_dir.path());
    // Skill should be ignored due to name mismatch
    assert!(skills.is_empty());
}
