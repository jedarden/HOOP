//! Upload secrets scanning tests (§18)
//!
//! Tests that attachments are scanned for secrets during upload:
//! - Text attachments are scanned before finalizing upload
//! - Findings trigger appropriate redaction policy actions
//! - Binary files are handled correctly (fail-open)
//! - Reject policy blocks uploads with secrets
//! - Warn policy allows uploads but records findings

use std::io::Write;
use tempfile::TempDir;

/// Helper to create a test attachment with secrets
fn create_attachment_with_secrets(
    dir: &TempDir,
    filename: &str,
    content: &str,
) -> std::path::PathBuf {
    let path = dir.path().join(filename);
    let mut file = std::fs::File::create(&path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    path
}

/// Helper to create a clean attachment (no secrets)
fn create_clean_attachment(dir: &TempDir, filename: &str, content: &str) -> std::path::PathBuf {
    create_attachment_with_secrets(dir, filename, content)
}

/// Test that text attachments with secrets are detected
#[test]
fn test_text_attachment_with_secrets_detected() {
    hoop_daemon::secrets_scanner::init();

    let dir = TempDir::new().unwrap();

    // Create an attachment with an Anthropic API key
    let attachment_path = create_attachment_with_secrets(
        &dir,
        "config.txt",
        "ANTHROPIC_API_KEY=sk-ant-api03-AAAA1111BBBB2222CCCC3333DDDD4444EEEE5555FFFF6666",
    );

    let findings = hoop_daemon::redaction::scan_attachment(&attachment_path).unwrap();

    assert!(!findings.is_empty(), "Should detect secret in attachment");
    assert!(
        findings
            .iter()
            .any(|f| f.pattern_name == "anthropic_api_key"),
        "Should detect anthropic_api_key pattern"
    );
}

/// Test that text attachments with multiple secrets are all detected
#[test]
fn test_attachment_with_multiple_secrets() {
    hoop_daemon::secrets_scanner::init();

    let dir = TempDir::new().unwrap();

    let content = r#"
# Configuration
stripe_key: sk_live_51AbCdEf1234567890AbCdEf
github_token: ghp_16C7e42F292c6912E7710c838347Ae178B4a
aws_access_key: AKIAIOSFODNN7EXAMPLE
"#;

    let attachment_path = create_attachment_with_secrets(&dir, "config.yml", content);

    let findings = hoop_daemon::redaction::scan_attachment(&attachment_path).unwrap();

    assert!(findings.len() >= 3, "Should detect at least 3 secrets");

    let pattern_ids: std::collections::HashSet<_> =
        findings.iter().map(|f| f.pattern_name).collect();

    assert!(pattern_ids.contains("stripe_api_key"));
    assert!(pattern_ids.contains("github_token"));
    assert!(pattern_ids.contains("aws_access_key"));
}

/// Test that clean attachments produce no findings
#[test]
fn test_clean_attachment_no_findings() {
    hoop_daemon::secrets_scanner::init();

    let dir = TempDir::new().unwrap();

    let content = r#"
# Project Documentation
This is a normal README file with no secrets.
It contains links like https://example.com and email addresses like user@example.com.
"#;

    let attachment_path = create_clean_attachment(&dir, "README.md", content);

    let findings = hoop_daemon::redaction::scan_attachment(&attachment_path).unwrap();

    assert!(
        findings.is_empty(),
        "Clean attachment should have no findings"
    );
}

/// Test that different text file extensions are scanned
#[test]
fn test_various_text_extensions_scanned() {
    hoop_daemon::secrets_scanner::init();

    let dir = TempDir::new().unwrap();

    let secret_content = "API_KEY=sk-ant-api03-TEST1234567890ABCDEFGHIJ";

    let extensions = vec!["txt", "md", "json", "yaml", "yml", "env", "sh", "py", "rs"];

    for ext in extensions {
        let filename = format!("config.{}", ext);
        let attachment_path = create_attachment_with_secrets(&dir, &filename, secret_content);

        let findings = hoop_daemon::redaction::scan_attachment(&attachment_path).unwrap();

        assert!(
            !findings.is_empty(),
            "Should detect secret in .{} file",
            ext
        );
    }
}

/// Test that binary files are not scanned (fail-open)
#[test]
fn test_binary_files_not_scanned() {
    hoop_daemon::secrets_scanner::init();

    let dir = TempDir::new().unwrap();

    // Create a fake binary file (PNG magic bytes + some data)
    let path = dir.path().join("image.png");
    let mut file = std::fs::File::create(&path).unwrap();
    file.write_all(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A])
        .unwrap(); // PNG header
    file.write_all(b"sk-ant-api03-FAKESECRET").unwrap(); // Secret-like data

    let findings = hoop_daemon::redaction::scan_attachment(&path).unwrap();

    // Binary files should be skipped
    assert!(findings.is_empty(), "Binary files should not be scanned");
}

/// Test that JSON files with secrets in nested structures are detected
#[test]
fn test_json_nested_secrets_detected() {
    hoop_daemon::secrets_scanner::init();

    let dir = TempDir::new().unwrap();

    let content = r#"{
  "database": {
    "host": "localhost",
    "credentials": {
      "username": "admin",
      "password": "sk-ant-api03-SECRET1234567890ABCDEFGHIJ"
    }
  },
  "api": {
    "key": "ghp_16C7e42F292c6912E7710c838347Ae178B4a"
  }
}"#;

    let attachment_path = create_attachment_with_secrets(&dir, "config.json", content);

    let findings = hoop_daemon::redaction::scan_attachment(&attachment_path).unwrap();

    assert!(!findings.is_empty(), "Should detect secrets in JSON");

    let pattern_ids: std::collections::HashSet<_> =
        findings.iter().map(|f| f.pattern_name).collect();

    // Should detect both the Anthropic key and GitHub token
    assert!(
        pattern_ids.contains("anthropic_api_key"),
        "Should detect anthropic_api_key in nested JSON"
    );
    assert!(
        pattern_ids.contains("github_token"),
        "Should detect github_token in nested JSON"
    );
}

/// Test that environment variable assignments are detected
#[test]
fn test_env_var_assignments_detected() {
    hoop_daemon::secrets_scanner::init();

    let dir = TempDir::new().unwrap();

    let content = r#"#!/bin/bash
export OPENAI_API_KEY=sk-proj-AbCdEf1234567890AbCdEf
export ANTHROPIC_API_KEY=sk-ant-api03-TEST1234567890ABCDEFGHIJ
export AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
"#;

    let attachment_path = create_attachment_with_secrets(&dir, "setup.sh", content);

    let findings = hoop_daemon::redaction::scan_attachment(&attachment_path).unwrap();

    assert!(
        findings.len() >= 2,
        "Should detect at least 2 env var secrets"
    );
}

/// Test that very large text files are skipped
#[test]
fn test_large_files_skipped() {
    hoop_daemon::secrets_scanner::init();

    let dir = TempDir::new().unwrap();

    let path = dir.path().join("large.txt");

    // Create a file larger than 10MB limit
    let mut file = std::fs::File::create(&path).unwrap();
    let large_data = "A".repeat(11 * 1024 * 1024); // 11MB
    file.write_all(large_data.as_bytes()).unwrap();

    let findings = hoop_daemon::redaction::scan_attachment(&path).unwrap();

    // Should be skipped due to size
    assert!(findings.is_empty(), "Large files should be skipped");
}

/// Test that scan findings have correct metadata
#[test]
fn test_scan_finding_metadata() {
    hoop_daemon::secrets_scanner::init();

    let dir = TempDir::new().unwrap();

    let content = "My key is sk-ant-api03-TEST1234567890ABCDEFGHIJ end";
    let attachment_path = create_attachment_with_secrets(&dir, "test.txt", content);

    let findings = hoop_daemon::redaction::scan_attachment(&attachment_path).unwrap();

    assert!(!findings.is_empty());
    let finding = &findings[0];

    assert!(!finding.pattern_name.is_empty());
    assert!(finding.match_start < content.len());
    assert!(finding.match_start + finding.match_len <= content.len());
}

/// Test findings are written to audit log
#[test]
fn test_findings_written_to_audit() {
    hoop_daemon::secrets_scanner::init();

    let findings = vec![hoop_daemon::redaction::SecretFinding {
        pattern_name: "test_pattern",
        match_start: 0,
        match_len: 20,
    }];

    // This should write to audit without errors
    let written = hoop_daemon::redaction::audit_findings(
        "attachment",
        &findings,
        hoop_daemon::redaction_policy::RedactionAction::Warn,
        "test-attachment-id",
        Some("test-project"),
        "system",
    );

    assert_eq!(written, 1, "Should write one audit entry");
}

/// Test benign files don't produce false positives
#[test]
fn test_benign_files_no_false_positives() {
    hoop_daemon::secrets_scanner::init();

    let dir = TempDir::new().unwrap();

    // Test various benign file contents
    let benign_files = vec![
        (
            "README.md",
            "Check out https://github.com/user/repo for more info",
        ),
        ("config.json", r#"{"timeout": 30, "retries": 3}"#),
        ("python.py", "print('Hello, World!')"),
        ("rust.rs", "fn main() { println!(\"Hello\"); }"),
        (
            "yaml.yml",
            "production:\n  replicas: 3\n  image: nginx:latest",
        ),
    ];

    for (filename, content) in benign_files {
        let attachment_path = create_clean_attachment(&dir, filename, content);
        let findings = hoop_daemon::redaction::scan_attachment(&attachment_path).unwrap();

        assert!(
            findings.is_empty(),
            "Benign file '{}' should have no findings, got: {:?}",
            filename,
            findings
        );
    }
}
