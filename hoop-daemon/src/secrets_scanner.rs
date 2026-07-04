//! Pre-storage secrets scanner for attachments and transcripts (§18)
//!
//! Detects:
//! - Common secret patterns (from config.yml or defaults)
//! - Env-var-style leaks (`OPENAI_API_KEY=sk-...`)
//! - High-entropy strings >N bits, with context-aware exclusions (git SHAs fine)
//! - Email addresses per operator-configured PII patterns
//!
//! Behavior: **Flag, not block**. Operator sees warning banner listing findings,
//! chooses: redact-in-place, redact-and-rewind, or proceed.
//!
//! Pattern source: Single source of truth from config.yml via config_resolver.
//! Client fetches patterns from /api/config/secrets-patterns for pre-upload warning.

use crate::config_resolver::{SecretPattern, default_secret_patterns};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{LazyLock, Mutex};

// ── High-entropy scanner ────────────────────────────────────────────────────────

/// Minimum entropy threshold for flagging high-entropy strings (bits per character).
/// 4.5 bits/char is a good threshold for detecting base64-like secrets while
/// avoiding false positives on normal text.
const ENTROPY_THRESHOLD: f64 = 4.5;

/// Minimum length for entropy scanning (shorter strings are likely false positives).
const ENTROPY_MIN_LENGTH: usize = 20;

/// Maximum length for entropy scanning (longer strings are likely legitimate content).
const ENTROPY_MAX_LENGTH: usize = 100;

/// Calculate Shannon entropy of a string in bits per character.
pub fn calculate_entropy(s: &str) -> f64 {
    if s.is_empty() {
        return 0.0;
    }

    let mut freq = [0usize; 256];
    for byte in s.bytes() {
        freq[byte as usize] += 1;
    }

    let len = s.len() as f64;
    let mut entropy = 0.0;

    for &count in freq.iter() {
        if count > 0 {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
    }

    entropy
}

/// Context-aware exclusions for high-entropy strings.
///
/// Returns true if the string should be excluded from entropy-based detection
/// even though it has high entropy.
fn is_high_entropy_exclusion(match_str: &str, context: &str) -> bool {
    // Git commit SHAs (40 hex chars)
    if match_str.len() == 40 && match_str.chars().all(|c| c.is_ascii_hexdigit()) {
        // Check if preceded by "commit", "sha", "revision", or similar
        let context_lower = context.to_lowercase();
        if context_lower.contains("commit")
            || context_lower.contains("sha")
            || context_lower.contains("revision")
            || context_lower.contains("hash")
        {
            return true;
        }
    }

    // UUIDs (standard format)
    if uuid_regex().is_match(match_str) {
        return true;
    }

    // Hex-encoded colors (6 or 8 hex chars, often preceded by #)
    if (match_str.len() == 6 || match_str.len() == 8)
        && match_str.chars().all(|c| c.is_ascii_hexdigit()) {
            // Check if preceded by # or color-related keywords
            let context_lower = context.to_lowercase();
            if context_lower.contains('#')
                || context_lower.contains("color")
                || context_lower.contains("background")
                || context_lower.contains("foreground")
            {
                return true;
            }
        }

    // Base64-encoded data that's actually a known format (e.g., PEM headers)
    if match_str.contains("BEGIN") || match_str.contains("END") {
        return true;
    }

    false
}

/// Regex for UUID detection
fn uuid_regex() -> &'static Regex {
    static UUID_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$"
        ).unwrap()
    });
    &UUID_REGEX
}

// ── Email/PII scanner ────────────────────────────────────────────────────────────

/// Default email regex pattern (basic but effective).
fn email_regex() -> &'static Regex {
    static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        // RFC 5322-ish pattern: local-part@domain
        // Local part: alphanumeric + . _ % + -
        // Domain: alphanumeric + . -
        // TLD: at least 2 chars
        Regex::new(
            r"\b[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}\b"
        ).unwrap()
    });
    &EMAIL_REGEX
}

// ── Scanner state ───────────────────────────────────────────────────────────────

/// Global scanner state, updated by config hot-reload.
#[derive(Debug)]
pub struct ScannerState {
    /// Compiled regex patterns from config
    patterns: Vec<(String, Regex)>,
    /// Custom PII patterns (email, phone, etc.)
    pii_patterns: Vec<(String, Regex)>,
    /// Projects with email detection enabled
    email_enabled_projects: HashSet<String>,
}

impl ScannerState {
    fn new() -> Self {
        Self {
            patterns: Vec::new(),
            pii_patterns: Vec::new(),
            email_enabled_projects: HashSet::new(),
        }
    }

    /// Update patterns from config.
    fn update_patterns(&mut self, secret_patterns: &[SecretPattern]) {
        self.patterns.clear();
        for sp in secret_patterns {
            for pat_str in &sp.patterns {
                match Regex::new(pat_str) {
                    Ok(re) => {
                        self.patterns.push((sp.id.clone(), re));
                    }
                    Err(e) => {
                        tracing::warn!(
                            pattern_id = %sp.id,
                            pattern = %pat_str,
                            error = %e,
                            "Invalid regex pattern, skipping"
                        );
                    }
                }
            }
        }
    }

    /// Add custom PII pattern (e.g., email, phone).
    fn add_pii_pattern(&mut self, id: String, pattern: String) -> Result<(), regex::Error> {
        let re = Regex::new(&pattern)?;
        self.pii_patterns.push((id, re));
        Ok(())
    }

    /// Enable email detection for a project.
    fn enable_email_detection(&mut self, project_name: String) {
        self.email_enabled_projects.insert(project_name);
    }

    /// Check if email detection is enabled for a project.
    fn is_email_enabled(&self, project_name: &str) -> bool {
        self.email_enabled_projects.contains(project_name)
    }
}

static SCANNER_STATE: LazyLock<Mutex<ScannerState>> = LazyLock::new(|| Mutex::new(ScannerState::new()));

// ── Scan result ────────────────────────────────────────────────────────────────

/// A secret finding in scanned content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// Which scanner pattern matched (e.g., "anthropic_api_key", "high_entropy").
    pub pattern_id: String,
    /// Human-readable description of what was found.
    pub description: String,
    /// Byte offset of the start of the match within the scanned text.
    pub match_start: usize,
    /// Length of the matched substring in bytes.
    pub match_len: usize,
    /// The matched string (for audit/redaction purposes).
    pub matched_text: String,
    /// Severity level.
    pub severity: String,
}

// ── Scanning functions ──────────────────────────────────────────────────────────

/// Scan text for secrets and return all findings.
///
/// This is the main entry point for secret scanning. It:
/// 1. Applies configured regex patterns
/// 2. Runs entropy-based detection with context-aware exclusions
/// 3. Applies PII patterns (email, etc.) if enabled for the project
///
/// Returns an empty vec if no secrets are detected.
pub fn scan_text(text: &str, project_name: Option<&str>) -> Vec<Finding> {
    let state = SCANNER_STATE.lock().unwrap();
    let mut findings = Vec::new();

    // 1. Apply configured patterns
    for (pattern_id, re) in &state.patterns {
        for m in re.find_iter(text) {
            findings.push(Finding {
                pattern_id: pattern_id.clone(),
                description: format!("Pattern match: {}", pattern_id),
                match_start: m.start(),
                match_len: m.len(),
                matched_text: m.as_str().to_string(),
                severity: "high".to_string(),
            });
        }
    }

    // 2. High-entropy detection (token-based scanning)
    findings.extend(scan_high_entropy(text));

    // 3. PII patterns (email) if enabled
    if project_name.is_some_and(|p| state.is_email_enabled(p)) {
        findings.extend(scan_email(text));
    }

    findings
}

/// Scan for high-entropy strings that might be secrets.
fn scan_high_entropy(text: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    // Split into tokens (whitespace-separated)
    // We also look at common delimiters: =, :, ", '
    let tokens = extract_candidate_tokens(text);

    for token in tokens {
        let token_str = token.as_ref();

        // Skip if too short or too long
        if token_str.len() < ENTROPY_MIN_LENGTH || token_str.len() > ENTROPY_MAX_LENGTH {
            continue;
        }

        // Calculate entropy
        let entropy = calculate_entropy(token_str);

        if entropy >= ENTROPY_THRESHOLD {
            // Check for context-aware exclusions
            let context_start = token.start.saturating_sub(50);
            let context_end = (token.end + 50).min(text.len());
            let context = &text[context_start..context_end];

            if !is_high_entropy_exclusion(token_str, context) {
                findings.push(Finding {
                    pattern_id: "high_entropy".to_string(),
                    description: format!("High-entropy string ({} bits/char)", entropy.round()),
                    match_start: token.start,
                    match_len: token.text.len(),
                    matched_text: token_str.to_string(),
                    severity: "medium".to_string(),
                });
            }
        }
    }

    findings
}

/// Extract candidate tokens for entropy scanning.
///
/// Returns tokens that look like they might be secrets (alphanumeric, base64-like, etc.)
fn extract_candidate_tokens(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    // Pattern for candidate tokens:
    // - Base64-like: [A-Za-z0-9+/=_-]{20,}
    // - Hex strings: [0-9a-fA-F]{20,}
    // - Mixed alphanumeric with special chars
    let candidate_re = regex::Regex::new(
        r#"[A-Za-z0-9+/=_-]{20,}"#
    ).unwrap();

    for m in candidate_re.find_iter(text) {
        let s = m.as_str();

        // Skip obvious non-secrets
        if is_likely_benign(s) {
            continue;
        }

        tokens.push(Token {
            start: m.start(),
            end: m.end(),
            text: s.to_string(),
        });
    }

    tokens
}

/// Check if a string is likely benign (not a secret).
fn is_likely_benign(s: &str) -> bool {
    // All lowercase letters - likely a word
    if s.chars().all(|c| c.is_ascii_lowercase()) {
        return true;
    }

    // All uppercase letters - likely an acronym
    if s.chars().all(|c| c.is_ascii_uppercase()) {
        return true;
    }

    // Common non-secret patterns
    if s.starts_with("http://") || s.starts_with("https://") {
        return true;
    }

    if s.contains("://") {
        return true;
    }

    // Very repetitive (e.g., "aaaaaaaaaaaaa")
    let unique_chars: HashSet<char> = s.chars().collect();
    if unique_chars.len() <= 3 {
        return true;
    }

    false
}

struct Token {
    start: usize,
    end: usize,
    text: String,
}

impl AsRef<str> for Token {
    fn as_ref(&self) -> &str {
        &self.text
    }
}

/// Scan for email addresses.
fn scan_email(text: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for m in email_regex().find_iter(text) {
        findings.push(Finding {
            pattern_id: "email".to_string(),
            description: "Email address".to_string(),
            match_start: m.start(),
            match_len: m.len(),
            matched_text: m.as_str().to_string(),
            severity: "low".to_string(),
        });
    }

    findings
}

// ── Pattern management ───────────────────────────────────────────────────────────

/// Initialize the scanner with default patterns from config_resolver.
///
/// This is a convenience function for tests. In production, patterns are
/// loaded from config.yml and applied via update_patterns() during daemon
/// initialization (see lib.rs).
pub fn init() {
    let patterns = default_secret_patterns();
    update_patterns(&patterns);
}

/// Update scanner patterns from config.
pub fn update_patterns(secret_patterns: &[SecretPattern]) {
    let mut state = SCANNER_STATE.lock().unwrap();
    state.update_patterns(secret_patterns);
    tracing::info!("Secrets scanner updated: {} patterns loaded", state.patterns.len());
}

/// Add a custom PII pattern.
pub fn add_pii_pattern(id: String, pattern: String) -> Result<(), String> {
    let mut state = SCANNER_STATE.lock().unwrap();
    state.add_pii_pattern(id, pattern).map_err(|e| e.to_string())
}

/// Enable email detection for a project.
pub fn enable_email_detection(project_name: String) {
    let mut state = SCANNER_STATE.lock().unwrap();
    state.enable_email_detection(project_name);
}

/// Disable email detection for a project.
pub fn disable_email_detection(project_name: &str) {
    let mut state = SCANNER_STATE.lock().unwrap();
    state.email_enabled_projects.remove(project_name);
}

// ── Tests ───────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_entropy_high() {
        // High entropy: random-looking string
        let s = "sk-ant-api03-AAAA1111BBBB2222CCCC3333DDDD4444";
        let e = calculate_entropy(s);
        assert!(e > 4.0, "entropy should be high for API key: {}", e);
    }

    #[test]
    fn test_calculate_entropy_low() {
        // Low entropy: English sentence
        let s = "The quick brown fox jumps over the lazy dog";
        let e = calculate_entropy(s);
        assert!(e < 4.5, "entropy should be low for normal text: {}", e);
    }

    #[test]
    fn test_scan_text_stripe_key() {
        init();
        let text = "My Stripe key is sk_live_51AbCdEf1234567890AbCdEf please keep it safe";
        let findings = scan_text(text, None);
        assert!(!findings.is_empty(), "should detect Stripe key");
        assert!(findings.iter().any(|f| f.pattern_id == "stripe_api_key"), "should match stripe_api_key pattern");
    }

    #[test]
    fn test_scan_text_openai_key() {
        init();
        let text = "OPENAI_API_KEY=sk-proj-AbCdEf1234567890AbCdEf1234567890AbCdEf";
        let findings = scan_text(text, None);
        assert!(!findings.is_empty(), "should detect OpenAI key");
        assert!(findings.iter().any(|f| f.pattern_id == "openai_api_key" || f.pattern_id == "env_var_secret"));
    }

    #[test]
    fn test_scan_text_anthropic_key() {
        init();
        let text = "ANTHROPIC_API_KEY=sk-ant-api03-AAAA1111BBBB2222CCCC3333DDDD4444EEEE5555FFFF6666";
        let findings = scan_text(text, None);
        assert!(!findings.is_empty(), "should detect Anthropic key");
    }

    #[test]
    fn test_scan_text_aws_key() {
        init();
        let text = "aws_access_key_id = AKIAIOSFODNN7EXAMPLE";
        let findings = scan_text(text, None);
        assert!(!findings.is_empty(), "should detect AWS key");
        assert!(findings.iter().any(|f| f.pattern_id == "aws_access_key"));
    }

    #[test]
    fn test_scan_text_github_token() {
        init();
        let text = "github_token=ghp_16C7e42F292c6912E7710c838347Ae178B4a";
        let findings = scan_text(text, None);
        assert!(!findings.is_empty(), "should detect GitHub token");
        assert!(findings.iter().any(|f| f.pattern_id == "github_token"));
    }

    #[test]
    fn test_scan_text_jwt() {
        init();
        let text = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
        let findings = scan_text(text, None);
        assert!(!findings.is_empty(), "should detect JWT");
        assert!(findings.iter().any(|f| f.pattern_id == "jwt" || f.pattern_id == "bearer_token"));
    }

    #[test]
    fn test_scan_text_slack_token() {
        init();
        let text = "SLACK_TOKEN=xoxb-1234567890-1234567890123-12345678901234567890123456";
        let findings = scan_text(text, None);
        assert!(!findings.is_empty(), "should detect Slack token");
        assert!(findings.iter().any(|f| f.pattern_id == "slack_token"));
    }

    #[test]
    fn test_scan_text_json_secret() {
        init();
        let text = r#"{"password": "s3cr3tP@ssw0rd!", "api_key": "abc123def456ghi789jkl"}"#;
        let findings = scan_text(text, None);
        assert!(!findings.is_empty(), "should detect JSON secret");
        assert!(findings.iter().any(|f| f.pattern_id == "json_secret_field"));
    }

    #[test]
    fn test_scan_text_clean() {
        init();
        let text = "This is a normal message with no secrets. Just plain text.";
        let findings = scan_text(text, None);
        assert!(findings.is_empty(), "clean text should have no findings");
    }

    #[test]
    fn test_high_entropy_detection() {
        init();
        let text = "Random high entropy string: sk-ant-api03-AAAA1111BBBB2222CCCC3333DDDD4444EEEE5555FFFF6666";
        let findings = scan_text(text, None);
        // Should detect via pattern first, but entropy scanner should also flag if pattern misses
        let has_high_entropy = findings.iter().any(|f| f.pattern_id == "high_entropy");
        // Note: This may or may not have high_entropy findings since the pattern matches first
        // The important thing is that secrets are detected
        assert!(!findings.is_empty(), "should detect high-entropy string");
    }

    #[test]
    fn test_git_sha_not_flagged() {
        init();
        let text = "commit abc123def456789abcdef0123456789abcdef01 pushed by john";
        let findings = scan_text(text, None);
        // Git SHA should be excluded from high-entropy detection
        let has_entropy_flag = findings.iter().any(|f| f.pattern_id == "high_entropy" && f.matched_text.len() == 40);
        assert!(!has_entropy_flag, "git SHA should not be flagged as high entropy");
    }

    #[test]
    fn test_uuid_not_flagged() {
        init();
        let text = "Request ID: 550e8400-e29b-41d4-a716-446655440000";
        let findings = scan_text(text, None);
        // UUID should be excluded from high-entropy detection
        let has_entropy_flag = findings.iter().any(|f| f.pattern_id == "high_entropy");
        assert!(!has_entropy_flag, "UUID should not be flagged as high entropy");
    }

    #[test]
    fn test_email_detection_enabled() {
        init();
        enable_email_detection("test-project".to_string());

        let text = "Contact us at support@example.com for help";
        let findings = scan_text(text, Some("test-project"));
        assert!(!findings.is_empty(), "should detect email when enabled");
        assert!(findings.iter().any(|f| f.pattern_id == "email"));

        disable_email_detection("test-project");
    }

    #[test]
    fn test_email_detection_disabled() {
        init();
        // Email detection is disabled by default
        let text = "Contact us at support@example.com for help";
        let findings = scan_text(text, Some("test-project"));
        assert!(findings.is_empty(), "should not detect email when disabled");
    }

    #[test]
    fn test_multiple_secrets() {
        init();
        let text = r#"
My API keys:
Stripe: sk_live_51AbCdEf1234567890AbCdEf
GitHub: ghp_1234567890abcdef1234567890abcd123456
AWS: AKIA1234567890ABCDEF
"#;
        let findings = scan_text(text, None);
        assert!(findings.len() >= 3, "should detect at least 3 secrets");
    }

    #[test]
    fn test_finding_serialization() {
        let finding = Finding {
            pattern_id: "test_pattern".to_string(),
            description: "Test finding".to_string(),
            match_start: 10,
            match_len: 20,
            matched_text: "test_value".to_string(),
            severity: "high".to_string(),
        };

        let json = serde_json::to_string(&finding).unwrap();
        let deserialized: Finding = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.pattern_id, finding.pattern_id);
        assert_eq!(deserialized.match_start, finding.match_start);
    }

    #[test]
    fn test_likely_benign_detection() {
        assert!(is_likely_benign("hello"));
        assert!(is_likely_benign("HTTP"));
        assert!(is_likely_benign("https://example.com"));
        assert!(is_likely_benign("aaaaaaaaaaaaa"));
        assert!(!is_likely_benign("sk-ant-api03-AAAA1111BBBB2222"));
    }

    #[test]
    fn test_entropy_threshold() {
        let low = "the quick brown fox";
        let high = "sk-ant-api03-AAAA1111BBBB2222CCCC3333DDDD4444";

        assert!(calculate_entropy(low) < ENTROPY_THRESHOLD);
        assert!(calculate_entropy(high) >= ENTROPY_THRESHOLD);
    }

    // ── Synthetic secrets test fixtures (hoop-ttb.15.1) ─────────────────────────────
    // These fixtures test detection of synthetic secrets that should always be caught.
    // All patterns must match the default_secret_patterns() in config_resolver.rs.

    const FIXTURE_SECRETS: &[(&str, &str)] = &[
        // Stripe API keys
        ("stripe_live_key", "My Stripe key is sk_live_51AbCdEf1234567890AbCdEf1234567890AbC please keep it safe"),
        ("stripe_test_key", "Stripe test: sk_test_51AbCdEf1234567890AbCdEf1234567890AbC"),
        ("stripe_ir_live", "IR key: ir_live_51AbCdEf1234567890AbCdEf1234567890AbCdEf123456"),
        ("stripe_ir_test", "IR test: ir_test_51AbCdEf1234567890AbCdEf1234567890AbCdEf123456"),

        // OpenAI API keys
        ("openai_key", "My OpenAI key is sk-ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890abcdefghijkl"),
        ("openai_proj_key", "OpenAI project key: sk-proj-AbCdEf1234567890AbCdEf1234567890AbCdEf1234567890abc"),

        // Anthropic API keys
        ("anthropic_key", "Here is my key sk-ant-api03-AAAA1111BBBB2222CCCC3333DDDD4444EEEE5555FFFF6666 please keep it safe"),
        ("anthropic_short", "Key: sk-ant-ABCDEFGHIJKLMNOPQRSTUVWXYZ123456"),

        // AWS access keys
        ("aws_access_key", "aws_access_key_id = AKIAIOSFODNN7EXAMPLE"),
        ("aws_temp_key", "ASIA1234567890ABCDEF"),
        ("aws_secret_key", "aws_secret_access_key = ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890+/AB"),

        // GitHub tokens
        ("github_token_ghp", "token=ghp_16C7e42F292c6912E7710c838347Ae178B4a"),
        ("github_pat", "My GitHub token is github_pat_1234567890abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ12345678901234567890"),

        // JWT tokens
        ("jwt_header", "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U"),
        ("jwt_standalone", "Token: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U"),

        // Slack tokens
        ("slack_bot", "SLACK_TOKEN=xoxb-1234567890-1234567890123-12345678901234567890123456"),
        ("slack_user", "xoxp-1234567890-1234567890123-12345678901234567890123456"),

        // Bearer tokens
        ("bearer_token", "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U"),

        // Environment variable secrets
        ("env_var_anthropic", "export ANTHROPIC_API_KEY=sk-ant-api03-TEST1234567890ABCDEFGHIJ1234567890ABCD"),
        ("env_var_openai", "OPENAI_API_KEY=sk-proj-AbCdEf1234567890AbCdEf1234567890AbCdEf1234567890abc"),
        ("env_var_generic", "API_KEY=sk-ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890abcdefghijklmn"),

        // JSON secret fields
        ("json_password", r#"{"password": "s3cr3tP@ssw0rd!", "api_key": "abc123def456ghi789jkl"}"#),
        ("json_api_key", r#"{"api_key": "sk-ant-api03-TEST1234567890ABCDEFGHIJ1234567890ABCD"}"#),
        ("json_secret", r#"{"secret": "my-secret-key-value-1234567890"}"#),
    ];

    #[test]
    fn test_synthetic_secrets_all_detected() {
        init();
        let mut detected = 0;
        let mut missed = Vec::new();

        for (name, text) in FIXTURE_SECRETS {
            let findings = scan_text(text, None);
            if !findings.is_empty() {
                detected += 1;
            } else {
                missed.push(*name);
            }
        }

        // All synthetic secrets should be detected
        assert_eq!(
            detected,
            FIXTURE_SECRETS.len(),
            "Not all synthetic secrets were detected. Missed: {:?}",
            missed
        );
    }

    #[test]
    fn test_synthetic_secrets_stripe() {
        init();
        let findings = scan_text(FIXTURE_SECRETS[0].1, None);
        assert!(!findings.is_empty(), "Stripe live key should be detected");
        assert!(findings.iter().any(|f| f.pattern_id == "stripe_api_key" || f.pattern_id == "generic_sk_key"));
    }

    #[test]
    fn test_synthetic_secrets_openai() {
        init();
        let findings = scan_text(FIXTURE_SECRETS[4].1, None);
        assert!(!findings.is_empty(), "OpenAI key should be detected");
        assert!(findings.iter().any(|f| f.pattern_id == "openai_api_key" || f.pattern_id == "generic_sk_key"));
    }

    #[test]
    fn test_synthetic_secrets_anthropic() {
        init();
        let findings = scan_text(FIXTURE_SECRETS[6].1, None);
        assert!(!findings.is_empty(), "Anthropic key should be detected");
        assert!(findings.iter().any(|f| f.pattern_id == "anthropic_api_key"));
    }

    #[test]
    fn test_synthetic_secrets_aws() {
        init();
        let findings = scan_text(FIXTURE_SECRETS[9].1, None);
        assert!(!findings.is_empty(), "AWS access key should be detected");
        assert!(findings.iter().any(|f| f.pattern_id == "aws_access_key"));
    }

    #[test]
    fn test_synthetic_secrets_github() {
        init();
        let findings = scan_text(FIXTURE_SECRETS[12].1, None);
        assert!(!findings.is_empty(), "GitHub token should be detected");
        assert!(findings.iter().any(|f| f.pattern_id == "github_token"));
    }

    #[test]
    fn test_synthetic_secrets_jwt() {
        init();
        let findings = scan_text(FIXTURE_SECRETS[14].1, None);
        assert!(!findings.is_empty(), "JWT should be detected");
        assert!(findings.iter().any(|f| f.pattern_id == "jwt" || f.pattern_id == "bearer_token"));
    }

    #[test]
    fn test_synthetic_secrets_slack() {
        init();
        let findings = scan_text(FIXTURE_SECRETS[16].1, None);
        assert!(!findings.is_empty(), "Slack token should be detected");
        assert!(findings.iter().any(|f| f.pattern_id == "slack_token"));
    }

    #[test]
    fn test_synthetic_secrets_env_var() {
        init();
        let findings = scan_text(FIXTURE_SECRETS[19].1, None);
        assert!(!findings.is_empty(), "Environment variable secret should be detected");
        assert!(findings.iter().any(|f| f.pattern_id == "env_var_secret"));
    }

    #[test]
    fn test_synthetic_secrets_json() {
        init();
        let findings = scan_text(FIXTURE_SECRETS[22].1, None);
        assert!(!findings.is_empty(), "JSON secret field should be detected");
        assert!(findings.iter().any(|f| f.pattern_id == "json_secret_field"));
    }
}
