//! In-memory vector index for semantic pre-dedup at draft time
//!
//! Maintains an index of open Stitches/beads across all projects.
//! Rebuilt on bead/Stitch events. Supports k-nearest-neighbor search
//! via brute-force cosine similarity (sufficient for the expected corpus
//! size of hundreds to low thousands of items).

use std::sync::RwLock;

use crate::embedding::{cosine_similarity, DedupMatch, Embedding, Embedder, IndexedItem, NgramEmbedder, EMBEDDING_DIM};

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
        Self {
            threshold: 0.82,
            max_results: 3,
        }
    }
}

/// An entry in the vector index
#[derive(Debug, Clone)]
struct IndexEntry {
    item: IndexedItem,
    embedding: Embedding,
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

/// Running statistics for false positive rate tracking
#[derive(Debug, Default, Clone)]
pub struct DedupStats {
    pub total_checks: u64,
    pub duplicates_found: u64,
    pub false_positives_reported: u64,
}

impl VectorIndex {
    /// Create a new empty vector index with default configuration
    pub fn new() -> Self {
        Self::with_config(DedupConfig::default())
    }

    /// Create a new vector index with custom configuration
    pub fn with_config(config: DedupConfig) -> Self {
        Self {
            embedder: Box::new(NgramEmbedder::new()),
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
                let text = format!("{} {}", item.title, item.kind);
                let embedding = self.embedder.embed(&text);
                IndexEntry { item, embedding }
            })
            .collect();
    }

    /// Add a single item to the index
    pub fn add(&mut self, item: IndexedItem) {
        let text = format!("{} {}", item.title, item.kind);
        let embedding = self.embedder.embed(&text);
        self.entries.push(IndexEntry { item, embedding });
    }

    /// Remove an item by ID
    pub fn remove(&mut self, id: &str) {
        self.entries.retain(|e| e.item.id != id);
    }

    /// Check a draft against all indexed items for potential duplicates
    ///
    /// Returns matches above the configured threshold, sorted by similarity descending.
    pub fn check_duplicate(&self, title: &str, description: Option<&str>) -> Vec<DedupMatch> {
        // Embed the draft text (title + description for richer matching)
        let text = match description {
            Some(desc) if !desc.is_empty() => format!("{} {}", title, desc),
            _ => title.to_string(),
        };
        let draft_embedding = self.embedder.embed(&text);

        let mut matches: Vec<DedupMatch> = self.entries
            .iter()
            .map(|entry| {
                let sim = cosine_similarity(&draft_embedding, &entry.embedding);
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
        }
    }

    /// Get the false positive rate
    pub fn false_positive_rate(&self) -> f64 {
        let stats = self.stats();
        if stats.duplicates_found == 0 {
            return 0.0;
        }
        stats.false_positives_reported as f64 / stats.duplicates_found as f64
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
            },
            IndexedItem {
                id: "bd-2".to_string(),
                project: "spaxel".to_string(),
                title: "Add dark mode toggle".to_string(),
                kind: "task".to_string(),
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
            },
            IndexedItem {
                id: "st-2".to_string(),
                project: "project-b".to_string(),
                title: "Add dark mode".to_string(),
                kind: "task".to_string(),
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
        // Create 20 pairs of similar titles across different projects
        let pairs = vec![
            ("Fix race condition in DB connection pool", "Fix DB connection pool race condition"),
            ("Implement user authentication with OAuth2", "Add OAuth2 user authentication"),
            ("Add rate limiting to API endpoints", "Implement API rate limiting"),
            ("Refactor database query builder", "Rewrite DB query builder"),
            ("Fix memory leak in worker process", "Repair worker memory leak"),
            ("Add pagination to list endpoints", "Implement list endpoint pagination"),
            ("Set up CI/CD pipeline for deploys", "Configure CI/CD deployment pipeline"),
            ("Implement caching layer with Redis", "Add Redis caching layer"),
            ("Fix timezone handling in scheduler", "Repair scheduler timezone bug"),
            ("Add WebSocket support for live updates", "Implement WebSocket live updates"),
        ];

        let mut index = VectorIndex::with_config(DedupConfig {
            threshold: 0.65,
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
            })
            .collect();

        index.rebuild(items);

        let mut caught = 0;
        let mut total = 0;
        for (i, (_, paraphrase)) in pairs.iter().enumerate() {
            total += 1;
            let matches = index.check_duplicate(paraphrase, None);
            if matches.iter().any(|m| m.item.id == format!("item-{}", i)) {
                caught += 1;
            }
        }

        let recall = caught as f64 / total as f64;
        assert!(
            recall >= 0.95,
            "synthetic cross-project recall should be >=95%, got {:.0}% ({}/{})",
            recall * 100.0,
            caught,
            total
        );
    }
}
