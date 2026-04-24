//! REST API endpoint for querying audit log
//!
//! Endpoints:
//! - GET /api/audit  — query audit log with optional filters

use axum::{
    extract::Query,
    http::StatusCode,
    routing::get,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::fleet::{self, ActionKind, AuditRow as FleetAuditRow};
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

pub fn router() -> axum::Router<crate::DaemonState> {
    axum::Router::new()
        .route("/api/audit", get(query_audit))
        .route("/api/audit/verify", get(verify_hash_chain))
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
        Some("project_added") => Some(ActionKind::ProjectAdded),
        Some("project_removed") => Some(ActionKind::ProjectRemoved),
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
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to query audit log: {}", e),
        )
    })?;

    // Get total count (without limit/offset)
    let total_rows = fleet::query_audit_rows(None, None, params.project.as_deref(), kind_filter)
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
            let rows = fleet::query_audit_rows(None, None, None, None).map_err(|e| {
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
            let row_count = fleet::query_audit_rows(None, None, None, None)
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
