//! REST API endpoints for screen-capture stitches
//!
//! GET /api/p/:project/screen-captures            — list screen-captures for a project
//! GET /api/screen-capture/:stitch_id             — JSON metadata (chapters + transcript)
//! GET /api/screen-capture/:stitch_id/video       — range-aware video stream

use crate::screen_capture;
use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use tower::ServiceExt;

pub fn router() -> Router<crate::DaemonState> {
    Router::new()
        .route("/api/p/{project}/screen-captures", get(list_screen_captures))
        .route("/api/screen-capture/{stitch_id}", get(get_metadata))
        .route("/api/screen-capture/{stitch_id}/video", get(get_video))
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
    crate::id_validators::validate_stitch_id(&stitch_id)
        .map_err(crate::id_validators::rejection)?;

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
        stitch_id,
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

    if let Err(e) = crate::id_validators::validate_stitch_id(&stitch_id) {
        let (status, msg) = crate::id_validators::rejection(e);
        return (status, msg).into_response();
    }

    let path = match screen_capture::video_path(&stitch_id) {
        Some(p) => p,
        None => return (StatusCode::NOT_FOUND, "No video file found").into_response(),
    };

    match ServeFile::new(path).oneshot(request).await {
        Ok(response) => response.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
