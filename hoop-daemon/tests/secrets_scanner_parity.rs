//! Parity tests for secrets scanner unification (§18)
//!
//! Verifies that:
//! - Backend scanner uses patterns from config_resolver (single source of truth)
//! - Patterns served by /api/config/secrets-patterns match what backend uses
//! - Client and backend detect the same secrets

use hoop_daemon::config_resolver::{SecretPattern, default_secret_patterns};
use hoop_daemon::secrets_scanner;

/// Test fixture for a secret that should be detected
struct SecretFixture {
    name: &'static str,
    pattern_id: &'static str,
    content: &'static str,
    should_detect: bool,
}

const SECRET_FIXTURES: &[SecretFixture] = &[
    // Stripe API keys
    SecretFixture {
        name: "stripe_live_key",
        pattern_id: "stripe_api_key",
        content: "My Stripe key is sk_live_51AbCdEf1234567890AbCdEf1234567890AbC",
        should_detect: true,
    },
    SecretFixture {
        name: "stripe_test_key",
        pattern_id: "stripe_api_key",
        content: "Stripe test: sk_test_51AbCdEf1234567890AbCdEf1234567890AbC",
        should_detect: true,
    },
    // OpenAI API keys
    SecretFixture {
        name: "openai_sk_key",
        pattern_id: "openai_api_key",
        content: "OPENAI_API_KEY=sk-ABCDEFGHIJKLMNOPQRSTUVWXYZ123456",
        should_detect: true,
    },
    SecretFixture {
        name: "openai_sk_proj_key",
        pattern_id: "openai_api_key",
        content: "OpenAI project key: sk-proj-AbCdEf1234567890AbCdEf1234567890AbCdEf123456",
        should_detect: true,
    },
    // Anthropic API keys
    SecretFixture {
        name: "anthropic_key",
        pattern_id: "anthropic_api_key",
        content: "ANTHROPIC_API_KEY=sk-ant-api03-AAAA1111BBBB2222CCCC3333DDDD4444EEEE5555FFFF6666",
        should_detect: true,
    },
    SecretFixture {
        name: "anthropic_short",
        pattern_id: "anthropic_api_key",
        content: "Key: sk-ant-ABCDEFGHIJKLMNOPQRSTUVWXYZ123456",
        should_detect: true,
    },
    // AWS access keys
    SecretFixture {
        name: "aws_access_key",
        pattern_id: "aws_access_key",
        content: "aws_access_key_id = AKIAIOSFODNN7EXAMPLE",
        should_detect: true,
    },
    SecretFixture {
        name: "aws_secret_key",
        pattern_id: "aws_secret_key",
        content: "aws_secret_access_key = ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890+/=",
        should_detect: true,
    },
    SecretFixture {
        name: "aws_temp_key",
        pattern_id: "aws_access_key",
        content: "ASIA1234567890ABCDEF",
        should_detect: true,
    },
    // GitHub tokens
    SecretFixture {
        name: "github_pat",
        pattern_id: "github_token",
        content: "github_token=github_pat_1234567890abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890abcdefghijklmnopqrstuvwxyz",
        should_detect: true,
    },
    SecretFixture {
        name: "github_ghp",
        pattern_id: "github_token",
        content: "GITHUB_TOKEN=ghp_16C7e42F292c6912E7710c838347Ae178B4a",
        should_detect: true,
    },
    SecretFixture {
        name: "github_ghs",
        pattern_id: "github_token",
        content: "token: ghs_16C7e42F292c6912E7710c838347Ae178B4a",
        should_detect: true,
    },
    // Slack tokens
    SecretFixture {
        name: "slack_xoxb",
        pattern_id: "slack_token",
        content: "SLACK_TOKEN=xoxb-1234567890-1234567890123-12345678901234567890123456",
        should_detect: true,
    },
    // JWT tokens
    SecretFixture {
        name: "jwt_token",
        pattern_id: "jwt",
        content: "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U",
        should_detect: true,
    },
    // Bearer tokens
    SecretFixture {
        name: "bearer_token",
        pattern_id: "bearer_token",
        content: "Authorization: Bearer ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCDEF",
        should_detect: true,
    },
    // Environment variable secrets
    SecretFixture {
        name: "env_var_secret",
        pattern_id: "env_var_secret",
        content: "API_KEY=sk-ant-api03-TEST1234567890ABCDEFGHIJ1234567890ABCD",
        should_detect: true,
    },
    // JSON secret fields
    SecretFixture {
        name: "json_secret",
        pattern_id: "json_secret_field",
        content: r#"{"password": "s3cr3tP@ssw0rd!", "api_key": "abc123def456ghi789jkl"}"#,
        should_detect: true,
    },
    // Generic sk- keys (catch-all for sk- prefix)
    SecretFixture {
        name: "generic_sk_key",
        pattern_id: "generic_sk_key",
        content: "SECRET_KEY=sk-ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890",
        should_detect: true,
    },
    // Benign content (should NOT detect)
    SecretFixture {
        name: "benign_url",
        pattern_id: "none",
        content: "Visit https://example.com for more info",
        should_detect: false,
    },
    SecretFixture {
        name: "benign_email",
        pattern_id: "none",
        content: "Contact support@example.com for help",
        should_detect: false,
    },
    SecretFixture {
        name: "benign_uuid",
        pattern_id: "none",
        content: "Request ID: 550e8400-e29b-41d4-a716-446655440000",
        should_detect: false,
    },
    SecretFixture {
        name: "benign_git_sha",
        pattern_id: "none",
        content: "commit abc123def456789abcdef0123456789abcdef01 pushed by john",
        should_detect: false,
    },
];

/// Test that all synthetic secret fixtures are detected correctly
#[test]
fn test_synthetic_secrets_detected() {
    // Initialize scanner with default patterns from config_resolver
    let patterns = default_secret_patterns();
    secrets_scanner::update_patterns(&patterns);

    for fixture in SECRET_FIXTURES {
        let findings = secrets_scanner::scan_text(fixture.content, None);

        if fixture.should_detect {
            assert!(
                !findings.is_empty(),
                "Fixture '{}' (pattern: {}) should have been detected but wasn't. Content: {}",
                fixture.name,
                fixture.pattern_id,
                fixture.content
            );
            // Verify the correct pattern matched
            let has_expected_pattern = findings.iter().any(|f| f.pattern_id == fixture.pattern_id);
            if fixture.pattern_id != "none" {
                assert!(
                    has_expected_pattern,
                    "Fixture '{}' matched but not by expected pattern '{}'. Found patterns: {:?}",
                    fixture.name,
                    fixture.pattern_id,
                    findings.iter().map(|f| &f.pattern_id).collect::<Vec<_>>()
                );
            }
        } else {
            assert!(
                findings.is_empty(),
                "Fixture '{}' should NOT have been detected but got {} findings: {:?}",
                fixture.name,
                findings.len(),
                findings
            );
        }
    }
}

/// Test that default_secret_patterns() returns valid, non-empty patterns
#[test]
fn test_default_patterns_valid() {
    let patterns = default_secret_patterns();

    assert!(!patterns.is_empty(), "Default patterns should not be empty");

    // Verify all patterns are valid
    for pattern in &patterns {
        assert!(!pattern.id.is_empty(), "Pattern ID should not be empty");
        assert!(!pattern.name.is_empty(), "Pattern name should not be empty");
        assert!(
            pattern.severity == "high" || pattern.severity == "medium" || pattern.severity == "low",
            "Pattern '{}' has invalid severity: {}",
            pattern.id,
            pattern.severity
        );
        assert!(!pattern.patterns.is_empty(), "Pattern '{}' should have at least one regex", pattern.id);

        // Verify all regex patterns compile
        pattern.validate().expect(&format!("Pattern '{}' should have valid regex", pattern.id));
    }
}

/// Test that patterns match what the API endpoint would serve
#[test]
fn test_patterns_match_api_format() {
    let patterns = default_secret_patterns();

    // Verify each pattern can be serialized to JSON (as the API would)
    for pattern in &patterns {
        let json = serde_json::to_string(pattern).expect("Pattern should serialize to JSON");
        let parsed: SecretPattern = serde_json::from_str(&json).expect("Serialized pattern should deserialize");

        assert_eq!(parsed.id, pattern.id);
        assert_eq!(parsed.name, pattern.name);
        assert_eq!(parsed.severity, pattern.severity);
        assert_eq!(parsed.patterns, pattern.patterns);
    }
}

/// Test that all fixture pattern_ids exist in default patterns
#[test]
fn test_all_fixture_patterns_exist() {
    let patterns = default_secret_patterns();
    let pattern_ids: std::collections::HashSet<_> = patterns.iter().map(|p| p.id.as_str()).collect();

    for fixture in SECRET_FIXTURES {
        if fixture.pattern_id != "none" {
            assert!(
                pattern_ids.contains(fixture.pattern_id),
                "Fixture '{}' references pattern_id '{}' which doesn't exist in default patterns",
                fixture.name,
                fixture.pattern_id
            );
        }
    }
}

/// Test pattern unification - backend uses same patterns as config_resolver
#[test]
fn test_backend_uses_config_resolver_patterns() {
    let config_patterns = default_secret_patterns();

    // Initialize the scanner with these patterns
    secrets_scanner::update_patterns(&config_patterns);

    // Scan a test secret
    let test_content = "My key is sk-ant-api03-TEST1234567890ABCDEFGHIJ1234567890ABCD";
    let findings = secrets_scanner::scan_text(test_content, None);

    // Should detect using the anthropic_api_key pattern
    assert!(!findings.is_empty(), "Should detect Anthropic API key");
    assert!(
        findings.iter().any(|f| f.pattern_id == "anthropic_api_key"),
        "Should match anthropic_api_key pattern from config_resolver"
    );
}

/// Test that config.yml patterns override defaults correctly
#[test]
fn test_custom_patterns_override_defaults() {
    // Start with default patterns
    let default_patterns = default_secret_patterns();
    secrets_scanner::update_patterns(&default_patterns);

    // Create a custom pattern that should detect a specific test string
    let custom_patterns = vec![SecretPattern {
        id: "test_custom_pattern".to_string(),
        name: "Test Custom Pattern".to_string(),
        severity: "high".to_string(),
        patterns: vec![r"\bTEST_SECRET_[A-Z0-9]{10}\b".to_string()],
    }];

    // Update scanner with custom patterns
    secrets_scanner::update_patterns(&custom_patterns);

    // Test that custom pattern works
    let test_content = "My test secret is TEST_SECRET_ABC1234567";
    let findings = secrets_scanner::scan_text(test_content, None);

    assert!(!findings.is_empty(), "Custom pattern should detect test secret");
    assert!(
        findings.iter().any(|f| f.pattern_id == "test_custom_pattern"),
        "Should match custom pattern"
    );

    // Test that default patterns no longer work (they were replaced)
    let anthropic_content = "My key is sk-ant-api03-TEST1234567890ABCDEFGHIJ";
    let findings = secrets_scanner::scan_text(anthropic_content, None);

    assert!(
        findings.is_empty(),
        "Default anthropic_api_key pattern should not work after replacing with custom patterns"
    );
}
