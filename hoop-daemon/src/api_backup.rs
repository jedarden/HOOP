//! REST API endpoints for the Backup feature.
//!
//! Routes:
//!   POST /api/backup/trigger  — manually trigger a backup
//!   GET  /api/backup/status   — get backup status

use crate::DaemonState;
use crate::backup_pipeline::AlreadyRunning;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Build the backup API router.
pub fn router() -> Router<DaemonState> {
    Router::new()
        .route("/api/backup/trigger", post(trigger_backup))
        .route("/api/backup/status", get(backup_status))
}

#[derive(Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct TriggerResponse {
    status: String,
    message: String,
}

#[derive(Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct StatusResponse {
    state: String,
}

/// POST /api/backup/trigger
///
/// Manually trigger a backup run. Returns an error if:
/// - Backup is not configured
/// - Credentials are not available
/// - A backup is already in progress
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/api/backup/trigger",
    tag = "backup",
    responses(
        (status = 200, description = "Backup started successfully", body = TriggerResponse),
        (status = 409, description = "A backup is already in progress", body = TriggerResponse),
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
            // Check if the error is AlreadyRunning
            if e.downcast_ref::<AlreadyRunning>().is_some() {
                return Err(axum::http::StatusCode::CONFLICT);
            }
            tracing::error!("Failed to trigger backup: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /api/backup/status
///
/// Get the current status of the backup system.
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/api/backup/status",
    tag = "backup",
    responses(
        (status = 200, description = "Backup status", body = StatusResponse),
        (status = 503, description = "Backup is not configured")
    )
))]
async fn backup_status(
    State(state): State<DaemonState>,
) -> Result<Json<StatusResponse>, axum::http::StatusCode> {
    let runner = state
        .backup_runner
        .as_ref()
        .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)?;

    let is_running = runner.is_running().await;
    Ok(Json(StatusResponse {
        state: if is_running {
            "running".to_string()
        } else {
            "idle".to_string()
        },
    }))
}
