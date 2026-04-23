//! REST API endpoints for bead creation and listing
//!
//! Endpoints:
//! - GET  /api/p/:project/beads  — list open beads for the dep picker
//! - POST /api/p/:project/beads  — create a bead via `br create`

use crate::br_verbs::{invoke_br_read, invoke_br_write, ReadVerb, WriteVerb};
use crate::fleet::{self, ActionKind, ActionResult, BeadActionArgs, BeadSource};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing::warn;

/// Open bead summary for the dep picker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeadSummary {
    pub id: String,
    pub title: String,
    pub issue_type: String,
    pub priority: i64,
    pub dependencies: Vec<String>,
}

/// Request body for creating a bead
#[derive(Debug, Deserialize)]
pub struct CreateBeadRequest {
    pub title: String,
    pub description: Option<String>,
    pub issue_type: Option<String>,
    pub priority: Option<i64>,
    pub dependencies: Option<Vec<String>>,
    pub assignee: Option<String>,
    pub labels: Option<Vec<String>>,
    /// Source of the bead creation (form, chat, bulk, template)
    #[serde(default)]
    pub source: String,
    /// Stitch ID if this bead is part of a Stitch
    pub stitch_id: Option<String>,
}

/// Response after creating a bead
#[derive(Debug, Serialize)]
pub struct CreateBeadResponse {
    pub id: String,
    pub title: String,
}

pub fn router() -> Router<crate::DaemonState> {
    Router::new()
        .route("/api/p/{project}/beads", get(list_open_beads))
        .route("/api/p/{project}/beads", post(create_bead))
}

/// GET /api/p/:project/beads — list open beads from the project's .beads/issues.jsonl
async fn list_open_beads(
    Path(project): Path<String>,
    State(state): State<crate::DaemonState>,
) -> Result<Json<Vec<BeadSummary>>, (StatusCode, String)> {
    let project_path = resolve_project_path(&project, &state)?;

    let beads_path = project_path.join(".beads").join("issues.jsonl");
    if !beads_path.exists() {
        return Ok(Json(vec![]));
    }

    let result = tokio::task::spawn_blocking(move || {
        let content = std::fs::read_to_string(&beads_path)
            .map_err(|e| format!("Failed to read issues.jsonl: {}", e))?;

        let mut seen_ids: std::collections::HashMap<String, crate::Bead> =
            std::collections::HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            match serde_json::from_str::<crate::Bead>(line) {
                Ok(bead) => {
                    seen_ids.insert(bead.id.clone(), bead);
                }
                Err(e) => {
                    warn!("Failed to parse bead line: {}", e);
                }
            }
        }

        let summaries: Vec<BeadSummary> = seen_ids
            .into_values()
            .filter(|b| matches!(b.status, crate::BeadStatus::Open))
            .map(|b| BeadSummary {
                id: b.id,
                title: b.title,
                issue_type: bead_type_str(&b.issue_type),
                priority: b.priority,
                dependencies: b.dependencies,
            })
            .collect();

        Ok::<Vec<BeadSummary>, String>(summaries)
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task failed: {}", e)))?
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(result))
}

/// POST /api/p/:project/beads — create a bead via br create
async fn create_bead(
    Path(project): Path<String>,
    State(state): State<crate::DaemonState>,
    Json(req): Json<CreateBeadRequest>,
) -> Result<Json<CreateBeadResponse>, (StatusCode, String)> {
    if req.title.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "title is required".to_string()));
    }

    let project_path = resolve_project_path(&project, &state)?;

    let beads_dir = project_path.join(".beads");
    if !beads_dir.exists() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("Project '{}' has no .beads directory — cannot create beads", project),
        ));
    }

    let title = req.title.clone();
    let description = req.description.clone();
    let issue_type = req.issue_type.clone().unwrap_or_else(|| "task".to_string());
    let priority = req.priority;
    let dependencies = req.dependencies.clone().unwrap_or_default();
    let assignee = req.assignee.clone();
    let labels = req.labels.clone().unwrap_or_default();

    // Determine source (default to "form" for backward compatibility)
    let source_str = req.source.clone();
    let source = if source_str.is_empty() {
        BeadSource::Form
    } else {
        match source_str.as_str() {
            "form" => BeadSource::Form,
            "chat" => BeadSource::Chat,
            "bulk" => BeadSource::Bulk,
            "template" => BeadSource::Template,
            _ => BeadSource::Form,
        }
    };

    let output = tokio::task::spawn_blocking(move || {
        let mut cmd = invoke_br_write(WriteVerb::Create, &[]);
        cmd.current_dir(&project_path);
        cmd.arg(&title);
        cmd.arg("--type").arg(&issue_type);

        if let Some(desc) = &description {
            if !desc.is_empty() {
                cmd.arg("--description").arg(desc);
            }
        }

        if let Some(p) = priority {
            cmd.arg("--priority").arg(p.to_string());
        }

        if !dependencies.is_empty() {
            cmd.arg("--deps").arg(dependencies.join(","));
        }

        if let Some(a) = &assignee {
            if !a.is_empty() {
                cmd.arg("--assignee").arg(a);
            }
        }

        if !labels.is_empty() {
            cmd.arg("--labels").arg(labels.join(","));
        }

        cmd.arg("--actor").arg("hoop:operator");
        cmd.arg("--silent");

        cmd.output()
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task join failed: {}", e)))?
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to run br: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Log failed attempt
        let args = BeadActionArgs {
            source: source.clone(),
            stitch_id: req.stitch_id.clone(),
            title: req.title.clone(),
            issue_type: issue_type.clone(),
            priority,
            dependencies: dependencies.clone(),
            labels: labels.clone(),
        };
        let args_json = serde_json::to_string(&args).ok();

        if let Err(e) = fleet::write_audit_row(
            "hoop:operator",
            ActionKind::BeadCreated,
            &format!("project:{},title:{}", project, req.title),
            Some(&project),
            args_json,
            ActionResult::Failure,
            Some(stderr.trim().to_string()),
        ) {
            tracing::warn!("Failed to write audit row for failed bead creation: {}", e);
        }

        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("br create failed: {}", stderr.trim()),
        ));
    }

    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if id.is_empty() {
        // Fallback: try to read the newest bead from issues.jsonl
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "br create did not return a bead ID".to_string(),
        ));
    }

    // Write audit row for successful bead creation
    let args = BeadActionArgs {
        source,
        stitch_id: req.stitch_id.clone(),
        title: req.title.clone(),
        issue_type,
        priority,
        dependencies,
        labels,
    };
    let args_json = serde_json::to_string(&args).ok();

    if let Err(e) = fleet::write_audit_row(
        "hoop:operator",
        ActionKind::BeadCreated,
        &id,
        Some(&project),
        args_json,
        ActionResult::Success,
        None,
    ) {
        tracing::warn!("Failed to write audit row for bead {}: {}", id, e);
        // Continue anyway — the bead was created successfully
    }

    Ok(Json(CreateBeadResponse {
        id,
        title: req.title,
    }))
}

fn resolve_project_path(
    project: &str,
    state: &crate::DaemonState,
) -> Result<std::path::PathBuf, (StatusCode, String)> {
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
        })
}

fn bead_type_str(t: &crate::BeadType) -> String {
    match t {
        crate::BeadType::Task => "task",
        crate::BeadType::Bug => "bug",
        crate::BeadType::Epic => "epic",
        crate::BeadType::Genesis => "genesis",
        crate::BeadType::Review => "review",
        crate::BeadType::Fix => "fix",
    }
    .to_string()
}

// Re-export for use by the list endpoint with br list as an alternative
#[allow(dead_code)]
fn list_via_br(project_path: &std::path::Path) -> Result<Vec<BeadSummary>, String> {
    let mut cmd = invoke_br_read(ReadVerb::List, &["--json"]);
    cmd.current_dir(project_path);
    let output = cmd
        .output()
        .map_err(|e| format!("Failed to run br list: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("br list failed: {}", stderr));
    }

    let json = String::from_utf8_lossy(&output.stdout);
    let beads: Vec<serde_json::Value> =
        serde_json::from_str(&json).map_err(|e| format!("Failed to parse br list output: {}", e))?;

    let summaries = beads
        .into_iter()
        .filter(|b| b.get("status").and_then(|s| s.as_str()) == Some("open"))
        .map(|b| BeadSummary {
            id: b
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            title: b
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            issue_type: b
                .get("issue_type")
                .and_then(|v| v.as_str())
                .unwrap_or("task")
                .to_string(),
            priority: b
                .get("priority")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            dependencies: b
                .get("dependencies")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
        })
        .collect();

    Ok(summaries)
}
