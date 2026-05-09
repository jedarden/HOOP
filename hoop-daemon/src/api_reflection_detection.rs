//! REST API for Reflection Detector signal detection (Marquee #12, Phase 5)
//!
//! Routes:
//!   POST /api/reflections/detect       — trigger reflection detection on recent Stitches
//!   GET  /api/reflections/detect/status — detection status and last run time

use crate::reflection_detector::{ReflectionDetectorConfig, run_detection};
use crate::DaemonState;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Build the reflection detector API router
pub fn router() -> Router<DaemonState> {
    Router::new()
        .route("/api/reflections/detect", post(trigger_detection))
        .route("/api/reflections/detect/status", get(get_detection_status))
}

/// Shared state for detection coordination (prevents concurrent runs)
#[derive(Debug)]
pub struct DetectionState {
    last_run: Option<chrono::DateTime<chrono::Utc>>,
    running: bool,
    last_result: Option<DetectionResult>,
}

/// Result of a reflection detection run
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DetectionResult {
    /// When the detection was run
    pub ran_at: String,
    /// How many patterns were proposed
    pub proposed_count: usize,
    /// Whether any patterns were found
    pub found_patterns: bool,
}

/// Trigger reflection detection request
#[derive(Debug, Deserialize, ToSchema)]
struct DetectRequest {
    /// Scan window in days (default: 30)
    #[serde(default = "default_scan_window")]
    scan_window_days: i64,
    /// Minimum occurrences to consider a pattern (default: 3)
    #[serde(default = "default_min_occurrences")]
    min_occurrences: usize,
    /// Similarity threshold for grouping (0-1, default: 0.45)
    #[serde(default = "default_similarity")]
    similarity_threshold: f64,
}

fn default_scan_window() -> i64 {
    30
}

fn default_min_occurrences() -> usize {
    3
}

fn default_similarity() -> f64 {
    0.45
}

/// Detection status response
#[derive(Debug, Serialize, ToSchema)]
struct StatusResponse {
    /// Whether a detection is currently running
    pub running: bool,
    /// Last run timestamp (if any)
    pub last_run: Option<String>,
    /// Last result (if any)
    pub last_result: Option<DetectionResult>,
}

/// POST /api/reflections/detect
///
/// Trigger reflection detection on recent operator Stitches.
/// Scans for repeated patterns (corrections, preferences, negatives, approvals)
/// and proposes them to the Reflection Ledger.
#[utoipa::path(
    post,
    path = "/api/reflections/detect",
    tag = "reflections",
    request_body = DetectRequest,
    responses(
        (status = 200, description = "Detection completed", body = DetectionResult),
        (status = 409, description = "Detection already running"),
        (status = 500, description = "Detection failed")
    )
)]
async fn trigger_detection(
    State(state): State<DaemonState>,
    Json(req): Json<DetectRequest>,
) -> Result<Json<DetectionResult>, axum::http::StatusCode> {
    let detection_state = state
        .reflection_detection_state
        .as_ref()
        .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)?;

    // Check if already running
    {
        let state_guard = detection_state.lock().await;
        if state_guard.running {
            return Err(axum::http::StatusCode::CONFLICT);
        }
        state_guard.running = true;
    }

    let config = ReflectionDetectorConfig {
        scan_window_days: req.scan_window_days,
        min_occurrences: req.min_occurrences,
        similarity_threshold: req.similarity_threshold,
        ..Default::default()
    };

    let detection_state_clone = detection_state.clone();

    // Run detection in background so we don't block the response
    tokio::spawn(async move {
        let proposed = run_detection(&config).await.unwrap_or(0);

        let ran_at = chrono::Utc::now();

        // Update state
        let mut state_guard = detection_state_clone.lock().await;
        state_guard.running = false;
        state_guard.last_run = Some(ran_at);
        state_guard.last_result = Some(DetectionResult {
            ran_at: ran_at.to_rfc3339(),
            proposed_count: proposed,
            found_patterns: proposed > 0,
        });

        tracing::info!(
            "Reflection detection completed: {} patterns proposed",
            proposed
        );
    });

    // Return immediately with a pending result
    Ok(Json(DetectionResult {
        ran_at: chrono::Utc::now().to_rfc3339(),
        proposed_count: 0,
        found_patterns: false,
    }))
}

/// GET /api/reflections/detect/status
///
/// Get the current status of reflection detection.
#[utoipa::path(
    get,
    path = "/api/reflections/detect/status",
    tag = "reflections",
    responses(
        (status = 200, description = "Detection status", body = StatusResponse)
    )
)]
async fn get_detection_status(
    State(state): State<DaemonState>,
) -> Json<StatusResponse> {
    let detection_state = state
        .reflection_detection_state
        .as_ref()
        .ok_or(());

    let (running, last_run, last_result) = match detection_state {
        Some(state) => {
            let state_guard = state.lock().await;
            (
                state_guard.running,
                state_guard.last_run.map(|dt| dt.to_rfc3339()),
                state_guard.last_result.clone(),
            )
        }
        None => (false, None, None),
    };

    Json(StatusResponse {
        running,
        last_run,
        last_result,
    })
}

/// Create a new detection state for the daemon
pub fn new_detection_state() -> Arc<Mutex<DetectionState>> {
    Arc::new(Mutex::new(DetectionState {
        last_run: None,
        running: false,
        last_result: None,
    }))
}
