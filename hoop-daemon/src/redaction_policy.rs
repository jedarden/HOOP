//! Per-project redaction policy resolver (§18.5)
//!
//! Resolves the effective redaction policy for a project.
//!
//! ## Precedence
//!
//! 1. Global `redaction:` block in config.yml
//! 2. Per-project override in projects.yaml
//! 3. Built-in defaults (if no policy is configured)
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
fn extract_redaction_from_project(project: &ProjectsRegistryProjectsItem) -> Option<ResolvedRedactionPolicy> {
    match project {
        hoop_schema::ProjectsRegistryProjectsItem::Variant0 { redaction, name, .. } => {
            redaction.as_ref().map(|r| {
                let patterns = convert_patterns(&r.patterns);
                ResolvedRedactionPolicy {
                    action: convert_action(&r.action),
                    patterns,
                    source: format!("project:{}", name),
                }
            })
        }
        hoop_schema::ProjectsRegistryProjectsItem::Variant1 { redaction, name, .. } => {
            redaction.as_ref().map(|r| {
                let patterns = convert_patterns_variant1(&r.patterns);
                ResolvedRedactionPolicy {
                    action: convert_action_variant1(&r.action),
                    patterns,
                    source: format!("project:{}", name),
                }
            })
        }
    }
}

/// Convert schema redaction action to internal RedactionAction.
fn convert_action(action: &hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionAction) -> RedactionAction {
    match action {
        hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionAction::Warn => RedactionAction::Warn,
        hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionAction::Redact => RedactionAction::Redact,
        hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionAction::Reject => RedactionAction::Reject,
    }
}

/// Convert schema redaction action from Variant1.
fn convert_action_variant1(action: &hoop_schema::ProjectsRegistryProjectsItemVariant1RedactionAction) -> RedactionAction {
    match action {
        hoop_schema::ProjectsRegistryProjectsItemVariant1RedactionAction::Warn => RedactionAction::Warn,
        hoop_schema::ProjectsRegistryProjectsItemVariant1RedactionAction::Redact => RedactionAction::Redact,
        hoop_schema::ProjectsRegistryProjectsItemVariant1RedactionAction::Reject => RedactionAction::Reject,
    }
}

/// Convert schema pattern enums to string pattern names.
fn convert_patterns(
    patterns: &[hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionPatternsItem],
) -> HashSet<String> {
    patterns
        .iter()
        .map(|p| match p {
            hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionPatternsItem::AnthropicApiKey => "anthropic_api_key",
            hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionPatternsItem::GenericSkKey => "generic_sk_key",
            hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionPatternsItem::AwsAccessKey => "aws_access_key",
            hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionPatternsItem::GithubToken => "github_token",
            hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionPatternsItem::SlackToken => "slack_token",
            hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionPatternsItem::Jwt => "jwt",
            hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionPatternsItem::BearerToken => "bearer_token",
            hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionPatternsItem::EnvVarSecret => "env_var_secret",
            hoop_schema::ProjectsRegistryProjectsItemVariant0RedactionPatternsItem::JsonSecretField => "json_secret_field",
        })
        .map(|s| s.to_string())
        .collect()
}

/// Convert schema pattern enums from Variant1.
fn convert_patterns_variant1(
    patterns: &[hoop_schema::ProjectsRegistryProjectsItemVariant1RedactionPatternsItem],
) -> HashSet<String> {
    patterns
        .iter()
        .map(|p| match p {
            hoop_schema::ProjectsRegistryProjectsItemVariant1RedactionPatternsItem::AnthropicApiKey => "anthropic_api_key",
            hoop_schema::ProjectsRegistryProjectsItemVariant1RedactionPatternsItem::GenericSkKey => "generic_sk_key",
            hoop_schema::ProjectsRegistryProjectsItemVariant1RedactionPatternsItem::AwsAccessKey => "aws_access_key",
            hoop_schema::ProjectsRegistryProjectsItemVariant1RedactionPatternsItem::GithubToken => "github_token",
            hoop_schema::ProjectsRegistryProjectsItemVariant1RedactionPatternsItem::SlackToken => "slack_token",
            hoop_schema::ProjectsRegistryProjectsItemVariant1RedactionPatternsItem::Jwt => "jwt",
            hoop_schema::ProjectsRegistryProjectsItemVariant1RedactionPatternsItem::BearerToken => "bearer_token",
            hoop_schema::ProjectsRegistryProjectsItemVariant1RedactionPatternsItem::EnvVarSecret => "env_var_secret",
            hoop_schema::ProjectsRegistryProjectsItemVariant1RedactionPatternsItem::JsonSecretField => "json_secret_field",
        })
        .map(|s| s.to_string())
        .collect()
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
    pub fn new(global_config: &HoopConfig, projects_registry: ProjectsRegistry) -> Self {
        // Parse global redaction policy from config if present
        let global_policy = global_config.redaction.as_ref().map(|config_redaction| {
            let action = match config_redaction.action {
                hoop_schema::HoopConfigRedactionAction::Warn => RedactionAction::Warn,
                hoop_schema::HoopConfigRedactionAction::Redact => RedactionAction::Redact,
                hoop_schema::HoopConfigRedactionAction::Reject => RedactionAction::Reject,
            };
            // Convert Vec<HoopConfigRedactionPatternsItem> to Vec<String>
            let patterns = config_redaction.patterns.iter().map(|p| p.to_string()).collect();
            GlobalRedactionPolicy {
                action,
                patterns,
            }
        });

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
    /// 1. Per-project override in projects.yaml
    /// 2. Global policy in config.yml
    /// 3. Built-in defaults
    pub async fn resolve_for_project(&self, project_name: &str) -> ResolvedRedactionPolicy {
        // Check for per-project override first
        let registry = self._projects_registry.read().await;
        if let Some(project) = registry
            .projects
            .iter()
            .find(|p| p.name() == project_name)
        {
            if let Some(policy) = extract_redaction_from_project(project) {
                return policy;
            }
        }

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

    /// Find the project name that contains a given workspace path.
    ///
    /// Returns `None` if the path doesn't match any registered project workspace.
    /// This handles both canonical path matching and prefix matching (for subdirectories).
    pub async fn find_project_by_workspace(&self, workspace_path: &std::path::Path) -> Option<String> {
        let registry = self._projects_registry.read().await;

        // First, try to canonicalize the input path for comparison
        let canon_input = std::fs::canonicalize(workspace_path).ok();

        for project in &registry.projects {
            for ws_view in project.workspace_views() {
                // Try canonical path first if available
                let canon_ws: Option<std::path::PathBuf> = ws_view.canonical_path.as_ref()
                    .map(|p| std::path::PathBuf::from(p))
                    .or_else(|| std::fs::canonicalize(&ws_view.path).ok());

                if let Some(ref canon_ws_path) = canon_ws {
                    if let Some(ref canon_input_path) = canon_input {
                        // Exact match
                        if canon_ws_path == canon_input_path {
                            return Some(project.name().to_string());
                        }
                        // Prefix match (workspace is a parent of the input path)
                        if canon_input_path.starts_with(canon_ws_path) {
                            return Some(project.name().to_string());
                        }
                    }
                }

                // Fallback: try direct path comparison
                if workspace_path.starts_with(&ws_view.path) {
                    return Some(project.name().to_string());
                }
            }
        }

        None
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

/// Scan content for secrets and write audit entries.
///
/// Returns `Ok(Vec<SecretFinding>)` with all findings (already written to audit),
/// or an error if the scan fails. This function writes audit entries for each
/// unique pattern detected, regardless of the project's action policy.
///
/// Use this function when you want to:
/// 1. Detect secrets in content
/// 2. Record what was flagged in the audit log
/// 3. Optionally take action based on the project's policy
///
/// # Arguments
/// * `state` - Redaction policy state
/// * `project_name` - Project name for policy resolution
/// * `content` - Content to scan
/// * `what_flagged` - Surface being scanned (e.g., "attachment", "transcript")
/// * `source_ref` - Reference to the source (attachment_id, stitch_id, etc.)
/// * `operator` - Operator who triggered the scan (or "system")
///
/// # Returns
/// All findings that were detected and written to audit
pub async fn scan_and_audit(
    state: &RedactionPolicyState,
    project_name: &str,
    content: &str,
    what_flagged: &str,
    source_ref: &str,
    operator: &str,
) -> Result<Vec<crate::redaction::SecretFinding>, RedactionRejectedError> {
    let policy = state.resolve_for_project(project_name).await;

    // Scan for secrets using only enabled patterns
    let findings = crate::redaction::scan_text_for_secrets(content);
    let filtered_findings: Vec<_> = findings
        .into_iter()
        .filter(|f| finding_matches_enabled_pattern(f.pattern_name, &policy.patterns))
        .collect();

    // Write audit entries for each unique pattern, recording the policy action
    crate::redaction::audit_findings(
        what_flagged,
        &filtered_findings,
        policy.action,
        source_ref,
        Some(project_name),
        operator,
    );

    // If policy is Reject and secrets were found, return an error
    if policy.action == RedactionAction::Reject && !filtered_findings.is_empty() {
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

    Ok(filtered_findings)
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
