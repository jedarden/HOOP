//! `/api/onboarding` endpoint — progressive capability introduction
//!
//! Provides "what's new" and feature discovery prompts that fire once per operator
//! unless dismissed. Respects the global "don't bug me" setting.
//!
//! Plan reference: §12 Onboarding (progressive introduction)

use axum::{
    extract::{ConnectInfo, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use crate::DaemonState;
use crate::fleet;

/// Current HOOP version (for "what's new" detection)
const HOOP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Prompt types for progressive introduction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum OnboardingPromptType {
    /// "What's new" card when upgrading to a new version
    WhatsNew { version: String },
    /// Suggest proposing rules when Reflection Ledger is empty after 30 days
    ReflectionLedgerEmpty,
    /// Suggest creating a Pattern when 10+ Stitches share a theme
    PatternSuggestion { theme: String, stitch_count: usize },
    /// Agent never used — inline prompt on chat pane
    AgentIntro,
    /// Mic never used — prompt near hotkey icon
    MicIntro,
}

/// Onboarding prompt data sent to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingPrompt {
    /// Unique prompt identifier
    pub id: String,
    /// Prompt type
    #[serde(flatten)]
    pub prompt_type: OnboardingPromptType,
    /// Human-readable title
    pub title: String,
    /// Detailed message (markdown supported)
    pub message: String,
    /// Optional action button label
    pub action_label: Option<String>,
    /// Optional action URL
    pub action_url: Option<String>,
    /// When this prompt becomes eligible
    pub eligible_at: String,
    /// ISO 8601 timestamp when dismissed (null if not dismissed)
    pub dismissed_at: Option<String>,
    /// Priority (higher = shown first)
    pub priority: i32,
}

/// Response for GET /api/onboarding/prompts
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct OnboardingPromptsResponse {
    /// Eligible prompts that haven't been dismissed
    pub prompts: Vec<OnboardingPrompt>,
    /// Whether prompts are globally enabled
    pub prompts_enabled: bool,
    /// Current HOOP version
    pub hoop_version: String,
    /// Last seen version by this operator
    pub last_seen_version: Option<String>,
}

/// Request to dismiss a prompt
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DismissPromptRequest {
    /// Prompt ID to dismiss
    pub prompt_id: String,
}

/// Request to record feature usage
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct RecordFeatureUsageRequest {
    /// Feature name (agent, mic, patterns, reflection_ledger)
    pub feature: String,
}

/// Query parameters for GET /api/onboarding/prompts
#[derive(Debug, Clone, Deserialize)]
pub struct OnboardingPromptsQuery {
    /// Filter by prompt type (optional)
    #[serde(rename = "type")]
    pub prompt_type: Option<String>,
    /// Include dismissed prompts (default: false)
    pub include_dismissed: Option<bool>,
}

/// GET /api/onboarding/prompts — list eligible prompts for the current operator
///
/// Returns prompts that the operator is eligible to see, excluding dismissed
/// prompts unless include_dismissed=true.
#[utoipa::path(
    get,
    path = "/api/onboarding/prompts",
    tag = "onboarding",
    params(
        ("type" = Option<String>, Query, description = "Filter by prompt type"),
        ("include_dismissed" = Option<bool>, Query, description = "Include dismissed prompts")
    ),
    responses(
        (status = 200, description = "Eligible prompts", body = OnboardingPromptsResponse),
        (status = 500, description = "Internal server error"),
    )
)]
async fn list_onboarding_prompts(
    State(state): State<DaemonState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Query(query): Query<OnboardingPromptsQuery>,
) -> Result<Json<OnboardingPromptsResponse>, StatusCode> {
    let operator_id = state.identity_cache.resolve(connect_info.map(|ci| ci.0));

    // Get current UI state
    let conn = Connection::open(fleet::db_path())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut stmt = conn
        .prepare("SELECT key, value FROM ui_state WHERE operator_id = ?1")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut ui_state: HashMap<String, String> = HashMap::new();
    let mut rows = stmt
        .query_map((&operator_id,), |row| {
            let key: String = row.get(0)?;
            let value: String = row.get(1)?;
            Ok((key, value))
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    while let Some(row) = rows.next() {
        let (key, value) = row.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        ui_state.insert(key, value);
    }
    drop(stmt);

    // Check if prompts are globally enabled
    let prompts_enabled: bool = ui_state
        .get("prompts_enabled")
        .and_then(|v| v.parse().ok())
        .unwrap_or(true);

    // Get dismissed prompts
    let dismissed_prompts: HashMap<String, String> = ui_state
        .get("prompts_dismissed")
        .and_then(|v| serde_json::from_str(v).ok())
        .unwrap_or_default();

    // Get last seen version
    let last_seen_version = ui_state.get("last_seen_version").cloned();

    // Get feature usage timestamps
    let feature_usage: HashMap<String, Option<String>> = [
        ("agent_first_used", ui_state.get("agent_first_used").cloned()),
        ("mic_first_used", ui_state.get("mic_first_used").cloned()),
        ("patterns_first_used", ui_state.get("patterns_first_used").cloned()),
        ("reflection_ledger_first_used", ui_state.get("reflection_ledger_first_used").cloned()),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect();

    let mut eligible_prompts = Vec::new();

    // Only check prompts if globally enabled
    if prompts_enabled {
        // Check each prompt type
        eligible_prompts.extend(check_whats_new_prompts(&dismissed_prompts, &last_seen_version));
        eligible_prompts.extend(check_reflection_ledger_prompt(&dismissed_prompts, &feature_usage).await);
        eligible_prompts.extend(check_pattern_suggestion_prompt(&dismissed_prompts, &feature_usage).await);
        eligible_prompts.extend(check_agent_intro_prompt(&dismissed_prompts, &feature_usage));
        eligible_prompts.extend(check_mic_intro_prompt(&dismissed_prompts, &feature_usage));
    }

    // Filter by type if requested
    let filtered = if let Some(ref pt) = query.prompt_type {
        eligible_prompts
            .into_iter()
            .filter(|p| {
                match &p.prompt_type {
                    OnboardingPromptType::WhatsNew { .. } => pt == "whats_new",
                    OnboardingPromptType::ReflectionLedgerEmpty => pt == "reflection_ledger_empty",
                    OnboardingPromptType::PatternSuggestion { .. } => pt == "pattern_suggestion",
                    OnboardingPromptType::AgentIntro => pt == "agent_intro",
                    OnboardingPromptType::MicIntro => pt == "mic_intro",
                }
            })
            .collect()
    } else {
        eligible_prompts
    };

    // Exclude dismissed prompts unless requested
    let final_prompts = if query.include_dismissed.unwrap_or(false) {
        filtered
    } else {
        filtered
            .into_iter()
            .filter(|p| p.dismissed_at.is_none())
            .collect()
    };

    // Sort by priority (descending)
    let mut sorted_prompts = final_prompts;
    sorted_prompts.sort_by(|a, b| b.priority.cmp(&a.priority));

    Ok(Json(OnboardingPromptsResponse {
        prompts: sorted_prompts,
        prompts_enabled,
        hoop_version: HOOP_VERSION.to_string(),
        last_seen_version,
    }))
}

/// POST /api/onboarding/dismiss — dismiss a prompt
///
/// Marks a prompt as dismissed so it won't be shown again.
#[utoipa::path(
    post,
    path = "/api/onboarding/dismiss",
    tag = "onboarding",
    request_body = DismissPromptRequest,
    responses(
        (status = 200, description = "Prompt dismissed successfully"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error"),
    )
)]
async fn dismiss_onboarding_prompt(
    State(state): State<DaemonState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Json(req): Json<DismissPromptRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let operator_id = state.identity_cache.resolve(connect_info.map(|ci| ci.0));

    let conn = Connection::open(fleet::db_path())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get current dismissed prompts
    let dismissed: HashMap<String, String> = conn
        .query_row(
            "SELECT value FROM ui_state WHERE operator_id = ?1 AND key = 'prompts_dismissed'",
            (&operator_id,),
            |row| {
                let value: String = row.get(0)?;
                serde_json::from_str::<HashMap<String, String>>(&value)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
            },
        )
        .ok()
        .unwrap_or_default();

    // Add/update the dismissed prompt
    let mut updated = dismissed;
    updated.insert(req.prompt_id.clone(), Utc::now().to_rfc3339());

    // Save back to database
    conn.execute(
        "INSERT INTO ui_state (operator_id, key, value, updated_at)
         VALUES (?1, 'prompts_dismissed', ?2, datetime('now'))
         ON CONFLICT (operator_id, key) DO UPDATE SET
             value = excluded.value,
             updated_at = datetime('now')",
        (&operator_id, &serde_json::to_string(&updated).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    drop(conn);

    Ok(Json(serde_json::json!({
        "success": true,
        "prompt_id": req.prompt_id,
        "dismissed_at": updated.get(&req.prompt_id)
    })))
}

/// POST /api/onboarding/enable — globally enable/disable prompts
#[utoipa::path(
    post,
    path = "/api/onboarding/enable",
    tag = "onboarding",
    responses(
        (status = 200, description = "Prompts enabled/disabled successfully"),
        (status = 500, description = "Internal server error")
    ),
)]
async fn set_onboarding_enabled(
    State(state): State<DaemonState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Json(enabled): Json<bool>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let operator_id = state.identity_cache.resolve(connect_info.map(|ci| ci.0));

    let conn = Connection::open(fleet::db_path())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    conn.execute(
        "INSERT INTO ui_state (operator_id, key, value, updated_at)
         VALUES (?1, 'prompts_enabled', ?2, datetime('now'))
         ON CONFLICT (operator_id, key) DO UPDATE SET
             value = excluded.value,
             updated_at = datetime('now')",
        (&operator_id, &if enabled { "true" } else { "false" }),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    drop(conn);

    Ok(Json(serde_json::json!({
        "success": true,
        "prompts_enabled": enabled
    })))
}

/// POST /api/onboarding/record-usage — record first usage of a feature
///
/// Called when a feature is used for the first time to prevent future intro prompts.
#[utoipa::path(
    post,
    path = "/api/onboarding/record-usage",
    tag = "onboarding",
    request_body = RecordFeatureUsageRequest,
    responses(
        (status = 200, description = "Usage recorded successfully"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
)]
async fn record_feature_usage(
    State(state): State<DaemonState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Json(req): Json<RecordFeatureUsageRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let operator_id = state.identity_cache.resolve(connect_info.map(|ci| ci.0));

    // Validate feature name
    let key = match req.feature.as_str() {
        "agent" => "agent_first_used",
        "mic" => "mic_first_used",
        "patterns" => "patterns_first_used",
        "reflection_ledger" => "reflection_ledger_first_used",
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    let conn = Connection::open(fleet::db_path())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Only set if not already set
    conn.execute(
        "INSERT INTO ui_state (operator_id, key, value, updated_at)
         VALUES (?1, ?2, ?3, datetime('now'))
         ON CONFLICT (operator_id, key) DO UPDATE SET
             value = excluded.value
             WHERE excluded.value IS NULL",
        (&operator_id, &key, &Utc::now().to_rfc3339()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    drop(conn);

    Ok(Json(serde_json::json!({
        "success": true,
        "feature": req.feature
    })))
}

/// POST /api/onboarding/ack-version — acknowledge current version
///
/// Called by the frontend to mark the current version as "seen" after
/// displaying the what's new card.
#[utoipa::path(
    post,
    path = "/api/onboarding/ack-version",
    tag = "onboarding",
    responses(
        (status = 200, description = "Version acknowledged successfully"),
        (status = 500, description = "Internal server error")
    ),
)]
async fn acknowledge_version(
    State(state): State<DaemonState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let operator_id = state.identity_cache.resolve(connect_info.map(|ci| ci.0));

    let conn = Connection::open(fleet::db_path())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    conn.execute(
        "INSERT INTO ui_state (operator_id, key, value, updated_at)
         VALUES (?1, 'last_seen_version', ?2, datetime('now'))
         ON CONFLICT (operator_id, key) DO UPDATE SET
             value = excluded.value,
             updated_at = datetime('now')",
        (&operator_id, HOOP_VERSION),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    drop(conn);

    Ok(Json(serde_json::json!({
        "success": true,
        "version": HOOP_VERSION
    })))
}

/// Check for "What's new" prompts when version changes
fn check_whats_new_prompts(
    dismissed: &HashMap<String, String>,
    last_seen_version: &Option<String>,
) -> Vec<OnboardingPrompt> {
    let mut prompts = Vec::new();

    if let Some(ref last_ver) = last_seen_version {
        // Only show if we've upgraded
        if last_ver != HOOP_VERSION {
            let prompt_id = format!("whats_new_{}", HOOP_VERSION.replace('.', "_"));

            if !dismissed.contains_key(&prompt_id) {
                prompts.push(OnboardingPrompt {
                    id: prompt_id,
                    prompt_type: OnboardingPromptType::WhatsNew {
                        version: HOOP_VERSION.to_string(),
                    },
                    title: format!("What's new in HOOP {}", HOOP_VERSION),
                    message: "HOOP has been updated with new features and improvements.".to_string(),
                    action_label: Some("View Release Notes".to_string()),
                    action_url: Some("/RELEASE_NOTES_v1.0.md".to_string()),
                    eligible_at: Utc::now().to_rfc3339(),
                    dismissed_at: dismissed.get(&format!("whats_new_{}", HOOP_VERSION.replace('.', "_"))).cloned(),
                    priority: 100,
                });
            }
        }
    }

    prompts
}

/// Check for Reflection Ledger empty after 30 days prompt
async fn check_reflection_ledger_prompt(
    dismissed: &HashMap<String, String>,
    feature_usage: &HashMap<String, Option<String>>,
) -> Vec<OnboardingPrompt> {
    let prompt_id = "reflection_ledger_empty_30d";

    if dismissed.contains_key(prompt_id) {
        return Vec::new();
    }

    // Skip if reflection ledger has been used
    if feature_usage.get("reflection_ledger_first_used").and_then(|v| v.as_ref()).is_some() {
        return Vec::new();
    }

    // Check if reflection ledger is empty
    let entries = match fleet::list_approved_reflection_entries(None) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    if !entries.is_empty() {
        return Vec::new();
    }

    // Check when the operator first started using HOOP
    // For simplicity, we'll use a heuristic: if any feature has been used >30 days ago
    let has_usage_older_than_30d = feature_usage.values().any(|v| {
        v.as_ref().and_then(|ts| ts.parse::<DateTime<Utc>>().ok())
            .map(|dt| Utc::now().signed_duration_since(dt).num_days() > 30)
            .unwrap_or(false)
    });

    if !has_usage_older_than_30d {
        return Vec::new();
    }

    vec![OnboardingPrompt {
        id: prompt_id.to_string(),
        prompt_type: OnboardingPromptType::ReflectionLedgerEmpty,
        title: "Start proposing rules?".to_string(),
        message: "You've been using HOOP for a while. Would you like me to start proposing rules based on your workflow patterns? Rules can automate repetitive tasks and enforce best practices.".to_string(),
        action_label: Some("Learn More".to_string()),
        action_url: Some("https://docs.hoop.dev/reflection-ledger".to_string()),
        eligible_at: Utc::now().to_rfc3339(),
        dismissed_at: dismissed.get(prompt_id).cloned(),
        priority: 50,
    }]
}

/// Check for Pattern suggestion when 10+ Stitches share a theme
async fn check_pattern_suggestion_prompt(
    dismissed: &HashMap<String, String>,
    feature_usage: &HashMap<String, Option<String>>,
) -> Vec<OnboardingPrompt> {
    let prompt_id = "pattern_suggestion_10plus";

    if dismissed.contains_key(prompt_id) {
        return Vec::new();
    }

    // Skip if patterns have been used
    if feature_usage.get("patterns_first_used").and_then(|v| v.as_ref()).is_some() {
        return Vec::new();
    }

    // Analyze stitches for common themes
    // For now, we'll use a simple heuristic based on stitch count
    let stitch_count = match fleet::get_total_stitch_count().await {
        Ok(count) => count,
        Err(_) => return Vec::new(),
    };

    if stitch_count < 10 {
        return Vec::new();
    }

    vec![OnboardingPrompt {
        id: prompt_id.to_string(),
        prompt_type: OnboardingPromptType::PatternSuggestion {
            theme: "repeated-tasks".to_string(),
            stitch_count,
        },
        title: "Create a Pattern?".to_string(),
        message: format!("You've completed {} Stitches with similar patterns. Creating a Pattern can help automate similar tasks in the future.", stitch_count),
        action_label: Some("Create Pattern".to_string()),
        action_url: Some("/patterns".to_string()),
        eligible_at: Utc::now().to_rfc3339(),
        dismissed_at: dismissed.get(prompt_id).cloned(),
        priority: 60,
    }]
}

/// Check for Agent intro prompt
fn check_agent_intro_prompt(
    dismissed: &HashMap<String, String>,
    feature_usage: &HashMap<String, Option<String>>,
) -> Vec<OnboardingPrompt> {
    let prompt_id = "agent_intro";

    if dismissed.contains_key(prompt_id) {
        return Vec::new();
    }

    // Skip if agent has been used
    if feature_usage.get("agent_first_used").and_then(|v| v.as_ref()).is_some() {
        return Vec::new();
    }

    vec![OnboardingPrompt {
        id: prompt_id.to_string(),
        prompt_type: OnboardingPromptType::AgentIntro,
        title: "Try the AI Agent".to_string(),
        message: "HOOP's AI agent can help you with complex tasks, answer questions about your codebase, and even create pull requests. Click to start a conversation.".to_string(),
        action_label: Some("Start Chat".to_string()),
        action_url: Some("/agent".to_string()),
        eligible_at: Utc::now().to_rfc3339(),
        dismissed_at: dismissed.get(prompt_id).cloned(),
        priority: 40,
    }]
}

/// Check for Mic intro prompt
fn check_mic_intro_prompt(
    dismissed: &HashMap<String, String>,
    feature_usage: &HashMap<String, Option<String>>,
) -> Vec<OnboardingPrompt> {
    let prompt_id = "mic_intro";

    if dismissed.contains_key(prompt_id) {
        return Vec::new();
    }

    // Skip if mic has been used
    if feature_usage.get("mic_first_used").and_then(|v| v.as_ref()).is_some() {
        return Vec::new();
    }

    vec![OnboardingPrompt {
        id: prompt_id.to_string(),
        prompt_type: OnboardingPromptType::MicIntro,
        title: "Dictate your notes".to_string(),
        message: "Press the mic hotkey (default: Ctrl+Shift+D) to start dictating. HOOP will transcribe your voice and create a Stitch from your words.".to_string(),
        action_label: None,
        action_url: None,
        eligible_at: Utc::now().to_rfc3339(),
        dismissed_at: dismissed.get(prompt_id).cloned(),
        priority: 30,
    }]
}

pub fn router() -> Router<DaemonState> {
    Router::new()
        .route("/api/onboarding/prompts", get(list_onboarding_prompts))
        .route("/api/onboarding/dismiss", post(dismiss_onboarding_prompt))
        .route("/api/onboarding/enable", post(set_onboarding_enabled))
        .route("/api/onboarding/record-usage", post(record_feature_usage))
        .route("/api/onboarding/ack-version", post(acknowledge_version))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_onboarding_prompt_type_serialization() {
        let pt = OnboardingPromptType::WhatsNew {
            version: "1.1.0".to_string(),
        };
        let json = serde_json::to_string(&pt).unwrap();
        assert!(json.contains("whats_new"));

        let pt2: OnboardingPromptType = serde_json::from_str(&json).unwrap();
        match pt2 {
            OnboardingPromptType::WhatsNew { version } => assert_eq!(version, "1.1.0"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_onboarding_prompt_serialization() {
        let prompt = OnboardingPrompt {
            id: "test_prompt".to_string(),
            prompt_type: OnboardingPromptType::AgentIntro,
            title: "Test".to_string(),
            message: "Test message".to_string(),
            action_label: Some("Action".to_string()),
            action_url: Some("/test".to_string()),
            eligible_at: "2024-01-01T00:00:00Z".to_string(),
            dismissed_at: None,
            priority: 50,
        };

        let json = serde_json::to_string(&prompt).unwrap();
        assert!(json.contains("agent_intro"));
        assert!(json.contains("Test message"));
    }
}
