//! REST API for stitch decomposition preview + submit
//!
//! Endpoints:
//! - POST /api/p/:project/stitch/decompose  — preview the bead graph for a Stitch intent
//! - POST /api/p/:project/stitch/submit     — submit a (possibly overridden) bead graph
//!
//! Submit flow: draft → validate → decompose → br create → audit → WS event → redirect
//! On partial failure, previously created beads are closed and audit rows are voided.

use crate::api_preview::FileConflict;
#[cfg(not(feature = "zero-write-v01"))]
use crate::br_verbs::invoke_br_create;
use crate::fleet::{self, ActionKind, ActionResult, BeadActionArgs};
use crate::metrics;
use crate::predictor::{predict_stitch, PercentileEstimate, DateRange};

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
use std::time::Instant;
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<String>,
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

/// Result of a successful stitch submission (shared between direct submit and draft approve)
#[derive(Debug, Serialize)]
pub struct SubmitResult {
    pub stitch_id: String,
    pub graph: BeadGraph,
    pub created_beads: Vec<CreatedBead>,
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
    crate::id_validators::validate_project_name(&project).map_err(crate::id_validators::rejection)?;
    let _project_path = resolve_project_path(&project, &state)?;

    validate_stitch_kind(&req.kind, req.has_acceptance_criteria.unwrap_or(false))?;

    let config = stitch_decompose::load_config_from_file();
    let labels = req.labels.clone().unwrap_or_default();
    let intent = StitchIntent {
        kind: req.kind,
        title: req.title.clone(),
        description: req.description.clone(),
        has_acceptance_criteria: req.has_acceptance_criteria.unwrap_or(false),
        project: project.clone(),
        priority: req.priority,
        labels: labels.clone(),
    };

    let graph = decompose(&config.rules, &intent)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, format!("No decomposition rule matches kind '{}'", intent.kind)))?;

    // Check for potential duplicates across all projects
    let dedup_refs = {
        let index = state.vector_index.read().unwrap();
        let dedup_matches = index.check_duplicate(&req.title, req.description.as_deref());
        if dedup_matches.is_empty() {
            None
        } else {
            Some(dedup_matches.into_iter().map(|m| DedupMatchRef {
                id: m.item.id,
                project: m.item.project,
                title: m.item.title,
                kind: m.item.kind,
                similarity: m.similarity,
            }).collect())
        }
    };

    // Fetch "What Will This Take?" preview data
    let preview = fetch_stitch_preview(&project, &req.title, req.description.as_deref(), &labels, &state).await;

    Ok(Json(DecomposePreviewResponse {
        rule_name: graph.rule_name.clone(),
        bead_count: graph.beads.len(),
        graph,
        dedup_matches: dedup_refs,
        preview,
    }))
}

/// POST /api/p/:project/stitch/submit — submit a decomposed Stitch, creating beads via br
///
/// Thin wrapper: validates, dedup-checks, resolves path/actor, then delegates to
/// `submit_stitch_internal` for the actual bead creation.
async fn submit_stitch(
    Path(project): Path<String>,
    State(state): State<crate::DaemonState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Json(req): Json<StitchSubmitRequest>,
) -> Result<Json<StitchSubmitResponse>, (StatusCode, String)> {
    // 1. Validate draft against schema
    validate_stitch_draft(&req)?;

    // 1a. Validate project name
    crate::id_validators::validate_project_name(&project).map_err(crate::id_validators::rejection)?;

    // 1b. Validate stitch_id if provided
    if let Some(ref sid) = req.stitch_id {
        crate::id_validators::validate_stitch_id(sid).map_err(crate::id_validators::rejection)?;
    }

    // 2. Check for potential duplicates (unless force_create is true)
    if !req.force_create {
        let index = state.vector_index.read().unwrap();
        let dedup_matches = index.check_duplicate(&req.title, req.description.as_deref());
        if !dedup_matches.is_empty() {
            metrics::metrics().hoop_already_started_dedup_hits_total.inc();
            let best = &dedup_matches[0];
            let message = format!(
                "This looks like `{}/{}` ({}), which is in progress. Continue that, add this as a child, or proceed as new?",
                best.item.project,
                best.item.id,
                best.item.title
            );
            let matches_json = serde_json::to_value(dedup_matches.iter().map(|m| DedupMatchRef {
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

    let actor = resolve_actor(connect_info.map(|ci| ci.0));

    let result = submit_stitch_internal(&project, &project_path, &req, &state, &actor).await?;

    Ok(Json(StitchSubmitResponse {
        stitch_id: result.stitch_id,
        graph: result.graph,
        created_beads: result.created_beads,
        errors: vec![],
        rolled_back: false,
    }))
}

/// Internal stitch submission: decompose intent, create beads via br, persist, audit, emit WS.
///
/// Called from both the direct submit endpoint and the draft approve flow.
/// Callers are responsible for validation, dedup, and path resolution before invoking this.
pub async fn submit_stitch_internal(
    project: &str,
    project_path: &std::path::Path,
    req: &StitchSubmitRequest,
    state: &crate::DaemonState,
    actor: &str,
) -> Result<SubmitResult, (StatusCode, String)> {
    // Zero-write guard: stitch submit creates beads via br create
    #[cfg(feature = "zero-write-v01")]
    return Err((StatusCode::FORBIDDEN, "Stitch submission is disabled in zero-write mode".to_string()));

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
        project: project.to_string(),
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

    let source_str = if req.source.is_empty() {
        "form".to_string()
    } else {
        req.source.clone()
    };
    let (source, _) = crate::api_beads::parse_source(&source_str);

    // Create beads in dependency order, with audit
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
        let actor_clone = actor.to_string();
        let cwd = project_path.to_path_buf();

        let bead_title_for_audit = bead_title.clone();
        let bead_issue_type_for_audit = bead_issue_type.clone();

        let resolved_deps: Vec<String> = bead
            .depends_on
            .iter()
            .filter_map(|k| key_to_id.get(k).cloned())
            .collect();

        let stitch_label = format!("stitch:{}", stitch_id_clone);
        let br_start = Instant::now();
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
        let br_elapsed_ms = br_start.elapsed().as_secs_f64() * 1_000.0;
        let br_ok = output.status.success();
        metrics::metrics().hoop_br_subprocess_total.inc(&["create", if br_ok { "ok" } else { "error" }]);
        metrics::metrics().hoop_br_subprocess_duration_ms.observe(&["create"], br_elapsed_ms);

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

        if let Err(e) = fleet::write_audit_row(
            actor,
            ActionKind::BeadCreated,
            &id,
            Some(project),
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
        metrics::metrics().hoop_bead_created_by_hoop_total.inc(&[project]);
        created_beads.push(CreatedBead {
            key: bead_key,
            id: id.clone(),
            title: bead_title_for_audit,
            issue_type: bead_issue_type_for_audit,
        });
    }

    // Roll back on partial failure
    if had_failure && !created_beads.is_empty() {
        warn!(
            "Stitch submit partial failure: rolling back {} created beads for stitch {}",
            created_beads.len(),
            stitch_id
        );

        for created in &created_beads {
            let close_stitch_id = stitch_id.clone();

            #[cfg(not(any(feature = "create-only-write", feature = "zero-write-v01")))]
            {
                let close_id = created.id.clone();
                let close_actor = actor.to_string();
                let close_cwd = project_path.to_path_buf();

                let _ = tokio::task::spawn_blocking(move || {
                    let start = std::time::Instant::now();
                    let mut cmd = crate::br_verbs::invoke_br_write(crate::br_verbs::WriteVerb::Close, &[]);
                    cmd.current_dir(&close_cwd);
                    cmd.arg(&close_id);
                    cmd.arg("--actor").arg(&close_actor);
                    cmd.arg("--silent");
                    let result = cmd.output();
                    let elapsed_ms = start.elapsed().as_secs_f64() * 1_000.0;
                    let ok = result.as_ref().map(|o| o.status.success()).unwrap_or(false);
                    crate::metrics::metrics().hoop_br_subprocess_total.inc(&["close", if ok { "ok" } else { "error" }]);
                    crate::metrics::metrics().hoop_br_subprocess_duration_ms.observe(&["close"], elapsed_ms);
                    result
                }).await;
            }

            let _ = fleet::write_audit_row(
                actor,
                ActionKind::BeadCreated,
                &created.id,
                Some(project),
                None,
                ActionResult::Failure,
                Some("Rolled back: subsequent bead creation failed in stitch submit".to_string()),
                Some(&source_str),
                Some(&close_stitch_id),
                None,
            );
        }

        if let Err(e) = fleet::delete_stitch(&stitch_id) {
            warn!("Failed to delete orphaned stitch row {}: {}", stitch_id, e);
        }

        let _ = fleet::write_audit_row(
            actor,
            ActionKind::StitchCreated,
            &stitch_id,
            Some(project),
            None,
            ActionResult::Failure,
            Some(format!("Rolled back: {} bead(s) created then closed due to partial failure", created_beads.len())),
            Some(&source_str),
            Some(&stitch_id),
            None,
        );

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

    if had_failure && created_beads.is_empty() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Stitch submit failed: {}", errors.join("; ")),
        ));
    }

    // Persist stitch row in fleet.db and link beads
    let bead_links: Vec<(&str, &str)> = created_beads
        .iter()
        .map(|b| (b.id.as_str(), project))
        .collect();
    if let Err(e) = fleet::create_stitch(
        &stitch_id,
        project,
        "operator",
        &req.title,
        actor,
        &bead_links,
    ) {
        warn!("Failed to persist stitch row for {}: {}", stitch_id, e);
    }

    // Emit stitch creation metrics
    metrics::metrics().hoop_stitch_created_total.inc(&[project, &req.kind]);
    metrics::metrics().hoop_stitches_created_per_day.inc();

    // Write StitchCreated audit row
    if let Err(e) = fleet::write_audit_row(
        actor,
        ActionKind::StitchCreated,
        &stitch_id,
        Some(project),
        Some(serde_json::json!({
            "source": source_str,
            "kind": req.kind,
            "title": req.title,
            "bead_count": created_beads.len(),
            "bead_ids": created_beads.iter().map(|b| &b.id).collect::<Vec<_>>(),
        }).to_string()),
        ActionResult::Success,
        None,
        Some(&source_str),
        Some(&stitch_id),
        None,
    ) {
        warn!("Failed to write StitchCreated audit row for {}: {}", stitch_id, e);
    }

    // Emit WS events
    for created in &created_beads {
        let created_at = chrono::Utc::now().to_rfc3339();
        let _ = state.stitch_tx.send(StitchCreatedData {
            bead_id: created.id.clone(),
            title: created.title.clone(),
            project: project.to_string(),
            stitch_id: Some(stitch_id.clone()),
            source: source_str.clone(),
            actor: actor.to_string(),
            created_at: created_at.clone(),
        });
        let _ = state.bead_tx.send(crate::ws::BeadData {
            id: created.id.clone(),
            title: created.title.clone(),
            status: "open".to_string(),
            priority: req.priority.unwrap_or(2),
            issue_type: created.issue_type.clone(),
            created_at: created_at.clone(),
            updated_at: created_at,
            created_by: actor.to_string(),
            dependencies: vec![],
        });
    }

    Ok(SubmitResult {
        stitch_id,
        graph,
        created_beads,
    })
}

// ---------------------------------------------------------------------------
// Preview data helpers
// ---------------------------------------------------------------------------

/// Fetch "What Will This Take?" preview data for a Stitch draft
async fn fetch_stitch_preview(
    project: &str,
    title: &str,
    description: Option<&str>,
    labels: &[String],
    state: &crate::DaemonState,
) -> Option<StitchPreviewData> {
    use crate::api_preview::{load_historical_stitches, load_risk_library, check_file_conflicts};
    use crate::similarity::find_similar_stitches;

    // Load historical Stitches from fleet.db
    let historical_stitches = load_historical_stitches(project).ok()?;

    // Get prediction (cost p50/p90, duration p50/p90, likely adapter+model)
    let prediction = predict_stitch(title, description, labels, historical_stitches, 90);

    // Match risk patterns from Fix Lineage library
    let risk_library = load_risk_library().ok()?;
    let risk_matches = risk_library.match_draft(title, description, labels);

    // Check for file conflicts with currently-executing beads
    let file_conflicts = check_file_conflicts(state).await;

    // Find similar Stitches for reference
    let similar_stitches_refs = {
        let beads = state.beads.read().unwrap();
        let historical: Vec<_> = beads
            .iter()
            .map(|b| (b.id.clone(), b.title.clone(), None as Option<String>, vec![]))
            .collect();
        let similar = find_similar_stitches(title, description, labels, historical, 0.3, 5);
        similar
            .into_iter()
            .map(|s| SimilarStitchRef {
                id: s.id,
                title: s.title,
                similarity: s.similarity.score,
            })
            .collect()
    };

    Some(StitchPreviewData {
        schema_version: Some("1.0.0".to_string()),
        prediction: prediction.map(|p| PredictionData {
            cost: p.cost,
            duration: p.duration,
            likely_adapter_model: p.likely_adapter_model,
            similar_count: p.similar_count,
            data_range: p.data_range,
        }),
        risk_patterns: risk_matches
            .into_iter()
            .map(|m| RiskPatternMatch {
                pattern: RiskPatternInfo {
                    id: m.pattern.id,
                    name: m.pattern.name,
                    description: m.pattern.description,
                    fix_recommendation: m.pattern.fix_recommendation,
                    severity: format!("{:?}", m.pattern.severity).to_lowercase(),
                    category: format!("{:?}", m.pattern.category),
                },
                confidence: m.confidence,
                matched_keywords: m.matched_keywords,
                matched_labels: m.matched_labels,
            })
            .collect(),
        file_conflicts,
        similar_stitches: similar_stitches_refs,
    })
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Validate the stitch submit request fields against schema constraints
pub fn validate_stitch_draft(req: &StitchSubmitRequest) -> Result<(), (StatusCode, String)> {
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
        if !(0..=9).contains(&p) {
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
        && !req.source.starts_with("draft:")
    {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "invalid source '{}'. Must be one of: form, chat, bulk, template:<name>, draft:<id>",
                req.source
            ),
        ));
    }

    // Validate that the kind matches a decomposition rule
    validate_stitch_kind(&req.kind, req.has_acceptance_criteria.unwrap_or(false))?;

    Ok(())
}

/// Validate that the stitch kind matches at least one decomposition rule
pub fn validate_stitch_kind(kind: &str, has_acceptance_criteria: bool) -> Result<(), (StatusCode, String)> {
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
