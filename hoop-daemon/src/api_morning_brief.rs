//! REST API endpoints for the Morning Brief feature.
//!
//! Routes:
//!   GET  /api/agent/morning-brief/latest   — most recent completed brief
//!   GET  /api/agent/morning-brief/list     — recent briefs (last 10)
//!   POST /api/agent/morning-brief/trigger  — manually trigger a new brief
//!   GET  /api/agent/morning-brief/status   — is a brief currently running?

use crate::fleet;
use crate::DaemonState;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;

/// Build the morning brief API router.
pub fn router() -> Router<DaemonState> {
    Router::new()
        .route("/api/agent/morning-brief/latest", get(get_latest))
        .route("/api/agent/morning-brief/list", get(list_briefs))
        .route("/api/agent/morning-brief/trigger", post(trigger_brief))
        .route("/api/agent/morning-brief/status", get(get_status))
}

/// GET /api/agent/morning-brief/latest
async fn get_latest() -> Result<Json<fleet::MorningBriefRow>, axum::http::StatusCode> {
    match fleet::get_latest_morning_brief() {
        Ok(Some(row)) => Ok(Json(row)),
        Ok(None) => Err(axum::http::StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to get latest morning brief: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /api/agent/morning-brief/list
async fn list_briefs() -> Result<Json<Vec<fleet::MorningBriefRow>>, axum::http::StatusCode> {
    match fleet::list_morning_briefs(10) {
        Ok(rows) => Ok(Json(rows)),
        Err(e) => {
            tracing::error!("Failed to list morning briefs: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Serialize)]
struct TriggerResponse {
    status: String,
    brief_id: Option<String>,
    message: String,
}

/// POST /api/agent/morning-brief/trigger
async fn trigger_brief(
    State(state): State<DaemonState>,
) -> Result<Json<TriggerResponse>, axum::http::StatusCode> {
    let runner = state
        .morning_brief_runner
        .as_ref()
        .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)?;

    match runner.trigger().await {
        Ok(Some(brief_id)) => Ok(Json(TriggerResponse {
            status: "started".to_string(),
            brief_id: Some(brief_id),
            message: "Morning brief generation started".to_string(),
        })),
        Ok(None) => Ok(Json(TriggerResponse {
            status: "already_running".to_string(),
            brief_id: None,
            message: "A morning brief is already running".to_string(),
        })),
        Err(e) => {
            tracing::error!("Failed to trigger morning brief: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Serialize)]
struct StatusResponse {
    running: bool,
}

/// GET /api/agent/morning-brief/status
async fn get_status(
    State(state): State<DaemonState>,
) -> Json<StatusResponse> {
    let running = match &state.morning_brief_runner {
        Some(runner) => runner.is_running().await,
        None => false,
    };
    Json(StatusResponse { running })
}
