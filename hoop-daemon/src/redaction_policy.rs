//! Per-project redaction policy resolver (§18.5)
//!
//! Resolves the effective redaction policy for a project.
//!
//! ## Precedence
//!
//! 1. Global `redaction:` block in config.yml (TODO: not yet implemented in schema)
//! 2. Built-in defaults (if no global policy is configured)
//!
//! ## Actions
//!
//! - `warn`: Log findings but allow the operation (default)
//! - `redact`: Replace secrets with [REDACTED] in stored content
//! - `reject`: Block the operation entirely (e.g., attachment upload)

use anyhow::Result;
use hoop_schema::{HoopConfig, ProjectsRegistry, ProjectsRegistryProjectsItem};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared redaction policy state, updated by config hot-reload.
#[derive(Debug)]
pub struct RedactionPolicyState {
    /// Global redaction policy from config.yml (may be None if not configured)
    global_policy: Option<GlobalRedactionPolicy>,
    /// Projects registry (kept for future per-project policy support)
    _projects_registry: Arc<RwLock<ProjectsRegistry>>,
}

impl Clone for RedactionPolicyState {
    fn clone(&self) -> Self {
        Self {
            global_policy: self.global_policy.clone(),
            _projects_registry: self._projects_registry.clone(),
        }
    }
}

/// Global redaction policy from config.yml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalRedactionPolicy {
    /// Action to take when secrets are detected
    pub action: RedactionAction,
    /// Pattern sets to enable (empty = all default patterns)
    #[serde(default)]
    pub patterns: Vec<String>,
}

/// Action to take when secrets are detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RedactionAction {
    /// Log findings but allow the operation
    Warn,
    /// Replace secrets with [REDACTED]
    Redact,
    /// Block the operation entirely
    Reject,
    /// Flag only (record audit entry but take no action)
    FlaggedOnly,
}

impl Default for RedactionAction {
    fn default() -> Self {
        Self::Warn
    }
}

/// Resolved redaction policy for a specific project.
///
/// Combines global defaults with per-project overrides.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedRedactionPolicy {
    /// Effective action for this project
    pub action: RedactionAction,
    /// Enabled pattern names for this project
    pub patterns: HashSet<String>,
    /// Source of the policy ("global" or "built-in default")
    pub source: String,
}

impl Default for ResolvedRedactionPolicy {
    fn default() -> Self {
        Self {
            action: RedactionAction::Warn,
            patterns: default_pattern_names(),
            source: "built-in default".to_string(),
        }
    }
}

/// Default pattern names when no policy is configured.
fn default_pattern_names() -> HashSet<String> {
    [
        "anthropic_api_key",
        "generic_sk_key",
        "aws_access_key",
        "github_token",
        "slack_token",
        "jwt",
        "bearer_token",
        "env_var_secret",
        "json_secret_field",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// Extract redaction policy from a ProjectsRegistryProjectsItem.
///
/// Returns None if no redaction override is configured for the project.
/// NOTE: Per-project redaction policy is not yet supported in the schema.
fn extract_redaction_from_project(_project: &ProjectsRegistryProjectsItem) -> Option<ResolvedRedactionPolicy> {
    // TODO: Implement per-project redaction policy when added to schema
    None
}

/// Map a high-level pattern name to its implementation-specific pattern names.
///
/// High-level pattern names (from the schema) map to one or more specific
/// pattern names used in the redaction implementation.
///
/// # Examples
///
/// - "github_token" → ["github_token_ghp", "github_token_ghs", "github_token_ghu", "github_pat"]
/// - "slack_token" → ["slack_bot_token", "slack_user_token"]
/// - "anthropic_api_key" → ["anthropic_api_key"]
fn map_pattern_to_impl_names(pattern_name: &str) -> Vec<&'static str> {
    match pattern_name {
        "anthropic_api_key" => vec!["anthropic_api_key"],
        "generic_sk_key" => vec!["generic_sk_key"],
        "aws_access_key" => vec!["aws_access_key"],
        "github_token" => vec![
            "github_token_ghp",
            "github_token_ghs",
            "github_token_ghu",
            "github_pat",
        ],
        "slack_token" => vec!["slack_bot_token", "slack_user_token"],
        "jwt" => vec!["jwt"],
        "bearer_token" => vec!["bearer_token"],
        "env_var_secret" => vec!["env_var_secret"],
        "json_secret_field" => vec!["json_secret_field"],
        _ => vec![],
    }
}

/// Check if a finding's pattern name matches any of the enabled patterns.
///
/// This handles the mapping between high-level pattern names (from the schema)
/// and implementation-specific pattern names (from the redaction engine).
fn finding_matches_enabled_pattern(
    finding_pattern_name: &str,
    enabled_patterns: &HashSet<String>,
) -> bool {
    // Direct match
    if enabled_patterns.contains(finding_pattern_name) {
        return true;
    }

    // Check if any high-level pattern maps to this finding pattern
    for high_level_pattern in enabled_patterns {
        let impl_names = map_pattern_to_impl_names(high_level_pattern);
        if impl_names.contains(&finding_pattern_name) {
            return true;
        }
    }

    false
}

impl RedactionPolicyState {
    /// Create a new policy state from the current config.
    pub fn new(_global_config: &HoopConfig, projects_registry: ProjectsRegistry) -> Self {
        // TODO: Parse global redaction policy from config when redaction field is added to HoopConfig
        let global_policy = None;

        Self {
            global_policy,
            _projects_registry: Arc::new(RwLock::new(projects_registry)),
        }
    }

    /// Update the projects registry (called on hot-reload).
    pub async fn update_projects(&self, registry: ProjectsRegistry) {
        *self._projects_registry.write().await = registry;
    }

    /// Resolve the redaction policy for a specific project.
    ///
    /// Returns the effective policy by checking:
    /// 1. Global policy in config.yml (TODO: not yet implemented in schema)
    /// 2. Built-in defaults
    pub async fn resolve_for_project(&self, _project_name: &str) -> ResolvedRedactionPolicy {
        // Fall back to global policy
        if let Some(global) = &self.global_policy {
            let patterns = if global.patterns.is_empty() {
                default_pattern_names()
            } else {
                global.patterns.iter().cloned().collect()
            };

            return ResolvedRedactionPolicy {
                action: global.action,
                patterns,
                source: "global".to_string(),
            };
        }

        // Built-in defaults
        ResolvedRedactionPolicy::default()
    }

    /// Check if a pattern name is enabled for a project.
    pub async fn is_pattern_enabled(&self, project_name: &str, pattern_name: &str) -> bool {
        let policy = self.resolve_for_project(project_name).await;
        policy.patterns.contains(pattern_name)
    }

    /// Get the action for a project.
    pub async fn get_action(&self, project_name: &str) -> RedactionAction {
        let policy = self.resolve_for_project(project_name).await;
        policy.action
    }
}

/// Error type for redaction policy violations.
#[derive(Debug, Clone)]
pub struct RedactionRejectedError {
    /// Project name
    pub project: String,
    /// Pattern that was detected
    pub pattern: String,
    /// Number of findings
    pub count: usize,
}

impl std::fmt::Display for RedactionRejectedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Redaction policy rejected: project '{}' detected {} {} pattern(s)",
            self.project, self.count, self.pattern
        )
    }
}

impl std::error::Error for RedactionRejectedError {}

/// Check if content should be rejected based on the project's redaction policy.
///
/// Returns `Ok(())` if the content is allowed, or `Err(RedactionRejectedError)` if
/// the project's policy is set to `reject` and secrets are detected.
pub async fn check_reject_policy(
    state: &RedactionPolicyState,
    project_name: &str,
    content: &str,
) -> Result<(), RedactionRejectedError> {
    let policy = state.resolve_for_project(project_name).await;

    if policy.action != RedactionAction::Reject {
        return Ok(());
    }

    // Scan for secrets using only enabled patterns
    let findings = crate::redaction::scan_text_for_secrets(content);
    let filtered_findings: Vec<_> = findings
        .into_iter()
        .filter(|f| finding_matches_enabled_pattern(f.pattern_name, &policy.patterns))
        .collect();

    if !filtered_findings.is_empty() {
        // Group by high-level pattern name for reporting
        let mut high_level_pattern_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for finding in &filtered_findings {
            // Find the high-level pattern that maps to this finding
            let mut found_high_level = None;
            for high_level_pattern in &policy.patterns {
                let impl_names = map_pattern_to_impl_names(high_level_pattern);
                if impl_names.contains(&finding.pattern_name) {
                    found_high_level = Some(high_level_pattern.clone());
                    break;
                }
            }

            // Use the high-level pattern name if found, otherwise use the finding's pattern name
            let pattern_name = found_high_level.unwrap_or_else(|| finding.pattern_name.to_string());
            *high_level_pattern_counts.entry(pattern_name).or_insert(0) += 1;
        }

        // Return error for the first detected pattern
        if let Some((pattern, count)) = high_level_pattern_counts.into_iter().next() {
            return Err(RedactionRejectedError {
                project: project_name.to_string(),
                pattern,
                count,
            });
        }
    }

    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use hoop_schema::{HoopConfig, HoopConfigSchemaVersion, ProjectsRegistry};

    #[test]
    fn test_default_policy_returns_defaults() {
        let config = HoopConfig {
            schema_version: HoopConfigSchemaVersion::default(),
            agent: None,
            agent_extensions: None,
            audit: None,
            backup: None,
            metrics: None,
            pricing: None,
            projects_file: None,
            reflection: None,
            stuck_detector: None,
            morning_brief: None,
            roles: None,
            server: None,
            ui: None,
            voice: None,
        };
        let projects = ProjectsRegistry::default();
        let state = RedactionPolicyState::new(&config, projects);

        let rt = tokio::runtime::Runtime::new().unwrap();
        let policy = rt.block_on(state.resolve_for_project("test-project"));

        assert_eq!(policy.action, RedactionAction::Warn);
        assert_eq!(policy.patterns.len(), 9); // all default patterns
        assert_eq!(policy.source, "built-in default");
    }
}
