//! REST API for the Reflection Ledger (§19.2 Multi-operator concurrency)
//!
//! Endpoints:
//! - GET  /api/reflections/proposals           — list pending proposals
//! - POST /api/reflections/{id}/approve        — approve a proposal
//! - POST /api/reflections/{id}/reject         — reject a proposal
//! - GET  /api/reflections                     — list approved reflections

use crate::fleet::{self, ReflectionLedgerEntry};
use axum::{
    extract::{ConnectInfo, Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing::info;

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

/// Response for listing reflection proposals
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ProposalsResponse {
    pub proposals: Vec<ReflectionLedgerEntry>,
    pub count: usize,
}

/// Response for listing approved reflections
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ReflectionsResponse {
    pub reflections: Vec<ReflectionLedgerEntry>,
    pub count: usize,
}

/// Request to approve a proposal
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ApproveProposalRequest {
    /// Optional comment on the approval
    pub comment: Option<String>,
}

/// Response after approving a proposal
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ApproveProposalResponse {
    pub proposal_id: String,
    pub status: String,
    pub approved_by: String,
    pub approved_at: String,
}

/// Request to reject a proposal
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RejectProposalRequest {
    /// Optional reason for rejection
    pub reason: Option<String>,
}

/// Response after rejecting a proposal
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RejectProposalResponse {
    pub proposal_id: String,
    pub status: String,
    pub rejection_count: i64,
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<crate::DaemonState> {
    Router::new()
        .route("/api/reflections/proposals", get(list_proposals))
        .route("/api/reflections", get(list_reflections))
        .route("/api/reflections/{id}/approve", post(approve_proposal))
        .route("/api/reflections/{id}/reject", post(reject_proposal))
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /api/reflections/proposals — list pending proposals (§19.2)
///
/// Returns all proposals in 'proposed' status for the approval queue UI.
#[utoipa::path(
    get,
    path = "/api/reflections/proposals",
    tag = "reflections",
    responses(
        (status = 200, description = "List of pending proposals", body = ProposalsResponse)
    )
)]
async fn list_proposals(
    State(_state): State<crate::DaemonState>,
) -> Result<Json<ProposalsResponse>, (StatusCode, String)> {
    let proposals = fleet::list_pending_reflection_proposals()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let count = proposals.len();
    Ok(Json(ProposalsResponse { proposals, count }))
}

/// GET /api/reflections — list approved reflections
///
/// Returns all approved reflection rules, optionally filtered by scope.
#[utoipa::path(
    get,
    path = "/api/reflections",
    tag = "reflections",
    responses(
        (status = 200, description = "List of approved reflection rules", body = ReflectionsResponse)
    )
)]
async fn list_reflections(
    State(_state): State<crate::DaemonState>,
) -> Result<Json<ReflectionsResponse>, (StatusCode, String)> {
    let reflections = fleet::list_approved_reflection_entries(None)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let count = reflections.len();
    Ok(Json(ReflectionsResponse {
        reflections,
        count,
    }))
}

/// POST /api/reflections/{id}/approve — approve a proposal (§19.2)
///
/// Single-operator action: marks a proposal as approved and records
/// who approved it and when.
#[utoipa::path(
    post,
    path = "/api/reflections/{id}/approve",
    tag = "reflections",
    params(
        ("id" = String, Path, description = "Proposal ID")
    ),
    request_body = ApproveProposalRequest,
    responses(
        (status = 200, description = "Proposal approved successfully", body = ApproveProposalResponse),
        (status = 404, description = "Proposal not found"),
        (status = 409, description = "Proposal not in 'proposed' status")
    )
)]
async fn approve_proposal(
    Path(id): Path<String>,
    State(state): State<crate::DaemonState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Json(_req): Json<ApproveProposalRequest>,
) -> Result<Json<ApproveProposalResponse>, (StatusCode, String)> {
    let actor = state.identity_cache.resolve(connect_info.map(|ci| ci.0));
    let now = chrono::Utc::now().to_rfc3339();

    let approved = fleet::approve_reflection_proposal(&id, &actor)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !approved {
        return Err((
            StatusCode::CONFLICT,
            format!(
                "Proposal '{}' not found or not in 'proposed' status",
                id
            ),
        ));
    }

    info!(
        proposal_id = %id,
        approved_by = %actor,
        "Reflection proposal approved"
    );

    // Emit reflection_proposal WS event
    let _ = state.reflection_tx.send(crate::ws::ReflectionProposalData {
        proposal_id: id.clone(),
        status: "approved".to_string(),
        rule: "".to_string(),
        scope: "".to_string(),
        action: "approved".to_string(),
        actor: actor.clone(),
    });

    Ok(Json(ApproveProposalResponse {
        proposal_id: id,
        status: "approved".to_string(),
        approved_by: actor,
        approved_at: now,
    }))
}

/// POST /api/reflections/{id}/reject — reject a proposal (§19.2)
///
/// Increments the rejection_count to prevent immediate re-proposal of
/// the same content (deduplicated via content_hash).
#[utoipa::path(
    post,
    path = "/api/reflections/{id}/reject",
    tag = "reflections",
    params(
        ("id" = String, Path, description = "Proposal ID")
    ),
    request_body = RejectProposalRequest,
    responses(
        (status = 200, description = "Proposal rejected successfully", body = RejectProposalResponse),
        (status = 404, description = "Proposal not found"),
        (status = 409, description = "Proposal not in 'proposed' status")
    )
)]
async fn reject_proposal(
    Path(id): Path<String>,
    State(state): State<crate::DaemonState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Json(_req): Json<RejectProposalRequest>,
) -> Result<Json<RejectProposalResponse>, (StatusCode, String)> {
    let rejected = fleet::reject_reflection_proposal(&id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !rejected {
        return Err((
            StatusCode::CONFLICT,
            format!(
                "Proposal '{}' not found or not in 'proposed' status",
                id
            ),
        ));
    }

    // Load the proposal to get the rejection_count
    let proposal = fleet::get_reflection_proposal(&id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("Proposal '{}' not found", id),
            )
        })?;

    info!(
        proposal_id = %id,
        rejection_count = proposal.rejection_count,
        "Reflection proposal rejected"
    );

    // Emit reflection_proposal WS event
    let actor = state.identity_cache.resolve(connect_info.map(|ci| ci.0));
    let _ = state.reflection_tx.send(crate::ws::ReflectionProposalData {
        proposal_id: id.clone(),
        status: "rejected".to_string(),
        rule: proposal.rule.clone(),
        scope: proposal.scope.clone(),
        action: "rejected".to_string(),
        actor,
    });

    Ok(Json(RejectProposalResponse {
        proposal_id: id,
        status: "rejected".to_string(),
        rejection_count: proposal.rejection_count,
    }))
}
