//! Deterministic config precedence resolver with attribution.
//!
//! Precedence: CLI flags > env vars > config.yml > compiled defaults.
//!
//! Every resolved key carries attribution — a human-readable string naming
//! which layer won (e.g. `"cli flag --addr"`, `"env HOOP_BIND_ADDR"`,
//! `"config.yml: server.bind_addr"`, `"compiled default"`).
//!
//! Plan reference: §17.2

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use tracing::{info, warn};

use crate::stuck_detector::StuckDetectorConfigMap;

// ---------------------------------------------------------------------------
// Secret pattern definition (§18)
// ---------------------------------------------------------------------------

/// A named secret pattern for the secrets scanner.
///
/// Each pattern has a human-readable name, a severity level, and one or more
/// regular expressions that match secrets of this type.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SecretPattern {
    /// Human-readable name for this pattern type (e.g., "AWS Access Key").
    pub name: String,
    /// Severity level: high, medium, or low.
    pub severity: String,
    /// Regular expressions that match this secret type.
    /// Multiple patterns are OR'd together.
    pub patterns: Vec<String>,
}

impl SecretPattern {
    /// Validate that all patterns compile to valid regexes.
    pub fn validate(&self) -> Result<(), String> {
        for pat in &self.patterns {
            Regex::new(pat)
                .map_err(|e| format!("invalid regex '{}': {}", pat, e))?;
        }
        Ok(())
    }

    /// Check if severity is valid.
    pub fn is_valid_severity(&self) -> bool {
        matches!(self.severity.as_str(), "high" | "medium" | "low")
    }

    /// Flatten a list of SecretPattern objects into a list of regex strings.
    ///
    /// This is used to convert the structured SecretPattern format into
    /// the flat list of regex strings expected by the redaction module.
    pub fn flatten_patterns(patterns: &[SecretPattern]) -> Vec<String> {
        patterns
            .iter()
            .flat_map(|sp| sp.patterns.iter().cloned())
            .collect()
    }

    /// Convert a list of SecretPattern objects into named pattern tuples.
    ///
    /// This is used to update both redaction and scanning patterns via
    /// `redaction::update_patterns_with_names()`.
    ///
    /// Returns a list of `(name, pattern_string)` tuples.
    pub fn to_named_patterns(patterns: &[SecretPattern]) -> Vec<(&str, String)> {
        patterns
            .iter()
            .flat_map(|sp| {
                let name: &str = &sp.name;
                sp.patterns.iter().map(move |pat| (name, pat.clone()))
            })
            .collect()
    }
}

/// Default secret patterns when none are configured.
///
/// These are the built-in patterns that ship with HOOP.
fn default_secret_patterns() -> Vec<SecretPattern> {
    vec![
        SecretPattern {
            name: "Anthropic API Key".to_string(),
            severity: "high".to_string(),
            patterns: vec![
                r"sk-ant-[a-zA-Z0-9_-]{20,}".to_string(),
            ],
        },
        SecretPattern {
            name: "Generic API Key".to_string(),
            severity: "high".to_string(),
            patterns: vec![
                r"\bsk-[a-zA-Z0-9]{20,}\b".to_string(),
            ],
        },
        SecretPattern {
            name: "AWS Access Key".to_string(),
            severity: "high".to_string(),
            patterns: vec![
                r"\bAKIA[A-Z0-9]{16}\b".to_string(),
            ],
        },
        SecretPattern {
            name: "GitHub Token".to_string(),
            severity: "high".to_string(),
            patterns: vec![
                r"\bghp_[a-zA-Z0-9]{36}\b".to_string(),
                r"\bghs_[a-zA-Z0-9]{36}\b".to_string(),
                r"\bghu_[a-zA-Z0-9]{36}\b".to_string(),
                r"\bgithub_pat_[a-zA-Z0-9_]{82}\b".to_string(),
            ],
        },
        SecretPattern {
            name: "Slack Token".to_string(),
            severity: "high".to_string(),
            patterns: vec![
                r"\bxoxb-[0-9A-Za-z-]{24,}\b".to_string(),
                r"\bxoxp-[0-9A-Za-z-]{24,}\b".to_string(),
            ],
        },
        SecretPattern {
            name: "JWT".to_string(),
            severity: "high".to_string(),
            patterns: vec![
                r"\bey[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b".to_string(),
            ],
        },
        SecretPattern {
            name: "Bearer Token".to_string(),
            severity: "high".to_string(),
            patterns: vec![
                r"(?i)bearer\s+[A-Za-z0-9._\-+/]{20,}".to_string(),
            ],
        },
        SecretPattern {
            name: "Environment Variable Secret".to_string(),
            severity: "high".to_string(),
            patterns: vec![
                r#"(?i)(?:api[_-]?key|secret[_-]?key|access[_-]?token|auth[_-]?token|private[_-]?key|client[_-]?secret|anthropic[_-]?api[_-]?key|openai[_-]?api[_-]?key|github[_-]?token)\s*[:=]\s*["']?([A-Za-z0-9+/_.~\-]{16,})["']?"#.to_string(),
            ],
        },
        SecretPattern {
            name: "JSON Secret Field".to_string(),
            severity: "high".to_string(),
            patterns: vec![
                r#"(?i)"(?:password|passwd|secret|token|api_key|apikey|access_token|auth_token|private_key|client_secret)"\s*:\s*"([^"]{8,})""#.to_string(),
            ],
        },
    ]
}

// ---------------------------------------------------------------------------
// Attribution types
// ---------------------------------------------------------------------------

/// Which layer provided a config value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigSource {
    CliFlag,
    EnvVar,
    ConfigYml,
    Default,
}

impl std::fmt::Display for ConfigSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigSource::CliFlag => write!(f, "cli flag"),
            ConfigSource::EnvVar => write!(f, "env var"),
            ConfigSource::ConfigYml => write!(f, "config.yml"),
            ConfigSource::Default => write!(f, "compiled default"),
        }
    }
}

/// A resolved config value with its attribution.
#[derive(Debug, Clone, Serialize)]
pub struct Resolved<T: Clone + Serialize> {
    pub value: T,
    pub source: ConfigSource,
    #[serde(rename = "resolved_from")]
    pub attribution: String,
    /// Whether this key requires daemon restart to take effect (§17.4)
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub restart_required: bool,
}

impl<T: Clone + Serialize> Resolved<T> {
    pub fn new(value: T, source: ConfigSource, attribution: impl Into<String>) -> Self {
        Self {
            value,
            source,
            attribution: attribution.into(),
            restart_required: false,
        }
    }

    /// Mark this config key as requiring daemon restart to take effect (§17.4)
    pub fn with_restart_required(mut self) -> Self {
        self.restart_required = true;
        self
    }
}

// ---------------------------------------------------------------------------
// Config error type for schema validation (§17.5)
// ---------------------------------------------------------------------------

/// Configuration parse error with structured details (§17.5)
///
/// Used by the config watcher to report validation failures with
/// enough detail for the UI to display helpful error messages.
#[derive(Debug, Clone)]
pub struct ConfigError {
    /// Human-readable error message
    pub message: String,
    /// Line number where the error occurred (1-indexed)
    pub line: usize,
    /// Column number where the error occurred (1-indexed)
    pub col: usize,
    /// Dotted path to the offending field (e.g. "agent.adapter")
    pub field: Option<String>,
    /// What was expected (e.g. "string", "one of: claude, codex")
    pub expected: Option<String>,
    /// What was actually found (e.g. "integer", "unknown_adapter")
    pub got: Option<String>,
}

impl ConfigError {
    /// Create a semantic validation error with structured details.
    pub fn validation(
        message: String,
        field: Option<String>,
        expected: Option<String>,
        got: Option<String>,
    ) -> Self {
        Self {
            message,
            line: 0,
            col: 0,
            field,
            expected,
            got,
        }
    }

    /// Create an error from a YAML parse failure with enhanced details (§17.5).
    ///
    /// Extracts structured error information including field path, expected type,
    /// and actual value from serde_yaml error messages for display in UI banners.
    pub fn from_yaml(err: &serde_yaml::Error) -> Self {
        let msg = err.to_string();
        let (field, expected, got) = parse_serde_yaml_details(&msg);

        // Build enhanced message with structured details
        let enhanced_msg = if let (Some(f), Some(exp), Some(g)) = (&field, &expected, &got) {
            format!("{} (field: {}, expected: {}, got: {})", msg, f, exp, g)
        } else if let (Some(f), Some(exp)) = (&field, &expected) {
            format!("{} (field: {}, expected: {})", msg, f, exp)
        } else if let Some(f) = &field {
            format!("{} (field: {})", msg, f)
        } else {
            msg.clone()
        };

        Self {
            message: enhanced_msg,
            line: err.location().map(|l| line(&l)).unwrap_or(0),
            col: err.location().map(|l| column(&l)).unwrap_or(0),
            field,
            expected,
            got,
        }
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ConfigError {}

/// Extract structured details from serde_yaml error messages.
///
/// Parses common error patterns to provide field-specific error information
/// for hot-reload validation (§17.5). Returns (field, expected, got).
fn parse_serde_yaml_details(msg: &str) -> (Option<String>, Option<String>, Option<String>) {
    // Pattern: missing field `name` at line X column Y
    if let Some(rest) = msg.strip_prefix("missing field `") {
        let field_end = rest.find('`').unwrap_or(rest.len());
        let field_name = &rest[..field_end];
        return (
            Some(field_name.to_string()),
            Some("field present".to_string()),
            Some("missing".to_string()),
        );
    }

    // Pattern: unknown field `extra`, expected ...
    if let Some(rest) = msg.strip_prefix("unknown field `") {
        let field_end = rest.find('`').unwrap_or(rest.len());
        let field_name = &rest[..field_end];
        return (
            Some(field_name.to_string()),
            Some("known field".to_string()),
            Some("unknown field".to_string()),
        );
    }

    // Pattern: invalid type: string <timestamp>, expected u32 at line X column Y
    // Pattern: invalid type: string "abc", expected a number at ...
    if msg.contains("invalid type") {
        // Extract expected type
        let expected = if msg.contains("expected u8") || msg.contains("expected u16") || msg.contains("expected u32") || msg.contains("expected u64") || msg.contains("expected usize") {
            Some("integer".to_string())
        } else if msg.contains("expected i8") || msg.contains("expected i16") || msg.contains("expected i32") || msg.contains("expected i64") || msg.contains("expected isize") {
            Some("integer".to_string())
        } else if msg.contains("expected f32") || msg.contains("expected f64") {
            Some("number".to_string())
        } else if msg.contains("expected bool") {
            Some("boolean".to_string())
        } else if msg.contains("expected string") || msg.contains("expected str") {
            Some("string".to_string())
        } else if let Some(idx) = msg.find("expected ") {
            let rest = &msg[idx + 9..];
            let end_idx = rest.find(|c| c == ',' || c == ' ').unwrap_or(rest.len());
            Some(rest[..end_idx].to_string())
        } else {
            None
        };

        // Extract actual type
        let got = if msg.contains("string \"") {
            Some("string".to_string())
        } else if msg.contains("null") {
            Some("null".to_string())
        } else if msg.contains("boolean") {
            Some("boolean".to_string())
        } else if let Some(idx) = msg.find("invalid type: ") {
            let rest = &msg[idx + 13..];
            if let Some(end_idx) = rest.find(',') {
                let actual = rest[..end_idx].trim();
                // Map common YAML type descriptions
                let got_type = if actual.starts_with('"') && actual.ends_with('"') {
                    "string"
                } else if actual == "null" {
                    "null"
                } else if actual == "true" || actual == "false" {
                    "boolean"
                } else if actual.parse::<f64>().is_ok() {
                    "number"
                } else {
                    actual
                };
                Some(got_type.to_string())
            } else {
                None
            }
        } else {
            None
        };

        // Try to extract field name from context
        let field = if let Some(idx) = msg.find(" at line ") {
            let before = &msg[..idx];
            if let Some(colon_idx) = before.rfind(':') {
                let after_colon = &before[colon_idx + 1..].trim();
                if !after_colon.is_empty() && after_colon.len() < 50 {
                    Some(after_colon.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        return (field, expected, got);
    }

    // Pattern: data did not match any variant of untagged enum
    if msg.contains("data did not match any variant") {
        return (
            None,
            Some("valid enum value".to_string()),
            Some("invalid value".to_string()),
        );
    }

    (None, None, None)
}

/// Helper to extract line number from serde_yaml::Location
fn line(loc: &serde_yaml::Location) -> usize {
    loc.line()
}

/// Helper to extract column number from serde_yaml::Location
fn column(loc: &serde_yaml::Location) -> usize {
    loc.column()
}

// ---------------------------------------------------------------------------
// Schema validation for config values (§17.5)
// ---------------------------------------------------------------------------

/// Validate agent.adapter value
fn validate_agent_adapter(value: &str) -> Result<(), String> {
    const VALID_ADAPTERS: &[&str] = &["claude", "codex", "opencode", "gemini", "aider"];
    if !VALID_ADAPTERS.contains(&value) {
        return Err(format!(
            "invalid value: \"{}\"; expected one of: {}",
            value,
            VALID_ADAPTERS.join(", ")
        ));
    }
    Ok(())
}

/// Validate ui.theme value
fn validate_ui_theme(value: &str) -> Result<(), String> {
    const VALID_THEMES: &[&str] = &["auto", "light", "dark", "solarized-light", "solarized-dark"];
    if !VALID_THEMES.contains(&value) {
        return Err(format!(
            "invalid value: \"{}\"; expected one of: {}",
            value,
            VALID_THEMES.join(", ")
        ));
    }
    Ok(())
}

/// Validate ui.default_project_sort value
fn validate_ui_sort(value: &str) -> Result<(), String> {
    const VALID_SORTS: &[&str] = &["name", "last_activity", "cost_today", "worker_count"];
    if !VALID_SORTS.contains(&value) {
        return Err(format!(
            "invalid value: \"{}\"; expected one of: {}",
            value,
            VALID_SORTS.join(", ")
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// CLI overrides (passed from hoop-cli)
// ---------------------------------------------------------------------------

/// Overrides that can only come from CLI flags.
#[derive(Debug, Clone, Default)]
pub struct CliOverrides {
    pub bind_addr: Option<SocketAddr>,
    pub allow_br_mismatch: Option<bool>,
}

// ---------------------------------------------------------------------------
// Resolved config — all keys, fully attributed
// ---------------------------------------------------------------------------

/// The fully resolved daemon configuration with attribution per key.
#[derive(Debug, Clone, Serialize)]
pub struct ResolvedConfig {
    // Server
    pub bind_addr: Resolved<String>,
    pub allow_br_mismatch: Resolved<bool>,

    // Agent
    pub agent_adapter: Resolved<String>,
    pub agent_model: Resolved<String>,
    pub agent_anthropic_api_key: Resolved<Option<String>>,
    pub agent_zai_base_url: Resolved<Option<String>>,
    pub agent_zai_api_key: Resolved<Option<String>>,
    pub agent_rate_limit_rpm: Resolved<Option<u32>>,
    pub agent_cost_cap_usd: Resolved<Option<f64>>,

    // Projects
    pub projects_file: Resolved<String>,

    // UI
    pub ui_theme: Resolved<String>,
    pub ui_default_project_sort: Resolved<String>,
    pub ui_archive_after_days: Resolved<u32>,

    // Metrics
    pub metrics_enabled: Resolved<bool>,
    pub metrics_port: Resolved<u16>,

    // Voice
    pub voice_whisper_model_path: Resolved<Option<String>>,
    pub voice_hotkey: Resolved<String>,
    pub voice_max_recording_seconds: Resolved<u32>,

    // Agent extensions (§22)
    pub agent_extensions_skills: Resolved<Option<String>>,
    pub agent_extensions_scripts: Resolved<Option<String>>,
    pub agent_extensions_notes: Resolved<Option<String>>,
    pub agent_extensions_prompts: Resolved<Option<String>>,

    // Audit
    pub audit_retention_days: Resolved<u32>,
    pub audit_hash_chain: Resolved<bool>,

    // Reflection
    pub reflection_enabled: Resolved<bool>,
    pub reflection_detection_threshold: Resolved<f64>,
    pub reflection_auto_archive_after_days: Resolved<u32>,

    // Backup (§15.2, §17.3)
    pub backup_endpoint: Resolved<Option<String>>,
    pub backup_bucket: Resolved<Option<String>>,
    pub backup_prefix: Resolved<Option<String>>,
    pub backup_schedule: Resolved<String>,
    pub backup_retention_days: Resolved<u32>,
    pub backup_encryption: Resolved<bool>,

    // Pricing (§17.3)
    pub pricing_file: Resolved<String>,

    // Secrets scanner patterns (§18)
    pub secrets_patterns: Resolved<Vec<SecretPattern>>,

    // Stuck detector (§C1, hoop-ttb.3.25)
    pub stuck_detector: Resolved<Option<StuckDetectorConfigMap>>,

    // Role-based access control (RBAC) - two-role model
    pub roles: Resolved<crate::auth::RoleConfig>,
}

// ---------------------------------------------------------------------------
// Resolver
// ---------------------------------------------------------------------------

/// Resolve a single key using the four-layer precedence.
///
/// Returns `Resolved<T>` from the first non-None layer:
/// `cli` > `env_val` > `file_val` > `default`.
fn resolve_opt<T: Clone + Serialize>(
    cli: Option<T>,
    env_val: Option<T>,
    file_val: Option<T>,
    default: T,
    cli_label: &str,
    env_label: &str,
    file_label: &str,
) -> Resolved<T> {
    if let Some(v) = cli {
        return Resolved::new(v, ConfigSource::CliFlag, format!("cli flag {}", cli_label));
    }
    if let Some(v) = env_val {
        return Resolved::new(v, ConfigSource::EnvVar, format!("env {}", env_label));
    }
    if let Some(v) = file_val {
        return Resolved::new(
            v,
            ConfigSource::ConfigYml,
            format!("config.yml: {}", file_label),
        );
    }
    Resolved::new(default, ConfigSource::Default, "compiled default")
}

/// Like `resolve_opt` but for values where the default is None (optional keys).
fn resolve_opt_none<T: Clone + Serialize>(
    cli: Option<T>,
    env_val: Option<T>,
    file_val: Option<T>,
    cli_label: &str,
    env_label: &str,
    file_label: &str,
    key_name: &str,
) -> Resolved<Option<T>> {
    if let Some(v) = cli {
        return Resolved::new(
            Some(v),
            ConfigSource::CliFlag,
            format!("cli flag {}", cli_label),
        );
    }
    if let Some(v) = env_val {
        return Resolved::new(Some(v), ConfigSource::EnvVar, format!("env {}", env_label));
    }
    if let Some(v) = file_val {
        return Resolved::new(
            Some(v),
            ConfigSource::ConfigYml,
            format!("config.yml: {}", file_label),
        );
    }
    Resolved::new(
        None,
        ConfigSource::Default,
        format!("compiled default ({} not set)", key_name),
    )
}

/// Strict resolve with type validation for hot-reload (§17.5).
///
/// Like `resolve_opt` but validates that config.yml values have the correct type.
/// Returns `ConfigError` if a value exists but has the wrong type.
fn resolve_opt_strict<T: Clone + Serialize>(
    cli: Option<T>,
    env_val: Option<T>,
    yml_ref: Option<&serde_yaml::Value>,
    yml_path: &str,
    default: T,
    cli_label: &str,
    env_label: &str,
    file_label: &str,
    type_validator: fn(&serde_yaml::Value, &str) -> Result<Option<T>, ConfigError>,
) -> Result<Resolved<T>, ConfigError> {
    if let Some(v) = cli {
        return Ok(Resolved::new(
            v,
            ConfigSource::CliFlag,
            format!("cli flag {}", cli_label),
        ));
    }
    if let Some(v) = env_val {
        return Ok(Resolved::new(
            v,
            ConfigSource::EnvVar,
            format!("env {}", env_label),
        ));
    }

    // Validate config.yml value type
    let file_val = if let Some(yml) = yml_ref {
        type_validator(yml, yml_path)?
    } else {
        None
    };

    if let Some(v) = file_val {
        return Ok(Resolved::new(
            v,
            ConfigSource::ConfigYml,
            format!("config.yml: {}", file_label),
        ));
    }

    Ok(Resolved::new(
        default,
        ConfigSource::Default,
        "compiled default".to_string(),
    ))
}

/// Load config.yml as a raw YAML value. Returns None if the file doesn't exist.
fn load_config_yml() -> Option<serde_yaml::Value> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let config_path = home.join(".hoop").join("config.yml");

    if !config_path.exists() {
        return None;
    }

    match std::fs::read_to_string(&config_path) {
        Ok(contents) => match serde_yaml::from_str::<serde_yaml::Value>(&contents) {
            Ok(v) => Some(v),
            Err(e) => {
                warn!("Failed to parse config.yml: {}, using defaults", e);
                None
            }
        },
        Err(e) => {
            warn!("Failed to read config.yml: {}, using defaults", e);
            None
        }
    }
}

/// Helper to extract a string from a YAML value by dotted path.
fn yaml_get_str<'a>(root: &'a serde_yaml::Value, path: &str) -> Option<&'a str> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut node = root;
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            return node.get(part).and_then(|v| v.as_str());
        }
        node = node.get(*part)?;
    }
    None
}

/// Helper to extract an integer from a YAML value by dotted path.
fn yaml_get_u64(root: &serde_yaml::Value, path: &str) -> Option<u64> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut node = root;
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            return node.get(part).and_then(|v| v.as_u64());
        }
        node = node.get(*part)?;
    }
    None
}

/// Helper to extract a float from a YAML value by dotted path.
fn yaml_get_f64(root: &serde_yaml::Value, path: &str) -> Option<f64> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut node = root;
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            return node.get(part).and_then(|v| v.as_f64());
        }
        node = node.get(*part)?;
    }
    None
}

/// Helper to extract a boolean from a YAML value by dotted path.
fn yaml_get_bool(root: &serde_yaml::Value, path: &str) -> Option<bool> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut node = root;
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            return node.get(part).and_then(|v| v.as_bool());
        }
        node = node.get(*part)?;
    }
    None
}

/// Helper to extract secret patterns from a YAML value.
fn yaml_get_secret_patterns(root: &serde_yaml::Value) -> Option<Vec<SecretPattern>> {
    root.get("secrets_patterns")
        .and_then(|v| v.as_sequence())
        .and_then(|seq| {
            let mut patterns = Vec::new();
            for item in seq {
                if let Ok(pat) = serde_yaml::from_value::<SecretPattern>(item.clone()) {
                    patterns.push(pat);
                }
            }
            if patterns.is_empty() { None } else { Some(patterns) }
        })
}

/// Helper to extract role configuration from a YAML value.
///
/// Parses the `roles` section from config.yml:
/// ```yaml
/// roles:
///   viewers:
///     - "viewer@example.com"
///     - "read-only-machine"
///   drafters:
///     - "drafter@example.com"
///     - "admin-machine"
/// ```
fn yaml_get_role_config(root: &serde_yaml::Value) -> Option<crate::auth::RoleConfig> {
    root.get("roles").and_then(|v| {
        // Parse viewers list
        let viewers: Vec<String> = v
            .get("viewers")
            .and_then(|vv| vv.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        // Parse drafters list
        let drafters = v
            .get("drafters")
            .and_then(|dv| dv.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        // Only return Some if at least one role has members
        if viewers.is_empty() && drafters.is_empty() {
            None
        } else {
            Some(crate::auth::RoleConfig { viewers, drafters })
        }
    })
}

/// Helper to read an env var and parse it.
fn env_parse<T: std::str::FromStr>(var: &str) -> Option<T> {
    std::env::var(var).ok().and_then(|v| v.parse().ok())
}

// ---------------------------------------------------------------------------
// Strict type validation helpers for config hot-reload (§17.5)
// These helpers check that the YAML value has the correct type and return
// a ConfigError if it doesn't. Used by resolve_from_raw to reject invalid
// config edits rather than silently falling back to defaults.
// ---------------------------------------------------------------------------

/// Navigate to a nested YAML value by dotted path.
fn yaml_navigate<'a>(root: &'a serde_yaml::Value, path: &str) -> Option<&'a serde_yaml::Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut node = root;
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            return node.get(part);
        }
        node = node.get(*part)?;
    }
    None
}

/// Strictly validate a boolean field — returns error if value exists but is not boolean.
fn yaml_validate_bool(root: &serde_yaml::Value, path: &str) -> Result<Option<bool>, ConfigError> {
    match yaml_navigate(root, path) {
        None => Ok(None),
        Some(v) => match v.as_bool() {
            Some(b) => Ok(Some(b)),
            None => Err(ConfigError::validation(
                format!(
                    "invalid type: expected boolean, found {}",
                    yaml_type_name(v)
                ),
                Some(path.to_string()),
                Some("boolean".to_string()),
                Some(yaml_type_name(v).to_string()),
            )),
        },
    }
}

/// Strictly validate an integer field — returns error if value exists but is not an integer.
fn yaml_validate_u64(root: &serde_yaml::Value, path: &str) -> Result<Option<u64>, ConfigError> {
    match yaml_navigate(root, path) {
        None => Ok(None),
        Some(v) => match v.as_u64() {
            Some(n) => Ok(Some(n)),
            None => Err(ConfigError::validation(
                format!(
                    "invalid type: expected integer, found {}",
                    yaml_type_name(v)
                ),
                Some(path.to_string()),
                Some("integer".to_string()),
                Some(yaml_type_name(v).to_string()),
            )),
        },
    }
}

/// Strictly validate a float field — returns error if value exists but is not a number.
fn yaml_validate_f64(root: &serde_yaml::Value, path: &str) -> Result<Option<f64>, ConfigError> {
    match yaml_navigate(root, path) {
        None => Ok(None),
        Some(v) => match v.as_f64() {
            Some(n) => Ok(Some(n)),
            None => Err(ConfigError::validation(
                format!("invalid type: expected number, found {}", yaml_type_name(v)),
                Some(path.to_string()),
                Some("number".to_string()),
                Some(yaml_type_name(v).to_string()),
            )),
        },
    }
}

/// Strictly validate a string field — returns error if value exists but is not a string.
fn yaml_validate_str<'a>(
    root: &'a serde_yaml::Value,
    path: &str,
) -> Result<Option<&'a str>, ConfigError> {
    match yaml_navigate(root, path) {
        None => Ok(None),
        Some(v) => match v.as_str() {
            Some(s) => Ok(Some(s)),
            None => Err(ConfigError::validation(
                format!("invalid type: expected string, found {}", yaml_type_name(v)),
                Some(path.to_string()),
                Some("string".to_string()),
                Some(yaml_type_name(v).to_string()),
            )),
        },
    }
}

/// Get a human-readable name for a YAML value's type.
fn yaml_type_name(v: &serde_yaml::Value) -> &str {
    match v {
        serde_yaml::Value::Null => "null",
        serde_yaml::Value::Bool(_) => "boolean",
        serde_yaml::Value::Number(_) => "number",
        serde_yaml::Value::String(_) => "string",
        serde_yaml::Value::Sequence(_) => "array",
        serde_yaml::Value::Mapping(_) => "object",
        serde_yaml::Value::Tagged(_) => "tagged",
    }
}

// ---------------------------------------------------------------------------
// Range validation helpers for config hot-reload (§17.5)
// These helpers check that numeric values are within valid ranges.
// ---------------------------------------------------------------------------

/// Validate a u64 value is within a specified range.
fn yaml_validate_u64_range(
    root: &serde_yaml::Value,
    path: &str,
    min: u64,
    max: u64,
) -> Result<Option<u64>, ConfigError> {
    let value = match yaml_navigate(root, path) {
        None => return Ok(None),
        Some(v) => match v.as_u64() {
            Some(n) => n,
            None => {
                return Err(ConfigError::validation(
                    format!(
                        "invalid type: expected integer, found {}",
                        yaml_type_name(v)
                    ),
                    Some(path.to_string()),
                    Some("integer".to_string()),
                    Some(yaml_type_name(v).to_string()),
                ))
            }
        },
    };

    if value < min || value > max {
        Err(ConfigError::validation(
            format!("value {} is out of valid range [{}-{}]", value, min, max),
            Some(path.to_string()),
            Some(format!("integer in range [{}-{}]", min, max)),
            Some(value.to_string()),
        ))
    } else {
        Ok(Some(value))
    }
}

/// Validate an f64 value is within a specified range.
fn yaml_validate_f64_range(
    root: &serde_yaml::Value,
    path: &str,
    min: f64,
    max: f64,
) -> Result<Option<f64>, ConfigError> {
    let value = match yaml_navigate(root, path) {
        None => return Ok(None),
        Some(v) => match v.as_f64() {
            Some(n) => n,
            None => {
                return Err(ConfigError::validation(
                    format!(
                        "invalid type: expected number, found {}",
                        yaml_type_name(v)
                    ),
                    Some(path.to_string()),
                    Some("number".to_string()),
                    Some(yaml_type_name(v).to_string()),
                ))
            }
        },
    };

    if value < min || value > max {
        Err(ConfigError::validation(
            format!("value {} is out of valid range [{}-{}]", value, min, max),
            Some(path.to_string()),
            Some(format!("number in range [{}-{}]", min, max)),
            Some(value.to_string()),
        ))
    } else {
        Ok(Some(value))
    }
}

// ---------------------------------------------------------------------------
// Schema validation helpers (§17.5)
// ---------------------------------------------------------------------------

/// Validate schema_version follows semver pattern (X.Y.Z).
fn validate_schema_version(value: &str) -> Result<(), String> {
    // Basic semver pattern: digit.digit.digit (e.g., "1.0.0", "2.3.45")
    let re = regex::Regex::new(r"^\d+\.\d+\.\d+$").unwrap();
    if !re.is_match(value) {
        return Err(format!(
            "invalid schema_version format: \"{}\"; expected semver format X.Y.Z (e.g., \"1.0.0\")",
            value
        ));
    }
    Ok(())
}

/// Resolve the full daemon configuration.
///
/// Applies the four-layer precedence: CLI flags > env vars > config.yml > defaults.
/// Returns a `ResolvedConfig` where every key carries attribution.
pub fn resolve(cli: CliOverrides) -> ResolvedConfig {
    let yml = load_config_yml();
    let yml_ref = yml.as_ref();

    // Server
    let bind_addr = resolve_opt(
        cli.bind_addr.map(|a| a.to_string()),
        env_parse::<SocketAddr>("HOOP_BIND_ADDR").map(|a| a.to_string()),
        yml_ref
            .and_then(|y| yaml_get_str(y, "server.bind_addr"))
            .map(|s| s.to_string()),
        "127.0.0.1:3000".to_string(),
        "--addr",
        "HOOP_BIND_ADDR",
        "server.bind_addr",
    );

    let allow_br_mismatch = resolve_opt(
        cli.allow_br_mismatch,
        env_parse("HOOP_ALLOW_BR_MISMATCH"),
        None, // not in config.yml
        false,
        "--allow-br-mismatch",
        "HOOP_ALLOW_BR_MISMATCH",
        "N/A",
    );

    // Agent
    let agent_adapter = resolve_opt(
        None, // no CLI flag
        std::env::var("HOOP_AGENT_ADAPTER").ok(),
        yml_ref
            .and_then(|y| yaml_get_str(y, "agent.adapter"))
            .map(|s| s.to_string()),
        "claude".to_string(),
        "N/A",
        "HOOP_AGENT_ADAPTER",
        "agent.adapter",
    );

    let agent_model = resolve_opt(
        None,
        std::env::var("HOOP_AGENT_MODEL").ok(),
        yml_ref
            .and_then(|y| yaml_get_str(y, "agent.model"))
            .map(|s| s.to_string()),
        "claude-opus-4-7".to_string(),
        "N/A",
        "HOOP_AGENT_MODEL",
        "agent.model",
    );

    let agent_anthropic_api_key = resolve_opt_none(
        None::<String>,
        std::env::var("ANTHROPIC_API_KEY").ok(),
        yml_ref
            .and_then(|y| yaml_get_str(y, "agent.anthropic_api_key"))
            .map(|s| s.to_string()),
        "N/A",
        "ANTHROPIC_API_KEY",
        "agent.anthropic_api_key",
        "anthropic_api_key",
    );

    let agent_zai_base_url = resolve_opt_none(
        None::<String>,
        std::env::var("HOOP_ZAI_BASE_URL").ok(),
        yml_ref
            .and_then(|y| yaml_get_str(y, "agent.zai_base_url"))
            .map(|s| s.to_string()),
        "N/A",
        "HOOP_ZAI_BASE_URL",
        "agent.zai_base_url",
        "zai_base_url",
    );

    let agent_zai_api_key = resolve_opt_none(
        None::<String>,
        std::env::var("HOOP_ZAI_API_KEY").ok(),
        yml_ref
            .and_then(|y| yaml_get_str(y, "agent.zai_api_key"))
            .map(|s| s.to_string()),
        "N/A",
        "HOOP_ZAI_API_KEY",
        "agent.zai_api_key",
        "zai_api_key",
    );

    let agent_rate_limit_rpm = resolve_opt_none(
        None::<u32>,
        env_parse("HOOP_RATE_LIMIT_RPM"),
        yml_ref
            .and_then(|y| yaml_get_u64(y, "agent.rate_limit_requests_per_minute"))
            .map(|v| v as u32),
        "N/A",
        "HOOP_RATE_LIMIT_RPM",
        "agent.rate_limit_requests_per_minute",
        "rate_limit_rpm",
    );

    let agent_cost_cap_usd = resolve_opt_none(
        None::<f64>,
        env_parse("HOOP_COST_CAP_USD"),
        yml_ref.and_then(|y| yaml_get_f64(y, "agent.cost_cap_per_session_usd")),
        "N/A",
        "HOOP_COST_CAP_USD",
        "agent.cost_cap_per_session_usd",
        "cost_cap_usd",
    );

    // Projects
    let default_projects_path = {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".hoop")
            .join("projects.yaml")
            .to_str()
            .unwrap_or("~/.hoop/projects.yaml")
            .to_string()
    };

    let projects_file = resolve_opt(
        None,
        std::env::var("HOOP_PROJECTS_FILE").ok(),
        yml_ref
            .and_then(|y| yaml_get_str(y, "projects_file"))
            .map(|s| s.to_string()),
        default_projects_path,
        "N/A",
        "HOOP_PROJECTS_FILE",
        "projects_file",
    );

    // UI
    let ui_theme = resolve_opt(
        None,
        std::env::var("HOOP_UI_THEME").ok(),
        yml_ref
            .and_then(|y| yaml_get_str(y, "ui.theme"))
            .map(|s| s.to_string()),
        "auto".to_string(),
        "N/A",
        "HOOP_UI_THEME",
        "ui.theme",
    );

    let ui_default_project_sort = resolve_opt(
        None,
        std::env::var("HOOP_UI_SORT").ok(),
        yml_ref
            .and_then(|y| yaml_get_str(y, "ui.default_project_sort"))
            .map(|s| s.to_string()),
        "last_activity".to_string(),
        "N/A",
        "HOOP_UI_SORT",
        "ui.default_project_sort",
    );

    let ui_archive_after_days = resolve_opt(
        None::<u32>,
        env_parse("HOOP_ARCHIVE_DAYS"),
        yml_ref
            .and_then(|y| yaml_get_u64(y, "ui.archive_after_days"))
            .map(|v| v as u32),
        30,
        "N/A",
        "HOOP_ARCHIVE_DAYS",
        "ui.archive_after_days",
    );

    // Metrics
    let metrics_enabled = resolve_opt(
        None::<bool>,
        env_parse("HOOP_METRICS_ENABLED"),
        yml_ref.and_then(|y| yaml_get_bool(y, "metrics.enabled")),
        false,
        "N/A",
        "HOOP_METRICS_ENABLED",
        "metrics.enabled",
    );

    let metrics_port = resolve_opt(
        None::<u16>,
        env_parse("HOOP_METRICS_PORT"),
        yml_ref
            .and_then(|y| yaml_get_u64(y, "metrics.port"))
            .map(|v| v as u16),
        9091,
        "N/A",
        "HOOP_METRICS_PORT",
        "metrics.port",
    );

    // Voice
    let default_whisper_path = {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".hoop")
            .join("models")
            .join("ggml-base.en.bin")
            .to_str()
            .unwrap_or("")
            .to_string()
    };

    let voice_whisper_model_path = resolve_opt_none(
        None::<String>,
        std::env::var("HOOP_WHISPER_MODEL_PATH").ok(),
        yml_ref
            .and_then(|y| yaml_get_str(y, "voice.whisper_model_path"))
            .map(|s| s.to_string()),
        "N/A",
        "HOOP_WHISPER_MODEL_PATH",
        "voice.whisper_model_path",
        "whisper_model_path",
    );

    // If the whisper path resolved to None but we have a default path, patch the attribution
    let voice_whisper_model_path = if voice_whisper_model_path.value.is_none() {
        Resolved::new(
            Some(default_whisper_path),
            ConfigSource::Default,
            "compiled default (~/.hoop/models/ggml-base.en.bin)".to_string(),
        )
    } else {
        voice_whisper_model_path
    };

    let voice_hotkey = resolve_opt(
        None,
        std::env::var("HOOP_VOICE_HOTKEY").ok(),
        yml_ref
            .and_then(|y| yaml_get_str(y, "voice.hotkey"))
            .map(|s| s.to_string()),
        "Ctrl+Shift+V".to_string(),
        "N/A",
        "HOOP_VOICE_HOTKEY",
        "voice.hotkey",
    );

    let voice_max_recording_seconds = resolve_opt(
        None::<u32>,
        env_parse("HOOP_VOICE_MAX_SECONDS"),
        yml_ref
            .and_then(|y| yaml_get_u64(y, "voice.max_recording_seconds"))
            .map(|v| v as u32),
        300,
        "N/A",
        "HOOP_VOICE_MAX_SECONDS",
        "voice.max_recording_seconds",
    );

    // Audit
    let audit_retention_days = resolve_opt(
        None::<u32>,
        env_parse("HOOP_AUDIT_RETENTION_DAYS"),
        yml_ref
            .and_then(|y| yaml_get_u64(y, "audit.retention_days"))
            .map(|v| v as u32),
        90,
        "N/A",
        "HOOP_AUDIT_RETENTION_DAYS",
        "audit.retention_days",
    );

    let audit_hash_chain = resolve_opt(
        None::<bool>,
        env_parse("HOOP_AUDIT_HASH_CHAIN"),
        yml_ref.and_then(|y| yaml_get_bool(y, "audit.hash_chain")),
        true,
        "N/A",
        "HOOP_AUDIT_HASH_CHAIN",
        "audit.hash_chain",
    );

    // Reflection
    let reflection_enabled = resolve_opt(
        None::<bool>,
        env_parse("HOOP_REFLECTION_ENABLED"),
        yml_ref.and_then(|y| yaml_get_bool(y, "reflection.enabled")),
        true,
        "N/A",
        "HOOP_REFLECTION_ENABLED",
        "reflection.enabled",
    );

    let reflection_detection_threshold = resolve_opt(
        None::<f64>,
        env_parse("HOOP_REFLECTION_THRESHOLD"),
        yml_ref.and_then(|y| yaml_get_f64(y, "reflection.detection_threshold")),
        0.8,
        "N/A",
        "HOOP_REFLECTION_THRESHOLD",
        "reflection.detection_threshold",
    );

    let reflection_auto_archive_after_days = resolve_opt(
        None::<u32>,
        env_parse("HOOP_REFLECTION_ARCHIVE_DAYS"),
        yml_ref
            .and_then(|y| yaml_get_u64(y, "reflection.auto_archive_after_days"))
            .map(|v| v as u32),
        30,
        "N/A",
        "HOOP_REFLECTION_ARCHIVE_DAYS",
        "reflection.auto_archive_after_days",
    );

    // Agent extensions (§22)
    let agent_extensions_skills = resolve_opt_none(
        None::<String>,
        std::env::var("HOOP_AGENT_EXTENSIONS_SKILLS").ok(),
        yml_ref
            .and_then(|y| yaml_get_str(y, "agent_extensions.skills"))
            .map(|s| s.to_string()),
        "N/A",
        "HOOP_AGENT_EXTENSIONS_SKILLS",
        "agent_extensions.skills",
        "skills",
    );

    let agent_extensions_scripts = resolve_opt_none(
        None::<String>,
        std::env::var("HOOP_AGENT_EXTENSIONS_SCRIPTS").ok(),
        yml_ref
            .and_then(|y| yaml_get_str(y, "agent_extensions.scripts"))
            .map(|s| s.to_string()),
        "N/A",
        "HOOP_AGENT_EXTENSIONS_SCRIPTS",
        "agent_extensions.scripts",
        "scripts",
    );

    let agent_extensions_notes = resolve_opt_none(
        None::<String>,
        std::env::var("HOOP_AGENT_EXTENSIONS_NOTES").ok(),
        yml_ref
            .and_then(|y| yaml_get_str(y, "agent_extensions.notes"))
            .map(|s| s.to_string()),
        "N/A",
        "HOOP_AGENT_EXTENSIONS_NOTES",
        "agent_extensions.notes",
        "notes",
    );

    let agent_extensions_prompts = resolve_opt_none(
        None::<String>,
        std::env::var("HOOP_AGENT_EXTENSIONS_PROMPTS").ok(),
        yml_ref
            .and_then(|y| yaml_get_str(y, "agent_extensions.prompts"))
            .map(|s| s.to_string()),
        "N/A",
        "HOOP_AGENT_EXTENSIONS_PROMPTS",
        "agent_extensions.prompts",
        "prompts",
    );

    // Backup (§15.2, §17.3) - credentials from env only, never from config.yml
    let backup_endpoint = resolve_opt_none(
        None::<String>,
        None, // no env var for endpoint (in config.yml only)
        yml_ref
            .and_then(|y| yaml_get_str(y, "backup.endpoint"))
            .map(|s| s.to_string()),
        "N/A",
        "N/A",
        "backup.endpoint",
        "endpoint",
    );
    let backup_bucket = resolve_opt_none(
        None::<String>,
        None, // no env var for bucket (in config.yml only)
        yml_ref
            .and_then(|y| yaml_get_str(y, "backup.bucket"))
            .map(|s| s.to_string()),
        "N/A",
        "N/A",
        "backup.bucket",
        "bucket",
    );
    let backup_prefix = resolve_opt_none(
        None::<String>,
        None, // no env var for prefix (in config.yml only)
        yml_ref
            .and_then(|y| yaml_get_str(y, "backup.prefix"))
            .map(|s| s.to_string()),
        "N/A",
        "N/A",
        "backup.prefix",
        "prefix",
    );
    let backup_schedule = resolve_opt(
        None,
        None,
        yml_ref
            .and_then(|y| yaml_get_str(y, "backup.schedule"))
            .map(|s| s.to_string()),
        "0 4 * * *".to_string(),
        "N/A",
        "N/A",
        "backup.schedule",
    );
    let backup_retention_days = resolve_opt(
        None::<u32>,
        env_parse("HOOP_BACKUP_RETENTION_DAYS"),
        yml_ref
            .and_then(|y| yaml_get_u64(y, "backup.retention_days"))
            .map(|v| v as u32),
        30,
        "N/A",
        "HOOP_BACKUP_RETENTION_DAYS",
        "backup.retention_days",
    );
    let backup_encryption = resolve_opt(
        None::<bool>,
        None, // encryption flag in config.yml only
        yml_ref.and_then(|y| yaml_get_bool(y, "backup.encryption")),
        false,
        "N/A",
        "N/A",
        "backup.encryption",
    );

    // Pricing (§17.3) - path to pricing config file
    let default_pricing_path = {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".hoop")
            .join("pricing.yml")
            .to_str()
            .unwrap_or("~/.hoop/pricing.yml")
            .to_string()
    };
    let pricing_file = resolve_opt(
        None,
        std::env::var("HOOP_PRICING_FILE").ok(),
        yml_ref
            .and_then(|y| yaml_get_str(y, "pricing_file"))
            .map(|s| s.to_string()),
        default_pricing_path,
        "N/A",
        "HOOP_PRICING_FILE",
        "pricing_file",
    );

    // Secrets scanner patterns (§18)
    let secrets_patterns = if let Some(patterns) = yml_ref.and_then(|y| yaml_get_secret_patterns(y)) {
        // Validate all patterns
        let mut valid_patterns = Vec::new();
        for pat in patterns {
            if let Err(e) = pat.validate() {
                warn!("Invalid secret pattern '{}': {}, using default", pat.name, e);
            } else if !pat.is_valid_severity() {
                warn!("Invalid severity '{}' for pattern '{}', using default", pat.severity, pat.name);
            } else {
                valid_patterns.push(pat);
            }
        }
        if valid_patterns.is_empty() {
            Resolved::new(
                default_secret_patterns(),
                ConfigSource::Default,
                "compiled default (configured patterns were invalid)".to_string(),
            )
        } else {
            Resolved::new(
                valid_patterns,
                ConfigSource::ConfigYml,
                "config.yml: secrets_patterns".to_string(),
            )
        }
    } else {
        Resolved::new(
            default_secret_patterns(),
            ConfigSource::Default,
            "compiled default".to_string(),
        )
    };

    // Load stuck detector configuration
    let stuck_detector_config_map = crate::stuck_detector::StuckDetector::load_config();
    let stuck_detector = Resolved::new(
        Some(stuck_detector_config_map),
        ConfigSource::ConfigYml,
        "config.yml: stuck_detector".to_string(),
    );

    // Load role configuration (RBAC)
    let roles = if let Some(role_config) = yml_ref.and_then(|y| yaml_get_role_config(y)) {
        Resolved::new(
            role_config,
            ConfigSource::ConfigYml,
            "config.yml: roles".to_string(),
        )
    } else {
        Resolved::new(
            crate::auth::RoleConfig::default(),
            ConfigSource::Default,
            "compiled default (no roles configured)".to_string(),
        )
    };

    // Mark restart-required keys (§17.4)
    let bind_addr = bind_addr.with_restart_required();
    let metrics_port = metrics_port.with_restart_required();

    let config = ResolvedConfig {
        bind_addr,
        allow_br_mismatch,
        agent_adapter,
        agent_model,
        agent_anthropic_api_key,
        agent_zai_base_url,
        agent_zai_api_key,
        agent_rate_limit_rpm,
        agent_cost_cap_usd,
        projects_file,
        ui_theme,
        ui_default_project_sort,
        ui_archive_after_days,
        metrics_enabled,
        metrics_port,
        voice_whisper_model_path,
        voice_hotkey,
        voice_max_recording_seconds,
        agent_extensions_skills,
        agent_extensions_scripts,
        agent_extensions_notes,
        agent_extensions_prompts,
        audit_retention_days,
        audit_hash_chain,
        reflection_enabled,
        reflection_detection_threshold,
        reflection_auto_archive_after_days,
        backup_endpoint,
        backup_bucket,
        backup_prefix,
        backup_schedule,
        backup_retention_days,
        backup_encryption,
        pricing_file,
        secrets_patterns,
        stuck_detector,
        roles,
    };

    // Log the resolution summary
    info!(
        "Config resolved: bind_addr={} ({})",
        config.bind_addr.value, config.bind_addr.attribution
    );
    info!(
        "Config resolved: agent.adapter={} ({})",
        config.agent_adapter.value, config.agent_adapter.attribution
    );
    info!(
        "Config resolved: agent.model={} ({})",
        config.agent_model.value, config.agent_model.attribution
    );

    config
}

impl ResolvedConfig {
    /// Convert to a flat map of key → {value, source, resolved_from} for the
    /// /debug/state endpoint.
    pub fn to_debug_map(&self) -> BTreeMap<String, serde_json::Value> {
        let mut map = BTreeMap::new();

        // Serialize the whole struct — each field is a Resolved<T> which
        // produces { value, source, resolved_from } per key.
        let full = serde_json::to_value(self).unwrap_or(serde_json::Value::Null);
        if let serde_json::Value::Object(obj) = full {
            for (key, val) in obj {
                map.insert(key, val);
            }
        }

        map
    }
}

// ---------------------------------------------------------------------------
// Raw YAML resolution with validation (for hot-reload, §17.5)
// ---------------------------------------------------------------------------

/// Resolve configuration from a raw YAML string with validation.
///
/// This function is used by the config watcher to parse and validate
/// config.yml content before applying it. Returns a `ConfigError` with
/// structured details if parsing or validation fails.
///
/// Pipeline:
/// 1. Parse YAML into `serde_yaml::Value`
/// 2. Extract each field using the yaml_get helpers
/// 3. Apply semantic validation (e.g. agent.adapter enum)
/// 4. Apply CLI overrides with proper precedence
/// 5. Return fully resolved `ResolvedConfig`
pub fn resolve_from_raw(cli: CliOverrides, raw: &str) -> Result<ResolvedConfig, ConfigError> {
    // Empty config is valid — uses all defaults
    let yml = if raw.trim().is_empty() {
        None
    } else {
        match serde_yaml::from_str::<serde_yaml::Value>(raw) {
            Ok(v) => Some(v),
            Err(e) => return Err(ConfigError::from_yaml(&e)),
        }
    };
    let yml_ref = yml.as_ref();

    // Validate schema_version format if present (§17.5)
    if let Some(yml) = yml_ref {
        if let Some(version_value) = yaml_get_str(yml, "schema_version") {
            if let Err(e) = validate_schema_version(version_value) {
                return Err(ConfigError::validation(
                    e,
                    Some("schema_version".to_string()),
                    Some("semver format X.Y.Z".to_string()),
                    Some(version_value.to_string()),
                ));
            }
        }
    }

    // ── Helper: resolve a string value with semantic validation ─────────────
    fn resolve_validated_str(
        cli: Option<String>,
        env_var: &str,
        yml_ref: Option<&serde_yaml::Value>,
        yml_path: &str,
        default: &str,
        cli_label: &str,
        env_label: &str,
        file_label: &str,
        validator: fn(&str) -> Result<(), String>,
    ) -> Result<Resolved<String>, ConfigError> {
        let file_val = yml_ref
            .and_then(|y| yaml_get_str(y, yml_path))
            .map(|s| s.to_string());
        let env_val = std::env::var(env_var).ok();

        let (value, source, attribution) = if let Some(v) = cli {
            (v, ConfigSource::CliFlag, format!("cli flag {}", cli_label))
        } else if let Some(v) = env_val {
            (v, ConfigSource::EnvVar, format!("env {}", env_label))
        } else if let Some(v) = file_val {
            (
                v.clone(),
                ConfigSource::ConfigYml,
                format!("config.yml: {}", file_label),
            )
        } else {
            (
                default.to_string(),
                ConfigSource::Default,
                "compiled default".to_string(),
            )
        };

        // Validate if the value came from config.yml or env var
        if matches!(source, ConfigSource::ConfigYml | ConfigSource::EnvVar) {
            if let Err(e) = validator(&value) {
                return Err(ConfigError::validation(
                    e,
                    Some(file_label.to_string()),
                    None,
                    Some(value),
                ));
            }
        }

        Ok(Resolved::new(value, source, attribution))
    }

    // Server
    let bind_addr = resolve_opt(
        cli.bind_addr.map(|a| a.to_string()),
        env_parse::<SocketAddr>("HOOP_BIND_ADDR").map(|a| a.to_string()),
        yml_ref
            .and_then(|y| yaml_get_str(y, "server.bind_addr"))
            .map(|s| s.to_string()),
        "127.0.0.1:3000".to_string(),
        "--addr",
        "HOOP_BIND_ADDR",
        "server.bind_addr",
    );

    let allow_br_mismatch = resolve_opt(
        cli.allow_br_mismatch,
        env_parse("HOOP_ALLOW_BR_MISMATCH"),
        None,
        false,
        "--allow-br-mismatch",
        "HOOP_ALLOW_BR_MISMATCH",
        "N/A",
    );

    // Agent — validate adapter enum
    let agent_adapter = resolve_validated_str(
        None,
        "HOOP_AGENT_ADAPTER",
        yml_ref,
        "agent.adapter",
        "claude",
        "N/A",
        "HOOP_AGENT_ADAPTER",
        "agent.adapter",
        validate_agent_adapter,
    )?;

    let agent_model = resolve_opt(
        None,
        std::env::var("HOOP_AGENT_MODEL").ok(),
        yml_ref
            .and_then(|y| yaml_get_str(y, "agent.model"))
            .map(|s| s.to_string()),
        "claude-opus-4-7".to_string(),
        "N/A",
        "HOOP_AGENT_MODEL",
        "agent.model",
    );

    let agent_anthropic_api_key = resolve_opt_none(
        None::<String>,
        std::env::var("ANTHROPIC_API_KEY").ok(),
        yml_ref
            .and_then(|y| yaml_get_str(y, "agent.anthropic_api_key"))
            .map(|s| s.to_string()),
        "N/A",
        "ANTHROPIC_API_KEY",
        "agent.anthropic_api_key",
        "anthropic_api_key",
    );

    let agent_zai_base_url = resolve_opt_none(
        None::<String>,
        std::env::var("HOOP_ZAI_BASE_URL").ok(),
        yml_ref
            .and_then(|y| yaml_get_str(y, "agent.zai_base_url"))
            .map(|s| s.to_string()),
        "N/A",
        "HOOP_ZAI_BASE_URL",
        "agent.zai_base_url",
        "zai_base_url",
    );

    let agent_zai_api_key = resolve_opt_none(
        None::<String>,
        std::env::var("HOOP_ZAI_API_KEY").ok(),
        yml_ref
            .and_then(|y| yaml_get_str(y, "agent.zai_api_key"))
            .map(|s| s.to_string()),
        "N/A",
        "HOOP_ZAI_API_KEY",
        "agent.zai_api_key",
        "zai_api_key",
    );

    let agent_rate_limit_rpm = resolve_opt_none(
        None::<u32>,
        env_parse("HOOP_RATE_LIMIT_RPM"),
        yml_ref
            .and_then(|y| yaml_get_u64(y, "agent.rate_limit_requests_per_minute"))
            .map(|v| v as u32),
        "N/A",
        "HOOP_RATE_LIMIT_RPM",
        "agent.rate_limit_requests_per_minute",
        "rate_limit_rpm",
    );

    let agent_cost_cap_usd = resolve_opt_none(
        None::<f64>,
        env_parse("HOOP_COST_CAP_USD"),
        yml_ref.and_then(|y| yaml_get_f64(y, "agent.cost_cap_per_session_usd")),
        "N/A",
        "HOOP_COST_CAP_USD",
        "agent.cost_cap_per_session_usd",
        "cost_cap_usd",
    );

    // Projects
    let default_projects_path = {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".hoop")
            .join("projects.yaml")
            .to_str()
            .unwrap_or("~/.hoop/projects.yaml")
            .to_string()
    };

    let projects_file = resolve_opt(
        None,
        std::env::var("HOOP_PROJECTS_FILE").ok(),
        yml_ref
            .and_then(|y| yaml_get_str(y, "projects_file"))
            .map(|s| s.to_string()),
        default_projects_path,
        "N/A",
        "HOOP_PROJECTS_FILE",
        "projects_file",
    );

    // UI — validate theme and sort enums
    let ui_theme = resolve_validated_str(
        None,
        "HOOP_UI_THEME",
        yml_ref,
        "ui.theme",
        "auto",
        "N/A",
        "HOOP_UI_THEME",
        "ui.theme",
        validate_ui_theme,
    )?;

    let ui_default_project_sort = resolve_validated_str(
        None,
        "HOOP_UI_SORT",
        yml_ref,
        "ui.default_project_sort",
        "last_activity",
        "N/A",
        "HOOP_UI_SORT",
        "ui.default_project_sort",
        validate_ui_sort,
    )?;

    let ui_archive_after_days = resolve_opt(
        None::<u32>,
        env_parse("HOOP_ARCHIVE_DAYS"),
        yml_ref
            .and_then(|y| yaml_get_u64(y, "ui.archive_after_days"))
            .map(|v| v as u32),
        30,
        "N/A",
        "HOOP_ARCHIVE_DAYS",
        "ui.archive_after_days",
    );

    // Metrics
    let metrics_enabled = resolve_opt(
        None::<bool>,
        env_parse("HOOP_METRICS_ENABLED"),
        yml_ref.and_then(|y| yaml_get_bool(y, "metrics.enabled")),
        false,
        "N/A",
        "HOOP_METRICS_ENABLED",
        "metrics.enabled",
    );

    let metrics_port = resolve_opt(
        None::<u16>,
        env_parse("HOOP_METRICS_PORT"),
        yml_ref
            .and_then(|y| yaml_get_u64(y, "metrics.port"))
            .map(|v| v as u16),
        9091,
        "N/A",
        "HOOP_METRICS_PORT",
        "metrics.port",
    );

    // Voice
    let default_whisper_path = {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".hoop")
            .join("models")
            .join("ggml-base.en.bin")
            .to_str()
            .unwrap_or("")
            .to_string()
    };

    let voice_whisper_model_path = resolve_opt_none(
        None::<String>,
        std::env::var("HOOP_WHISPER_MODEL_PATH").ok(),
        yml_ref
            .and_then(|y| yaml_get_str(y, "voice.whisper_model_path"))
            .map(|s| s.to_string()),
        "N/A",
        "HOOP_WHISPER_MODEL_PATH",
        "voice.whisper_model_path",
        "whisper_model_path",
    );

    let voice_whisper_model_path = if voice_whisper_model_path.value.is_none() {
        Resolved::new(
            Some(default_whisper_path),
            ConfigSource::Default,
            "compiled default (~/.hoop/models/ggml-base.en.bin)".to_string(),
        )
    } else {
        voice_whisper_model_path
    };

    let voice_hotkey = resolve_opt(
        None,
        std::env::var("HOOP_VOICE_HOTKEY").ok(),
        yml_ref
            .and_then(|y| yaml_get_str(y, "voice.hotkey"))
            .map(|s| s.to_string()),
        "Ctrl+Shift+V".to_string(),
        "N/A",
        "HOOP_VOICE_HOTKEY",
        "voice.hotkey",
    );

    let voice_max_recording_seconds = resolve_opt(
        None::<u32>,
        env_parse("HOOP_VOICE_MAX_SECONDS"),
        yml_ref
            .and_then(|y| yaml_get_u64(y, "voice.max_recording_seconds"))
            .map(|v| v as u32),
        300,
        "N/A",
        "HOOP_VOICE_MAX_SECONDS",
        "voice.max_recording_seconds",
    );

    // Agent extensions (§22)
    let agent_extensions_skills = resolve_opt_none(
        None::<String>,
        std::env::var("HOOP_AGENT_EXTENSIONS_SKILLS").ok(),
        yml_ref
            .and_then(|y| yaml_get_str(y, "agent_extensions.skills"))
            .map(|s| s.to_string()),
        "N/A",
        "HOOP_AGENT_EXTENSIONS_SKILLS",
        "agent_extensions.skills",
        "skills",
    );

    let agent_extensions_scripts = resolve_opt_none(
        None::<String>,
        std::env::var("HOOP_AGENT_EXTENSIONS_SCRIPTS").ok(),
        yml_ref
            .and_then(|y| yaml_get_str(y, "agent_extensions.scripts"))
            .map(|s| s.to_string()),
        "N/A",
        "HOOP_AGENT_EXTENSIONS_SCRIPTS",
        "agent_extensions.scripts",
        "scripts",
    );

    let agent_extensions_notes = resolve_opt_none(
        None::<String>,
        std::env::var("HOOP_AGENT_EXTENSIONS_NOTES").ok(),
        yml_ref
            .and_then(|y| yaml_get_str(y, "agent_extensions.notes"))
            .map(|s| s.to_string()),
        "N/A",
        "HOOP_AGENT_EXTENSIONS_NOTES",
        "agent_extensions.notes",
        "notes",
    );

    let agent_extensions_prompts = resolve_opt_none(
        None::<String>,
        std::env::var("HOOP_AGENT_EXTENSIONS_PROMPTS").ok(),
        yml_ref
            .and_then(|y| yaml_get_str(y, "agent_extensions.prompts"))
            .map(|s| s.to_string()),
        "N/A",
        "HOOP_AGENT_EXTENSIONS_PROMPTS",
        "agent_extensions.prompts",
        "prompts",
    );

    // Audit
    let audit_retention_days = resolve_opt(
        None::<u32>,
        env_parse("HOOP_AUDIT_RETENTION_DAYS"),
        yml_ref
            .and_then(|y| yaml_get_u64(y, "audit.retention_days"))
            .map(|v| v as u32),
        90,
        "N/A",
        "HOOP_AUDIT_RETENTION_DAYS",
        "audit.retention_days",
    );

    let audit_hash_chain = resolve_opt(
        None::<bool>,
        env_parse("HOOP_AUDIT_HASH_CHAIN"),
        yml_ref.and_then(|y| yaml_get_bool(y, "audit.hash_chain")),
        true,
        "N/A",
        "HOOP_AUDIT_HASH_CHAIN",
        "audit.hash_chain",
    );

    // Reflection
    let reflection_enabled = resolve_opt(
        None::<bool>,
        env_parse("HOOP_REFLECTION_ENABLED"),
        yml_ref.and_then(|y| yaml_get_bool(y, "reflection.enabled")),
        true,
        "N/A",
        "HOOP_REFLECTION_ENABLED",
        "reflection.enabled",
    );

    let reflection_detection_threshold = resolve_opt(
        None::<f64>,
        env_parse("HOOP_REFLECTION_THRESHOLD"),
        yml_ref.and_then(|y| yaml_get_f64(y, "reflection.detection_threshold")),
        0.8,
        "N/A",
        "HOOP_REFLECTION_THRESHOLD",
        "reflection.detection_threshold",
    );

    let reflection_auto_archive_after_days = resolve_opt(
        None::<u32>,
        env_parse("HOOP_REFLECTION_ARCHIVE_DAYS"),
        yml_ref
            .and_then(|y| yaml_get_u64(y, "reflection.auto_archive_after_days"))
            .map(|v| v as u32),
        30,
        "N/A",
        "HOOP_REFLECTION_ARCHIVE_DAYS",
        "reflection.auto_archive_after_days",
    );

    // Backup (§15.2, §17.3) - credentials from env only, never from config.yml
    let backup_endpoint = resolve_opt_none(
        None::<String>,
        None, // no env var for endpoint (in config.yml only)
        yml_ref
            .and_then(|y| yaml_get_str(y, "backup.endpoint"))
            .map(|s| s.to_string()),
        "N/A",
        "N/A",
        "backup.endpoint",
        "endpoint",
    );
    let backup_bucket = resolve_opt_none(
        None::<String>,
        None, // no env var for bucket (in config.yml only)
        yml_ref
            .and_then(|y| yaml_get_str(y, "backup.bucket"))
            .map(|s| s.to_string()),
        "N/A",
        "N/A",
        "backup.bucket",
        "bucket",
    );
    let backup_prefix = resolve_opt_none(
        None::<String>,
        None, // no env var for prefix (in config.yml only)
        yml_ref
            .and_then(|y| yaml_get_str(y, "backup.prefix"))
            .map(|s| s.to_string()),
        "N/A",
        "N/A",
        "backup.prefix",
        "prefix",
    );
    let backup_schedule = resolve_opt(
        None,
        None,
        yml_ref
            .and_then(|y| yaml_get_str(y, "backup.schedule"))
            .map(|s| s.to_string()),
        "0 4 * * *".to_string(),
        "N/A",
        "N/A",
        "backup.schedule",
    );
    let backup_retention_days = resolve_opt(
        None::<u32>,
        env_parse("HOOP_BACKUP_RETENTION_DAYS"),
        yml_ref
            .and_then(|y| yaml_get_u64(y, "backup.retention_days"))
            .map(|v| v as u32),
        30,
        "N/A",
        "HOOP_BACKUP_RETENTION_DAYS",
        "backup.retention_days",
    );
    let backup_encryption = resolve_opt(
        None::<bool>,
        None, // encryption flag in config.yml only
        yml_ref.and_then(|y| yaml_get_bool(y, "backup.encryption")),
        false,
        "N/A",
        "N/A",
        "backup.encryption",
    );

    // Pricing (§17.3) - path to pricing config file
    let default_pricing_path = {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".hoop")
            .join("pricing.yml")
            .to_str()
            .unwrap_or("~/.hoop/pricing.yml")
            .to_string()
    };
    let pricing_file = resolve_opt(
        None,
        std::env::var("HOOP_PRICING_FILE").ok(),
        yml_ref
            .and_then(|y| yaml_get_str(y, "pricing_file"))
            .map(|s| s.to_string()),
        default_pricing_path,
        "N/A",
        "HOOP_PRICING_FILE",
        "pricing_file",
    );

    // Secrets scanner patterns (§18)
    let secrets_patterns = if let Some(patterns) = yml_ref.and_then(|y| yaml_get_secret_patterns(y)) {
        // Validate all patterns
        let mut valid_patterns = Vec::new();
        for pat in patterns {
            if let Err(e) = pat.validate() {
                return Err(ConfigError::validation(
                    format!("invalid secret pattern '{}': {}", pat.name, e),
                    Some("secrets_patterns".to_string()),
                    None,
                    None,
                ));
            } else if !pat.is_valid_severity() {
                return Err(ConfigError::validation(
                    format!("invalid severity '{}' for pattern '{}'", pat.severity, pat.name),
                    Some("secrets_patterns".to_string()),
                    Some("one of: high, medium, low".to_string()),
                    Some(pat.severity.clone()),
                ));
            } else {
                valid_patterns.push(pat);
            }
        }
        Resolved::new(
            valid_patterns,
            ConfigSource::ConfigYml,
            "config.yml: secrets_patterns".to_string(),
        )
    } else {
        Resolved::new(
            default_secret_patterns(),
            ConfigSource::Default,
            "compiled default".to_string(),
        )
    };

    // Validate unknown top-level fields (config.yml only)
    if let Some(yml) = yml_ref {
        if let Some(mapping) = yml.as_mapping() {
            for key in mapping.keys() {
                if let Some(field_name) = key.as_str() {
                    const VALID_TOP_LEVEL_KEYS: &[&str] = &[
                        "schema_version",
                        "server",
                        "agent",
                        "projects_file",
                        "ui",
                        "metrics",
                        "voice",
                        "agent_extensions",
                        "audit",
                        "reflection",
                        "backup",
                        "pricing",
                        "secrets_patterns",
                        "stuck_detector",
                        "roles",
                    ];
                    if !VALID_TOP_LEVEL_KEYS.contains(&field_name) {
                        return Err(ConfigError::validation(
                            format!(
                                "unknown field `{}`, expected one of: {}",
                                field_name,
                                VALID_TOP_LEVEL_KEYS.join(", ")
                            ),
                            Some(field_name.to_string()),
                            Some("known field".to_string()),
                            Some("unknown field".to_string()),
                        ));
                    }
                }
            }
        }
    }

    // ── Range validations (§17.5) ─────────────────────────────────────────────────────

    // Validate metrics.port is in valid port range (1-65535)
    let port = metrics_port.value;
    if port == 0 || port > 65535 {
        return Err(ConfigError::validation(
            format!("metrics.port {} is out of valid port range [1-65535]", port),
            Some("metrics.port".to_string()),
            Some("1-65535".to_string()),
            Some(port.to_string()),
        ));
    }

    // Validate days fields are positive
    if audit_retention_days.value == 0 {
        return Err(ConfigError::validation(
            "audit.retention_days must be positive".to_string(),
            Some("audit.retention_days".to_string()),
            Some("positive integer".to_string()),
            Some(audit_retention_days.value.to_string()),
        ));
    }

    if ui_archive_after_days.value == 0 {
        return Err(ConfigError::validation(
            "ui.archive_after_days must be positive".to_string(),
            Some("ui.archive_after_days".to_string()),
            Some("positive integer".to_string()),
            Some(ui_archive_after_days.value.to_string()),
        ));
    }

    if reflection_auto_archive_after_days.value == 0 {
        return Err(ConfigError::validation(
            "reflection.auto_archive_after_days must be positive".to_string(),
            Some("reflection.auto_archive_after_days".to_string()),
            Some("positive integer".to_string()),
            Some(reflection_auto_archive_after_days.value.to_string()),
        ));
    }

    if backup_retention_days.value == 0 {
        return Err(ConfigError::validation(
            "backup.retention_days must be positive".to_string(),
            Some("backup.retention_days".to_string()),
            Some("positive integer".to_string()),
            Some(backup_retention_days.value.to_string()),
        ));
    }

    if voice_max_recording_seconds.value == 0 {
        return Err(ConfigError::validation(
            "voice.max_recording_seconds must be positive".to_string(),
            Some("voice.max_recording_seconds".to_string()),
            Some("positive integer".to_string()),
            Some(voice_max_recording_seconds.value.to_string()),
        ));
    }

    // Validate reflection.detection_threshold is in valid range [0-1]
    if reflection_detection_threshold.value < 0.0 || reflection_detection_threshold.value > 1.0 {
        return Err(ConfigError::validation(
            format!(
                "reflection.detection_threshold {} is out of valid range [0.0-1.0]",
                reflection_detection_threshold.value
            ),
            Some("reflection.detection_threshold".to_string()),
            Some("0.0-1.0".to_string()),
            Some(reflection_detection_threshold.value.to_string()),
        ));
    }

    // Mark restart-required keys (§17.4)
    let bind_addr = bind_addr.with_restart_required();
    let metrics_port = metrics_port.with_restart_required();

    Ok(ResolvedConfig {
        bind_addr,
        allow_br_mismatch,
        agent_adapter,
        agent_model,
        agent_anthropic_api_key,
        agent_zai_base_url,
        agent_zai_api_key,
        agent_rate_limit_rpm,
        agent_cost_cap_usd,
        projects_file,
        ui_theme,
        ui_default_project_sort,
        ui_archive_after_days,
        metrics_enabled,
        metrics_port,
        voice_whisper_model_path,
        voice_hotkey,
        voice_max_recording_seconds,
        agent_extensions_skills,
        agent_extensions_scripts,
        agent_extensions_notes,
        agent_extensions_prompts,
        audit_retention_days,
        audit_hash_chain,
        reflection_enabled,
        reflection_detection_threshold,
        reflection_auto_archive_after_days,
        backup_endpoint,
        backup_bucket,
        backup_prefix,
        backup_schedule,
        backup_retention_days,
        backup_encryption,
        pricing_file,
        secrets_patterns,
        stuck_detector: Resolved::new(
            Some(crate::stuck_detector::StuckDetector::load_config()),
            ConfigSource::ConfigYml,
            "config.yml: stuck_detector".to_string(),
        ),
        roles: if let Some(role_config) = yml_ref.and_then(|y| yaml_get_role_config(y)) {
            Resolved::new(
                role_config,
                ConfigSource::ConfigYml,
                "config.yml: roles".to_string(),
            )
        } else {
            Resolved::new(
                crate::auth::RoleConfig::default(),
                ConfigSource::Default,
                "compiled default (no roles configured)".to_string(),
            )
        },
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    fn parse_addr(s: &str) -> SocketAddr {
        s.parse().unwrap()
    }

    /// CLI flag wins over everything.
    #[test]
    fn cli_flag_wins_over_env_and_file() {
        // Set env var to a different value
        std::env::set_var("HOOP_BIND_ADDR", "0.0.0.0:9999");

        let cli = CliOverrides {
            bind_addr: Some(parse_addr("127.0.0.1:4000")),
            allow_br_mismatch: None,
        };

        let config = resolve(cli);
        assert_eq!(config.bind_addr.value, "127.0.0.1:4000");
        assert_eq!(config.bind_addr.source, ConfigSource::CliFlag);
        assert!(config.bind_addr.attribution.contains("--addr"));

        std::env::remove_var("HOOP_BIND_ADDR");
    }

    /// Env var wins over config.yml and default (no CLI flag).
    #[test]
    fn env_var_wins_over_file_and_default() {
        std::env::set_var("HOOP_METRICS_PORT", "8080");

        let cli = CliOverrides::default();
        let config = resolve(cli);

        assert_eq!(config.metrics_port.value, 8080);
        assert_eq!(config.metrics_port.source, ConfigSource::EnvVar);
        assert!(config
            .metrics_port
            .attribution
            .contains("HOOP_METRICS_PORT"));

        std::env::remove_var("HOOP_METRICS_PORT");
    }

    /// Default is used when no higher layer provides a value.
    ///
    /// Only checks keys whose env vars are not touched by any other parallel
    /// test, to avoid races from `std::env::set_var` being process-global.
    #[test]
    fn default_used_when_no_overrides() {
        let cli = CliOverrides::default();
        let config = resolve(cli);

        // These keys have no env vars set by other parallel tests
        assert_eq!(config.agent_adapter.value, "claude");
        assert_eq!(config.agent_adapter.source, ConfigSource::Default);

        assert_eq!(config.agent_model.value, "claude-opus-4-7");
        assert_eq!(config.agent_model.source, ConfigSource::Default);

        assert_eq!(config.voice_max_recording_seconds.value, 300);
        assert_eq!(
            config.voice_max_recording_seconds.source,
            ConfigSource::Default
        );
    }

    /// CLI allow_br_mismatch flag resolves correctly.
    #[test]
    fn allow_br_mismatch_cli_wins() {
        std::env::set_var("HOOP_ALLOW_BR_MISMATCH", "true");

        let cli = CliOverrides {
            allow_br_mismatch: Some(true),
            ..Default::default()
        };
        let config = resolve(cli);

        assert!(config.allow_br_mismatch.value);
        assert_eq!(config.allow_br_mismatch.source, ConfigSource::CliFlag);

        std::env::remove_var("HOOP_ALLOW_BR_MISMATCH");
    }

    /// Env var allow_br_mismatch when no CLI flag.
    #[test]
    fn allow_br_mismatch_env_fallback() {
        std::env::set_var("HOOP_ALLOW_BR_MISMATCH", "true");

        let cli = CliOverrides::default();
        let config = resolve(cli);

        assert!(config.allow_br_mismatch.value);
        assert_eq!(config.allow_br_mismatch.source, ConfigSource::EnvVar);

        std::env::remove_var("HOOP_ALLOW_BR_MISMATCH");
    }

    /// resolve_opt helper — each layer wins in its scenario.
    #[test]
    fn resolve_opt_cli_over_all() {
        let r: Resolved<String> = resolve_opt(
            Some("cli".to_string()),
            Some("env".to_string()),
            Some("file".to_string()),
            "default".to_string(),
            "--flag",
            "ENV_VAR",
            "section.key",
        );
        assert_eq!(r.value, "cli");
        assert_eq!(r.source, ConfigSource::CliFlag);
    }

    #[test]
    fn resolve_opt_env_over_file() {
        let r: Resolved<String> = resolve_opt(
            None,
            Some("env".to_string()),
            Some("file".to_string()),
            "default".to_string(),
            "--flag",
            "ENV_VAR",
            "section.key",
        );
        assert_eq!(r.value, "env");
        assert_eq!(r.source, ConfigSource::EnvVar);
    }

    #[test]
    fn resolve_opt_file_over_default() {
        let r: Resolved<String> = resolve_opt(
            None,
            None,
            Some("file".to_string()),
            "default".to_string(),
            "--flag",
            "ENV_VAR",
            "section.key",
        );
        assert_eq!(r.value, "file");
        assert_eq!(r.source, ConfigSource::ConfigYml);
    }

    #[test]
    fn resolve_opt_default_fallback() {
        let r: Resolved<String> = resolve_opt(
            None,
            None,
            None,
            "default".to_string(),
            "--flag",
            "ENV_VAR",
            "section.key",
        );
        assert_eq!(r.value, "default");
        assert_eq!(r.source, ConfigSource::Default);
    }

    /// resolve_opt_none — attribution for optional keys.
    #[test]
    fn resolve_opt_none_all_missing() {
        let r: Resolved<Option<String>> = resolve_opt_none(
            None::<String>,
            None,
            None,
            "N/A",
            "ENV_VAR",
            "section.key",
            "my_key",
        );
        assert!(r.value.is_none());
        assert_eq!(r.source, ConfigSource::Default);
        assert!(r.attribution.contains("not set"));
    }

    #[test]
    fn resolve_opt_none_env_wins() {
        let r: Resolved<Option<String>> = resolve_opt_none(
            None::<String>,
            Some("from_env".to_string()),
            Some("from_file".to_string()),
            "N/A",
            "ENV_VAR",
            "section.key",
            "my_key",
        );
        assert_eq!(r.value, Some("from_env".to_string()));
        assert_eq!(r.source, ConfigSource::EnvVar);
    }

    /// Full config resolution produces attribution for every key.
    #[test]
    fn all_keys_have_attribution() {
        let cli = CliOverrides::default();
        let config = resolve(cli);

        // Spot-check a representative set of keys
        assert!(!config.bind_addr.attribution.is_empty());
        assert!(!config.allow_br_mismatch.attribution.is_empty());
        assert!(!config.agent_adapter.attribution.is_empty());
        assert!(!config.agent_model.attribution.is_empty());
        assert!(!config.ui_theme.attribution.is_empty());
        assert!(!config.metrics_enabled.attribution.is_empty());
        assert!(!config.metrics_port.attribution.is_empty());
        assert!(!config.audit_retention_days.attribution.is_empty());
        assert!(!config.audit_hash_chain.attribution.is_empty());
        assert!(!config.reflection_enabled.attribution.is_empty());
        assert!(!config.voice_hotkey.attribution.is_empty());
        assert!(!config.voice_max_recording_seconds.attribution.is_empty());
    }

    /// to_debug_map produces a serializable map.
    #[test]
    fn debug_map_is_serializable() {
        let cli = CliOverrides::default();
        let config = resolve(cli);
        let map = config.to_debug_map();
        let json = serde_json::to_string(&map).unwrap();
        assert!(!json.is_empty());

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        // Should contain key config entries
        assert!(parsed.get("bind_addr").is_some());
        assert!(parsed.get("agent_adapter").is_some());
    }

    /// Attribution strings follow the documented format.
    #[test]
    fn attribution_format_matches_spec() {
        let cli = CliOverrides {
            bind_addr: Some(parse_addr("10.0.0.1:8080")),
            ..Default::default()
        };
        let config = resolve(cli);

        // CLI: "cli flag --foo"
        assert_eq!(config.bind_addr.attribution, "cli flag --addr");

        // Default: "compiled default"
        assert_eq!(config.agent_adapter.attribution, "compiled default");
    }

    /// Env var parsing works for numeric types.
    #[test]
    fn env_var_numeric_parsing() {
        std::env::set_var("HOOP_ARCHIVE_DAYS", "60");

        let cli = CliOverrides::default();
        let config = resolve(cli);

        assert_eq!(config.ui_archive_after_days.value, 60);
        assert_eq!(config.ui_archive_after_days.source, ConfigSource::EnvVar);
        assert!(config
            .ui_archive_after_days
            .attribution
            .contains("HOOP_ARCHIVE_DAYS"));

        std::env::remove_var("HOOP_ARCHIVE_DAYS");
    }

    /// Boolean env var parsing (uses a key not touched by other tests to avoid
    /// parallel env-var races).
    #[test]
    fn env_var_boolean_parsing() {
        std::env::set_var("HOOP_AUDIT_HASH_CHAIN", "false");

        let cli = CliOverrides::default();
        let config = resolve(cli);

        assert!(!config.audit_hash_chain.value);
        assert_eq!(config.audit_hash_chain.source, ConfigSource::EnvVar);
        assert!(config
            .audit_hash_chain
            .attribution
            .contains("HOOP_AUDIT_HASH_CHAIN"));

        std::env::remove_var("HOOP_AUDIT_HASH_CHAIN");
    }

    /// YAML helper functions extract values correctly from parsed config.
    #[test]
    fn yaml_helpers_extract_values() {
        let yaml: serde_yaml::Value = serde_yaml::from_str(
            "server:\n  bind_addr: \"0.0.0.0:9999\"\nui:\n  theme: dark\n  archive_after_days: 60\nmetrics:\n  enabled: true\n  port: 8080\nagent:\n  cost_cap_per_session_usd: 5.5\n"
        ).unwrap();

        assert_eq!(
            yaml_get_str(&yaml, "server.bind_addr"),
            Some("0.0.0.0:9999")
        );
        assert_eq!(yaml_get_str(&yaml, "ui.theme"), Some("dark"));
        assert_eq!(yaml_get_u64(&yaml, "ui.archive_after_days"), Some(60));
        assert_eq!(yaml_get_bool(&yaml, "metrics.enabled"), Some(true));
        assert_eq!(yaml_get_u64(&yaml, "metrics.port"), Some(8080));
        assert_eq!(
            yaml_get_f64(&yaml, "agent.cost_cap_per_session_usd"),
            Some(5.5)
        );

        // Missing keys return None
        assert_eq!(yaml_get_str(&yaml, "nonexistent.key"), None);
        assert_eq!(yaml_get_u64(&yaml, "ui.theme"), None); // string, not u64
    }

    /// resolve_opt correctly propagates config.yml values when CLI and env are absent.
    /// This tests the config.yml > default precedence path.
    #[test]
    fn resolve_opt_config_yml_over_default() {
        let r: Resolved<u32> = resolve_opt(
            None,
            None,
            Some(60u32),
            30,
            "--archive-days",
            "HOOP_ARCHIVE_DAYS",
            "ui.archive_after_days",
        );
        assert_eq!(r.value, 60);
        assert_eq!(r.source, ConfigSource::ConfigYml);
        assert_eq!(r.attribution, "config.yml: ui.archive_after_days");
    }

    /// Config.yml loses to env var at the resolve_opt level.
    #[test]
    fn resolve_opt_env_beats_config_yml() {
        let r: Resolved<u16> = resolve_opt(
            None,
            Some(6060u16),
            Some(7777u16),
            9091,
            "N/A",
            "HOOP_METRICS_PORT",
            "metrics.port",
        );
        assert_eq!(r.value, 6060);
        assert_eq!(r.source, ConfigSource::EnvVar);
        assert!(r.attribution.contains("HOOP_METRICS_PORT"));
    }

    /// Debug map has the expected structure: each key has value, source, resolved_from.
    #[test]
    fn debug_map_structure() {
        let cli = CliOverrides::default();
        let config = resolve(cli);
        let map = config.to_debug_map();

        // Verify every entry has the expected sub-keys
        for (key, val) in &map {
            let obj = val.as_object().unwrap_or_else(|| {
                panic!(
                    "key '{}' should be an object with value/source/resolved_from",
                    key
                )
            });
            assert!(obj.contains_key("value"), "key '{}' missing 'value'", key);
            assert!(obj.contains_key("source"), "key '{}' missing 'source'", key);
            assert!(
                obj.contains_key("resolved_from"),
                "key '{}' missing 'resolved_from'",
                key
            );

            // source should be one of the four valid values
            let source = obj["source"]
                .as_str()
                .unwrap_or_else(|| panic!("key '{}' source should be a string", key));
            assert!(
                ["cli_flag", "env_var", "config_yml", "default"].contains(&source),
                "key '{}' has invalid source: {}",
                key,
                source
            );

            // resolved_from should be a non-empty string
            let resolved_from = obj["resolved_from"]
                .as_str()
                .unwrap_or_else(|| panic!("key '{}' resolved_from should be a string", key));
            assert!(
                !resolved_from.is_empty(),
                "key '{}' has empty resolved_from",
                key
            );
        }
    }
}
