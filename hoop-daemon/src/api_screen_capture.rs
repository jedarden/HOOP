//! REST API endpoints for screen-capture stitches
//!
//! POST  /api/p/:project/screen-captures               — create a new screen-capture (complete upload)
//! POST  /api/p/:project/screen-captures/stream        — start a streaming screen-capture upload
//! PATCH /api/p/:project/screen-captures/stream/:id    — append chunk to streaming upload
//! POST  /api/p/:project/screen-captures/stream/:id    — finalize streaming upload
//! GET   /api/p/:project/screen-captures               — list screen-captures for a project
//! GET   /api/screen-capture/:stitch_id                — JSON metadata (chapters + transcript)
//! GET   /api/screen-capture/:stitch_id/video          — range-aware video stream

use crate::atomic_write;
use crate::fleet;
use crate::id_validators::ValidStitchId;
use crate::screen_capture::{self, FrameSample};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
    Json, Router,
};
use base64::Engine;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tower::ServiceExt;
use uuid::Uuid;

/// Wrapper for raw bytes that implements ToSchema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RawBytes(pub Vec<u8>);

/// Request body for creating a screen capture
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct CreateScreenCaptureRequest {
    video_data: String,
    video_content_type: String,
    duration_secs: f64,
    frame_samples: Vec<FrameSample>,
}

/// Response after creating a screen capture
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct CreateScreenCaptureResponse {
    stitch_id: String,
    project: String,
    title: String,
    recorded_at: String,
}

pub fn router() -> Router<crate::DaemonState> {
    Router::new()
        .route(
            "/api/p/{project}/screen-captures",
            post(create_screen_capture),
        )
        .route(
            "/api/p/{project}/screen-captures",
            get(list_screen_captures),
        )
        .route(
            "/api/p/{project}/screen-captures/stream",
            post(start_streaming_upload),
        )
        .route(
            "/api/p/{project}/screen-captures/stream/{stream_id}",
            patch(append_stream_chunk),
        )
        .route(
            "/api/p/{project}/screen-captures/stream/{stream_id}/complete",
            post(complete_streaming_upload),
        )
        .route("/api/screen-capture/{stitch_id}", get(get_metadata))
        .route("/api/screen-capture/{stitch_id}/video", get(get_video))
}

/// POST /api/p/:project/screen-captures — create a new screen capture
#[utoipa::path(
    post,
    path = "/api/p/{project}/screen-captures",
    params(
        ("project" = String, Path, description = "Project name")
    ),
    request_body = CreateScreenCaptureRequest,
    responses(
        (status = 201, description = "Screen capture created", body = CreateScreenCaptureResponse),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "Project not found")
    ),
    tag = "screen_capture"
)]
async fn create_screen_capture(
    Path(project): Path<String>,
    State(state): State<crate::DaemonState>,
    Json(req): Json<CreateScreenCaptureRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    crate::id_validators::validate_project_name(&project)
        .map_err(crate::id_validators::rejection)?;

    // Validate project exists
    {
        let projects = state.projects.read().unwrap();
        if !projects.iter().any(|p| p.name == project) {
            return Err((
                StatusCode::NOT_FOUND,
                format!("Project '{}' not found", project),
            ));
        }
    }

    // Decode video data
    let video_data = base64::engine::general_purpose::STANDARD
        .decode(&req.video_data)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("Invalid base64 video data: {}", e),
            )
        })?;

    let stitch_id = Uuid::new_v4().to_string();
    let valid_stitch_id = ValidStitchId::parse(&stitch_id).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Generated invalid UUID: {}", e),
        )
    })?;
    let now = chrono::Utc::now();
    let title = format!("Screen capture {}", now.format("%Y-%m-%d %H:%M"));

    // Store video file and metadata
    let attachments_dir = screen_capture::attachments_dir(&valid_stitch_id).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create attachments dir: {}", e),
        )
    })?;

    // Determine file extension
    let ext = if req.video_content_type.contains("mp4") {
        "mp4"
    } else {
        "webm"
    };

    // Write video file
    let video_path = attachments_dir.join(format!("screen.{}", ext));
    atomic_write::atomic_write_file(&video_path, &video_data).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to write video file: {}", e),
        )
    })?;

    // Write frame_samples.json
    let frame_samples_path = attachments_dir.join("frame_samples.json");
    let frame_samples_json = serde_json::to_string_pretty(&req.frame_samples).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to serialize frame samples: {}", e),
        )
    })?;
    atomic_write::atomic_write_file_str(&frame_samples_path, &frame_samples_json).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to write frame samples: {}", e),
        )
    })?;

    // §18.1 secrets scan: screen capture text (frame labels, transcript)
    {
        // Scan frame labels for secrets
        let frame_labels: String = req
            .frame_samples
            .iter()
            .map(|f| f.label.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        let findings = crate::redaction::scan_screen_capture_text(&frame_labels);
        if !findings.is_empty() {
            tracing::warn!(
                stitch_id = %stitch_id,
                project = %project,
                findings = findings.len(),
                "Screen capture frame labels contain potential secrets — flagged for operator review (§18.1)"
            );
            crate::redaction::audit_findings(
                "screen_capture",
                &findings,
                crate::redaction_policy::RedactionAction::FlaggedOnly,
                &stitch_id,
                Some(&project),
                "system",
            );
        }
    }

    // Write meta.json
    use crate::screen_capture::ScreenCaptureMeta;
    let meta = ScreenCaptureMeta {
        stitch_id: stitch_id.clone(),
        project: project.clone(),
        title: title.clone(),
        recorded_at: now.to_rfc3339(),
        duration_secs: Some(req.duration_secs),
    };
    let meta_path = attachments_dir.join("meta.json");
    let meta_json = serde_json::to_string_pretty(&meta).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to serialize metadata: {}", e),
        )
    })?;
    atomic_write::atomic_write_file_str(&meta_path, &meta_json).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to write metadata: {}", e),
        )
    })?;

    // Insert into fleet.db
    let db_path = fleet::db_path();
    let conn = rusqlite::Connection::open(&db_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("DB open error: {}", e),
        )
    })?;
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("DB WAL error: {}", e),
            )
        })?;

    // Insert stitch row
    conn.execute(
        "INSERT INTO stitches (id, project, kind, title, created_by, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![&stitch_id, &project, "screen-capture", &title, "operator", now.to_rfc3339()],
    ).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create stitch: {}", e),
        )
    })?;

    // §4.7 Evaluate pattern queries for the new stitch
    if let Err(e) = crate::pattern_query_evaluator::sync_and_emit_pattern_queries(
        &stitch_id,
        &project,
        "screen-capture",
        &title,
        &state.pattern_tx,
    ) {
        tracing::warn!(
            "Failed to sync pattern queries for stitch {}: {}",
            stitch_id,
            e
        );
    }

    tracing::info!(
        "Created screen capture {} in project {} (duration: {:.1}s)",
        stitch_id,
        project,
        req.duration_secs
    );

    let response = CreateScreenCaptureResponse {
        stitch_id,
        project,
        title,
        recorded_at: now.to_rfc3339(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// GET /api/p/:project/screen-captures
async fn list_screen_captures(
    Path(project): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    crate::id_validators::validate_project_name(&project)
        .map_err(crate::id_validators::rejection)?;

    let summaries = screen_capture::list_for_project(&project);
    Ok(Json(summaries))
}

/// GET /api/screen-capture/:stitch_id
async fn get_metadata(
    Path(stitch_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Validate and parse stitch_id at API boundary (path-traversal protection, §13)
    let stitch_id = ValidStitchId::parse(&stitch_id).map_err(crate::id_validators::rejection)?;

    if !screen_capture::has_video(&stitch_id) {
        return Err((
            StatusCode::NOT_FOUND,
            "No screen capture for this stitch".to_string(),
        ));
    }

    let meta = screen_capture::load_meta(&stitch_id);
    let chapters = screen_capture::load_frame_samples(&stitch_id);
    let transcript = screen_capture::load_transcript(&stitch_id);

    let data = screen_capture::ScreenCaptureData {
        video_url: format!("/api/screen-capture/{}/video", stitch_id),
        stitch_id: stitch_id.to_string(),
        title: meta.title,
        project: meta.project,
        recorded_at: meta.recorded_at,
        duration_secs: meta.duration_secs,
        chapters,
        transcript,
    };

    Ok(Json(data))
}

/// GET /api/screen-capture/:stitch_id/video
///
/// Serves the video file with range-request support for smooth seeking.
/// Uses tower-http ServeFile which handles Accept-Ranges, Content-Range,
/// ETags, and conditional GET automatically.
async fn get_video(
    Path(stitch_id): Path<String>,
    request: axum::http::Request<axum::body::Body>,
) -> axum::response::Response {
    use tower_http::services::ServeFile;

    // Validate and parse stitch_id at API boundary (path-traversal protection, §13)
    let stitch_id = match ValidStitchId::parse(&stitch_id) {
        Ok(id) => id,
        Err(e) => {
            let (status, msg) = crate::id_validators::rejection(e);
            return (status, msg).into_response();
        }
    };

    let path = match screen_capture::video_path(&stitch_id) {
        Some(p) => p,
        None => return (StatusCode::NOT_FOUND, "No video file found").into_response(),
    };

    match ServeFile::new(path).oneshot(request).await {
        Ok(response) => response.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// Request body for starting a streaming screen capture upload
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct StartStreamingUploadRequest {
    video_content_type: String,
}

/// POST /api/p/:project/screen-captures/stream
///
/// Start a new streaming screen capture upload session.
/// Returns a stream_id and stitch_id for the session.
#[utoipa::path(
    post,
    path = "/api/p/{project}/screen-captures/stream",
    params(
        ("project" = String, Path, description = "Project name")
    ),
    request_body = StartStreamingUploadRequest,
    responses(
        (status = 200, description = "Streaming upload session created"),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "Project not found")
    ),
    tag = "screen_capture"
)]
async fn start_streaming_upload(
    Path(project): Path<String>,
    State(state): State<crate::DaemonState>,
    Json(req): Json<StartStreamingUploadRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    crate::id_validators::validate_project_name(&project)
        .map_err(crate::id_validators::rejection)?;

    // Validate project exists
    {
        let projects = state.projects.read().unwrap();
        if !projects.iter().any(|p| p.name == project) {
            return Err((
                StatusCode::NOT_FOUND,
                format!("Project '{}' not found", project),
            ));
        }
    }

    // Validate content type
    let valid_content_type = req.video_content_type.contains("video")
        || req.video_content_type.contains("mp4")
        || req.video_content_type.contains("webm")
        || req.video_content_type.contains("mov");

    if !valid_content_type {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid video_content_type. Must be a video MIME type (e.g., video/webm, video/mp4)"
                .into(),
        ));
    }

    let registry = screen_capture::StreamingUploadRegistry::new().map_err(|e| {
        tracing::error!("Failed to create streaming upload registry: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create upload session".into(),
        )
    })?;

    let response = registry
        .start_session(project, req.video_content_type)
        .map_err(|e| {
            tracing::error!("Failed to start streaming upload: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to start upload: {}", e),
            )
        })?;

    tracing::info!(
        "Started streaming screen capture session {} for stitch {}",
        response.stream_id,
        response.stitch_id
    );

    Ok((StatusCode::OK, Json(response)))
}

/// PATCH /api/p/:project/screen-captures/stream/:stream_id
///
/// Append a chunk of video data to a streaming upload.
/// Body: raw video bytes
#[utoipa::path(
    patch,
    path = "/api/p/{project}/screen-captures/stream/{stream_id}",
    params(
        ("project" = String, Path, description = "Project name"),
        ("stream_id" = String, Path, description = "Stream session ID")
    ),
    request_body(content = RawBytes, description = "Raw video bytes", content_type = "application/json"),
    responses(
        (status = 200, description = "Chunk appended successfully"),
        (status = 400, description = "Invalid stream ID"),
        (status = 404, description = "Stream not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "screen_capture"
)]
async fn append_stream_chunk(
    Path((_project, stream_id)): Path<(String, String)>,
    State(_state): State<crate::DaemonState>,
    Json(body): Json<RawBytes>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let registry = screen_capture::StreamingUploadRegistry::new().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create registry".into(),
        )
    })?;

    let received_bytes = registry.append_chunk(&stream_id, &body.0).map_err(|e| {
        tracing::error!("Failed to append chunk to stream {}: {}", stream_id, e);
        (
            StatusCode::NOT_FOUND,
            format!("Stream not found or error: {}", e),
        )
    })?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "stream_id": stream_id,
            "received_bytes": received_bytes,
        })),
    ))
}

/// Request body for completing a streaming upload
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct CompleteStreamingUploadRequest {
    duration_secs: f64,
    frame_samples: Vec<screen_capture::FrameSample>,
}

/// POST /api/p/:project/screen-captures/stream/:stream_id/complete
///
/// Finalize a streaming upload and create the stitch record.
#[utoipa::path(
    post,
    path = "/api/p/{project}/screen-captures/stream/{stream_id}/complete",
    params(
        ("project" = String, Path, description = "Project name"),
        ("stream_id" = String, Path, description = "Stream session ID")
    ),
    request_body = CompleteStreamingUploadRequest,
    responses(
        (status = 200, description = "Upload completed successfully"),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "Stream not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "screen_capture"
)]
async fn complete_streaming_upload(
    Path((_project, stream_id)): Path<(String, String)>,
    State(state): State<crate::DaemonState>,
    Json(req): Json<CompleteStreamingUploadRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let registry = screen_capture::StreamingUploadRegistry::new().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create registry".into(),
        )
    })?;

    let data = registry
        .complete_session(&stream_id, req.duration_secs, req.frame_samples, &state)
        .map_err(|e| {
            tracing::error!("Failed to complete streaming upload {}: {}", stream_id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to complete upload: {}", e),
            )
        })?;

    tracing::info!(
        "Completed streaming screen capture {} (stitch: {})",
        stream_id,
        data.stitch_id
    );

    Ok((StatusCode::OK, Json(data)))
}
