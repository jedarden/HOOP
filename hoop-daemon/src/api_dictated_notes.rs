//! REST API endpoints for dictated notes
//!
//! Endpoints:
//! - POST   /api/p/:project/dictated-notes       — create a new dictated note
//! - GET    /api/p/:project/dictated-notes       — list notes for a project
//! - GET    /api/dictated-notes/:stitch_id       — get a single note
//! - GET    /api/dictated-notes/:stitch_id/audio — serve the audio file
//!
//! On creation, if no pre-computed transcript is provided, the audio is
//! submitted to the Whisper transcription queue for async processing.

use crate::dictated_notes::{
    self, CreateNoteRequest, CreateNoteResponse, DictatedNote, TranscriptionStatus,
};
use crate::fleet;
use crate::id_validators::ValidStitchId;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, patch, post},
    Json, Router,
};
use serde::Deserialize;
use base64::Engine;
use uuid::Uuid;

/// Build the router for dictated note endpoints
pub fn router() -> Router<crate::DaemonState> {
    Router::new()
        .route("/api/p/{project}/dictated-notes", post(create_note))
        .route("/api/p/{project}/dictated-notes", get(list_notes))
        .route("/api/dictated-notes/{stitch_id}", get(get_note))
        .route("/api/dictated-notes/{stitch_id}", patch(update_note))
        .route("/api/dictated-notes/{stitch_id}/audio", get(get_audio))
}

/// POST /api/p/:project/dictated-notes — create a new dictated note
///
/// If `transcript` is provided in the request, the note is created with
/// `transcription_status: Completed`. Otherwise, the note is created with
/// status `Pending` and a Whisper transcription job is enqueued.
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
    let valid_stitch_id = ValidStitchId::parse(&stitch_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Generated invalid UUID: {}", e)))?;
    let now = chrono::Utc::now();

    // Determine initial state: pre-transcribed or pending
    let has_transcript = req.transcript.is_some();
    let (transcript, transcription_status) = if let Some(t) = &req.transcript {
        (t.clone(), TranscriptionStatus::Completed)
    } else {
        ("Transcription pending...".to_string(), TranscriptionStatus::Pending)
    };

    let title = if has_transcript {
        dictated_notes::derive_title(&transcript)
    } else {
        format!("Voice note {}", now.format("%Y-%m-%d %H:%M"))
    };

    // Store audio file
    let audio_path = dictated_notes::store_audio(&valid_stitch_id, &req.audio_filename, &audio_data)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to store audio: {}", e)))?;

    // Insert into fleet.db
    let db_path = fleet::db_path();
    let conn = rusqlite::Connection::open(&db_path)
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
        transcript,
        transcript_words: req.transcript_words.unwrap_or_default(),
        duration_secs: req.duration_secs,
        language: req.language,
        tags: req.tags.unwrap_or_default(),
        transcription_status: transcription_status.clone(),
    };
    dictated_notes::insert_note(&conn, &note)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to insert note: {}", e)))?;

    // Submit transcription job if no pre-computed transcript
    if !has_transcript {
        if let Some(ref svc) = state.transcription_service {
            match svc.submit_job(stitch_id.clone(), audio_path).await {
                Ok(job_id) => {
                    tracing::info!(
                        "Submitted transcription job {} for dictated note {}",
                        job_id, stitch_id
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to submit transcription job for {}: {}. Note will remain in Pending state.",
                        stitch_id, e
                    );
                }
            }
        } else {
            tracing::warn!(
                "No transcription service available for dictated note {}. Note will remain in Pending state.",
                stitch_id
            );
        }
    }

    tracing::info!(
        "Created dictated note {} in project {} (status: {:?})",
        stitch_id,
        project,
        transcription_status
    );

    let response = CreateNoteResponse {
        stitch_id,
        project,
        title,
        recorded_at: note.recorded_at,
        transcribed_at: note.transcribed_at,
        transcription_status,
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
    let valid_id = ValidStitchId::parse(&stitch_id).map_err(crate::id_validators::rejection)?;

    let db_path = fleet::db_path();
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;

    let note = dictated_notes::get_note(&conn, valid_id.as_str())
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
    let valid_id = ValidStitchId::parse(&stitch_id).map_err(crate::id_validators::rejection)?;

    let db_path = fleet::db_path();
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;

    let note = dictated_notes::get_note(&conn, valid_id.as_str())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {}", e)))?;

    let note = match note {
        Some(n) => n,
        None => return Err((StatusCode::NOT_FOUND, "Note not found".to_string())),
    };

    let audio_path = dictated_notes::audio_path(&valid_id, &note.audio_filename)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let contents = std::fs::read(&audio_path)
        .map_err(|_| (StatusCode::NOT_FOUND, "Audio file not found".to_string()))?;

    let mime_type = infer_audio_mime(&note.audio_filename);

    Ok(([(header::CONTENT_TYPE, mime_type)], contents).into_response())
}

/// PATCH request body for updating a dictated note
#[derive(Debug, Deserialize)]
struct UpdateNoteRequest {
    title: Option<String>,
    transcript: Option<String>,
    tags: Option<Vec<String>>,
}

/// PATCH /api/dictated-notes/:stitch_id — update a note's transcript or tags
async fn update_note(
    Path(stitch_id): Path<String>,
    Json(req): Json<UpdateNoteRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let valid_id = ValidStitchId::parse(&stitch_id).map_err(crate::id_validators::rejection)?;

    let db_path = fleet::db_path();
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;

    let note = dictated_notes::get_note(&conn, valid_id.as_str())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {}", e)))?;

    let mut note = match note {
        Some(n) => n,
        None => return Err((StatusCode::NOT_FOUND, "Note not found".to_string())),
    };

    if let Some(title) = req.title {
        dictated_notes::update_stitch_title(&conn, valid_id.as_str(), &title)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Title update error: {}", e)))?;
    }
    if let Some(transcript) = req.transcript {
        note.transcript = transcript;
        note.transcription_status = TranscriptionStatus::Completed;
    }
    if let Some(tags) = req.tags {
        note.tags = tags;
    }

    dictated_notes::update_note(&conn, &note)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {}", e)))?;

    Ok(Json(note))
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
