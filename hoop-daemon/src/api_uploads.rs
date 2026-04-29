//! HTTP handlers for resumable upload API

use crate::DaemonState;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::{delete, get, patch, post},
    Router,
};
use serde::Deserialize;

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
    State(state): State<DaemonState>,
    Json(req): Json<InitUploadRequest>,
) -> Result<Json<crate::uploads::InitUploadResponse>, (StatusCode, String)> {
    // Validate resource_id at the HTTP boundary before it reaches storage
    match req.attachment_type.as_str() {
        "bead" => crate::id_validators::validate_bead_id(&req.resource_id)
            .map_err(crate::id_validators::rejection)?,
        "stitch" => crate::id_validators::validate_stitch_id(&req.resource_id)
            .map_err(crate::id_validators::rejection)?,
        _ => return Err((StatusCode::BAD_REQUEST, "Invalid attachment_type".into())),
    }

    state
        .upload_registry
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
            (StatusCode::BAD_REQUEST, e.to_string())
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
    State(state): State<DaemonState>,
    Path(upload_id): Path<String>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Result<Json<crate::uploads::UploadProgressResponse>, StatusCode> {
    let valid_id = crate::id_validators::ValidUploadId::parse(&upload_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Parse Upload-Offset header
    let offset = headers
        .get("Upload-Offset")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    state
        .upload_registry
        .append_chunk(&valid_id, offset, &body)
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
    State(state): State<DaemonState>,
    Path(upload_id): Path<String>,
) -> Result<Response, StatusCode> {
    let valid_id = crate::id_validators::ValidUploadId::parse(&upload_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let progress = state.upload_registry.get_progress(&valid_id).map_err(|e| {
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
    State(state): State<DaemonState>,
    Path(upload_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let valid_id = crate::id_validators::ValidUploadId::parse(&upload_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Get redaction policy state for project-specific checks
    let redaction_state = state.redaction_policy_state.read().await.clone();

    // Get upload metadata to check the attachment type
    let meta = state
        .upload_registry
        .get_metadata(&valid_id)
        .map_err(|e| {
            tracing::error!("Failed to get upload metadata: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Only check redaction policy for bead attachments (stitches are global)
    if meta.attachment_type == "bead" {
        // Get the workspace path for this bead attachment
        let workspace = std::env::current_dir().map_err(|_| {
            tracing::error!("Failed to get current directory");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // Find the project name for this workspace
        let project_name = redaction_state
            .find_project_by_workspace(&workspace)
            .await;

        if let Some(proj_name) = project_name {
            // Get the partial file path to scan for secrets
            let partial_path = state.upload_registry.get_partial_path(&valid_id).map_err(|e| {
                tracing::error!("Failed to get partial path: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            // Read file content for secret scanning
            let content = std::fs::read_to_string(&partial_path);

            // If we can't read the file as text, we'll allow the upload (fail-open for binary files)
            if let Ok(content) = content {
                // Scan for secrets and write audit entries
                // This will reject the upload if the project's policy is set to Reject
                if let Err(reject_err) = crate::redaction_policy::scan_and_audit(
                    &redaction_state,
                    &proj_name,
                    &content,
                    "attachment",
                    &upload_id,
                    "system",  // Attachment scans are automatic
                )
                .await
                {
                    tracing::warn!(
                        upload_id = %upload_id,
                        project = %proj_name,
                        pattern = %reject_err.pattern,
                        count = %reject_err.count,
                        "Upload rejected due to redaction policy"
                    );
                    return Err(StatusCode::FORBIDDEN);
                }
            }
        }
    }

    state
        .upload_registry
        .complete_upload(&valid_id)
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
    State(state): State<DaemonState>,
    Path(upload_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let valid_id = crate::id_validators::ValidUploadId::parse(&upload_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    state
        .upload_registry
        .cancel_upload(&valid_id)
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| {
            tracing::error!("Failed to cancel upload: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

/// Build the uploads router
pub fn router() -> Router<DaemonState> {
    Router::new()
        .route("/", post(init_upload))
        .route("/:upload_id", patch(upload_chunk))
        .route("/:upload_id", get(get_progress))
        .route("/:upload_id/complete", post(complete_upload))
        .route("/:upload_id", delete(cancel_upload))
}
