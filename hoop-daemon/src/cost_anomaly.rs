//! Cost-anomaly detector for Stitches (§6 Phase 2 marquee #4)
//!
//! For each closed Stitch, computes whether its cost is an outlier relative
//! to historically similar Stitches in a 90-day window.
//!
//! Similarity (v0.2 — lexical):
//!   60% lexical title Jaccard + 25% body-length proximity + 15% attachment-count proximity
//!
//! Anomaly threshold: cost > mean + 2σ across similar Stitches.
//!
//! Phase 3 will replace the lexical component with embedding-based similarity.

use chrono::{DateTime, Duration, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::similarity::text_similarity;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Stitch data required for anomaly detection
#[derive(Debug, Clone)]
pub struct CostAnomalyStitch {
    pub id: String,
    pub title: String,
    /// Raw body text (used for length measurement, not content matching)
    pub body: Option<String>,
    pub cost_usd: f64,
    pub closed_at: DateTime<Utc>,
    pub attachment_count: usize,
    /// Adapter that created this stitch (e.g., "claude", "openai")
    pub adapter: String,
}

impl CostAnomalyStitch {
    /// Body length in bytes (0 when absent)
    pub fn body_len(&self) -> usize {
        self.body.as_deref().map(|b| b.len()).unwrap_or(0)
    }
}

/// Statistical band computed from similar historical Stitches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBand {
    /// Mean cost in USD
    pub mean_usd: f64,
    /// Population standard deviation of cost
    pub std_dev_usd: f64,
    /// Upper 2σ threshold: mean + 2 × std_dev
    pub upper_2sigma_usd: f64,
    /// Number of similar Stitches used
    pub similar_count: usize,
    /// Similarity threshold applied
    pub min_similarity: f64,
    /// Look-back window in days
    pub window_days: i64,
}

/// Result of a cost-anomaly check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostAnomalyResult {
    /// True when cost > mean + 2σ among similar Stitches
    pub is_anomaly: bool,
    /// The Stitch's actual cost in USD
    pub cost_usd: f64,
    /// Statistical band used for the check (None when too few comparables)
    pub band: Option<CostBand>,
    /// Computed similarity score vs each comparable (0–1, for diagnostics)
    pub similar_stitch_ids: Vec<String>,
    /// Matching fix patterns with recommended fixes
    pub matching_patterns: Vec<MatchingPattern>,
}

/// A matching fix pattern with similarity score and recommended fix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchingPattern {
    /// Pattern ID
    pub pattern_id: String,
    /// Pattern name
    pub pattern_name: String,
    /// Similarity score (0.0 to 1.0)
    pub similarity: f32,
    /// Recommended fix template in markdown
    pub recommended_fix_template_md: String,
    /// Example source stitches where this pattern was applied
    pub example_source_stitches: Vec<String>,
}

// ---------------------------------------------------------------------------
// Similarity (v0.2 — lexical)
// ---------------------------------------------------------------------------

/// Compute v0.2 similarity between two Stitches.
///
/// Components:
///   - 60 % lexical Jaccard on lowercased title tokens
///   - 25 % body-length proximity  (1 – |len_a – len_b| / max(len_a, len_b))
///   - 15 % attachment-count proximity (1 – |cnt_a – cnt_b| / max(cnt_a, cnt_b))
pub fn stitch_similarity(a: &CostAnomalyStitch, b: &CostAnomalyStitch) -> f64 {
    let title_score = text_similarity(&a.title, &b.title).jaccard;

    let body_score = {
        let la = a.body_len();
        let lb = b.body_len();
        let denom = la.max(lb);
        if denom == 0 {
            1.0 // both empty → equal
        } else {
            1.0 - (la as f64 - lb as f64).abs() / denom as f64
        }
    };

    let attach_score = {
        let ca = a.attachment_count;
        let cb = b.attachment_count;
        let denom = ca.max(cb);
        if denom == 0 {
            1.0 // both zero → equal
        } else {
            1.0 - (ca as f64 - cb as f64).abs() / denom as f64
        }
    };

    0.60 * title_score + 0.25 * body_score + 0.15 * attach_score
}

// ---------------------------------------------------------------------------
// Statistical band
// ---------------------------------------------------------------------------

/// Compute the cost band (mean ± 2σ) from a slice of cost values.
///
/// Returns `None` when fewer than `min_count` samples are present;
/// a σ estimate from a very small sample is too noisy to be useful.
pub fn compute_band(costs: &[f64], min_count: usize) -> Option<CostBand> {
    if costs.len() < min_count {
        return None;
    }

    let n = costs.len() as f64;
    let mean = costs.iter().sum::<f64>() / n;
    let variance = costs.iter().map(|c| (c - mean).powi(2)).sum::<f64>() / n;
    let std_dev = variance.sqrt();

    Some(CostBand {
        mean_usd: mean,
        std_dev_usd: std_dev,
        upper_2sigma_usd: mean + 2.0 * std_dev,
        similar_count: costs.len(),
        min_similarity: 0.0, // filled in by caller
        window_days: 0,      // filled in by caller
    })
}

// ---------------------------------------------------------------------------
// Main entry point
// ---------------------------------------------------------------------------

/// Minimum number of similar Stitches required before the detector fires.
///
/// Below this threshold the σ estimate is too noisy; no anomaly is reported.
pub const MIN_COMPARABLE_STITCHES: usize = 3;

/// Default similarity threshold for "similar enough" Stitches.
///
/// With weights 60/25/15 (title/body-len/attach), two Stitches with
/// completely disjoint titles but no body or attachments score 0.40 —
/// so the threshold must be > 0.40 to exclude them when they're unrelated.
/// 0.45 requires at least a small title-token overlap (~8% Jaccard) on top
/// of the body-len / attachment neutral components.
pub const DEFAULT_MIN_SIMILARITY: f64 = 0.45;

/// Default look-back window in days
pub const DEFAULT_WINDOW_DAYS: i64 = 90;

/// Check whether `stitch` is a cost anomaly relative to `historical`.
///
/// 1. Filter historical Stitches to those closed within `window_days`.
/// 2. Score similarity of each to `stitch`; keep those ≥ `min_similarity`.
/// 3. Compute mean + 2σ of their costs.
/// 4. Flag `stitch` if its cost exceeds that threshold.
/// 5. Increment `hoop_cost_anomaly_alerts_total` if anomalous.
pub fn check_cost_anomaly(
    stitch: &CostAnomalyStitch,
    historical: &[CostAnomalyStitch],
    window_days: i64,
    min_similarity: f64,
) -> CostAnomalyResult {
    let cutoff = Utc::now() - Duration::days(window_days);

    // Find similar Stitches within the window (exclude the stitch itself)
    let mut similar_ids: Vec<String> = Vec::new();
    let mut costs: Vec<f64> = Vec::new();

    for h in historical {
        if h.id == stitch.id {
            continue;
        }
        if h.closed_at < cutoff {
            continue;
        }
        let sim = stitch_similarity(stitch, h);
        if sim >= min_similarity {
            similar_ids.push(h.id.clone());
            costs.push(h.cost_usd);
        }
    }

    let mut band = compute_band(&costs, MIN_COMPARABLE_STITCHES);
    if let Some(ref mut b) = band {
        b.min_similarity = min_similarity;
        b.window_days = window_days;
    }

    let is_anomaly = match &band {
        Some(b) => stitch.cost_usd > b.upper_2sigma_usd,
        None => false,
    };

    if is_anomaly {
        crate::metrics::metrics()
            .hoop_cost_anomaly_alerts_total
            .inc();
    }

    // Find matching fix patterns based on stitch signature
    let matching_patterns = find_matching_patterns(stitch);

    CostAnomalyResult {
        is_anomaly,
        cost_usd: stitch.cost_usd,
        band,
        similar_stitch_ids: similar_ids,
        matching_patterns,
    }
}

// ---------------------------------------------------------------------------
// Integration: check on Stitch close
// ---------------------------------------------------------------------------

/// Check for cost anomalies when a Stitch is closed.
///
/// This is called from the bead close event handler in lib.rs.
/// Returns Ok(false) if the Stitch is not fully closed yet.
pub fn check_on_stitch_close(
    stitch_id: &str,
    sender: Option<&tokio::sync::broadcast::Sender<crate::ws::CostAnomalyAlertData>>,
) -> anyhow::Result<bool> {
    let db_path = std::path::PathBuf::from(dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from(".")))
        .join(".hoop")
        .join("fleet.db");

    if !db_path.exists() {
        return Ok(false);
    }

    let conn = Connection::open(&db_path)?;

    // Check if this Stitch is "closed" (no recent activity)
    let (last_activity_at, project): (String, String) = match conn.query_row(
        "SELECT last_activity_at, project FROM stitches WHERE id = ?1",
        params![stitch_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ) {
        Ok(data) => data,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Ok(false);
        }
        Err(e) => return Err(e.into()),
    };

    let last_activity_dt = chrono::DateTime::parse_from_rfc3339(&last_activity_at)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now());

    let now = chrono::Utc::now();
    let inactive_seconds = (now - last_activity_dt).num_seconds().max(0);

    // Stitch must be inactive for at least 5 minutes to be considered closed
    const STITCH_CLOSED_THRESHOLD_SECONDS: i64 = 300;
    if inactive_seconds < STITCH_CLOSED_THRESHOLD_SECONDS {
        return Ok(false);
    }

    // Load the current Stitch's data
    let current = load_stitch_for_anomaly(&conn, stitch_id)?;

    // Record cost per stitch metric (§16.7)
    crate::metrics::metrics()
        .hoop_cost_per_stitch_usd
        .observe(&[&current.adapter], current.cost_usd);

    // Load historical Stitches within the 90-day window
    let historical = load_historical_stitches(&conn, DEFAULT_WINDOW_DAYS)?;

    // Run the anomaly check
    let result = check_cost_anomaly(
        &current,
        &historical,
        DEFAULT_WINDOW_DAYS,
        DEFAULT_MIN_SIMILARITY,
    );

    // Broadcast alert if anomalous
    if result.is_anomaly {
        tracing::warn!(
            stitch_id = %stitch_id,
            stitch_title = %current.title,
            cost_usd = result.cost_usd,
            mean_usd = result.band.as_ref().map(|b| b.mean_usd),
            std_dev_usd = result.band.as_ref().map(|b| b.std_dev_usd),
            upper_2sigma_usd = result.band.as_ref().map(|b| b.upper_2sigma_usd),
            similar_count = result.band.as_ref().map(|b| b.similar_count),
            similar_stitch_ids = ?result.similar_stitch_ids,
            "Cost anomaly detected: Stitch cost exceeds 2σ band"
        );

        // Broadcast the alert via WebSocket if sender is available
        if let Some(sender) = sender {
            let band = result.band.unwrap();
            let closest_pattern = result.matching_patterns.first().map(|p| {
                crate::ws::ClosestPatternMatch {
                    pattern_id: p.pattern_id.clone(),
                    pattern_name: p.pattern_name.clone(),
                    similarity: p.similarity as f64,
                    recommended_fix_template_md: p.recommended_fix_template_md.clone(),
                }
            });

            let alert = crate::ws::CostAnomalyAlertData {
                alert_id: uuid::Uuid::new_v4().to_string(),
                stitch_id: stitch_id.to_string(),
                stitch_title: current.title.clone(),
                project,
                cost_usd: result.cost_usd,
                band: crate::ws::CostBand {
                    mean_usd: band.mean_usd,
                    std_dev_usd: band.std_dev_usd,
                    upper_2sigma_usd: band.upper_2sigma_usd,
                    similar_count: band.similar_count,
                    min_similarity: band.min_similarity,
                    window_days: band.window_days,
                },
                closest_pattern,
                detected_at: Utc::now().to_rfc3339(),
            };

            let _ = sender.send(alert);
        }
    }

    Ok(true)
}

/// Load a Stitch's data for anomaly detection.
fn load_stitch_for_anomaly(conn: &Connection, stitch_id: &str) -> anyhow::Result<CostAnomalyStitch> {
    // Load the Stitch row
    let (title, last_activity_at, adapter): (String, String, Option<String>) = conn.query_row(
        "SELECT title, last_activity_at, created_by_adapter FROM stitches WHERE id = ?1",
        params![stitch_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;

    // Parse timestamp
    let closed_at = chrono::DateTime::parse_from_rfc3339(&last_activity_at)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now());

    // Load the body from the first user message
    let body: Option<String> = conn
        .query_row(
            r#"
            SELECT sm.content
            FROM stitch_messages sm
            WHERE sm.stitch_id = ?1 AND sm.role = 'user'
            ORDER BY sm.ts ASC LIMIT 1
            "#,
            params![stitch_id],
            |row| row.get(0),
        )
        .ok();

    // Load attachments count
    let attachments_path: Option<String> = conn
        .query_row(
            "SELECT attachments_path FROM stitches WHERE id = ?1",
            params![stitch_id],
            |row| row.get(0),
        )
        .ok();
    let attachment_count = attachments_path
        .as_deref()
        .map(count_attachments)
        .unwrap_or(0);

    // Calculate cost from total tokens
    let total_tokens: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(tokens), 0) FROM stitch_messages WHERE stitch_id = ?1",
            params![stitch_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let cost_usd = (total_tokens as f64) * 30.0 / 1_000_000.0;

    Ok(CostAnomalyStitch {
        id: stitch_id.to_string(),
        title,
        body,
        cost_usd,
        closed_at,
        attachment_count,
        adapter: adapter.unwrap_or_else(|| "unknown".to_string()),
    })
}

/// Load historical Stitches within the given look-back window.
fn load_historical_stitches(
    conn: &Connection,
    window_days: i64,
) -> anyhow::Result<Vec<CostAnomalyStitch>> {
    let cutoff = chrono::Utc::now() - chrono::Duration::days(window_days);
    let cutoff_str = cutoff.to_rfc3339();

    let mut stmt = conn.prepare(
        r#"
        SELECT
            s.id,
            s.title,
            s.last_activity_at,
            (SELECT sm.content FROM stitch_messages sm
             WHERE sm.stitch_id = s.id AND sm.role = 'user'
             ORDER BY sm.ts ASC LIMIT 1) AS body,
            s.attachments_path,
            (SELECT COALESCE(SUM(sm.tokens), 0) FROM stitch_messages sm
             WHERE sm.stitch_id = s.id) AS total_tokens,
            s.created_by_adapter
        FROM stitches s
        WHERE s.last_activity_at >= ?1
        ORDER BY s.last_activity_at DESC
        "#,
    )?;

    let rows = stmt.query_map(params![cutoff_str], |row| {
        let id: String = row.get(0)?;
        let title: String = row.get(1)?;
        let last_activity_at: String = row.get(2)?;
        let body: Option<String> = row.get(3).unwrap_or(None);
        let attachments_path: Option<String> = row.get(4).unwrap_or(None);
        let total_tokens: i64 = row.get(5).unwrap_or(0);
        let adapter: Option<String> = row.get(6).unwrap_or(None);

        let closed_at = chrono::DateTime::parse_from_rfc3339(&last_activity_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now());

        let cost_usd = (total_tokens as f64) * 30.0 / 1_000_000.0;

        let attachment_count = attachments_path
            .as_deref()
            .map(count_attachments)
            .unwrap_or(0);

        Ok(CostAnomalyStitch {
            id,
            title,
            body,
            cost_usd,
            closed_at,
            attachment_count,
            adapter: adapter.unwrap_or_else(|| "unknown".to_string()),
        })
    })?;

    let mut stitches = Vec::new();
    for row in rows {
        stitches.push(row?);
    }

    Ok(stitches)
}

/// Count the number of files in an attachments directory.
fn count_attachments(attachments_path: &str) -> usize {
    let path = std::path::Path::new(attachments_path);
    if !path.exists() {
        return 0;
    }

    path.read_dir()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_file())
                .count()
        })
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Pattern matching integration
// ---------------------------------------------------------------------------

/// Find matching fix patterns for a cost-anomaly stitch.
///
/// Computes a signature vector from the stitch's title and body,
/// then queries the fix_patterns service for matches above threshold.
fn find_matching_patterns(stitch: &CostAnomalyStitch) -> Vec<MatchingPattern> {
    // Compute signature vector from stitch (simple hash-based approach)
    let signature = compute_stitch_signature(stitch);

    // Query fix_patterns service for matches
    match crate::fix_patterns::FixPatternService::match_by_signature(&signature, 0.5, 5) {
        Ok(matches) => matches
            .into_iter()
            .map(|m| MatchingPattern {
                pattern_id: m.pattern.id.clone(),
                pattern_name: m.pattern.name.clone(),
                similarity: m.similarity,
                recommended_fix_template_md: m.pattern.recommended_fix_template_md.clone(),
                example_source_stitches: m.pattern.example_source_stitches.clone(),
            })
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Compute a signature vector for a stitch for pattern matching.
///
/// Uses a simple hash-based approach to convert the stitch title
/// and body into a fixed-length vector for similarity matching.
fn compute_stitch_signature(stitch: &CostAnomalyStitch) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    const VECTOR_SIZE: usize = 8;

    let mut hasher = DefaultHasher::new();
    stitch.title.hash(&mut hasher);
    if let Some(ref body) = stitch.body {
        body.hash(&mut hasher);
    }

    let hash = hasher.finish();
    let mut signature = Vec::with_capacity(VECTOR_SIZE);

    // Convert hash to a vector of floats in [0, 1]
    for i in 0..VECTOR_SIZE {
        let byte = (hash >> (i * 8)) as u8;
        signature.push(byte as f32 / 255.0);
    }

    signature
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_stitch(id: &str, title: &str, cost: f64, days_ago: i64) -> CostAnomalyStitch {
        CostAnomalyStitch {
            id: id.to_string(),
            title: title.to_string(),
            body: None,
            cost_usd: cost,
            closed_at: Utc::now() - Duration::days(days_ago),
            attachment_count: 0,
            adapter: "claude".to_string(),
        }
    }

    fn make_stitch_with_body(
        id: &str,
        title: &str,
        body: &str,
        attachments: usize,
        cost: f64,
        days_ago: i64,
    ) -> CostAnomalyStitch {
        CostAnomalyStitch {
            id: id.to_string(),
            title: title.to_string(),
            body: Some(body.to_string()),
            cost_usd: cost,
            closed_at: Utc::now() - Duration::days(days_ago),
            attachment_count: attachments,
            adapter: "claude".to_string(),
        }
    }

    // ── similarity tests ────────────────────────────────────────────────────

    #[test]
    fn test_similarity_identical_title() {
        let a = make_stitch("a", "fix authentication bug", 1.0, 5);
        let b = make_stitch("b", "fix authentication bug", 1.0, 5);
        let s = stitch_similarity(&a, &b);
        // Title = 1.0 (60%) + body = 1.0 (25%) + attach = 1.0 (15%) = 1.0
        assert!((s - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_similarity_disjoint_titles() {
        let a = make_stitch("a", "fix authentication bug", 1.0, 5);
        let b = make_stitch("b", "add payment feature", 1.0, 5);
        let s = stitch_similarity(&a, &b);
        // Jaccard("fix auth bug", "add payment feature") = 0 → 0*0.60 + 0.25 + 0.15 = 0.40
        assert!(s < 0.5);
    }

    #[test]
    fn test_similarity_body_length_component() {
        let a = make_stitch_with_body("a", "fix bug", "short", 0, 1.0, 5);
        let b = make_stitch_with_body("b", "fix bug", "a".repeat(500).as_str(), 0, 1.0, 5);
        let s = stitch_similarity(&a, &b);
        // Title matches perfectly (0.60) but body lengths differ greatly
        assert!(s < 0.9);
    }

    #[test]
    fn test_similarity_attachment_count_component() {
        let a = make_stitch_with_body("a", "fix bug", "", 0, 1.0, 5);
        let b = make_stitch_with_body("b", "fix bug", "", 10, 1.0, 5);
        let s = stitch_similarity(&a, &b);
        // Title: 1.0 (60%) + body: 1.0 (25%) + attach: 0.0 (15%) = 0.85
        assert!((s - 0.85).abs() < 1e-9);
    }

    // ── band computation tests ───────────────────────────────────────────────

    #[test]
    fn test_compute_band_basic() {
        let costs = vec![1.0, 2.0, 3.0];
        let band = compute_band(&costs, 3).unwrap();
        // mean = 2.0, variance = ((1-2)² + (2-2)² + (3-2)²) / 3 = 2/3
        // std_dev = sqrt(2/3) ≈ 0.8165
        assert!((band.mean_usd - 2.0).abs() < 1e-9);
        assert!((band.std_dev_usd - (2.0_f64 / 3.0).sqrt()).abs() < 1e-9);
        assert!((band.upper_2sigma_usd - (2.0 + 2.0 * (2.0_f64 / 3.0).sqrt())).abs() < 1e-9);
    }

    #[test]
    fn test_compute_band_too_few() {
        let costs = vec![1.0, 2.0]; // only 2, need 3
        assert!(compute_band(&costs, 3).is_none());
    }

    #[test]
    fn test_compute_band_uniform() {
        let costs = vec![5.0; 10];
        let band = compute_band(&costs, 3).unwrap();
        assert!((band.mean_usd - 5.0).abs() < 1e-9);
        assert!(band.std_dev_usd < 1e-9);
        assert!((band.upper_2sigma_usd - 5.0).abs() < 1e-9);
    }

    // ── end-to-end anomaly detection tests ──────────────────────────────────

    /// Build 10 normal-cost Stitches (≈$1–$2) and one 3σ outlier.
    /// The outlier must be detected; all normal Stitches must not be flagged.
    #[test]
    fn test_3sigma_case_flagged() {
        // 10 normal Stitches: costs 1.0, 1.1, …, 1.9 USD
        let normal: Vec<CostAnomalyStitch> = (0..10)
            .map(|i| {
                make_stitch(
                    &format!("n{i}"),
                    "fix deployment bug",
                    1.0 + i as f64 * 0.1,
                    10 + i,
                )
            })
            .collect();

        // Compute population stats for the normal group
        let costs: Vec<f64> = normal.iter().map(|s| s.cost_usd).collect();
        let mean = costs.iter().sum::<f64>() / costs.len() as f64;
        let variance = costs.iter().map(|c| (c - mean).powi(2)).sum::<f64>() / costs.len() as f64;
        let std_dev = variance.sqrt();

        // Outlier at 3σ above mean
        let outlier_cost = mean + 3.0 * std_dev;
        let outlier = make_stitch("outlier", "fix deployment bug", outlier_cost, 1);

        let historical = normal.clone();
        // Historical does not include the outlier (it just closed)

        let result = check_cost_anomaly(
            &outlier,
            &historical,
            DEFAULT_WINDOW_DAYS,
            DEFAULT_MIN_SIMILARITY,
        );

        assert!(
            result.is_anomaly,
            "3σ outlier should be flagged; cost={outlier_cost:.4}, band={:?}",
            result.band
        );
        assert_eq!(result.cost_usd, outlier_cost);
        assert!(result.band.is_some());

        // Normal Stitches should not trigger anomalies
        for stitch in &normal {
            let hist: Vec<CostAnomalyStitch> = historical
                .iter()
                .filter(|h| h.id != stitch.id)
                .cloned()
                .collect();
            let r = check_cost_anomaly(stitch, &hist, DEFAULT_WINDOW_DAYS, DEFAULT_MIN_SIMILARITY);
            assert!(
                !r.is_anomaly,
                "Normal stitch {} (cost={}) should not be flagged; band={:?}",
                stitch.id, stitch.cost_usd, r.band
            );
        }
    }

    #[test]
    fn test_no_anomaly_when_below_2sigma() {
        let historical: Vec<CostAnomalyStitch> = (0..5)
            .map(|i| make_stitch(&format!("h{i}"), "refactor module", 2.0, 10 + i))
            .collect();

        // Stitch at exactly mean (2.0) — not anomalous
        let stitch = make_stitch("target", "refactor module", 2.0, 1);
        let result = check_cost_anomaly(
            &stitch,
            &historical,
            DEFAULT_WINDOW_DAYS,
            DEFAULT_MIN_SIMILARITY,
        );
        assert!(!result.is_anomaly);
    }

    #[test]
    fn test_no_alert_with_too_few_comparables() {
        // Only 2 historical Stitches — below MIN_COMPARABLE_STITCHES
        let historical: Vec<CostAnomalyStitch> = (0..2)
            .map(|i| make_stitch(&format!("h{i}"), "fix crash", 1.0, 5))
            .collect();

        let stitch = make_stitch("target", "fix crash", 999.0, 1);
        let result = check_cost_anomaly(
            &stitch,
            &historical,
            DEFAULT_WINDOW_DAYS,
            DEFAULT_MIN_SIMILARITY,
        );

        // No band → no anomaly (too few data points)
        assert!(!result.is_anomaly);
        assert!(result.band.is_none());
    }

    #[test]
    fn test_old_stitches_excluded_from_window() {
        // 5 Stitches all more than 90 days old
        let historical: Vec<CostAnomalyStitch> = (0..5)
            .map(|i| make_stitch(&format!("h{i}"), "deploy service", 1.0, 100 + i))
            .collect();

        let stitch = make_stitch("target", "deploy service", 999.0, 1);
        let result = check_cost_anomaly(
            &stitch,
            &historical,
            DEFAULT_WINDOW_DAYS,
            DEFAULT_MIN_SIMILARITY,
        );

        // Historical too old → no band → no anomaly
        assert!(!result.is_anomaly);
        assert!(result.band.is_none());
    }

    #[test]
    fn test_dissimilar_stitches_not_counted() {
        // Historical Stitches with completely different titles
        let historical: Vec<CostAnomalyStitch> = (0..5)
            .map(|i| make_stitch(&format!("h{i}"), "write documentation", 1.0, i + 1))
            .collect();

        // High cost stitch with unrelated title
        let stitch = make_stitch("target", "setup kubernetes cluster", 999.0, 1);
        let result = check_cost_anomaly(
            &stitch,
            &historical,
            DEFAULT_WINDOW_DAYS,
            DEFAULT_MIN_SIMILARITY,
        );

        // Dissimilar → not enough comparables → no anomaly
        assert!(!result.is_anomaly);
        assert!(result.band.is_none());
    }

    #[test]
    fn test_band_with_zero_variance() {
        let costs = vec![5.0; 100];
        let band = compute_band(&costs, 3).unwrap();
        assert!((band.mean_usd - 5.0).abs() < 1e-9);
        assert!(band.std_dev_usd < 1e-9);
        assert!((band.upper_2sigma_usd - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_similarity_with_no_body_or_attachments() {
        let a = make_stitch("a", "fix bug", 1.0, 5);
        let b = make_stitch("b", "fix bug", 1.0, 5);
        let s = stitch_similarity(&a, &b);
        // Only title matches (60%)
        assert!((s - 0.60).abs() < 1e-9);
    }

    #[test]
    fn test_excludes_self_from_historical() {
        let historical = vec![
            make_stitch("self", "fix bug", 1.0, 5),
            make_stitch("other", "fix bug", 1.0, 5),
        ];

        let stitch = make_stitch("self", "fix bug", 1.0, 5);
        let result = check_cost_anomaly(
            &stitch,
            &historical,
            DEFAULT_WINDOW_DAYS,
            DEFAULT_MIN_SIMILARITY,
        );

        // Should exclude self from historical
        assert_eq!(result.similar_stitch_ids.len(), 1);
        assert_eq!(result.similar_stitch_ids[0], "other");
    }

    #[test]
    fn test_performance_100_historical_stitches() {
        let historical: Vec<CostAnomalyStitch> = (0..100)
            .map(|i| make_stitch(&format!("h{i}"), "fix bug", 1.0 + i as f64 * 0.01, i))
            .collect();

        let stitch = make_stitch("target", "fix bug", 1.5, 1);

        let start = std::time::Instant::now();
        let _result = check_cost_anomaly(
            &stitch,
            &historical,
            DEFAULT_WINDOW_DAYS,
            DEFAULT_MIN_SIMILARITY,
        );
        let elapsed = start.elapsed();

        assert!(
            elapsed.as_millis() < 100,
            "Checking 100 historical stitches should take < 100ms, took {}ms",
            elapsed.as_millis()
        );
    }
}
