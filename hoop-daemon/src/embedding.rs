//! Local text embedding via n-gram hashing (CPU-bound, no external API)
//!
//! Produces fixed-dimension vectors from text using character and word n-gram
//! hashing. This approach is used in production systems (fastText, Vowpal Wabbit)
//! and provides robust semantic similarity for short text like titles and
//! descriptions without requiring model downloads or GPU inference.

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
    pub kind: String, // "bead" or "stitch"
}

/// Result of a dedup check
#[derive(Debug, Clone)]
pub struct DedupMatch {
    pub item: IndexedItem,
    pub similarity: f64,
}

/// Trait for embedding text into fixed-dimension vectors.
///
/// The production implementation uses n-gram hashing. A future implementation
/// could swap in a transformer model (gte-small, all-MiniLM-L6-v2) via candle/ort.
pub trait Embedder: Send + Sync {
    fn embed(&self, text: &str) -> Embedding;
}

/// N-gram hashing embedder — CPU-bound, no external dependencies.
///
/// How it works:
/// 1. Tokenize text into lowercase words
/// 2. Generate character n-grams (3, 4, 5 chars) per word (captures morphological similarity)
/// 3. Generate word unigrams and bigrams (captures lexical similarity)
/// 4. Hash each n-gram to two dimensions (double hashing reduces collision noise)
/// 5. Add +1 to one dimension and -1 to the other (sign hashing)
/// 6. L2-normalize the vector for cosine similarity
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

    /// Generate character n-grams from a word
    fn char_ngrams(word: &str, min_n: usize, max_n: usize) -> Vec<String> {
        let chars: Vec<char> = format!("<{}>", word).chars().collect();
        let mut ngrams = Vec::new();
        for n in min_n..=max_n {
            if chars.len() >= n {
                for i in 0..=chars.len() - n {
                    let ng: String = chars[i..i + n].iter().collect();
                    ngrams.push(ng);
                }
            }
        }
        ngrams
    }

    /// Generate word n-grams from a list of tokens
    fn word_ngrams(tokens: &[String], max_n: usize) -> Vec<String> {
        let mut ngrams = Vec::new();
        for n in 1..=max_n {
            if tokens.len() >= n {
                for i in 0..=tokens.len() - n {
                    let ng = tokens[i..i + n].join(" ");
                    ngrams.push(ng);
                }
            }
        }
        ngrams
    }

    /// Simple FNV-1a hash for consistent dimension mapping
    fn hash_ngram(ngram: &str) -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325;
        for byte in ngram.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }
}

impl Default for NgramEmbedder {
    fn default() -> Self {
        Self::new()
    }
}

impl Embedder for NgramEmbedder {
    fn embed(&self, text: &str) -> Embedding {
        let mut vec = vec![0.0f32; self.dims];

        // Tokenize
        let tokens: Vec<String> = text
            .to_lowercase()
            .split_whitespace()
            .map(|s| {
                s.trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string()
            })
            .filter(|s| !s.is_empty())
            .collect();

        if tokens.is_empty() {
            let mut emb = [0.0f32; EMBEDDING_DIM];
            return emb;
        }

        // Collect all n-grams
        let mut all_ngrams: Vec<String> = Vec::new();

        // Character n-grams from each word (3, 4, 5 chars)
        for token in &tokens {
            all_ngrams.extend(Self::char_ngrams(token, 3, 5));
        }

        // Word unigrams and bigrams
        all_ngrams.extend(Self::word_ngrams(&tokens, 2));

        // Hash each n-gram into the vector using double hashing
        let dims = self.dims;
        for ngram in &all_ngrams {
            let h1 = Self::hash_ngram(ngram);
            let h2 = {
                // Second hash from a different seed
                let mut hash: u64 = 0x9e3779b97f4a7c15;
                for byte in ngram.bytes() {
                    hash ^= byte as u64;
                    hash = hash.wrapping_mul(0x100000001b3);
                }
                hash
            };

            let idx1 = (h1 % dims as u64) as usize;
            let idx2 = (h2 % dims as u64) as usize;

            vec[idx1] += 1.0;
            vec[idx2] -= 1.0;
        }

        // L2-normalize
        let norm: f32 = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in vec.iter_mut() {
                *v /= norm;
            }
        }

        // Copy into fixed-size array
        let mut emb = [0.0f32; EMBEDDING_DIM];
        let copy_len = dims.min(EMBEDDING_DIM);
        emb[..copy_len].copy_from_slice(&vec[..copy_len]);
        emb
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_texts_high_similarity() {
        let embedder = NgramEmbedder::new();
        let a = embedder.embed("Fix the authentication bug in login flow");
        let b = embedder.embed("Fix the authentication bug in login flow");
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 0.001, "identical texts should have sim ~1.0, got {}", sim);
    }

    #[test]
    fn test_similar_texts_high_similarity() {
        let embedder = NgramEmbedder::new();
        let a = embedder.embed("Fix authentication bug in login flow");
        let b = embedder.embed("Fix auth bug in the login process");
        let sim = cosine_similarity(&a, &b);
        assert!(sim > 0.7, "similar texts should have sim > 0.7, got {}", sim);
    }

    #[test]
    fn test_different_texts_low_similarity() {
        let embedder = NgramEmbedder::new();
        let a = embedder.embed("Fix authentication bug in login flow");
        let b = embedder.embed("Add dark mode support to settings page");
        let sim = cosine_similarity(&a, &b);
        assert!(sim < 0.5, "unrelated texts should have sim < 0.5, got {}", sim);
    }

    #[test]
    fn test_empty_text_zero_norm() {
        let embedder = NgramEmbedder::new();
        let a = embedder.embed("some text here");
        let b = embedder.embed("");
        let sim = cosine_similarity(&a, &b);
        assert_eq!(sim, 0.0, "empty text should give 0 similarity");
    }

    #[test]
    fn test_both_empty() {
        let embedder = NgramEmbedder::new();
        let a = embedder.embed("");
        let b = embedder.embed("");
        let sim = cosine_similarity(&a, &b);
        assert_eq!(sim, 0.0);
    }

    #[test]
    fn test_cross_project_duplicate_detection() {
        // Simulates a duplicate across projects:
        // Project A has "Implement user auth with OAuth2"
        // Project B drafts "Implement OAuth2 user authentication"
        let embedder = NgramEmbedder::new();
        let existing = embedder.embed("Implement user auth with OAuth2 provider");
        let draft = embedder.embed("Implement OAuth2 user authentication provider");
        let sim = cosine_similarity(&existing, &draft);
        assert!(sim > 0.82, "cross-project duplicate should exceed 0.82 threshold, got {}", sim);
    }

    #[test]
    fn test_semdup_recall_synthetic() {
        // Synthetic test: 10 paraphrases of the same intent should all match
        let embedder = NgramEmbedder::new();
        let originals = vec![
            "Fix the race condition in session handler",
            "Fix race condition in session handling",
            "Session handler race condition fix",
            "Resolve race condition in session management",
            "Fix concurrent session handler bug",
        ];
        let drafts = vec![
            "Fix race condition in session handler",
            "Patch session race condition bug",
            "Session race condition repair",
            "Fix threading issue in sessions",
            "Session handler concurrency fix",
        ];

        let orig_embs: Vec<_> = originals.iter().map(|t| embedder.embed(t)).collect();
        let draft_embs: Vec<_> = drafts.iter().map(|t| embedder.embed(t)).collect();

        let mut matches = 0;
        let mut total = 0;
        for draft_emb in &draft_embs {
            for orig_emb in &orig_embs {
                let sim = cosine_similarity(draft_emb, orig_emb);
                total += 1;
                if sim > 0.65 {
                    matches += 1;
                }
            }
        }
        let recall = matches as f64 / total as f64;
        assert!(recall > 0.95, "recall should be >95% for synthetic duplicates, got {:.1}%", recall * 100.0);
    }

    #[test]
    fn test_char_ngrams_correct() {
        let ngrams = NgramEmbedder::char_ngrams("hello", 3, 3);
        assert!(ngrams.contains(&"<he".to_string()));
        assert!(ngrams.contains(&"hel".to_string()));
        assert!(ngrams.contains(&"ell".to_string()));
        assert!(ngrams.contains(&"llo".to_string()));
        assert!(ngrams.contains(&"lo>".to_string()));
    }

    #[test]
    fn test_word_ngrams_correct() {
        let tokens = vec!["fix".to_string(), "bug".to_string(), "now".to_string()];
        let ngrams = NgramEmbedder::word_ngrams(&tokens, 2);
        assert!(ngrams.contains(&"fix".to_string()));
        assert!(ngrams.contains(&"bug".to_string()));
        assert!(ngrams.contains(&"now".to_string()));
        assert!(ngrams.contains(&"fix bug".to_string()));
        assert!(ngrams.contains(&"bug now".to_string()));
    }

    #[test]
    fn test_embedding_normalized() {
        let embedder = NgramEmbedder::new();
        let emb = embedder.embed("some text for normalization test");
        let norm: f32 = emb.iter().map(|v| v * v).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.001, "embedding should be L2-normalized, got norm {}", norm);
    }
}
