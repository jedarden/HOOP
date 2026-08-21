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
//!
//! ## Pattern Source (§18)
//!
//! All patterns are sourced from config.yml via `config_resolver::SecretPattern`.
//! This ensures a single source of truth for both client and backend scanners.

use regex::Regex;
use serde_json::Value;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{LazyLock, Mutex};

/// Maximum cached entries before the cache is cleared.
const MAX_CACHE_ENTRIES: usize = 50_000;

// ── Global singletons ──────────────────────────────────────────────────────────

static REDACTOR: LazyLock<Mutex<Redactor>> = LazyLock::new(|| Mutex::new(Redactor::new()));

/// Named patterns for detection. Updated by `update_patterns_with_names()`.
///
/// This is the single source of truth for scanning. Each entry is `(id, Regex)`.
/// The ids match the `id` field from `config_resolver::SecretPattern`.
static NAMED_PATTERNS: LazyLock<Mutex<Vec<(&'static str, Regex)>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

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

/// Update the redaction patterns from a list of regex strings.
///
/// Compiles each pattern string into a Regex and replaces the current
/// pattern set in the global REDACTOR. Invalid patterns are logged and skipped.
///
/// Note: This function does NOT update the named patterns used by `scan_text_for_secrets()`.
/// Use `update_patterns_with_names()` for full synchronization.
pub fn update_patterns(pattern_strings: &[String]) {
    let mut new_patterns = Vec::new();
    for (i, pat_str) in pattern_strings.iter().enumerate() {
        match Regex::new(pat_str) {
            Ok(re) => new_patterns.push(re),
            Err(e) => {
                tracing::warn!(
                    "Invalid redaction pattern at index {}: '{}' - {}. Skipping.",
                    i,
                    pat_str,
                    e
                );
            }
        }
    }

    if let Ok(mut r) = REDACTOR.lock() {
        r.patterns = new_patterns;
        r.cache.clear(); // Clear cache to avoid stale matches
        tracing::info!(
            "Redaction patterns updated: {} patterns loaded",
            r.patterns.len()
        );
    }
}

/// Update both redaction and scanning patterns from named patterns.
///
/// This is the primary pattern update function. It synchronizes both:
/// 1. The redaction patterns (used by `redact_text()`)
/// 2. The named scanning patterns (used by `scan_text_for_secrets()`)
///
/// The pattern names are stored as static strings for the lifetime of the process.
/// This ensures that scanning and redaction always use the same pattern set.
///
/// # Arguments
/// * `patterns` - Slice of `(name, pattern_string)` tuples
pub fn update_patterns_with_names(patterns: &[(&str, String)]) {
    let mut redaction_patterns = Vec::new();
    let mut named_patterns = Vec::new();

    for (name, pat_str) in patterns {
        match Regex::new(pat_str) {
            Ok(re) => {
                redaction_patterns.push(re.clone());
                // Leak the name string to get a &'static str
                // This is safe because the patterns live for the process lifetime
                let static_name: &'static str = Box::leak(name.to_string().into_boxed_str());
                named_patterns.push((static_name, re));
            }
            Err(e) => {
                tracing::warn!(
                    "Invalid redaction pattern '{}': '{}' - {}. Skipping.",
                    name,
                    pat_str,
                    e
                );
            }
        }
    }

    if let Ok(mut r) = REDACTOR.lock() {
        r.patterns = redaction_patterns;
        r.cache.clear(); // Clear cache to avoid stale matches
        tracing::info!(
            "Redaction patterns updated: {} patterns loaded",
            r.patterns.len()
        );
    }

    if let Ok(mut np) = NAMED_PATTERNS.lock() {
        *np = named_patterns;
        tracing::info!("Named patterns updated: {} patterns loaded", np.len());
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
            patterns: Vec::new(), // Patterns loaded via update_patterns_with_names()
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

// ── Pattern application ───────────────────────────────────────────────────────────

fn apply_patterns(patterns: &[Regex], text: &str) -> String {
    let mut out = text.to_owned();
    for re in patterns {
        out = re.replace_all(&out, "[REDACTED]").into_owned();
    }
    out
}

fn apply_patterns_uncached(text: &str) -> String {
    // Fallback: use current patterns from REDACTOR
    if let Ok(r) = REDACTOR.lock() {
        apply_patterns(&r.patterns, text)
    } else {
        // If lock fails, return text unchanged (fail-safe)
        text.to_string()
    }
}

fn hash_str(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

// ── Scanning (detection without mutation) ─────────────────────────────────────

/// A secret detected in scanned text. Returned by `scan_text_for_secrets`.
///
/// Per §18.1 the finding is **flagged, not blocked** — nothing is silently
/// deleted. The operator sees which surface was scanned and how many findings
/// were detected, then chooses to redact-in-place, redact-and-delete, or
/// proceed anyway.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretFinding {
    /// Which scanner pattern matched (e.g. `"anthropic_api_key"`).
    pub pattern_name: &'static str,
    /// Byte offset of the start of the match within the scanned text.
    pub match_start: usize,
    /// Length of the matched substring in bytes.
    pub match_len: usize,
}

/// Scan `text` for secrets and return all findings.
///
/// Returns an empty vec if no secrets are detected. The text is never mutated
/// — this is detection-only. Use `redact_text` if you also want to replace
/// findings with `[REDACTED]`.
///
/// Returned findings may overlap when the same key matches multiple patterns
/// (e.g. both `anthropic_api_key` and `env_var_secret`).
pub fn scan_text_for_secrets(text: &str) -> Vec<SecretFinding> {
    let mut findings = Vec::new();
    if let Ok(named_patterns) = NAMED_PATTERNS.lock() {
        for (name, re) in named_patterns.iter() {
            for m in re.find_iter(text) {
                findings.push(SecretFinding {
                    pattern_name: name,
                    match_start: m.start(),
                    match_len: m.len(),
                });
            }
        }
    }
    findings
}

// ── Per-surface scanning entry points (§18) ───────────────────────────────────
//
// Each function is a thin named wrapper over `scan_text_for_secrets`. The name
// labels the ingestion surface so call sites are self-documenting in review.

/// Phase 3: Scan a Whisper voice transcript for secrets before storage (§18.2).
pub fn scan_voice_transcript(transcript: &str) -> Vec<SecretFinding> {
    scan_text_for_secrets(transcript)
}

/// Phase 3: Scan text extracted from a screen-capture frame for secrets (§18.1).
///
/// Frame text may be produced by OCR of individual frames or from the
/// narration transcript attached to a screen walkthrough.
pub fn scan_screen_capture_text(frame_text: &str) -> Vec<SecretFinding> {
    scan_text_for_secrets(frame_text)
}

/// Phase 4: Scan a draft title and optional description body for secrets (§18.1).
///
/// Both single-item and bulk-draft creation paths call this before the draft
/// is inserted into the queue.
pub fn scan_draft_body(title: &str, body: Option<&str>) -> Vec<SecretFinding> {
    let mut combined = title.to_owned();
    if let Some(b) = body {
        combined.push('\n');
        combined.push_str(b);
    }
    scan_text_for_secrets(&combined)
}

/// Phase 5: Scan a morning brief's markdown content for secrets before storage
/// and before it is forwarded to Stitches (§18.1 lateral-leak prevention).
pub fn scan_morning_brief(content: &str) -> Vec<SecretFinding> {
    scan_text_for_secrets(content)
}

/// Phase 5: Scan a cross-project propagation draft for secrets (§18.1).
///
/// Propagation drafts are synthesised by the human-interface agent from
/// patterns observed in one project and proposed for sibling projects. Any
/// secret embedded in the source project's context must not propagate laterally.
pub fn scan_propagation_draft(title: &str, body: &str) -> Vec<SecretFinding> {
    let combined = format!("{title}\n{body}");
    scan_text_for_secrets(&combined)
}

/// Scan a text-based attachment file for secrets (§18.1).
///
/// Only scans files that are likely to contain text (based on extension).
/// Binary files (images, PDFs, etc.) are skipped since they require OCR.
/// Returns findings if the file could be read and scanned, or an error.
pub fn scan_attachment(path: &std::path::Path) -> Result<Vec<SecretFinding>, std::io::Error> {
    // Only scan text-based file extensions
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    let is_text_file = matches!(
        extension.to_lowercase().as_str(),
        "txt"
            | "md"
            | "markdown"
            | "json"
            | "yaml"
            | "yml"
            | "toml"
            | "ini"
            | "cfg"
            | "conf"
            | "sh"
            | "bash"
            | "zsh"
            | "fish"
            | "ps1"
            | "py"
            | "rs"
            | "js"
            | "ts"
            | "jsx"
            | "tsx"
            | "go"
            | "java"
            | "c"
            | "cpp"
            | "h"
            | "hpp"
            | "cs"
            | "php"
            | "rb"
            | "lua"
            | "pl"
            | "sql"
            | "env"
            | "dockerenv"
            | "gitignore"
            | "gitattributes"
    );

    if !is_text_file {
        return Ok(Vec::new());
    }

    // Read file content with size limit (10MB)
    let metadata = std::fs::metadata(path)?;
    if metadata.len() > 10 * 1024 * 1024 {
        tracing::warn!(
            path = %path.display(),
            size = metadata.len(),
            "Attachment too large for secret scanning, skipping"
        );
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(path)?;
    Ok(scan_text_for_secrets(&content))
}

/// Scan a CLI session JSONL file for secrets (§18.1).
///
/// Parses the JSONL file and scans all text content for secrets.
/// This is used when sessions are filtered by project before display.
pub fn scan_session_jsonl(path: &std::path::Path) -> Result<Vec<SecretFinding>, std::io::Error> {
    let metadata = std::fs::metadata(path)?;
    if metadata.len() > 50 * 1024 * 1024 {
        tracing::warn!(
            path = %path.display(),
            size = metadata.len(),
            "Session file too large for secret scanning, skipping"
        );
        return Ok(Vec::new());
    }

    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let mut all_findings = Vec::new();

    for line in std::io::BufRead::lines(reader) {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        // Parse JSON and extract all string values for scanning
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) {
            let text_content = extract_all_strings(&value);
            all_findings.extend(scan_text_for_secrets(&text_content));
        }
    }

    Ok(all_findings)
}

/// Recursively extract all string values from a JSON value.
///
/// This helper function walks through JSON objects, arrays, and primitives
/// to collect all string values into a single text blob for scanning.
fn extract_all_strings(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => {
            let mut result = String::new();
            for item in arr {
                let extracted = extract_all_strings(item);
                if !extracted.is_empty() {
                    if !result.is_empty() {
                        result.push(' ');
                    }
                    result.push_str(&extracted);
                }
            }
            result
        }
        serde_json::Value::Object(obj) => {
            let mut result = String::new();
            for (_key, val) in obj {
                let extracted = extract_all_strings(val);
                if !extracted.is_empty() {
                    if !result.is_empty() {
                        result.push(' ');
                    }
                    result.push_str(&extracted);
                }
            }
            result
        }
        _ => String::new(),
    }
}

// ── Audit integration (§18.5) ─────────────────────────────────────────────────────

/// Write redaction audit entries for detected findings.
///
/// This helper function writes an audit entry for each unique pattern found
/// in the scan results. Call this after any scan operation to record what
/// was flagged for operator review.
///
/// # Arguments
/// * `what_flagged` - What was scanned (e.g., "transcript", "attachment", "draft")
/// * `findings` - Secret findings from a scan operation
/// * `action` - Action taken (flagged_only, redacted_in_place, proceeded_anyway, rejected)
/// * `source_ref` - Reference to the source (stitch_id, attachment_id, etc.)
/// * `project` - Optional project name
/// * `operator` - Operator who triggered the scan (or "system" for automatic scans)
///
/// # Returns
/// Number of audit entries written (one per unique pattern)
pub fn audit_findings(
    what_flagged: &str,
    findings: &[SecretFinding],
    action: crate::redaction_policy::RedactionAction,
    source_ref: &str,
    project: Option<&str>,
    operator: &str,
) -> usize {
    use crate::fleet;
    use std::collections::HashSet;

    if findings.is_empty() {
        return 0;
    }

    // Collect unique pattern names
    let unique_patterns: HashSet<&'static str> = findings.iter().map(|f| f.pattern_name).collect();

    let mut written = 0;
    for pattern_name in unique_patterns {
        let match_count = findings
            .iter()
            .filter(|f| f.pattern_name == pattern_name)
            .count();

        // Build metadata with match count and positions
        let metadata = serde_json::json!({
            "match_count": match_count,
            "matches": findings.iter()
                .filter(|f| f.pattern_name == pattern_name)
                .map(|f| {
                    serde_json::json!({
                        "offset": f.match_start,
                        "length": f.match_len,
                    })
                })
                .collect::<Vec<_>>()
        });

        if let Err(e) = fleet::insert_redaction_audit(
            what_flagged,
            pattern_name,
            action,
            operator,
            Some(source_ref),
            project,
            Some(metadata),
        ) {
            tracing::error!(
                what_flagged,
                pattern_name,
                source_ref,
                error = %e,
                "Failed to write redaction audit entry"
            );
        } else {
            written += 1;
        }
    }

    written
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_resolver::SecretPattern;

    /// Initialize default patterns for tests.
    /// This ensures both REDACTOR and NAMED_PATTERNS are populated.
    fn init_default_patterns() {
        let default_patterns = crate::config_resolver::default_secret_patterns();
        let named_patterns = SecretPattern::to_named_patterns(&default_patterns);
        update_patterns_with_names(&named_patterns);
    }

    fn redact(s: &str) -> String {
        apply_patterns_uncached(s)
    }

    #[test]
    fn test_anthropic_key_redacted() {
        let input =
            "ANTHROPIC_API_KEY=sk-ant-api03-AAAA1111BBBB2222CCCC3333DDDD4444EEEE5555FFFF6666";
        let out = redact(input);
        assert!(out.contains("[REDACTED]"), "expected redaction, got: {out}");
        assert!(
            !out.contains("sk-ant-"),
            "raw key must not appear in output"
        );
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

    // ── Client-Backend Parity Tests (§18) ─────────────────────────────────────────────
    //
    // These tests use the same fixtures as the client-side tests in
    // hoop-ui/web/src/secretsScanner.test.ts to verify parity between
    // client pre-upload warnings and backend authoritative scanning.

    #[test]
    fn test_parity_anthropic_key() {
        init_default_patterns();
        // Fixture from client test: anthropicKey
        let input = "Here is my key sk-ant-api03-AAAA1111BBBB2222CCCC3333DDDD4444EEEE5555FFFF6666 please keep it safe";
        let findings = scan_text_for_secrets(input);
        assert!(!findings.is_empty(), "should detect Anthropic API key");
        assert!(
            findings
                .iter()
                .any(|f| f.pattern_name == "anthropic_api_key"),
            "should detect anthropic_api_key pattern"
        );
    }

    #[test]
    fn test_parity_generic_sk_key() {
        init_default_patterns();
        // Fixture from client test: genericKey
        let input = "Key: sk-ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890abcdefghijklmn";
        let findings = scan_text_for_secrets(input);
        assert!(!findings.is_empty(), "should detect generic API key");
        assert!(
            findings.iter().any(|f| f.pattern_name == "generic_sk_key"),
            "should detect generic_sk_key pattern"
        );
    }

    #[test]
    fn test_parity_aws_access_key() {
        init_default_patterns();
        // Fixture from client test: awsKey
        let input = "aws_access_key_id = AKIAIOSFODNN7EXAMPLE";
        let findings = scan_text_for_secrets(input);
        assert!(!findings.is_empty(), "should detect AWS access key");
        assert!(
            findings.iter().any(|f| f.pattern_name == "aws_access_key"),
            "should detect aws_access_key pattern"
        );
    }

    #[test]
    fn test_parity_github_token() {
        init_default_patterns();
        // Fixture from client test: githubToken
        let input = "token=ghp_16C7e42F292c6912E7710c838347Ae178B4a";
        let findings = scan_text_for_secrets(input);
        assert!(!findings.is_empty(), "should detect GitHub token");
        assert!(
            findings.iter().any(|f| f.pattern_name == "github_token"),
            "should detect github_token pattern"
        );
    }

    #[test]
    fn test_parity_slack_token() {
        init_default_patterns();
        // Fixture from client test: slackToken
        let input = "SLACK_TOKEN=xoxb-1234567890-1234567890123-12345678901234567890123456";
        let findings = scan_text_for_secrets(input);
        assert!(!findings.is_empty(), "should detect Slack token");
        assert!(
            findings.iter().any(|f| f.pattern_name == "slack_token"),
            "should detect slack_token pattern"
        );
    }

    #[test]
    fn test_parity_jwt() {
        init_default_patterns();
        // Fixture from client test: jwt
        let input = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
        let findings = scan_text_for_secrets(input);
        // JWT is detected either by JWT pattern or Bearer Token pattern
        assert!(!findings.is_empty(), "should detect JWT or Bearer token");
        let has_jwt_or_bearer = findings
            .iter()
            .any(|f| f.pattern_name == "jwt" || f.pattern_name == "bearer_token");
        assert!(
            has_jwt_or_bearer,
            "should detect jwt or bearer_token pattern"
        );
    }

    #[test]
    fn test_parity_env_var_secret() {
        init_default_patterns();
        // Fixture from client test: envVarSecret
        let input = "export openai_api_key=sk-proj-AbCdEf1234567890";
        let findings = scan_text_for_secrets(input);
        assert!(
            !findings.is_empty(),
            "should detect environment variable secret"
        );
        assert!(
            findings.iter().any(|f| f.pattern_name == "env_var_secret"),
            "should detect env_var_secret pattern"
        );
    }

    #[test]
    fn test_parity_json_secret() {
        init_default_patterns();
        // Fixture from client test: jsonSecret
        let input = r#"{"password": "s3cr3tP@ssw0rd!", "api_key": "abc123def456ghi789jkl"}"#;
        let findings = scan_text_for_secrets(input);
        assert!(!findings.is_empty(), "should detect JSON secret field");
        assert!(
            findings
                .iter()
                .any(|f| f.pattern_name == "json_secret_field"),
            "should detect json_secret_field pattern"
        );
    }

    #[test]
    fn test_parity_multiple_secrets() {
        init_default_patterns();
        // Fixture from client test: multipleSecrets
        let input = r#"
    My API keys:
    Anthropic: sk-ant-api03-TEST1234567890ABCDEFGHIJ1234567890ABCD
    GitHub: ghp_1234567890abcdef1234567890abcd123456
    AWS: AKIA1234567890ABCDEF
  "#;
        let findings = scan_text_for_secrets(input);
        // Should detect at least 3 secrets (Anthropic, GitHub, AWS)
        assert!(
            findings.len() >= 3,
            "should detect at least 3 secrets, got {}",
            findings.len()
        );
        let pattern_names: Vec<_> = findings.iter().map(|f| f.pattern_name).collect();
        assert!(
            pattern_names.contains(&"anthropic_api_key"),
            "should detect anthropic_api_key"
        );
        assert!(
            pattern_names.contains(&"github_token"),
            "should detect github_token"
        );
        assert!(
            pattern_names.contains(&"aws_access_key"),
            "should detect aws_access_key"
        );
    }

    #[test]
    fn test_parity_clean_text() {
        init_default_patterns();
        // Clean text should have no findings
        let input = "This is a normal message with no secrets. Just plain text.";
        let findings = scan_text_for_secrets(input);
        assert!(findings.is_empty(), "clean text should have no findings");
    }

    #[test]
    fn test_parity_match_positions() {
        init_default_patterns();
        // Verify match positions are correct
        let input = "My key is sk-ant-test1234567890ABCDEFGH end";
        let findings = scan_text_for_secrets(input);
        assert!(!findings.is_empty(), "should detect secret");
        let finding = &findings[0];
        assert!(
            finding.match_start < input.len(),
            "match start should be within text"
        );
        assert!(
            finding.match_start + finding.match_len <= input.len(),
            "match end should be within text"
        );
    }

    #[test]
    fn test_parity_redaction_matches_scanning() {
        init_default_patterns();
        // Verify that redaction and scanning use the same patterns
        let input = "My key sk-ant-api03-TEST1234567890ABCDEFGHIJ1234567890ABCD is here";
        let findings = scan_text_for_secrets(input);
        let redacted = redact_text(input);

        // If scanning finds something, redaction should remove it
        if !findings.is_empty() {
            assert!(
                redacted.contains("[REDACTED]"),
                "redaction should replace detected secrets"
            );
            assert!(
                !redacted.contains("sk-ant-"),
                "redacted text should not contain raw secret"
            );
        }
    }
}
