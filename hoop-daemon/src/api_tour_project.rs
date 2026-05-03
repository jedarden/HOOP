//! `/api/tour` endpoint — Sample-Stitches tour project management
//!
//! Provides one-click spin-up of a demo workspace with example Stitches
//! demonstrating typical patterns (voice note, agent chat, linked beads, cost-anomaly).
//!
//! Plan reference: §12 Onboarding aids (hoop-ttb.9.5)

use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use rusqlite::OpenFlags;
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use tracing::{error, info, warn};

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

use crate::DaemonState;
use crate::fleet;

/// Tour project identifier (must not conflict with real projects)
pub const TOUR_PROJECT_NAME: &str = "__hoop_tour__";

/// Tour state stored in UI state
pub const TOUR_ENABLED_KEY: &str = "tour_enabled";
pub const TOUR_PATH_KEY: &str = "tour_path";

/// Request to enable the tour project
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct EnableTourRequest {
    /// Optional custom path (defaults to ~/.hoop/tour/)
    pub path: Option<String>,
}

/// Response for tour project operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TourProjectResponse {
    /// Whether the tour project is enabled
    pub enabled: bool,
    /// Path to the tour workspace
    pub path: Option<String>,
    /// Example stitches created
    pub example_stitches: Vec<TourStitchInfo>,
}

/// Information about an example stitch in the tour
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TourStitchInfo {
    /// Stitch ID
    pub id: String,
    /// Stitch kind
    pub kind: String,
    /// Stitch title
    pub title: String,
    /// Description of what this demonstrates
    pub description: String,
}

/// POST /api/tour/enable — create the tour project
///
/// Creates a sandbox workspace with example Stitches demonstrating
/// typical HOOP patterns.
#[utoipa::path(
    post,
    path = "/api/tour/enable",
    tag = "tour",
    request_body = EnableTourRequest,
    responses(
        (status = 200, description = "Tour project enabled successfully", body = TourProjectResponse),
        (status = 500, description = "Internal server error")
    ),
)]
async fn enable_tour_project(
    State(state): State<DaemonState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Json(req): Json<EnableTourRequest>,
) -> Result<Json<TourProjectResponse>, StatusCode> {
    let operator_id = state.identity_cache.resolve(connect_info.map(|ci| ci.0));

    let db_path = fleet::db_path();
    let mut conn = rusqlite::Connection::open_with_flags(
        &db_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| {
        error!("Failed to open fleet.db: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Check if tour is already enabled
    let existing_path: Option<String> = conn
        .query_row(
            "SELECT value FROM ui_state WHERE operator_id = ?1 AND key = ?2",
            (&operator_id, TOUR_PATH_KEY),
            |row| row.get::<_, String>(0),
        )
        .ok()
        .flatten();

    if let Some(path) = existing_path {
        // Tour already enabled, return current state
        let stitches = list_tour_stitches(&conn, TOUR_PROJECT_NAME);
        return Ok(Json(TourProjectResponse {
            enabled: true,
            path: Some(path),
            example_stitches: stitches,
        }));
    }

    // Create the tour workspace directory
    let tour_path = req.path.unwrap_or_else(|| {
        let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.push(".hoop");
        home.push("tour");
        home.to_string_lossy().to_string()
    });

    let tour_path_buf = PathBuf::from(&tour_path);

    // Create the workspace structure
    fs::create_dir_all(&tour_path_buf).map_err(|e| {
        error!("Failed to create tour workspace: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Create .beads directory
    let beads_path = tour_path_buf.join(".beads");
    fs::create_dir_all(&beads_path).map_err(|e| {
        error!("Failed to create .beads directory: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Store tour state in UI state
    conn.execute(
        "INSERT INTO ui_state (operator_id, key, value, updated_at)
         VALUES (?1, ?2, ?3, datetime('now'))
         ON CONFLICT (operator_id, key) DO UPDATE SET
             value = excluded.value,
             updated_at = datetime('now')",
        (&operator_id, TOUR_ENABLED_KEY, "true"),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    conn.execute(
        "INSERT INTO ui_state (operator_id, key, value, updated_at)
         VALUES (?1, ?2, ?3, datetime('now'))
         ON CONFLICT (operator_id, key) DO UPDATE SET
             value = excluded.value,
             updated_at = datetime('now')",
        (&operator_id, TOUR_PATH_KEY, &tour_path),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create example stitches
    let mut example_stitches = Vec::new();

    // Example 1: Voice note stitch
    let voice_note_stitch = create_voice_note_example(&mut conn, &tour_path);
    example_stitches.push(voice_note_stitch);

    // Example 2: Agent chat stitch
    let agent_chat_stitch = create_agent_chat_example(&mut conn, &tour_path);
    example_stitches.push(agent_chat_stitch);

    // Example 3: Stitch with linked beads
    let linked_stitch = create_linked_beads_example(&mut conn, &tour_path);
    example_stitches.push(linked_stitch);

    // Example 4: Cost anomaly example
    let cost_stitch = create_cost_anomaly_example(&mut conn, &tour_path);
    example_stitches.push(cost_stitch);

    drop(conn);

    // Add tour project to the project cards list
    {
        use crate::ws::ProjectCardData;
        let mut projects = state.projects.write().unwrap();
        // Remove any existing tour project entry first
        projects.retain(|p| p.name != TOUR_PROJECT_NAME);
        // Add the tour project card
        projects.push(ProjectCardData {
            name: TOUR_PROJECT_NAME.to_string(),
            label: "HOOP Tour".to_string(),
            color: "#8b5cf6".to_string(), // Purple color for tour
            path: tour_path.clone(),
            degraded: false,
            runtime_state: Some("healthy".to_string()),
            runtime_error: None,
            bead_count: example_stitches.len(),
            worker_count: 0,
            active_stitch_count: 0,
            cost_today: 0.0,
            stuck_count: 0,
            last_activity: Some(chrono::Utc::now().to_rfc3339()),
        });
        // Broadcast the tour project card
        let _ = state.project_status_tx.send(projects.last().unwrap().clone());
    }

    info!(
        "Tour project enabled by operator {}: path={}, examples={}",
        operator_id,
        tour_path,
        example_stitches.len()
    );

    Ok(Json(TourProjectResponse {
        enabled: true,
        path: Some(tour_path),
        example_stitches,
    }))
}

/// DELETE /api/tour/disable — remove the tour project
///
/// Tears down the tour workspace and removes all example data.
#[utoipa::path(
    delete,
    path = "/api/tour/disable",
    tag = "tour",
    responses(
        (status = 200, description = "Tour project disabled successfully", body = TourProjectResponse),
        (status = 500, description = "Internal server error")
    ),
)]
async fn disable_tour_project(
    State(state): State<DaemonState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
) -> Result<Json<TourProjectResponse>, StatusCode> {
    let operator_id = state.identity_cache.resolve(connect_info.map(|ci| ci.0));

    let db_path = fleet::db_path();
    let mut conn = rusqlite::Connection::open_with_flags(
        &db_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get tour path before removing state
    let tour_path: Option<String> = conn
        .query_row(
            "SELECT value FROM ui_state WHERE operator_id = ?1 AND key = ?2",
            (&operator_id, TOUR_PATH_KEY),
            |row| row.get::<_, String>(0),
        )
        .ok()
        .flatten();

    // Remove tour stitches from database
    if let Err(e) = conn.execute(
        "DELETE FROM stitches WHERE project = ?1",
        (TOUR_PROJECT_NAME,),
    ) {
        warn!("Failed to remove tour stitches: {}", e);
    }

    // Remove tour state from UI state
    conn.execute(
        "DELETE FROM ui_state WHERE operator_id = ?1 AND key IN (?2, ?3)",
        (&operator_id, TOUR_ENABLED_KEY, TOUR_PATH_KEY),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    drop(conn);

    // Remove tour project from the project cards list
    {
        let mut projects = state.projects.write().unwrap();
        projects.retain(|p| p.name != TOUR_PROJECT_NAME);
    }

    // Remove the tour workspace directory
    if let Some(path) = tour_path {
        let tour_path_buf = PathBuf::from(&path);
        if tour_path_buf.exists() {
            if let Err(e) = fs::remove_dir_all(&tour_path_buf) {
                warn!(
                    "Failed to remove tour workspace directory {}: {}",
                    path, e
                );
            }
        }
    }

    info!("Tour project disabled by operator {}", operator_id);

    Ok(Json(TourProjectResponse {
        enabled: false,
        path: None,
        example_stitches: vec![],
    }))
}

/// GET /api/tour/status — check tour project status
///
/// Returns whether the tour project is enabled and its current state.
#[utoipa::path(
    get,
    path = "/api/tour/status",
    tag = "tour",
    responses(
        (status = 200, description = "Tour status retrieved successfully", body = TourProjectResponse),
        (status = 500, description = "Internal server error")
    ),
)]
async fn get_tour_status(
    State(state): State<DaemonState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
) -> Result<Json<TourProjectResponse>, StatusCode> {
    let operator_id = state.identity_cache.resolve(connect_info.map(|ci| ci.0));

    let db_path = fleet::db_path();
    let conn = rusqlite::Connection::open_with_flags(
        &db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let enabled: bool = conn
        .query_row(
            "SELECT value FROM ui_state WHERE operator_id = ?1 AND key = ?2",
            (&operator_id, TOUR_ENABLED_KEY),
            |row| {
                let value: String = row.get(0)?;
                Ok(value == "true")
            },
        )
        .ok()
        .unwrap_or(false);

    let path: Option<String> = conn
        .query_row(
            "SELECT value FROM ui_state WHERE operator_id = ?1 AND key = ?2",
            (&operator_id, TOUR_PATH_KEY),
            |row| row.get::<_, String>(0),
        )
        .ok()
        .flatten();

    let example_stitches = if enabled {
        list_tour_stitches(&conn, TOUR_PROJECT_NAME)
    } else {
        vec![]
    };

    drop(conn);

    Ok(Json(TourProjectResponse {
        enabled,
        path,
        example_stitches,
    }))
}

/// List all example stitches for the tour project
fn list_tour_stitches(conn: &rusqlite::Connection, project: &str) -> Vec<TourStitchInfo> {
    let mut stmt = conn
        .prepare("SELECT id, kind, title FROM stitches WHERE project = ?1 ORDER BY created_at")
        .unwrap_or_else(|_| {
            // Table might not exist yet
            return conn
                .prepare(
                    "SELECT id, kind, title FROM stitches WHERE project = ?1 ORDER BY created_at",
                )
                .unwrap();
        });

    let mut stitches = Vec::new();
    let rows = stmt.query_map((project,), |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    });

    if let Ok(mut rows) = rows {
        while let Some(Ok((id, kind, title))) = rows.next() {
            let description = match kind.as_str() {
                "dictated" => "Voice dictation example",
                "operator" => "AI agent conversation example",
                "ad-hoc" => "Stitch with linked beads example",
                "worker" => "Cost anomaly detection example",
                _ => "Example stitch",
            };
            stitches.push(TourStitchInfo {
                id,
                kind,
                title,
                description: description.to_string(),
            });
        }
    }

    stitches
}

/// Create an example voice note stitch (dictated)
fn create_voice_note_example(
    conn: &mut rusqlite::Connection,
    _tour_path: &str,
) -> TourStitchInfo {
    use uuid::Uuid;
    use chrono::Utc;

    let stitch_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let _ = conn.execute(
        "INSERT INTO stitches (id, project, kind, title, created_by, created_at, last_activity_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (&stitch_id, TOUR_PROJECT_NAME, "dictated", "Tour: Voice Note Demo", "system", &now, &now),
    );

    // Add example voice note message
    let msg_id = Uuid::new_v4().to_string();
    let content = serde_json::json!({
        "text": "This is an example dictated note. In HOOP, you can use voice dictation to quickly capture thoughts, meeting notes, or ideas. The dictated notes are automatically transcribed and stored in Stitches.",
        "metadata": {"source": "voice_dictation"}
    });
    let _ = conn.execute(
        "INSERT INTO stitch_messages (id, stitch_id, ts, role, content)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        (&msg_id, &stitch_id, &now, "user", &content.to_string()),
    );

    TourStitchInfo {
        id: stitch_id,
        kind: "dictated".to_string(),
        title: "Tour: Voice Note Demo".to_string(),
        description: "Voice dictation example".to_string(),
    }
}

/// Create an example agent chat stitch (operator)
fn create_agent_chat_example(
    conn: &mut rusqlite::Connection,
    _tour_path: &str,
) -> TourStitchInfo {
    use uuid::Uuid;
    use chrono::Utc;

    let stitch_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let _ = conn.execute(
        "INSERT INTO stitches (id, project, kind, title, created_by, created_at, last_activity_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (&stitch_id, TOUR_PROJECT_NAME, "operator", "Tour: Agent Chat Demo", "system", &now, &now),
    );

    // Add example conversation messages
    let msg1_id = Uuid::new_v4().to_string();
    let content1 = serde_json::json!({
        "text": "How do I create a new Stitch in HOOP?",
    });
    let _ = conn.execute(
        "INSERT INTO stitch_messages (id, stitch_id, ts, role, content)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        (&msg1_id, &stitch_id, &now, "user", &content1.to_string()),
    );

    let msg2_id = Uuid::new_v4().to_string();
    let content2 = serde_json::json!({
        "text": "To create a new Stitch in HOOP:\n\n1. Click the '+' button in the Stitches panel\n2. Choose a Stitch type (operator, dictated, worker, or ad-hoc)\n3. Give it a descriptive title\n4. Start adding content via messages, voice notes, or linked beads\n\nStitches are the primary way to organize and track work in HOOP. Each Stitch can contain messages, attachments, and links to beads for full context tracking.",
    });
    let _ = conn.execute(
        "INSERT INTO stitch_messages (id, stitch_id, ts, role, content)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        (&msg2_id, &stitch_id, &now, "assistant", &content2.to_string()),
    );

    TourStitchInfo {
        id: stitch_id,
        kind: "operator".to_string(),
        title: "Tour: Agent Chat Demo".to_string(),
        description: "AI agent conversation example".to_string(),
    }
}

/// Create an example stitch with linked beads (ad-hoc)
fn create_linked_beads_example(
    conn: &mut rusqlite::Connection,
    _tour_path: &str,
) -> TourStitchInfo {
    use uuid::Uuid;
    use chrono::Utc;

    let stitch_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let _ = conn.execute(
        "INSERT INTO stitches (id, project, kind, title, created_by, created_at, last_activity_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (&stitch_id, TOUR_PROJECT_NAME, "ad-hoc", "Tour: Linked Beads Demo", "system", &now, &now),
    );

    // Add example message about linked beads
    let msg_id = Uuid::new_v4().to_string();
    let content = serde_json::json!({
        "text": "This Stitch demonstrates linking beads for context. In HOOP, you can connect beads (tasks, issues, tracking items) to Stitches to maintain full traceability between your work items and conversations.\n\nTo link a bead:\n1. Open a Stitch\n2. Click the 'Link Bead' button\n3. Search for the bead by ID or title\n4. The bead will appear in the Stitch's context panel\n\nLinked beads let you see the full story - from the original task to the conversation that resolved it.",
    });
    let _ = conn.execute(
        "INSERT INTO stitch_messages (id, stitch_id, ts, role, content)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        (&msg_id, &stitch_id, &now, "user", &content.to_string()),
    );

    TourStitchInfo {
        id: stitch_id,
        kind: "ad-hoc".to_string(),
        title: "Tour: Linked Beads Demo".to_string(),
        description: "Stitch with linked beads example".to_string(),
    }
}

/// Create an example cost anomaly stitch (worker)
fn create_cost_anomaly_example(
    conn: &mut rusqlite::Connection,
    _tour_path: &str,
) -> TourStitchInfo {
    use uuid::Uuid;
    use chrono::Utc;

    let stitch_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let _ = conn.execute(
        "INSERT INTO stitches (id, project, kind, title, created_by, created_at, last_activity_at, classification)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        (&stitch_id, TOUR_PROJECT_NAME, "worker", "Tour: Cost Anomaly Demo", "system", &now, &now, "fleet"),
    );

    // Add example cost anomaly alert
    let msg_id = Uuid::new_v4().to_string();
    let content = serde_json::json!({
        "text": "⚠️ Cost Anomaly Detected\n\nThis Stitch demonstrates HOOP's cost monitoring capabilities. When a worker or project shows unusual spending patterns, HOOP creates an alert Stitch to track the investigation.\n\nTypical cost anomalies include:\n• Sudden spike in API usage\n• Unexpected worker runtime duration\n• Resource consumption above baseline\n\nThe Stitch format lets you:\n1. Document the investigation findings\n2. Link to related beads (incident reports, remediation tasks)\n3. Track resolution with timestamps\n4. Reference in future audits",
        "metadata": {"alert_type": "cost_anomaly", "severity": "warning"}
    });
    let _ = conn.execute(
        "INSERT INTO stitch_messages (id, stitch_id, ts, role, content)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        (&msg_id, &stitch_id, &now, "system", &content.to_string()),
    );

    TourStitchInfo {
        id: stitch_id,
        kind: "worker".to_string(),
        title: "Tour: Cost Anomaly Demo".to_string(),
        description: "Cost anomaly detection example".to_string(),
    }
}

/// Check if the tour project is enabled and get its project card
///
/// Returns None if the tour is not enabled, or Some(ProjectCardData) if enabled.
pub fn get_tour_project_card(
    conn: &rusqlite::Connection,
    operator_id: &str,
) -> Option<crate::ws::ProjectCardData> {
    use crate::ws::ProjectCardData;

    // Check if tour is enabled
    let enabled: bool = conn
        .query_row(
            "SELECT value FROM ui_state WHERE operator_id = ?1 AND key = ?2",
            (operator_id, TOUR_ENABLED_KEY),
            |row| {
                let value: String = row.get(0)?;
                Ok(value == "true")
            },
        )
        .unwrap_or(Ok(false))
        .unwrap_or(false);

    if !enabled {
        return None;
    }

    // Get tour path
    let tour_path: String = conn
        .query_row(
            "SELECT value FROM ui_state WHERE operator_id = ?1 AND key = ?2",
            (operator_id, TOUR_PATH_KEY),
            |row| row.get(0),
        )
        .unwrap_or_else(|_| Ok(String::from(".hoop/tour")))
        .unwrap_or_else(|_| String::from(".hoop/tour"));

    // Count tour stitches
    let stitch_count: usize = conn
        .query_row(
            "SELECT COUNT(*) FROM stitches WHERE project = ?1",
            (TOUR_PROJECT_NAME,),
            |row| row.get(0),
        )
        .unwrap_or(Ok(0))
        .unwrap_or(0);

    Some(ProjectCardData {
        name: TOUR_PROJECT_NAME.to_string(),
        label: "HOOP Tour".to_string(),
        color: "#8b5cf6".to_string(), // Purple color for tour
        path: tour_path,
        degraded: false,
        runtime_state: Some("healthy".to_string()),
        runtime_error: None,
        bead_count: stitch_count,
        worker_count: 0,
        active_stitch_count: 0,
        cost_today: 0.0,
        stuck_count: 0,
        last_activity: Some(chrono::Utc::now().to_rfc3339()),
    })
}

pub fn router() -> Router<DaemonState> {
    Router::new()
        .route("/api/tour/enable", post(enable_tour_project))
        .route("/api/tour/disable", delete(disable_tour_project))
        .route("/api/tour/status", axum::routing::get(get_tour_status))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tour_project_constants() {
        assert_eq!(TOUR_PROJECT_NAME, "__hoop_tour__");
        assert_eq!(TOUR_ENABLED_KEY, "tour_enabled");
        assert_eq!(TOUR_PATH_KEY, "tour_path");
    }

    #[test]
    fn test_tour_stitch_info_serialization() {
        let info = TourStitchInfo {
            id: "test-id".to_string(),
            kind: "voice".to_string(),
            title: "Test Stitch".to_string(),
            description: "Test description".to_string(),
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("voice"));
        assert!(json.contains("Test Stitch"));

        let decoded: TourStitchInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.id, "test-id");
        assert_eq!(decoded.kind, "voice");
    }

    #[test]
    fn test_tour_response_serialization() {
        let response = TourProjectResponse {
            enabled: true,
            path: Some("/tmp/tour".to_string()),
            example_stitches: vec![],
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("/tmp/tour"));
    }
}
