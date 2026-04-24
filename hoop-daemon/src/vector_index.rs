//! In-memory vector index for semantic pre-dedup at draft time
//!
//! Maintains an index of open Stitches/beads across all projects.
//! Rebuilt on bead/Stitch events. Supports k-nearest-neighbor search
//! via brute-force cosine similarity (sufficient for the expected corpus
//! size of hundreds to low thousands of items).

use std::sync::RwLock;

use crate::embedding::{cosine_similarity, jaccard_similarity, DedupMatch, Embedding, Embedder, IndexedItem, NgramEmbedder, TransformerEmbedder};

/// Configuration for the dedup check
#[derive(Debug, Clone)]
pub struct DedupConfig {
    /// Minimum cosine similarity to report as a potential duplicate (default: 0.82)
    pub threshold: f64,
    /// Maximum number of matches to return
    pub max_results: usize,
}

impl Default for DedupConfig {
    fn default() -> Self {
        let threshold = std::env::var("HOOP_DEDUP_THRESHOLD")
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.82);
        Self {
            threshold,
            max_results: 3,
        }
    }
}

/// An entry in the vector index
#[derive(Debug, Clone)]
struct IndexEntry {
    item: IndexedItem,
    embedding: Embedding,
    /// Original text for combined similarity computation
    text: String,
    /// Canonical tokens for Jaccard similarity
    tokens: Vec<String>,
}

/// In-memory vector index for open stitches/beads across all projects
pub struct VectorIndex {
    embedder: Box<dyn Embedder>,
    entries: Vec<IndexEntry>,
    config: DedupConfig,
    /// Track false positive/negative counts for threshold tuning
    stats: RwLock<DedupStats>,
}

impl std::fmt::Debug for VectorIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VectorIndex")
            .field("entries", &self.entries.len())
            .field("config", &self.config)
            .finish()
    }
}

/// A timestamped false positive report
#[derive(Debug, Clone)]
pub struct FalsePositiveReport {
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Running statistics for false positive rate tracking
#[derive(Debug, Default, Clone)]
pub struct DedupStats {
    pub total_checks: u64,
    pub duplicates_found: u64,
    pub false_positives_reported: u64,
    /// Timestamped false positive reports for 30-day rolling calculation
    pub false_positive_reports: Vec<FalsePositiveReport>,
}

impl Default for VectorIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl VectorIndex {
    /// Create a new empty vector index with default configuration
    ///
    /// Uses TransformerEmbedder (BGE-small-en-v1.5) with automatic fallback
    /// to NgramEmbedder if model loading fails.
    pub fn new() -> Self {
        Self::with_config(DedupConfig::default())
    }

    /// Create a new vector index with custom configuration
    pub fn with_config(config: DedupConfig) -> Self {
        let embedder: Box<dyn Embedder> = match TransformerEmbedder::new() {
            Ok(model) => {
                tracing::info!("Using TransformerEmbedder (BGE-small-en-v1.5) for semantic dedup");
                Box::new(model)
            }
            Err(e) => {
                tracing::warn!("Failed to initialize TransformerEmbedder: {}. Falling back to NgramEmbedder", e);
                Box::new(NgramEmbedder::new())
            }
        };
        Self {
            embedder,
            entries: Vec::new(),
            config,
            stats: RwLock::new(DedupStats::default()),
        }
    }

    /// Rebuild the index from scratch with the given items
    pub fn rebuild(&mut self, items: Vec<IndexedItem>) {
        self.entries = items
            .into_iter()
            .map(|item| {
                let text = match &item.description {
                    Some(desc) if !desc.is_empty() => format!("{} {}", item.title, desc),
                    _ => item.title.clone(),
                };
                let embedding = self.embedder.embed(&text);
                let tokens = self.embedder.canonical_tokens(&text);
                IndexEntry { item, embedding, text, tokens }
            })
            .collect();
    }

    /// Add a single item to the index
    pub fn add(&mut self, item: IndexedItem) {
        let text = match &item.description {
            Some(desc) if !desc.is_empty() => format!("{} {}", item.title, desc),
            _ => item.title.clone(),
        };
        let embedding = self.embedder.embed(&text);
        let tokens = self.embedder.canonical_tokens(&text);
        self.entries.push(IndexEntry { item, embedding, text, tokens });
    }

    /// Remove an item by ID
    pub fn remove(&mut self, id: &str) {
        self.entries.retain(|e| e.item.id != id);
    }

    /// Check a draft against all indexed items for potential duplicates
    ///
    /// Returns matches above the configured threshold, sorted by similarity descending.
    /// Uses adaptive combined similarity: max(cosine, Jaccard) with boost when both agree.
    pub fn check_duplicate(&self, title: &str, description: Option<&str>) -> Vec<DedupMatch> {
        // Embed the draft text (title + description for richer matching)
        let text = match description {
            Some(desc) if !desc.is_empty() => format!("{} {}", title, desc),
            _ => title.to_string(),
        };
        let draft_embedding = self.embedder.embed(&text);
        let draft_tokens = self.embedder.canonical_tokens(&text);

        let mut matches: Vec<DedupMatch> = self.entries
            .iter()
            .map(|entry| {
                // Cosine similarity from embeddings (captures morphological + lexical similarity)
                let cosine = cosine_similarity(&draft_embedding, &entry.embedding);
                // Jaccard similarity from tokens (captures word overlap, order-independent)
                let jaccard = jaccard_similarity(&draft_tokens, &entry.tokens);
                // Adaptive: use max, but boost when both metrics agree (both > 0.65)
                let base = cosine.max(jaccard);
                let boost = if cosine > 0.65 && jaccard > 0.65 {
                    0.05 * cosine.min(jaccard) // Small boost when both are reasonably strong
                } else {
                    0.0
                };
                let sim = base + boost;
                DedupMatch {
                    item: entry.item.clone(),
                    similarity: sim,
                }
            })
            .filter(|m| m.similarity >= self.config.threshold)
            .collect();

        matches.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        matches.truncate(self.config.max_results);

        // Update stats
        if let Ok(mut stats) = self.stats.write() {
            stats.total_checks += 1;
            if !matches.is_empty() {
                stats.duplicates_found += 1;
            }
        }

        matches
    }

    /// Get current dedup statistics
    pub fn stats(&self) -> DedupStats {
        self.stats.read().unwrap().clone()
    }

    /// Report a false positive (user dismissed a dedup match as incorrect)
    pub fn report_false_positive(&self) {
        if let Ok(mut stats) = self.stats.write() {
            stats.false_positives_reported += 1;
            stats.false_positive_reports.push(FalsePositiveReport {
                timestamp: chrono::Utc::now(),
            });
            // Prune old reports (>30 days) to keep memory bounded
            let cutoff = chrono::Utc::now() - chrono::Duration::days(30);
            stats.false_positive_reports.retain(|r| r.timestamp > cutoff);
        }
    }

    /// Get the false positive rate (cumulative, all-time)
    pub fn false_positive_rate(&self) -> f64 {
        let stats = self.stats();
        if stats.duplicates_found == 0 {
            return 0.0;
        }
        stats.false_positives_reported as f64 / stats.duplicates_found as f64
    }

    /// Get the false positive rate over the last 30 days
    pub fn false_positive_rate_30d(&self) -> f64 {
        let stats = self.stats();
        if stats.duplicates_found == 0 {
            return 0.0;
        }
        // Count false positives in the last 30 days
        let cutoff = chrono::Utc::now() - chrono::Duration::days(30);
        let recent_fp = stats.false_positive_reports.iter()
            .filter(|r| r.timestamp > cutoff)
            .count() as f64;
        recent_fp / stats.duplicates_found as f64
    }

    /// Get the number of items in the index
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the index is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the configured threshold
    pub fn threshold(&self) -> f64 {
        self.config.threshold
    }

    /// Update the threshold
    pub fn set_threshold(&mut self, threshold: f64) {
        self.config.threshold = threshold;
    }
}

/// Build a vector index from all open beads and stitches across all projects
pub fn build_index_from_state(
    beads: &[crate::Bead],
    projects: &[crate::ws::ProjectCardData],
) -> Vec<IndexedItem> {
    let mut items = Vec::new();

    // Index open beads
    for bead in beads {
        if bead.status == crate::BeadStatus::Open {
            items.push(IndexedItem {
                id: bead.id.clone(),
                project: String::new(), // beads don't carry project in the struct
                title: bead.title.clone(),
                kind: format!("{:?}", bead.issue_type).to_lowercase(),
                description: bead.description.clone(),
            });
        }
    }

    // Index open stitches from fleet.db
    if let Ok(stitch_items) = load_open_stitches(projects) {
        items.extend(stitch_items);
    }

    items
}

/// Load open stitches from fleet.db for all known projects
fn load_open_stitches(projects: &[crate::ws::ProjectCardData]) -> Result<Vec<IndexedItem>, String> {
    use rusqlite::Connection;

    let db_path = crate::fleet::db_path();
    if !db_path.exists() {
        return Ok(vec![]);
    }

    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open fleet.db: {}", e))?;

    let project_names: Vec<&str> = projects.iter().map(|p| p.name.as_str()).collect();

    let mut stmt = conn
        .prepare(
            r#"
            SELECT s.id, s.project, s.title, s.kind
            FROM stitches s
            WHERE s.project IN (SELECT value FROM json_each(?1))
            ORDER BY s.last_activity_at DESC
            "#,
        )
        .map_err(|e| format!("Failed to prepare stitch query: {}", e))?;

    let projects_json = serde_json::to_string(&project_names)
        .unwrap_or_else(|_| "[]".to_string());

    let items: Vec<IndexedItem> = stmt
        .query_map(rusqlite::params![projects_json], |row| {
            let id: String = row.get(0)?;
            let project: String = row.get(1)?;
            let title: String = row.get(2)?;
            let kind: String = row.get(3)?;
            Ok(IndexedItem {
                id,
                project,
                title,
                kind,
                description: None,
            })
        })
        .map_err(|e| format!("Failed to query stitches: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_rebuild_and_search() {
        let mut index = VectorIndex::new();
        index.rebuild(vec![
            IndexedItem {
                id: "bd-1".to_string(),
                project: "hoop".to_string(),
                title: "Fix authentication race condition".to_string(),
                kind: "fix".to_string(),
                description: None,
            },
            IndexedItem {
                id: "bd-2".to_string(),
                project: "spaxel".to_string(),
                title: "Add dark mode toggle".to_string(),
                kind: "task".to_string(),
                description: None,
            },
        ]);

        let matches = index.check_duplicate("Fix auth race condition bug", None);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].item.id, "bd-1");
        assert!(matches[0].similarity > 0.82);
    }

    #[test]
    fn test_cross_project_duplicate() {
        let mut index = VectorIndex::new();
        index.rebuild(vec![
            IndexedItem {
                id: "st-1".to_string(),
                project: "project-a".to_string(),
                title: "Implement OAuth2 authentication flow".to_string(),
                kind: "feature".to_string(),
                description: None,
            },
            IndexedItem {
                id: "st-2".to_string(),
                project: "project-b".to_string(),
                title: "Add dark mode".to_string(),
                kind: "task".to_string(),
                description: None,
            },
        ]);

        // Cross-project duplicate should be caught
        let matches = index.check_duplicate("Implement OAuth2 auth flow", Some("Set up OAuth2 provider for user login"));
        assert!(!matches.is_empty(), "should detect cross-project duplicate");
        assert_eq!(matches[0].item.id, "st-1");
        assert_eq!(matches[0].item.project, "project-a");
    }

    #[test]
    fn test_no_match_below_threshold() {
        let mut index = VectorIndex::with_config(DedupConfig {
            threshold: 0.82,
            max_results: 3,
        });
        index.rebuild(vec![
            IndexedItem {
                id: "bd-1".to_string(),
                project: "hoop".to_string(),
                title: "Fix authentication race condition".to_string(),
                kind: "fix".to_string(),
                description: None,
            },
        ]);

        // Completely unrelated text should not match
        let matches = index.check_duplicate("Update README documentation", None);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_add_and_remove() {
        let mut index = VectorIndex::new();
        index.add(IndexedItem {
            id: "bd-1".to_string(),
            project: "hoop".to_string(),
            title: "Test item".to_string(),
            kind: "task".to_string(),
            description: None,
        });
        assert_eq!(index.len(), 1);

        index.remove("bd-1");
        assert!(index.is_empty());
    }

    #[test]
    fn test_stats_tracking() {
        let mut index = VectorIndex::new();
        index.rebuild(vec![
            IndexedItem {
                id: "bd-1".to_string(),
                project: "hoop".to_string(),
                title: "Fix auth bug".to_string(),
                kind: "fix".to_string(),
                description: None,
            },
        ]);

        // Check a duplicate
        let _ = index.check_duplicate("Fix auth bug", None);
        let stats = index.stats();
        assert_eq!(stats.total_checks, 1);
        assert_eq!(stats.duplicates_found, 1);

        // Check a non-duplicate
        let _ = index.check_duplicate("Unrelated task completely different", None);
        let stats = index.stats();
        assert_eq!(stats.total_checks, 2);
    }

    #[test]
    fn test_synthetic_cross_project_recall() {
        // 20 pairs of semantically similar titles across projects.
        // Pairs are designed to test: reordering, abbreviation expansion,
        // synonym matching, and realistic paraphrase detection.
        // These represent realistic duplicate scenarios where the same task
        // is described with similar vocabulary (not extreme paraphrasing).
        let pairs = vec![
            ("Fix race condition in DB connection pool", "Fix database connection pool race condition"),
            ("Implement user authentication with OAuth2", "Add OAuth2 user authentication"),
            ("Add rate limiting to API endpoints", "Implement API endpoint rate limiting"),
            ("Refactor database query builder", "Rewrite database query builder"),
            ("Fix memory leak in worker process", "Repair worker process memory leak"),
            ("Add pagination to list endpoints", "Implement pagination for list endpoints"),
            ("Set up CI/CD pipeline for deploys", "Configure continuous deployment pipeline"),
            ("Implement caching layer with Redis", "Add Redis caching for performance"),
            ("Fix timezone handling in scheduler", "Repair scheduler timezone handling"),
            ("Add WebSocket support for live updates", "Implement WebSocket for real-time updates"),
            ("Fix auth race condition bug", "Fix authentication race condition bug"),
            ("Setup config for production deploy", "Configure production deployment settings"),
            ("Add user model CRUD operations", "Implement CRUD for user model"),
            ("Refactor ORM mapping layer", "Restructure ORM layer mappings"),
            ("Fix SSL certificate validation error", "Repair SSL certificate validation"),
            ("Implement DNS resolution caching", "Add DNS caching for resolution"),
            ("Add VPN tunnel support", "Implement VPN tunnel functionality"),
            ("Fix async task queue deadlock", "Repair async queue deadlock issue"),
            ("Implement RPC error handling", "Add error handling for RPC calls"),
            ("Add HTML sanitizer for user input", "Implement HTML sanitization for input"),
        ];

        // Test at production threshold (0.82)
        let mut index = VectorIndex::with_config(DedupConfig {
            threshold: 0.82,
            max_results: 3,
        });

        let items: Vec<IndexedItem> = pairs
            .iter()
            .enumerate()
            .map(|(i, (title, _))| IndexedItem {
                id: format!("item-{}", i),
                project: format!("project-{}", i),
                title: title.to_string(),
                kind: "task".to_string(),
                description: None,
            })
            .collect();

        index.rebuild(items);

        let mut caught = 0;
        let mut total = 0;
        for (i, (original, paraphrase)) in pairs.iter().enumerate() {
            total += 1;
            let matches = index.check_duplicate(paraphrase, None);
            let found = matches.iter().any(|m| m.item.id == format!("item-{}", i));
            if found {
                caught += 1;
            } else {
                eprintln!("MISSED: '{}' vs '{}'", original, paraphrase);
                if let Some(best) = matches.first() {
                    eprintln!("  -> Best match: {} (sim: {:.2})", best.item.title, best.similarity);
                } else {
                    eprintln!("  -> No matches at all");
                }
            }
        }

        let recall = caught as f64 / total as f64;
        assert!(
            recall >= 0.95,
            "synthetic cross-project recall should be >=95% at threshold 0.82, got {:.0}% ({}/{})",
            recall * 100.0,
            caught,
            total
        );
    }

    #[test]
    fn test_description_enhances_matching() {
        let mut index = VectorIndex::with_config(DedupConfig {
            threshold: 0.75,
            max_results: 3,
        });
        index.rebuild(vec![
            IndexedItem {
                id: "bd-1".to_string(),
                project: "hoop".to_string(),
                title: "Fix connection issue".to_string(),
                kind: "fix".to_string(),
                description: Some("The database connection pool is exhausting under high concurrency".to_string()),
            },
            IndexedItem {
                id: "bd-2".to_string(),
                project: "hoop".to_string(),
                title: "Fix connection issue".to_string(),
                kind: "fix".to_string(),
                description: None,
            },
        ]);

        // Query with database-specific context should match the one with description
        let matches = index.check_duplicate("Fix DB connection pool exhaustion", None);
        assert!(!matches.is_empty());
        // The item with the richer description should match better
        assert_eq!(matches[0].item.id, "bd-1");
    }
}
