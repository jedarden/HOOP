//! REST API for the stitch draft queue (§3.10 read-first principle)
//!
//! Endpoints:
//! - GET  /api/drafts                                   — list pending drafts (all projects)
//! - GET  /api/p/{project}/drafts                       — list pending drafts for a project
//! - GET  /api/drafts/{draft_id}                        — get a single draft
//! - POST /api/drafts/{draft_id}/approve                — approve → submit (triggers br create flow)
//! - POST /api/drafts/{draft_id}/edit                   — edit draft fields (new version)
//! - POST /api/drafts/{draft_id}/reject                 — reject with optional reason
//!
//! Agent calls to `create_stitch` insert into draft_queue, not directly to br.
//! The operator sees drafts in the UI preview queue and must explicitly approve
//! before any beads are created.

use crate::fleet::{self, ActionKind, ActionResult, DraftRow};
use axum::{
    extract::{ConnectInfo, Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing::{info, warn};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

/// Response for a single draft
#[derive(Debug, Serialize)]
pub struct DraftResponse {
    #[serde(flatten)]
    pub draft: DraftRow,
}

/// Response for listing drafts
#[derive(Debug, Serialize)]
pub struct DraftListResponse {
    pub drafts: Vec<DraftRow>,
    pub count: usize,
}

/// Request to create a new draft
#[derive(Debug, Deserialize)]
pub struct CreateDraftRequest {
    pub project: String,
    pub title: String,
    pub kind: String,
    pub description: Option<String>,
    pub has_acceptance_criteria: Option<bool>,
    pub priority: Option<i64>,
    pub labels: Option<Vec<String>>,
    /// Source of the request: "chat", "bulk", or "template:<name>"
    #[serde(default)]
    pub source: String,
    /// Optional agent session ID
    pub agent_session_id: Option<String>,
    /// Optional turn ID for tracking which agent turn created this draft
    pub turn_id: Option<String>,
    /// If true, bypass the dedup check and create anyway
    #[serde(default)]
    pub force_create: bool,
}

/// Response after creating a draft
#[derive(Debug, Serialize)]
pub struct CreateDraftResponse {
    pub draft_id: String,
    pub status: String,
}

/// Request to approve a draft
#[derive(Debug, Deserialize)]
pub struct ApproveRequest {
    /// Optional override for the decomposition graph
    pub override_: Option<crate::stitch_decompose::GraphOverride>,
    /// Allow bypassing dedup check
    #[serde(default)]
    pub force_create: bool,
}

/// Request to edit a draft
#[derive(Debug, Deserialize)]
pub struct EditDraftRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub kind: Option<String>,
    pub priority: Option<i64>,
    pub labels: Option<Vec<String>>,
}

/// Request to reject a draft
#[derive(Debug, Deserialize)]
pub struct RejectRequest {
    /// Optional reason for rejection
    pub reason: Option<String>,
}

/// Response after approving and submitting a draft
#[derive(Debug, Serialize)]
pub struct ApproveResponse {
    pub draft_id: String,
    pub stitch_id: String,
    pub created_beads: Vec<crate::api_stitch_decompose::CreatedBead>,
    pub graph: Option<crate::stitch_decompose::BeadGraph>,
}

/// Response after editing a draft
#[derive(Debug, Serialize)]
pub struct EditResponse {
    pub draft_id: String,
    pub version: i64,
    pub status: String,
}

/// Response after rejecting a draft
#[derive(Debug, Serialize)]
pub struct RejectResponse {
    pub draft_id: String,
    pub status: String,
    pub reason: Option<String>,
}

/// Request to report a false positive dedup match
#[derive(Debug, Deserialize)]
pub struct ReportFalsePositiveRequest {
    /// The match ID that was incorrectly flagged
    pub match_id: String,
}

/// Response after reporting a false positive
#[derive(Debug, Serialize)]
pub struct ReportFalsePositiveResponse {
    pub success: bool,
    pub false_positive_rate: f64,
}

/// Response for deduplication statistics
#[derive(Debug, Serialize)]
pub struct DedupStatsResponse {
    pub total_checks: u64,
    pub duplicates_found: u64,
    pub false_positives_reported: u64,
    pub false_positive_rate_cumulative: f64,
    pub false_positive_rate_30d: f64,
    pub threshold: f64,
    pub indexed_items: usize,
}

// ---------------------------------------------------------------------------
// §19.1 Draft concurrency types
// ---------------------------------------------------------------------------

/// Request to open a draft form
#[derive(Debug, Deserialize)]
pub struct OpenDraftRequest {
    pub project: String,
}

/// Response after opening a draft
#[derive(Debug, Serialize)]
pub struct OpenDraftResponse {
    pub draft_id: String,
    pub status: String,
    pub opened_at: String,
}

/// Request to autosave draft content
#[derive(Debug, Deserialize)]
pub struct AutosaveDraftRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub kind: Option<String>,
    pub priority: Option<i64>,
    pub labels: Option<Vec<String>>,
}

/// Response after autosaving a draft
#[derive(Debug, Serialize)]
pub struct AutosaveDraftResponse {
    pub draft_id: String,
    pub last_autosave_at: String,
}

/// Response after abandoning a draft
#[derive(Debug, Serialize)]
pub struct AbandonDraftResponse {
    pub draft_id: String,
    pub status: String,
    pub abandoned_at: String,
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<crate::DaemonState> {
    Router::new()
        .route("/api/drafts", get(list_all_drafts).post(create_draft))
        .route("/api/p/{project}/drafts", get(list_project_drafts))
        .route("/api/drafts/{draft_id}", get(get_draft))
        .route("/api/drafts/{draft_id}/approve", post(approve_draft))
        .route("/api/drafts/{draft_id}/edit", post(edit_draft))
        .route("/api/drafts/{draft_id}/reject", post(reject_draft))
        .route("/api/drafts/{draft_id}/open", post(open_draft))
        .route("/api/drafts/{draft_id}/autosave", post(autosave_draft))
        .route("/api/drafts/{draft_id}/abandon", post(abandon_draft))
        .route("/api/dedup/stats", get(get_dedup_stats))
        .route("/api/dedup/false-positive", post(report_false_positive))
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /api/drafts — list all actionable drafts (pending + edited) across projects
async fn list_all_drafts(
    State(_state): State<crate::DaemonState>,
) -> Result<Json<DraftListResponse>, (StatusCode, String)> {
    let mut drafts = fleet::list_drafts(None, Some("pending"), 200)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    drafts.extend(
        fleet::list_drafts(None, Some("edited"), 200)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    );
    drafts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    let count = drafts.len();
    Ok(Json(DraftListResponse { drafts, count }))
}

/// GET /api/p/{project}/drafts — list actionable drafts for a project
async fn list_project_drafts(
    Path(project): Path<String>,
    State(_state): State<crate::DaemonState>,
) -> Result<Json<DraftListResponse>, (StatusCode, String)> {
    crate::id_validators::validate_project_name(&project).map_err(crate::id_validators::rejection)?;
    let mut drafts = fleet::list_drafts(Some(&project), Some("pending"), 200)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    drafts.extend(
        fleet::list_drafts(Some(&project), Some("edited"), 200)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    );
    drafts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    let count = drafts.len();
    Ok(Json(DraftListResponse { drafts, count }))
}

/// GET /api/drafts/{draft_id} — get a single draft
async fn get_draft(
    Path(draft_id): Path<String>,
    State(_state): State<crate::DaemonState>,
) -> Result<Json<DraftResponse>, (StatusCode, String)> {
    crate::id_validators::validate_draft_id(&draft_id)
        .map_err(crate::id_validators::rejection)?;

    let draft = fleet::get_draft(&draft_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Draft '{}' not found", draft_id)))?;
    Ok(Json(DraftResponse { draft }))
}

/// POST /api/drafts — create a new draft with deduplication check
///
/// This is the primary entry point for agent-initiated stitch creation.
/// The draft is checked for duplicates against all open stitches/beads
/// across all projects before being inserted into the queue.
async fn create_draft(
    State(state): State<crate::DaemonState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Json(req): Json<CreateDraftRequest>,
) -> Result<Json<CreateDraftResponse>, (StatusCode, String)> {
    // Validate project name from request body
    crate::id_validators::validate_project_name(&req.project).map_err(crate::id_validators::rejection)?;

    // Validate project exists
    let _project_path = resolve_project_path(&req.project, &state)?;

    // Validate the stitch kind
    crate::api_stitch_decompose::validate_stitch_kind(&req.kind, req.has_acceptance_criteria.unwrap_or(false))?;

    // Dedup check (unless force_create)
    if !req.force_create {
        let index = state.vector_index.read().unwrap();
        let matches = index.check_duplicate(&req.title, req.description.as_deref());
        if !matches.is_empty() {
            let best = &matches[0];
            let message = format!(
                "this looks like `{}/{}`, which is in progress. Continue that, add this as a child, or proceed as new?",
                best.item.project, best.item.id
            );
            return Err((StatusCode::CONFLICT, message));
        }
    }

    let actor = resolve_actor(connect_info.map(|ci| ci.0));
    let now = chrono::Utc::now().to_rfc3339();
    let draft_id = format!("draft-{}", Uuid::new_v4());

    // §18.1 secrets scan: flag secrets in draft title and body (Phase 4)
    {
        let findings = crate::redaction::scan_draft_body(&req.title, req.description.as_deref());
        if !findings.is_empty() {
            warn!(
                project = %req.project,
                draft_id = %draft_id,
                findings = findings.len(),
                "Draft body contains potential secrets — flagged for operator review (§18.1)"
            );
        }
    }

    // Build the draft row
    let draft_row = fleet::DraftRow {
        id: draft_id.clone(),
        project: req.project.clone(),
        title: req.title.clone(),
        kind: req.kind.clone(),
        description: req.description.clone(),
        has_acceptance_criteria: req.has_acceptance_criteria.unwrap_or(false),
        priority: req.priority,
        labels: req.labels.clone().unwrap_or_default(),
        created_by: actor.clone(),
        created_at: now.clone(),
        source: if req.source.is_empty() { "chat".to_string() } else { req.source.clone() },
        agent_session_id: req.agent_session_id.clone(),
        turn_id: req.turn_id.clone(),
        status: "pending".to_string(),
        version: 1,
        original_json: None,
        resolved_by: None,
        resolved_at: None,
        rejection_reason: None,
        stitch_id: None,
        preview_json: None,
        // §19.1 Draft concurrency fields
        opened_by: Some(actor.clone()),
        opened_at: Some(now.clone()),
        last_autosave_at: None,
        abandoned_at: None,
    };

    // Insert the draft into the queue
    fleet::insert_draft(&draft_row)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Audit: draft created
    if let Err(e) = fleet::write_audit_row(
        &actor,
        fleet::ActionKind::DraftCreated,
        &draft_id,
        Some(&req.project),
        Some(serde_json::json!({
            "title": req.title,
            "kind": req.kind,
            "source": req.source,
        }).to_string()),
        fleet::ActionResult::Success,
        None,
        Some("agent"),
        None,
        None,
    ) {
        warn!("Failed to write DraftCreated audit row: {}", e);
    }

    info!("Draft {} created by {} in project '{}': {}", draft_id, actor, req.project, req.title);

    // Emit draft_update WS event
    let _ = state.draft_tx.send(crate::ws::DraftUpdateData {
        draft_id: draft_id.clone(),
        project: req.project.clone(),
        title: req.title.clone(),
        kind: req.kind.clone(),
        status: "pending".to_string(),
        action: "created".to_string(),
        actor: actor.clone(),
        created_by: actor,
        version: 1,
        rejection_reason: None,
    });

    Ok(Json(CreateDraftResponse {
        draft_id,
        status: "pending".to_string(),
    }))
}

/// POST /api/drafts/{draft_id}/approve — approve a draft and submit it
///
/// This triggers the full submit flow: validate → decompose → br create → audit → WS event.
/// The draft status transitions to 'approved' and then 'submitted' on success.
async fn approve_draft(
    Path(draft_id): Path<String>,
    State(state): State<crate::DaemonState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Json(req): Json<ApproveRequest>,
) -> Result<Json<ApproveResponse>, (StatusCode, String)> {
    crate::id_validators::validate_draft_id(&draft_id)
        .map_err(crate::id_validators::rejection)?;

    let draft = fleet::get_draft(&draft_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Draft '{}' not found", draft_id)))?;

    if draft.status != "pending" && draft.status != "edited" {
        return Err((
            StatusCode::CONFLICT,
            format!("Draft '{}' is in status '{}', expected 'pending' or 'edited'", draft_id, draft.status),
        ));
    }

    let actor = resolve_actor(connect_info.map(|ci| ci.0));
    let now = chrono::Utc::now().to_rfc3339();

    // Mark draft as approved
    fleet::update_draft_status(&draft_id, "approved", Some(&actor), Some(&now), None, None)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Audit: draft approved
    if let Err(e) = fleet::write_audit_row(
        &actor,
        ActionKind::DraftApproved,
        &draft_id,
        Some(&draft.project),
        Some(serde_json::json!({
            "title": draft.title,
            "kind": draft.kind,
            "source": draft.source,
            "original_actor": draft.created_by,
        }).to_string()),
        ActionResult::Success,
        None,
        Some("operator"),
        None,
        None,
    ) {
        warn!("Failed to write DraftApproved audit row: {}", e);
    }

    // Build a StitchSubmitRequest from the draft and call the existing submit flow
    let submit_req = crate::api_stitch_decompose::StitchSubmitRequest {
        kind: draft.kind.clone(),
        title: draft.title.clone(),
        description: draft.description.clone(),
        has_acceptance_criteria: Some(draft.has_acceptance_criteria),
        priority: draft.priority,
        labels: Some(draft.labels.clone()),
        override_: req.override_,
        source: format!("draft:{}", draft_id),
        stitch_id: None,
        force_create: req.force_create,
    };

    // Call the existing submit_stitch handler logic by invoking it directly
    let project_path = resolve_project_path(&draft.project, &state)?;

    // Validate
    crate::api_stitch_decompose::validate_stitch_draft(&submit_req)?;

    // Dedup check (unless force_create)
    if !submit_req.force_create {
        let index = state.vector_index.read().unwrap();
        let dedup_matches = index.check_duplicate(&submit_req.title, submit_req.description.as_deref());
        if !dedup_matches.is_empty() {
            let best = &dedup_matches[0];
            let message = format!(
                "this looks like `{}/{}`, which is in progress. Continue that, add this as a child, or proceed as new?",
                best.item.project, best.item.id
            );
            return Err((StatusCode::CONFLICT, message));
        }
    }

    let beads_dir = project_path.join(".beads");
    if !beads_dir.exists() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("Project '{}' has no .beads directory", draft.project),
        ));
    }

    // Use the internal submit logic
    let submit_result = crate::api_stitch_decompose::submit_stitch_internal(
        &draft.project,
        &project_path,
        &submit_req,
        &state,
        &actor,
    )
    .await?;

    // Update draft with stitch_id and mark as submitted
    fleet::update_draft_status(
        &draft_id,
        "submitted",
        Some(&actor),
        Some(&now),
        None,
        Some(&submit_result.stitch_id),
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Audit: draft submitted
    if let Err(e) = fleet::write_audit_row(
        &actor,
        ActionKind::DraftApproved,
        &draft_id,
        Some(&draft.project),
        Some(serde_json::json!({
            "stitch_id": submit_result.stitch_id,
            "bead_count": submit_result.created_beads.len(),
        }).to_string()),
        ActionResult::Success,
        None,
        Some("operator"),
        None,
        None,
    ) {
        warn!("Failed to write draft submitted audit row: {}", e);
    }

    // Emit stitch_created WS event
    let _ = state.stitch_tx.send(crate::ws::StitchCreatedData {
        bead_id: draft_id.clone(),
        title: format!("Draft approved: {}", draft.title),
        project: draft.project.clone(),
        stitch_id: Some(submit_result.stitch_id.clone()),
        source: format!("draft:{}", draft_id),
        actor: actor.clone(),
        created_at: now.clone(),
    });

    // Emit draft_update WS event
    let _ = state.draft_tx.send(crate::ws::DraftUpdateData {
        draft_id: draft_id.clone(),
        project: draft.project.clone(),
        title: draft.title.clone(),
        kind: draft.kind.clone(),
        status: "approved".to_string(),
        action: "approved".to_string(),
        actor: actor.clone(),
        created_by: draft.created_by.clone(),
        version: draft.version,
        rejection_reason: None,
    });

    Ok(Json(ApproveResponse {
        draft_id,
        stitch_id: submit_result.stitch_id,
        created_beads: submit_result.created_beads,
        graph: Some(submit_result.graph),
    }))
}

/// POST /api/drafts/{draft_id}/edit — edit a draft's fields
async fn edit_draft(
    Path(draft_id): Path<String>,
    State(_state): State<crate::DaemonState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Json(req): Json<EditDraftRequest>,
) -> Result<Json<EditResponse>, (StatusCode, String)> {
    crate::id_validators::validate_draft_id(&draft_id)
        .map_err(crate::id_validators::rejection)?;

    let draft = fleet::get_draft(&draft_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Draft '{}' not found", draft_id)))?;

    if draft.status != "pending" && draft.status != "edited" {
        return Err((
            StatusCode::CONFLICT,
            format!("Draft '{}' is in status '{}', cannot edit", draft_id, draft.status),
        ));
    }

    // Validate kind if provided
    if let Some(kind) = &req.kind {
        crate::api_stitch_decompose::validate_stitch_kind(kind, draft.has_acceptance_criteria)?;
    }

    fleet::edit_draft(
        &draft_id,
        req.title.as_deref(),
        req.description.as_deref(),
        req.kind.as_deref(),
        req.priority,
        req.labels.as_deref(),
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let updated = fleet::get_draft(&draft_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "Draft disappeared after edit".to_string()))?;

    // Audit: draft edited
    let actor = resolve_actor(connect_info.map(|ci| ci.0));
    let ws_state = _state;
    if let Err(e) = fleet::write_audit_row(
        &actor,
        ActionKind::DraftEdited,
        &draft_id,
        Some(&draft.project),
        Some(serde_json::json!({
            "version": updated.version,
            "title": updated.title,
        }).to_string()),
        ActionResult::Success,
        None,
        Some("operator"),
        None,
        None,
    ) {
        warn!("Failed to write DraftEdited audit row: {}", e);
    }

    // Emit draft_update WS event
    let _ = ws_state.draft_tx.send(crate::ws::DraftUpdateData {
        draft_id: draft_id.clone(),
        project: draft.project.clone(),
        title: updated.title.clone(),
        kind: updated.kind.clone(),
        status: "edited".to_string(),
        action: "edited".to_string(),
        actor: actor.clone(),
        created_by: draft.created_by.clone(),
        version: updated.version,
        rejection_reason: None,
    });

    Ok(Json(EditResponse {
        draft_id,
        version: updated.version,
        status: updated.status,
    }))
}

/// POST /api/drafts/{draft_id}/reject — reject a draft with optional reason
async fn reject_draft(
    Path(draft_id): Path<String>,
    State(state): State<crate::DaemonState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Json(req): Json<RejectRequest>,
) -> Result<Json<RejectResponse>, (StatusCode, String)> {
    crate::id_validators::validate_draft_id(&draft_id)
        .map_err(crate::id_validators::rejection)?;

    let draft = fleet::get_draft(&draft_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Draft '{}' not found", draft_id)))?;

    if draft.status != "pending" && draft.status != "edited" {
        return Err((
            StatusCode::CONFLICT,
            format!("Draft '{}' is in status '{}', cannot reject", draft_id, draft.status),
        ));
    }

    let actor = resolve_actor(connect_info.map(|ci| ci.0));
    let now = chrono::Utc::now().to_rfc3339();

    fleet::update_draft_status(
        &draft_id,
        "rejected",
        Some(&actor),
        Some(&now),
        req.reason.as_deref(),
        None,
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Audit: draft rejected
    if let Err(e) = fleet::write_audit_row(
        &actor,
        ActionKind::DraftRejected,
        &draft_id,
        Some(&draft.project),
        Some(serde_json::json!({
            "title": draft.title,
            "kind": draft.kind,
            "rejection_reason": req.reason,
        }).to_string()),
        ActionResult::Success,
        None,
        Some("operator"),
        None,
        None,
    ) {
        warn!("Failed to write DraftRejected audit row: {}", e);
    }

    info!(
        "Draft {} rejected by {} (reason: {})",
        draft_id,
        actor,
        req.reason.as_deref().unwrap_or("none")
    );

    // Emit draft_update WS event
    let _ = state.draft_tx.send(crate::ws::DraftUpdateData {
        draft_id: draft_id.clone(),
        project: draft.project.clone(),
        title: draft.title.clone(),
        kind: draft.kind.clone(),
        status: "rejected".to_string(),
        action: "rejected".to_string(),
        actor: actor.clone(),
        created_by: draft.created_by.clone(),
        version: draft.version,
        rejection_reason: req.reason.clone(),
    });

    Ok(Json(RejectResponse {
        draft_id,
        status: "rejected".to_string(),
        reason: req.reason,
    }))
}

// ---------------------------------------------------------------------------
// §19.1 Draft concurrency handlers
// ---------------------------------------------------------------------------

/// POST /api/drafts/{draft_id}/open — open a draft form (§19.1 Draft concurrency)
///
/// Creates or updates a draft with opened_by/opened_at tracking.
/// When the operator opens the draft form, this endpoint is called to
/// ensure the draft is persisted server-side immediately. If a draft
/// with this ID already exists (e.g., was autosaved before), it updates
/// the opened_by/opened_at fields and clears any abandoned_at.
async fn open_draft(
    Path(draft_id): Path<String>,
    State(_state): State<crate::DaemonState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Json(req): Json<OpenDraftRequest>,
) -> Result<Json<OpenDraftResponse>, (StatusCode, String)> {
    crate::id_validators::validate_draft_id(&draft_id)
        .map_err(crate::id_validators::rejection)?;
    crate::id_validators::validate_project_name(&req.project).map_err(crate::id_validators::rejection)?;

    let actor = resolve_actor(connect_info.map(|ci| ci.0));
    let now = chrono::Utc::now().to_rfc3339();

    fleet::open_draft(&draft_id, &req.project, &actor)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Audit: draft opened
    if let Err(e) = fleet::write_audit_row(
        &actor,
        fleet::ActionKind::DraftOpened,
        &draft_id,
        Some(&req.project),
        None,
        fleet::ActionResult::Success,
        None,
        Some("operator"),
        None,
        None,
    ) {
        warn!("Failed to write DraftOpened audit row: {}", e);
    }

    info!("Draft {} opened by {} in project '{}'", draft_id, actor, req.project);

    Ok(Json(OpenDraftResponse {
        draft_id,
        status: "pending".to_string(),
        opened_at: now,
    }))
}

/// POST /api/drafts/{draft_id}/autosave — autosave draft content (§19.1 Draft concurrency)
///
/// Autosaves draft fields every 5 seconds or on field change.
/// Updates title, description, kind, priority, labels, and last_autosave_at.
/// Does not increment version (only manual edits via /edit increment version).
async fn autosave_draft(
    Path(draft_id): Path<String>,
    State(state): State<crate::DaemonState>,
    Json(req): Json<AutosaveDraftRequest>,
) -> Result<Json<AutosaveDraftResponse>, (StatusCode, String)> {
    crate::id_validators::validate_draft_id(&draft_id)
        .map_err(crate::id_validators::rejection)?;

    // Verify draft exists
    let _draft = fleet::get_draft(&draft_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Draft '{}' not found", draft_id)))?;

    let now = chrono::Utc::now().to_rfc3339();

    fleet::autosave_draft(
        &draft_id,
        req.title.as_deref(),
        req.description.as_deref(),
        req.kind.as_deref(),
        req.priority,
        req.labels.as_deref(),
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Emit draft_update WS event for real-time collaboration
    let _ = state.draft_tx.send(crate::ws::DraftUpdateData {
        draft_id: draft_id.clone(),
        project: "".to_string(), // Will be filled by listener
        title: req.title.unwrap_or_default(),
        kind: req.kind.unwrap_or_default(),
        status: "pending".to_string(),
        action: "autosaved".to_string(),
        actor: "".to_string(),
        created_by: "".to_string(),
        version: 0,
        rejection_reason: None,
    });

    Ok(Json(AutosaveDraftResponse {
        draft_id,
        last_autosave_at: now,
    }))
}

/// POST /api/drafts/{draft_id}/abandon — abandon a draft (§19.1 Draft concurrency)
///
/// Marks a draft as abandoned when the form closes without submit.
/// Sets abandoned_at timestamp. Drafts are retained for 7 days before cleanup.
async fn abandon_draft(
    Path(draft_id): Path<String>,
    State(_state): State<crate::DaemonState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
) -> Result<Json<AbandonDraftResponse>, (StatusCode, String)> {
    crate::id_validators::validate_draft_id(&draft_id)
        .map_err(crate::id_validators::rejection)?;

    // Verify draft exists and is in an abandonable state
    let draft = fleet::get_draft(&draft_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Draft '{}' not found", draft_id)))?;

    if draft.status == "submitted" || draft.status == "approved" || draft.status == "rejected" {
        return Err((
            StatusCode::CONFLICT,
            format!("Draft '{}' is in status '{}', cannot be abandoned", draft_id, draft.status),
        ));
    }

    let actor = resolve_actor(connect_info.map(|ci| ci.0));
    let now = chrono::Utc::now().to_rfc3339();

    fleet::abandon_draft(&draft_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Audit: draft abandoned
    if let Err(e) = fleet::write_audit_row(
        &actor,
        fleet::ActionKind::DraftAbandoned,
        &draft_id,
        Some(&draft.project),
        Some(serde_json::json!({
            "title": draft.title,
            "kind": draft.kind,
        }).to_string()),
        fleet::ActionResult::Success,
        None,
        Some("operator"),
        None,
        None,
    ) {
        warn!("Failed to write DraftAbandoned audit row: {}", e);
    }

    info!("Draft {} abandoned by {}", draft_id, actor);

    Ok(Json(AbandonDraftResponse {
        draft_id,
        status: "abandoned".to_string(),
        abandoned_at: now,
    }))
}

/// POST /api/dedup/false-positive — report a false positive dedup match
///
/// Users can call this when the dedup system incorrectly flags a draft as duplicate.
/// This helps track the false positive rate for threshold tuning.
async fn report_false_positive(
    State(state): State<crate::DaemonState>,
    _connect_info: Option<ConnectInfo<SocketAddr>>,
    Json(_req): Json<ReportFalsePositiveRequest>,
) -> Result<Json<ReportFalsePositiveResponse>, (StatusCode, String)> {
    // Record the false positive
    state.vector_index.read().unwrap().report_false_positive();

    // Get updated false positive rates (cumulative and 30-day rolling)
    let index = state.vector_index.read().unwrap();
    let fp_rate = index.false_positive_rate();
    let fp_rate_30d = index.false_positive_rate_30d();

    info!(
        "False positive reported; FP rate: {:.2}% (30d: {:.2}%)",
        fp_rate * 100.0,
        fp_rate_30d * 100.0
    );

    Ok(Json(ReportFalsePositiveResponse {
        success: true,
        false_positive_rate: fp_rate_30d, // Return 30-day rate for threshold tuning
    }))
}

/// GET /api/dedup/stats — get deduplication statistics
///
/// Returns current statistics for the semantic deduplication system,
/// including false positive rates (cumulative and 30-day rolling) for
/// monitoring threshold tuning.
async fn get_dedup_stats(
    State(state): State<crate::DaemonState>,
) -> Result<Json<DedupStatsResponse>, (StatusCode, String)> {
    let index = state.vector_index.read().unwrap();
    let stats = index.stats();

    Ok(Json(DedupStatsResponse {
        total_checks: stats.total_checks,
        duplicates_found: stats.duplicates_found,
        false_positives_reported: stats.false_positives_reported,
        false_positive_rate_cumulative: index.false_positive_rate(),
        false_positive_rate_30d: index.false_positive_rate_30d(),
        threshold: index.threshold(),
        indexed_items: index.len(),
    }))
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

/// Resolve actor identity via Tailscale whois, falling back to OS username
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
