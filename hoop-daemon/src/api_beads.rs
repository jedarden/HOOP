//! REST API endpoints for bead creation, listing, and dedup checking
//!
//! Endpoints:
//! - GET  /api/p/:project/beads           — list open beads for the dep picker
//! - POST /api/p/:project/beads           — create a bead via `br create`
//! - POST /api/p/:project/beads/dedup     — check for similar existing work
//!
//! Submit flow: draft → validate → dedup check → br create → audit → WS event → response

use crate::br_verbs::{invoke_br_create, invoke_br_read, ReadVerb};
use crate::fleet::{self, ActionKind, ActionResult, BeadActionArgs, BeadSource};
use crate::ws::StitchCreatedData;
use axum::{
    extract::{ConnectInfo, Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing::warn;

/// Valid issue types for bead creation
const VALID_ISSUE_TYPES: &[&str] = &["task", "bug", "epic", "genesis", "review", "fix"];

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
    /// Source of the bead creation: "form", "chat", "bulk", or "template:<name>"
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
    pub project: String,
    pub source: String,
    pub actor: String,
    pub stitch_id: Option<String>,
}

pub fn router() -> Router<crate::DaemonState> {
    Router::new()
        .route("/api/p/{project}/beads", get(list_open_beads))
        .route("/api/p/{project}/beads", post(create_bead))
        .route("/api/p/{project}/beads/dedup", post(check_dedup))
        .route("/api/p/{project}/beads/dedup-dismiss", post(dismiss_dedup))
}

/// Request body for dedup check
#[derive(Debug, Deserialize)]
pub struct DedupCheckRequest {
    pub title: String,
    pub description: Option<String>,
}

/// A match found during dedup check
#[derive(Debug, Serialize)]
pub struct DedupMatchRef {
    pub id: String,
    pub project: String,
    pub title: String,
    pub kind: String,
    pub similarity: f64,
}

/// Response from dedup check
#[derive(Debug, Serialize)]
pub struct DedupCheckResponse {
    pub matches: Vec<DedupMatchRef>,
    pub threshold: f64,
    pub message: Option<String>,
}

/// POST /api/p/:project/beads/dedup — check for similar existing work
///
/// Returns potential duplicates across all projects above the configured threshold.
async fn check_dedup(
    Path(project): Path<String>,
    State(state): State<crate::DaemonState>,
    Json(req): Json<DedupCheckRequest>,
) -> Result<Json<DedupCheckResponse>, (StatusCode, String)> {
    let _ = resolve_project_path(&project, &state)?;

    let title = req.title.trim().to_string();
    if title.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "title is required".to_string()));
    }

    let index = state.vector_index.read().unwrap();
    let matches = index.check_duplicate(&title, req.description.as_deref());

    let message = if !matches.is_empty() {
        let best = &matches[0];
        Some(format!(
            "This looks like `{}/{}` ({}), which is in progress. Continue that, add this as a child, or proceed as new?",
            best.item.project,
            best.item.id,
            best.item.title
        ))
    } else {
        None
    };

    Ok(Json(DedupCheckResponse {
        matches: matches
            .into_iter()
            .map(|m| DedupMatchRef {
                id: m.item.id,
                project: m.item.project,
                title: m.item.title,
                kind: m.item.kind,
                similarity: m.similarity,
            })
            .collect(),
        threshold: index.threshold(),
        message,
    }))
}

/// POST /api/p/:project/beads/dedup-dismiss — report a false positive
async fn dismiss_dedup(
    Path(project): Path<String>,
    State(state): State<crate::DaemonState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let _ = resolve_project_path(&project, &state)?;
    state.vector_index.read().unwrap().report_false_positive();
    Ok(Json(serde_json::json!({"status": "ok"})))
}

/// Resolve the actor identity for audit purposes.
///
/// Per §13: identity from Tailscale whois where available,
/// falling back to the OS user running the HOOP process.
fn resolve_actor(remote_addr: Option<SocketAddr>) -> String {
    if let Some(addr) = remote_addr {
        let ip = addr.ip();
        let output = std::process::Command::new("tailscale")
            .arg("whois")
            .arg(ip.to_string())
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                let identity = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !identity.is_empty() {
                    return format!("tailscale:{}", identity);
                }
            }
        }
    }

    // Fallback: OS username
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    format!("os:{}", user)
}

/// Parse the source field into a BeadSource enum.
/// Handles "form", "chat", "bulk", "template", and "template:<name>" patterns.
pub(crate) fn parse_source(source_str: &str) -> (BeadSource, String) {
    if source_str.is_empty() {
        return (BeadSource::Form, "form".to_string());
    }
    if source_str.starts_with("template:") {
        let name = source_str.strip_prefix("template:").unwrap_or("").to_string();
        return (BeadSource::Template, format!("template:{}", name));
    }
    let source = match source_str {
        "form" => BeadSource::Form,
        "chat" => BeadSource::Chat,
        "bulk" => BeadSource::Bulk,
        "template" => BeadSource::Template,
        _ => BeadSource::Form,
    };
    (source, source_str.to_string())
}

/// Validate the draft fields against the bead schema.
fn validate_draft(req: &CreateBeadRequest) -> Result<(), (StatusCode, String)> {
    let title = req.title.trim();
    if title.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "title is required".to_string()));
    }
    if title.len() > 280 {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("title too long ({} chars, max 280)", title.len()),
        ));
    }

    if let Some(ref it) = req.issue_type {
        if !VALID_ISSUE_TYPES.contains(&it.as_str()) {
            return Err((
                StatusCode::BAD_REQUEST,
                format!(
                    "invalid issue_type '{}'. Must be one of: {}",
                    it,
                    VALID_ISSUE_TYPES.join(", ")
                ),
            ));
        }
    }

    if let Some(p) = req.priority {
        if p < 0 || p > 9 {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("priority must be 0-9, got {}", p),
            ));
        }
    }

    Ok(())
}

/// GET /api/p/:project/beads — list open beads from the project's .beads/issues.jsonl
async fn list_open_beads(
    Path(project): Path<String>,
    State(state): State<crate::DaemonState>,
) -> Result<Json<Vec<BeadSummary>>, (StatusCode, String)> {
    let project_path = resolve_project_path(&project, &state)?;

    let result = tokio::task::spawn_blocking(move || {
        let beads_path = project_path.join(".beads").join("issues.jsonl");
        if !beads_path.exists() {
            return Ok(Vec::<BeadSummary>::new());
        }

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
///
/// Full submit flow:
/// 1. Validate draft against schema
/// 2. Build br create command with stitch label
/// 3. Execute br create in project cwd
/// 4. Insert audit row with actor + source
/// 5. Emit stitch_created event on WS
/// 6. Return response with bead data
async fn create_bead(
    Path(project): Path<String>,
    State(state): State<crate::DaemonState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Json(req): Json<CreateBeadRequest>,
) -> Result<Json<CreateBeadResponse>, (StatusCode, String)> {
    // 1. Validate draft against schema
    validate_draft(&req)?;

    let project_path = resolve_project_path(&project, &state)?;

    let beads_dir = project_path.join(".beads");
    if !beads_dir.exists() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("Project '{}' has no .beads directory — cannot create beads", project),
        ));
    }

    // Resolve actor identity
    let actor = resolve_actor(connect_info.map(|ci| ci.0));

    let title = req.title.clone();
    let description = req.description.clone();
    let issue_type = req.issue_type.clone().unwrap_or_else(|| "task".to_string());
    let priority = req.priority;
    let dependencies = req.dependencies.clone().unwrap_or_default();
    let assignee = req.assignee.clone();
    let labels = req.labels.clone().unwrap_or_default();
    let stitch_id = req.stitch_id.clone();

    // Parse source
    let (source, source_str) = parse_source(&req.source);

    // 2. Build label list with stitch label if applicable
    let mut all_labels = labels.clone();
    if let Some(ref sid) = stitch_id {
        all_labels.push(format!("stitch:{}", sid));
    }

    // 3. Execute br create in project cwd
    let title_for_br = title.clone();
    let desc_for_br = description.clone();
    let issue_type_for_br = issue_type.clone();
    let deps_for_br = dependencies.clone();
    let assignee_for_br = assignee.clone();
    let labels_for_br = all_labels.clone();
    let actor_for_br = actor.clone();

    let output = tokio::task::spawn_blocking(move || {
        let mut cmd = invoke_br_create(&[]);
        cmd.current_dir(&project_path);
        cmd.arg(&title_for_br);
        cmd.arg("--type").arg(&issue_type_for_br);

        if let Some(desc) = &desc_for_br {
            if !desc.is_empty() {
                cmd.arg("--description").arg(desc);
            }
        }

        if let Some(p) = priority {
            cmd.arg("--priority").arg(p.to_string());
        }

        if !deps_for_br.is_empty() {
            cmd.arg("--deps").arg(deps_for_br.join(","));
        }

        if let Some(a) = &assignee_for_br {
            if !a.is_empty() {
                cmd.arg("--assignee").arg(a);
            }
        }

        if !labels_for_br.is_empty() {
            cmd.arg("--labels").arg(labels_for_br.join(","));
        }

        cmd.arg("--actor").arg(&actor_for_br);
        cmd.arg("--silent");

        cmd.output()
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task join failed: {}", e)))?
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to run br: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        let args = BeadActionArgs {
            source: source.clone(),
            stitch_id: stitch_id.clone(),
            title: req.title.clone(),
            issue_type: issue_type.clone(),
            priority,
            dependencies: dependencies.clone(),
            labels: labels.clone(),
        };
        let args_json = serde_json::to_string(&args).ok();
        let args_hash_value = args.args_hash();

        if let Err(e) = fleet::write_audit_row(
            &actor,
            ActionKind::BeadCreated,
            &format!("project:{},title:{}", project, req.title),
            Some(&project),
            args_json,
            ActionResult::Failure,
            Some(stderr.trim().to_string()),
            Some(&source_str),
            stitch_id.as_deref(),
            Some(&args_hash_value),
        ) {
            warn!("Failed to write audit row for failed bead creation: {}", e);
        }

        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("br create failed: {}", stderr.trim()),
        ));
    }

    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if id.is_empty() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "br create did not return a bead ID".to_string(),
        ));
    }

    // 4. Insert audit row with actor + source
    let args = BeadActionArgs {
        source: source.clone(),
        stitch_id: stitch_id.clone(),
        title: req.title.clone(),
        issue_type: issue_type.clone(),
        priority,
        dependencies: dependencies.clone(),
        labels: labels.clone(),
    };
    let args_json = serde_json::to_string(&args).ok();
    let args_hash_value = args.args_hash();

    if let Err(e) = fleet::write_audit_row(
        &actor,
        ActionKind::BeadCreated,
        &id,
        Some(&project),
        args_json,
        ActionResult::Success,
        None,
        Some(&source_str),
        stitch_id.as_deref(),
        Some(&args_hash_value),
    ) {
        warn!("Failed to write audit row for bead {}: {}", id, e);
        // Audit failure is non-fatal — the bead was created successfully
    }

    // 5. Emit stitch_created event on WS
    let created_at = chrono::Utc::now().to_rfc3339();
    let stitch_event = StitchCreatedData {
        bead_id: id.clone(),
        title: title.clone(),
        project: project.clone(),
        stitch_id: stitch_id.clone(),
        source: source_str.clone(),
        actor: actor.clone(),
        created_at: created_at.clone(),
    };

    // Broadcast the stitch_created event to all WS clients
    let _ = state.stitch_tx.send(stitch_event);
    let _ = state.bead_tx.send(crate::ws::BeadData {
        id: id.clone(),
        title: title.clone(),
        status: "open".to_string(),
        priority: priority.unwrap_or(2),
        issue_type: req.issue_type.clone().unwrap_or_else(|| "task".to_string()),
        created_at: created_at.clone(),
        updated_at: created_at,
        created_by: actor.clone(),
        dependencies: req.dependencies.clone().unwrap_or_default(),
    });

    // 6. Return response
    Ok(Json(CreateBeadResponse {
        id,
        title: req.title,
        project,
        source: source_str,
        actor,
        stitch_id,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_source_form() {
        let (source, str) = parse_source("form");
        assert!(matches!(source, BeadSource::Form));
        assert_eq!(str, "form");
    }

    #[test]
    fn test_parse_source_empty() {
        let (source, str) = parse_source("");
        assert!(matches!(source, BeadSource::Form));
        assert_eq!(str, "form");
    }

    #[test]
    fn test_parse_source_chat() {
        let (source, str) = parse_source("chat");
        assert!(matches!(source, BeadSource::Chat));
        assert_eq!(str, "chat");
    }

    #[test]
    fn test_parse_source_bulk() {
        let (source, str) = parse_source("bulk");
        assert!(matches!(source, BeadSource::Bulk));
        assert_eq!(str, "bulk");
    }

    #[test]
    fn test_parse_source_template_with_name() {
        let (source, str) = parse_source("template:bug-fix");
        assert!(matches!(source, BeadSource::Template));
        assert_eq!(str, "template:bug-fix");
    }

    #[test]
    fn test_parse_source_template_bare() {
        let (source, str) = parse_source("template");
        assert!(matches!(source, BeadSource::Template));
        assert_eq!(str, "template");
    }

    #[test]
    fn test_parse_source_unknown_falls_back() {
        let (source, str) = parse_source("unknown");
        assert!(matches!(source, BeadSource::Form));
        assert_eq!(str, "unknown");
    }

    #[test]
    fn test_validate_draft_valid() {
        let req = CreateBeadRequest {
            title: "Fix the bug".to_string(),
            description: None,
            issue_type: Some("task".to_string()),
            priority: Some(2),
            dependencies: None,
            assignee: None,
            labels: None,
            source: "form".to_string(),
            stitch_id: None,
        };
        assert!(validate_draft(&req).is_ok());
    }

    #[test]
    fn test_validate_draft_empty_title() {
        let req = CreateBeadRequest {
            title: "  ".to_string(),
            description: None,
            issue_type: None,
            priority: None,
            dependencies: None,
            assignee: None,
            labels: None,
            source: String::new(),
            stitch_id: None,
        };
        let result = validate_draft(&req);
        assert!(result.is_err());
        let (status, msg) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(msg.contains("title is required"));
    }

    #[test]
    fn test_validate_draft_title_too_long() {
        let req = CreateBeadRequest {
            title: "x".repeat(300),
            description: None,
            issue_type: None,
            priority: None,
            dependencies: None,
            assignee: None,
            labels: None,
            source: String::new(),
            stitch_id: None,
        };
        let result = validate_draft(&req);
        assert!(result.is_err());
        let (status, msg) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(msg.contains("title too long"));
    }

    #[test]
    fn test_validate_draft_invalid_issue_type() {
        let req = CreateBeadRequest {
            title: "Test".to_string(),
            description: None,
            issue_type: Some("invalid".to_string()),
            priority: None,
            dependencies: None,
            assignee: None,
            labels: None,
            source: String::new(),
            stitch_id: None,
        };
        let result = validate_draft(&req);
        assert!(result.is_err());
        let (status, msg) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(msg.contains("invalid issue_type"));
    }

    #[test]
    fn test_validate_draft_priority_out_of_range() {
        let req = CreateBeadRequest {
            title: "Test".to_string(),
            description: None,
            issue_type: None,
            priority: Some(10),
            dependencies: None,
            assignee: None,
            labels: None,
            source: String::new(),
            stitch_id: None,
        };
        let result = validate_draft(&req);
        assert!(result.is_err());
        let (status, msg) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(msg.contains("priority must be 0-9"));
    }

    #[test]
    fn test_validate_draft_negative_priority() {
        let req = CreateBeadRequest {
            title: "Test".to_string(),
            description: None,
            issue_type: None,
            priority: Some(-1),
            dependencies: None,
            assignee: None,
            labels: None,
            source: String::new(),
            stitch_id: None,
        };
        assert!(validate_draft(&req).is_err());
    }

    #[test]
    fn test_valid_issue_types() {
        for it in VALID_ISSUE_TYPES {
            let req = CreateBeadRequest {
                title: "Test".to_string(),
                description: None,
                issue_type: Some(it.to_string()),
                priority: None,
                dependencies: None,
                assignee: None,
                labels: None,
                source: String::new(),
                stitch_id: None,
            };
            assert!(validate_draft(&req).is_ok(), "issue_type '{}' should be valid", it);
        }
    }

    #[test]
    fn test_resolve_actor_fallback() {
        let actor = resolve_actor(None);
        assert!(actor.starts_with("os:"));
    }
}
