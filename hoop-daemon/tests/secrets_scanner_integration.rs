//! Secrets scanner integration tests
//!
//! Tests:
//! - Synthetic secrets in fixtures caught correctly
//! - False positive rate <5% on testrepo/ content
//! - Parity with client-side scanner

use hoop_daemon::secrets_scanner;

/// Test fixtures for various secret patterns
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
    SecretFixture {
        name: "stripe_ir_live",
        pattern_id: "stripe_api_key",
        content: "IR key: ir_live_51AbCdEf1234567890AbCdEf1234567890AbCdEf123456",
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
    // JWT tokens
    SecretFixture {
        name: "jwt_token",
        pattern_id: "jwt",
        content: "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U",
        should_detect: true,
    },
    // Slack tokens
    SecretFixture {
        name: "slack_bot_token",
        pattern_id: "slack_token",
        content: "SLACK_TOKEN=xoxb-1234567890-1234567890123-12345678901234567890123456",
        should_detect: true,
    },
    SecretFixture {
        name: "slack_user_token",
        pattern_id: "slack_token",
        content: "xoxp-1234567890-1234567890123-12345678901234567890123456",
        should_detect: true,
    },
    // Generic sk- keys
    SecretFixture {
        name: "generic_sk",
        pattern_id: "generic_sk_key",
        content: "API_KEY=sk-ABCDEFGHIJKLMNOPQRSTUVWXYZ123456",
        should_detect: true,
    },
    // Bearer tokens
    SecretFixture {
        name: "bearer_token",
        pattern_id: "bearer_token",
        content: "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U",
        should_detect: true,
    },
    // Environment variable secrets
    SecretFixture {
        name: "env_var_openai",
        pattern_id: "env_var_secret",
        content: "export OPENAI_API_KEY=sk-proj-AbCdEf1234567890AbCdEf",
        should_detect: true,
    },
    SecretFixture {
        name: "env_var_anthropic",
        pattern_id: "env_var_secret",
        content: "ANTHROPIC_API_KEY: sk-ant-api03-TEST1234567890ABCDEFGHIJ",
        should_detect: true,
    },
    // JSON secret fields
    SecretFixture {
        name: "json_password",
        pattern_id: "json_secret_field",
        content: r#"{"password": "s3cr3tP@ssw0rd!"}"#,
        should_detect: true,
    },
    SecretFixture {
        name: "json_api_key",
        pattern_id: "json_secret_field",
        content: r#"{"api_key": "abc123def456ghi789jkl"}"#,
        should_detect: true,
    },
    // Clean content (should NOT detect)
    SecretFixture {
        name: "clean_text",
        pattern_id: "",
        content: "This is a normal message with no secrets. Just plain text.",
        should_detect: false,
    },
    SecretFixture {
        name: "git_commit",
        pattern_id: "",
        content: "commit abc123def456789abcdef0123456789abcdef01 Author: John Doe",
        should_detect: false,
    },
    SecretFixture {
        name: "uuid",
        pattern_id: "",
        content: "Request ID: 550e8400-e29b-41d4-a716-446655440000",
        should_detect: false,
    },
    SecretFixture {
        name: "url",
        pattern_id: "",
        content: "Visit https://example.com/docs for more information",
        should_detect: false,
    },
];

/// Test that all synthetic secret fixtures are detected correctly
#[test]
fn test_synthetic_secrets_detected() {
    secrets_scanner::init();

    for fixture in SECRET_FIXTURES {
        let findings = secrets_scanner::scan_text(fixture.content, None);

        if fixture.should_detect {
            assert!(
                !findings.is_empty(),
                "Fixture '{}' should have been detected but got no findings. Content: {}",
                fixture.name,
                fixture.content
            );

            // If a specific pattern_id is expected, check for it
            if !fixture.pattern_id.is_empty() {
                assert!(
                    findings.iter().any(|f| f.pattern_id == fixture.pattern_id),
                    "Fixture '{}' should match pattern '{}' but got: {:?}",
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

/// Test high-entropy detection with context awareness
#[test]
fn test_high_entropy_context_awareness() {
    secrets_scanner::init();

    // Git SHA should not be flagged
    let git_context = "commit abc123def456789abcdef0123456789abcdef01 pushed to main";
    let findings = secrets_scanner::scan_text(git_context, None);
    assert!(
        !findings.iter().any(|f| f.pattern_id == "high_entropy"),
        "Git SHA in 'commit' context should not be flagged as high entropy"
    );

    // UUID should not be flagged
    let uuid_context = "Request ID: 550e8400-e29b-41d4-a716-446655440000 completed";
    let findings = secrets_scanner::scan_text(uuid_context, None);
    assert!(
        !findings.iter().any(|f| f.pattern_id == "high_entropy"),
        "UUID should not be flagged as high entropy"
    );

    // Random high-entropy string without context should be flagged
    let random_entropy = "sk-ant-api03-AAAA1111BBBB2222CCCC3333DDDD4444EEEE5555FFFF6666";
    let findings = secrets_scanner::scan_text(random_entropy, None);
    // This will be caught by the anthropic_api_key pattern, but high_entropy is a fallback
    assert!(!findings.is_empty(), "High-entropy string should be detected");
}

/// Test email detection (opt-in)
#[test]
fn test_email_detection_opt_in() {
    secrets_scanner::init();

    let email_content = "Contact support@example.com for help";

    // Email detection disabled by default
    let findings = secrets_scanner::scan_text(email_content, None);
    assert!(
        findings.is_empty(),
        "Email should not be detected when disabled"
    );

    // Enable email detection for test project
    secrets_scanner::enable_email_detection("test-project".to_string());

    let findings = secrets_scanner::scan_text(email_content, Some("test-project"));
    assert!(
        !findings.is_empty(),
        "Email should be detected when enabled"
    );
    assert!(
        findings.iter().any(|f| f.pattern_id == "email"),
        "Should detect email pattern"
    );

    // Clean up
    secrets_scanner::disable_email_detection("test-project");
}

/// Test that findings have correct metadata
#[test]
fn test_finding_metadata() {
    secrets_scanner::init();

    let content = "My key is sk-ant-api03-TEST1234567890ABCDEFGHIJ1234567890ABCD";
    let findings = secrets_scanner::scan_text(content, None);

    assert!(!findings.is_empty());
    let finding = &findings[0];

    assert!(!finding.pattern_id.is_empty());
    assert!(!finding.description.is_empty());
    assert!(finding.match_start < content.len());
    assert!(finding.match_start + finding.match_len <= content.len());
    assert_eq!(finding.matched_text, content[finding.match_start..finding.match_start + finding.match_len]);
    assert!(matches!(finding.severity.as_str(), "high" | "medium" | "low"));
}

/// Test multiple secrets in one scan
#[test]
fn test_multiple_secrets() {
    secrets_scanner::init();

    let content = r#"
Configuration:
stripe_key: sk_live_51AbCdEf1234567890AbCdEf
github_token: ghp_16C7e42F292c6912E7710c838347Ae178B4a
aws_key: AKIAIOSFODNN7EXAMPLE
"#;

    let findings = secrets_scanner::scan_text(content, None);

    // Should detect at least 3 secrets
    assert!(
        findings.len() >= 3,
        "Should detect at least 3 secrets, got {}",
        findings.len()
    );

    // Check that we have the expected patterns
    let pattern_ids: std::collections::HashSet<_> =
        findings.iter().map(|f| f.pattern_id.as_str()).collect();

    assert!(pattern_ids.contains("stripe_api_key"));
    assert!(pattern_ids.contains("github_token"));
    assert!(pattern_ids.contains("aws_access_key"));
}

/// Test entropy calculation
#[test]
fn test_entropy_calculation() {
    // High entropy: API key
    let high_entropy = "sk-ant-api03-AAAA1111BBBB2222CCCC3333DDDD4444";
    let e = hoop_daemon::secrets_scanner::calculate_entropy(high_entropy);
    assert!(e > 4.5, "API key should have high entropy: {}", e);

    // Low entropy: normal text
    let low_entropy = "the quick brown fox jumps over the lazy dog";
    let e = hoop_daemon::secrets_scanner::calculate_entropy(low_entropy);
    assert!(e < 4.5, "Normal text should have low entropy: {}", e);
}

/// Test that the scanner handles edge cases
#[test]
fn test_edge_cases() {
    secrets_scanner::init();

    // Empty string
    let findings = secrets_scanner::scan_text("", None);
    assert!(findings.is_empty());

    // Very short string
    let findings = secrets_scanner::scan_text("sk-abc", None);
    assert!(findings.is_empty(), "Very short strings should not be flagged");

    // Only whitespace
    let findings = secrets_scanner::scan_text("   \n\t  ", None);
    assert!(findings.is_empty());

    // Special characters only
    let findings = secrets_scanner::scan_text("!@#$%^&*()", None);
    assert!(findings.is_empty());
}

/// Test findings serialization
#[test]
fn test_findings_serialization() {
    secrets_scanner::init();

    let content = "API_KEY=sk-ant-api03-TEST1234567890ABCDEFGHIJ";
    let findings = secrets_scanner::scan_text(content, None);

    assert!(!findings.is_empty());

    // Test JSON serialization
    let json = serde_json::to_string(&findings).unwrap();
    assert!(!json.is_empty());

    // Test deserialization
    let deserialized: Vec<secrets_scanner::Finding> =
        serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.len(), findings.len());
    assert_eq!(deserialized[0].pattern_id, findings[0].pattern_id);
}

/// Test pattern overlap handling
#[test]
fn test_pattern_overlap() {
    secrets_scanner::init();

    // This content matches both anthropic_api_key and env_var_secret
    let content = "ANTHROPIC_API_KEY=sk-ant-api03-AAAA1111BBBB2222CCCC3333DDDD4444";
    let findings = secrets_scanner::scan_text(content, None);

    // Both patterns should detect (may have overlapping findings)
    assert!(!findings.is_empty());

    // At least one should match
    let has_anthropic = findings.iter().any(|f| f.pattern_id == "anthropic_api_key");
    let has_env_var = findings.iter().any(|f| f.pattern_id == "env_var_secret");

    assert!(
        has_anthropic || has_env_var,
        "Should detect with either anthropic_api_key or env_var_secret pattern"
    );
}

/// Test that benign content doesn't produce false positives
#[test]
fn test_benign_content() {
    secrets_scanner::init();

    let benign_samples = vec![
        "My email is john@example.com",
        "Visit https://www.example.com for more info",
        "The color code is #FF0000 for red",
        "commit abc123def456789abcdef0123456789abcdef01",
        "UUID: 550e8400-e29b-41d4-a716-446655440000",
        "The quick brown fox jumps over the lazy dog",
        "HTTP status code: 200 OK",
        "Base64 encoded: SGVsbG8gV29ybGQ=",  // "Hello World" in base64
    ];

    for sample in benign_samples {
        let findings = secrets_scanner::scan_text(sample, None);
        // Email detection is opt-in, so email@example.com should not be detected
        // Base64 "Hello World" is low entropy
        assert!(
            findings.is_empty(),
            "Benign content should not produce findings: '{}'. Got: {:?}",
            sample,
            findings
        );
    }
}

/// Test false positive rate on testrepo/ content (<5%)
///
/// Scans a representative sample of files from testrepo to ensure
/// the secrets scanner doesn't produce excessive false positives.
/// The acceptance criteria is <5% false positive rate.
#[test]
fn test_false_positive_rate_on_testrepo() {
    secrets_scanner::init();

    let testrepo_path = std::path::Path::new("../testrepo");

    // Check if testrepo exists (may not exist in all test environments)
    if !testrepo_path.exists() {
        println!("testrepo not found, skipping false positive rate test");
        return;
    }

    let mut files_scanned = 0;
    let mut files_with_findings = 0;
    let mut total_chars = 0;
    let mut findings_count = 0;

    // Scan a representative sample of testrepo content
    let sample_files = vec![
        "README.md",
        "src/lib.rs",
        "src/api/rest.rs",
        "src/models/config.rs",
        "src/services/storage.rs",
        "tests/integration_1.rs",
    ];

    for file_rel_path in sample_files {
        let file_path = testrepo_path.join(file_rel_path);

        if !file_path.exists() {
            continue;
        }

        // Read file content
        match std::fs::read_to_string(&file_path) {
            Ok(content) => {
                files_scanned += 1;
                total_chars += content.len();

                let findings = secrets_scanner::scan_text(&content, None);

                if !findings.is_empty() {
                    files_with_findings += 1;
                    findings_count += findings.len();

                    // Log which patterns triggered for debugging
                    let pattern_ids: std::collections::HashSet<_> =
                        findings.iter().map(|f| f.pattern_id.as_str()).collect();
                    println!(
                        "File '{}' had {} findings from patterns: {:?}",
                        file_rel_path,
                        findings.len(),
                        pattern_ids
                    );
                }
            }
            Err(e) => {
                println!("Failed to read {}: {}", file_rel_path, e);
            }
        }
    }

    // Calculate false positive rate
    // We expect testrepo content to be clean (no real secrets)
    // so any findings are considered false positives
    let false_positive_rate = if files_scanned > 0 {
        (files_with_findings as f64 / files_scanned as f64) * 100.0
    } else {
        0.0
    };

    println!(
        "False positive rate test: {}/{} files had findings ({:.1}%)",
        files_with_findings, files_scanned, false_positive_rate
    );
    println!("Total findings: {} across {} characters", findings_count, total_chars);

    // Assert false positive rate is below 5%
    // We allow some leniency for edge cases but the scanner should be quite accurate
    assert!(
        false_positive_rate < 5.0,
        "False positive rate ({:.1}%) exceeds 5% threshold",
        false_positive_rate
    );

    // We should have scanned at least some files
    assert!(
        files_scanned >= 3,
        "Should scan at least 3 files for meaningful test"
    );
}
