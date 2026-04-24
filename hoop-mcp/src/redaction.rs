//! Read-side redaction filter for MCP tool responses (§18.3)
//!
//! Mirrors hoop-daemon/src/redaction.rs — same pattern set, same per-content-hash
//! cache. Applied to any session or conversation content forwarded to the agent
//! via MCP tool responses so secrets in CLI session JSONL never reach the agent.

use regex::Regex;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::sync::{LazyLock, Mutex};

const MAX_CACHE_ENTRIES: usize = 50_000;

static REDACTOR: LazyLock<Mutex<Redactor>> = LazyLock::new(|| Mutex::new(Redactor::new()));

/// Redact a text string using the process-wide cache.
pub fn redact_text(text: &str) -> String {
    match REDACTOR.lock() {
        Ok(mut r) => r.redact(text),
        Err(_) => apply_patterns_uncached(text),
    }
}

/// Redact a JSON string value returned by a tool. Non-string input is returned as-is.
pub fn redact_json_string(value: &serde_json::Value) -> String {
    if let Some(s) = value.as_str() {
        redact_text(s)
    } else {
        value.to_string()
    }
}

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

        if self.cache.len() >= MAX_CACHE_ENTRIES {
            self.cache.clear();
        }
        self.cache.insert(key, result.clone());
        result
    }
}

fn build_patterns() -> Vec<Regex> {
    vec![
        Regex::new(r"sk-ant-[a-zA-Z0-9_-]{20,}").unwrap(),
        Regex::new(r"\bsk-[a-zA-Z0-9]{20,}\b").unwrap(),
        Regex::new(r"\bAKIA[A-Z0-9]{16}\b").unwrap(),
        Regex::new(r"\bghp_[a-zA-Z0-9]{36}\b").unwrap(),
        Regex::new(r"\bghs_[a-zA-Z0-9]{36}\b").unwrap(),
        Regex::new(r"\bghu_[a-zA-Z0-9]{36}\b").unwrap(),
        Regex::new(r"\bgithub_pat_[a-zA-Z0-9_]{82}\b").unwrap(),
        Regex::new(r"\bxoxb-[0-9A-Za-z-]{24,}\b").unwrap(),
        Regex::new(r"\bxoxp-[0-9A-Za-z-]{24,}\b").unwrap(),
        Regex::new(r"\bey[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b").unwrap(),
        Regex::new(r"(?i)bearer\s+[A-Za-z0-9._\-+/]{20,}").unwrap(),
        Regex::new(
            r#"(?i)(?:api[_-]?key|secret[_-]?key|access[_-]?token|auth[_-]?token|private[_-]?key|client[_-]?secret|anthropic[_-]?api[_-]?key|openai[_-]?api[_-]?key|github[_-]?token)\s*[:=]\s*["']?([A-Za-z0-9+/_.~\-]{16,})["']?"#
        ).unwrap(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anthropic_key_redacted() {
        let out = apply_patterns_uncached("sk-ant-api03-AAAA1111BBBB2222CCCC3333DDDD4444");
        assert!(out.contains("[REDACTED]"), "got: {out}");
    }

    #[test]
    fn test_conversation_content_redacted() {
        let content = r#"Here is my api_key=sk-ant-api03-XXXX1111YYYY2222ZZZZ3333AAAA4444 use it"#;
        let out = redact_text(content);
        assert!(out.contains("[REDACTED]"), "got: {out}");
        assert!(!out.contains("sk-ant-"), "got: {out}");
    }

    #[test]
    fn test_plain_text_unchanged() {
        let s = "This message has no secrets in it.";
        assert_eq!(redact_text(s), s);
    }
}
