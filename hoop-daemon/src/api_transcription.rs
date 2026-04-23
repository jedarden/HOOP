//! REST API endpoints for transcription jobs
//!
//! Endpoints:
//! - GET /api/transcription-jobs/:job_id       — get a single job status
//! - GET /api/transcription-jobs               — list jobs with optional filters

use crate::transcription::{JobStatus, TranscriptionJob};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;

/// Query parameters for listing transcription jobs
#[derive(Debug, Deserialize)]
pub struct ListJobsQuery {
    /// Filter by stitch_id
    pub stitch_id: Option<String>,
    /// Filter by status (pending, running, completed, failed)
    pub status: Option<String>,
}

/// Build the router for transcription job endpoints
pub fn router() -> Router<crate::DaemonState> {
    Router::new()
        .route("/api/transcription-jobs", get(list_jobs))
        .route("/api/transcription-jobs/:job_id", get(get_job))
}

/// GET /api/transcription-jobs/:job_id — get a single job status
async fn get_job(
    Path(job_id): Path<String>,
    State(state): State<crate::DaemonState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let transcription_service = state.transcription_service.as_ref()
        .ok_or_else(|| (StatusCode::SERVICE_UNAVAILABLE, "Transcription service not available".to_string()))?;

    let job = transcription_service.get_job(&job_id).await
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Job {} not found", job_id)))?;

    Ok(Json(job))
}

/// GET /api/transcription-jobs — list jobs with optional filters
async fn list_jobs(
    Query(params): Query<ListJobsQuery>,
    State(state): State<crate::DaemonState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let transcription_service = state.transcription_service.as_ref()
        .ok_or_else(|| (StatusCode::SERVICE_UNAVAILABLE, "Transcription service not available".to_string()))?;

    // If stitch_id is specified, filter by stitch
    if let Some(stitch_id) = params.stitch_id {
        let jobs = transcription_service.get_jobs_for_stitch(&stitch_id).await;

        // Optionally filter by status
        let jobs = if let Some(status_str) = params.status {
            let status = parse_job_status(&status_str)
                .ok_or_else(|| (StatusCode::BAD_REQUEST, format!("Invalid status: {}", status_str)))?;
            jobs.into_iter()
                .filter(|j| j.status == status)
                .collect()
        } else {
            jobs
        };

        return Ok(Json(jobs));
    }

    // If no filters specified, return an empty list (we don't expose all jobs across all stitches)
    Ok(Json(Vec::<TranscriptionJob>::new()))
}

/// Parse job status from string
fn parse_job_status(s: &str) -> Option<JobStatus> {
    match s.to_lowercase().as_str() {
        "pending" => Some(JobStatus::Pending),
        "running" => Some(JobStatus::Running),
        "completed" => Some(JobStatus::Completed),
        "failed" => Some(JobStatus::Failed),
        _ => None,
    }
}
