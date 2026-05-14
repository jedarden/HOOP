//! REST API endpoints for the Backup feature.
//!
//! Routes:
//!   POST /api/backup/trigger  — manually trigger a backup

use crate::DaemonState;
use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use serde::Serialize;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Build the backup API router.
pub fn router() -> Router<DaemonState> {
    Router::new().route("/api/backup/trigger", post(trigger_backup))
}

#[derive(Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct TriggerResponse {
    status: String,
    message: String,
}

/// POST /api/backup/trigger
///
/// Manually trigger a backup run. Returns an error if:
/// - Backup is not configured
/// - Credentials are not available
/// - A backup is already in progress (TODO: add run state tracking)
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/api/backup/trigger",
    tag = "backup",
    responses(
        (status = 200, description = "Backup started successfully", body = TriggerResponse),
        (status = 503, description = "Backup is not configured")
    )
))]
async fn trigger_backup(
    State(state): State<DaemonState>,
) -> Result<Json<TriggerResponse>, axum::http::StatusCode> {
    let runner = state
        .backup_runner
        .as_ref()
        .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)?;

    match runner.trigger().await {
        Ok(()) => Ok(Json(TriggerResponse {
            status: "started".to_string(),
            message: "Backup started".to_string(),
        })),
        Err(e) => {
            tracing::error!("Failed to trigger backup: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
