//! REST API endpoints for Stitch Replay
//!
//! Endpoints:
//! - GET  /api/p/:project/replay/:bead_id — get replay options for a failed bead
//! - POST /api/p/:project/replay/:bead_id/resume-as-new — create new bead with reconstructed state
//! - POST /api/p/:project/replay/:bead_id/restore-state — restore workspace from stash

use crate::stitch_reconstruction::{self, ReplayOptions};
use axum::{
    extract::{ConnectInfo, Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing::info;

/// Request body for resume-as-new-bead
#[derive(Debug, Deserialize)]
pub struct ResumeAsNewRequest {
    /// Custom title override (optional)
    #[serde(default)]
    title: Option<String>,
    /// Custom body override (optional)
    #[serde(default)]
    description: Option<String>,
    /// Additional labels to include
    #[serde(default)]
    extra_labels: Vec<String>,
}

/// Response for replay options
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ReplayOptionsResponse {
    /// The original bead ID that failed
    pub original_bead_id: String,
    /// The stitch ID
    pub stitch_id: String,
    /// Suggested title for new bead
    pub suggested_title: String,
    /// Suggested body for new bead
    pub suggested_body: String,
    /// Labels to include
    pub labels: Vec<String>,
    /// Resume step description
    pub resume_step: String,
    /// Failure timestamp
    pub failed_at: String,
    /// Error message
    pub error: Option<String>,
    /// Duration before failure (ms)
    pub duration_ms: Option<u64>,
    /// Number of touched files
    pub touched_files_count: usize,
    /// Git stash SHA (if available)
    pub stash_sha: Option<String>,
}

/// Response for resume-as-new-bead
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ResumeAsNewResponse {
    /// The new bead ID
    pub bead_id: String,
    /// The stitch ID
    pub stitch_id: String,
    /// Message
    pub message: String,
}

pub fn router() -> Router<crate::DaemonState> {
    Router::new()
        .route("/api/p/{project}/replay/{bead_id}", get(get_replay_options))
        .route(
            "/api/p/{project}/replay/{bead_id}/resume-as-new",
            post(resume_as_new_bead),
        )
        .route(
            "/api/p/{project}/replay/{bead_id}/restore-state",
            post(restore_workspace_state),
        )
}

/// GET /api/p/:project/replay/:bead_id — get replay options for a failed bead
async fn get_replay_options(
    Path((project, bead_id)): Path<(String, String)>,
    State(state): State<crate::DaemonState>,
) -> Result<Json<ReplayOptionsResponse>, (StatusCode, String)> {
    crate::id_validators::validate_project_name(&project)
        .map_err(crate::id_validators::rejection)?;
    crate::id_validators::validate_bead_id(&bead_id).map_err(crate::id_validators::rejection)?;

    let project_path = crate::api_beads::resolve_project_path(&project, &state)?;

    // Check if events.jsonl exists
    let events_jsonl_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".hoop")
        .join("events.jsonl");

    if !events_jsonl_path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Events file not found at {}", events_jsonl_path.display()),
        ));
    }

    // Reconstruct failure state
    let failure_state = tokio::task::spawn_blocking(move || {
        stitch_reconstruction::reconstruct_failure_state(&bead_id, &project_path, &events_jsonl_path)
    })
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Task join failed: {}", e),
        )
    })?
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to reconstruct failure state: {}", e),
        )
    })?;

    let replay_options = stitch_reconstruction::generate_replay_options(&failure_state);

    Ok(Json(ReplayOptionsResponse {
        original_bead_id: replay_options.original_bead_id,
        stitch_id: replay_options.stitch_id,
        suggested_title: replay_options.suggested_title,
        suggested_body: replay_options.suggested_body,
        labels: replay_options.labels,
        resume_step: replay_options.resume_step,
        failed_at: failure_state.fail_event.timestamp.to_rfc3339(),
        error: failure_state.fail_event.error,
        duration_ms: failure_state.fail_event.duration_ms,
        touched_files_count: failure_state.touched_files.len(),
        stash_sha: failure_state.fail_event.stash_sha,
    }))
}

/// POST /api/p/:project/replay/:bead_id/resume-as-new — create new bead with reconstructed state
#[cfg(not(feature = "zero-write-v01"))]
async fn resume_as_new_bead(
    Path((project, bead_id)): Path<(String, String)>,
    State(state): State<crate::DaemonState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Json(req): Json<ResumeAsNewRequest>,
) -> Result<Json<ResumeAsNewResponse>, (StatusCode, String)> {
    // Zero-write guard: bead creation is a write operation
    #[cfg(feature = "zero-write-v01")]
    {
        let _ = (&state, connect_info, req);
        return Err((
            StatusCode::FORBIDDEN,
            "Bead creation is disabled in zero-write mode".to_string(),
        ));
    }

    crate::id_validators::validate_project_name(&project)
        .map_err(crate::id_validators::rejection)?;
    crate::id_validators::validate_bead_id(&bead_id).map_err(crate::id_validators::rejection)?;

    // Role check: bead creation requires drafter role
    crate::auth::check_role_for_addr(
        &state.role_resolver,
        connect_info.map(|ci| ci.0),
        crate::auth::Role::Drafter,
    )
    .map_err(|e| (e.0, serde_json::to_string(&e.1 .0).unwrap_or_else(|_| e.0.to_string())))?;

    let project_path = crate::api_beads::resolve_project_path(&project, &state)?;

    // Check if events.jsonl exists
    let events_jsonl_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".hoop")
        .join("events.jsonl");

    if !events_jsonl_path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Events file not found at {}", events_jsonl_path.display()),
        ));
    }

    // Reconstruct failure state
    let failure_state = tokio::task::spawn_blocking({
        let bead_id = bead_id.clone();
        let project_path = project_path.clone();
        let events_jsonl_path = events_jsonl_path.clone();
        move || {
            stitch_reconstruction::reconstruct_failure_state(&bead_id, &project_path, &events_jsonl_path)
        }
    })
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Task join failed: {}", e),
        )
    })?
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to reconstruct failure state: {}", e),
        )
    })?;

    let replay_options = stitch_reconstruction::generate_replay_options(&failure_state);

    // Create new bead with reconstructed state
    let title = req.title.unwrap_or_else(|| replay_options.suggested_title.clone());
    let description = req.description.unwrap_or_else(|| replay_options.suggested_body.clone());

    let mut labels = replay_options.labels.clone();
    labels.extend(req.extra_labels);

    let stitch_id = replay_options.stitch_id.clone();

    // Use the create_bead endpoint logic
    let create_req = crate::api_beads::CreateBeadRequest {
        title: title.clone(),
        description: Some(description),
        issue_type: Some("task".to_string()),
        priority: None,
        dependencies: None,
        assignee: None,
        labels: Some(labels),
        source: "replay".to_string(),
        stitch_id: Some(stitch_id.clone()),
        force_create: true,
        parent_bead_id: None,
    };

    // Call create_bead
    let create_response = crate::api_beads::create_bead_internal(
        project,
        &state,
        connect_info,
        create_req,
    )
    .await
    .map_err(|(status, msg)| {
        (status, format!("Failed to create new bead: {}", msg))
    })?;

    let new_bead_id = create_response.id.clone();

    info!(
        "Created new bead {} as replay for failed bead {} (stitch: {})",
        new_bead_id, bead_id, stitch_id
    );

    Ok(Json(ResumeAsNewResponse {
        bead_id: new_bead_id.clone(),
        stitch_id,
        message: format!(
            "Created new bead {} to resume from failed bead {}",
            new_bead_id, bead_id
        ),
    }))
}

/// POST /api/p/:project/replay/:bead_id/restore-state — restore workspace from stash
async fn restore_workspace_state(
    Path((project, bead_id)): Path<(String, String)>,
    State(state): State<crate::DaemonState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    crate::id_validators::validate_project_name(&project)
        .map_err(crate::id_validators::rejection)?;
    crate::id_validators::validate_bead_id(&bead_id).map_err(crate::id_validators::rejection)?;

    let project_path = crate::api_beads::resolve_project_path(&project, &state)?;

    // Check if events.jsonl exists
    let events_jsonl_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".hoop")
        .join("events.jsonl");

    if !events_jsonl_path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Events file not found at {}", events_jsonl_path.display()),
        ));
    }

    // Reconstruct failure state to get stash_sha
    let bead_id_clone = bead_id.clone();
    let project_path_clone = project_path.clone();
    let failure_state = tokio::task::spawn_blocking(move || {
        stitch_reconstruction::reconstruct_failure_state(&bead_id_clone, &project_path_clone, &events_jsonl_path)
    })
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Task join failed: {}", e),
        )
    })?
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to reconstruct failure state: {}", e),
        )
    })?;

    let stash_sha = failure_state
        .fail_event
        .stash_sha
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("No stash_sha found for bead {}", bead_id),
            )
        })?;

    // Restore workspace state
    let project_path_clone = project_path.clone();
    tokio::task::spawn_blocking({
        let stash_sha = stash_sha.clone();
        move || {
            stitch_reconstruction::restore_workspace_state(&stash_sha, &project_path_clone)
        }
    })
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Task join failed: {}", e),
        )
    })?
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to restore workspace state: {}", e),
        )
    })?;

    info!(
        "Restored workspace state for bead {} from stash {}",
        bead_id, stash_sha
    );

    Ok(Json(serde_json::json!({
        "status": "ok",
        "message": format!("Restored workspace state from stash {}", stash_sha),
        "stash_sha": stash_sha
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replay_options_response_serialization() {
        let response = ReplayOptionsResponse {
            original_bead_id: "test-bead-1".to_string(),
            stitch_id: "stitch-abc123".to_string(),
            suggested_title: "Resume: test".to_string(),
            suggested_body: "Test body".to_string(),
            labels: vec!["stitch:abc123".to_string(), "resume".to_string()],
            resume_step: "step 3".to_string(),
            failed_at: "2026-04-26T10:00:00Z".to_string(),
            error: Some("test error".to_string()),
            duration_ms: Some(300000),
            touched_files_count: 5,
            stash_sha: Some("abc123".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("test-bead-1"));
        assert!(json.contains("stitch-abc123"));
        assert!(json.contains("resume"));
    }

    #[test]
    fn test_resume_as_new_request_deserialization() {
        let json = r#"{"title":"Custom title","description":"Custom body","extra_labels":["urgent"]}"#;
        let req: ResumeAsNewRequest = serde_json::from_str(json).unwrap();

        assert_eq!(req.title, Some("Custom title".to_string()));
        assert_eq!(req.description, Some("Custom body".to_string()));
        assert_eq!(req.extra_labels, vec!["urgent".to_string()]);
    }

    #[test]
    fn test_resume_as_new_request_defaults() {
        let json = r#"{}"#;
        let req: ResumeAsNewRequest = serde_json::from_str(json).unwrap();

        assert!(req.title.is_none());
        assert!(req.description.is_none());
        assert!(req.extra_labels.is_empty());
    }

    #[test]
    fn test_resume_as_new_response_serialization() {
        let response = ResumeAsNewResponse {
            bead_id: "new-bead-1".to_string(),
            stitch_id: "stitch-abc123".to_string(),
            message: "Created new bead".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("new-bead-1"));
        assert!(json.contains("stitch-abc123"));
    }
}
