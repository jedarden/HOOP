//! Local text embedding for semantic deduplication (CPU-bound, no external API)
//!
//! The local embedder uses deterministic character n-gram feature hashing, so
//! semantic deduplication remains available without an external model.

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

/// N-gram hashing embedder.
///
/// Each lowercase character trigram contributes one count to a deterministic
/// bucket. The resulting vector is L2-normalized before it is returned.
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

    fn embed(&self, text: &str) -> Embedding {
        let mut embedding = [0.0f32; EMBEDDING_DIM];
        let bucket_count = self.dims.min(EMBEDDING_DIM);
        if bucket_count == 0 {
            return embedding;
        }

        let lowercase: Vec<char> = text.chars().flat_map(char::to_lowercase).collect();
        for ngram in lowercase.windows(3) {
            let bucket = hash_ngram(ngram) % bucket_count;
            embedding[bucket] += 1.0;
        }
        if !lowercase.is_empty() && lowercase.len() < 3 {
            let bucket = hash_ngram(&lowercase) % bucket_count;
            embedding[bucket] += 1.0;
        }

        let norm = embedding
            .iter()
            .map(|value| value * value)
            .sum::<f32>()
            .sqrt();
        if norm > 0.0 {
            for value in &mut embedding {
                *value /= norm;
            }
        }

        embedding
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

/// Hash a character n-gram without relying on a process-randomized hasher.
fn hash_ngram(ngram: &[char]) -> usize {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    ngram.iter().fold(FNV_OFFSET_BASIS, |hash, character| {
        (*character as u32)
            .to_le_bytes()
            .into_iter()
            .fold(hash, |hash, byte| {
                (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME)
            })
    }) as usize
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embeds_non_empty_text_into_a_non_zero_vector() {
        let embedding = NgramEmbedder::new().embed("login bug");

        assert!(embedding.iter().any(|value| *value != 0.0));
    }

    #[test]
    fn embeds_short_non_empty_text_into_a_non_zero_vector() {
        let embedding = NgramEmbedder::new().embed("x");

        assert!(embedding.iter().any(|value| *value != 0.0));
    }

    #[test]
    fn similar_sentences_have_high_cosine_similarity() {
        let embedder = NgramEmbedder::new();
        let first = embedder.embed("fix login bug in auth module");
        let second = embedder.embed("fix the login bug in the auth module");

        assert!(
            cosine_similarity(&first, &second) > 0.7,
            "expected similar sentences to have high cosine similarity"
        );
    }

    #[test]
    fn unrelated_sentences_have_low_cosine_similarity() {
        let embedder = NgramEmbedder::new();
        let first = embedder.embed("fix login bug in auth module");
        let second = embedder.embed("the ocean tide moves beneath the moon");

        assert!(
            cosine_similarity(&first, &second) < 0.3,
            "expected unrelated sentences to have low cosine similarity"
        );
    }

    #[test]
    fn hashing_is_deterministic_and_case_insensitive() {
        let embedder = NgramEmbedder::new();
        let lowercase = embedder.embed("Fix Login Bug");
        let uppercase = embedder.embed("fix login bug");

        assert_eq!(lowercase, uppercase);
    }
}
