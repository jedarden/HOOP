//! `/api/config` endpoint — running daemon config for diff (§17.4)
//!
//! Returns the current running config values for comparison with config.yml.
//! The CLI's `hoop config diff` command queries this endpoint to show
//! which values would change and which require restart.

use axum::{extract::State, response::Json, routing::get, Router};
use serde::{Deserialize, Serialize};
use crate::config_resolver::SecretPattern;
use crate::DaemonState;

/// Response for GET /api/config
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ConfigResponse {
    /// Schema version for compatibility tracking
    pub schema_version: String,
    /// Running config values
    pub config: RunningConfig,
}

/// Subset of running config values relevant for diff (restart-required keys first)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RunningConfig {
    /// Server bind address (RESTART REQUIRED)
    pub server_bind_addr: String,
    /// Agent adapter
    pub agent_adapter: String,
    /// Agent model
    pub agent_model: String,
    /// Projects file path
    pub projects_file: String,
    /// UI theme
    pub ui_theme: String,
    /// Metrics enabled
    pub metrics_enabled: bool,
    /// Metrics port (RESTART REQUIRED when changed with enabled=true)
    pub metrics_port: u16,
    /// Voice hotkey
    pub voice_hotkey: String,
    /// Agent extensions: skills directory
    pub agent_extensions_skills: Option<String>,
    /// Agent extensions: scripts directory
    pub agent_extensions_scripts: Option<String>,
    /// Agent extensions: notes directory
    pub agent_extensions_notes: Option<String>,
    /// Agent extensions: prompts directory
    pub agent_extensions_prompts: Option<String>,
    /// Audit retention days
    pub audit_retention_days: u32,
    /// Audit hash chain enabled
    pub audit_hash_chain: bool,
    /// Reflection enabled
    pub reflection_enabled: bool,
    /// Reflection detection threshold
    pub reflection_detection_threshold: f64,
    /// Reflection auto-archive after days
    pub reflection_auto_archive_after_days: u32,
    /// Backup: S3 endpoint
    pub backup_endpoint: Option<String>,
    /// Backup: S3 bucket
    pub backup_bucket: Option<String>,
    /// Backup: S3 key prefix
    pub backup_prefix: Option<String>,
    /// Backup: cron schedule
    pub backup_schedule: String,
    /// Backup: retention days
    pub backup_retention_days: u32,
    /// Backup: encryption enabled
    pub backup_encryption: bool,
    /// Pricing: file path
    pub pricing_file: String,
    /// UI: archive after days
    pub ui_archive_after_days: u32,
}

/// Config keys that require daemon restart to take effect (§17.4)
pub const RESTART_REQUIRED_KEYS: &[&str] = &["server.bind_addr", "metrics.port"];

/// Check if a config key requires restart
pub fn is_restart_required_key(key: &str) -> bool {
    RESTART_REQUIRED_KEYS.contains(&key)
}

/// Response for GET /api/config/secrets-patterns (§18)
///
/// Exposes the current secret scanning patterns to the client for pre-upload
/// warning. This ensures client and backend use the same pattern set.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SecretsPatternsResponse {
    /// Schema version for compatibility tracking
    pub schema_version: String,
    /// Secret patterns from config.yml or defaults
    pub patterns: Vec<SecretPattern>,
}

/// GET /api/config
///
/// Returns the current running config values for comparison with config.yml.
/// The CLI's `hoop config diff` command queries this endpoint to show
/// which values would change and which require restart.
#[utoipa::path(
    get,
    path = "/api/config",
    tag = "config",
    responses(
        (status = 200, description = "Running config returned successfully", body = ConfigResponse)
    )
)]
async fn get_config(State(state): State<DaemonState>) -> Json<ConfigResponse> {
    let cfg = &*state.resolved_config;

    Json(ConfigResponse {
        schema_version: "1.0.0".to_string(),
        config: RunningConfig {
            server_bind_addr: cfg.bind_addr.value.clone(),
            agent_adapter: cfg.agent_adapter.value.clone(),
            agent_model: cfg.agent_model.value.clone(),
            projects_file: cfg.projects_file.value.clone(),
            ui_theme: cfg.ui_theme.value.clone(),
            metrics_enabled: cfg.metrics_enabled.value,
            metrics_port: cfg.metrics_port.value,
            voice_hotkey: cfg.voice_hotkey.value.clone(),
            agent_extensions_skills: cfg.agent_extensions_skills.value.clone(),
            agent_extensions_scripts: cfg.agent_extensions_scripts.value.clone(),
            agent_extensions_notes: cfg.agent_extensions_notes.value.clone(),
            agent_extensions_prompts: cfg.agent_extensions_prompts.value.clone(),
            audit_retention_days: cfg.audit_retention_days.value,
            audit_hash_chain: cfg.audit_hash_chain.value,
            reflection_enabled: cfg.reflection_enabled.value,
            reflection_detection_threshold: cfg.reflection_detection_threshold.value,
            reflection_auto_archive_after_days: cfg.reflection_auto_archive_after_days.value,
            backup_endpoint: cfg.backup_endpoint.value.clone(),
            backup_bucket: cfg.backup_bucket.value.clone(),
            backup_prefix: cfg.backup_prefix.value.clone(),
            backup_schedule: cfg.backup_schedule.value.clone(),
            backup_retention_days: cfg.backup_retention_days.value,
            backup_encryption: cfg.backup_encryption.value,
            pricing_file: cfg.pricing_file.value.clone(),
            ui_archive_after_days: cfg.ui_archive_after_days.value,
        },
    })
}

/// GET /api/config/secrets-patterns
///
/// Returns the current secret scanning patterns for client-side pre-upload
/// warning (§18). Ensures parity between client warning and backend blocking.
#[utoipa::path(
    get,
    path = "/api/config/secrets-patterns",
    tag = "config",
    responses(
        (status = 200, description = "Secret patterns returned successfully", body = SecretsPatternsResponse)
    )
)]
async fn get_secrets_patterns(State(state): State<DaemonState>) -> Json<SecretsPatternsResponse> {
    let cfg = &*state.resolved_config;

    Json(SecretsPatternsResponse {
        schema_version: "1.0.0".to_string(),
        patterns: cfg.secrets_patterns.value.clone(),
    })
}

pub fn router() -> Router<DaemonState> {
    Router::new()
        .route("/api/config", get(get_config))
        .route("/api/config/secrets-patterns", get(get_secrets_patterns))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restart_required_keys() {
        assert!(is_restart_required_key("server.bind_addr"));
        assert!(is_restart_required_key("metrics.port"));
        assert!(!is_restart_required_key("agent.adapter"));
        assert!(!is_restart_required_key("ui.theme"));
    }
}
