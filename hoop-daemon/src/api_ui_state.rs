//! `/api/ui/state` endpoint — per-operator UI state persistence
//!
//! Provides per-operator UI state persistence across sessions.
//! Keyed by Tailscale identity (with fallback to OS user).
//!
//! Plan reference: §6 Phase 7 deliverable 4

use axum::{
    extract::{ConnectInfo, Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, put, delete},
    Router,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use crate::DaemonState;

/// Current UI state schema version
const UI_STATE_SCHEMA_VERSION: &str = "1.1.0";

/// Response for GET /api/ui/state
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UiStateResponse {
    /// Schema version for compatibility tracking
    pub schema_version: String,
    /// All UI state for the current operator (key → value)
    pub state: HashMap<String, String>,
    /// Operator identity (Tailscale or OS fallback)
    pub operator_id: String,
}

/// Request body for PUT /api/ui/state
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UiStateUpdate {
    /// State key to update (e.g., "pinned_projects", "active_project", "filters")
    pub key: String,
    /// JSON-encoded value
    pub value: String,
}

/// Batch update request for PUT /api/ui/state/batch
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UiStateBatchUpdate {
    /// Multiple key-value pairs to update
    pub state: HashMap<String, String>,
}

/// GET /api/ui/state — retrieve all UI state for the current operator
///
/// Returns all UI state for the current operator, keyed by Tailscale identity
/// (with fallback to OS user). State values are JSON-encoded strings.
#[cfg_attr(feature = "openapi", utoipa::path(
get,
path = "/api/ui/state",
tag = "ui_state",
responses(
    (status = 200, description = "UI state retrieved successfully", body = UiStateResponse),
    (status = 500, description = "Internal server error")
),
))]
async fn get_ui_state(
    State(state): State<DaemonState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
) -> Result<Json<UiStateResponse>, StatusCode> {
    let operator_id = state.identity_cache.resolve(connect_info.map(|ci| ci.0));

    let mut state_map = HashMap::new();

    // Query all state for this operator
    let db_path = crate::fleet::db_path();
    let conn = Connection::open(&db_path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut stmt = conn
        .prepare(
            "SELECT key, value FROM ui_state WHERE operator_id = ?1",
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut rows = stmt
        .query((&operator_id,))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    while let Ok(Some(row)) = rows.next() {
        let key: String = row.get(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let value: String = row.get(1).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        state_map.insert(key, value);
    }

    drop(conn); // Release lock before returning

    Ok(Json(UiStateResponse {
        schema_version: UI_STATE_SCHEMA_VERSION.to_string(),
        state: state_map,
        operator_id,
    }))
}

/// PUT /api/ui/state — upsert a single UI state key-value pair
///
/// Updates or inserts a single UI state key-value pair for the current operator.
/// The value must be a valid JSON string (e.g., "\"dark\"" for a string, "[\"project1\",\"project2\"]" for an array).
#[cfg_attr(feature = "openapi", utoipa::path(
put,
path = "/api/ui/state",
tag = "ui_state",
request_body = UiStateUpdate,
responses(
    (status = 200, description = "UI state updated successfully", body = UiStateResponse),
    (status = 400, description = "Invalid request parameters"),
    (status = 500, description = "Internal server error")
),
))]
async fn put_ui_state(
    State(state): State<DaemonState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Json(payload): Json<UiStateUpdate>,
) -> Result<Json<UiStateResponse>, StatusCode> {
    let operator_id = state.identity_cache.resolve(connect_info.map(|ci| ci.0));

    let db_path = crate::fleet::db_path();
    let conn = Connection::open(&db_path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    conn.execute(
        "INSERT INTO ui_state (operator_id, key, value, updated_at)
         VALUES (?1, ?2, ?3, datetime('now'))
         ON CONFLICT (operator_id, key) DO UPDATE SET
             value = excluded.value,
             updated_at = datetime('now')",
        (&operator_id, &payload.key, &payload.value),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    drop(conn);

    // Fetch and return updated state
    let conn = Connection::open(&db_path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut state_map = HashMap::new();

    let mut stmt = conn
        .prepare("SELECT key, value FROM ui_state WHERE operator_id = ?1")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut rows = stmt
        .query((&operator_id,))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    while let Ok(Some(row)) = rows.next() {
        let key: String = row.get(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let value: String = row.get(1).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        state_map.insert(key, value);
    }

    drop(conn);

    Ok(Json(UiStateResponse {
        schema_version: UI_STATE_SCHEMA_VERSION.to_string(),
        state: state_map,
        operator_id,
    }))
}

/// PUT /api/ui/state/batch
///
/// Upserts multiple UI state key-value pairs for the current operator.
async fn put_ui_state_batch(
    State(state): State<DaemonState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Json(payload): Json<UiStateBatchUpdate>,
) -> Result<Json<UiStateResponse>, StatusCode> {
    let operator_id = state.identity_cache.resolve(connect_info.map(|ci| ci.0));

    let db_path = crate::fleet::db_path();
    let mut conn = Connection::open(&db_path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tx = conn.transaction().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Batch upsert all key-value pairs
    for (key, value) in &payload.state {
        tx.execute(
            "INSERT INTO ui_state (operator_id, key, value, updated_at)
             VALUES (?1, ?2, ?3, datetime('now'))
             ON CONFLICT (operator_id, key) DO UPDATE SET
                 value = excluded.value,
                 updated_at = datetime('now')",
            (&operator_id, key, value),
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    tx.commit().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    drop(conn);

    // Fetch and return updated state
    let conn = Connection::open(&db_path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut state_map = HashMap::new();

    let mut stmt = conn
        .prepare("SELECT key, value FROM ui_state WHERE operator_id = ?1")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut rows = stmt
        .query((&operator_id,))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    while let Ok(Some(row)) = rows.next() {
        let key: String = row.get(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let value: String = row.get(1).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        state_map.insert(key, value);
    }

    drop(conn);

    Ok(Json(UiStateResponse {
        schema_version: UI_STATE_SCHEMA_VERSION.to_string(),
        state: state_map,
        operator_id,
    }))
}

/// DELETE /api/ui/state/:key
///
/// Deletes a single UI state key for the current operator.
async fn delete_ui_state(
    State(state): State<DaemonState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Path(key): Path<String>,
) -> Result<Json<UiStateResponse>, StatusCode> {
    let operator_id = state.identity_cache.resolve(connect_info.map(|ci| ci.0));

    let db_path = crate::fleet::db_path();
    let conn = Connection::open(&db_path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    conn.execute(
        "DELETE FROM ui_state WHERE operator_id = ?1 AND key = ?2",
        (&operator_id, &key),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    drop(conn);

    // Fetch and return remaining state
    let conn = Connection::open(&db_path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut state_map = HashMap::new();

    let mut stmt = conn
        .prepare("SELECT key, value FROM ui_state WHERE operator_id = ?1")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut rows = stmt
        .query((&operator_id,))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    while let Ok(Some(row)) = rows.next() {
        let key: String = row.get(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let value: String = row.get(1).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        state_map.insert(key, value);
    }

    drop(conn);

    Ok(Json(UiStateResponse {
        schema_version: UI_STATE_SCHEMA_VERSION.to_string(),
        state: state_map,
        operator_id,
    }))
}

pub fn router() -> Router<DaemonState> {
    Router::new()
        .route("/api/ui/state", get(get_ui_state).put(put_ui_state))
        .route("/api/ui/state/batch", put(put_ui_state_batch))
        .route("/api/ui/state/:key", delete(delete_ui_state))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_state_response_serialization() {
        let mut state = HashMap::new();
        state.insert("stitch_view_mode".to_string(), "\"list\"".to_string());
        state.insert("archive_filter".to_string(), "{\"showArchived\":false}".to_string());

        let response = UiStateResponse {
            schema_version: UI_STATE_SCHEMA_VERSION.to_string(),
            state,
            operator_id: "tailscale:user@example.com".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("stitch_view_mode"));
        assert!(json.contains("tailscale:user@example.com"));
    }

    #[test]
    fn test_ui_state_update_deserialization() {
        let json = r#"{"key":"stitch_view_mode","value":"\"list\""}"#;
        let update: UiStateUpdate = serde_json::from_str(json).unwrap();
        assert_eq!(update.key, "stitch_view_mode");
        assert_eq!(update.value, "\"list\"");
    }
}
