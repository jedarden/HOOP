//! REST API for stitch decomposition preview + submit
//!
//! Endpoints:
//! - POST /api/p/:project/stitch/decompose  — preview the bead graph for a Stitch intent
//! - POST /api/p/:project/stitch/submit     — submit a (possibly overridden) bead graph
//!
//! Submit flow: draft → validate → decompose → br create → audit → WS event → redirect
//! On partial failure, previously created beads are closed and audit rows are voided.

use crate::br_verbs::invoke_br_create;
use crate::fleet::{self, ActionKind, ActionResult, BeadActionArgs};
use crate::predictor::{predict_stitch, HistoricalStitch};
use crate::risk_patterns::{FixLineageLibrary, default_risk_patterns};
use crate::stitch_decompose::{
    self, apply_override, decompose, BeadGraph, GraphOverride, StitchIntent,
};
use crate::ws::StitchCreatedData;
use axum::{
    extract::{ConnectInfo, Path, State},
    http::StatusCode,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing::warn;

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

/// Request to preview a decomposition
#[derive(Debug, Deserialize)]
pub struct DecomposePreviewRequest {
    pub kind: String,
    pub title: String,
    pub description: Option<String>,
    pub has_acceptance_criteria: Option<bool>,
    pub priority: Option<i64>,
    pub labels: Option<Vec<String>>,
}

/// Response to a decomposition preview
#[derive(Debug, Serialize)]
pub struct DecomposePreviewResponse {
    pub graph: BeadGraph,
    pub rule_name: String,
    pub bead_count: usize,
    /// Potential duplicates found during dedup check
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dedup_matches: Option<Vec<DedupMatchRef>>,
    /// "What Will This Take?" preview data (cost, duration, risks, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<StitchPreviewData>,
}

/// "What Will This Take?" preview data for a Stitch
#[derive(Debug, Serialize)]
pub struct StitchPreviewData {
    pub prediction: Option<PredictionData>,
    pub risk_patterns: Vec<RiskPatternMatch>,
    pub file_conflicts: Vec<FileConflict>,
    pub similar_stitches: Vec<SimilarStitchRef>,
}

#[derive(Debug, Serialize)]
pub struct PredictionData {
    pub cost: PercentileEstimate,
    pub duration: PercentileEstimate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub likely_adapter_model: Option<String>,
    pub similar_count: usize,
    pub data_range: DateRange,
}

#[derive(Debug, Serialize)]
pub struct PercentileEstimate {
    pub p50: f64,
    pub p90: f64,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct DateRange {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Serialize)]
pub struct RiskPatternMatch {
    pub pattern: RiskPatternInfo,
    pub confidence: f64,
    pub matched_keywords: Vec<String>,
    pub matched_labels: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RiskPatternInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub fix_recommendation: String,
    pub severity: String,
    pub category: String,
}

#[derive(Debug, Serialize)]
pub struct FileConflict {
    pub bead_id: String,
    pub title: String,
    pub project: String,
    pub overlapping_files: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SimilarStitchRef {
    pub id: String,
    pub title: String,
    pub similarity: f64,
}

/// A dedup match found during stitch preview/submit
#[derive(Debug, Serialize, Clone)]
pub struct DedupMatchRef {
    pub id: String,
    pub project: String,
    pub title: String,
    pub kind: String,
    pub similarity: f64,
}

/// Request to submit a decomposed Stitch (possibly with overrides)
#[derive(Debug, Deserialize)]
pub struct StitchSubmitRequest {
    pub kind: String,
    pub title: String,
    pub description: Option<String>,
    pub has_acceptance_criteria: Option<bool>,
    pub priority: Option<i64>,
    pub labels: Option<Vec<String>>,
    /// Optional override to modify the default graph
    pub override_: Option<GraphOverride>,
    /// Source of the request: "form", "chat", "bulk", or "template:<name>"
    #[serde(default)]
    pub source: String,
    /// Stitch ID to associate beads with (auto-generated if not provided)
    pub stitch_id: Option<String>,
    /// If true, bypass the dedup check and create anyway
    #[serde(default)]
    pub force_create: bool,
}

/// Response after submitting a decomposed Stitch
#[derive(Debug, Serialize)]
pub struct StitchSubmitResponse {
    pub stitch_id: String,
    pub graph: BeadGraph,
    pub created_beads: Vec<CreatedBead>,
    pub errors: Vec<String>,
    /// If true, a partial failure occurred and previously created beads were rolled back
    pub rolled_back: bool,
}

/// A bead that was created as part of Stitch decomposition
#[derive(Debug, Serialize)]
pub struct CreatedBead {
    pub key: String,
    pub id: String,
    pub title: String,
    pub issue_type: String,
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<crate::DaemonState> {
    Router::new()
        .route("/api/p/{project}/stitch/decompose", post(preview_decompose))
        .route("/api/p/{project}/stitch/submit", post(submit_stitch))
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/p/:project/stitch/decompose — preview decomposition without creating beads
async fn preview_decompose(
    Path(project): Path<String>,
    State(state): State<crate::DaemonState>,
    Json(req): Json<DecomposePreviewRequest>,
) -> Result<Json<DecomposePreviewResponse>, (StatusCode, String)> {
    let _project_path = resolve_project_path(&project, &state)?;

    validate_stitch_kind(&req.kind, req.has_acceptance_criteria.unwrap_or(false))?;

    let config = stitch_decompose::load_config_from_file();
    let intent = StitchIntent {
        kind: req.kind,
        title: req.title.clone(),
        description: req.description.clone(),
        has_acceptance_criteria: req.has_acceptance_criteria.unwrap_or(false),
        project,
        priority: req.priority,
        labels: req.labels.unwrap_or_default(),
    };

    let graph = decompose(&config.rules, &intent)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, format!("No decomposition rule matches kind '{}'", intent.kind)))?;

    // Check for potential duplicates across all projects
    let index = state.vector_index.read().unwrap();
    let dedup_matches = index.check_duplicate(&req.title, req.description.as_deref());
    let dedup_refs = if dedup_matches.is_empty() {
        None
    } else {
        Some(dedup_matches.into_iter().map(|m| DedupMatchRef {
            id: m.item.id,
            project: m.item.project,
            title: m.item.title,
            kind: m.item.kind,
            similarity: m.similarity,
        }).collect())
    };

    Ok(Json(DecomposePreviewResponse {
        rule_name: graph.rule_name.clone(),
        bead_count: graph.beads.len(),
        graph,
        dedup_matches: dedup_refs,
    }))
}

/// POST /api/p/:project/stitch/submit — submit a decomposed Stitch, creating beads via br
///
/// Full submit flow:
/// 1. Validate draft against schema
/// 2. Check for potential duplicates (unless force_create is true)
/// 3. Decompose into bead payloads via Stitch decomposition service
/// 4. Execute `br create` with `stitch:<stitch-id>` label for each bead
/// 5. Insert audit row with actor + source
/// 6. Emit `stitch_created` event on WS
/// 7. Return response with created bead IDs
///
/// On partial failure, previously created beads are closed and audit rows are voided.
async fn submit_stitch(
    Path(project): Path<String>,
    State(state): State<crate::DaemonState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Json(req): Json<StitchSubmitRequest>,
) -> Result<Json<StitchSubmitResponse>, (StatusCode, String)> {
    // 1. Validate draft against schema
    validate_stitch_draft(&req)?;

    // 2. Check for potential duplicates (unless force_create is true)
    if !req.force_create {
        let index = state.vector_index.read().unwrap();
        let dedup_matches = index.check_duplicate(&req.title, req.description.as_deref());
        if !dedup_matches.is_empty() {
            let best = &dedup_matches[0];
            let message = format!(
                "This looks like `{}/{}` ({}), which is in progress. Continue that, add this as a child, or proceed as new?",
                best.item.project,
                best.item.id,
                best.item.title
            );
            // Return the matches as JSON with a 409 Conflict status
            let matches_json = serde_json::to_value(&dedup_matches.iter().map(|m| DedupMatchRef {
                id: m.item.id.clone(),
                project: m.item.project.clone(),
                title: m.item.title.clone(),
                kind: m.item.kind.clone(),
                similarity: m.similarity,
            }).collect::<Vec<_>>()).unwrap_or(serde_json::Value::Null);
            let error_json = serde_json::json!({
                "message": message,
                "dedup_matches": matches_json,
                "threshold": index.threshold(),
            });
            return Err((StatusCode::CONFLICT, error_json.to_string()));
        }
    }

    let project_path = resolve_project_path(&project, &state)?;

    let beads_dir = project_path.join(".beads");
    if !beads_dir.exists() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("Project '{}' has no .beads directory — cannot create beads", project),
        ));
    }

    // Auto-generate stitch_id if not provided
    let stitch_id = req.stitch_id.clone().unwrap_or_else(|| {
        format!("stitch-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("unknown"))
    });

    let config = stitch_decompose::load_config_from_file();
    let intent = StitchIntent {
        kind: req.kind.clone(),
        title: req.title.clone(),
        description: req.description.clone(),
        has_acceptance_criteria: req.has_acceptance_criteria.unwrap_or(false),
        project: project.clone(),
        priority: req.priority,
        labels: req.labels.clone().unwrap_or_default(),
    };

    let base_graph = decompose(&config.rules, &intent)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, format!("No decomposition rule matches kind '{}'", intent.kind)))?;

    let graph = if let Some(over) = &req.override_ {
        apply_override(&base_graph, over)
    } else {
        base_graph
    };

    // Resolve actor identity (§13: Tailscale whois → OS username fallback)
    let actor = resolve_actor(connect_info.map(|ci| ci.0));

    let source_str = if req.source.is_empty() {
        "form".to_string()
    } else {
        req.source.clone()
    };
    let (source, _) = crate::api_beads::parse_source(&source_str);

    // 2–4. Create beads in dependency order, with audit
    let mut created_beads: Vec<CreatedBead> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let mut key_to_id: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut had_failure = false;

    for bead in &graph.beads {
        let bead_title = bead.title.clone();
        let bead_issue_type = bead.issue_type.clone();
        let bead_priority = bead.priority;
        let bead_labels = bead.labels.clone();
        let bead_body = bead.body_template.clone();
        let bead_key = bead.key.clone();
        let bead_depends_on = bead.depends_on.clone();
        let stitch_id_clone = stitch_id.clone();
        let actor_clone = actor.clone();
        let cwd = project_path.clone();

        // Clones for audit row (originals move into spawn_blocking)
        let bead_title_for_audit = bead_title.clone();
        let bead_issue_type_for_audit = bead_issue_type.clone();

        let resolved_deps: Vec<String> = bead
            .depends_on
            .iter()
            .filter_map(|k| key_to_id.get(k).cloned())
            .collect();

        let stitch_label = format!("stitch:{}", stitch_id_clone);
        let output = tokio::task::spawn_blocking(move || {
            let mut all_labels = bead_labels;
            all_labels.push(stitch_label);

            let mut cmd = invoke_br_create(&[]);
            cmd.current_dir(&cwd);
            cmd.arg(&bead_title);
            cmd.arg("--type").arg(&bead_issue_type);

            if let Some(body) = &bead_body {
                if !body.is_empty() {
                    cmd.arg("--description").arg(body);
                }
            }

            if let Some(p) = bead_priority {
                cmd.arg("--priority").arg(p.to_string());
            }

            if !resolved_deps.is_empty() {
                cmd.arg("--deps").arg(resolved_deps.join(","));
            }

            if !all_labels.is_empty() {
                cmd.arg("--labels").arg(all_labels.join(","));
            }

            cmd.arg("--actor").arg(&actor_clone);
            cmd.arg("--silent");

            cmd.output()
        })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task join failed: {}", e)))?
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to run br: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            errors.push(format!("Failed to create bead '{}': {}", bead.key, stderr.trim()));
            had_failure = true;
            break;
        }

        let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if id.is_empty() {
            errors.push(format!("br create for bead '{}' returned no ID", bead.key));
            had_failure = true;
            break;
        }

        // 4. Insert audit row with actor + source
        if let Err(e) = fleet::write_audit_row(
            &actor,
            ActionKind::BeadCreated,
            &id,
            Some(&project),
            Some(serde_json::to_string(&BeadActionArgs {
                source: source.clone(),
                stitch_id: Some(stitch_id.clone()),
                title: bead_title_for_audit.clone(),
                issue_type: bead_issue_type_for_audit.clone(),
                priority: bead_priority,
                dependencies: bead_depends_on,
                labels: bead.labels.clone(),
            }).unwrap_or_default()),
            ActionResult::Success,
            None,
            Some(&source_str),
            Some(&stitch_id),
            None,
        ) {
            warn!("Failed to write audit row for bead {}: {}", id, e);
        }

        key_to_id.insert(bead_key.clone(), id.clone());
        created_beads.push(CreatedBead {
            key: bead_key,
            id: id.clone(),
            title: bead_title_for_audit,
            issue_type: bead_issue_type_for_audit,
        });
    }

    // Roll back on partial failure: close created beads and void audit rows
    if had_failure && !created_beads.is_empty() {
        warn!(
            "Stitch submit partial failure: rolling back {} created beads for stitch {}",
            created_beads.len(),
            stitch_id
        );

        for created in &created_beads {
            let close_stitch_id = stitch_id.clone();

            // Under create-only-write, br close is forbidden — beads remain open.
            // The audit trail still records the partial failure.
            #[cfg(not(feature = "create-only-write"))]
            {
                let close_id = created.id.clone();
                let close_actor = actor.clone();
                let close_cwd = project_path.clone();

                let _ = tokio::task::spawn_blocking(move || {
                    let mut cmd = crate::br_verbs::invoke_br_write(crate::br_verbs::WriteVerb::Close, &[]);
                    cmd.current_dir(&close_cwd);
                    cmd.arg(&close_id);
                    cmd.arg("--actor").arg(&close_actor);
                    cmd.arg("--silent");
                    cmd.output()
                }).await;
            }

            // Write a rollback audit entry
            let _ = fleet::write_audit_row(
                &actor,
                ActionKind::BeadCreated,
                &created.id,
                Some(&project),
                None,
                ActionResult::Failure,
                Some("Rolled back: subsequent bead creation failed in stitch submit".to_string()),
                Some(&source_str),
                Some(&close_stitch_id),
                None,
            );
        }

        let rollback_msg = if cfg!(feature = "create-only-write") {
            format!(
                "Stitch submit failed after creating {} bead(s). Beads could not be closed (create-only mode). Errors: {}",
                created_beads.len(),
                errors.join("; ")
            )
        } else {
            format!(
                "Stitch submit failed after creating {} bead(s). All created beads have been rolled back. Errors: {}",
                created_beads.len(),
                errors.join("; ")
            )
        };

        return Err((StatusCode::INTERNAL_SERVER_ERROR, rollback_msg));
    }

    // 5. Emit stitch_created event on WS for each created bead
    for created in &created_beads {
        let created_at = chrono::Utc::now().to_rfc3339();
        let stitch_event = StitchCreatedData {
            bead_id: created.id.clone(),
            title: created.title.clone(),
            project: project.clone(),
            stitch_id: Some(stitch_id.clone()),
            source: source_str.clone(),
            actor: actor.clone(),
            created_at: created_at.clone(),
        };

        let _ = state.stitch_tx.send(stitch_event);
        let _ = state.bead_tx.send(crate::ws::BeadData {
            id: created.id.clone(),
            title: created.title.clone(),
            status: "open".to_string(),
            priority: req.priority.unwrap_or(2),
            issue_type: created.issue_type.clone(),
            created_at: created_at.clone(),
            updated_at: created_at,
            created_by: actor.clone(),
            dependencies: vec![],
        });
    }

    // 6. Return response
    Ok(Json(StitchSubmitResponse {
        stitch_id,
        graph,
        created_beads,
        errors,
        rolled_back: false,
    }))
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Validate the stitch submit request fields against schema constraints
fn validate_stitch_draft(req: &StitchSubmitRequest) -> Result<(), (StatusCode, String)> {
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

    if let Some(p) = req.priority {
        if p < 0 || p > 9 {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("priority must be 0-9, got {}", p),
            ));
        }
    }

    // Validate source field
    let valid_sources = ["form", "chat", "bulk"];
    if !req.source.is_empty()
        && !valid_sources.iter().any(|s| req.source == *s)
        && !req.source.starts_with("template:")
    {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "invalid source '{}'. Must be one of: form, chat, bulk, template:<name>",
                req.source
            ),
        ));
    }

    // Validate that the kind matches a decomposition rule
    validate_stitch_kind(&req.kind, req.has_acceptance_criteria.unwrap_or(false))?;

    Ok(())
}

/// Validate that the stitch kind matches at least one decomposition rule
fn validate_stitch_kind(kind: &str, has_acceptance_criteria: bool) -> Result<(), (StatusCode, String)> {
    let config = stitch_decompose::load_config_from_file();
    let test_intent = StitchIntent {
        kind: kind.to_string(),
        title: "test".to_string(),
        description: None,
        has_acceptance_criteria,
        project: String::new(),
        priority: None,
        labels: vec![],
    };

    if decompose(&config.rules, &test_intent).is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "No decomposition rule matches kind '{}'{}. Valid kinds: investigation, fix, feature",
                kind,
                if has_acceptance_criteria { " (with acceptance criteria)" } else { "" }
            ),
        ));
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

/// Resolve actor identity via Tailscale whois (§13), falling back to OS username
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

    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    format!("os:{}", user)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_stitch_draft_valid() {
        let req = StitchSubmitRequest {
            kind: "fix".to_string(),
            title: "Fix auth race condition".to_string(),
            description: None,
            has_acceptance_criteria: Some(true),
            priority: Some(2),
            labels: None,
            override_: None,
            source: "form".to_string(),
            stitch_id: None,
            force_create: false,
        };
        assert!(validate_stitch_draft(&req).is_ok());
    }

    #[test]
    fn test_validate_stitch_draft_empty_title() {
        let req = StitchSubmitRequest {
            kind: "fix".to_string(),
            title: "   ".to_string(),
            description: None,
            has_acceptance_criteria: None,
            priority: None,
            labels: None,
            override_: None,
            source: String::new(),
            stitch_id: None,
            force_create: false,
        };
        let result = validate_stitch_draft(&req);
        assert!(result.is_err());
        let (status, msg) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(msg.contains("title is required"));
    }

    #[test]
    fn test_validate_stitch_draft_title_too_long() {
        let req = StitchSubmitRequest {
            kind: "fix".to_string(),
            title: "x".repeat(300),
            description: None,
            has_acceptance_criteria: None,
            priority: None,
            labels: None,
            override_: None,
            source: String::new(),
            stitch_id: None,
            force_create: false,
        };
        let result = validate_stitch_draft(&req);
        assert!(result.is_err());
        let (status, msg) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(msg.contains("title too long"));
    }

    #[test]
    fn test_validate_stitch_draft_invalid_priority() {
        let req = StitchSubmitRequest {
            kind: "fix".to_string(),
            title: "Test".to_string(),
            description: None,
            has_acceptance_criteria: None,
            priority: Some(10),
            labels: None,
            override_: None,
            source: String::new(),
            stitch_id: None,
            force_create: false,
        };
        let result = validate_stitch_draft(&req);
        assert!(result.is_err());
        let (_, msg) = result.unwrap_err();
        assert!(msg.contains("priority must be 0-9"));
    }

    #[test]
    fn test_validate_stitch_draft_invalid_source() {
        let req = StitchSubmitRequest {
            kind: "fix".to_string(),
            title: "Test".to_string(),
            description: None,
            has_acceptance_criteria: None,
            priority: None,
            labels: None,
            override_: None,
            source: "invalid".to_string(),
            stitch_id: None,
            force_create: false,
        };
        let result = validate_stitch_draft(&req);
        assert!(result.is_err());
        let (_, msg) = result.unwrap_err();
        assert!(msg.contains("invalid source"));
    }

    #[test]
    fn test_validate_stitch_draft_template_source() {
        let req = StitchSubmitRequest {
            kind: "fix".to_string(),
            title: "Test".to_string(),
            description: None,
            has_acceptance_criteria: None,
            priority: None,
            labels: None,
            override_: None,
            source: "template:bug-fix".to_string(),
            stitch_id: None,
            force_create: false,
        };
        assert!(validate_stitch_draft(&req).is_ok());
    }

    #[test]
    fn test_validate_stitch_kind_unknown() {
        let result = validate_stitch_kind("unknown-kind", false);
        assert!(result.is_err());
        let (status, msg) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(msg.contains("No decomposition rule matches"));
    }

    #[test]
    fn test_validate_stitch_kind_investigation() {
        assert!(validate_stitch_kind("investigation", false).is_ok());
    }

    #[test]
    fn test_validate_stitch_kind_fix() {
        assert!(validate_stitch_kind("fix", false).is_ok());
        assert!(validate_stitch_kind("fix", true).is_ok());
    }

    #[test]
    fn test_validate_stitch_kind_feature() {
        assert!(validate_stitch_kind("feature", false).is_ok());
    }

    #[test]
    fn test_resolve_actor_fallback() {
        let actor = resolve_actor(None);
        assert!(actor.starts_with("os:"));
    }
}
