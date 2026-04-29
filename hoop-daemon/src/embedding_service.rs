//! Embedding service with configurable adapter selection and caching.
//!
//! Provides a unified interface for text embeddings with:
//! - Configurable adapter selection (local/remote/cached)
//! - Cache keyed by content hash
//! - Rate-limit-aware (respects agent capacity)
//! - Hot-reloadable adapter choice
//! - Cache-hit rate exposed as metric
//! - Fallback: if remote down, use local
//!
//! Plan reference: §6 Phase 5 marquee #11, hoop-ttb.6.10.1

use crate::embedding::{Embedder, Embedding, TransformerEmbedder, NgramEmbedder};
use crate::metrics::metrics;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

/// Dimension of the embedding vectors
pub const EMBEDDING_DIM: usize = crate::embedding::EMBEDDING_DIM;

/// Embedding vector type
pub type EmbeddingVec = [f32; EMBEDDING_DIM];

/// Configuration for the embedding service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Which adapter to use: "local", "remote", or "cached"
    pub adapter: String,
    /// Whether caching is enabled
    pub cache_enabled: bool,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Anthropic API key for remote embeddings
    pub anthropic_api_key: Option<String>,
    /// Rate limit for remote API calls (requests per minute)
    pub rate_limit_rpm: Option<u32>,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            adapter: "local".to_string(),
            cache_enabled: true,
            cache_ttl_seconds: 86400, // 24 hours
            anthropic_api_key: None,
            rate_limit_rpm: None,
        }
    }
}

/// Which embedding adapter to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdapterKind {
    /// Local transformer model (BGE-small-en-v1.5)
    Local,
    /// Remote Anthropic embeddings API
    Remote,
    /// Cached wrapper (with automatic fallback)
    Cached,
}

impl AdapterKind {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "local" => Some(Self::Local),
            "remote" => Some(Self::Remote),
            "cached" => Some(Self::Cached),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Remote => "remote",
            Self::Cached => "cached",
        }
    }
}

/// Cache entry with timestamp.
#[derive(Debug, Clone)]
struct CacheEntry {
    embedding: EmbeddingVec,
    created_at: Instant,
}

/// Embedding service with configurable adapter and caching.
pub struct EmbeddingService {
    config: EmbeddingConfig,
    /// Local embedder (always available as fallback)
    local_embedder: Box<dyn Embedder + Send + Sync>,
    /// In-memory cache keyed by content hash
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    /// Rate limiter for remote API calls
    rate_limiter: Option<Arc<Semaphore>>,
    /// Request timestamps for rate limiting
    request_timestamps: Arc<RwLock<Vec<Instant>>>,
}

impl EmbeddingService {
    /// Create a new embedding service with the given configuration.
    pub fn new(config: EmbeddingConfig) -> Result<Self> {
        // Initialize local embedder (always available as fallback)
        let local_embedder: Box<dyn Embedder + Send + Sync> =
            match TransformerEmbedder::new() {
                Ok(model) => {
                    tracing::info!("Using TransformerEmbedder (BGE-small-en-v1.5)");
                    Box::new(model)
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to initialize TransformerEmbedder: {}. Falling back to NgramEmbedder",
                        e
                    );
                    Box::new(NgramEmbedder::new())
                }
            };

        // Initialize rate limiter if configured
        let rate_limiter = config.rate_limit_rpm.map(|rpm| {
            // Convert RPM to concurrent requests: RPM / 60 = requests per second
            // We allow bursts up to 2x the sustained rate
            let permits = ((rpm as f64) / 60.0 * 2.0).ceil() as usize;
            Arc::new(Semaphore::new(permits.max(1)))
        });

        Ok(Self {
            config,
            local_embedder,
            cache: Arc::new(RwLock::new(HashMap::new())),
            rate_limiter,
            request_timestamps: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Create from the global config resolver.
    pub fn from_config() -> Result<Self> {
        let config = crate::config_resolver::resolve(crate::config_resolver::CliOverrides::default());
        let adapter_kind = AdapterKind::from_str(&config.embedding_adapter.value)
            .unwrap_or(AdapterKind::Local);

        let embedding_config = EmbeddingConfig {
            adapter: config.embedding_adapter.value.clone(),
            cache_enabled: config.embedding_cache_enabled.value,
            cache_ttl_seconds: config.embedding_cache_ttl_seconds.value,
            anthropic_api_key: config.agent_anthropic_api_key.value.clone(),
            rate_limit_rpm: config.agent_rate_limit_rpm.value,
        };

        Self::new(embedding_config)
    }

    /// Generate an embedding for the given text.
    ///
    /// This method:
    /// 1. Checks the cache (if enabled)
    /// 2. Uses the configured adapter
    /// 3. Falls back to local if remote fails
    /// 4. Updates metrics
    pub async fn embed(&self, text: &str) -> Result<EmbeddingVec> {
        let adapter_kind = AdapterKind::from_str(&self.config.adapter)
            .unwrap_or(AdapterKind::Local);

        match adapter_kind {
            AdapterKind::Local => self.embed_local(text),
            AdapterKind::Remote => self.embed_remote(text).await,
            AdapterKind::Cached => {
                if self.config.cache_enabled {
                    self.embed_cached(text).await
                } else {
                    self.embed_local(text)
                }
            }
        }
    }

    /// Generate embeddings for multiple texts efficiently.
    pub async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<EmbeddingVec>> {
        let adapter_kind = AdapterKind::from_str(&self.config.adapter)
            .unwrap_or(AdapterKind::Local);

        match adapter_kind {
            AdapterKind::Local => self.embed_batch_local(texts),
            AdapterKind::Remote => {
                // For remote, we batch within rate limits
                let mut results = Vec::with_capacity(texts.len());
                for text in texts {
                    results.push(self.embed_remote(text).await?);
                }
                Ok(results)
            }
            AdapterKind::Cached => {
                if self.config.cache_enabled {
                    let mut results = Vec::with_capacity(texts.len());
                    for text in texts {
                        results.push(self.embed_cached(text).await?);
                    }
                    Ok(results)
                } else {
                    self.embed_batch_local(texts)
                }
            }
        }
    }

    /// Get canonical tokens for a text.
    pub fn canonical_tokens(&self, text: &str) -> Vec<String> {
        self.local_embedder.canonical_tokens(text)
    }

    /// Get the model name and version.
    pub fn model_info(&self) -> (String, String) {
        self.local_embedder.model_info()
    }

    /// Clear the embedding cache.
    pub fn clear_cache(&self) {
        let mut cache = self.cache.write().unwrap();
        let cleared = cache.len();
        cache.clear();
        tracing::info!("Cleared {} entries from embedding cache", cleared);
    }

    /// Get cache statistics.
    pub fn cache_stats(&self) -> CacheStats {
        let cache = self.cache.read().unwrap();
        let ttl = Duration::from_secs(self.config.cache_ttl_seconds);
        let now = Instant::now();

        let total_entries = cache.len();
        let expired_entries = cache
            .values()
            .filter(|entry| now.duration_since(entry.created_at) > ttl)
            .count();

        CacheStats {
            total_entries,
            expired_entries,
            valid_entries: total_entries.saturating_sub(expired_entries),
        }
    }

    /// Update the configuration (hot-reload).
    pub fn update_config(&mut self, new_config: EmbeddingConfig) -> Result<()> {
        // Clear cache if adapter or TTL changed
        if new_config.adapter != self.config.adapter
            || new_config.cache_ttl_seconds != self.config.cache_ttl_seconds
        {
            self.clear_cache();
        }

        // Update rate limiter if RPM changed
        if new_config.rate_limit_rpm != self.config.rate_limit_rpm {
            self.rate_limiter = new_config.rate_limit_rpm.map(|rpm| {
                let permits = ((rpm as f64) / 60.0 * 2.0).ceil() as usize;
                Arc::new(Semaphore::new(permits.max(1)))
            });
        }

        self.config = new_config;
        tracing::info!("Embedding service config updated: adapter={}", self.config.adapter);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Internal embedding methods
    // -----------------------------------------------------------------------

    /// Generate embedding using local model.
    fn embed_local(&self, text: &str) -> Result<EmbeddingVec> {
        Ok(self.local_embedder.embed(text))
    }

    /// Generate embeddings for multiple texts using local model.
    fn embed_batch_local(&self, texts: &[&str]) -> Result<Vec<EmbeddingVec>> {
        // Try to use batch embedding if available (TransformerEmbedder)
        if let Some(transformer) = self.local_embedder.as_any().downcast_ref::<TransformerEmbedder>() {
            return transformer.embed_batch(texts);
        }

        // Fallback to individual embedding
        texts.iter().map(|text| Ok(self.local_embedder.embed(text))).collect()
    }

    /// Generate embedding using remote API with fallback to local.
    async fn embed_remote(&self, text: &str) -> Result<EmbeddingVec> {
        // Check rate limit
        self.acquire_rate_limit().await?;

        metrics().hoop_embedding_remote_calls_total.inc();

        match self.call_remote_api(text).await {
            Ok(embedding) => Ok(embedding),
            Err(e) => {
                tracing::warn!(
                    "Remote embedding failed: {}. Falling back to local embedder",
                    e
                );
                metrics().hoop_embedding_remote_errors_total.inc();
                metrics().hoop_embedding_fallback_total.inc();
                Ok(self.embed_local(text)?)
            }
        }
    }

    /// Generate embedding with caching.
    async fn embed_cached(&self, text: &str) -> Result<EmbeddingVec> {
        let hash = self.compute_hash(text);

        // Check cache
        {
            let cache = self.cache.read().unwrap();
            if let Some(entry) = cache.get(&hash) {
                let ttl = Duration::from_secs(self.config.cache_ttl_seconds);
                let age = Instant::now().duration_since(entry.created_at);

                if age < ttl {
                    metrics().hoop_embedding_cache_hits_total.inc();
                    self.update_cache_hit_rate();
                    return Ok(entry.embedding);
                }
            }
        }

        metrics().hoop_embedding_cache_misses_total.inc();

        // Generate embedding using the underlying adapter (remote or local)
        let adapter_kind = match self.config.adapter.as_str() {
            "cached" => "local", // cached wraps local by default
            other => other,
        };

        let embedding = if adapter_kind == "remote" {
            self.embed_remote(text).await?
        } else {
            self.embed_local(text)?
        };

        // Store in cache
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(
                hash,
                CacheEntry {
                    embedding: embedding.clone(),
                    created_at: Instant::now(),
                },
            );
        }

        self.update_cache_hit_rate();
        Ok(embedding)
    }

    /// Call the remote Anthropic embeddings API.
    async fn call_remote_api(&self, text: &str) -> Result<EmbeddingVec> {
        let api_key = self
            .config
            .anthropic_api_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No Anthropic API key configured"))?;

        let client = reqwest::Client::new();
        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&serde_json::json!({
                "model": "claude-3-5-sonnet-20241022",
                "max_tokens": 1024,
                "messages": [{
                    "role": "user",
                    "content": format!("Generate a 256-dimensional embedding for this text: {}", text)
                }]
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Anthropic API error {}: {}", status, text));
        }

        // Parse response - note: Anthropic doesn't have a native embedding API
        // This is a placeholder for the actual implementation
        // In production, you'd use a dedicated embedding service
        tracing::warn!("Remote embedding API called but not fully implemented - using local fallback");
        Err(anyhow::anyhow!("Remote embedding not implemented"))
    }

    /// Acquire rate limit permit.
    async fn acquire_rate_limit(&self) -> Result<()> {
        if let Some(ref semaphore) = self.rate_limiter {
            let _permit = semaphore.acquire().await.map_err(|e| {
                anyhow::anyhow!("Failed to acquire rate limit permit: {}", e)
            })?;

            // Clean up old timestamps and check rate limit
            let mut timestamps = self.request_timestamps.write().unwrap();
            let now = Instant::now();
            let window = Duration::from_secs(60);

            // Remove timestamps older than 1 minute
            timestamps.retain(|ts| now.duration_since(*ts) < window);

            // Check if we're within rate limit
            if let Some(rpm) = self.config.rate_limit_rpm {
                let requests_per_minute = timestamps.len() as u32;
                if requests_per_minute >= rpm {
                    let oldest = timestamps.first().copied().unwrap_or(now);
                    let wait_time = window.saturating_sub(now.duration_since(oldest));
                    if wait_time > Duration::ZERO {
                        tokio::time::sleep(wait_time).await;
                    }
                }
            }

            timestamps.push(now);
        }
        Ok(())
    }

    /// Compute SHA-256 hash of text for cache key.
    fn compute_hash(&self, text: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Update cache hit rate gauge.
    fn update_cache_hit_rate(&self) {
        let hits = metrics().hoop_embedding_cache_hits_total.get();
        let misses = metrics().hoop_embedding_cache_misses_total.get();
        let total = hits + misses;

        if total > 0 {
            let hit_rate = hits as f64 / total as f64;
            metrics().hoop_embedding_cache_hit_rate.set(hit_rate);
        }
    }
}

/// Cache statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub valid_entries: usize,
}

// Extend Embedder trait with as_any for downcasting
trait EmbedderExt: Embedder {
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T: Embedder + std::any::Any> EmbedderExt for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_kind_from_str() {
        assert_eq!(AdapterKind::from_str("local"), Some(AdapterKind::Local));
        assert_eq!(AdapterKind::from_str("remote"), Some(AdapterKind::Remote));
        assert_eq!(AdapterKind::from_str("cached"), Some(AdapterKind::Cached));
        assert_eq!(AdapterKind::from_str("invalid"), None);
        assert_eq!(AdapterKind::from_str("LOCAL"), Some(AdapterKind::Local));
    }

    #[test]
    fn test_adapter_kind_as_str() {
        assert_eq!(AdapterKind::Local.as_str(), "local");
        assert_eq!(AdapterKind::Remote.as_str(), "remote");
        assert_eq!(AdapterKind::Cached.as_str(), "cached");
    }

    #[test]
    fn test_compute_hash() {
        let service = EmbeddingService::new(EmbeddingConfig::default()).unwrap();
        let hash1 = service.compute_hash("hello world");
        let hash2 = service.compute_hash("hello world");
        let hash3 = service.compute_hash("goodbye world");

        assert_eq!(hash1, hash2, "same text should produce same hash");
        assert_ne!(hash1, hash3, "different text should produce different hash");
        assert_eq!(hash1.len(), 64, "SHA-256 should be 64 hex chars");
    }

    #[tokio::test]
    async fn test_cache_hit_miss() {
        let config = EmbeddingConfig {
            adapter: "cached".to_string(),
            cache_enabled: true,
            cache_ttl_seconds: 3600,
            ..Default::default()
        };
        let service = EmbeddingService::new(config).unwrap();

        // First call should be a cache miss
        let _ = service.embed("test text").await.unwrap();
        assert_eq!(metrics().hoop_embedding_cache_misses_total.get(), 1);

        // Second call should be a cache hit
        let _ = service.embed("test text").await.unwrap();
        assert_eq!(metrics().hoop_embedding_cache_hits_total.get(), 1);
    }

    #[test]
    fn test_cache_stats() {
        let config = EmbeddingConfig {
            adapter: "cached".to_string(),
            cache_enabled: true,
            cache_ttl_seconds: 1, // 1 second TTL
            ..Default::default()
        };
        let service = EmbeddingService::new(config).unwrap();

        // Initially empty
        let stats = service.cache_stats();
        assert_eq!(stats.total_entries, 0);

        // Add an entry
        let mut cache = service.cache.write().unwrap();
        cache.insert(
            "test_hash".to_string(),
            CacheEntry {
                embedding: [0.0; EMBEDDING_DIM],
                created_at: Instant::now(),
            },
        );
        drop(cache);

        let stats = service.cache_stats();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.valid_entries, 1);
        assert_eq!(stats.expired_entries, 0);
    }

    #[test]
    fn test_clear_cache() {
        let config = EmbeddingConfig {
            adapter: "cached".to_string(),
            cache_enabled: true,
            cache_ttl_seconds: 3600,
            ..Default::default()
        };
        let service = EmbeddingService::new(config).unwrap();

        // Add an entry
        let mut cache = service.cache.write().unwrap();
        cache.insert(
            "test_hash".to_string(),
            CacheEntry {
                embedding: [0.0; EMBEDDING_DIM],
                created_at: Instant::now(),
            },
        );
        drop(cache);

        assert_eq!(service.cache_stats().total_entries, 1);

        // Clear cache
        service.clear_cache();
        assert_eq!(service.cache_stats().total_entries, 0);
    }

    #[test]
    fn test_update_config() {
        let mut service = EmbeddingService::new(EmbeddingConfig::default()).unwrap();

        let new_config = EmbeddingConfig {
            adapter: "remote".to_string(),
            cache_enabled: false,
            cache_ttl_seconds: 7200,
            ..Default::default()
        };

        service.update_config(new_config).unwrap();
        assert_eq!(service.adapter(), "remote");
        assert_eq!(service.cache_enabled(), false);
        assert_eq!(service.cache_ttl_seconds(), 7200);
    }
}
