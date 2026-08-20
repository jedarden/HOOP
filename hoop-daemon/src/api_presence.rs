//! REST API for multi-operator presence tracking (§19.4)
//!
//! Endpoints:
//! - GET  /api/presence                  — query all presence (or filtered by project/stitch)
//! - POST /api/presence                  — update or insert presence record
//! - DELETE /api/presence                — remove presence record
//!
//! Presence indicates which operators are currently viewing which projects
//! or Stitches. Records time out after 30 seconds; clients should heartbeat
//! every 15-20 seconds. Privacy toggle (visibility) allows operators to hide
//! their presence from others.

use crate::fleet;
use axum::{
    extract::{ConnectInfo, Query, State},
    http::StatusCode,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing::info;

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

/// Response for a single presence entry
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PresenceResponse {
    pub operator_id: String,
    pub project: Option<String>,
    pub stitch_id: Option<String>,
    pub last_seen: String,
    pub visibility: String,
}

/// Response for listing presence entries
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PresenceListResponse {
    pub presence: Vec<PresenceResponse>,
    pub count: usize,
}

/// Request to update presence
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdatePresenceRequest {
    /// Operator identifier (auto-resolved from connection if not provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_id: Option<String>,
    /// Project name if viewing a project
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    /// Stitch ID if viewing a specific Stitch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stitch_id: Option<String>,
    /// Privacy toggle: "visible" or "hidden"
    #[serde(default = "default_visibility")]
    pub visibility: String,
}

fn default_visibility() -> String {
    "visible".to_string()
}

/// Query parameters for presence filtering
#[derive(Debug, Deserialize)]
pub struct PresenceQueryParams {
    pub project: Option<String>,
    pub stitch_id: Option<String>,
}

/// Request to remove presence
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RemovePresenceRequest {
    /// Project name to remove presence for
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    /// Stitch ID to remove presence for
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stitch_id: Option<String>,
}

/// Response after updating presence
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdatePresenceResponse {
    pub operator_id: String,
    pub last_seen: String,
}

/// Response after removing presence
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RemovePresenceResponse {
    pub success: bool,
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<crate::DaemonState> {
    Router::new().route(
        "/api/presence",
        axum::routing::get(get_presence)
            .post(update_presence)
            .delete(remove_presence),
    )
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /api/presence — query presence entries
///
/// Returns operators currently present at a project or Stitch.
/// Filters out hidden entries (privacy toggle) and stale records (>30s).
#[utoipa::path(
    get,
    path = "/api/presence",
    tag = "presence",
    params(
        ("project" = Option<String>, Query, description = "Filter by project name"),
        ("stitch_id" = Option<String>, Query, description = "Filter by stitch ID")
    ),
    responses(
        (status = 200, description = "List of presence entries", body = PresenceListResponse)
    ),
)]
async fn get_presence(
    State(state): State<crate::DaemonState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    Query(params): Query<PresenceQueryParams>,
) -> Result<Json<PresenceListResponse>, (StatusCode, String)> {
    // Resolve operator identity for potential logging
    let _actor = state.identity_cache.resolve(Some(addr));

    let project = params.project.as_deref();
    let stitch_id = params.stitch_id.as_deref();

    let entries = fleet::query_presence(project, stitch_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let presence: Vec<PresenceResponse> = entries
        .into_iter()
        .map(|e| PresenceResponse {
            operator_id: e.operator_id,
            project: e.project,
            stitch_id: e.stitch_id,
            last_seen: e.last_seen,
            visibility: e.visibility,
        })
        .collect();

    let count = presence.len();

    Ok(Json(PresenceListResponse { presence, count }))
}

/// POST /api/presence — update or insert presence record
///
/// Called when an operator navigates to a project or Stitch.
/// Should be called every 15-20 seconds to heartbeat and keep presence alive.
#[utoipa::path(
    post,
    path = "/api/presence",
    tag = "presence",
    request_body = UpdatePresenceRequest,
    responses(
        (status = 200, description = "Presence updated successfully", body = UpdatePresenceResponse),
        (status = 400, description = "Invalid request parameters")
    )
)]
async fn update_presence(
    State(state): State<crate::DaemonState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    Json(req): Json<UpdatePresenceRequest>,
) -> Result<Json<UpdatePresenceResponse>, (StatusCode, String)> {
    // Resolve operator identity from connection, falling back to request body
    let base_actor = state.identity_cache.resolve(Some(addr));
    let operator_id = req.operator_id.unwrap_or_else(|| base_actor.clone());

    // Validate visibility enum
    if req.visibility != "visible" && req.visibility != "hidden" {
        return Err((
            StatusCode::BAD_REQUEST,
            "visibility must be 'visible' or 'hidden'".to_string(),
        ));
    }

    let project = req.project.as_deref();
    let stitch_id = req.stitch_id.as_deref();

    fleet::update_presence(&operator_id, project, stitch_id, &req.visibility)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let now = chrono::Utc::now().to_rfc3339();

    info!(
        "Presence updated: operator={}, project={:?}, stitch_id={:?}, visibility={}",
        operator_id, project, stitch_id, req.visibility
    );

    // Emit presence_update WS event for real-time collaboration
    // (The presence update will be reflected when clients poll or via WS broadcast)
    let _ = state.presence_tx.send(crate::ws::PresenceUpdateData {
        operator_id: operator_id.clone(),
        project: project.map(|s| s.to_string()),
        stitch_id: stitch_id.map(|s| s.to_string()),
        visibility: req.visibility.clone(),
        last_seen: now.clone(),
    });

    Ok(Json(UpdatePresenceResponse {
        operator_id,
        last_seen: now,
    }))
}

/// DELETE /api/presence — remove presence record
///
/// Called when an operator navigates away from a project or Stitch.
#[utoipa::path(
    delete,
    path = "/api/presence",
    tag = "presence",
    request_body = RemovePresenceRequest,
    responses(
        (status = 200, description = "Presence removed successfully", body = RemovePresenceResponse),
    )
)]
async fn remove_presence(
    State(state): State<crate::DaemonState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    Json(req): Json<RemovePresenceRequest>,
) -> Result<Json<RemovePresenceResponse>, (StatusCode, String)> {
    let actor = state.identity_cache.resolve(Some(addr));

    let project = req.project.as_deref();
    let stitch_id = req.stitch_id.as_deref();

    fleet::remove_presence(&actor, project, stitch_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    info!(
        "Presence removed: operator={}, project={:?}, stitch_id={:?}",
        actor, project, stitch_id
    );

    // Emit presence_update WS event to signal removal
    let _ = state.presence_tx.send(crate::ws::PresenceUpdateData {
        operator_id: actor.clone(),
        project: project.map(|s| s.to_string()),
        stitch_id: stitch_id.map(|s| s.to_string()),
        visibility: "removed".to_string(),
        last_seen: chrono::Utc::now().to_rfc3339(),
    });

    Ok(Json(RemovePresenceResponse { success: true }))
}
