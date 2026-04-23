//! HTTP handlers for resumable upload API

use crate::uploads::{UploadConfig, UploadRegistry};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use std::sync::Arc;

/// Upload state shared across handlers
#[derive(Clone)]
pub struct UploadState {
    pub registry: Arc<UploadRegistry>,
}

/// Request body for upload initiation
#[derive(Debug, Deserialize)]
struct InitUploadRequest {
    filename: String,
    total_size: u64,
    checksum: String,
    attachment_type: String,
    resource_id: String,
}

/// Initiate a new chunked upload
///
/// POST /api/uploads
///
/// Request body:
/// ```json
/// {
///   "filename": "video.mp4",
///   "total_size": 104857600,
///   "checksum": "0123456789abcdef...64 hex chars",
///   "attachment_type": "bead",
///   "resource_id": "hoop-ttb.4.12"
/// }
/// ```
async fn init_upload(
    State(state): State<UploadState>,
    Json(req): Json<InitUploadRequest>,
) -> Result<Json<crate::uploads::InitUploadResponse>, StatusCode> {
    state.registry
        .initiate_upload(
            req.filename,
            req.total_size,
            req.checksum,
            req.attachment_type,
            req.resource_id,
        )
        .map(Json)
        .map_err(|e| {
            tracing::error!("Failed to initiate upload: {}", e);
            StatusCode::BAD_REQUEST
        })
}

/// Upload a chunk
///
/// PATCH /api/uploads/{upload_id}
///
/// Headers:
/// - Upload-Offset: byte offset where chunk starts
/// - Content-Length: size of chunk body
///
/// Body: raw chunk bytes
async fn upload_chunk(
    State(state): State<UploadState>,
    Path(upload_id): Path<String>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Result<Json<crate::uploads::UploadProgressResponse>, StatusCode> {
    // Parse Upload-Offset header
    let offset = headers
        .get("Upload-Offset")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    state
        .registry
        .append_chunk(&upload_id, offset, &body)
        .map(Json)
        .map_err(|e| {
            tracing::error!("Failed to append chunk: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

/// Get upload progress
///
/// HEAD /api/uploads/{upload_id}
///
/// Returns headers:
/// - Upload-Offset: current byte offset
/// - Upload-Length: total file size
async fn get_progress(
    State(state): State<UploadState>,
    Path(upload_id): Path<String>,
) -> Result<Response, StatusCode> {
    let progress = state
        .registry
        .get_progress(&upload_id)
        .map_err(|e| {
            tracing::debug!("Upload not found: {}", e);
            StatusCode::NOT_FOUND
        })?;

    Ok((
        [(axum::http::header::CONTENT_TYPE, "application/json")],
        [
            ("Upload-Offset", progress.offset.to_string()),
            ("Upload-Length", progress.total_size.to_string()),
        ],
        Json(progress),
    )
        .into_response())
}

/// Complete upload and verify checksum
///
/// POST /api/uploads/{upload_id}/complete
async fn complete_upload(
    State(state): State<UploadState>,
    Path(upload_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    state
        .registry
        .complete_upload(&upload_id)
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| {
            tracing::error!("Failed to complete upload: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

/// Cancel an upload
///
/// DELETE /api/uploads/{upload_id}
async fn cancel_upload(
    State(state): State<UploadState>,
    Path(upload_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    state
        .registry
        .cancel_upload(&upload_id)
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| {
            tracing::error!("Failed to cancel upload: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

/// Build the uploads router
pub fn router() -> Router<UploadState> {
    let config = UploadConfig::default();
    let registry = Arc::new(UploadRegistry::new(config).unwrap());
    let state = UploadState { registry };

    Router::new()
        .route("/", post(init_upload))
        .route("/:upload_id", axum::routing::patch(upload_chunk))
        .route("/:upload_id", get(get_progress))
        .route("/:upload_id/complete", post(complete_upload))
        .route("/:upload_id", axum::routing::delete(cancel_upload))
        .with_state(state)
}
