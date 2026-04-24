//! Attachment serving API
//!
//! Serves audio, video, image, and PDF attachments from bead and stitch storage.

use crate::attachments;
use crate::id_validators::{ValidBeadId, ValidStitchId};
use axum::{
    extract::Path,
    response::IntoResponse,
};

/// Serve an attachment file (audio, video, image, pdf)
///
/// Path parameters:
/// - `attachment_type`: either "bead" or "stitch"
/// - `id`: bead ID (e.g., "hoop-ttb.4.12") or stitch ID (UUID v4)
/// - `filename`: the attachment filename
///
/// Bead attachments are served from `<workspace>/.beads/attachments/<bead-id>/<filename>`
/// Stitch attachments are served from `~/.hoop/attachments/<stitch-id>/<filename>`
pub async fn serve_attachment(
    Path((attachment_type, id, filename)): Path<(String, String, String)>,
) -> impl IntoResponse {
    use std::fs;

    // Validate ID at the HTTP boundary before any filesystem path construction
    match attachment_type.as_str() {
        "bead" => {
            if let Err(e) = crate::id_validators::validate_bead_id(&id) {
                let (status, body) = crate::id_validators::rejection(e);
                return Err((status, body));
            }
        }
        "stitch" => {
            if let Err(e) = crate::id_validators::validate_stitch_id(&id) {
                let (status, body) = crate::id_validators::rejection(e);
                return Err((status, body));
            }
        }
        _ => return Err((axum::http::StatusCode::BAD_REQUEST, "invalid attachment type".to_string())),
    }

    let file_path = match attachment_type.as_str() {
        "bead" => {
            let bead_id = ValidBeadId::parse(&id)
                .map_err(|_| (axum::http::StatusCode::BAD_REQUEST, "invalid bead id".to_string()))?;
            // Resolve workspace from current directory
            let workspace = std::env::current_dir()
                .map_err(|_| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "internal error".to_string()))?;
            attachments::bead_attachment_path(&workspace, &bead_id, &filename)
                .map_err(|_| (axum::http::StatusCode::NOT_FOUND, "attachment not found".to_string()))?
        }
        "stitch" => {
            let stitch_id = ValidStitchId::parse(&id)
                .map_err(|_| (axum::http::StatusCode::BAD_REQUEST, "invalid stitch id".to_string()))?;
            attachments::stitch_attachment_path(&stitch_id, &filename)
                .map_err(|_| (axum::http::StatusCode::NOT_FOUND, "attachment not found".to_string()))?
        }
        _ => return Err((axum::http::StatusCode::BAD_REQUEST, "invalid attachment type".to_string())),
    };

    // Read file contents
    let contents = fs::read(&file_path)
        .map_err(|_| (axum::http::StatusCode::NOT_FOUND, "attachment not found".to_string()))?;

    // Detect content type from magic bytes
    let mime_type = attachments::AttachmentKind::from_magic(&contents)
        .map(|kind| match kind {
            attachments::AttachmentKind::Image => infer_image_mime(&filename),
            attachments::AttachmentKind::Audio => infer_audio_mime(&filename),
            attachments::AttachmentKind::Video => infer_video_mime(&filename),
            attachments::AttachmentKind::Pdf => "application/pdf",
        })
        .unwrap_or_else(|| infer_audio_mime(&filename));

    let headers = [(axum::http::header::CONTENT_TYPE, mime_type)];
    Ok((headers, contents))
}

/// Infer image MIME type from file extension
fn infer_image_mime(filename: &str) -> &'static str {
    if let Some(ext) = filename.rsplit('.').next() {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" => return "image/jpeg",
            "png" => return "image/png",
            "gif" => return "image/gif",
            "webp" => return "image/webp",
            "svg" => return "image/svg+xml",
            _ => {}
        }
    }
    "image/jpeg"
}

/// Infer audio MIME type from file extension
fn infer_audio_mime(filename: &str) -> &'static str {
    if let Some(ext) = filename.rsplit('.').next() {
        match ext.to_lowercase().as_str() {
            "mp3" => return "audio/mpeg",
            "m4a" => return "audio/mp4",
            "wav" => return "audio/wav",
            "ogg" | "oga" => return "audio/ogg",
            "flac" => return "audio/flac",
            "opus" => return "audio/opus",
            "webm" => return "audio/webm",
            _ => {}
        }
    }
    "audio/mpeg"
}

/// Infer video MIME type from file extension
fn infer_video_mime(filename: &str) -> &'static str {
    if let Some(ext) = filename.rsplit('.').next() {
        match ext.to_lowercase().as_str() {
            "mp4" | "m4v" => return "video/mp4",
            "webm" => return "video/webm",
            "mov" => return "video/quicktime",
            "avi" => return "video/x-msvideo",
            "mkv" => return "video/x-matroska",
            _ => {}
        }
    }
    "video/mp4"
}
