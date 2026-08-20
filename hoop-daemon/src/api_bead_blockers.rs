//! API endpoints for bead blocker resolution
//!
//! This module provides endpoints for resolving cross-workspace blockers
//! for a bead based on Stitch-child relationships (§4.2).
//!
//! ## Blocker Resolution Algorithm
//!
//! 1. Given a bead_id, find its parent stitch via `stitch_beads`
//! 2. Find all child stitches via `stitch_links` where `kind = 'spawned'`
//! 3. For each child stitch, find all beads via `stitch_beads`
//! 4. Query bead status from br CLI (`br list --json`) in the child's workspace
//! 5. Return open beads (status != Closed) as blockers with workspace context

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

use crate::br_verbs::ReadVerb;
use crate::fleet;

/// Cross-workspace blocker entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CrossWorkspaceBlocker {
    /// Bead ID that is blocking completion
    pub bead_id: String,
    /// Workspace where the bead exists (for cross-workspace context)
    pub workspace: String,
    /// Bead title
    pub title: String,
    /// Bead status (e.g., "Open", "Closed")
    pub status: String,
    /// Bead priority (1-5)
    pub priority: i64,
    /// Bead issue type (e.g., "task", "bug", "feature")
    pub issue_type: String,
}

/// Response for GET /api/beads/{id}/blockers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct BeadBlockersResponse {
    /// The bead ID that was queried
    pub bead_id: String,
    /// List of cross-workspace blockers
    pub blockers: Vec<CrossWorkspaceBlocker>,
}

/// GET /api/beads/:id/blockers
///
/// Returns cross-workspace blockers for a bead based on Stitch-child relationships.
/// This resolves the dependency graph described in §4.2: child stitches spawned
/// from a parent stitch create implicit cross-workspace dependencies.
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/api/beads/{id}/blockers",
    tag = "blockers",
    params(
        ("id" = String, Path, description = "Bead ID to query blockers for")
    ),
    responses(
        (status = 200, description = "List of cross-workspace blockers", body = BeadBlockersResponse),
        (status = 404, description = "Bead not found")
    )
))]
pub async fn get_bead_blockers(
    Path(id): Path<String>,
    State(_state): State<crate::DaemonState>,
) -> Result<Json<BeadBlockersResponse>, (StatusCode, String)> {
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

    // Step 1: Find the parent stitch for this bead
    let parent_stitch_id: String = conn
        .query_row(
            "SELECT stitch_id FROM stitch_beads WHERE bead_id = ?1 LIMIT 1",
            [&id],
            |row| row.get(0),
        )
        .map_err(|e| {
            if e == rusqlite::Error::QueryReturnedNoRows {
                (StatusCode::NOT_FOUND, format!("Bead '{}' not found", id))
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to query stitch_beads: {}", e),
                )
            }
        })?;

    // Step 2: Find all child stitches via stitch_links (spawned kind)
    let mut child_stmt = conn
        .prepare(
            "SELECT to_stitch, workspace_to FROM stitch_links
             WHERE from_stitch = ?1 AND kind = 'spawned'",
        )
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to prepare stitch_links query: {}", e),
            )
        })?;

    let child_stitches: Vec<(String, String)> = child_stmt
        .query_map([&parent_stitch_id], |row| {
            Ok((
                row.get::<_, String>(0)?, // to_stitch
                row.get::<_, String>(1)?, // workspace_to
            ))
        })
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to query child stitches: {}", e),
            )
        })?
        .filter_map(Result::ok)
        .collect();

    if child_stitches.is_empty() {
        return Ok(Json(BeadBlockersResponse {
            bead_id: id,
            blockers: Vec::new(),
        }));
    }

    // Step 3: For each child stitch, find all beads via stitch_beads
    let mut child_bead_queries: Vec<(String, String)> = Vec::new();
    for (child_stitch_id, workspace) in child_stitches {
        let mut bead_stmt = conn
            .prepare("SELECT bead_id FROM stitch_beads WHERE stitch_id = ?1")
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to prepare stitch_beads query: {}", e),
                )
            })?;

        let bead_ids: Vec<String> = bead_stmt
            .query_map([&child_stitch_id], |row| row.get::<_, String>(0))
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to query child beads: {}", e),
                )
            })?
            .filter_map(Result::ok)
            .collect();

        for bead_id in bead_ids {
            child_bead_queries.push((bead_id, workspace.clone()));
        }
    }

    // Step 4: Query bead status from br CLI in each workspace
    let mut blockers = Vec::new();
    let mut seen_beads = HashSet::new();

    for (bead_id, workspace) in child_bead_queries {
        // Skip if we've already processed this bead (dedup across workspaces)
        if !seen_beads.insert(bead_id.clone()) {
            continue;
        }

        // Check if the workspace exists
        let workspace_path = std::path::Path::new(&workspace);
        if !workspace_path.exists() {
            tracing::warn!(
                "Workspace path does not exist for bead {}: {}",
                bead_id,
                workspace
            );
            continue;
        }

        // Query bead status via br CLI
        let mut cmd = crate::br_verbs::invoke_br_read(ReadVerb::List, &["--json"]);
        cmd.current_dir(workspace_path);

        match cmd.output() {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Ok(beads) = serde_json::from_str::<Vec<serde_json::Value>>(&stdout) {
                    for bead_json in beads {
                        let current_id = bead_json.get("id").and_then(|v| v.as_str()).unwrap_or("");

                        if current_id != bead_id {
                            continue;
                        }

                        // Extract bead properties
                        let title = bead_json
                            .get("title")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();

                        let status = bead_json
                            .get("status")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Open")
                            .to_string();

                        // Only include open beads as blockers
                        if status != "Closed" {
                            let priority = bead_json
                                .get("priority")
                                .and_then(|v| v.as_i64())
                                .unwrap_or(3);

                            let issue_type = bead_json
                                .get("issue_type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("task")
                                .to_string();

                            blockers.push(CrossWorkspaceBlocker {
                                bead_id: bead_id.clone(),
                                workspace: workspace.clone(),
                                title,
                                status,
                                priority,
                                issue_type,
                            });
                        }
                        break;
                    }
                }
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::warn!(
                    "Failed to query bead {} in workspace {}: br list failed: {}",
                    bead_id,
                    workspace,
                    stderr
                );
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to query bead {} in workspace {}: {}",
                    bead_id,
                    workspace,
                    e
                );
            }
        }
    }

    Ok(Json(BeadBlockersResponse {
        bead_id: id,
        blockers,
    }))
}

pub fn router() -> axum::Router<crate::DaemonState> {
    axum::Router::new().route(
        "/api/beads/:id/blockers",
        axum::routing::get(get_bead_blockers),
    )
}
