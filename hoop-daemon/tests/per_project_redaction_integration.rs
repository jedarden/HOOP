//! Integration tests for §18.5 per-project redaction policy override
//!
//! Acceptance criteria:
//! - Schema updated; hot-reload supported
//! - Customer-data project set to `reject` correctly blocks risky attachments
//! - Test: same attachment, different policy → different outcomes
//!
//! CI command:
//!   cargo test -p hoop-daemon --test per_project_redaction_integration

use hoop_daemon::redaction_policy::{self, RedactionAction, RedactionPolicyState};
use hoop_schema::{
    HoopConfig, HoopConfigRedaction, HoopConfigRedactionAction, HoopConfigRedactionPatternsItem,
    ProjectsRegistry, ProjectsRegistryProjectsItem,
};

// ── Fixtures ──────────────────────────────────────────────────────────────────

/// Create a global config with permissive redaction policy
fn make_global_permissive_config() -> HoopConfig {
    HoopConfig {
        schema_version: Default::default(),
        redaction: Some(HoopConfigRedaction {
            action: HoopConfigRedactionAction::Warn,
            patterns: vec![
                HoopConfigRedactionPatternsItem::AnthropicApiKey,
                HoopConfigRedactionPatternsItem::GenericSkKey,
                HoopConfigRedactionPatternsItem::AwsAccessKey,
                HoopConfigRedactionPatternsItem::GithubToken,
                HoopConfigRedactionPatternsItem::SlackToken,
                HoopConfigRedactionPatternsItem::Jwt,
                HoopConfigRedactionPatternsItem::BearerToken,
                HoopConfigRedactionPatternsItem::EnvVarSecret,
                HoopConfigRedactionPatternsItem::JsonSecretField,
            ],
        }),
        agent: None,
        agent_extensions: None,
        audit: None,
        backup: None,
        metrics: None,
        pricing: None,
        projects_file: None,
        reflection: None,
        server: None,
        ui: None,
        voice: None,
    }
}

/// Create projects registry with two projects:
/// - customer-data: reject policy (blocks risky attachments)
/// - internal-tools: warn policy (allows but logs)
fn make_mixed_policy_projects() -> ProjectsRegistry {
    ProjectsRegistry {
        projects: vec![
            // customer-data: strict reject policy
            ProjectsRegistryProjectsItem::Variant0 {
                name: "customer-data".to_string(),
                path: "/tmp/customer-data".to_string(),
                canonical_path: None,
                label: None,
                color: None,
                redaction: Some(hoop_schema::ProjectsRegistryProjectsItemVariant0Redaction {
                    action: hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionAction::Reject,
                    patterns: vec![
                        hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionPatternsItem::AnthropicApiKey,
                        hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionPatternsItem::GenericSkKey,
                        hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionPatternsItem::AwsAccessKey,
                    ],
                }),
            },
            // internal-tools: permissive warn policy
            ProjectsRegistryProjectsItem::Variant0 {
                name: "internal-tools".to_string(),
                path: "/tmp/internal-tools".to_string(),
                canonical_path: None,
                label: None,
                color: None,
                redaction: Some(hoop_schema::ProjectsRegistryProjectsItemVariant0Redaction {
                    action: hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionAction::Warn,
                    patterns: vec![
                        hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionPatternsItem::AnthropicApiKey,
                        hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionPatternsItem::GithubToken,
                    ],
                }),
            },
            // legacy-project: no override (uses global policy)
            ProjectsRegistryProjectsItem::Variant0 {
                name: "legacy-project".to_string(),
                path: "/tmp/legacy".to_string(),
                canonical_path: None,
                label: None,
                color: None,
                redaction: None,
            },
        ],
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn test_schema_redaction_field_exists() {
    // Verify that project entries support the redaction field
    let projects = make_mixed_policy_projects();

    // Check customer-data has reject policy
    if let ProjectsRegistryProjectsItem::Variant0 { redaction, .. } = &projects.projects[0] {
        assert!(
            redaction.is_some(),
            "customer-data should have redaction policy"
        );
        let policy = redaction.as_ref().unwrap();
        assert!(matches!(
            policy.action,
            hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionAction::Reject
        ));
    } else {
        panic!("Expected Variant0 project");
    }

    // Check internal-tools has warn policy
    if let ProjectsRegistryProjectsItem::Variant0 { redaction, .. } = &projects.projects[1] {
        assert!(
            redaction.is_some(),
            "internal-tools should have redaction policy"
        );
        let policy = redaction.as_ref().unwrap();
        assert!(matches!(
            policy.action,
            hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionAction::Warn
        ));
    } else {
        panic!("Expected Variant0 project");
    }

    // Check legacy-project has no override
    if let ProjectsRegistryProjectsItem::Variant0 { redaction, .. } = &projects.projects[2] {
        assert!(
            redaction.is_none(),
            "legacy-project should not have redaction override"
        );
    } else {
        panic!("Expected Variant0 project");
    }
}

#[test]
fn test_per_project_policy_resolution() {
    let config = make_global_permissive_config();
    let projects = make_mixed_policy_projects();
    let state = RedactionPolicyState::new(&config, projects);

    let rt = tokio::runtime::Runtime::new().unwrap();

    // customer-data: reject action from project override
    let policy = rt.block_on(state.resolve_for_project("customer-data"));
    assert_eq!(policy.action, RedactionAction::Reject);
    assert_eq!(policy.source, "project:customer-data");
    assert_eq!(policy.patterns.len(), 3); // Only the 3 patterns in project override

    // internal-tools: warn action from project override
    let policy = rt.block_on(state.resolve_for_project("internal-tools"));
    assert_eq!(policy.action, RedactionAction::Warn);
    assert_eq!(policy.source, "project:internal-tools");
    assert_eq!(policy.patterns.len(), 2); // Only the 2 patterns in project override

    // legacy-project: falls back to global policy
    let policy = rt.block_on(state.resolve_for_project("legacy-project"));
    assert_eq!(policy.action, RedactionAction::Warn);
    assert_eq!(policy.source, "global");
    assert_eq!(policy.patterns.len(), 9); // All patterns from global config

    // unknown-project: falls back to global policy (since it is configured)
    let policy = rt.block_on(state.resolve_for_project("unknown-project"));
    assert_eq!(policy.action, RedactionAction::Warn);
    assert_eq!(policy.source, "global");
    assert_eq!(policy.patterns.len(), 9); // All patterns from global config
}

#[test]
fn test_same_attachment_different_outcomes() {
    // Acceptance criterion: "Test: same attachment, different policy → different outcomes"
    let config = make_global_permissive_config();
    let projects = make_mixed_policy_projects();
    let state = RedactionPolicyState::new(&config, projects);

    let rt = tokio::runtime::Runtime::new().unwrap();

    // Same content with Anthropic API key
    let content_with_secret =
        "ANTHROPIC_API_KEY=sk-ant-api03-AAAA1111BBBB2222CCCC3333DDDD4444EEEE5555FFFF6666";

    // customer-data (reject): should fail
    let result = rt.block_on(redaction_policy::check_reject_policy(
        &state,
        "customer-data",
        content_with_secret,
    ));
    assert!(
        result.is_err(),
        "customer-data with reject policy should block attachment with secret"
    );
    let err = result.unwrap_err();
    assert_eq!(err.project, "customer-data");
    assert!(err.pattern.contains("anthropic"));

    // internal-tools (warn): should pass
    let result = rt.block_on(redaction_policy::check_reject_policy(
        &state,
        "internal-tools",
        content_with_secret,
    ));
    assert!(
        result.is_ok(),
        "internal-tools with warn policy should allow attachment with secret"
    );

    // legacy-project (global warn): should pass
    let result = rt.block_on(redaction_policy::check_reject_policy(
        &state,
        "legacy-project",
        content_with_secret,
    ));
    assert!(
        result.is_ok(),
        "legacy-project with global warn policy should allow attachment with secret"
    );
}

#[test]
fn test_customer_data_reject_blocks_risky_attachments() {
    // Acceptance criterion: "Customer-data project set to `reject` correctly blocks risky attachments"
    let config = make_global_permissive_config();
    let mut projects = make_mixed_policy_projects();

    // Ensure customer-data has reject policy with all secret patterns
    if let ProjectsRegistryProjectsItem::Variant0 { redaction, .. } = &mut projects.projects[0] {
        *redaction = Some(hoop_schema::ProjectsRegistryProjectsItemVariant0Redaction {
            action: hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionAction::Reject,
            patterns: vec![
                hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionPatternsItem::AnthropicApiKey,
                hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionPatternsItem::GenericSkKey,
                hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionPatternsItem::AwsAccessKey,
                hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionPatternsItem::GithubToken,
                hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionPatternsItem::SlackToken,
                hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionPatternsItem::Jwt,
                hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionPatternsItem::BearerToken,
                hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionPatternsItem::EnvVarSecret,
                hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionPatternsItem::JsonSecretField,
            ],
        });
    }

    let state = RedactionPolicyState::new(&config, projects);
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Test all supported secret patterns are blocked
    let test_cases = vec![
        ("ANTHROPIC_API_KEY=sk-ant-api03-AAAA1111BBBB2222CCCC3333DDDD4444EEEE5555FFFF6666", "anthropic_api_key"),
        ("API_KEY=sk-1234567890abcdef", "generic_sk_key"),
        ("AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE", "aws_access_key"),
        ("GITHUB_TOKEN=ghp_16C7e42F292c6912E7710c838347Ae178B4a", "github_token"),
        ("SLACK_TOKEN=xoxb-1234567890-1234567890123-AbCdEfGhIjKlMnOpQrStUvWx", "slack_token"),
        ("Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U", "jwt"),
        ("Authorization: Bearer tok_1234567890abcdef", "bearer_token"),
        ("API_KEY=supersecretvalue123456", "env_var_secret"),
        (r#"{"password": "secret123"}"#, "json_secret_field"),
    ];

    for (content, expected_pattern) in test_cases {
        let result = rt.block_on(redaction_policy::check_reject_policy(
            &state,
            "customer-data",
            content,
        ));
        assert!(
            result.is_err(),
            "customer-data should block content with {}",
            expected_pattern
        );
        let err = result.unwrap_err();
        assert_eq!(err.project, "customer-data");
    }

    // Clean content should still be allowed
    let clean_content = "This is a clean document with no secrets. Safe to upload.";
    let result = rt.block_on(redaction_policy::check_reject_policy(
        &state,
        "customer-data",
        clean_content,
    ));
    assert!(result.is_ok(), "customer-data should allow clean content");
}

#[test]
fn test_pattern_filtering_in_project_override() {
    // Test that project override can limit which patterns are checked
    let config = make_global_permissive_config();
    let mut projects = make_mixed_policy_projects();

    // Set customer-data to only check for Anthropic keys (not GitHub tokens)
    if let ProjectsRegistryProjectsItem::Variant0 { redaction, .. } = &mut projects.projects[0] {
        *redaction = Some(hoop_schema::ProjectsRegistryProjectsItemVariant0Redaction {
            action: hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionAction::Reject,
            patterns: vec![
                hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionPatternsItem::AnthropicApiKey,
            ],
        });
    }

    let state = RedactionPolicyState::new(&config, projects);
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Anthropic key should be blocked
    let anthropic_content =
        "ANTHROPIC_API_KEY=sk-ant-api03-AAAA1111BBBB2222CCCC3333DDDD4444EEEE5555FFFF6666";
    let result = rt.block_on(redaction_policy::check_reject_policy(
        &state,
        "customer-data",
        anthropic_content,
    ));
    assert!(result.is_err(), "customer-data should block Anthropic keys");

    // GitHub token should NOT be blocked (not in patterns list)
    let github_content = "GITHUB_TOKEN=ghp_16C7e42F292c6912E7710c838347Ae178B4a";
    let result = rt.block_on(redaction_policy::check_reject_policy(
        &state,
        "customer-data",
        github_content,
    ));
    assert!(
        result.is_ok(),
        "customer-data should allow GitHub tokens when not in pattern list"
    );
}

#[test]
fn test_multi_workspace_project_redaction_override() {
    // Test that multi-workspace projects also support redaction overrides
    let config = make_global_permissive_config();

    let multi_workspace_project = ProjectsRegistryProjectsItem::Variant1 {
        name: "multi-workspace-project".to_string(),
        label: None,
        color: None,
        redaction: Some(hoop_schema::ProjectsRegistryProjectsItemVariant1Redaction {
            action: hoop_schema::ProjectsRegistryProjectsItemVariant1RedactionAction::Redact,
            patterns: vec![
                hoop_schema::ProjectsRegistryProjectsItemVariant1RedactionPatternsItem::AwsAccessKey,
                hoop_schema::ProjectsRegistryProjectsItemVariant1RedactionPatternsItem::GithubToken,
            ],
        }),
        workspaces: vec![
            hoop_schema::ProjectsRegistryProjectsItemVariant1WorkspacesItem {
                path: "/tmp/workspace1".to_string(),
                canonical_path: None,
                role: hoop_schema::ProjectsRegistryProjectsItemVariant1WorkspacesItemRole::Primary,
            },
            hoop_schema::ProjectsRegistryProjectsItemVariant1WorkspacesItem {
                path: "/tmp/workspace2".to_string(),
                canonical_path: None,
                role: hoop_schema::ProjectsRegistryProjectsItemVariant1WorkspacesItemRole::Source,
            },
        ],
    };

    let projects = ProjectsRegistry {
        projects: vec![multi_workspace_project],
    };

    let state = RedactionPolicyState::new(&config, projects);
    let rt = tokio::runtime::Runtime::new().unwrap();

    let policy = rt.block_on(state.resolve_for_project("multi-workspace-project"));
    assert_eq!(policy.action, RedactionAction::Redact);
    assert_eq!(policy.source, "project:multi-workspace-project");
    assert_eq!(policy.patterns.len(), 2);
    assert!(policy.patterns.contains("aws_access_key"));
    assert!(policy.patterns.contains("github_token"));
}

#[test]
fn test_hot_reload_policy_changes() {
    // Test that policy changes take effect without restart
    let config = make_global_permissive_config();
    let mut projects = make_mixed_policy_projects();

    let state = RedactionPolicyState::new(&config, projects.clone());
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Initial state: customer-data has reject policy
    let content_with_secret =
        "ANTHROPIC_API_KEY=sk-ant-api03-AAAA1111BBBB2222CCCC3333DDDD4444EEEE5555FFFF6666";
    let result = rt.block_on(redaction_policy::check_reject_policy(
        &state,
        "customer-data",
        content_with_secret,
    ));
    assert!(
        result.is_err(),
        "Initial: customer-data should block secrets"
    );

    // Simulate hot-reload: change customer-data to warn policy
    if let ProjectsRegistryProjectsItem::Variant0 { redaction, .. } = &mut projects.projects[0] {
        *redaction = Some(hoop_schema::ProjectsRegistryProjectsItemVariant0Redaction {
            action: hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionAction::Warn,
            patterns: vec![
                hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionPatternsItem::AnthropicApiKey,
            ],
        });
    }

    rt.block_on(state.update_projects(projects));

    // After hot-reload: customer-data now has warn policy
    let result = rt.block_on(redaction_policy::check_reject_policy(
        &state,
        "customer-data",
        content_with_secret,
    ));
    assert!(
        result.is_ok(),
        "After hot-reload: customer-data should allow secrets with warn policy"
    );
}
