//! Read-side redaction filter for CLI session JSONL content (§18.3)
//!
//! Applies secrets-scanner patterns to text extracted from session JSONL before
//! HOOP UI projections or MCP tool responses. Raw CLI session files are never
//! modified — only the projections HOOP emits are redacted.
//!
//! Per-line content-hash cache avoids re-scanning unchanged lines.
//! Cache automatically handles file rotation: new session content produces new
//! hashes, so rotated files get fresh redaction without explicit eviction.
//! Call `clear_cache()` after bulk reloads to reclaim memory.

use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::sync::{LazyLock, Mutex};

/// Maximum cached entries before the cache is cleared.
const MAX_CACHE_ENTRIES: usize = 50_000;

// ── Global singleton ──────────────────────────────────────────────────────────

static REDACTOR: LazyLock<Mutex<Redactor>> = LazyLock::new(|| Mutex::new(Redactor::new()));

/// Redact a text string, returning a new string with secrets replaced.
/// Uses the process-wide cache; safe to call from multiple threads.
pub fn redact_text(text: &str) -> String {
    match REDACTOR.lock() {
        Ok(mut r) => r.redact(text),
        Err(_) => apply_patterns_uncached(text),
    }
}

/// Recursively redact all string values within a JSON value.
/// Objects, arrays, and non-string primitives are traversed but not altered
/// in structure — only string leaves are redacted.
pub fn redact_json_value(value: Value) -> Value {
    match value {
        Value::String(s) => Value::String(redact_text(&s)),
        Value::Array(arr) => Value::Array(arr.into_iter().map(redact_json_value).collect()),
        Value::Object(mut obj) => {
            for v in obj.values_mut() {
                *v = redact_json_value(v.take());
            }
            Value::Object(obj)
        }
        other => other,
    }
}

/// Clear the process-wide cache. Call after large bulk reloads to reclaim memory.
pub fn clear_cache() {
    if let Ok(mut r) = REDACTOR.lock() {
        r.cache.clear();
    }
}

// ── Redactor ──────────────────────────────────────────────────────────────────

struct Redactor {
    patterns: Vec<Regex>,
    cache: HashMap<u64, String>,
}

impl Redactor {
    fn new() -> Self {
        Self {
            patterns: build_patterns(),
            cache: HashMap::new(),
        }
    }

    fn redact(&mut self, text: &str) -> String {
        let key = hash_str(text);
        if let Some(cached) = self.cache.get(&key) {
            return cached.clone();
        }

        let result = apply_patterns(&self.patterns, text);

        // Simple bounded eviction: clear when full.
        if self.cache.len() >= MAX_CACHE_ENTRIES {
            self.cache.clear();
        }
        self.cache.insert(key, result.clone());
        result
    }
}

// ── Pattern set ───────────────────────────────────────────────────────────────

fn build_patterns() -> Vec<Regex> {
    vec![
        // Anthropic API keys: sk-ant-api03-*
        Regex::new(r"sk-ant-[a-zA-Z0-9_-]{20,}").unwrap(),
        // Generic sk-* keys (OpenAI, etc.)
        Regex::new(r"\bsk-[a-zA-Z0-9]{20,}\b").unwrap(),
        // AWS Access Key ID
        Regex::new(r"\bAKIA[A-Z0-9]{16}\b").unwrap(),
        // GitHub personal access tokens (classic + fine-grained)
        Regex::new(r"\bghp_[a-zA-Z0-9]{36}\b").unwrap(),
        Regex::new(r"\bghs_[a-zA-Z0-9]{36}\b").unwrap(),
        Regex::new(r"\bghu_[a-zA-Z0-9]{36}\b").unwrap(),
        Regex::new(r"\bgithub_pat_[a-zA-Z0-9_]{82}\b").unwrap(),
        // Slack bot/user tokens
        Regex::new(r"\bxoxb-[0-9A-Za-z-]{24,}\b").unwrap(),
        Regex::new(r"\bxoxp-[0-9A-Za-z-]{24,}\b").unwrap(),
        // JWTs: three base64url segments
        Regex::new(r"\bey[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b").unwrap(),
        // Bearer tokens in HTTP headers / curl calls
        Regex::new(r"(?i)bearer\s+[A-Za-z0-9._\-+/]{20,}").unwrap(),
        // env-var style assignments: API_KEY=<value>
        Regex::new(
            r#"(?i)(?:api[_-]?key|secret[_-]?key|access[_-]?token|auth[_-]?token|private[_-]?key|client[_-]?secret|anthropic[_-]?api[_-]?key|openai[_-]?api[_-]?key|github[_-]?token)\s*[:=]\s*["']?([A-Za-z0-9+/_.~\-]{16,})["']?"#
        ).unwrap(),
        // JSON-object style: "password": "…"  / "token": "…"
        Regex::new(
            r#"(?i)"(?:password|passwd|secret|token|api_key|apikey|access_token|auth_token|private_key|client_secret)"\s*:\s*"([^"]{8,})""#
        ).unwrap(),
    ]
}

fn apply_patterns(patterns: &[Regex], text: &str) -> String {
    let mut out = text.to_owned();
    for re in patterns {
        out = re.replace_all(&out, "[REDACTED]").into_owned();
    }
    out
}

fn apply_patterns_uncached(text: &str) -> String {
    apply_patterns(&build_patterns(), text)
}

fn hash_str(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn redact(s: &str) -> String {
        apply_patterns_uncached(s)
    }

    #[test]
    fn test_anthropic_key_redacted() {
        let input = "ANTHROPIC_API_KEY=sk-ant-api03-AAAA1111BBBB2222CCCC3333DDDD4444EEEE5555FFFF6666";
        let out = redact(input);
        assert!(out.contains("[REDACTED]"), "expected redaction, got: {out}");
        assert!(!out.contains("sk-ant-"), "raw key must not appear in output");
    }

    #[test]
    fn test_openai_sk_key_redacted() {
        let input = "Key: sk-ABCDEFGHIJKLMNOPQRSTUVWXYZ123456";
        let out = redact(input);
        assert!(out.contains("[REDACTED]"), "got: {out}");
        assert!(!out.contains("sk-ABCDEF"), "got: {out}");
    }

    #[test]
    fn test_aws_access_key_redacted() {
        let input = "aws_access_key_id = AKIAIOSFODNN7EXAMPLE";
        let out = redact(input);
        assert!(out.contains("[REDACTED]"), "got: {out}");
        assert!(!out.contains("AKIAIO"), "got: {out}");
    }

    #[test]
    fn test_github_token_redacted() {
        let input = "token ghp_16C7e42F292c6912E7710c838347Ae178B4a";
        let out = redact(input);
        assert!(out.contains("[REDACTED]"), "got: {out}");
        assert!(!out.contains("ghp_"), "got: {out}");
    }

    #[test]
    fn test_bearer_token_redacted() {
        let input = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
        let out = redact(input);
        assert!(out.contains("[REDACTED]"), "got: {out}");
        assert!(!out.contains("Bearer ey"), "got: {out}");
    }

    #[test]
    fn test_jwt_redacted() {
        let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
        let out = redact(jwt);
        assert!(out.contains("[REDACTED]"), "got: {out}");
    }

    #[test]
    fn test_json_password_field_redacted() {
        let input = r#"{"password": "s3cr3tP@ssw0rd!"}"#;
        let out = redact(input);
        assert!(out.contains("[REDACTED]"), "got: {out}");
        assert!(!out.contains("s3cr3t"), "got: {out}");
    }

    #[test]
    fn test_json_api_key_field_redacted() {
        let input = r#"{"api_key": "abc123def456ghi789jkl"}"#;
        let out = redact(input);
        assert!(out.contains("[REDACTED]"), "got: {out}");
    }

    #[test]
    fn test_plain_text_unchanged() {
        let input = "This is a normal message with no secrets.";
        let out = redact(input);
        assert_eq!(out, input);
    }

    #[test]
    fn test_git_hash_not_redacted() {
        // Git commit SHAs look like long hex strings but should not be redacted
        // (they don't match any of our specific patterns)
        let input = "commit abc123def456789abcdef0123456789abcdef01";
        let out = redact(input);
        assert_eq!(out, input, "git hashes must not be redacted");
    }

    #[test]
    fn test_redact_json_value_string() {
        let v = Value::String("sk-ant-api03-AAAA1111BBBB2222CCCC3333DDDD4444EEEE5555".to_string());
        let out = redact_json_value(v);
        assert_eq!(out, Value::String("[REDACTED]".to_string()));
    }

    #[test]
    fn test_redact_json_value_array_text_block() {
        let v = serde_json::json!([
            {"type": "text", "text": "The API key is sk-ant-api03-AAAA1111BBBB2222CCCC3333DDDD4444EEEE5555 please use it"},
            {"type": "text", "text": "Nothing sensitive here"}
        ]);
        let out = redact_json_value(v);
        let text0 = out[0]["text"].as_str().unwrap();
        assert!(text0.contains("[REDACTED]"), "got: {text0}");
        assert!(!text0.contains("sk-ant-"), "got: {text0}");
        let text1 = out[1]["text"].as_str().unwrap();
        assert_eq!(text1, "Nothing sensitive here");
    }

    #[test]
    fn test_redact_json_value_nested_object() {
        let v = serde_json::json!({
            "role": "user",
            "content": [
                {"type": "text", "text": "My token: ghp_16C7e42F292c6912E7710c838347Ae178B4a"}
            ]
        });
        let out = redact_json_value(v);
        let text = out["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("[REDACTED]"), "got: {text}");
        assert!(!text.contains("ghp_"), "got: {text}");
    }

    #[test]
    fn test_cache_returns_same_result() {
        // Verify the cache doesn't corrupt results across calls
        let input = "token: sk-ant-api03-XXXX1111YYYY2222ZZZZ3333AAAA4444BBBB5555";
        let first = redact_text(input);
        let second = redact_text(input);
        let third = redact_text(input);
        assert_eq!(first, second);
        assert_eq!(second, third);
        assert!(first.contains("[REDACTED]"), "got: {first}");
    }

    #[test]
    fn test_env_var_assignment_redacted() {
        let input = "export ANTHROPIC_API_KEY=sk-ant-api03-testkey1234567890abcdefgh";
        let out = redact(input);
        assert!(out.contains("[REDACTED]"), "got: {out}");
        assert!(!out.contains("testkey"), "got: {out}");
    }
}
