//! Integration tests for §18.3 read-side session JSONL redaction
//!
//! Verifies that:
//! - Synthetic secrets in a session JSONL file appear as [REDACTED] after
//!   the redaction filter is applied.
//! - The raw JSONL file on disk is never modified.
//! - The per-line cache returns identical results on repeated calls.
//! - File rotation (new content at same conceptual slot) is handled transparently.
//!
//! CI command:
//!   cargo test -p hoop-daemon --test session_redaction

use hoop_daemon::redaction;
use serde_json::{json, Value};
use std::fs;
use tempfile::NamedTempFile;

// ── Fixtures ──────────────────────────────────────────────────────────────────

/// A realistic Claude Code JSONL session with a synthetic Anthropic API key
/// embedded in a user message (simulating an accidental paste).
fn synthetic_session_lines() -> Vec<String> {
    vec![
        // Metadata line
        json!({
            "type": "metadata",
            "session_id": "test-session-001",
            "cwd": "/home/coding/myproject",
            "title": "Test session with leaked key"
        }).to_string(),
        // User message containing a fake API key
        json!({
            "role": "user",
            "content": "Please use this key: sk-ant-api03-FAKEKEY1111AAAAABBBBBCCCCCDDDDDEEEEE to call the API.",
            "timestamp": "2026-04-24T10:00:00Z"
        }).to_string(),
        // Assistant reply (clean)
        json!({
            "role": "assistant",
            "content": "I can see the key you provided. I will use it now.",
            "timestamp": "2026-04-24T10:00:01Z"
        }).to_string(),
        // User message with env-var style secret
        json!({
            "role": "user",
            "content": "Also ANTHROPIC_API_KEY=sk-ant-api03-ANOTHERKEY2222BBBBBBCCCCCCDDDDDDEEEEEE in case you need it.",
            "timestamp": "2026-04-24T10:00:05Z"
        }).to_string(),
        // Message with structured content blocks (array format)
        json!({
            "role": "user",
            "content": [
                {"type": "text", "text": "My GitHub token is ghp_16C7e42F292c6912E7710c838347Ae178B4a."},
                {"type": "text", "text": "This block is clean."}
            ],
            "timestamp": "2026-04-24T10:00:10Z"
        }).to_string(),
    ]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn test_raw_file_never_modified() {
    // Write a session JSONL to a temp file
    let mut tmp = NamedTempFile::new().unwrap();
    let original_content = synthetic_session_lines().join("\n");
    fs::write(tmp.path(), &original_content).unwrap();

    // Apply redaction to each line (simulating what ws.rs does)
    let _ = synthetic_session_lines()
        .iter()
        .map(|line| {
            if let Ok(v) = serde_json::from_str::<Value>(line) {
                if let Some(content) = v.get("content") {
                    redaction::redact_json_value(content.clone())
                } else {
                    v
                }
            } else {
                Value::Null
            }
        })
        .collect::<Vec<_>>();

    // Original file must be unchanged
    let after = fs::read_to_string(tmp.path()).unwrap();
    assert_eq!(original_content, after, "Raw session file must not be modified by redaction");
    assert!(after.contains("sk-ant-api03-FAKEKEY1111"), "Raw file must still contain original key");
}

#[test]
fn test_anthropic_key_in_string_content_redacted() {
    let raw = "Please use this key: sk-ant-api03-FAKEKEY1111AAAAABBBBBCCCCCDDDDDEEEEE to call the API.";
    let out = redaction::redact_text(raw);
    assert!(out.contains("[REDACTED]"), "expected [REDACTED], got: {out}");
    assert!(!out.contains("sk-ant-api03-"), "raw key must not appear: {out}");
    assert!(out.contains("Please use this key:"), "surrounding text must be preserved: {out}");
}

#[test]
fn test_env_var_style_key_in_string_content_redacted() {
    let raw = "ANTHROPIC_API_KEY=sk-ant-api03-ANOTHERKEY2222BBBBBBCCCCCCDDDDDDEEEEEE in case you need it.";
    let out = redaction::redact_text(raw);
    assert!(out.contains("[REDACTED]"), "expected [REDACTED], got: {out}");
    assert!(!out.contains("ANOTHERKEY"), "raw key must not appear: {out}");
}

#[test]
fn test_array_content_blocks_redacted() {
    let content = json!([
        {"type": "text", "text": "My GitHub token is ghp_16C7e42F292c6912E7710c838347Ae178B4a."},
        {"type": "text", "text": "This block is clean."}
    ]);
    let out = redaction::redact_json_value(content);

    let text0 = out[0]["text"].as_str().unwrap();
    assert!(text0.contains("[REDACTED]"), "token in block 0 must be redacted: {text0}");
    assert!(!text0.contains("ghp_"), "raw token must not appear in block 0: {text0}");

    let text1 = out[1]["text"].as_str().unwrap();
    assert_eq!(text1, "This block is clean.", "clean block must be unchanged");
}

#[test]
fn test_clean_message_content_unchanged() {
    let raw = "I have reviewed the code and everything looks correct. No secrets here.";
    let out = redaction::redact_text(raw);
    assert_eq!(out, raw, "clean content must pass through unchanged");
}

#[test]
fn test_per_line_cache_consistency() {
    let raw = "The key is sk-ant-api03-CACHETEST111AAAABBBBCCCCDDDDEEEEFFFF.";
    // Call three times — second and third should hit the cache
    let r1 = redaction::redact_text(raw);
    let r2 = redaction::redact_text(raw);
    let r3 = redaction::redact_text(raw);
    assert_eq!(r1, r2, "cache must return same result");
    assert_eq!(r2, r3, "cache must return same result");
    assert!(r1.contains("[REDACTED]"), "must be redacted: {r1}");
    assert!(!r1.contains("CACHETEST"), "raw key must not appear: {r1}");
}

#[test]
fn test_file_rotation_handled_transparently() {
    // Simulate a "rotated" file: different content at the same conceptual slot.
    // Since the cache is keyed by content hash, new content = no cache hit.
    let old_content = "Key: sk-ant-api03-OLDKEY1111AAAAAABBBBBCCCCCDDDDDEEEEE";
    let new_content = "Key: sk-ant-api03-NEWKEY2222AAAAAABBBBBCCCCCDDDDDEEEEE";

    let r_old = redaction::redact_text(old_content);
    let r_new = redaction::redact_text(new_content);

    assert!(r_old.contains("[REDACTED]"), "old content must be redacted: {r_old}");
    assert!(r_new.contains("[REDACTED]"), "new content must be redacted: {r_new}");
    assert!(!r_old.contains("OLDKEY"), "old key must not appear: {r_old}");
    assert!(!r_new.contains("NEWKEY"), "new key must not appear: {r_new}");
    // Both should result in the same [REDACTED] output shape
    assert_eq!(r_old, r_new, "both rotated-file variants should produce identical redacted form");
}

#[test]
fn test_full_session_line_redaction_pipeline() {
    // Simulate the full pipeline: parse session JSONL line → extract content →
    // apply redact_json_value → verify secret is gone.
    let lines = synthetic_session_lines();

    for (i, line) in lines.iter().enumerate() {
        let v: Value = serde_json::from_str(line).expect("valid JSON");
        let content = match v.get("content") {
            Some(c) => c.clone(),
            None => continue, // metadata line has no content
        };

        let redacted = redaction::redact_json_value(content);

        // Serialise back to check for raw secrets
        let serialised = redacted.to_string();
        assert!(!serialised.contains("sk-ant-api03-"), "line {i}: raw Anthropic key must not appear after redaction");
        assert!(!serialised.contains("ghp_16C7e"), "line {i}: raw GitHub token must not appear after redaction");
        assert!(!serialised.contains("FAKEKEY"), "line {i}: raw fake key fragment must not appear after redaction");
    }
}

#[test]
fn test_jwt_in_session_redacted() {
    let raw = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
    let out = redaction::redact_text(raw);
    assert!(out.contains("[REDACTED]"), "JWT must be redacted: {out}");
    assert!(!out.contains("eyJhbGci"), "raw JWT must not appear: {out}");
}

#[test]
fn test_tool_result_content_redacted() {
    // Tool results may contain file content that includes secrets
    let content = json!([
        {
            "type": "tool_result",
            "tool_use_id": "toolu_01",
            "content": [
                {"type": "text", "text": "File contents:\nANTHROPIC_API_KEY=sk-ant-api03-TOOLRESULT111AAABBBCCCDDDEEE\nEND"}
            ]
        }
    ]);
    let out = redaction::redact_json_value(content);
    let serialised = out.to_string();
    assert!(serialised.contains("[REDACTED]"), "tool result secret must be redacted: {serialised}");
    assert!(!serialised.contains("TOOLRESULT"), "raw key fragment must not appear: {serialised}");
}
