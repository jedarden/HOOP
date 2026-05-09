//! Bulk draft creation API (Phase 4 deliverable #5)
//!
//! POST /api/bulk/parse — parse markdown into previewable drafts
//! POST /api/bulk/submit — submit selected drafts for approval
//!
//! Acceptance (§6 Phase 4):
//! - Paste a bullet list or markdown doc; HOOP splits it into multiple drafts
//! - Hard cap at 50 drafts with explicit override
//! - Each draft is previewable before submit
//! - Source is recorded as 'bulk' in audit trail

use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing::{info, warn};
use utoipa::ToSchema;

/// Maximum number of drafts allowed in a single bulk submit
const MAX_BULK_DRAFTS: usize = 50;

/// Parse request - markdown content to split into drafts
#[derive(Debug, Deserialize, ToSchema)]
pub struct ParseBulkRequest {
    /// Target project for all drafts
    pub project: String,
    /// Markdown content to parse (bullet list, numbered list, or doc with headers)
    pub content: String,
    /// Default stitch kind for all drafts
    #[serde(default = "default_kind")]
    pub kind: String,
    /// Default priority for all drafts
    #[serde(default)]
    pub priority: Option<i64>,
    /// Default labels for all drafts
    #[serde(default)]
    pub labels: Vec<String>,
    /// Override the 50-draft limit (requires explicit confirmation)
    #[serde(default)]
    pub override_limit: bool,
}

fn default_kind() -> String {
    "task".to_string()
}

/// Single parsed draft from bulk content
#[derive(Debug, Serialize, ToSchema)]
pub struct ParsedDraft {
    /// Sequential index (1-based)
    pub index: usize,
    /// Draft title (extracted from heading or first line)
    pub title: String,
    /// Draft description/body
    pub description: String,
    /// Stitch kind
    pub kind: String,
    /// Priority
    pub priority: Option<i64>,
    /// Labels
    pub labels: Vec<String>,
}

/// Parse response - list of previewable drafts
#[derive(Debug, Serialize, ToSchema)]
pub struct ParseBulkResponse {
    /// Parsed drafts
    pub drafts: Vec<ParsedDraft>,
    /// Total count
    pub count: usize,
    /// Whether the count exceeds the limit
    pub exceeds_limit: bool,
    /// The hard limit
    pub limit: usize,
}

/// Submit request - submit selected drafts for approval
#[derive(Debug, Deserialize, ToSchema)]
pub struct SubmitBulkRequest {
    /// Target project
    pub project: String,
    /// Drafts to submit (indices from parse response)
    pub drafts: Vec<BulkDraftItem>,
    /// Override the 50-draft limit
    #[serde(default)]
    pub override_limit: bool,
}

/// Single draft item for submission
#[derive(Debug, Deserialize, ToSchema)]
pub struct BulkDraftItem {
    /// Draft title
    pub title: String,
    /// Draft description
    pub description: String,
    /// Stitch kind
    pub kind: String,
    /// Priority
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i64>,
    /// Labels
    #[serde(default)]
    pub labels: Vec<String>,
}

/// Submit response - created draft IDs
#[derive(Debug, Serialize, ToSchema)]
pub struct SubmitBulkResponse {
    /// Created draft IDs (in order)
    pub draft_ids: Vec<String>,
    /// Number successfully created
    pub created: usize,
    /// Number failed
    pub failed: usize,
    /// Errors (if any)
    pub errors: Vec<String>,
}

pub fn router() -> Router<crate::DaemonState> {
    Router::new()
        .route("/api/bulk/parse", post(parse_bulk))
        .route("/api/bulk/submit", post(submit_bulk))
}

/// POST /api/bulk/parse — parse markdown content into previewable drafts
///
/// Accepts a bullet list, numbered list, or markdown document and splits it
/// into individual draft items. Enforces a 50-draft hard limit unless override_limit is true.
#[utoipa::path(
    post,
    path = "/api/bulk/parse",
    tag = "bulk",
    request_body = ParseBulkRequest,
    responses(
        (status = 200, description = "Parsed drafts", body = ParseBulkResponse),
        (status = 400, description = "Invalid request"),
        (status = 403, description = "Exceeds limit without override"),
    ),
)]
async fn parse_bulk(
    State(state): State<crate::DaemonState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Json(req): Json<ParseBulkRequest>,
) -> Result<Json<ParseBulkResponse>, (StatusCode, String)> {
    // Role check: bulk creation requires drafter role
    crate::auth::check_role_for_addr(
        &state.role_resolver,
        connect_info.map(|ci| ci.0),
        crate::auth::Role::Drafter,
    )
    .map_err(|e| (e.0, serde_json::to_string(&e.1 .0).unwrap_or_else(|_| e.0.to_string())))?;

    crate::id_validators::validate_project_name(&req.project)
        .map_err(crate::id_validators::rejection)?;

    // Validate project exists
    let _project_path = crate::api_draft_queue::resolve_project_path(&req.project, &state)?;

    // Parse the markdown content
    let drafts = parse_markdown_to_drafts(&req.content, &req.kind, req.priority, req.labels)?;

    let exceeds_limit = drafts.len() > MAX_BULK_DRAFTS;

    if exceeds_limit && !req.override_limit {
        return Err((
            StatusCode::FORBIDDEN,
            format!(
                "Bulk draft limit exceeded: {} drafts (max {}). Set override_limit=true to proceed.",
                drafts.len(),
                MAX_BULK_DRAFTS
            ),
        ));
    }

    info!(
        "Parsed {} bulk drafts for project '{}'",
        drafts.len(),
        req.project
    );

    Ok(Json(ParseBulkResponse {
        count: drafts.len(),
        exceeds_limit,
        limit: MAX_BULK_DRAFTS,
        drafts,
    }))
}

/// POST /api/bulk/submit — submit selected drafts for approval
///
/// Creates draft queue entries from the submitted bulk drafts.
#[utoipa::path(
    post,
    path = "/api/bulk/submit",
    tag = "bulk",
    request_body = SubmitBulkRequest,
    responses(
        (status = 200, description = "Created drafts", body = SubmitBulkResponse),
        (status = 400, description = "Invalid request"),
        (status = 403, description = "Exceeds limit without override"),
    ),
)]
async fn submit_bulk(
    State(state): State<crate::DaemonState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Json(req): Json<SubmitBulkRequest>,
) -> Result<Json<SubmitBulkResponse>, (StatusCode, String)> {
    // Role check: bulk creation requires drafter role
    crate::auth::check_role_for_addr(
        &state.role_resolver,
        connect_info.map(|ci| ci.0),
        crate::auth::Role::Drafter,
    )
    .map_err(|e| (e.0, serde_json::to_string(&e.1 .0).unwrap_or_else(|_| e.0.to_string())))?;

    crate::id_validators::validate_project_name(&req.project)
        .map_err(crate::id_validators::rejection)?;

    // Validate project exists
    let _project_path = crate::api_draft_queue::resolve_project_path(&req.project, &state)?;

    // Check limit
    if req.drafts.len() > MAX_BULK_DRAFTS && !req.override_limit {
        return Err((
            StatusCode::FORBIDDEN,
            format!(
                "Bulk draft limit exceeded: {} drafts (max {}). Set override_limit=true to proceed.",
                req.drafts.len(),
                MAX_BULK_DRAFTS
            ),
        ));
    }

    let actor = state.identity_cache.resolve(connect_info.map(|ci| ci.0));
    let now = chrono::Utc::now().to_rfc3339();

    let mut draft_ids = Vec::new();
    let mut errors = Vec::new();
    let mut created = 0;

    for (idx, draft_item) in req.drafts.iter().enumerate() {
        let draft_id = format!("draft-{}", uuid::Uuid::new_v4());

        // Validate stitch kind
        if let Err(e) = crate::api_stitch_decompose::validate_stitch_kind(&draft_item.kind, false) {
            errors.push(format!("Draft #{}: {}", idx + 1, e));
            continue;
        }

        // Build the draft row
        let draft_row = crate::fleet::DraftRow {
            id: draft_id.clone(),
            project: req.project.clone(),
            title: draft_item.title.clone(),
            kind: draft_item.kind.clone(),
            description: Some(draft_item.description.clone()),
            has_acceptance_criteria: false,
            priority: draft_item.priority,
            labels: draft_item.labels.clone(),
            created_by: actor.clone(),
            created_at: now.clone(),
            source: "bulk".to_string(),
            agent_session_id: None,
            turn_id: None,
            status: "pending".to_string(),
            version: 1,
            original_json: None,
            resolved_by: None,
            resolved_at: None,
            rejection_reason: None,
            stitch_id: None,
            preview_json: None,
            opened_by: Some(actor.clone()),
            opened_at: Some(now.clone()),
            last_autosave_at: None,
            abandoned_at: None,
        };

        // Insert the draft
        match crate::fleet::insert_draft(&draft_row) {
            Ok(_) => {
                draft_ids.push(draft_id.clone());
                created += 1;

                // Audit: draft created
                if let Err(e) = crate::fleet::write_audit_row(
                    &actor,
                    crate::fleet::ActionKind::DraftCreated,
                    &draft_id,
                    Some(&req.project),
                    Some(
                        serde_json::json!({
                            "title": draft_item.title,
                            "kind": draft_item.kind,
                            "source": "bulk",
                            "bulk_index": idx + 1,
                        })
                        .to_string(),
                    ),
                    crate::fleet::ActionResult::Success,
                    None,
                    Some("bulk"),
                    None,
                    None,
                ) {
                    warn!("Failed to write DraftCreated audit row: {}", e);
                }

                // Emit draft_update WS event
                let _ = state.draft_tx.send(crate::ws::DraftUpdateData {
                    draft_id: draft_id.clone(),
                    project: req.project.clone(),
                    title: draft_item.title.clone(),
                    kind: draft_item.kind.clone(),
                    status: "pending".to_string(),
                    action: "created".to_string(),
                    actor: actor.clone(),
                    created_by: actor.clone(),
                    version: 1,
                    rejection_reason: None,
                });
            }
            Err(e) => {
                errors.push(format!("Draft #{}: {}", idx + 1, e));
            }
        }
    }

    info!(
        "Bulk submit created {} drafts for project '{}' ({} failed)",
        created,
        req.project,
        errors.len()
    );

    Ok(Json(SubmitBulkResponse {
        draft_ids,
        created,
        failed: errors.len(),
        errors,
    }))
}

/// Parse markdown content into individual drafts.
///
/// Supports:
/// - Bullet list: "- Task one", "- Task two"
/// - Numbered list: "1. Task one", "2. Task two"
/// - Headers: "## Task one\nDescription", "## Task two\nDescription"
/// - Mixed: Headers with content, followed by lists
fn parse_markdown_to_drafts(
    content: &str,
    default_kind: &str,
    default_priority: Option<i64>,
    default_labels: Vec<String>,
) -> Result<Vec<ParsedDraft>, String> {
    let mut drafts = Vec::new();

    // First, try to parse as headers with content
    let header_drafts = parse_headers(content);
    if !header_drafts.is_empty() {
        for (idx, (title, desc)) in header_drafts.iter().enumerate() {
            drafts.push(ParsedDraft {
                index: idx + 1,
                title: title.clone(),
                description: desc.clone(),
                kind: default_kind.to_string(),
                priority: default_priority,
                labels: default_labels.clone(),
            });
        }
        return Ok(drafts);
    }

    // Try to parse as lists
    let list_drafts = parse_lists(content);
    if !list_drafts.is_empty() {
        for (idx, item) in list_drafts.iter().enumerate() {
            drafts.push(ParsedDraft {
                index: idx + 1,
                title: item.clone(),
                description: String::new(),
                kind: default_kind.to_string(),
                priority: default_priority,
                labels: default_labels.clone(),
            });
        }
        return Ok(drafts);
    }

    // If no structure found, treat each line as a separate draft
    let line_drafts: Vec<&str> = content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    if !line_drafts.is_empty() {
        for (idx, line) in line_drafts.iter().enumerate() {
            drafts.push(ParsedDraft {
                index: idx + 1,
                title: line.to_string(),
                description: String::new(),
                kind: default_kind.to_string(),
                priority: default_priority,
                labels: default_labels.clone(),
            });
        }
        return Ok(drafts);
    }

    Err("No parseable content found".to_string())
}

/// Parse markdown headers (## or ###) with their content.
fn parse_headers(content: &str) -> Vec<(String, String)> {
    let mut drafts = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    let mut current_title: Option<String> = None;
    let mut current_desc: Vec<String> = Vec::new();

    for line in lines {
        let trimmed = line.trim();

        if trimmed.starts_with("## ") || trimmed.starts_with("### ") {
            // Save previous section if any
            if let Some(title) = current_title.take() {
                let desc = current_desc.join("\n").trim().to_string();
                drafts.push((title, desc));
                current_desc.clear();
            }

            // Extract new title
            let title = trimmed
                .trim_start_matches("##")
                .trim_start_matches("#")
                .trim()
                .to_string();
            current_title = Some(title);
        } else if current_title.is_some() {
            // Accumulate content for current section
            if !trimmed.is_empty() {
                current_desc.push(trimmed.to_string());
            }
        }
    }

    // Don't forget the last section
    if let Some(title) = current_title {
        let desc = current_desc.join("\n").trim().to_string();
        drafts.push((title, desc));
    }

    drafts
}

/// Parse bullet or numbered lists.
fn parse_lists(content: &str) -> Vec<String> {
    let mut items = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Bullet list: "- item", "* item"
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let item = trimmed[2..].trim().to_string();
            if !item.is_empty() {
                items.push(item);
            }
        }
        // Numbered list: "1. item", "2. item", etc.
        else if let Some(dot_idx) = trimmed.find(|c: char| !c.is_ascii_digit()) {
            let rest = &trimmed[dot_idx..];
            if rest.starts_with(". ") || rest.starts_with(") ") {
                let item = rest[2..].trim().to_string();
                if !item.is_empty() {
                    items.push(item);
                }
            }
        }
    }

    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_headers() {
        let content = r#"
## Task One
Description for task one.

## Task Two
Description for task two.
"#;

        let drafts = parse_headers(content);
        assert_eq!(drafts.len(), 2);
        assert_eq!(drafts[0].0, "Task One");
        assert_eq!(drafts[0].1, "Description for task one.");
        assert_eq!(drafts[1].0, "Task Two");
        assert_eq!(drafts[1].1, "Description for task two.");
    }

    #[test]
    fn test_parse_bullet_list() {
        let content = r#"
- First task
- Second task
- Third task
"#;

        let drafts = parse_lists(content);
        assert_eq!(drafts.len(), 3);
        assert_eq!(drafts[0], "First task");
        assert_eq!(drafts[1], "Second task");
        assert_eq!(drafts[2], "Third task");
    }

    #[test]
    fn test_parse_numbered_list() {
        let content = r#"
1. First task
2. Second task
3. Third task
"#;

        let drafts = parse_lists(content);
        assert_eq!(drafts.len(), 3);
        assert_eq!(drafts[0], "First task");
        assert_eq!(drafts[1], "Second task");
        assert_eq!(drafts[2], "Third task");
    }

    #[test]
    fn test_parse_markdown_to_drafts_headers() {
        let content = "## Task One\nDescription\n\n## Task Two\nDescription";
        let drafts = parse_markdown_to_drafts(content, "task", Some(0), vec![]).unwrap();
        assert_eq!(drafts.len(), 2);
        assert_eq!(drafts[0].title, "Task One");
        assert_eq!(drafts[0].description, "Description");
    }

    #[test]
    fn test_parse_markdown_to_drafts_lists() {
        let content = "- Task one\n- Task two\n- Task three";
        let drafts = parse_markdown_to_drafts(content, "task", Some(0), vec![]).unwrap();
        assert_eq!(drafts.len(), 3);
        assert_eq!(drafts[0].title, "Task one");
        assert_eq!(drafts[1].title, "Task two");
        assert_eq!(drafts[2].title, "Task three");
    }

    #[test]
    fn test_parse_markdown_to_drafts_lines() {
        let content = "Task one\nTask two\nTask three";
        let drafts = parse_markdown_to_drafts(content, "task", Some(0), vec![]).unwrap();
        assert_eq!(drafts.len(), 3);
        assert_eq!(drafts[0].title, "Task one");
    }

    #[test]
    fn test_max_bulk_drafts_limit() {
        assert_eq!(MAX_BULK_DRAFTS, 50);
    }
}
