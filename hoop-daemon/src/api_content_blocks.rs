//! Content blocks API
//!
//! REST API for managing content blocks associated with stitches.

use crate::content_blocks::{delete_content_block, get_content_blocks, reorder_content_blocks, update_content_block, ContentBlock, ContentBlockCreate, ContentBlockUpdate};
use crate::content_blocks::create_content_block as db_create_content_block;
use crate::DaemonState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use rusqlite::Connection;
use tracing::error;

/// List content blocks for a stitch
#[utoipa::path(
    get,
    path = "/api/stitches/{stitch_id}/content-blocks",
    params(
        ("stitch_id" = String, Path, description = "Stitch ID")
    ),
    responses(
        (status = 200, description = "Content blocks retrieved", body = Vec<ContentBlock>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Content Blocks"
)]
pub async fn list_content_blocks(
    State(_state): State<DaemonState>,
    Path(stitch_id): Path<String>,
) -> Result<Json<Vec<ContentBlock>>, (StatusCode, String)> {
    let db_path = crate::fleet::db_path();
    let conn = Connection::open(&db_path).map_err(|e| {
        error!("Failed to open database: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    let blocks = get_content_blocks(&mut conn.into(), &stitch_id).map_err(|e| {
        error!("Failed to get content blocks for stitch {}: {}", stitch_id, e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    Ok(Json(blocks))
}

/// Create a new content block
#[utoipa::path(
    post,
    path = "/api/stitches/{stitch_id}/content-blocks",
    params(
        ("stitch_id" = String, Path, description = "Stitch ID")
    ),
    request_body = ContentBlockCreate,
    responses(
        (status = 201, description = "Content block created", body = ContentBlock),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Content Blocks"
)]
pub async fn create_content_block(
    State(_state): State<DaemonState>,
    Path(stitch_id): Path<String>,
    Json(req): Json<ContentBlockCreate>,
) -> Result<Json<ContentBlock>, (StatusCode, String)> {
    use uuid::Uuid;
    use chrono::Utc;

    let id = format!("cb-{}", Uuid::new_v4());
    let block = ContentBlock {
        id: id.clone(),
        stitch_id: stitch_id.clone(),
        block_type: req.block_type,
        content: req.content,
        metadata: req.metadata,
        block_order: req.block_order.unwrap_or(0),
        created_at: Utc::now().to_rfc3339(),
    };

    let db_path = crate::fleet::db_path();
    let conn = Connection::open(&db_path).map_err(|e| {
        error!("Failed to open database: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    db_create_content_block(&mut conn.into(), &block).map_err(|e| {
        error!("Failed to create content block for stitch {}: {}", stitch_id, e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    Ok(Json(block))
}

/// Update a content block
#[utoipa::path(
    put,
    path = "/api/stitches/{stitch_id}/content-blocks/{block_id}",
    params(
        ("stitch_id" = String, Path, description = "Stitch ID"),
        ("block_id" = String, Path, description = "Content block ID")
    ),
    request_body = ContentBlockUpdate,
    responses(
        (status = 200, description = "Content block updated", body = ContentBlock),
        (status = 404, description = "Content block not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Content Blocks"
)]
pub async fn update_content_block_endpoint(
    State(_state): State<DaemonState>,
    Path((_stitch_id, block_id)): Path<(String, String)>,
    Json(req): Json<ContentBlockUpdate>,
) -> Result<Json<ContentBlock>, (StatusCode, String)> {
    let db_path = crate::fleet::db_path();
    let conn = Connection::open(&db_path).map_err(|e| {
        error!("Failed to open database: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    let updated = update_content_block(&mut conn.into(), &block_id, req).map_err(|e| {
        error!("Failed to update content block {}: {}", block_id, e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    Ok(Json(updated))
}

/// Delete a content block
#[utoipa::path(
    delete,
    path = "/api/stitches/{stitch_id}/content-blocks/{block_id}",
    params(
        ("stitch_id" = String, Path, description = "Stitch ID"),
        ("block_id" = String, Path, description = "Content block ID")
    ),
    responses(
        (status = 204, description = "Content block deleted"),
        (status = 404, description = "Content block not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Content Blocks"
)]
pub async fn delete_content_block_endpoint(
    State(_state): State<DaemonState>,
    Path((_stitch_id, block_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, String)> {
    let db_path = crate::fleet::db_path();
    let conn = Connection::open(&db_path).map_err(|e| {
        error!("Failed to open database: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    delete_content_block(&mut conn.into(), &block_id).map_err(|e| {
        error!("Failed to delete content block {}: {}", block_id, e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Reorder content blocks for a stitch
#[utoipa::path(
    post,
    path = "/api/stitches/{stitch_id}/content-blocks/reorder",
    params(
        ("stitch_id" = String, Path, description = "Stitch ID")
    ),
    request_body = Vec<String>,
    responses(
        (status = 204, description = "Content blocks reordered"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Content Blocks"
)]
pub async fn reorder_content_blocks_endpoint(
    State(_state): State<DaemonState>,
    Path(stitch_id): Path<String>,
    Json(ordering): Json<Vec<String>>,
) -> Result<StatusCode, (StatusCode, String)> {
    let db_path = crate::fleet::db_path();
    let conn = Connection::open(&db_path).map_err(|e| {
        error!("Failed to open database: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    reorder_content_blocks(&mut conn.into(), &stitch_id, &ordering).map_err(|e| {
        error!("Failed to reorder content blocks for stitch {}: {}", stitch_id, e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Build the router for content block endpoints
pub fn router() -> Router<DaemonState> {
    Router::new()
        .route("/api/stitches/:stitch_id/content-blocks", get(list_content_blocks))
        .route("/api/stitches/:stitch_id/content-blocks", post(create_content_block))
        .route("/api/stitches/:stitch_id/content-blocks/:block_id", put(update_content_block_endpoint))
        .route("/api/stitches/:stitch_id/content-blocks/:block_id", delete(delete_content_block_endpoint))
        .route("/api/stitches/:stitch_id/content-blocks/reorder", post(reorder_content_blocks_endpoint))
}
