//! Stitch link traversal API (§4.7)
//!
//! HTTP endpoints for the stitch_traversal service:
//! - `GET /api/stitches/{id}/parents` - Get parent Stitches (incoming links)
//! - `GET /api/stitches/{id}/children` - Get child Stitches (outgoing links)
//! - `GET /api/stitches/{id}/referenced_by` - Get Stitches that reference this Stitch
//! - `GET /api/stitches/{id}/closure` - Get transitive closure with optional depth limit

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::fleet;
use crate::stitch_traversal;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ParentsResponse {
    pub stitch_id: String,
    pub parents: Vec<StitchLinkInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<f64>,
}

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ChildrenResponse {
    pub stitch_id: String,
    pub children: Vec<StitchLinkInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<f64>,
}

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ReferencedByResponse {
    pub stitch_id: String,
    pub references: Vec<StitchLinkInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<f64>,
}

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ClosureResponse {
    pub root_stitch_id: String,
    pub kind: String,
    pub max_depth: Option<u32>,
    pub nodes: Vec<ClosureNodeInfo>,
    pub total_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<f64>,
}

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct StitchLinkInfo {
    pub stitch_id: String,
    pub kind: String,
    pub workspace_from: String,
    pub workspace_to: String,
}

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ClosureNodeInfo {
    pub stitch_id: String,
    pub depth: u32,
    pub path: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ClosureQuery {
    #[serde(default = "default_kind")]
    kind: String,
    #[serde(default)]
    max_depth: Option<u32>,
}

fn default_kind() -> String {
    "spawned".to_string()
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<crate::DaemonState> {
    Router::new()
        .route("/api/stitches/{id}/parents", get(get_parents))
        .route("/api/stitches/{id}/children", get(get_children))
        .route("/api/stitches/{id}/referenced_by", get(get_referenced_by))
        .route("/api/stitches/{id}/closure", get(get_closure))
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/api/stitches/{id}/parents",
    tag = "stitch_traversal",
    params(
        ("id" = String, Path, description = "Stitch UUID")
    ),
    responses(
        (status = 200, description = "Parent stitches found", body = ParentsResponse),
        (status = 404, description = "Stitch not found")
    )
)]
async fn get_parents(
    Path(stitch_id): Path<String>,
    State(_state): State<crate::DaemonState>,
) -> Result<Json<ParentsResponse>, (StatusCode, String)> {
    let start = Instant::now();

    crate::id_validators::validate_stitch_id(&stitch_id)
        .map_err(crate::id_validators::rejection)?;

    let db_path = fleet::db_path();
    let conn = rusqlite::Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to open fleet.db: {}", e),
        )
    })?;

    // Verify stitch exists
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM stitches WHERE id = ?1",
            [&stitch_id],
            |row| row.get(0),
        )
        .unwrap_or(0)
        > 0;

    if !exists {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Stitch '{}' not found", stitch_id),
        ));
    }

    let parents = stitch_traversal::parents(&conn, &stitch_id).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to query parents: {}", e),
        )
    })?;

    let parents_info: Vec<StitchLinkInfo> = parents
        .into_iter()
        .map(|p| StitchLinkInfo {
            stitch_id: p.stitch_id,
            kind: p.kind,
            workspace_from: p.workspace_from,
            workspace_to: p.workspace_to,
        })
        .collect();

    let elapsed_ms = start.elapsed().as_secs_f64() * 1_000.0;

    Ok(Json(ParentsResponse {
        stitch_id,
        parents: parents_info,
        elapsed_ms: Some(elapsed_ms),
    }))
}

#[utoipa::path(
    get,
    path = "/api/stitches/{id}/children",
    tag = "stitch_traversal",
    params(
        ("id" = String, Path, description = "Stitch UUID")
    ),
    responses(
        (status = 200, description = "Child stitches found", body = ChildrenResponse),
        (status = 404, description = "Stitch not found")
    )
)]
async fn get_children(
    Path(stitch_id): Path<String>,
    State(_state): State<crate::DaemonState>,
) -> Result<Json<ChildrenResponse>, (StatusCode, String)> {
    let start = Instant::now();

    crate::id_validators::validate_stitch_id(&stitch_id)
        .map_err(crate::id_validators::rejection)?;

    let db_path = fleet::db_path();
    let conn = rusqlite::Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to open fleet.db: {}", e),
        )
    })?;

    // Verify stitch exists
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM stitches WHERE id = ?1",
            [&stitch_id],
            |row| row.get(0),
        )
        .unwrap_or(0)
        > 0;

    if !exists {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Stitch '{}' not found", stitch_id),
        ));
    }

    let children = stitch_traversal::children(&conn, &stitch_id).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to query children: {}", e),
        )
    })?;

    let children_info: Vec<StitchLinkInfo> = children
        .into_iter()
        .map(|c| StitchLinkInfo {
            stitch_id: c.stitch_id,
            kind: c.kind,
            workspace_from: c.workspace_from,
            workspace_to: c.workspace_to,
        })
        .collect();

    let elapsed_ms = start.elapsed().as_secs_f64() * 1_000.0;

    Ok(Json(ChildrenResponse {
        stitch_id,
        children: children_info,
        elapsed_ms: Some(elapsed_ms),
    }))
}

#[utoipa::path(
    get,
    path = "/api/stitches/{id}/referenced_by",
    tag = "stitch_traversal",
    params(
        ("id" = String, Path, description = "Stitch UUID")
    ),
    responses(
        (status = 200, description = "Referencing stitches found", body = ReferencedByResponse),
        (status = 404, description = "Stitch not found")
    )
)]
async fn get_referenced_by(
    Path(stitch_id): Path<String>,
    State(_state): State<crate::DaemonState>,
) -> Result<Json<ReferencedByResponse>, (StatusCode, String)> {
    let start = Instant::now();

    crate::id_validators::validate_stitch_id(&stitch_id)
        .map_err(crate::id_validators::rejection)?;

    let db_path = fleet::db_path();
    let conn = rusqlite::Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to open fleet.db: {}", e),
        )
    })?;

    // Verify stitch exists
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM stitches WHERE id = ?1",
            [&stitch_id],
            |row| row.get(0),
        )
        .unwrap_or(0)
        > 0;

    if !exists {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Stitch '{}' not found", stitch_id),
        ));
    }

    let references =
        stitch_traversal::referenced_by(&conn, &stitch_id).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to query references: {}", e),
            )
        })?;

    let refs_info: Vec<StitchLinkInfo> = references
        .into_iter()
        .map(|r| StitchLinkInfo {
            stitch_id: r.stitch_id,
            kind: r.kind,
            workspace_from: r.workspace_from,
            workspace_to: r.workspace_to,
        })
        .collect();

    let elapsed_ms = start.elapsed().as_secs_f64() * 1_000.0;

    Ok(Json(ReferencedByResponse {
        stitch_id,
        references: refs_info,
        elapsed_ms: Some(elapsed_ms),
    }))
}

#[utoipa::path(
    get,
    path = "/api/stitches/{id}/closure",
    tag = "stitch_traversal",
    params(
        ("id" = String, Path, description = "Stitch UUID"),
        ("kind" = Option<String>, Query, description = "Link kind to follow ('spawned', 'references', 'all')"),
        ("max_depth" = Option<u32>, Query, description = "Maximum traversal depth")
    ),
    responses(
        (status = 200, description = "Closure computed", body = ClosureResponse),
        (status = 404, description = "Stitch not found"),
        (status = 400, description = "Invalid kind parameter")
    )
)]
async fn get_closure(
    Path(stitch_id): Path<String>,
    State(_state): State<crate::DaemonState>,
    Query(query): Query<ClosureQuery>,
) -> Result<Json<ClosureResponse>, (StatusCode, String)> {
    let start = Instant::now();

    crate::id_validators::validate_stitch_id(&stitch_id)
        .map_err(crate::id_validators::rejection)?;

    // Validate kind parameter
    if query.kind != "spawned" && query.kind != "references" && query.kind != "all" {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "Invalid kind: '{}'. Must be 'spawned', 'references', or 'all'",
                query.kind
            ),
        ));
    }

    let db_path = fleet::db_path();
    let conn = rusqlite::Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to open fleet.db: {}", e),
        )
    })?;

    // Verify stitch exists
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM stitches WHERE id = ?1",
            [&stitch_id],
            |row| row.get(0),
        )
        .unwrap_or(0)
        > 0;

    if !exists {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Stitch '{}' not found", stitch_id),
        ));
    }

    let nodes =
        stitch_traversal::closure(&conn, &stitch_id, &query.kind, query.max_depth).map_err(
            |e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to compute closure: {}", e),
                )
            },
        )?;

    let nodes_info: Vec<ClosureNodeInfo> = nodes
        .into_iter()
        .map(|n| ClosureNodeInfo {
            stitch_id: n.stitch_id,
            depth: n.depth,
            path: n.path,
        })
        .collect();

    let total_count = nodes_info.len();
    let elapsed_ms = start.elapsed().as_secs_f64() * 1_000.0;

    Ok(Json(ClosureResponse {
        root_stitch_id: stitch_id,
        kind: query.kind,
        max_depth: query.max_depth,
        nodes: nodes_info,
        total_count,
        elapsed_ms: Some(elapsed_ms),
    }))
}
