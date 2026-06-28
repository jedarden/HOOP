//! REST API for Cross-Project Stitch Propagation (Marquee #11, Phase 5)
//!
//! Routes:
//!   POST /api/propagation/detect       — detect sibling projects for a closed Stitch
//!   GET  /api/propagation/{stitch_id}  — get propagation results for a Stitch

use crate::cross_project_propagation::{self, DetectionConfig, PropagationResult};
use crate::DaemonState;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

/// Build the propagation API router
pub fn router() -> Router<DaemonState> {
    Router::new()
        .route("/api/propagation/detect", post(detect_siblings))
        .route("/api/propagation/:stitch_id", get(get_propagation_result))
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DetectRequest {
    /// Stitch ID that was just closed
    stitch_id: String,
    /// Minimum similarity threshold (0-1, default: 0.5)
    #[serde(default = "default_min_similarity")]
    min_similarity: f64,
    /// Maximum sibling projects to return (default: 5)
    #[serde(default = "default_max_siblings")]
    max_siblings: usize,
    /// Lookback window in days (default: 90)
    #[serde(default = "default_lookback_days")]
    lookback_days: i64,
}

fn default_min_similarity() -> f64 {
    0.5
}

fn default_max_siblings() -> usize {
    5
}

fn default_lookback_days() -> i64 {
    90
}

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DetectResponse {
    /// Propagation detection result
    #[serde(flatten)]
    result: PropagationResult,
    /// Whether siblings were found
    has_siblings: bool,
    /// Sibling count
    sibling_count: usize,
}

/// POST /api/propagation/detect
///
/// Detect sibling projects for a recently closed Stitch.
/// This is the entry point for Marquee #11: Cross-Project Stitch Propagation.
#[utoipa::path(
    post,
    path = "/api/propagation/detect",
    tag = "propagation",
    request_body = DetectRequest,
    responses(
        (status = 200, description = "Sibling detection completed", body = DetectResponse),
        (status = 404, description = "Stitch not found"),
        (status = 500, description = "Detection failed")
    )
)]
async fn detect_siblings(
    State(_state): State<DaemonState>,
    Json(req): Json<DetectRequest>,
) -> Result<Json<DetectResponse>, axum::http::StatusCode> {
    let config = DetectionConfig {
        min_similarity: req.min_similarity,
        max_siblings: req.max_siblings,
        max_matches_per_project: 3,
        lookback_days: req.lookback_days,
    };

    match cross_project_propagation::detect_sibling_projects(&req.stitch_id, config) {
        Ok(result) => {
            let sibling_count = result.siblings.len();
            let has_siblings = sibling_count > 0;

            tracing::info!(
                stitch_id = %req.stitch_id,
                sibling_count,
                "Cross-project propagation detection completed"
            );

            Ok(Json(DetectResponse {
                result,
                has_siblings,
                sibling_count,
            }))
        }
        Err(e) => {
            tracing::error!("Failed to detect sibling projects: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /api/propagation/:stitch_id
///
/// Get cached propagation results for a Stitch.
#[utoipa::path(
    get,
    path = "/api/propagation/{stitch_id}",
    tag = "propagation",
    params(
        ("stitch_id" = String, Path, description = "Stitch ID")
    ),
    responses(
        (status = 200, description = "Propagation results", body = PropagationResult),
        (status = 404, description = "No propagation results found"),
        (status = 500, description = "Query failed")
    )
)]
async fn get_propagation_result(
    Path(stitch_id): Path<String>,
    State(_state): State<DaemonState>,
) -> Result<Json<PropagationResult>, axum::http::StatusCode> {
    // For now, this runs fresh detection each time
    // In production, we'd cache results in fleet.db
    let config = DetectionConfig::default();

    match cross_project_propagation::detect_sibling_projects(&stitch_id, config) {
        Ok(result) => {
            if result.siblings.is_empty() {
                Err(axum::http::StatusCode::NOT_FOUND)
            } else {
                Ok(Json(result))
            }
        }
        Err(e) => {
            tracing::error!("Failed to get propagation results: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
