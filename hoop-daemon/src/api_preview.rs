//! Preview API for "What Will This Take?" before submitting a bead
//!
//! GET /api/p/:project/beads/preview — preview cost, duration, risk patterns,
//! likely claimer, and file-overlap conflicts.
//!
//! Acceptance (§6 Phase 4 marquee #7):
//! - Similarity: lexical on title/body/labels; embedding deferred to phase 5
//! - Cost/duration percentile from last 90d
//! - Risk pattern from Fix Lineage library (phase 2 marquee #4)
//! - Strand NEVER appears as a routing factor (§8.4 non-goal)
//! - UI preview card renders within 2s of draft change

use crate::predictor::{HistoricalStitch, predict_stitch};
use crate::risk_patterns::{FixLineageLibrary, default_risk_patterns};
use crate::similarity::find_similar_stitches;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Preview request query parameters
#[derive(Debug, Deserialize)]
pub struct PreviewRequest {
    title: String,
    description: Option<String>,
    labels: Option<String>,
}

/// Stitch preview response (matches hoop-schema/schemas/stitch_preview.json)
#[derive(Debug, Serialize)]
pub struct StitchPreview {
    schema_version: String,
    prediction: Option<PredictionData>,
    risk_patterns: Vec<RiskPatternMatch>,
    file_conflicts: Vec<FileConflict>,
    similar_stitches: Vec<SimilarStitchRef>,
}

#[derive(Debug, Serialize)]
struct PredictionData {
    cost: PercentileEstimate,
    duration: PercentileEstimate,
    #[serde(skip_serializing_if = "Option::is_none")]
    likely_adapter_model: Option<String>,
    similar_count: usize,
    data_range: DateRange,
}

#[derive(Debug, Serialize)]
struct PercentileEstimate {
    p50: f64,
    p90: f64,
    count: usize,
}

#[derive(Debug, Serialize)]
struct DateRange {
    start: String,
    end: String,
}

#[derive(Debug, Serialize)]
struct RiskPatternMatch {
    pattern: RiskPatternInfo,
    confidence: f64,
    matched_keywords: Vec<String>,
    matched_labels: Vec<String>,
}

#[derive(Debug, Serialize)]
struct RiskPatternInfo {
    id: String,
    name: String,
    description: String,
    fix_recommendation: String,
    severity: String,
    category: String,
}

#[derive(Debug, Serialize)]
pub struct FileConflict {
    pub bead_id: String,
    pub title: String,
    pub project: String,
    pub overlapping_files: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SimilarStitchRef {
    id: String,
    title: String,
    similarity: f64,
}

/// Router for preview endpoints
pub fn router() -> Router<crate::DaemonState> {
    Router::new().route("/api/p/{project}/beads/preview", get(preview_bead))
}

/// GET /api/p/:project/beads/preview — preview what a bead will take
async fn preview_bead(
    Path(project): Path<String>,
    State(state): State<crate::DaemonState>,
    Query(params): Query<PreviewRequest>,
) -> Result<Json<StitchPreview>, (StatusCode, String)> {
    crate::id_validators::validate_project_name(&project).map_err(crate::id_validators::rejection)?;
    let _project_path = resolve_project_path(&project, &state)?;

    let labels: Vec<String> = params
        .labels
        .as_ref()
        .map(|l| l.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    // Load historical Stitches from fleet.db
    let historical_stitches = load_historical_stitches(&project)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Get prediction (cost p50/p90, duration p50/p90, likely adapter+model)
    let prediction = predict_stitch(
        &params.title,
        params.description.as_deref(),
        &labels,
        historical_stitches,
        90,
    );

    // Match risk patterns from Fix Lineage library
    let risk_library = load_risk_library()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let risk_matches = risk_library.match_draft(
        &params.title,
        params.description.as_deref(),
        &labels,
    );

    // Check for file conflicts with currently-executing beads
    let file_conflicts = check_file_conflicts(&state).await;

    // Find similar Stitches for reference
    let similar_stitches_refs = find_similar_stitches_refs(
        &params.title,
        params.description.as_deref(),
        &labels,
        &state,
    ).await;

    let preview = StitchPreview {
        schema_version: "1.0.0".to_string(),
        prediction: prediction.map(|p| PredictionData {
            cost: PercentileEstimate {
                p50: p.cost.p50,
                p90: p.cost.p90,
                count: p.cost.count,
            },
            duration: PercentileEstimate {
                p50: p.duration.p50,
                p90: p.duration.p90,
                count: p.duration.count,
            },
            likely_adapter_model: p.likely_adapter_model,
            similar_count: p.similar_count,
            data_range: DateRange {
                start: p.data_range.start,
                end: p.data_range.end,
            },
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
    };

    Ok(Json(preview))
}

/// Load historical Stitches from fleet.db for prediction.
///
/// Derives cost from token usage, duration from timestamps, adapter/model
/// from participants JSON, and body/labels from messages and audit log.
pub fn load_historical_stitches(project: &str) -> Result<Vec<HistoricalStitch>, String> {
    use rusqlite::Connection;

    let db_path = fleet_db_path()?;

    if !db_path.exists() {
        return Ok(vec![]);
    }

    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open fleet.db: {}", e))?;

    // Query stitches with derived cost and duration.
    // The stitches table has: id, project, kind, title, created_by,
    //   created_at, last_activity_at, participants, attachments_path
    // Cost is derived from token counts (~$30/M tokens blended rate).
    // Duration is derived from created_at to last_activity_at.
    let mut stmt = conn
        .prepare(
            r#"
            SELECT
                s.id,
                s.title,
                s.created_at,
                s.last_activity_at,
                s.participants,
                (SELECT sm.content FROM stitch_messages sm
                 WHERE sm.stitch_id = s.id AND sm.role = 'user'
                 ORDER BY sm.ts ASC LIMIT 1) AS body,
                (SELECT COALESCE(SUM(sm.tokens), 0) FROM stitch_messages sm
                 WHERE sm.stitch_id = s.id) AS total_tokens
            FROM stitches s
            WHERE s.project = ?1
            ORDER BY s.last_activity_at DESC
            LIMIT 500
            "#,
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let stitches = stmt
        .query_map(rusqlite::params![project], |row| {
            let id: String = row.get(0)?;
            let title: String = row.get(1)?;
            let created_at: String = row.get(2)?;
            let last_activity_at: String = row.get(3)?;
            let participants_json: String = row.get(4).unwrap_or_else(|_| "[]".to_string());
            let body: Option<String> = row.get(5).unwrap_or(None);
            let total_tokens: i64 = row.get(6).unwrap_or(0);

            let created_dt = DateTime::parse_from_rfc3339(&created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            let last_activity_dt = DateTime::parse_from_rfc3339(&last_activity_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            let duration_seconds = (last_activity_dt - created_dt).num_seconds().max(0);

            let cost_usd = (total_tokens as f64) * 30.0 / 1_000_000.0;

            let adapter_model = extract_adapter_model(&participants_json);
            let labels = load_labels_for_stitch(&id, &conn);

            Ok(HistoricalStitch {
                id,
                title,
                body,
                labels,
                adapter_model,
                cost_usd,
                duration_seconds,
                closed_at: last_activity_dt,
            })
        })
        .map_err(|e| format!("Failed to query stitches: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to parse stitch rows: {}", e))?;

    Ok(stitches)
}

/// Extract adapter:model from participants JSON.
/// Participants is a JSON array like `[{"adapter":"claude","model":"opus"}]`
fn extract_adapter_model(participants_json: &str) -> Option<String> {
    let participants: Vec<serde_json::Value> =
        serde_json::from_str(participants_json).unwrap_or_default();

    for p in &participants {
        if let (Some(adapter), Some(model)) = (p.get("adapter"), p.get("model")) {
            if let (Some(a), Some(m)) = (adapter.as_str(), model.as_str()) {
                return Some(format!("{}:{}", a, m));
            }
        }
    }
    None
}

/// Load labels for a stitch from the audit log.
/// The actions table stores labels in args_json for StitchCreated events.
fn load_labels_for_stitch(stitch_id: &str, conn: &rusqlite::Connection) -> Vec<String> {
    let mut stmt = match conn.prepare(
        "SELECT args_json FROM actions WHERE stitch_id = ?1 AND kind = 'stitch_created' ORDER BY ts DESC LIMIT 1"
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let args_json: Option<String> = stmt
        .query_row(rusqlite::params![stitch_id], |row| row.get(0))
        .ok();

    args_json
        .and_then(|json| {
            let v: serde_json::Value = serde_json::from_str(&json).ok()?;
            v.get("labels")?
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|l| l.as_str().map(String::from))
                        .collect()
                })
        })
        .unwrap_or_default()
}

/// Load risk pattern library from ~/.hoop/risk_patterns.json or use defaults
pub fn load_risk_library() -> Result<FixLineageLibrary, String> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let risk_patterns_path = home.join(".hoop").join("risk_patterns.json");

    if risk_patterns_path.exists() {
        FixLineageLibrary::load_from_file(&risk_patterns_path)
            .map_err(|e| format!("Failed to load risk patterns: {}", e))
    } else {
        Ok(FixLineageLibrary::from_patterns(default_risk_patterns()))
    }
}

/// Check for file conflicts with currently-executing beads.
///
/// Looks at workers in the Executing state and extracts file paths from
/// their bead event history to detect potential overlaps.
pub async fn check_file_conflicts(state: &crate::DaemonState) -> Vec<FileConflict> {
    let workers = state.worker_registry.snapshot().await;

    let executing_beads: Vec<_> = workers
        .iter()
        .filter_map(|w| {
            if let crate::ws::WorkerDisplayState::Executing { bead, .. } = &w.state {
                Some(bead.clone())
            } else {
                None
            }
        })
        .collect();

    if executing_beads.is_empty() {
        return vec![];
    }

    // Collect bead titles while holding the lock, then drop it before any await
    let bead_titles: std::collections::HashMap<String, String> = {
        let beads = state.beads.read().unwrap();
        executing_beads
            .iter()
            .filter_map(|bead_id| {
                beads
                    .iter()
                    .find(|b| &b.id == bead_id)
                    .map(|b| (bead_id.clone(), b.title.clone()))
            })
            .collect()
    };

    let mut conflicts = Vec::new();

    for bead_id in &executing_beads {
        if let Some(title) = bead_titles.get(bead_id) {
            let bead_events = state.worker_registry.get_bead_events(bead_id).await;

            // Extract file paths from tool_call events
            let overlapping: Vec<String> = bead_events
                .iter()
                .filter_map(|e| {
                    if e.event_type == "tool_call" {
                        // Parse the raw JSON to extract file_path
                        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&e.raw) {
                            value.get("file_path")
                                .and_then(|v| v.as_str())
                                .map(String::from)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            if !overlapping.is_empty() {
                conflicts.push(FileConflict {
                    bead_id: bead_id.clone(),
                    title: title.clone(),
                    project: String::new(),
                    overlapping_files: overlapping,
                });
            }
        }
    }

    conflicts
}

/// Find similar Stitches for reference
async fn find_similar_stitches_refs(
    title: &str,
    body: Option<&str>,
    labels: &[String],
    state: &crate::DaemonState,
) -> Vec<SimilarStitchRef> {
    let beads = state.beads.read().unwrap();

    let historical: Vec<_> = beads
        .iter()
        .map(|b| {
            (
                b.id.clone(),
                b.title.clone(),
                None as Option<String>,
                vec![],
            )
        })
        .collect();

    let similar = find_similar_stitches(title, body, labels, historical, 0.3, 5);

    similar
        .into_iter()
        .map(|s| SimilarStitchRef {
            id: s.id,
            title: s.title,
            similarity: s.similarity.score,
        })
        .collect()
}

/// Get path to fleet.db
fn fleet_db_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    Ok(home.join(".hoop").join("fleet.db"))
}

/// Resolve project path from project name
fn resolve_project_path(
    project: &str,
    state: &crate::DaemonState,
) -> Result<PathBuf, (StatusCode, String)> {
    let projects = state.projects.read().unwrap();
    projects
        .iter()
        .find(|p| p.name == project)
        .map(|p| PathBuf::from(&p.path))
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("Project '{}' not found", project),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preview_request_parse() {
        let params = PreviewRequest {
            title: "Fix bug".to_string(),
            description: Some("Fix the crash".to_string()),
            labels: Some("urgent,bug".to_string()),
        };

        assert_eq!(params.title, "Fix bug");
        assert_eq!(params.description, Some("Fix the crash".to_string()));

        let labels: Vec<_> = params
            .labels
            .as_ref()
            .map(|l| l.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        assert_eq!(labels, vec!["urgent", "bug"]);
    }

    #[test]
    fn test_extract_adapter_model_valid() {
        let json = r#"[{"adapter":"claude","model":"opus"}]"#;
        assert_eq!(extract_adapter_model(json), Some("claude:opus".to_string()));
    }

    #[test]
    fn test_extract_adapter_model_multiple() {
        let json = r#"[{"adapter":"codex","model":"gpt4"},{"adapter":"claude","model":"opus"}]"#;
        assert_eq!(extract_adapter_model(json), Some("codex:gpt4".to_string()));
    }

    #[test]
    fn test_extract_adapter_model_empty() {
        assert_eq!(extract_adapter_model("[]"), None);
    }

    #[test]
    fn test_extract_adapter_model_invalid() {
        assert_eq!(extract_adapter_model("not json"), None);
    }

    #[test]
    fn test_extract_adapter_model_no_adapter_field() {
        let json = r#"[{"name":"worker-1"}]"#;
        assert_eq!(extract_adapter_model(json), None);
    }

    #[test]
    fn test_load_historical_stitches_no_db() {
        let result = load_historical_stitches("_nonexistent_project_test_");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
