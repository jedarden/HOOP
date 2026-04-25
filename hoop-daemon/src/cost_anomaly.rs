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
        crate::metrics::metrics().hoop_cost_anomaly_alerts_total.inc();
    }

    CostAnomalyResult {
        is_anomaly,
        cost_usd: stitch.cost_usd,
        band,
        similar_stitch_ids: similar_ids,
    }
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
            .map(|i| make_stitch(&format!("n{i}"), "fix deployment bug", 1.0 + i as f64 * 0.1, 10 + i))
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

        assert!(result.is_anomaly, "3σ outlier should be flagged; cost={outlier_cost:.4}, band={:?}", result.band);
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
        let result = check_cost_anomaly(&stitch, &historical, DEFAULT_WINDOW_DAYS, DEFAULT_MIN_SIMILARITY);
        assert!(!result.is_anomaly);
    }

    #[test]
    fn test_no_alert_with_too_few_comparables() {
        // Only 2 historical Stitches — below MIN_COMPARABLE_STITCHES
        let historical: Vec<CostAnomalyStitch> = (0..2)
            .map(|i| make_stitch(&format!("h{i}"), "fix crash", 1.0, 5))
            .collect();

        let stitch = make_stitch("target", "fix crash", 999.0, 1);
        let result = check_cost_anomaly(&stitch, &historical, DEFAULT_WINDOW_DAYS, DEFAULT_MIN_SIMILARITY);

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
        let result = check_cost_anomaly(&stitch, &historical, DEFAULT_WINDOW_DAYS, DEFAULT_MIN_SIMILARITY);

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
        let result = check_cost_anomaly(&stitch, &historical, DEFAULT_WINDOW_DAYS, DEFAULT_MIN_SIMILARITY);

        // Dissimilar → not enough comparables → no anomaly
        assert!(!result.is_anomaly);
        assert!(result.band.is_none());
    }
}
