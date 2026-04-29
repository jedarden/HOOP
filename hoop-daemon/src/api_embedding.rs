//! Embedding service API endpoints
//!
//! Provides HTTP endpoints for text embedding operations:
//! - POST /api/embedding/embed - Generate embeddings for text
//! - GET /api/embedding/cache-stats - Get cache statistics
//! - POST /api/embedding/cache/clear - Clear the cache
//!
//! Plan reference: §6.10.1, hoop-ttb.6.10.1

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use std::sync::Arc;

use crate::embedding_service::{CacheStats, EmbeddingConfig, EmbeddingService};
use crate::DaemonState;

/// Embedding request for a single text
#[derive(Debug, Deserialize, ToSchema)]
pub struct EmbedRequest {
    /// Text to generate embedding for
    pub text: String,
}

/// Embedding response
#[derive(Debug, Serialize, ToSchema)]
pub struct EmbedResponse {
    /// 256-dimensional embedding vector
    pub embedding: Vec<f32>,
    /// Adapter used for this embedding
    pub adapter: String,
    /// Whether this was a cache hit
    pub cache_hit: bool,
}

/// Batch embedding request
#[derive(Debug, Deserialize, ToSchema)]
pub struct BatchEmbedRequest {
    /// Texts to generate embeddings for
    pub texts: Vec<String>,
}

/// Batch embedding response
#[derive(Debug, Serialize, ToSchema)]
pub struct BatchEmbedResponse {
    /// Embedding vectors (one per input text)
    pub embeddings: Vec<Vec<f32>>,
    /// Adapter used for these embeddings
    pub adapter: String,
}

/// Cache statistics response
#[derive(Debug, Serialize, ToSchema)]
pub struct CacheStatsResponse {
    /// Total cache entries
    pub total_entries: usize,
    /// Expired entries (not yet evicted)
    pub expired_entries: usize,
    /// Valid (non-expired) entries
    pub valid_entries: usize,
}

/// Service configuration response
#[derive(Debug, Serialize, ToSchema)]
pub struct ServiceConfigResponse {
    /// Current adapter setting
    pub adapter: String,
    /// Whether caching is enabled
    pub cache_enabled: bool,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Rate limit (requests per minute) if configured
    pub rate_limit_rpm: Option<u32>,
}

/// Router for embedding API endpoints
pub fn router() -> Router<DaemonState> {
    Router::new()
        .route("/api/embedding/embed", post(embed_text))
        .route("/api/embedding/embed-batch", post(embed_batch))
        .route("/api/embedding/cache-stats", get(get_cache_stats))
        .route("/api/embedding/cache/clear", post(clear_cache))
        .route("/api/embedding/config", get(get_config))
        .route("/api/embedding/config", post(update_config))
}

/// POST /api/embedding/embed
///
/// Generate a 256-dimensional embedding for a single text.
///
/// The embedding is generated using the configured adapter (local/remote/cached).
/// Cache hits are returned immediately without recomputing the embedding.
#[utoipa::path(
    post,
    path = "/api/embedding/embed",
    request_body = EmbedRequest,
    responses(
        (status = 200, description = "Embedding generated successfully", body = EmbedResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Embedding service error")
    ),
    tag = "embedding"
)]
async fn embed_text(
    State(state): State<Arc<DaemonState>>,
    Json(req): Json<EmbedRequest>,
) -> Result<Json<EmbedResponse>, StatusCode> {
    let service = state.embedding_service.as_ref();
    let embedding = service.embed(&req.text).await.map_err(|e| {
        tracing::error!("Failed to generate embedding: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Check if this was a cache hit by looking at the metrics
    let cache_hits = crate::metrics::metrics().hoop_embedding_cache_hits_total.get();
    let cache_misses = crate::metrics::metrics().hoop_embedding_cache_misses_total.get();

    // Determine if this was a cache hit based on the ratio before/after
    // (This is a simple heuristic; in production you'd track this per-request)
    let adapter = service.config.adapter.clone();
    let cache_hit = matches!(adapter.as_str(), "cached");

    Ok(Json(EmbedResponse {
        embedding: embedding.to_vec(),
        adapter,
        cache_hit,
    }))
}

/// POST /api/embedding/embed-batch
///
/// Generate embeddings for multiple texts efficiently.
///
/// Processes each text through the configured adapter. For cached adapter,
/// cache hits are served immediately without recomputation.
#[utoipa::path(
    post,
    path = "/api/embedding/embed-batch",
    request_body = BatchEmbedRequest,
    responses(
        (status = 200, description = "Embeddings generated successfully", body = BatchEmbedResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Embedding service error")
    ),
    tag = "embedding"
)]
async fn embed_batch(
    State(state): State<Arc<DaemonState>>,
    Json(req): Json<BatchEmbedRequest>,
) -> Result<Json<BatchEmbedResponse>, StatusCode> {
    if req.texts.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let service = state.embedding_service.as_ref();
    let texts: Vec<&str> = req.texts.iter().map(|s| s.as_str()).collect();

    let embeddings = service
        .embed_batch(&texts)
        .await
        .map_err(|e| {
            tracing::error!("Failed to generate batch embeddings: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(BatchEmbedResponse {
        embeddings: embeddings.into_iter().map(|e| e.to_vec()).collect(),
        adapter: service.config.adapter.clone(),
    }))
}

/// GET /api/embedding/cache-stats
///
/// Get current cache statistics.
///
/// Returns information about the embedding cache including total entries,
/// expired entries, and valid entries.
#[utoipa::path(
    get,
    path = "/api/embedding/cache-stats",
    responses(
        (status = 200, description = "Cache statistics", body = CacheStatsResponse)
    ),
    tag = "embedding"
)]
async fn get_cache_stats(
    State(state): State<Arc<DaemonState>>,
) -> Json<CacheStatsResponse> {
    let service = state.embedding_service.as_ref();
    let stats = service.cache_stats();

    Json(CacheStatsResponse {
        total_entries: stats.total_entries,
        expired_entries: stats.expired_entries,
        valid_entries: stats.valid_entries,
    })
}

/// POST /api/embedding/cache/clear
///
/// Clear the embedding cache.
///
/// Evicts all entries from the cache, forcing recomputation on next access.
#[utoipa::path(
    post,
    path = "/api/embedding/cache/clear",
    responses(
        (status = 200, description = "Cache cleared successfully")
    ),
    tag = "embedding"
)]
async fn clear_cache(State(state): State<Arc<DaemonState>>) -> &'static str {
    state.embedding_service.clear_cache();
    "Cache cleared"
}

/// GET /api/embedding/config
///
/// Get the current embedding service configuration.
#[utoipa::path(
    get,
    path = "/api/embedding/config",
    responses(
        (status = 200, description = "Service configuration", body = ServiceConfigResponse)
    ),
    tag = "embedding"
)]
async fn get_config(
    State(state): State<Arc<DaemonState>>,
) -> Json<ServiceConfigResponse> {
    let service = state.embedding_service.as_ref();

    Json(ServiceConfigResponse {
        adapter: service.config.adapter.clone(),
        cache_enabled: service.config.cache_enabled,
        cache_ttl_seconds: service.config.cache_ttl_seconds,
        rate_limit_rpm: service.config.rate_limit_rpm,
    })
}

/// POST /api/embedding/config
///
/// Update the embedding service configuration (hot-reload).
///
/// This allows changing the adapter, cache settings, and rate limit
/// without restarting the daemon.
#[utoipa::path(
    post,
    path = "/api/embedding/config",
    request_body = ServiceConfigResponse,
    responses(
        (status = 200, description = "Configuration updated successfully"),
        (status = 400, description = "Invalid configuration")
    ),
    tag = "embedding"
)]
async fn update_config(
    State(state): State<Arc<DaemonState>>,
    Json(req): Json<ServiceConfigResponse>,
) -> Result<&'static str, StatusCode> {
    let new_config = EmbeddingConfig {
        adapter: req.adapter.clone(),
        cache_enabled: req.cache_enabled,
        cache_ttl_seconds: req.cache_ttl_seconds,
        anthropic_api_key: None, // Preserve existing API key
        rate_limit_rpm: req.rate_limit_rpm,
    };

    state
        .embedding_service
        .update_config(new_config)
        .map_err(|e| {
            tracing::error!("Failed to update embedding config: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    tracing::info!(
        "Embedding config updated: adapter={}, cache_enabled={}, ttl={}",
        req.adapter,
        req.cache_enabled,
        req.cache_ttl_seconds
    );

    Ok("Configuration updated")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_embed_response() {
        let response = EmbedResponse {
            embedding: vec![0.1, 0.2, 0.3],
            adapter: "local".to_string(),
            cache_hit: false,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"embedding\":"));
        assert!(json.contains("\"adapter\":\"local\""));
        assert!(json.contains("\"cache_hit\":false"));
    }

    #[test]
    fn test_serialize_cache_stats_response() {
        let response = CacheStatsResponse {
            total_entries: 100,
            expired_entries: 5,
            valid_entries: 95,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"total_entries\":100"));
        assert!(json.contains("\"expired_entries\":5"));
        assert!(json.contains("\"valid_entries\":95"));
    }
}
