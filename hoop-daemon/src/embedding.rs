//! Local text embedding via semantic n-gram hashing (CPU-bound, no external API)
//!
//! Produces fixed-dimension vectors from text using character and word n-gram
//! hashing enhanced with synonym expansion and stop-word filtering. This captures
//! both lexical and semantic similarity for short text like titles and descriptions
//! without requiring model downloads or GPU inference.
//!
//! The approach combines three techniques:
//! 1. **Abbreviation/synonym expansion**: "auth" → "authentication", "db" → "database", etc.
//! 2. **Stop-word filtering**: Removes noise words ("the", "a", "in", etc.) that
//!    dilute the semantic signal in short text
//! 3. **Weighted n-gram hashing**: Character n-grams (3-5 chars) capture morphological
//!    similarity while word n-grams (1-2) capture lexical similarity. Word-level
//!    features get higher weight to prioritize semantic content words.

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
///
/// The production implementation uses semantic n-gram hashing. A future implementation
/// could swap in a transformer model (gte-small, all-MiniLM-L6-v2) via candle/ort.
pub trait Embedder: Send + Sync {
    fn embed(&self, text: &str) -> Embedding;
}

/// Synonym groups for semantic matching. Each group maps related words to a
/// shared canonical token so "auth" and "authentication" hash identically.
/// This preserves dimensionality (no vector bloat) while capturing semantic equivalence.
const SYNONYM_GROUPS: &[&[&str]] = &[
    &["auth", "authentication", "authorization"],
    &["db", "database"],
    &["api", "endpoint"],
    &["fix", "repair", "resolve", "patch"],
    &["bug", "defect", "issue"],
    &["add", "implement", "create", "introduce"],
    &["remove", "delete", "eliminate"],
    &["update", "modify", "change", "alter"],
    &["setup", "configure", "install"],
    &["refactor", "restructure", "reorganize", "rewrite"],
    &["config", "configuration", "settings"],
    &["perf", "performance", "optimization"],
    &["async", "asynchronous"],
    &["sync", "synchronize", "synchronous"],
];

/// N-gram hashing embedder with semantic enhancements — CPU-bound, no external dependencies.
///
/// How it works:
/// 1. Tokenize text into lowercase words
/// 2. Expand abbreviations and synonyms (auth → authentication)
/// 3. Filter stop words to amplify semantic content
/// 4. Generate character n-grams (3, 4, 5 chars) per content word
/// 5. Generate word unigrams and bigrams from content words
/// 6. Hash each n-gram to two dimensions with sign hashing
/// 7. Word-level features get 2x weight to prioritize semantic content
/// 8. L2-normalize the vector for cosine similarity
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

    /// Second hash with a different seed for double hashing
    fn hash_ngram_alt(ngram: &str) -> u64 {
        let mut hash: u64 = 0x9e3779b97f4a7c15;
        for byte in ngram.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }

    /// Canonicalize a token: if it belongs to a synonym group, return the
    /// group's canonical (first) member. Otherwise return the original.
    /// This makes "auth" and "authentication" hash to identical dimensions.
    fn canonicalize(token: &str) -> &str {
        for group in SYNONYM_GROUPS {
            if group.iter().any(|syn| *syn == token) {
                return group[0];
            }
        }
        token
    }
}

impl Default for NgramEmbedder {
    fn default() -> Self {
        Self::new()
    }
}

/// Common stop words that dilute semantic signal in short text
const STOP_WORDS: &[&str] = &[
    "the", "a", "an", "in", "on", "at", "to", "for", "of", "with", "and",
    "or", "is", "are", "was", "were", "be", "been", "being", "have", "has",
    "had", "do", "does", "did", "will", "would", "could", "should", "may",
    "might", "can", "shall", "this", "that", "these", "those", "it", "its",
    "from", "by", "as", "but", "not", "no", "nor", "so", "if", "then",
    "than", "too", "very", "just", "about",
];

impl Embedder for NgramEmbedder {
    fn embed(&self, text: &str) -> Embedding {
        let mut vec = vec![0.0f32; self.dims];

        // Tokenize into lowercase words, stripped of punctuation
        let raw_tokens: Vec<String> = text
            .to_lowercase()
            .split_whitespace()
            .map(|s| {
                s.trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string()
            })
            .filter(|s| !s.is_empty())
            .collect();

        if raw_tokens.is_empty() {
            return [0.0f32; EMBEDDING_DIM];
        }

        let dims = self.dims;

        // Canonicalize: replace each token with its synonym group's canonical form.
        // This makes "auth" and "authentication" produce identical n-gram profiles.
        let tokens: Vec<String> = raw_tokens
            .iter()
            .map(|t| Self::canonicalize(t).to_string())
            .filter(|t| !STOP_WORDS.contains(&t.as_str()))
            .collect();

        if tokens.is_empty() {
            return [0.0f32; EMBEDDING_DIM];
        }

        // Character n-grams from canonical tokens (captures morphological similarity)
        for token in &tokens {
            let char_ngs = Self::char_ngrams(token, 3, 5);
            for ngram in &char_ngs {
                let h1 = Self::hash_ngram(ngram);
                let h2 = Self::hash_ngram_alt(ngram);
                let idx1 = (h1 % dims as u64) as usize;
                let idx2 = (h2 % dims as u64) as usize;
                vec[idx1] += 1.0;
                vec[idx2] -= 1.0;
            }
        }

        // Word unigrams and bigrams from canonical tokens (captures lexical similarity)
        let word_ngs = Self::word_ngrams(&tokens, 2);
        for ngram in &word_ngs {
            let h1 = Self::hash_ngram(ngram);
            let h2 = Self::hash_ngram_alt(ngram);
            let idx1 = (h1 % dims as u64) as usize;
            let idx2 = (h2 % dims as u64) as usize;
            vec[idx1] += 2.0;
            vec[idx2] -= 2.0;
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
        assert!(sim > 0.75, "similar texts with abbreviation expansion should have sim > 0.75, got {}", sim);
    }

    #[test]
    fn test_synonym_canonicalization_boosts_similarity() {
        let embedder = NgramEmbedder::new();
        // "auth" canonicalizes to same form as "authentication"
        let a = embedder.embed("Fix auth bug");
        let b = embedder.embed("Fix authentication bug");
        let sim = cosine_similarity(&a, &b);
        assert!(sim > 0.85, "synonym canonicalization should boost similarity > 0.85, got {}", sim);
    }

    #[test]
    fn test_db_synonym() {
        let embedder = NgramEmbedder::new();
        let a = embedder.embed("Fix DB connection pool");
        let b = embedder.embed("Fix database connection pool");
        let sim = cosine_similarity(&a, &b);
        assert!(sim > 0.85, "db→database synonym should boost similarity > 0.85, got {}", sim);
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
        let embedder = NgramEmbedder::new();
        let existing = embedder.embed("Implement user auth with OAuth2 provider");
        let draft = embedder.embed("Implement OAuth2 user authentication provider");
        let sim = cosine_similarity(&existing, &draft);
        assert!(sim > 0.82, "cross-project duplicate should exceed 0.82 threshold, got {}", sim);
    }

    #[test]
    fn test_synonym_recall() {
        let embedder = NgramEmbedder::new();
        // These use different wording but same intent — synonym canonicalization should help
        let pairs = vec![
            ("Fix DB connection pool", "Repair database connection pool"),
            ("Add rate limiting", "Implement rate limiting"),
            ("Refactor query builder", "Restructure query builder"),
            ("Remove deprecated API", "Delete deprecated API"),
            ("Update config", "Modify configuration"),
        ];

        for (a, b) in &pairs {
            let emb_a = embedder.embed(a);
            let emb_b = embedder.embed(b);
            let sim = cosine_similarity(&emb_a, &emb_b);
            assert!(sim > 0.75, "synonym pair '{}' vs '{}' should have sim > 0.75, got {}", a, b, sim);
        }
    }

    #[test]
    fn test_semdup_recall_synthetic() {
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

    #[test]
    fn test_canonicalize() {
        assert_eq!(NgramEmbedder::canonicalize("auth"), "auth");
        assert_eq!(NgramEmbedder::canonicalize("authentication"), "auth");
        assert_eq!(NgramEmbedder::canonicalize("authorization"), "auth");
        assert_eq!(NgramEmbedder::canonicalize("db"), "db");
        assert_eq!(NgramEmbedder::canonicalize("database"), "db");
        assert_eq!(NgramEmbedder::canonicalize("fix"), "fix");
        assert_eq!(NgramEmbedder::canonicalize("repair"), "fix");
        assert_eq!(NgramEmbedder::canonicalize("resolve"), "fix");
        // Non-synonym passes through unchanged
        assert_eq!(NgramEmbedder::canonicalize("race"), "race");
        assert_eq!(NgramEmbedder::canonicalize("condition"), "condition");
    }
}
