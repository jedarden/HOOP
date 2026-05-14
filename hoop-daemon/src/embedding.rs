//! Local text embedding for semantic deduplication (CPU-bound, no external API)
//!
//! STUB VERSION: fastembed temporarily disabled for compilation
//! TODO: Re-enable fastembed after fixing compilation errors

/// Dimension of the embedding vectors
pub const EMBEDDING_DIM: usize = 256;

/// Embedding vector type
pub type Embedding = [f32; EMBEDDING_DIM];

/// A record in the vector index, representing an open stitch/bead across projects
#[derive(Debug, Clone)]
pub struct IndexedItem {
    pub id: String,
    pub project: String,
    pub title: String,
    pub kind: String,
    pub description: Option<String>,
}

/// Result of a dedup check
#[derive(Debug, Clone)]
pub struct DedupMatch {
    pub item: IndexedItem,
    pub similarity: f64,
}

/// Trait for embedding text into fixed-dimension vectors.
pub trait Embedder: Send + Sync {
    fn embed(&self, text: &str) -> Embedding;
    fn canonical_tokens(&self, text: &str) -> Vec<String>;
    fn model_info(&self) -> (String, String);
    fn as_any(&self) -> &dyn std::any::Any;
}

/// N-gram hashing embedder (fallback implementation)
pub struct NgramEmbedder {
    dims: usize,
}

impl NgramEmbedder {
    pub fn new() -> Self {
        Self::with_dims(EMBEDDING_DIM)
    }

    pub fn with_dims(dims: usize) -> Self {
        Self { dims }
    }
}

impl Default for NgramEmbedder {
    fn default() -> Self {
        Self::new()
    }
}

impl Embedder for NgramEmbedder {
    fn model_info(&self) -> (String, String) {
        ("ngram-hash".to_string(), format!("dims-{}", self.dims))
    }

    fn embed(&self, _text: &str) -> Embedding {
        [0.0f32; EMBEDDING_DIM]
    }

    fn canonical_tokens(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Compute cosine similarity between two embeddings
pub fn cosine_similarity(a: &Embedding, b: &Embedding) -> f64 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|v| v * v).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|v| v * v).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    (dot / (norm_a * norm_b)) as f64
}

/// Compute Jaccard similarity between two token sets
pub fn jaccard_similarity(tokens_a: &[String], tokens_b: &[String]) -> f64 {
    if tokens_a.is_empty() && tokens_b.is_empty() {
        return 1.0;
    }
    if tokens_a.is_empty() || tokens_b.is_empty() {
        return 0.0;
    }

    let set_a: std::collections::HashSet<_> = tokens_a.iter().collect();
    let set_b: std::collections::HashSet<_> = tokens_b.iter().collect();

    let intersection = set_a.intersection(&set_b).count() as f64;
    let union = set_a.union(&set_b).count() as f64;

    if union == 0.0 {
        return 0.0;
    }

    intersection / union
}
