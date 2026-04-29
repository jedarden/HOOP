//! Fix patterns API
//!
//! `POST   /api/fix-patterns` — create a new fix pattern
//! `GET    /api/fix-patterns` — list all fix patterns
//! `GET    /api/fix-patterns/:id` — get a single pattern by ID
//! `PUT    /api/fix-patterns/:id` — update an existing pattern
//! `DELETE /api/fix-patterns/:id` — delete a pattern
//! `POST   /api/fix-patterns/match` — find patterns matching a signature vector
//! `GET    /api/fix-patterns/search?q=` — search patterns by keywords
//! `GET    /api/fix-patterns/export` — export all patterns as JSON
//! `POST   /api/fix-patterns/import` — import patterns from JSON

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

use crate::fix_patterns::{CreatePatternRequest, FixPatternService, UpdatePatternRequest};

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
    pub signature_vector: Vec<f32>,
    pub keywords: String,
    pub recommended_fix_template_md: String,
    pub example_source_stitches: Vec<String>,
    pub created_at: String,
    pub applied_count: i64,
}

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PatternCreatedResponse {
    pub id: String,
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
    pub similarity: f32,
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
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
    pub signature_vector: Vec<f32>,
    pub keywords: String,
    pub recommended_fix_template_md: String,
    pub example_source_stitches: Vec<String>,
    pub created_at: String,
    pub applied_count: i64,
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
        .route("/api/fix-patterns", get(list_patterns).post(create_pattern))
        .route(
            "/api/fix-patterns/:id",
            get(get_pattern).put(update_pattern).delete(delete_pattern),
        )
        .route("/api/fix-patterns/match", post(match_patterns))
        .route("/api/fix-patterns/search", get(search_patterns))
        .route("/api/fix-patterns/export", get(export_patterns))
        .route("/api/fix-patterns/import", post(import_patterns))
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/api/fix-patterns",
    tag = "fix_patterns",
    request_body = CreatePatternRequest,
    responses(
        (status = 200, description = "Pattern created successfully", body = PatternCreatedResponse)
    )
))]
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

#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/api/fix-patterns",
    tag = "fix_patterns",
    responses(
        (status = 200, description = "List of all fix patterns", body = PatternListResponse)
    )
))]
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

#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/api/fix-patterns/{id}",
    tag = "fix_patterns",
    params(
        ("id" = String, Path, description = "Pattern ID")
    ),
    responses(
        (status = 200, description = "Pattern details", body = PatternDetail),
        (status = 404, description = "Pattern not found")
    )
))]
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

#[cfg_attr(feature = "openapi", utoipa::path(
    put,
    path = "/api/fix-patterns/{id}",
    tag = "fix_patterns",
    params(
        ("id" = String, Path, description = "Pattern ID")
    ),
    request_body = UpdatePatternRequest,
    responses(
        (status = 204, description = "Pattern updated successfully"),
        (status = 404, description = "Pattern not found")
    )
))]
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

#[cfg_attr(feature = "openapi", utoipa::path(
    delete,
    path = "/api/fix-patterns/{id}",
    tag = "fix_patterns",
    params(
        ("id" = String, Path, description = "Pattern ID")
    ),
    responses(
        (status = 204, description = "Pattern deleted successfully"),
        (status = 404, description = "Pattern not found")
    )
))]
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

#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/api/fix-patterns/match",
    tag = "fix_patterns",
    request_body = MatchRequest,
    responses(
        (status = 200, description = "Matching patterns with similarity scores", body = PatternMatchResponse)
    )
))]
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

#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/api/fix-patterns/search",
    tag = "fix_patterns",
    params(
        ("q" = String, Query, description = "Search query for keywords")
    ),
    responses(
        (status = 200, description = "Patterns matching the search query", body = PatternListResponse)
    )
))]
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

#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/api/fix-patterns/export",
    tag = "fix_patterns",
    responses(
        (status = 200, description = "All patterns exported as JSON", body = PatternsExportResponse)
    )
))]
async fn export_patterns() -> Result<Json<PatternsExportResponse>, (StatusCode, String)> {
    let patterns = FixPatternService::list().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("export failed: {e}"),
        )
    })?;

    let exports = patterns
        .into_iter()
        .map(|p| PatternExport {
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

    Ok(Json(PatternsExportResponse {
        patterns: exports,
        exported_at: chrono::Utc::now().to_rfc3339(),
        version: "1.0.0".to_string(),
    }))
}

#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/api/fix-patterns/import",
    tag = "fix_patterns",
    request_body = PatternsImportRequest,
    responses(
        (status = 200, description = "Import results with imported/skipped counts", body = PatternsImportResponse)
    )
))]
async fn import_patterns(
    Json(req): Json<PatternsImportRequest>,
) -> Result<Json<PatternsImportResponse>, (StatusCode, String)> {
    let mut imported = 0;
    let mut skipped = 0;
    let mut ids = Vec::new();

    for pattern in &req.patterns {
        // Check if pattern already exists by ID
        match FixPatternService::get(&pattern.id) {
            Ok(Some(_)) => {
                skipped += 1;
                continue;
            }
            Ok(None) => {}
            Err(e) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("import check failed: {e}"),
                ))
            }
        }

        // Create new pattern with preserved ID
        let create_req = CreatePatternRequest {
            name: pattern.name.clone(),
            signature_vector: pattern.signature_vector.clone(),
            keywords: pattern.keywords.clone(),
            recommended_fix_template_md: pattern.recommended_fix_template_md.clone(),
            example_source_stitches: pattern.example_source_stitches.clone(),
        };

        // Note: create() generates a new ID, so we need to insert directly
        let db_path = crate::fleet::db_path();
        let mut conn = match rusqlite::Connection::open(&db_path) {
            Ok(c) => c,
            Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("db open failed: {e}"))),
        };

        let signature_json = match serde_json::to_string(&pattern.signature_vector) {
            Ok(j) => j,
            Err(e) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("json serialize failed: {e}"),
                ))
            }
        };

        let examples_json = match serde_json::to_string(&pattern.example_source_stitches) {
            Ok(j) => j,
            Err(e) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("json serialize failed: {e}"),
                ))
            }
        };

        match conn.execute(
            r#"
            INSERT INTO fix_patterns (
                id, name, signature_vector_json, keywords,
                recommended_fix_template_md, example_source_stitches_json,
                created_at, applied_count
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            rusqlite::params![
                &pattern.id,
                &pattern.name,
                &signature_json,
                &pattern.keywords,
                &pattern.recommended_fix_template_md,
                &examples_json,
                &pattern.created_at,
                pattern.applied_count,
            ],
        ) {
            Ok(_) => {
                imported += 1;
                ids.push(pattern.id.clone());
            }
            Err(e) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("insert failed: {e}"),
                ))
            }
        }
    }

    Ok(Json(PatternsImportResponse {
        imported,
        skipped,
        ids,
    }))
}
