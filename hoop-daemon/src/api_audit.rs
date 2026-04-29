//! REST API endpoint for querying audit log
//!
//! Endpoints:
//! - GET /api/audit  — query audit log with optional filters

use axum::{extract::Query, http::StatusCode, routing::get, Json};
use serde::{Deserialize, Serialize};

use crate::fleet::{self, ActionKind, AuditRow as FleetAuditRow, RedactionAuditRow as FleetRedactionAuditRow};
use crate::id_validators::{rejection, validate_project_name};

/// Query parameters for audit log
#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    /// Maximum number of results to return
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
    /// Filter by project name
    pub project: Option<String>,
    /// Filter by action kind (bead_created, stitch_created, etc.)
    pub kind: Option<String>,
    /// Filter by redaction pattern name (e.g., "anthropic_api_key", "jwt")
    pub pattern: Option<String>,
    /// Filter by operator (actor who triggered the action)
    pub operator: Option<String>,
}

/// Query parameters for redaction audit log
#[derive(Debug, Deserialize)]
pub struct RedactionAuditQuery {
    /// Maximum number of results to return
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
    /// Filter by project name
    pub project: Option<String>,
    /// Filter by redaction pattern name (e.g., "anthropic_api_key", "jwt")
    pub pattern: Option<String>,
    /// Filter by operator (actor who triggered the action)
    pub operator: Option<String>,
    /// Filter by what was flagged (e.g., "transcript", "attachment", "draft")
    pub what_flagged: Option<String>,
}

/// Response for audit log query
#[derive(Debug, Serialize)]
pub struct AuditResponse {
    pub audit_rows: Vec<AuditRow>,
    pub total_count: usize,
}

/// Response for hash chain verification
#[derive(Debug, Serialize)]
pub struct HashChainVerifyResponse {
    pub valid: bool,
    pub message: String,
    pub row_count: usize,
}

/// Response for redaction audit log query
#[derive(Debug, Serialize)]
pub struct RedactionAuditResponse {
    pub audit_rows: Vec<RedactionAuditRow>,
    pub total_count: usize,
}

/// Audit row for API responses (matches frontend types)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRow {
    pub id: String,
    pub ts: String,
    pub actor: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub target: String,
    pub project: Option<String>,
    pub args: Option<serde_json::Value>,
    pub result: String,
    pub error: Option<String>,
    pub schema_version: String,
}

/// Redaction audit row for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionAuditRow {
    pub id: String,
    pub ts: String,
    pub what_flagged: String,
    pub pattern_name: String,
    pub action: String,
    pub operator: String,
    pub source_ref: Option<String>,
    pub project: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
}

impl From<FleetAuditRow> for AuditRow {
    fn from(row: FleetAuditRow) -> Self {
        let kind_str = match row.kind {
            ActionKind::BeadCreated => "bead_created".to_string(),
            ActionKind::StitchCreated => "stitch_created".to_string(),
            ActionKind::ConfigChanged => "config_changed".to_string(),
            ActionKind::ProjectAdded => "project_added".to_string(),
            ActionKind::ProjectRemoved => "project_removed".to_string(),
            ActionKind::DraftCreated => "draft_created".to_string(),
            ActionKind::DraftApproved => "draft_approved".to_string(),
            ActionKind::DraftEdited => "draft_edited".to_string(),
            ActionKind::DraftRejected => "draft_rejected".to_string(),
            ActionKind::DraftOpened => "draft_opened".to_string(),
            ActionKind::DraftAutosaved => "draft_autosaved".to_string(),
            ActionKind::DraftAbandoned => "draft_abandoned".to_string(),
            ActionKind::ConfigReloaded => "config_reloaded".to_string(),
            ActionKind::ConfigReloadRejected => "config_reload_rejected".to_string(),
            ActionKind::WordsRedacted => "words_redacted".to_string(),
            ActionKind::RedactionFlagged => "redaction_flagged".to_string(),
            ActionKind::ScriptExecuted => "script_executed".to_string(),
            ActionKind::BackupStarted => "backup_started".to_string(),
            ActionKind::BackupFinished => "backup_finished".to_string(),
            ActionKind::BackupFailed => "backup_failed".to_string(),
            ActionKind::RestoreStarted => "restore_started".to_string(),
            ActionKind::RestoreFinished => "restore_finished".to_string(),
            ActionKind::RestoreFailed => "restore_failed".to_string(),
            ActionKind::SchemaMigrated => "schema_migrated".to_string(),
            ActionKind::ReflectionInjected => "reflection_injected".to_string(),
            ActionKind::SkillInvoked => "skill_invoked".to_string(),
        };

        let result_str = match row.result {
            fleet::ActionResult::Success => "success".to_string(),
            fleet::ActionResult::Failure => "failure".to_string(),
            fleet::ActionResult::Partial => "partial".to_string(),
        };

        let args = row.args_json.and_then(|s| serde_json::from_str(&s).ok());

        Self {
            id: row.id,
            ts: row.ts,
            actor: row.actor,
            kind: kind_str,
            target: row.target,
            project: row.project,
            args,
            result: result_str,
            error: row.error,
            schema_version: "1.0.0".to_string(),
        }
    }
}

impl From<FleetRedactionAuditRow> for RedactionAuditRow {
    fn from(row: FleetRedactionAuditRow) -> Self {
        let metadata = row.metadata_json.and_then(|s| serde_json::from_str(&s).ok());

        Self {
            id: row.id,
            ts: row.ts,
            what_flagged: row.what_flagged,
            pattern_name: row.pattern_name,
            action: row.action,
            operator: row.operator,
            source_ref: row.source_ref,
            project: row.project,
            metadata,
            created_at: row.created_at,
        }
    }
}

pub fn router() -> axum::Router<crate::DaemonState> {
    axum::Router::new()
        .route("/api/audit", get(query_audit))
        .route("/api/audit/verify", get(verify_hash_chain))
        .route("/api/redaction-audit", get(query_redaction_audit))
}

/// GET /api/audit — query audit log
async fn query_audit(
    Query(params): Query<AuditQuery>,
) -> Result<Json<AuditResponse>, (StatusCode, String)> {
    // Validate project filter if provided
    if let Some(ref project) = params.project {
        validate_project_name(project).map_err(rejection)?;
    }

    // Parse kind filter if provided
    let kind_filter = match params.kind.as_deref() {
        Some("bead_created") => Some(ActionKind::BeadCreated),
        Some("stitch_created") => Some(ActionKind::StitchCreated),
        Some("config_changed") => Some(ActionKind::ConfigChanged),
        Some("config_reloaded") => Some(ActionKind::ConfigReloaded),
        Some("config_reload_rejected") => Some(ActionKind::ConfigReloadRejected),
        Some("project_added") => Some(ActionKind::ProjectAdded),
        Some("project_removed") => Some(ActionKind::ProjectRemoved),
        Some("draft_created") => Some(ActionKind::DraftCreated),
        Some("draft_approved") => Some(ActionKind::DraftApproved),
        Some("draft_edited") => Some(ActionKind::DraftEdited),
        Some("draft_rejected") => Some(ActionKind::DraftRejected),
        Some("draft_opened") => Some(ActionKind::DraftOpened),
        Some("draft_autosaved") => Some(ActionKind::DraftAutosaved),
        Some("draft_abandoned") => Some(ActionKind::DraftAbandoned),
        Some("words_redacted") => Some(ActionKind::WordsRedacted),
        Some("redaction_flagged") => Some(ActionKind::RedactionFlagged),
        Some("skill_invoked") => Some(ActionKind::SkillInvoked),
        Some(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Invalid kind: {}", params.kind.unwrap()),
            ))
        }
        None => None,
    };

    let limit = params.limit.unwrap_or(100).min(1000); // Cap at 1000
    let offset = params.offset.unwrap_or(0);

    let rows = fleet::query_audit_rows(
        Some(limit),
        Some(offset),
        params.project.as_deref(),
        kind_filter.clone(),
        params.operator.as_deref(),
        params.pattern.as_deref(),
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to query audit log: {}", e),
        )
    })?;

    // Get total count (without limit/offset)
    let total_rows = fleet::query_audit_rows(
        None,
        None,
        params.project.as_deref(),
        kind_filter.clone(),
        params.operator.as_deref(),
        params.pattern.as_deref(),
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get audit count: {}", e),
        )
    })?
    .len();

    let audit_rows: Vec<AuditRow> = rows.into_iter().map(AuditRow::from).collect();

    Ok(Json(AuditResponse {
        audit_rows,
        total_count: total_rows,
    }))
}

/// GET /api/audit/verify — verify hash chain integrity
async fn verify_hash_chain() -> Result<Json<HashChainVerifyResponse>, (StatusCode, String)> {
    match fleet::verify_hash_chain() {
        Ok(()) => {
            // Get row count
            let rows = fleet::query_audit_rows(None, None, None, None, None, None).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to query audit rows: {}", e),
                )
            })?;

            Ok(Json(HashChainVerifyResponse {
                valid: true,
                message: "Hash chain is valid".to_string(),
                row_count: rows.len(),
            }))
        }
        Err(e) => {
            // Get row count even if verification failed
            let row_count = fleet::query_audit_rows(None, None, None, None, None, None)
                .map(|r| r.len())
                .unwrap_or(0);

            Ok(Json(HashChainVerifyResponse {
                valid: false,
                message: format!("Hash chain verification failed: {}", e),
                row_count,
            }))
        }
    }
}

/// GET /api/redaction-audit — query redaction audit log
async fn query_redaction_audit(
    Query(params): Query<RedactionAuditQuery>,
) -> Result<Json<RedactionAuditResponse>, (StatusCode, String)> {
    // Validate project filter if provided
    if let Some(ref project) = params.project {
        validate_project_name(project).map_err(rejection)?;
    }

    let limit = params.limit.unwrap_or(100).min(1000); // Cap at 1000
    let offset = params.offset.unwrap_or(0);

    let rows = fleet::query_redaction_audit_rows(
        Some(limit),
        Some(offset),
        params.project.as_deref(),
        params.pattern.as_deref(),
        params.operator.as_deref(),
        params.what_flagged.as_deref(),
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to query redaction audit log: {}", e),
        )
    })?;

    // Get total count (without limit/offset)
    let total_rows = fleet::query_redaction_audit_rows(
        None,
        None,
        params.project.as_deref(),
        params.pattern.as_deref(),
        params.operator.as_deref(),
        params.what_flagged.as_deref(),
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get redaction audit count: {}", e),
        )
    })?
    .len();

    let audit_rows: Vec<RedactionAuditRow> = rows.into_iter().map(RedactionAuditRow::from).collect();

    Ok(Json(RedactionAuditResponse {
        audit_rows,
        total_count: total_rows,
    }))
}
