//! API endpoints for orphan bead detection and management
//!
//! Endpoints:
//! - GET  /api/p/:project/orphans        — list orphan beads (no stitch:* label)
//! - POST /api/p/:project/orphans/attach — attach orphan to existing Stitch

use crate::id_validators;
use crate::orphan_beads::{attach_orphan_to_stitch, detect_orphans, OrphansResponse};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing::info;

/// Request body for attaching an orphan bead to a Stitch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachOrphanRequest {
    /// The orphan bead ID to attach
    bead_id: String,
    /// The Stitch ID to attach to
    stitch_id: String,
}

/// Response after attaching an orphan bead
#[derive(Debug, Serialize)]
pub struct AttachOrphanResponse {
    pub bead_id: String,
    pub stitch_id: String,
    pub message: String,
}

pub fn router() -> Router<crate::DaemonState> {
    Router::new()
        .route("/api/p/{project}/orphans", get(list_orphans))
        .route("/api/p/{project}/orphans/attach", post(attach_orphan))
}

/// GET /api/p/:project/orphans — list orphan beads
///
/// Returns all beads in the project that have no `stitch:*` label.
/// Orphan beads are those created outside of HOOP's Stitch workflow
/// (e.g., via `br create` in a terminal).
async fn list_orphans(
    Path(project): Path<String>,
    State(state): State<crate::DaemonState>,
) -> Result<Json<OrphansResponse>, (StatusCode, String)> {
    id_validators::validate_project_name(&project)
        .map_err(id_validators::rejection)?;

    let project_path = {
        let projects = state.projects.read().unwrap();
        projects
            .iter()
            .find(|p| p.name == project)
            .map(|p| std::path::PathBuf::from(&p.path))
            .ok_or_else(|| {
                (
                    StatusCode::NOT_FOUND,
                    format!("Project '{}' not found", project),
                )
            })?
    };

    let project_for_log = project.clone();
    let result = tokio::task::spawn_blocking(move || {
        detect_orphans(&project, &project_path)
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task join failed: {}", e)))?
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Detection failed: {}", e)))?;

    info!(
        "Listed {} orphan beads for project '{}'",
        result.total_count,
        project_for_log
    );

    Ok(Json(result))
}

/// POST /api/p/:project/orphans/attach — attach orphan bead to existing Stitch
///
/// Creates a `stitch_beads` row with `kind = 'referenced'`.
/// This associates the bead with the Stitch for UI display without
/// implying the Stitch created it.
async fn attach_orphan(
    Path(project): Path<String>,
    State(state): State<crate::DaemonState>,
    Json(req): Json<AttachOrphanRequest>,
) -> Result<Json<AttachOrphanResponse>, (StatusCode, String)> {
    id_validators::validate_project_name(&project)
        .map_err(id_validators::rejection)?;

    id_validators::validate_bead_id(&req.bead_id)
        .map_err(id_validators::rejection)?;

    id_validators::validate_stitch_id(&req.stitch_id)
        .map_err(id_validators::rejection)?;

    let project_path = {
        let projects = state.projects.read().unwrap();
        projects
            .iter()
            .find(|p| p.name == project)
            .map(|p| std::path::PathBuf::from(&p.path))
            .ok_or_else(|| {
                (
                    StatusCode::NOT_FOUND,
                    format!("Project '{}' not found", project),
                )
            })?
    };

    // Verify the bead exists in this project
    let bead_exists = tokio::task::spawn_blocking({
        let bead_id = req.bead_id.clone();
        let project_path_clone = project_path.clone();
        move || {
            // Check if bead exists via br get
            let mut cmd = crate::br_verbs::invoke_br_read(
                crate::br_verbs::ReadVerb::Get,
                &[&bead_id, "--json"],
            );
            cmd.current_dir(&project_path_clone);
            cmd.output().ok().map(|o| o.status.success())
        }
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task failed: {}", e)))?
    .unwrap_or(false);

    if !bead_exists {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Bead '{}' not found in project '{}'", req.bead_id, project),
        ));
    }

    // Attach the orphan to the stitch
    let project_for_log = project.clone();
    let bead_id = req.bead_id.clone();
    let stitch_id = req.stitch_id.clone();
    let workspace = project_path.to_string_lossy().to_string();

    // Clone again for the closure since we need the originals later
    let bead_id_for_closure = bead_id.clone();
    let stitch_id_for_closure = stitch_id.clone();

    tokio::task::spawn_blocking(move || {
        attach_orphan_to_stitch(&stitch_id_for_closure, &bead_id_for_closure, &workspace)
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task failed: {}", e)))?
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    info!(
        "Attached orphan bead '{}' to stitch '{}' in project '{}'",
        bead_id, stitch_id, project_for_log
    );

    let message = format!(
        "Bead {} attached to Stitch {}",
        bead_id, stitch_id
    );

    Ok(Json(AttachOrphanResponse {
        bead_id,
        stitch_id,
        message,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attach_orphan_request_serialization() {
        let req = AttachOrphanRequest {
            bead_id: "hoop-ttb.1".to_string(),
            stitch_id: "abc123-def456".to_string(),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("hoop-ttb.1"));
        assert!(json.contains("abc123-def456"));
    }

    #[test]
    fn test_attach_orphan_response_serialization() {
        let resp = AttachOrphanResponse {
            bead_id: "hoop-ttb.1".to_string(),
            stitch_id: "abc123-def456".to_string(),
            message: "Attached successfully".to_string(),
        };

        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("hoop-ttb.1"));
        assert!(json.contains("abc123-def456"));
        assert!(json.contains("Attached successfully"));
    }
}
