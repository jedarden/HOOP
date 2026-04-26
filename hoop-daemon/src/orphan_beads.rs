//! Orphan bead detection and management
//!
//! An "orphan bead" is a bead with no `stitch:*` label — it was created
//! outside of HOOP's Stitch workflow (e.g., via `br create` in a terminal).
//!
//! This module provides:
//! - Detection of orphan beads per project
//! - Metric updates for `hoop_orphan_bead_count{project}`
//! - API for attaching orphan beads to existing Stitches

use crate::br_verbs::{invoke_br_read, ReadVerb};
use crate::metrics;
use crate::Bead;
use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;
use tracing::{debug, info, warn};

/// Summary of an orphan bead
#[derive(Debug, Clone, serde::Serialize)]
pub struct OrphanBead {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: i64,
    pub issue_type: String,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: String,
    pub dependencies: Vec<String>,
    pub labels: Vec<String>,
}

/// Response for the orphans list endpoint
#[derive(Debug, Clone, serde::Serialize)]
pub struct OrphansResponse {
    pub project: String,
    pub orphans: Vec<OrphanBead>,
    pub total_count: usize,
}

/// Detect orphan beads in a project.
///
/// An orphan bead is one with no `stitch:*` label.
/// This function queries br list --json and checks labels directly.
pub fn detect_orphans(project_name: &str, project_path: &Path) -> Result<OrphansResponse> {
    let mut cmd = invoke_br_read(ReadVerb::List, &["--json"]);
    cmd.current_dir(project_path);

    let output = cmd
        .output()
        .context("Failed to run br list --json")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("br list failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let beads: Vec<serde_json::Value> = serde_json::from_str(&stdout)
        .context("Failed to parse br list output")?;

    let mut orphans = Vec::new();

    for bead_json in beads {
        let bead_id = bead_json
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if bead_id.is_empty() {
            continue;
        }

        // Get labels from the bead JSON directly
        let labels = bead_json
            .get("labels")
            .and_then(|l| l.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let has_stitch_label = labels
            .iter()
            .any(|l| l.starts_with("stitch:"));

        if !has_stitch_label {
            // This is an orphan
            if let Some(b) = serde_json::from_value::<Bead>(bead_json).ok() {
                orphans.push(OrphanBead {
                    id: b.id,
                    title: b.title,
                    status: format!("{:?}", b.status).to_lowercase(),
                    priority: b.priority,
                    issue_type: format!("{:?}", b.issue_type).to_lowercase(),
                    created_at: b.created_at.to_rfc3339(),
                    updated_at: b.updated_at.to_rfc3339(),
                    created_by: b.created_by,
                    dependencies: b.dependencies,
                    labels,
                });
            }
        }
    }

    // Update the metric
    metrics::metrics().hoop_orphan_bead_count.set(
        &[project_name],
        orphans.len() as i64,
    );

    Ok(OrphansResponse {
        project: project_name.to_string(),
        total_count: orphans.len(),
        orphans,
    })
}

/// Attach an orphan bead to an existing Stitch.
///
/// Creates a `stitch_beads` row with `kind = 'referenced'`.
/// This associates the bead with the Stitch without implying
/// that the Stitch created it.
pub fn attach_orphan_to_stitch(
    stitch_id: &str,
    bead_id: &str,
    workspace: &str,
) -> Result<()> {
    let db_path = crate::fleet::db_path();
    let conn = Connection::open(&db_path)
        .context("Failed to open fleet.db")?;

    // Get canonical workspace path
    let canonical_ws = std::fs::canonicalize(workspace)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| workspace.to_string());

    // Check if the link already exists
    let exists: bool = conn.query_row(
        "SELECT 1 FROM stitch_beads WHERE stitch_id = ?1 AND bead_id = ?2",
        [stitch_id, bead_id],
        |_| Ok(true),
    ).unwrap_or(false);

    if exists {
        info!(
            "Bead {} is already linked to stitch {}",
            bead_id, stitch_id
        );
        return Ok(());
    }

    // Insert the reference link
    conn.execute(
        "INSERT INTO stitch_beads (stitch_id, bead_id, workspace, canonical_workspace, relationship)
         VALUES (?1, ?2, ?3, ?4, 'referenced')",
        [stitch_id, bead_id, workspace, &canonical_ws],
    ).context("Failed to insert stitch_beads row")?;

    info!(
        "Attached orphan bead {} to stitch {} (referenced)",
        bead_id, stitch_id
    );

    // Note: We don't add the stitch: label to the bead here.
    // The bead remains an "orphan" in terms of labels, but is now
    // linked in the database for UI display.

    Ok(())
}

/// Update orphan bead counts for all projects.
///
/// Should be called periodically to keep metrics fresh.
pub fn update_all_orphan_metrics(projects: &[crate::ws::ProjectCardData]) {
    for project in projects {
        let name = &project.name;
        let path = std::path::Path::new(&project.path);

        match detect_orphans(name, path) {
            Ok(result) => {
                debug!(
                    "Updated orphan count for {}: {}",
                    name, result.total_count
                );
            }
            Err(e) => {
                warn!("Failed to update orphan count for {}: {}", name, e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orphan_bead_response_serialization() {
        let response = OrphansResponse {
            project: "test-project".to_string(),
            total_count: 1,
            orphans: vec![OrphanBead {
                id: "hoop-ttb.1".to_string(),
                title: "Test orphan".to_string(),
                status: "open".to_string(),
                priority: 2,
                issue_type: "task".to_string(),
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
                created_by: "user".to_string(),
                dependencies: vec![],
                labels: vec![],
            }],
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("hoop-ttb.1"));
        assert!(json.contains("test-project"));
    }

    #[test]
    fn test_stitch_label_detection() {
        let labels_with_stitch = vec![
            "stitch:abc123".to_string(),
            "urgent".to_string(),
        ];
        let labels_without = vec![
            "urgent".to_string(),
            "bug".to_string(),
        ];

        assert!(labels_with_stitch.iter().any(|l| l.starts_with("stitch:")));
        assert!(!labels_without.iter().any(|l| l.starts_with("stitch:")));
    }
}
