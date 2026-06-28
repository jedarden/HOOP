//! Pattern write API - CRUD operations for Patterns
//!
//! POST /api/patterns — create pattern
//! PUT /api/patterns/:id — update pattern
//! DELETE /api/patterns/:id — delete pattern
//! POST /api/patterns/:id/members — add stitch member
//! DELETE /api/patterns/:id/members/:stitch_id — remove stitch member
//! POST /api/patterns/:id/queries — add saved query
//! DELETE /api/patterns/:id/queries/:query — remove saved query

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::fleet;

// Needed for URL decoding in remove_query
use urlencoding;

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreatePatternRequest {
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub owner: Option<String>,
    pub deadline: Option<String>,
    pub parent_pattern: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePatternRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub owner: Option<String>,
    pub deadline: Option<String>,
    pub parent_pattern: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
    pub stitch_id: String,
}

#[derive(Debug, Deserialize)]
pub struct AddQueryRequest {
    pub query: String,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PatternResponse {
    pub pattern: PatternRow,
}

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PatternRow {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_pattern: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MessageResponse {
    pub message: String,
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<crate::DaemonState> {
    Router::new()
        .route("/api/patterns", post(create_pattern))
        .route("/api/patterns/:id", put(update_pattern))
        .route("/api/patterns/:id", delete(delete_pattern))
        .route("/api/patterns/:id/members", post(add_member))
        .route(
            "/api/patterns/:id/members/:stitch_id",
            delete(remove_member),
        )
        .route("/api/patterns/:id/queries", post(add_query))
        .route("/api/patterns/:id/queries/:query", delete(remove_query))
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn create_pattern(
    State(_state): State<crate::DaemonState>,
    Json(req): Json<CreatePatternRequest>,
) -> Result<Json<PatternResponse>, (StatusCode, String)> {
    let db_path = fleet::db_path();
    let conn = Connection::open(&db_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("open fleet.db: {e}")))?;

    // Validate status
    let status = req.status.unwrap_or_else(|| "planned".to_string());
    if !is_valid_status(&status) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Invalid status '{}'. Valid statuses are: planned, active, blocked, done, abandoned", status),
        ));
    }

    // Validate parent_pattern exists if provided
    if let Some(ref parent_id) = req.parent_pattern {
        if !pattern_exists(&conn, parent_id)? {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Parent pattern '{}' not found", parent_id),
            ));
        }
    }

    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO patterns (id, title, description, status, owner, deadline, parent_pattern, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            &id,
            &req.title,
            &req.description,
            &status,
            &req.owner,
            &req.deadline,
            &req.parent_pattern,
            &now,
            &now,
        ],
    )
    .map_err(|e| {
        if e.to_string().contains("Pattern parent cycle detected")
            || e.to_string().contains("Pattern cannot reference itself")
        {
            (StatusCode::BAD_REQUEST, e.to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("create pattern: {e}"))
        }
    })?;

    let pattern = PatternRow {
        id,
        title: req.title,
        description: req.description,
        status,
        owner: req.owner,
        deadline: req.deadline,
        parent_pattern: req.parent_pattern,
        created_at: now.clone(),
        updated_at: now,
        closed_at: None,
    };

    Ok(Json(PatternResponse { pattern }))
}

async fn update_pattern(
    Path(pattern_id): Path<String>,
    Json(req): Json<UpdatePatternRequest>,
) -> Result<Json<PatternResponse>, (StatusCode, String)> {
    let db_path = fleet::db_path();
    let conn = Connection::open(&db_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("open fleet.db: {e}")))?;

    // Check pattern exists
    let current_status: String = conn
        .query_row("SELECT status FROM patterns WHERE id = ?1", params![&pattern_id], |row| {
            row.get(0)
        })
        .map_err(|e| {
            if e == rusqlite::Error::QueryReturnedNoRows {
                (
                    StatusCode::NOT_FOUND,
                    format!("Pattern '{}' not found", pattern_id),
                )
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
            }
        })?;

    // Validate status transition
    if let Some(ref new_status) = req.status {
        if !is_valid_transition(&current_status, new_status) {
            return Err((
                StatusCode::BAD_REQUEST,
                format!(
                    "Invalid status transition: '{}' → '{}'. Valid transitions from '{}' are: {}",
                    current_status,
                    new_status,
                    current_status,
                    valid_transitions_from(&current_status).join(", ")
                ),
            ));
        }
    }

    // Validate parent_pattern exists if provided
    if let Some(ref parent_id) = req.parent_pattern {
        if !pattern_exists(&conn, parent_id)? {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Parent pattern '{}' not found", parent_id),
            ));
        }
    }

    // Build update query dynamically
    let mut updates = Vec::new();
    let mut values: Vec<&dyn rusqlite::ToSql> = Vec::new();

    if req.title.is_some() {
        updates.push("title = ?");
    }
    if req.description.is_some() {
        updates.push("description = ?");
    }
    if req.status.is_some() {
        updates.push("status = ?");
    }
    if req.owner.is_some() {
        updates.push("owner = ?");
    }
    if req.deadline.is_some() {
        updates.push("deadline = ?");
    }
    if req.parent_pattern.is_some() {
        updates.push("parent_pattern = ?");
    }
    updates.push("updated_at = ?");

    if let Some(ref title) = req.title {
        values.push(title);
    }
    if let Some(ref description) = req.description {
        values.push(description);
    }
    if let Some(ref status) = req.status {
        values.push(status);
    }
    if let Some(ref owner) = req.owner {
        values.push(owner);
    }
    if let Some(ref deadline) = req.deadline {
        values.push(deadline);
    }
    if let Some(ref parent_pattern) = req.parent_pattern {
        values.push(parent_pattern);
    }
    let now = chrono::Utc::now().to_rfc3339();
    values.push(&now);
    values.push(&pattern_id);

    let query = format!(
        "UPDATE patterns SET {} WHERE id = ?",
        updates.join(", ")
    );

    conn.execute(&query, params_from_slice(&values))
        .map_err(|e| {
            if e.to_string().contains("Pattern parent cycle detected")
                || e.to_string().contains("Pattern cannot reference itself")
            {
                (StatusCode::BAD_REQUEST, e.to_string())
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("update pattern: {e}"))
            }
        })?;

    // Fetch updated pattern
    let pattern = conn
        .query_row(
            "SELECT id, title, description, status, owner, deadline, parent_pattern, created_at, updated_at
             FROM patterns WHERE id = ?1",
            params![&pattern_id],
            |row| {
                Ok(PatternRow {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    status: row.get(3)?,
                    owner: row.get(4)?,
                    deadline: row.get(5)?,
                    parent_pattern: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                    closed_at: None,
                })
            },
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("fetch pattern: {e}")))?;

    Ok(Json(PatternResponse { pattern }))
}

async fn delete_pattern(
    Path(pattern_id): Path<String>,
) -> Result<Json<MessageResponse>, (StatusCode, String)> {
    let db_path = fleet::db_path();
    let conn = Connection::open(&db_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("open fleet.db: {e}")))?;

    // Check pattern exists
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM patterns WHERE id = ?1",
            params![&pattern_id],
            |row| row.get::<_, i64>(0).map(|c| c > 0),
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))?;

    if !exists {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Pattern '{}' not found", pattern_id),
        ));
    }

    // Delete will cascade to pattern_members and pattern_queries
    conn.execute("DELETE FROM patterns WHERE id = ?1", params![&pattern_id])
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("delete pattern: {e}")))?;

    Ok(Json(MessageResponse {
        message: format!("Pattern '{}' deleted", pattern_id),
    }))
}

async fn add_member(
    Path(pattern_id): Path<String>,
    Json(req): Json<AddMemberRequest>,
) -> Result<Json<MessageResponse>, (StatusCode, String)> {
    let db_path = fleet::db_path();
    let conn = Connection::open(&db_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("open fleet.db: {e}")))?;

    // Check pattern exists
    if !pattern_exists(&conn, &pattern_id)? {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Pattern '{}' not found", pattern_id),
        ));
    }

    // Check stitch exists
    let stitch_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM stitches WHERE id = ?1",
            params![&req.stitch_id],
            |row| row.get::<_, i64>(0).map(|c| c > 0),
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))?;

    if !stitch_exists {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Stitch '{}' not found", req.stitch_id),
        ));
    }

    // Insert member (idempotent - ON CONFLICT DO NOTHING)
    conn.execute(
        "INSERT OR IGNORE INTO pattern_members (pattern_id, stitch_id) VALUES (?1, ?2)",
        params![&pattern_id, &req.stitch_id],
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("add member: {e}")))?;

    Ok(Json(MessageResponse {
        message: format!(
            "Stitch '{}' added to pattern '{}'",
            req.stitch_id, pattern_id
        ),
    }))
}

async fn remove_member(
    Path((pattern_id, stitch_id)): Path<(String, String)>,
) -> Result<Json<MessageResponse>, (StatusCode, String)> {
    let db_path = fleet::db_path();
    let conn = Connection::open(&db_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("open fleet.db: {e}")))?;

    let rows_affected = conn
        .execute(
            "DELETE FROM pattern_members WHERE pattern_id = ?1 AND stitch_id = ?2",
            params![&pattern_id, &stitch_id],
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("remove member: {e}")))?;

    if rows_affected == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            format!(
                "Member relationship not found: pattern '{}', stitch '{}'",
                pattern_id, stitch_id
            ),
        ));
    }

    Ok(Json(MessageResponse {
        message: format!(
            "Stitch '{}' removed from pattern '{}'",
            stitch_id, pattern_id
        ),
    }))
}

async fn add_query(
    Path(pattern_id): Path<String>,
    Json(req): Json<AddQueryRequest>,
) -> Result<Json<MessageResponse>, (StatusCode, String)> {
    let db_path = fleet::db_path();
    let conn = Connection::open(&db_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("open fleet.db: {e}")))?;

    // Check pattern exists
    if !pattern_exists(&conn, &pattern_id)? {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Pattern '{}' not found", pattern_id),
        ));
    }

    // Insert query (idempotent)
    conn.execute(
        "INSERT OR IGNORE INTO pattern_queries (pattern_id, saved_query) VALUES (?1, ?2)",
        params![&pattern_id, &req.query],
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("add query: {e}")))?;

    Ok(Json(MessageResponse {
        message: format!("Query added to pattern '{}'", pattern_id),
    }))
}

async fn remove_query(
    Path((pattern_id, query)): Path<(String, String)>,
) -> Result<Json<MessageResponse>, (StatusCode, String)> {
    let db_path = fleet::db_path();
    let conn = Connection::open(&db_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("open fleet.db: {e}")))?;

    // URL decode the query parameter
    let query = urlencoding::decode(&query)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("decode query: {e}")))?
        .into_owned();

    let rows_affected = conn
        .execute(
            "DELETE FROM pattern_queries WHERE pattern_id = ?1 AND saved_query = ?2",
            params![&pattern_id, &query],
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("remove query: {e}")))?;

    if rows_affected == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Query not found in pattern '{}'", pattern_id),
        ));
    }

    Ok(Json(MessageResponse {
        message: format!("Query removed from pattern '{}'", pattern_id),
    }))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Check if a pattern exists by ID
fn pattern_exists(conn: &Connection, pattern_id: &str) -> Result<bool, (StatusCode, String)> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM patterns WHERE id = ?1",
            params![pattern_id],
            |row| row.get(0),
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("check pattern exists: {e}")))?;
    Ok(count > 0)
}

/// Check if a status string is valid
fn is_valid_status(status: &str) -> bool {
    matches!(status, "planned" | "active" | "blocked" | "done" | "abandoned")
}

/// Valid status transitions
///
/// From plan reference §4.7:
/// - Done → Active is allowed (reopening)
/// - Abandoned → Done is NOT allowed (must go through Active first)
/// - Any status can move to Abandoned
/// - Planned → Active is allowed (starting work)
/// - Active → Blocked is allowed (blocked by dependency)
/// - Blocked → Active is allowed (unblocked)
/// - Active → Done is allowed (completion)
fn is_valid_transition(from: &str, to: &str) -> bool {
    if from == to {
        return true; // No-op is allowed
    }

    match (from, to) {
        // Reopening: Done → Active
        ("done", "active") => true,
        // Starting: Planned → Active
        ("planned", "active") => true,
        // Blocking: Active → Blocked
        ("active", "blocked") => true,
        // Unblocking: Blocked → Active
        ("blocked", "active") => true,
        // Completion: Active → Done
        ("active", "done") => true,
        // Abandonment: Any → Abandoned
        (_, "abandoned") => true,
        // Recovery: Abandoned → Active (reactivating)
        ("abandoned", "active") => true,
        // All other transitions are invalid
        _ => false,
    }
}

/// Get list of valid transition target statuses from a given status
fn valid_transitions_from(from: &str) -> Vec<&'static str> {
    match from {
        "planned" => vec!["active", "abandoned"],
        "active" => vec!["blocked", "done", "abandoned"],
        "blocked" => vec!["active", "abandoned"],
        "done" => vec!["active", "abandoned"],
        "abandoned" => vec!["active"],
        _ => vec![],
    }
}

/// Helper to convert Vec<&dyn ToSql> to &[&dyn ToSql] for rusqlite
fn params_from_slice<'a>(values: &'a [&'a dyn rusqlite::ToSql]) -> &[&'a dyn rusqlite::ToSql] {
    values
}
