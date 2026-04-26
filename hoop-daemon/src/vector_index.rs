//! In-memory vector index for semantic pre-dedup at draft time
//!
//! Maintains an index of open Stitches/beads across all projects.
//! Rebuilt on bead/Stitch events. Supports k-nearest-neighbor search
//! via brute-force cosine similarity (sufficient for the expected corpus
//! size of hundreds to low thousands of items).
//!
//! ## Persistence (hoop-ttb.5.9.1)
//!
//! The index is persisted to fleet.db across daemon restarts. On startup,
//! the index loads from the database and only rebuilds if the embedding
//! model has changed. Model changes are detected via the model_name and
//! model_version columns in the vector_index table.

use anyhow::Result;
use rusqlite::{params, Connection};
use serde_json;
use std::sync::RwLock;

use crate::embedding::{
    cosine_similarity, jaccard_similarity, DedupMatch, Embedder, Embedding, IndexedItem,
    NgramEmbedder, TransformerEmbedder, EMBEDDING_DIM,
};
use crate::fleet::db_path;

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
    /// Current model name for change detection
    current_model_name: String,
    /// Current model version for change detection
    current_model_version: String,
    /// Rebuild progress for UI feedback (current, total)
    rebuild_progress: RwLock<(usize, usize)>,
}

impl std::fmt::Debug for VectorIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VectorIndex")
            .field("entries", &self.entries.len())
            .field("config", &self.config)
            .field("model_name", &self.current_model_name)
            .field("model_version", &self.current_model_version)
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
                tracing::warn!(
                    "Failed to initialize TransformerEmbedder: {}. Falling back to NgramEmbedder",
                    e
                );
                Box::new(NgramEmbedder::new())
            }
        };
        let (model_name, model_version) = embedder.model_info();
        Self {
            embedder,
            entries: Vec::new(),
            config,
            stats: RwLock::new(DedupStats::default()),
            current_model_name: model_name,
            current_model_version: model_version,
            rebuild_progress: RwLock::new((0, 0)),
        }
    }

    /// Rebuild the index from scratch with the given items
    ///
    /// Updates progress tracking during rebuild for UI feedback.
    pub fn rebuild(&mut self, items: Vec<IndexedItem>) {
        let total = items.len();
        *self.rebuild_progress.write().unwrap() = (0, total);

        self.entries = items
            .into_iter()
            .enumerate()
            .map(|(i, item)| {
                let text = match &item.description {
                    Some(desc) if !desc.is_empty() => format!("{} {}", item.title, desc),
                    _ => item.title.clone(),
                };
                let embedding = self.embedder.embed(&text);
                let tokens = self.embedder.canonical_tokens(&text);
                *self.rebuild_progress.write().unwrap() = (i + 1, total);
                IndexEntry {
                    item,
                    embedding,
                    text,
                    tokens,
                }
            })
            .collect();
    }

    /// Get current rebuild progress as (current, total)
    pub fn rebuild_progress(&self) -> (usize, usize) {
        *self.rebuild_progress.read().unwrap()
    }

    /// Add a single item to the index
    pub fn add(&mut self, item: IndexedItem) {
        let text = match &item.description {
            Some(desc) if !desc.is_empty() => format!("{} {}", item.title, desc),
            _ => item.title.clone(),
        };
        let embedding = self.embedder.embed(&text);
        let tokens = self.embedder.canonical_tokens(&text);
        self.entries.push(IndexEntry {
            item,
            embedding,
            text,
            tokens,
        });
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

        let mut matches: Vec<DedupMatch> = self
            .entries
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
            stats
                .false_positive_reports
                .retain(|r| r.timestamp > cutoff);
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
        let recent_fp = stats
            .false_positive_reports
            .iter()
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

    /// Get the current model name
    pub fn model_name(&self) -> String {
        self.current_model_name.clone()
    }

    /// Get the current model version
    pub fn model_version(&self) -> String {
        self.current_model_version.clone()
    }

    // -----------------------------------------------------------------------
    // Persistence methods (hoop-ttb.5.9.1)
    // -----------------------------------------------------------------------

    /// Load the vector index from fleet.db.
    ///
    /// Returns `Ok(true)` if the index was loaded successfully.
    /// Returns `Ok(false)` if the index needs to be rebuilt (model changed or DB empty).
    /// Returns `Err` if there was a database error.
    pub fn load_from_db(&mut self) -> Result<bool, String> {
        let db_path = db_path();
        if !db_path.exists() {
            return Ok(false);
        }

        let conn =
            Connection::open(&db_path).map_err(|e| format!("Failed to open fleet.db: {}", e))?;

        // Check if vector_index table exists (schema version >= 1.24.0)
        let table_exists: bool = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='vector_index'")
            .and_then(|mut stmt| stmt.query_row([], |row| row.get(0)))
            .unwrap_or(false);

        if !table_exists {
            return Ok(false);
        }

        // Check if model has changed
        let stored_model_name: Option<String> = conn
            .query_row(
                "SELECT value FROM vector_index_metadata WHERE key = 'current_model_name'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(None);

        let stored_model_version: Option<String> = conn
            .query_row(
                "SELECT value FROM vector_index_metadata WHERE key = 'current_model_version'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(None);

        let (current_name, current_version) = self.embedder.model_info();

        // Model changed - need full rebuild
        if stored_model_name.as_ref() != Some(&current_name)
            || stored_model_version.as_ref() != Some(&current_version)
        {
            tracing::info!(
                "Embedding model changed: {:?} -> {:?}, rebuilding index",
                stored_model_name,
                current_name
            );
            return Ok(false);
        }

        // Load entries from database
        let mut stmt = conn
            .prepare(
                r#"
                SELECT id, project, title, kind, description, embedding, tokens
                FROM vector_index
                ORDER BY indexed_at DESC
                "#,
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let entries_iter = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let project: String = row.get(1)?;
                let title: String = row.get(2)?;
                let kind: String = row.get(3)?;
                let description: Option<String> = row.get(4)?;
                let embedding_blob: Vec<u8> = row.get(5)?;
                let tokens_json: String = row.get(6)?;

                // Deserialize embedding from BLOB
                let embedding_vec: Vec<f32> = bincode::deserialize(&embedding_blob)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                let mut embedding = [0.0f32; EMBEDDING_DIM];
                let copy_len = EMBEDDING_DIM.min(embedding_vec.len());
                embedding[..copy_len].copy_from_slice(&embedding_vec[..copy_len]);

                // Deserialize tokens from JSON
                let tokens: Vec<String> = serde_json::from_str(&tokens_json)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                Ok(IndexedItem {
                    id,
                    project,
                    title,
                    kind,
                    description,
                })
                .map(move |item| (item, embedding, tokens))
            })
            .map_err(|e| format!("Failed to query vector_index: {}", e))?;

        let mut loaded_entries = Vec::new();
        for result in entries_iter {
            let (item, embedding, tokens) =
                result.map_err(|e| format!("Failed to read row: {}", e))?;
            let text = match &item.description {
                Some(desc) if !desc.is_empty() => format!("{} {}", item.title, desc),
                _ => item.title.clone(),
            };
            loaded_entries.push(IndexEntry {
                item,
                embedding,
                text,
                tokens,
            });
        }

        self.entries = loaded_entries;
        tracing::info!("Loaded {} entries from vector_index", self.entries.len());
        Ok(true)
    }

    /// Save the entire index to fleet.db.
    ///
    /// This is called after a full rebuild or when the model changes.
    pub fn save_to_db(&self) -> Result<(), String> {
        let db_path = db_path();
        let mut conn =
            Connection::open(&db_path).map_err(|e| format!("Failed to open fleet.db: {}", e))?;

        // Begin transaction for atomic write
        let tx = conn
            .transaction()
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;

        // Clear existing entries
        tx.execute("DELETE FROM vector_index", [])
            .map_err(|e| format!("Failed to clear vector_index: {}", e))?;

        let now = chrono::Utc::now().to_rfc3339();
        let (model_name, model_version) = self.embedder.model_info();

        // Insert all entries
        for entry in &self.entries {
            // Serialize embedding to BLOB
            let embedding_vec = entry.embedding.to_vec();
            let embedding_blob = bincode::serialize(&embedding_vec)
                .map_err(|e| format!("Failed to serialize embedding: {}", e))?;

            // Serialize tokens to JSON
            let tokens_json = serde_json::to_string(&entry.tokens)
                .map_err(|e| format!("Failed to serialize tokens: {}", e))?;

            tx.execute(
                r#"
                INSERT INTO vector_index (id, project, title, kind, description, embedding, tokens,
                                         model_name, model_version, indexed_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                params![
                    entry.item.id,
                    entry.item.project,
                    entry.item.title,
                    entry.item.kind,
                    entry.item.description,
                    embedding_blob,
                    tokens_json,
                    model_name,
                    model_version,
                    now,
                ],
            )
            .map_err(|e| format!("Failed to insert entry: {}", e))?;
        }

        // Update metadata
        tx.execute(
            "INSERT OR REPLACE INTO vector_index_metadata (key, value) VALUES (?, ?)",
            ["current_model_name", &model_name],
        )
        .map_err(|e| format!("Failed to update metadata: {}", e))?;

        tx.execute(
            "INSERT OR REPLACE INTO vector_index_metadata (key, value) VALUES (?, ?)",
            ["current_model_version", &model_version],
        )
        .map_err(|e| format!("Failed to update metadata: {}", e))?;

        tx.execute(
            "INSERT OR REPLACE INTO vector_index_metadata (key, value) VALUES (?, ?)",
            ["last_rebuild_at", &now],
        )
        .map_err(|e| format!("Failed to update metadata: {}", e))?;

        tx.commit()
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;

        tracing::info!("Saved {} entries to vector_index", self.entries.len());
        Ok(())
    }

    /// Add a single item to the index and persist it to the database.
    ///
    /// This is the incremental update path for Stitch create/update events.
    pub fn add_to_db(&mut self, item: IndexedItem) -> Result<(), String> {
        let text = match &item.description {
            Some(desc) if !desc.is_empty() => format!("{} {}", item.title, desc),
            _ => item.title.clone(),
        };
        let embedding = self.embedder.embed(&text);
        let tokens = self.embedder.canonical_tokens(&text);

        // Add to in-memory index
        self.entries.push(IndexEntry {
            item: item.clone(),
            embedding,
            text: text.clone(),
            tokens: tokens.clone(),
        });

        // Persist to database
        let db_path = db_path();
        let conn =
            Connection::open(&db_path).map_err(|e| format!("Failed to open fleet.db: {}", e))?;

        let now = chrono::Utc::now().to_rfc3339();
        let (model_name, model_version) = self.embedder.model_info();

        // Serialize embedding to BLOB
        let embedding_vec = embedding.to_vec();
        let embedding_blob = bincode::serialize(&embedding_vec)
            .map_err(|e| format!("Failed to serialize embedding: {}", e))?;

        // Serialize tokens to JSON
        let tokens_json = serde_json::to_string(&tokens)
            .map_err(|e| format!("Failed to serialize tokens: {}", e))?;

        conn.execute(
            r#"
            INSERT INTO vector_index (id, project, title, kind, description, embedding, tokens,
                                     model_name, model_version, indexed_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                item.id,
                item.project,
                item.title,
                item.kind,
                item.description,
                embedding_blob,
                tokens_json,
                model_name,
                model_version,
                now,
            ],
        )
        .map_err(|e| format!("Failed to insert entry: {}", e))?;

        Ok(())
    }

    /// Remove an item by ID from both memory and database.
    ///
    /// This is called when a Stitch is closed or deleted.
    pub fn remove_from_db(&mut self, id: &str) -> Result<(), String> {
        // Remove from in-memory index
        self.entries.retain(|e| e.item.id != id);

        // Remove from database
        let db_path = db_path();
        let conn =
            Connection::open(&db_path).map_err(|e| format!("Failed to open fleet.db: {}", e))?;

        conn.execute("DELETE FROM vector_index WHERE id = ?", params![id])
            .map_err(|e| format!("Failed to delete entry: {}", e))?;

        Ok(())
    }

    /// Rebuild the index from scratch and persist to database.
    ///
    /// This is called on startup when the model has changed or the database is empty.
    pub fn rebuild_and_persist(&mut self, items: Vec<IndexedItem>) -> Result<(), String> {
        self.rebuild(items);
        self.save_to_db()
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

    let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open fleet.db: {}", e))?;

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

    let projects_json = serde_json::to_string(&project_names).unwrap_or_else(|_| "[]".to_string());

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
        let matches = index.check_duplicate(
            "Implement OAuth2 auth flow",
            Some("Set up OAuth2 provider for user login"),
        );
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
        index.rebuild(vec![IndexedItem {
            id: "bd-1".to_string(),
            project: "hoop".to_string(),
            title: "Fix authentication race condition".to_string(),
            kind: "fix".to_string(),
            description: None,
        }]);

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
        index.rebuild(vec![IndexedItem {
            id: "bd-1".to_string(),
            project: "hoop".to_string(),
            title: "Fix auth bug".to_string(),
            kind: "fix".to_string(),
            description: None,
        }]);

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
            (
                "Fix race condition in DB connection pool",
                "Fix database connection pool race condition",
            ),
            (
                "Implement user authentication with OAuth2",
                "Add OAuth2 user authentication",
            ),
            (
                "Add rate limiting to API endpoints",
                "Implement API endpoint rate limiting",
            ),
            (
                "Refactor database query builder",
                "Rewrite database query builder",
            ),
            (
                "Fix memory leak in worker process",
                "Repair worker process memory leak",
            ),
            (
                "Add pagination to list endpoints",
                "Implement pagination for list endpoints",
            ),
            (
                "Set up CI/CD pipeline for deploys",
                "Configure continuous deployment pipeline",
            ),
            (
                "Implement caching layer with Redis",
                "Add Redis caching for performance",
            ),
            (
                "Fix timezone handling in scheduler",
                "Repair scheduler timezone handling",
            ),
            (
                "Add WebSocket support for live updates",
                "Implement WebSocket for real-time updates",
            ),
            (
                "Fix auth race condition bug",
                "Fix authentication race condition bug",
            ),
            (
                "Setup config for production deploy",
                "Configure production deployment settings",
            ),
            (
                "Add user model CRUD operations",
                "Implement CRUD for user model",
            ),
            (
                "Refactor ORM mapping layer",
                "Restructure ORM layer mappings",
            ),
            (
                "Fix SSL certificate validation error",
                "Repair SSL certificate validation",
            ),
            (
                "Implement DNS resolution caching",
                "Add DNS caching for resolution",
            ),
            (
                "Add VPN tunnel support",
                "Implement VPN tunnel functionality",
            ),
            (
                "Fix async task queue deadlock",
                "Repair async queue deadlock issue",
            ),
            (
                "Implement RPC error handling",
                "Add error handling for RPC calls",
            ),
            (
                "Add HTML sanitizer for user input",
                "Implement HTML sanitization for input",
            ),
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
                    eprintln!(
                        "  -> Best match: {} (sim: {:.2})",
                        best.item.title, best.similarity
                    );
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
                description: Some(
                    "The database connection pool is exhausting under high concurrency".to_string(),
                ),
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

    #[test]
    fn test_performance_10k_items_p95_under_50ms() {
        // Generate 10k synthetic items representing a realistic workload
        let items: Vec<IndexedItem> = (0..10_000)
            .map(|i| {
                // Use a variety of realistic titles to simulate actual work
                let titles = vec![
                    "Fix authentication bug in login flow",
                    "Add rate limiting to API endpoints",
                    "Implement OAuth2 provider",
                    "Refactor database query builder",
                    "Set up CI/CD pipeline",
                    "Add pagination to list endpoints",
                    "Fix memory leak in worker process",
                    "Implement caching layer with Redis",
                    "Add WebSocket support for live updates",
                    "Fix timezone handling in scheduler",
                    "Setup config for production deploy",
                    "Add user model CRUD operations",
                    "Refactor ORM mapping layer",
                    "Fix SSL certificate validation error",
                    "Implement DNS resolution caching",
                    "Add VPN tunnel support",
                    "Fix async task queue deadlock",
                    "Implement RPC error handling",
                    "Add HTML sanitizer for user input",
                    "Refactor authentication middleware",
                    "Add support for multi-factor authentication",
                    "Fix connection pool exhaustion",
                    "Implement request logging middleware",
                    "Add feature flag support",
                ];
                let title = titles[i % titles.len()].to_string();
                IndexedItem {
                    id: format!("bd-{}", i),
                    project: format!("project-{}", i % 20), // 20 projects
                    title: format!("{} {}", title, i),      // Make each unique
                    kind: "task".to_string(),
                    description: Some(format!("Description for item {}", i)),
                }
            })
            .collect();

        let mut index = VectorIndex::new();
        index.rebuild(items);

        // Run 100 queries to measure p95 latency
        let query_titles = vec![
            "Fix auth bug in login flow",
            "Add API rate limiting",
            "Implement OAuth2 authentication",
            "Refactor query builder",
            "Set up CI pipeline",
        ];

        let mut latencies = Vec::new();
        for _ in 0..100 {
            let query = query_titles[latencies.len() % query_titles.len()];
            let start = std::time::Instant::now();
            let _matches = index.check_duplicate(query, None);
            let elapsed = start.elapsed().as_millis();
            latencies.push(elapsed);
        }

        // Calculate p95
        latencies.sort();
        let p95_index = (latencies.len() as f64 * 0.95) as usize;
        let p95 = latencies[p95_index];

        // Assert p95 < 50ms
        assert!(
            p95 < 50,
            "p95 latency {}ms exceeds 50ms requirement for 10k item index",
            p95
        );

        // Also log p50 and p99 for visibility
        let p50 = latencies[latencies.len() / 2];
        let p99_index = (latencies.len() as f64 * 0.99) as usize;
        let p99 = latencies[p99_index];
        eprintln!(
            "Performance metrics for 10k item index: p50={}ms, p95={}ms, p99={}ms",
            p50, p95, p99
        );
    }
}
