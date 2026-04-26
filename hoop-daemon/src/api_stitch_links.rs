//! Stitch link management API
//!
//! `POST /api/stitches/:id/links` - Create a reference link
//! `DELETE /api/stitches/:id/links/:to_stitch_id` - Remove a reference link
//! `GET /api/stitches/search` - Search stitches across all projects

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::fleet;

// ---------------------------------------------------------------------------
// Request/Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateLinkRequest {
    pub to_stitch_id: String,
    pub kind: String,
}

#[derive(Debug, Serialize)]
pub struct CreateLinkResponse {
    pub from_stitch_id: String,
    pub to_stitch_id: String,
    pub kind: String,
    pub created_at: String,
    pub warning: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchStitchesResponse {
    pub results: Vec<StitchSearchResult>,
    pub elapsed_ms: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct StitchSearchResult {
    pub id: String,
    pub project: String,
    pub title: String,
    pub kind: String,
    pub created_at: String,
    pub last_activity_at: String,
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<crate::DaemonState> {
    Router::new()
        .route("/api/stitches/:id/links", post(create_link))
        .route("/api/stitches/:id/links/:to_stitch_id", axum::routing::delete(delete_link))
        .route("/api/stitches/search", get(search_stitches))
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn create_link(
    Path(from_stitch_id): Path<String>,
    State(state): State<crate::DaemonState>,
    Json(req): Json<CreateLinkRequest>,
) -> Result<Json<CreateLinkResponse>, (StatusCode, String)> {
    crate::id_validators::validate_stitch_id(&from_stitch_id).map_err(crate::id_validators::rejection)?;
    crate::id_validators::validate_stitch_id(&req.to_stitch_id).map_err(crate::id_validators::rejection)?;

    let start = Instant::now();

    // Validate kind
    if req.kind != "references" {
        return Err((StatusCode::BAD_REQUEST, format!("Invalid link kind: {}", req.kind)));
    }

    let db_path = fleet::db_path();
    let conn = rusqlite::Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to open fleet.db: {}", e)))?;

    // Check for self-reference
    if from_stitch_id == req.to_stitch_id {
        return Err((StatusCode::BAD_REQUEST, "Cannot link a stitch to itself".to_string()));
    }

    // Check if both stitches exist
    let from_exists: bool = conn
        .query_row("SELECT COUNT(*) FROM stitches WHERE id = ?1", [&from_stitch_id], |row| row.get(0))
        .unwrap_or(0) > 0;
    let to_exists: bool = conn
        .query_row("SELECT COUNT(*) FROM stitches WHERE id = ?1", [&req.to_stitch_id], |row| row.get(0))
        .unwrap_or(0) > 0;

    if !from_exists {
        return Err((StatusCode::NOT_FOUND, format!("Source stitch '{}' not found", from_stitch_id)));
    }
    if !to_exists {
        return Err((StatusCode::NOT_FOUND, format!("Target stitch '{}' not found", req.to_stitch_id)));
    }

    let mut warning = None;

    // Check for reverse link (A->B when B->A exists)
    let reverse_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM stitch_links WHERE from_stitch = ?1 AND to_stitch = ?2 AND kind = ?3",
            [&req.to_stitch_id, &from_stitch_id, &req.kind],
            |row| row.get(0),
        )
        .unwrap_or(0) > 0;

    if reverse_exists {
        warning = Some(format!(
            "Note: '{}' already references this stitch. Creating reciprocal link.",
            req.to_stitch_id
        ));
    }

    // Check for duplicate link
    let link_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM stitch_links WHERE from_stitch = ?1 AND to_stitch = ?2 AND kind = ?3",
            [&from_stitch_id, &req.to_stitch_id, &req.kind],
            |row| row.get(0),
        )
        .unwrap_or(0) > 0;

    if link_exists {
        return Err((
            StatusCode::CONFLICT,
            format!("Link from '{}' to '{}' already exists", from_stitch_id, req.to_stitch_id),
        ));
    }

    // Create the link
    conn.execute(
        "INSERT INTO stitch_links (from_stitch, to_stitch, kind) VALUES (?1, ?2, ?3)",
        [&from_stitch_id, &req.to_stitch_id, &req.kind],
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create link: {}", e)))?;

    let elapsed_ms = start.elapsed().as_secs_f64() * 1_000.0;

    Ok(Json(CreateLinkResponse {
        from_stitch_id,
        to_stitch_id: req.to_stitch_id,
        kind: req.kind,
        created_at: chrono::Utc::now().to_rfc3339(),
        warning,
    }))
}

async fn delete_link(
    Path((from_stitch_id, to_stitch_id)): Path<(String, String)>,
    State(_state): State<crate::DaemonState>,
) -> Result<StatusCode, (StatusCode, String)> {
    crate::id_validators::validate_stitch_id(&from_stitch_id).map_err(crate::id_validators::rejection)?;
    crate::id_validators::validate_stitch_id(&to_stitch_id).map_err(crate::id_validators::rejection)?;

    let db_path = fleet::db_path();
    let conn = rusqlite::Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to open fleet.db: {}", e)))?;

    let rows_affected = conn
        .execute(
            "DELETE FROM stitch_links WHERE from_stitch = ?1 AND to_stitch = ?2",
            [&from_stitch_id, &to_stitch_id],
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete link: {}", e)))?;

    if rows_affected == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Link from '{}' to '{}' not found", from_stitch_id, to_stitch_id),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn search_stitches(
    State(_state): State<crate::DaemonState>,
    axum::extract::Query(params): axum::extract::Query<SearchQuery>,
) -> Result<Json<SearchStitchesResponse>, (StatusCode, String)> {
    let start = Instant::now();

    if params.q.is_empty() || params.q.len() < 2 {
        return Ok(Json(SearchStitchesResponse {
            results: vec![],
            elapsed_ms: Some(start.elapsed().as_secs_f64() * 1_000.0),
        }));
    }

    let db_path = fleet::db_path();
    let conn = rusqlite::Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to open fleet.db: {}", e)))?;

    let search_pattern = format!("%{}%", params.q);
    let project_filter = params.project.as_ref().map(|p| format!("{}", p));

    let results = if let Some(project) = &project_filter {
        let mut stmt = conn
            .prepare(
                "SELECT id, project, title, kind, created_at, last_activity_at \
                 FROM stitches \
                 WHERE project = ?1 AND (id LIKE ?2 OR title LIKE ?2) \
                 ORDER BY last_activity_at DESC \
                 LIMIT ?3",
            )
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Prepare search: {}", e)))?;

        let rows = stmt
            .query_map([&project, &search_pattern, &params.limit], |row| {
                Ok(StitchSearchResult {
                    id: row.get(0)?,
                    project: row.get(1)?,
                    title: row.get(2)?,
                    kind: row.get(3)?,
                    created_at: row.get(4)?,
                    last_activity_at: row.get(5)?,
                })
            })
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query search: {}", e)))?;

        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Row error: {}", e)))?);
        }
        out
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT id, project, title, kind, created_at, last_activity_at \
                 FROM stitches \
                 WHERE id LIKE ?1 OR title LIKE ?1 \
                 ORDER BY last_activity_at DESC \
                 LIMIT ?2",
            )
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Prepare search: {}", e)))?;

        let rows = stmt
            .query_map([&search_pattern, &params.limit], |row| {
                Ok(StitchSearchResult {
                    id: row.get(0)?,
                    project: row.get(1)?,
                    title: row.get(2)?,
                    kind: row.get(3)?,
                    created_at: row.get(4)?,
                    last_activity_at: row.get(5)?,
                })
            })
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query search: {}", e)))?;

        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Row error: {}", e)))?);
        }
        out
    };

    let elapsed_ms = start.elapsed().as_secs_f64() * 1_000.0;

    Ok(Json(SearchStitchesResponse {
        results,
        elapsed_ms: Some(elapsed_ms),
    }))
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: String,
    #[serde(default)]
    project: Option<String>,
    #[serde(default = "default_search_limit")]
    limit: i64,
}

fn default_search_limit() -> i64 {
    20
}
