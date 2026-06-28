//! Bead file links API endpoints
//!
//! `GET /api/beads/{bead_id}/files` — returns files touched by a specific bead
//! with revision information for artifact-aware navigation.
//!
//! This enables artifact-aware links: from a bead view, navigate to the file browser
//! at the right revision to see what files were changed. Combined with the net-diff
//! and file blame APIs, this provides complete traceability from bead → file → revision.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::{bead_commit_index, id_validators};

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BeadFileLink {
    /// File path relative to workspace root
    pub path: String,
    /// Workspace containing the file
    pub workspace: String,
    /// Git SHA of the commit that modified this file (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
    /// Timestamp of the commit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_ts: Option<String>,
    /// Number of lines added/removed (if available from diff)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub added: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removed: Option<i64>,
}

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BeadFilesResponse {
    pub bead_id: String,
    pub files: Vec<BeadFileLink>,
    /// Total files touched
    pub total_files: usize,
    /// Earliest commit timestamp
    pub earliest_ts: Option<String>,
    /// Latest commit timestamp
    pub latest_ts: Option<String>,
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<crate::DaemonState> {
    Router::new().route("/api/beads/{bead_id}/files", get(get_bead_files))
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/// GET /api/beads/{bead_id}/files — get files touched by a bead
///
/// Returns all files modified by commits linked to this bead, with revision
/// information for artifact-aware navigation. Each file includes the workspace,
/// commit SHA, and timestamp, enabling navigation to the file browser at the
/// right revision.
#[utoipa::path(
    get,
    path = "/api/beads/{bead_id}/files",
    tag = "beads",
    params(
        ("bead_id" = String, Path, description = "Bead ID (e.g., hoop-ttb.4.12)")
    ),
    responses(
        (status = 200, description = "Files touched by the bead", body = BeadFilesResponse),
        (status = 400, description = "Invalid bead ID"),
        (status = 404, description = "Bead not found or has no file links")
    )
)]
async fn get_bead_files(
    Path(bead_id): Path<String>,
) -> Result<Json<BeadFilesResponse>, (StatusCode, String)> {
    id_validators::validate_bead_id(&bead_id).map_err(id_validators::rejection)?;

    // Query bead_commits for this bead
    let commits = bead_commit_index::get_commits_for_bead(&bead_id)
        .map_err(|e| {
            tracing::error!("Failed to query bead_commits for {}: {}", bead_id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
        })?;

    if commits.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            format!("No file links found for bead {}", bead_id),
        ));
    }

    // Group files by unique (workspace, path) pairs
    let mut file_map: std::collections::HashMap<(String, String), BeadFileLink> = std::collections::HashMap::new();

    for commit in commits {
        // Get files changed in this commit using git diff
        let files = get_files_for_commit(&commit.workspace, &commit.sha).unwrap_or_default();

        for file_path in files {
            let key = (commit.workspace.clone(), file_path.clone());
            let entry = file_map.entry(key).or_insert(BeadFileLink {
                path: file_path,
                workspace: commit.workspace.clone(),
                sha: Some(commit.sha.clone()),
                commit_ts: Some(commit.ts.clone()),
                added: None,
                removed: None,
            });

            // Update to latest commit (timestamps are ISO-8601, lexicographic order works)
            if Some(commit.ts.as_str()) > entry.commit_ts.as_deref() {
                entry.sha = Some(commit.sha.clone());
                entry.commit_ts = Some(commit.ts.clone());
            }
        }
    }

    let mut files: Vec<BeadFileLink> = file_map.into_values().collect();
    // Sort by workspace then path
    files.sort_by(|a, b| {
        a.workspace
            .cmp(&b.workspace)
            .then_with(|| a.path.cmp(&b.path))
    });

    // Calculate timestamp range
    let timestamps: Vec<&str> = files
        .iter()
        .filter_map(|f| f.commit_ts.as_deref())
        .collect();
    let (earliest_ts, latest_ts) = if timestamps.is_empty() {
        (None, None)
    } else {
        (
            Some(timestamps.iter().min().unwrap().to_string()),
            Some(timestamps.iter().max().unwrap().to_string()),
        )
    };

    let total_files = files.len();

    Ok(Json(BeadFilesResponse {
        bead_id,
        files,
        total_files,
        earliest_ts,
        latest_ts,
    }))
}

// ---------------------------------------------------------------------------
// Git helpers
// ---------------------------------------------------------------------------

/// Get list of files changed in a commit using git diff
fn get_files_for_commit(workspace: &str, sha: &str) -> anyhow::Result<Vec<String>> {
    let output = std::process::Command::new("git")
        .args(["-C", workspace, "diff", "--name-only", &format!("{}^..{}", sha, sha)])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git diff failed for {}: {}", sha, stderr.trim());
    }

    let stdout = String::from_utf8(output.stdout)?;
    let files: Vec<String> = stdout
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(files)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bead_files_response_serialization() {
        let response = BeadFilesResponse {
            bead_id: "hoop-ttb.4.12".to_string(),
            files: vec![
                BeadFileLink {
                    path: "src/main.rs".to_string(),
                    workspace: "/home/user/project".to_string(),
                    sha: Some("abc123".to_string()),
                    commit_ts: Some("2024-01-01T00:00:00Z".to_string()),
                    added: Some(10),
                    removed: Some(5),
                },
            ],
            total_files: 1,
            earliest_ts: Some("2024-01-01T00:00:00Z".to_string()),
            latest_ts: Some("2024-01-01T00:00:00Z".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("src/main.rs"));
        assert!(json.contains("abc123"));
    }
}
