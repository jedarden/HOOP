//! Risk patterns API
//!
//! `GET    /api/risk-patterns` — list all risk patterns
//! `GET    /api/risk-patterns/:id` — get a single pattern by ID
//! `POST   /api/risk-patterns/match` — match patterns against a title/body/labels
//! `GET    /api/risk-patterns/export` — export all patterns as JSON
//! `POST   /api/risk-patterns/import` — import patterns from JSON

use axum::{
    extract::{ConnectInfo, Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

use crate::atomic_write;
use crate::risk_patterns::{default_risk_patterns, FixLineageLibrary, RiskMatch, RiskPattern};
use crate::DaemonState;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PatternListResponse {
    pub patterns: Vec<PatternDetail>,
}

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PatternDetail {
    pub id: String,
    pub name: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub label_keywords: Vec<String>,
    pub fix_recommendation: String,
    pub severity: String,
    pub category: String,
}

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PatternMatchResponse {
    pub matches: Vec<PatternMatchDetail>,
}

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PatternMatchDetail {
    pub pattern: PatternDetail,
    pub confidence: f64,
    pub matched_keywords: Vec<String>,
    pub matched_labels: Vec<String>,
}

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PatternsExportResponse {
    pub patterns: Vec<PatternExport>,
    pub exported_at: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PatternExport {
    pub id: String,
    pub name: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub label_keywords: Vec<String>,
    pub fix_recommendation: String,
    pub severity: String,
    pub category: String,
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct MatchRequest {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub labels: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PatternsImportRequest {
    pub patterns: Vec<PatternExport>,
}

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PatternsImportResponse {
    pub imported: usize,
    pub skipped: usize,
    pub ids: Vec<String>,
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<crate::DaemonState> {
    Router::new()
        .route("/api/risk-patterns", get(list_patterns))
        .route("/api/risk-patterns/:id", get(get_pattern))
        .route("/api/risk-patterns/match", post(match_patterns))
        .route("/api/risk-patterns/export", get(export_patterns))
        .route("/api/risk-patterns/import", post(import_patterns))
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/api/risk-patterns",
    tag = "risk_patterns",
    responses(
        (status = 200, description = "List of all risk patterns", body = PatternListResponse)
    )
))]
async fn list_patterns() -> Result<Json<PatternListResponse>, (StatusCode, String)> {
    let library = load_risk_library().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("load failed: {e}"),
        )
    })?;

    let details = library
        .patterns()
        .iter()
        .map(|p| PatternDetail {
            id: p.id.clone(),
            name: p.name.clone(),
            description: p.description.clone(),
            keywords: p.keywords.clone(),
            label_keywords: p.label_keywords.clone(),
            fix_recommendation: p.fix_recommendation.clone(),
            severity: format!("{:?}", p.severity).to_lowercase(),
            category: format!("{:?}", p.category),
        })
        .collect();

    Ok(Json(PatternListResponse { patterns: details }))
}

#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/api/risk-patterns/{id}",
    tag = "risk_patterns",
    params(
        ("id" = String, Path, description = "Pattern ID")
    ),
    responses(
        (status = 200, description = "Pattern details", body = PatternDetail),
        (status = 404, description = "Pattern not found")
    )
))]
async fn get_pattern(Path(id): Path<String>) -> Result<Json<PatternDetail>, (StatusCode, String)> {
    let library = load_risk_library().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("load failed: {e}"),
        )
    })?;

    let pattern = library
        .patterns()
        .iter()
        .find(|p| p.id == id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Pattern '{}' not found", id)))?;

    Ok(Json(PatternDetail {
        id: pattern.id.clone(),
        name: pattern.name.clone(),
        description: pattern.description.clone(),
        keywords: pattern.keywords.clone(),
        label_keywords: pattern.label_keywords.clone(),
        fix_recommendation: pattern.fix_recommendation.clone(),
        severity: format!("{:?}", pattern.severity).to_lowercase(),
        category: format!("{:?}", pattern.category),
    }))
}

#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/api/risk-patterns/match",
    tag = "risk_patterns",
    request_body = MatchRequest,
    responses(
        (status = 200, description = "Matching patterns with confidence scores", body = PatternMatchResponse)
    )
))]
async fn match_patterns(
    Json(req): Json<MatchRequest>,
) -> Result<Json<PatternMatchResponse>, (StatusCode, String)> {
    let library = load_risk_library().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("load failed: {e}"),
        )
    })?;

    let matches = library.match_draft(&req.title, req.description.as_deref(), &req.labels);

    let details = matches
        .into_iter()
        .map(|m| PatternMatchDetail {
            pattern: PatternDetail {
                id: m.pattern.id.clone(),
                name: m.pattern.name.clone(),
                description: m.pattern.description.clone(),
                keywords: m.pattern.keywords.clone(),
                label_keywords: m.pattern.label_keywords.clone(),
                fix_recommendation: m.pattern.fix_recommendation.clone(),
                severity: format!("{:?}", m.pattern.severity).to_lowercase(),
                category: format!("{:?}", m.pattern.category),
            },
            confidence: m.confidence,
            matched_keywords: m.matched_keywords,
            matched_labels: m.matched_labels,
        })
        .collect();

    Ok(Json(PatternMatchResponse { matches: details }))
}

#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/api/risk-patterns/export",
    tag = "risk_patterns",
    responses(
        (status = 200, description = "All patterns exported as JSON", body = PatternsExportResponse)
    )
))]
async fn export_patterns() -> Result<Json<PatternsExportResponse>, (StatusCode, String)> {
    let library = load_risk_library().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("export failed: {e}"),
        )
    })?;

    let exports = library
        .patterns()
        .iter()
        .map(|p| PatternExport {
            id: p.id.clone(),
            name: p.name.clone(),
            description: p.description.clone(),
            keywords: p.keywords.clone(),
            label_keywords: p.label_keywords.clone(),
            fix_recommendation: p.fix_recommendation.clone(),
            severity: format!("{:?}", p.severity).to_lowercase(),
            category: format!("{:?}", p.category),
        })
        .collect();

    Ok(Json(PatternsExportResponse {
        patterns: exports,
        exported_at: chrono::Utc::now().to_rfc3339(),
        version: "1.0.0".to_string(),
    }))
}

#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/api/risk-patterns/import",
    tag = "risk_patterns",
    request_body = PatternsImportRequest,
    responses(
        (status = 200, description = "Import results with imported/skipped counts", body = PatternsImportResponse)
    )
))]
async fn import_patterns(
    Json(req): Json<PatternsImportRequest>,
) -> Result<Json<PatternsImportResponse>, (StatusCode, String)> {
    use crate::risk_patterns::{RiskCategory, RiskSeverity};

    let risk_patterns_path = risk_patterns_path()?;

    // Load existing patterns or create empty library
    let mut library = if risk_patterns_path.exists() {
        FixLineageLibrary::load_from_file(&risk_patterns_path).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("load failed: {e}"),
            )
        })?
    } else {
        FixLineageLibrary::new()
    };

    let mut imported = 0;
    let mut skipped = 0;
    let mut ids = Vec::new();

    let existing_ids: std::collections::HashSet<_> =
        library.patterns().iter().map(|p| &p.id).collect();

    for pattern in &req.patterns {
        if existing_ids.contains(&pattern.id) {
            skipped += 1;
            continue;
        }

        // Parse severity
        let severity_val = match pattern.severity.to_lowercase().as_str() {
            "low" => RiskSeverity::Low,
            "medium" => RiskSeverity::Medium,
            "high" => RiskSeverity::High,
            "critical" => RiskSeverity::Critical,
            _ => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!("Invalid severity '{}'", pattern.severity),
                ))
            }
        };

        // Parse category
        let category_val = match pattern.category.to_lowercase().as_str() {
            "performance" => RiskCategory::Performance,
            "correctness" => RiskCategory::Correctness,
            "security" => RiskCategory::Security,
            "integration" => RiskCategory::Integration,
            "code_quality" => RiskCategory::CodeQuality,
            "infrastructure" => RiskCategory::Infrastructure,
            _ => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!("Invalid category '{}'", pattern.category),
                ))
            }
        };

        let risk_pattern = RiskPattern {
            id: pattern.id.clone(),
            name: pattern.name.clone(),
            description: pattern.description.clone(),
            keywords: pattern.keywords.clone(),
            label_keywords: pattern.label_keywords.clone(),
            fix_recommendation: pattern.fix_recommendation.clone(),
            severity: severity_val,
            category: category_val,
        };

        library.add_pattern(risk_pattern);
        imported += 1;
        ids.push(pattern.id.clone());
    }

    // Save to file
    if imported > 0 {
        if let Some(parent) = risk_patterns_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("create dir failed: {e}"),
                )
            })?;
        }

        let patterns_json = serde_json::to_string_pretty(library.patterns()).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("serialize failed: {e}"),
            )
        })?;

        atomic_write::atomic_write_file_str(&risk_patterns_path, &patterns_json).map_err(
            |e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("atomic write failed: {e}"),
                )
            },
        )?;
    }

    Ok(Json(PatternsImportResponse {
        imported,
        skipped,
        ids,
    }))
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Load risk pattern library from ~/.hoop/risk_patterns.json or use defaults
fn load_risk_library() -> Result<FixLineageLibrary, String> {
    let path = risk_patterns_path()?;

    if path.exists() {
        FixLineageLibrary::load_from_file(&path)
            .map_err(|e| format!("Failed to load risk patterns: {}", e))
    } else {
        Ok(FixLineageLibrary::from_patterns(default_risk_patterns()))
    }
}

/// Get the path to the risk patterns file
fn risk_patterns_path() -> Result<PathBuf, String> {
    let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.push(".hoop");
    home.push("risk_patterns.json");
    Ok(home)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_detail_from_risk_pattern() {
        let pattern = crate::risk_patterns::RiskPattern {
            id: "test_pattern".to_string(),
            name: "Test Pattern".to_string(),
            description: "Test description".to_string(),
            keywords: vec!["test".to_string()],
            label_keywords: vec![],
            fix_recommendation: "Fix it".to_string(),
            severity: crate::risk_patterns::RiskSeverity::High,
            category: crate::risk_patterns::RiskCategory::CodeQuality,
        };

        let detail = PatternDetail {
            id: pattern.id.clone(),
            name: pattern.name.clone(),
            description: pattern.description.clone(),
            keywords: pattern.keywords.clone(),
            label_keywords: pattern.label_keywords.clone(),
            fix_recommendation: pattern.fix_recommendation.clone(),
            severity: "high".to_string(),
            category: "CodeQuality".to_string(),
        };

        assert_eq!(detail.id, "test_pattern");
        assert_eq!(detail.severity, "high");
        assert_eq!(detail.category, "CodeQuality");
    }
}
