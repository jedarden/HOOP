//! Cost-per-stitch analytics API
//!
//! Provides cost trends for stitches over time windows (30/90/180 days).
//! Data is grouped by adapter and project for comparison.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json, Router,
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::fleet;

/// Cost trend data point
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CostTrendPoint {
    /// Date in YYYY-MM-DD format
    pub date: String,
    /// Total cost in USD for this period
    pub cost_usd: f64,
    /// Total tokens for this period
    pub total_tokens: i64,
    /// Number of stitches
    pub stitch_count: i64,
}

/// Cost trend response grouped by adapter
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AdapterCostTrend {
    /// Adapter name (e.g., "claude", "codex", "gemini")
    pub adapter: String,
    /// Model name (e.g., "sonnet", "opus", "haiku")
    pub model: String,
    /// Daily cost data points
    pub data_points: Vec<CostTrendPoint>,
    /// Total cost over the period
    pub total_cost_usd: f64,
    /// Total tokens over the period
    pub total_tokens: i64,
}

/// Cost trend response grouped by project
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ProjectCostTrend {
    /// Project name
    pub project: String,
    /// Daily cost data points
    pub data_points: Vec<CostTrendPoint>,
    /// Total cost over the period
    pub total_cost_usd: f64,
    /// Total tokens over the period
    pub total_tokens: i64,
}

/// Combined cost trends response
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CostTrendsResponse {
    /// Trend window in days
    pub window_days: i64,
    /// Cost per stitch (single value)
    pub cost_per_stitch_usd: f64,
    /// Grouped by adapter/model
    pub by_adapter: Vec<AdapterCostTrend>,
    /// Grouped by project
    pub by_project: Vec<ProjectCostTrend>,
}

/// Query parameters for cost trends
#[derive(Debug, Deserialize)]
pub struct CostTrendsQuery {
    /// Time window in days (default: 30)
    #[serde(default = "default_window_days")]
    window_days: i64,
}

fn default_window_days() -> i64 {
    30
}

pub fn router() -> Router<crate::DaemonState> {
    Router::new()
        .route("/api/cost/stitch-trends", axum::routing::get(get_stitch_trends))
        .route("/api/stitches/{id}/cost", axum::routing::get(get_stitch_cost))
}

/// Get cost trends for all stitches over a time window
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/api/cost/stitch-trends",
    tag = "cost",
    params(
        ("window_days" = Option<i64>, Query, description = "Time window in days (default: 30, max: 180)"),
    ),
    responses(
        (status = 200, description = "Cost trends data", body = CostTrendsResponse),
    )
))]
async fn get_stitch_trends(
    Query(params): Query<CostTrendsQuery>,
    State(_state): State<crate::DaemonState>,
) -> Result<Json<CostTrendsResponse>, (StatusCode, String)> {
    let window_days = params.window_days.min(180).max(1);

    let db_path = fleet::db_path();
    let conn = rusqlite::Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to open fleet.db: {}", e),
        )
    })?;

    let start_date = Utc::now() - Duration::days(window_days);

    // Query cost data grouped by adapter/model and date
    let mut by_adapter_stmt = conn
        .prepare(
            r#"
            SELECT
                created_by_adapter,
                created_by_model,
                DATE(created_at) as stitch_date,
                COALESCE(SUM(total_cost_usd), 0.0) as cost_usd,
                COALESCE(SUM(total_tokens), 0) as total_tokens,
                COUNT(*) as stitch_count
            FROM stitches
            WHERE created_at >= ?1
                AND created_by_adapter IS NOT NULL
            GROUP BY created_by_adapter, created_by_model, DATE(created_at)
            ORDER BY stitch_date DESC
            "#,
        )
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to prepare adapter query: {}", e),
            )
        })?;

    let mut adapter_map: HashMap<(String, String), Vec<CostTrendPoint>> = HashMap::new();
    let mut total_cost_all: f64 = 0.0;
    let mut total_tokens_all: i64 = 0;
    let mut total_stitches: i64 = 0;

    let rows = by_adapter_stmt
        .query_map([start_date.format("%Y-%m-%dT%H:%M:%S").to_string()], |row| {
            let adapter: String = row.get(0)?;
            let model: String = row.get(1)?;
            let date_str: String = row.get(2)?;
            let cost_usd: f64 = row.get(3)?;
            let total_tokens: i64 = row.get(4)?;
            let stitch_count: i64 = row.get(5)?;

            total_cost_all += cost_usd;
            total_tokens_all += total_tokens;
            total_stitches += stitch_count;

            Ok((
                (adapter, model),
                CostTrendPoint {
                    date: date_str,
                    cost_usd,
                    total_tokens,
                    stitch_count,
                },
            ))
        })
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to query adapter trends: {}", e),
            )
        })?;

    for row in rows {
        let ((adapter, model), point) = row.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read adapter row: {}", e),
            )
        })?;
        adapter_map
            .entry((adapter, model))
            .or_default()
            .push(point);
    }

    let by_adapter: Vec<AdapterCostTrend> = adapter_map
        .into_iter()
        .map(|((adapter, model), mut points)| {
            points.sort_by(|a, b| b.date.cmp(&a.date));
            let total_cost_usd = points.iter().map(|p| p.cost_usd).sum();
            let total_tokens = points.iter().map(|p| p.total_tokens).sum();
            AdapterCostTrend {
                adapter,
                model: if model.is_empty() {
                    "unknown".to_string()
                } else {
                    model
                },
                data_points: points,
                total_cost_usd,
                total_tokens,
            }
        })
        .collect();

    // Query cost data grouped by project and date
    let mut by_project_stmt = conn
        .prepare(
            r#"
            SELECT
                project,
                DATE(created_at) as stitch_date,
                COALESCE(SUM(total_cost_usd), 0.0) as cost_usd,
                COALESCE(SUM(total_tokens), 0) as total_tokens,
                COUNT(*) as stitch_count
            FROM stitches
            WHERE created_at >= ?1
            GROUP BY project, DATE(created_at)
            ORDER BY project, stitch_date DESC
            "#,
        )
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to prepare project query: {}", e),
            )
        })?;

    let mut project_map: HashMap<String, Vec<CostTrendPoint>> = HashMap::new();

    let rows = by_project_stmt
        .query_map([start_date.format("%Y-%m-%dT%H:%M:%S").to_string()], |row| {
            let project: String = row.get(0)?;
            let date_str: String = row.get(1)?;
            let cost_usd: f64 = row.get(2)?;
            let total_tokens: i64 = row.get(3)?;
            let stitch_count: i64 = row.get(4)?;

            Ok((
                project,
                CostTrendPoint {
                    date: date_str,
                    cost_usd,
                    total_tokens,
                    stitch_count,
                },
            ))
        })
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to query project trends: {}", e),
            )
        })?;

    for row in rows {
        let (project, point) = row.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read project row: {}", e),
            )
        })?;
        project_map.entry(project).or_default().push(point);
    }

    let by_project: Vec<ProjectCostTrend> = project_map
        .into_iter()
        .map(|(project, mut points)| {
            points.sort_by(|a, b| b.date.cmp(&a.date));
            let total_cost_usd = points.iter().map(|p| p.cost_usd).sum();
            let total_tokens = points.iter().map(|p| p.total_tokens).sum();
            ProjectCostTrend {
                project,
                data_points: points,
                total_cost_usd,
                total_tokens,
            }
        })
        .collect();

    // Calculate cost per stitch
    let cost_per_stitch_usd = if total_stitches > 0 {
        total_cost_all / total_stitches as f64
    } else {
        0.0
    };

    Ok(Json(CostTrendsResponse {
        window_days,
        cost_per_stitch_usd,
        by_adapter,
        by_project,
    }))
}

/// Stitch cost response
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct StitchCostResponse {
    /// Stitch ID
    pub stitch_id: String,
    /// Total cost in USD
    pub total_cost_usd: f64,
    /// Total tokens
    pub total_tokens: i64,
    /// Number of messages
    pub message_count: i64,
    /// Cost per message
    pub cost_per_message_usd: f64,
}

/// Get cost breakdown for a specific stitch
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/api/stitches/{id}/cost",
    tag = "cost",
    params(
        ("id" = String, Path, description = "Stitch ID"),
    ),
    responses(
        (status = 200, description = "Stitch cost breakdown", body = StitchCostResponse),
        (status = 404, description = "Stitch not found"),
    )
))]
async fn get_stitch_cost(
    Path(stitch_id): Path<String>,
    State(_state): State<crate::DaemonState>,
) -> Result<Json<StitchCostResponse>, (StatusCode, String)> {
    crate::id_validators::validate_stitch_id(&stitch_id)
        .map_err(crate::id_validators::rejection)?;

    let db_path = fleet::db_path();
    let conn = rusqlite::Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to open fleet.db: {}", e),
        )
    })?;

    // Get stitch cost data
    let (total_cost_usd, total_tokens): (f64, i64) = conn
        .query_row(
            "SELECT COALESCE(total_cost_usd, 0.0), COALESCE(total_tokens, 0) FROM stitches WHERE id = ?1",
            [&stitch_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| {
            if e == rusqlite::Error::QueryReturnedNoRows {
                (StatusCode::NOT_FOUND, format!("Stitch '{}' not found", stitch_id))
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("DB error: {}", e),
                )
            }
        })?;

    // Get message count
    let message_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM stitch_messages WHERE stitch_id = ?1",
            [&stitch_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let cost_per_message_usd = if message_count > 0 {
        total_cost_usd / message_count as f64
    } else {
        0.0
    };

    Ok(Json(StitchCostResponse {
        stitch_id,
        total_cost_usd,
        total_tokens,
        message_count,
        cost_per_message_usd,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_window_days() {
        assert_eq!(default_window_days(), 30);
    }
}
