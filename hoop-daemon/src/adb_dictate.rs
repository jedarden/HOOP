//! ADB push-to-talk dictation integration
//!
//! Provides two things:
//! 1. `POST /api/adb/dictate` — receives raw audio bytes from a Pixel 6 over Tailscale
//!    and creates a dictated note in the currently-focused (or query-specified) project.
//! 2. `PUT /api/ui/active-project` / `GET /api/ui/active-project` — the UI calls the PUT
//!    each time the user navigates to a project, so the ADB endpoint knows where to file
//!    the note without requiring a `?project=` parameter.
//!
//! ## Phone-side setup
//!
//! See README.md §"Pixel 6 ADB dictation" for full setup instructions.
//! The short version: install Termux + Termux:API on the Pixel 6, deploy
//! `scripts/termux-hoop-listener.sh`, then use `scripts/hoop-adb start/stop`
//! from the coding host to trigger recording.

use axum::{
    body::Bytes,
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Build the router for ADB dictation and active-project tracking endpoints
pub fn router() -> Router<crate::DaemonState> {
    Router::new()
        .route("/api/adb/dictate", post(adb_dictate))
        .route("/api/ui/active-project", put(set_active_project))
        .route("/api/ui/active-project", get(get_active_project))
}

#[derive(Deserialize)]
struct DictateQuery {
    /// Project to file the note under. Falls back to the active project set by the UI.
    project: Option<String>,
    /// Audio filename including extension (e.g. `recording.m4a`).
    /// Defaults to `adb-YYYYMMDD-HHMMSS.m4a`.
    filename: Option<String>,
}

/// Response returned after successfully creating an ADB dictated note
#[derive(Debug, Serialize)]
pub struct AdbDictateResponse {
    pub stitch_id: String,
    pub project: String,
    pub transcription_status: String,
}

/// POST /api/adb/dictate
///
/// Body: raw audio bytes (any format Whisper supports: m4a, wav, mp3, opus, …)
/// Query params:
///   - `project` — target project name (optional if UI has set an active project)
///   - `filename` — audio filename with extension (optional, defaults to timestamped .m4a)
///
/// Example curl from Termux on Pixel 6 (Tailscale IP):
///   curl -X POST "http://100.x.y.z:3000/api/adb/dictate?project=HOOP" \
///        --data-binary @/sdcard/hoop-recording.m4a \
///        -H "Content-Type: audio/mp4"
async fn adb_dictate(
    State(state): State<crate::DaemonState>,
    Query(query): Query<DictateQuery>,
    body: Bytes,
) -> Result<(StatusCode, Json<AdbDictateResponse>), (StatusCode, String)> {
    if body.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Audio body is empty".to_string()));
    }

    // Resolve project: explicit query param wins, then active-project fallback
    let project = resolve_project(&state, query.project)?;

    let filename = query
        .filename
        .filter(|f| !f.is_empty())
        .unwrap_or_else(|| format!("adb-{}.m4a", Utc::now().format("%Y%m%d-%H%M%S")));

    if filename.contains('/') || filename.contains("..") || filename.contains('\\') {
        return Err((StatusCode::BAD_REQUEST, "Invalid filename".to_string()));
    }

    let stitch_id = Uuid::new_v4().to_string();
    let valid_stitch_id = crate::id_validators::ValidStitchId::parse(&stitch_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Generated invalid UUID: {}", e)))?;
    let now = Utc::now();
    let audio_data = body.to_vec();

    // Store audio atomically
    let audio_path = crate::dictated_notes::store_audio(&valid_stitch_id, &filename, &audio_data)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to store audio: {}", e),
            )
        })?;

    let title = format!("ADB note {}", now.format("%Y-%m-%d %H:%M"));

    // Persist to fleet.db
    let db_path = crate::fleet::db_path();
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB open: {}", e)))?;
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB WAL: {}", e)))?;

    crate::dictated_notes::insert_stitch(&conn, &stitch_id, &project, &title, "adb").map_err(
        |e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create stitch: {}", e),
            )
        },
    )?;

    let note = crate::dictated_notes::DictatedNote {
        stitch_id: stitch_id.clone(),
        recorded_at: now,
        transcribed_at: now,
        audio_filename: filename,
        transcript: String::new(),
        transcript_words: vec![],
        duration_secs: None,
        language: None,
        tags: vec!["adb".to_string()],
        transcription_status: crate::dictated_notes::TranscriptionStatus::Pending,
    };
    crate::dictated_notes::insert_note(&conn, &note).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to insert note: {}", e),
        )
    })?;

    // Enqueue Whisper transcription
    if let Some(ref svc) = state.transcription_service {
        match svc.submit_job(stitch_id.clone(), audio_path).await {
            Ok(job_id) => {
                tracing::info!(
                    "Submitted transcription job {} for ADB note {}",
                    job_id,
                    stitch_id
                );
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to submit transcription job for ADB note {}: {}",
                    stitch_id,
                    e
                );
            }
        }
    }

    tracing::info!("Created ADB dictated note {} in project {}", stitch_id, project);

    Ok((
        StatusCode::CREATED,
        Json(AdbDictateResponse {
            stitch_id,
            project,
            transcription_status: "Pending".to_string(),
        }),
    ))
}

/// Resolve which project to use: explicit > active-project > error
fn resolve_project(
    state: &crate::DaemonState,
    explicit: Option<String>,
) -> Result<String, (StatusCode, String)> {
    let project = if let Some(p) = explicit.filter(|s| !s.is_empty()) {
        p
    } else {
        state
            .active_project
            .read()
            .unwrap()
            .clone()
            .ok_or_else(|| {
                (
                    StatusCode::BAD_REQUEST,
                    "No active project. Navigate to a project in the UI or pass ?project=name"
                        .to_string(),
                )
            })?
    };

    // Validate the project is registered
    {
        let projects = state.projects.read().unwrap();
        if !projects.iter().any(|p| p.name == project) {
            return Err((
                StatusCode::NOT_FOUND,
                format!("Project '{}' not found", project),
            ));
        }
    }

    Ok(project)
}

// ── Active-project tracker ────────────────────────────────────────────────────

#[derive(Deserialize)]
struct SetActiveProjectRequest {
    /// Project name. Pass an empty string to clear.
    project: String,
}

#[derive(Serialize)]
struct ActiveProjectResponse {
    project: Option<String>,
}

/// PUT /api/ui/active-project
///
/// Called by the UI whenever the user navigates to a project card. This lets
/// the ADB endpoint know which project to file a note under without requiring
/// the user to specify it on the phone.
async fn set_active_project(
    State(state): State<crate::DaemonState>,
    Json(req): Json<SetActiveProjectRequest>,
) -> Result<Json<ActiveProjectResponse>, (StatusCode, String)> {
    let project: Option<String> = if req.project.is_empty() {
        None
    } else {
        // Validate project exists before accepting it
        {
            let projects = state.projects.read().unwrap();
            if !projects.iter().any(|p| p.name == req.project) {
                return Err((
                    StatusCode::NOT_FOUND,
                    format!("Project '{}' not found", req.project),
                ));
            }
        }
        Some(req.project)
    };

    *state.active_project.write().unwrap() = project.clone();
    tracing::debug!("Active project updated to: {:?}", project);

    Ok(Json(ActiveProjectResponse { project }))
}

/// GET /api/ui/active-project
///
/// Returns the currently active project (for debugging and UI sync).
async fn get_active_project(State(state): State<crate::DaemonState>) -> Json<ActiveProjectResponse> {
    let project = state.active_project.read().unwrap().clone();
    Json(ActiveProjectResponse { project })
}
