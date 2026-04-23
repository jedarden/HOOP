//! REST API endpoints for dictated notes
//!
//! Endpoints:
//! - POST   /api/p/:project/dictated-notes       — create a new dictated note
//! - GET    /api/p/:project/dictated-notes       — list notes for a project
//! - GET    /api/dictated-notes/:stitch_id       — get a single note
//! - GET    /api/dictated-notes/:stitch_id/audio — serve the audio file

use crate::dictated_notes::{
    self, CreateNoteRequest, CreateNoteResponse, DictatedNote,
};
use crate::fleet;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use base64::Engine;
use uuid::Uuid;

/// Build the router for dictated note endpoints
pub fn router() -> Router<crate::DaemonState> {
    Router::new()
        .route("/api/p/{project}/dictated-notes", post(create_note))
        .route("/api/p/{project}/dictated-notes", get(list_notes))
        .route("/api/dictated-notes/{stitch_id}", get(get_note))
        .route("/api/dictated-notes/{stitch_id}/audio", get(get_audio))
}

/// POST /api/p/:project/dictated-notes — create a new dictated note
async fn create_note(
    Path(project): Path<String>,
    State(state): State<crate::DaemonState>,
    Json(req): Json<CreateNoteRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Validate project exists
    {
        let projects = state.projects.read().unwrap();
        if !projects.iter().any(|p| p.name == project) {
            return Err((StatusCode::NOT_FOUND, format!("Project '{}' not found", project)));
        }
    }

    // Decode audio data
    let audio_data = base64::engine::general_purpose::STANDARD
        .decode(&req.audio_data)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid base64 audio data: {}", e)))?;

    let stitch_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    // Get transcript (from request or stub)
    let transcript = req.transcript.unwrap_or_else(|| {
        format!("Voice note recorded at {}", now.to_rfc3339())
    });
    let title = dictated_notes::derive_title(&transcript);

    // Store audio file
    dictated_notes::store_audio(&stitch_id, &req.audio_filename, &audio_data)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to store audio: {}", e)))?;

    // Insert into fleet.db
    let db_path = fleet::db_path();
    let mut conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB open error: {}", e)))?;
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB WAL error: {}", e)))?;

    // Insert stitch row
    dictated_notes::insert_stitch(&conn, &stitch_id, &project, &title, "operator")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create stitch: {}", e)))?;

    // Insert note metadata
    let note = DictatedNote {
        stitch_id: stitch_id.clone(),
        recorded_at: now,
        transcribed_at: now,
        audio_filename: req.audio_filename.clone(),
        transcript: transcript.clone(),
        transcript_words: req.transcript_words.unwrap_or_default(),
        duration_secs: req.duration_secs,
        language: req.language,
        tags: req.tags.unwrap_or_default(),
    };
    dictated_notes::insert_note(&conn, &note)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to insert note: {}", e)))?;

    tracing::info!(
        "Created dictated note {} in project {}",
        stitch_id,
        project
    );

    let response = CreateNoteResponse {
        stitch_id,
        project,
        title,
        recorded_at: note.recorded_at,
        transcribed_at: note.transcribed_at,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// GET /api/p/:project/dictated-notes — list notes for a project
async fn list_notes(
    Path(project): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let db_path = fleet::db_path();
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;

    let notes = dictated_notes::list_notes_for_project(&conn, &project)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {}", e)))?;

    Ok(Json(notes))
}

/// GET /api/dictated-notes/:stitch_id — get a single note
async fn get_note(
    Path(stitch_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let db_path = fleet::db_path();
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;

    let note = dictated_notes::get_note(&conn, &stitch_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {}", e)))?;

    match note {
        Some(n) => Ok(Json(n).into_response()),
        None => Err((StatusCode::NOT_FOUND, "Note not found".to_string())),
    }
}

/// GET /api/dictated-notes/:stitch_id/audio — serve the audio file
async fn get_audio(
    Path(stitch_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let db_path = fleet::db_path();
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;

    let note = dictated_notes::get_note(&conn, &stitch_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {}", e)))?;

    let note = match note {
        Some(n) => n,
        None => return Err((StatusCode::NOT_FOUND, "Note not found".to_string())),
    };

    let audio_path = dictated_notes::audio_path(&stitch_id, &note.audio_filename)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let contents = std::fs::read(&audio_path)
        .map_err(|_| (StatusCode::NOT_FOUND, "Audio file not found".to_string()))?;

    let mime_type = infer_audio_mime(&note.audio_filename);

    Ok(([(header::CONTENT_TYPE, mime_type)], contents).into_response())
}

fn infer_audio_mime(filename: &str) -> String {
    if let Some(ext) = filename.rsplit('.').next() {
        match ext.to_lowercase().as_str() {
            "mp3" => return "audio/mpeg".to_string(),
            "m4a" => return "audio/mp4".to_string(),
            "wav" => return "audio/wav".to_string(),
            "ogg" | "oga" => return "audio/ogg".to_string(),
            "flac" => return "audio/flac".to_string(),
            "opus" => return "audio/opus".to_string(),
            "webm" => return "audio/webm".to_string(),
            _ => {}
        }
    }
    "audio/mpeg".to_string()
}
