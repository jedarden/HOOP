//! REST API endpoints for dictated notes
//!
//! Endpoints:
//! - POST   /api/p/:project/dictated-notes       — create a new dictated note
//! - GET    /api/p/:project/dictated-notes       — list notes for a project
//! - GET    /api/dictated-notes/:stitch_id       — get a single note
//! - PATCH  /api/dictated-notes/:stitch_id       — update a note
//! - POST   /api/dictated-notes/:stitch_id/redact — redact words from note
//! - GET    /api/dictated-notes/:stitch_id/audio — serve the audio file
//!
//! On creation, if no pre-computed transcript is provided, the audio is
//! submitted to the Whisper transcription queue for async processing.

use crate::dictated_notes::{
    self, CreateNoteRequest, CreateNoteResponse, DictatedNote, TranscriptionStatus,
};
use crate::fleet::{self, ActionKind, ActionResult};
use crate::id_validators::{self, ValidStitchId};
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, patch, post},
    Json, Router,
};
use base64::Engine;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Build the router for dictated note endpoints
pub fn router() -> Router<crate::DaemonState> {
    Router::new()
        .route("/api/p/{project}/dictated-notes", post(create_note))
        .route("/api/p/{project}/dictated-notes", get(list_notes))
        .route("/api/dictated-notes/{stitch_id}", get(get_note))
        .route("/api/dictated-notes/{stitch_id}", patch(update_note))
        .route("/api/dictated-notes/{stitch_id}/redact", post(redact_words))
        .route("/api/dictated-notes/{stitch_id}/audio", get(get_audio))
        .route("/api/dictated-notes/{stitch_id}/synthesize", post(synthesize_draft))
        .route("/api/dictated-notes/{stitch_id}/draft", post(create_draft_from_note))
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

    // Decode audio data
    let audio_data = base64::engine::general_purpose::STANDARD
        .decode(&req.audio_data)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("Invalid base64 audio data: {}", e),
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

    // Determine initial state: pre-transcribed or pending
    let has_transcript = req.transcript.is_some();
    let (transcript, transcription_status) = if let Some(t) = &req.transcript {
        (t.clone(), TranscriptionStatus::Completed)
    } else {
        (
            "Transcription pending...".to_string(),
            TranscriptionStatus::Pending,
        )
    };

    // §18.2 secrets scan: flag secrets in the incoming transcript (Phase 3)
    if has_transcript {
        let findings = crate::redaction::scan_voice_transcript(&transcript);
        if !findings.is_empty() {
            tracing::warn!(
                project = %project,
                findings = findings.len(),
                "Voice transcript contains potential secrets — flagged for operator review (§18.2)"
            );
        }
    }

    let title = if has_transcript {
        dictated_notes::derive_title(&transcript)
    } else {
        format!("Voice note {}", now.format("%Y-%m-%d %H:%M"))
    };

    // Store audio file
    let audio_path =
        dictated_notes::store_audio(&valid_stitch_id, &req.audio_filename, &audio_data).map_err(
            |e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to store audio: {}", e),
                )
            },
        )?;

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
    dictated_notes::insert_stitch(&conn, &valid_stitch_id, &project, &title, "operator").map_err(
        |e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create stitch: {}", e),
            )
        },
    )?;

    // Insert note metadata
    let note = DictatedNote {
        stitch_id: stitch_id.clone(),
        recorded_at: now,
        transcribed_at: now,
        audio_filename: req.audio_filename.clone(),
        transcript,
        transcript_words: req.transcript_words.unwrap_or_default(),
        redacted_words: vec![],
        duration_secs: req.duration_secs,
        language: req.language,
        tags: req.tags.unwrap_or_default(),
        transcription_status: transcription_status.clone(),
    };
    dictated_notes::insert_note(&conn, &note).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to insert note: {}", e),
        )
    })?;

    // Submit transcription job if no pre-computed transcript
    if !has_transcript {
        if let Some(ref svc) = state.transcription_service {
            match svc.submit_job(stitch_id.clone(), audio_path).await {
                Ok(job_id) => {
                    tracing::info!(
                        "Submitted transcription job {} for dictated note {}",
                        job_id,
                        stitch_id
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
    crate::id_validators::validate_project_name(&project)
        .map_err(crate::id_validators::rejection)?;

    let db_path = fleet::db_path();
    let conn = rusqlite::Connection::open(&db_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("DB error: {}", e),
        )
    })?;

    let notes = dictated_notes::list_notes_for_project(&conn, &project).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Query error: {}", e),
        )
    })?;

    Ok(Json(notes))
}

/// GET /api/dictated-notes/:stitch_id — get a single note
async fn get_note(
    Path(stitch_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let valid_id = ValidStitchId::parse(&stitch_id).map_err(crate::id_validators::rejection)?;

    let db_path = fleet::db_path();
    let conn = rusqlite::Connection::open(&db_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("DB error: {}", e),
        )
    })?;

    let note = dictated_notes::get_note(&conn, valid_id.as_str()).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Query error: {}", e),
        )
    })?;

    match note {
        Some(n) => Ok(Json(n).into_response()),
        None => Err((StatusCode::NOT_FOUND, "Note not found".to_string())),
    }
}

/// GET /api/dictated-notes/:stitch_id/audio — serve the audio file
///
/// Serves redacted audio if redactions exist, otherwise serves original audio.
async fn get_audio(
    Path(stitch_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let valid_id = ValidStitchId::parse(&stitch_id).map_err(crate::id_validators::rejection)?;

    let db_path = fleet::db_path();
    let conn = rusqlite::Connection::open(&db_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("DB error: {}", e),
        )
    })?;

    let note = dictated_notes::get_note(&conn, valid_id.as_str()).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Query error: {}", e),
        )
    })?;

    let note = match note {
        Some(n) => n,
        None => return Err((StatusCode::NOT_FOUND, "Note not found".to_string())),
    };

    let audio_path = dictated_notes::audio_path(&valid_id, &note.audio_filename)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    // Serve redacted audio if redactions exist, otherwise serve original
    let audio_file_path = if note.redacted_words.is_empty() {
        audio_path.clone()
    } else {
        let redacted_path = crate::audio_redaction::redacted_audio_path(&audio_path);
        // Generate redacted audio if it doesn't exist
        if !redacted_path.exists() {
            crate::audio_redaction::mute_audio_segments(
                &audio_path,
                &redacted_path,
                &note.redacted_words,
            )
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Audio redaction failed: {}", e),
                )
            })?;
        }
        redacted_path
    };

    let contents = std::fs::read(&audio_file_path)
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
    let conn = rusqlite::Connection::open(&db_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("DB error: {}", e),
        )
    })?;

    let note = dictated_notes::get_note(&conn, valid_id.as_str()).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Query error: {}", e),
        )
    })?;

    let mut note = match note {
        Some(n) => n,
        None => return Err((StatusCode::NOT_FOUND, "Note not found".to_string())),
    };

    if let Some(title) = req.title {
        dictated_notes::update_stitch_title(&conn, valid_id.as_str(), &title).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Title update error: {}", e),
            )
        })?;
    }
    if let Some(transcript) = req.transcript {
        // §18.2 secrets scan: flag secrets in the updated transcript (Phase 3)
        let findings = crate::redaction::scan_voice_transcript(&transcript);
        if !findings.is_empty() {
            tracing::warn!(
                stitch_id = %valid_id.as_str(),
                findings = findings.len(),
                "Updated voice transcript contains potential secrets — flagged for operator review (§18.2)"
            );
        }
        note.transcript = transcript;
        note.transcription_status = TranscriptionStatus::Completed;
    }
    if let Some(tags) = req.tags {
        note.tags = tags;
    }

    dictated_notes::update_note(&conn, &note).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Update error: {}", e),
        )
    })?;

    Ok(Json(note))
}

/// Request body for redacting words from a dictated note
#[derive(Debug, Deserialize)]
struct RedactWordsRequest {
    /// Indices of words to redact (into transcript_words array)
    word_indices: Vec<usize>,
}

/// POST /api/dictated-notes/:stitch_id/redact — redact words from a note
///
/// Redacts the specified words atomically by:
/// 1. Checking for duplicates (idempotency - re-redacting same words is a no-op)
/// 2. Generating a redacted audio file with muted segments
/// 3. Reconstructing transcript with [REDACTED] placeholders
/// 4. Updating database with redacted word list and new transcript
/// 5. Writing an audit log entry for reversible tracking (§18.2)
///
/// The operation is atomic: if audio generation fails, no database changes are made.
/// Original words are preserved in redacted_words for audit trail.
async fn redact_words(
    Path(stitch_id): Path<String>,
    Json(req): Json<RedactWordsRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let valid_id = ValidStitchId::parse(&stitch_id).map_err(id_validators::rejection)?;

    let db_path = fleet::db_path();
    let conn = rusqlite::Connection::open(&db_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("DB error: {}", e),
        )
    })?;

    let note = dictated_notes::get_note(&conn, valid_id.as_str())
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Query error: {}", e),
            )
        })?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Note not found".to_string()))?;

    // Get project for audit logging
    let project: Option<String> = conn
        .query_row(
            "SELECT project FROM stitches WHERE id = ?",
            params![valid_id.as_str()],
            |row| row.get(0),
        )
        .ok();

    // Get the audio path
    let audio_path = dictated_notes::audio_path(&valid_id, &note.audio_filename).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Audio path error: {}", e),
        )
    })?;

    // Perform atomic redaction: validates indices, generates audio, reconstructs transcript
    let (all_redacted, redacted_transcript) = crate::audio_redaction::atomic_redact_words(
        &audio_path,
        &note.transcript_words,
        &note.redacted_words,
        &req.word_indices,
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Audio redaction failed: {}", e),
        )
    })?;

    // Collect newly redacted words for audit log (those not in existing redacted_words)
    let existing_indices: std::collections::HashSet<usize> =
        note.redacted_words.iter().map(|rw| rw.word_index).collect();
    let newly_redacted: Vec<&dictated_notes::RedactedWord> = all_redacted
        .iter()
        .filter(|rw| !existing_indices.contains(&rw.word_index))
        .collect();

    // Build audit args with word indices and original words for reversible tracking
    let audit_args = serde_json::json!({
        "word_indices": &req.word_indices,
        "redacted_words": newly_redacted.iter().map(|rw| {
            serde_json::json!({
                "word_index": rw.word_index,
                "original_word": rw.original_word,
                "start": rw.start,
                "end": rw.end,
            })
        }).collect::<Vec<_>>(),
        "audio_filename": &note.audio_filename,
    });

    // Update note with new redacted list and transcript
    let updated_note = dictated_notes::DictatedNote {
        redacted_words: all_redacted.clone(),
        transcript: redacted_transcript.clone(),
        ..note
    };

    dictated_notes::update_note(&conn, &updated_note).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Update error: {}", e),
        )
    })?;

    // Write audit log entry (§18.2) - reversible only from audit log
    let _ = fleet::write_audit_row(
        "operator",
        ActionKind::WordsRedacted,
        valid_id.as_str(),
        project.as_deref(),
        Some(audit_args.to_string()),
        ActionResult::Success,
        None,
        None,
        Some(valid_id.as_str()),
        None,
    );

    tracing::info!(
        stitch_id = %valid_id.as_str(),
        words_redacted = newly_redacted.len(),
        total_redacted = all_redacted.len(),
        "Redacted {} words from dictated note ({} total redacted)",
        newly_redacted.len(),
        all_redacted.len()
    );

    Ok(Json(updated_note))
}

/// Request to synthesize a title and body from a dictated note transcript
#[derive(Debug, Deserialize)]
struct SynthesizeRequest {
    /// Optional override for the stitch kind (defaults to "task")
    #[serde(default)]
    kind: Option<String>,
}

/// Response with synthesized title and body
#[derive(Debug, Serialize)]
struct SynthesizeResponse {
    /// Synthesized title (max 280 chars)
    pub title: String,
    /// Synthesized body/description
    pub body: String,
    /// Inferred stitch kind
    pub kind: String,
    /// Whether this looks like a fix, task, or investigation
    pub confidence: String,
}

/// POST /api/dictated-notes/:stitch_id/synthesize — synthesize title + body from transcript
///
/// Uses the agent session to generate a concise title and structured body
/// from the transcript. This is called after transcription completes to
/// propose a Stitch draft.
async fn synthesize_draft(
    Path(stitch_id): Path<String>,
    State(state): State<crate::DaemonState>,
    Json(req): Json<SynthesizeRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let valid_id = ValidStitchId::parse(&stitch_id).map_err(crate::id_validators::rejection)?;

    let db_path = fleet::db_path();
    let conn = rusqlite::Connection::open(&db_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("DB error: {}", e),
        )
    })?;

    let note = dictated_notes::get_note(&conn, valid_id.as_str()).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Query error: {}", e),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            "Note not found".to_string(),
        )
    })?;

    // Check if transcription is complete
    if note.transcription_status != dictated_notes::TranscriptionStatus::Completed {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "Note transcription is not complete (status: {:?}). Wait for transcription to finish.",
                note.transcription_status
            ),
        ));
    }

    // Use agent session to synthesize title and body
    let mgr = state
        .agent_session_manager
        .as_ref()
        .ok_or_else(|| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "Agent session manager not available".to_string(),
            )
        })?;

    // Build synthesis prompt
    let synthesis_prompt = format!(
        r#"Analyze the following voice transcript and synthesize a concise Stitch title and body.

Transcript:
{}

Requirements:
1. Title: Maximum 280 characters, capture the core intent
2. Body: Structured markdown with:
   - Brief context (what triggered this)
   - Key points or action items
   - Any relevant details mentioned
3. Infer the kind: "fix" (bug/error), "task" (feature/work), or "investigation" (needs research)
4. Output ONLY valid JSON matching this schema:
   {{"title": "...", "body": "...", "kind": "fix|task|investigation", "confidence": "high|medium|low"}}

Be concise and actionable. Focus on what needs to be done."#,
        note.transcript
    );

    // Send turn to agent and collect response
    let mut stream = mgr.send_turn(synthesis_prompt, vec![]).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to send synthesis request: {}", e),
        )
    })?;

    // Collect the full response
    let mut full_response = String::new();
    use futures_util::StreamExt;
    while let Some(item) = stream.next().await {
        match item {
            Ok(crate::agent_adapter::AgentEvent::TextDelta { text }) => {
                full_response.push_str(&text);
            }
            Ok(crate::agent_adapter::AgentEvent::TurnComplete { .. }) => break,
            Ok(crate::agent_adapter::AgentEvent::Error { message }) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Agent error during synthesis: {}", message),
                ));
            }
            Err(e) => {
                tracing::warn!("Agent stream error: {}", e);
                break;
            }
            _ => {}
        }
    }

    // Parse JSON response from agent
    let response_text = full_response.trim();

    // Try to extract JSON from the response (agent may wrap it in markdown)
    let json_str = if let Some(start) = response_text.find('{') {
        if let Some(end) = response_text.rfind('}') {
            &response_text[start..=end]
        } else {
            response_text
        }
    } else {
        response_text
    };

    let parsed: serde_json::Value = serde_json::from_str(json_str).map_err(|e| {
        tracing::warn!("Failed to parse agent JSON response: {}, raw: {}", e, json_str);
        // Fallback to derived title
        serde_json::json!({
            "title": dictated_notes::derive_title(&note.transcript),
            "body": note.transcript.clone(),
            "kind": req.kind.unwrap_or_else(|| "task".to_string()),
            "confidence": "low"
        })
    })?;

    let title = parsed["title"]
        .as_str()
        .unwrap_or_else(|| dictated_notes::derive_title(&note.transcript).as_str())
        .to_string();
    let body = parsed["body"]
        .as_str()
        .unwrap_or_else(|| note.transcript.as_str())
        .to_string();
    let kind = parsed["kind"]
        .as_str()
        .unwrap_or_else(|| req.kind.as_deref().unwrap_or("task"))
        .to_string();
    let confidence = parsed["confidence"]
        .as_str()
        .unwrap_or("low")
        .to_string();

    // Validate and truncate title
    let title = if title.len() > 280 {
        format!("{}…", &title[..279])
    } else {
        title
    };

    tracing::info!(
        "Synthesized draft from note {}: kind={}, confidence={}",
        stitch_id,
        kind,
        confidence
    );

    // Store synthesis result in dictated_notes for reference
    let synthesis_json = serde_json::json!({
        "title": title,
        "body": body,
        "kind": kind,
        "confidence": confidence
    });
    if let Err(e) = dictated_notes::update_synthesis_result(&conn, &stitch_id, &synthesis_json.to_string()) {
        tracing::warn!("Failed to store synthesis_result for {}: {}", stitch_id, e);
    }

    Ok(Json(SynthesizeResponse {
        title,
        body,
        kind,
        confidence,
    }))
}

/// POST /api/dictated-notes/:stitch_id/draft — create a Stitch draft from a dictated note
///
/// Creates a draft in the draft queue using the synthesized or provided title/body.
/// The note and draft are linked bidirectionally for reference.
async fn create_draft_from_note(
    Path(stitch_id): Path<String>,
    State(state): State<crate::DaemonState>,
    Json(req): Json<SynthesizeRequest>,
) -> Result<Json<crate::api_draft_queue::CreateDraftResponse>, (StatusCode, String)> {
    let valid_id = ValidStitchId::parse(&stitch_id).map_err(crate::id_validators::rejection)?;

    let db_path = fleet::db_path();
    let conn = rusqlite::Connection::open(&db_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("DB error: {}", e),
        )
    })?;

    let note = dictated_notes::get_note(&conn, valid_id.as_str()).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Query error: {}", e),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            "Note not found".to_string(),
        )
    })?;

    // Get the project from the stitch
    let project: String = conn
        .query_row(
            "SELECT project FROM stitches WHERE id = ?1",
            [&stitch_id],
            |row| row.get(0),
        )
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get project: {}", e),
            )
        })?;

    // Check if transcription is complete
    if note.transcription_status != dictated_notes::TranscriptionStatus::Completed {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "Note transcription is not complete (status: {:?})",
                note.transcription_status
            ),
        ));
    }

    // First synthesize to get title/body
    let synthesis = synthesize_draft(
        Path(stitch_id.clone()),
        State(state.clone()),
        Json(req.clone()),
    )
    .await?;

    // Create the draft using the draft queue API logic
    let draft_req = crate::api_draft_queue::CreateDraftRequest {
        project: project.clone(),
        title: synthesis.title.clone(),
        kind: synthesis.kind.clone(),
        description: Some(synthesis.body.clone()),
        has_acceptance_criteria: Some(false),
        priority: Some(2),
        labels: Some(vec!["voice-capture".to_string(), "from-dictation".to_string()]),
        source: format!("dictated-note:{}", stitch_id),
        agent_session_id: None,
        turn_id: None,
        force_create: false,
    };

    // Resolve project path for validation
    let _project_path = crate::api_draft_queue::resolve_project_path(&project, &state)?;

    // Validate the stitch kind
    crate::api_stitch_decompose::validate_stitch_kind(&draft_req.kind, draft_req.has_acceptance_criteria.unwrap_or(false))?;

    // Dedup check
    let index = state.vector_index.read().unwrap();
    let matches = index.check_duplicate(&draft_req.title, draft_req.description.as_deref());
    if !matches.is_empty() {
        let best = &matches[0];
        let message = format!(
            "this looks like `{}/{}`, which is in progress. Continue that, add this as a child, or proceed as new?",
            best.item.project, best.item.id
        );
        return Err((StatusCode::CONFLICT, message));
    }

    let actor = crate::api_stitch_decompose::resolve_actor(None);
    let now = chrono::Utc::now().to_rfc3339();
    let draft_id = format!("draft-{}", uuid::Uuid::new_v4());

    // Build the draft row
    let draft_row = fleet::DraftRow {
        id: draft_id.clone(),
        project: draft_req.project.clone(),
        title: draft_req.title.clone(),
        kind: draft_req.kind.clone(),
        description: draft_req.description.clone(),
        has_acceptance_criteria: draft_req.has_acceptance_criteria.unwrap_or(false),
        priority: draft_req.priority,
        labels: draft_req.labels.clone().unwrap_or_default(),
        created_by: actor.clone(),
        created_at: now.clone(),
        source: draft_req.source.clone(),
        agent_session_id: draft_req.agent_session_id.clone(),
        turn_id: draft_req.turn_id.clone(),
        status: "pending".to_string(),
        version: 1,
        original_json: None,
        resolved_by: None,
        resolved_at: None,
        rejection_reason: None,
        stitch_id: None,
        preview_json: None,
        opened_by: Some(actor.clone()),
        opened_at: Some(now.clone()),
        last_autosave_at: None,
        abandoned_at: None,
    };

    // Insert the draft into the queue
    fleet::insert_draft(&draft_row)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Update dictated_note with draft_id (bidirectional link)
    if let Err(e) = dictated_notes::update_draft_id(&conn, &stitch_id, &draft_id) {
        tracing::warn!("Failed to update draft_id for dictated note {}: {}", stitch_id, e);
    }

    // Audit: draft created from dictated note
    if let Err(e) = fleet::write_audit_row(
        &actor,
        fleet::ActionKind::DraftCreated,
        &draft_id,
        Some(&project),
        Some(
            serde_json::json!({
                "title": draft_req.title,
                "kind": draft_req.kind,
                "source": draft_req.source,
                "from_note": stitch_id,
            })
            .to_string(),
        ),
        fleet::ActionResult::Success,
        None,
        Some("dictated-note"),
        None,
        None,
    ) {
        tracing::warn!("Failed to write DraftCreated audit row: {}", e);
    }

    tracing::info!(
        "Draft {} created from dictated note {} in project '{}': {}",
        draft_id,
        stitch_id,
        project,
        draft_req.title
    );

    // Emit draft_update WS event
    let _ = state.draft_tx.send(crate::ws::DraftUpdateData {
        draft_id: draft_id.clone(),
        project: project.clone(),
        title: draft_req.title.clone(),
        kind: draft_req.kind.clone(),
        status: "pending".to_string(),
        action: "created".to_string(),
        actor: actor.clone(),
        created_by: actor,
        version: 1,
        rejection_reason: None,
    });

    Ok(Json(crate::api_draft_queue::CreateDraftResponse {
        draft_id,
        status: "pending".to_string(),
    }))
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
