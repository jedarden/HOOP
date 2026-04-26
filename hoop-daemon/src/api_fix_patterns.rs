//! Fix patterns API
//!
//! `POST   /api/fix-patterns` — create a new fix pattern
//! `GET    /api/fix-patterns` — list all fix patterns
//! `GET    /api/fix-patterns/:id` — get a single pattern by ID
//! `PUT    /api/fix-patterns/:id` — update an existing pattern
//! `DELETE /api/fix-patterns/:id` — delete a pattern
//! `POST   /api/fix-patterns/match` — find patterns matching a signature vector
//! `GET    /api/fix-patterns/search?q=` — search patterns by keywords

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::fix_patterns::{CreatePatternRequest, FixPatternService, UpdatePatternRequest};

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct PatternListResponse {
    pub patterns: Vec<PatternDetail>,
}

#[derive(Debug, Serialize)]
pub struct PatternDetail {
    pub id: String,
    pub name: String,
    pub signature_vector: Vec<f32>,
    pub keywords: String,
    pub recommended_fix_template_md: String,
    pub example_source_stitches: Vec<String>,
    pub created_at: String,
    pub applied_count: i64,
}

#[derive(Debug, Serialize)]
pub struct PatternCreatedResponse {
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct PatternMatchResponse {
    pub matches: Vec<PatternMatchDetail>,
}

#[derive(Debug, Serialize)]
pub struct PatternMatchDetail {
    pub pattern: PatternDetail,
    pub similarity: f32,
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct MatchRequest {
    pub signature_vector: Vec<f32>,
    #[serde(default = "default_threshold")]
    pub threshold: f32,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_threshold() -> f32 {
    0.5
}

fn default_limit() -> usize {
    10
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<crate::DaemonState> {
    Router::new()
        .route("/api/fix-patterns", get(list_patterns).post(create_pattern))
        .route(
            "/api/fix-patterns/:id",
            get(get_pattern).put(update_pattern).delete(delete_pattern),
        )
        .route("/api/fix-patterns/match", post(match_patterns))
        .route("/api/fix-patterns/search", get(search_patterns))
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn create_pattern(
    Json(req): Json<CreatePatternRequest>,
) -> Result<Json<PatternCreatedResponse>, (StatusCode, String)> {
    let id = FixPatternService::create(&req).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("create failed: {e}"),
        )
    })?;

    Ok(Json(PatternCreatedResponse { id }))
}

async fn list_patterns() -> Result<Json<PatternListResponse>, (StatusCode, String)> {
    let patterns = FixPatternService::list().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("list failed: {e}"),
        )
    })?;

    let details = patterns
        .into_iter()
        .map(|p| PatternDetail {
            id: p.id,
            name: p.name,
            signature_vector: p.signature_vector,
            keywords: p.keywords,
            recommended_fix_template_md: p.recommended_fix_template_md,
            example_source_stitches: p.example_source_stitches,
            created_at: p.created_at,
            applied_count: p.applied_count,
        })
        .collect();

    Ok(Json(PatternListResponse { patterns: details }))
}

async fn get_pattern(Path(id): Path<String>) -> Result<Json<PatternDetail>, (StatusCode, String)> {
    let pattern = FixPatternService::get(&id)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("get failed: {e}"),
            )
        })?
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Pattern '{}' not found", id)))?;

    Ok(Json(PatternDetail {
        id: pattern.id,
        name: pattern.name,
        signature_vector: pattern.signature_vector,
        keywords: pattern.keywords,
        recommended_fix_template_md: pattern.recommended_fix_template_md,
        example_source_stitches: pattern.example_source_stitches,
        created_at: pattern.created_at,
        applied_count: pattern.applied_count,
    }))
}

async fn update_pattern(
    Path(id): Path<String>,
    Json(mut req): Json<UpdatePatternRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    req.id = id;
    FixPatternService::update(&req).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("update failed: {e}"),
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}

async fn delete_pattern(Path(id): Path<String>) -> Result<StatusCode, (StatusCode, String)> {
    FixPatternService::delete(&id).map_err(|e| {
        if e.to_string().contains("not found") {
            (StatusCode::NOT_FOUND, format!("Pattern '{}' not found", id))
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("delete failed: {e}"),
            )
        }
    })?;

    Ok(StatusCode::NO_CONTENT)
}

async fn match_patterns(
    Json(req): Json<MatchRequest>,
) -> Result<Json<PatternMatchResponse>, (StatusCode, String)> {
    let matches =
        FixPatternService::match_by_signature(&req.signature_vector, req.threshold, req.limit)
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("match failed: {e}"),
                )
            })?;

    let details = matches
        .into_iter()
        .map(|m| PatternMatchDetail {
            pattern: PatternDetail {
                id: m.pattern.id,
                name: m.pattern.name,
                signature_vector: m.pattern.signature_vector,
                keywords: m.pattern.keywords,
                recommended_fix_template_md: m.pattern.recommended_fix_template_md,
                example_source_stitches: m.pattern.example_source_stitches,
                created_at: m.pattern.created_at,
                applied_count: m.pattern.applied_count,
            },
            similarity: m.similarity,
        })
        .collect();

    Ok(Json(PatternMatchResponse { matches: details }))
}

async fn search_patterns(
    Query(query): Query<SearchQuery>,
) -> Result<Json<PatternListResponse>, (StatusCode, String)> {
    let patterns = FixPatternService::search_by_keywords(&query.q).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("search failed: {e}"),
        )
    })?;

    let details = patterns
        .into_iter()
        .map(|p| PatternDetail {
            id: p.id,
            name: p.name,
            signature_vector: p.signature_vector,
            keywords: p.keywords,
            recommended_fix_template_md: p.recommended_fix_template_md,
            example_source_stitches: p.example_source_stitches,
            created_at: p.created_at,
            applied_count: p.applied_count,
        })
        .collect();

    Ok(Json(PatternListResponse { patterns: details }))
}
