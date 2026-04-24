//! Cost and duration predictor for Stitches
//!
//! Estimates cost p50/p90 and duration p50/p90 from similar historical Stitches.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Historical Stitch data for prediction
#[derive(Debug, Clone)]
pub struct HistoricalStitch {
    pub id: String,
    pub title: String,
    pub body: Option<String>,
    pub labels: Vec<String>,
    /// Adapter + model that claimed the first bead (e.g., "claude:opus")
    pub adapter_model: Option<String>,
    /// Cost in USD
    pub cost_usd: f64,
    /// Duration in seconds
    pub duration_seconds: i64,
    /// When the Stitch was closed
    pub closed_at: DateTime<Utc>,
}

/// Percentile estimates for a metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PercentileEstimate {
    /// 50th percentile (median)
    pub p50: f64,
    /// 90th percentile
    pub p90: f64,
    /// Number of data points
    pub count: usize,
}

/// Prediction result for a Stitch draft
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StitchPrediction {
    /// Cost estimate in USD
    pub cost: PercentileEstimate,
    /// Duration estimate in seconds
    pub duration: PercentileEstimate,
    /// Most likely adapter + model (by historical fit)
    /// NOTE: This is based on adapter + model, NOT strand (§8.4 non-goal)
    pub likely_adapter_model: Option<String>,
    /// Number of similar Stitches used for prediction
    pub similar_count: usize,
    /// Date range of historical data used
    pub data_range: DateRange,
}

/// Date range for historical data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: String,
    pub end: String,
}

/// Predict cost and duration for a Stitch draft
///
/// Uses historical Stitches from the last 90 days that are similar
/// to the draft (similarity threshold: 0.3).
pub fn predict_stitch(
    draft_title: &str,
    draft_body: Option<&str>,
    draft_labels: &[String],
    historical_stitches: Vec<HistoricalStitch>,
    cutoff_days: i64,
) -> Option<StitchPrediction> {
    if historical_stitches.is_empty() {
        return None;
    }

    let cutoff = Utc::now() - Duration::days(cutoff_days);

    // Filter by date range and compute similarity
    let recent_stitches: Vec<_> = historical_stitches
        .into_iter()
        .filter(|s| s.closed_at > cutoff)
        .collect();

    if recent_stitches.is_empty() {
        return None;
    }

    // Compute similarity for each
    let similarities: Vec<_> = recent_stitches
        .iter()
        .map(|s| {
            let sim = crate::similarity::combined_similarity(
                draft_title,
                draft_body,
                draft_labels,
                &s.title,
                s.body.as_deref(),
                &s.labels,
            );
            (s, sim.score)
        })
        .filter(|(_, score)| *score >= 0.3)
        .collect();

    if similarities.is_empty() {
        return None;
    }

    // Get data range
    let dates: Vec<_> = similarities.iter().map(|(s, _)| s.closed_at).collect();
    let start = dates.iter().min()?.to_rfc3339();
    let end = dates.iter().max()?.to_rfc3339();

    // Extract costs and durations
    let mut costs: Vec<f64> = similarities.iter().map(|(s, _)| s.cost_usd).collect();
    let mut durations: Vec<i64> = similarities.iter().map(|(s, _)| s.duration_seconds).collect();

    costs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    durations.sort();

    // Compute percentiles
    let cost = compute_percentiles(&costs);
    let duration = compute_percentiles_i64(&durations);

    // Find most likely adapter + model (by historical frequency)
    let likely_adapter_model = most_common_adapter_model(&similarities);

    Some(StitchPrediction {
        cost,
        duration,
        likely_adapter_model,
        similar_count: similarities.len(),
        data_range: DateRange { start, end },
    })
}

/// Compute p50 and p90 from a sorted float slice
fn compute_percentiles(sorted: &[f64]) -> PercentileEstimate {
    let count = sorted.len();
    if count == 0 {
        return PercentileEstimate {
            p50: 0.0,
            p90: 0.0,
            count: 0,
        };
    }

    let p50_idx = (count as f64 * 0.5).floor() as usize;
    let p90_idx = (count as f64 * 0.9).floor() as usize;

    PercentileEstimate {
        p50: sorted.get(p50_idx).copied().unwrap_or(0.0),
        p90: sorted.get(p90_idx).copied().unwrap_or(0.0),
        count,
    }
}

/// Compute p50 and p90 from a sorted i64 slice
fn compute_percentiles_i64(sorted: &[i64]) -> PercentileEstimate {
    let count = sorted.len();
    if count == 0 {
        return PercentileEstimate {
            p50: 0.0,
            p90: 0.0,
            count: 0,
        };
    }

    let p50_idx = (count as f64 * 0.5).floor() as usize;
    let p90_idx = (count as f64 * 0.9).floor() as usize;

    PercentileEstimate {
        p50: sorted.get(p50_idx).copied().unwrap_or(0) as f64,
        p90: sorted.get(p90_idx).copied().unwrap_or(0) as f64,
        count,
    }
}

/// Find the most likely adapter + model from similar Stitches.
///
/// Uses similarity-weighted scoring rather than raw frequency, so a
/// highly-similar Stitch weighted at 0.9 counts more than a weakly-similar
/// one at 0.3. This is "historical adapter-work-type fit" — NOT strand-based
/// (§8.4 non-goal).
fn most_common_adapter_model(
    similarities: &[(&HistoricalStitch, f64)],
) -> Option<String> {
    use std::collections::HashMap;

    let mut scores: HashMap<&str, f64> = HashMap::new();

    for (stitch, sim) in similarities {
        if let Some(ref am) = stitch.adapter_model {
            *scores.entry(am).or_insert(0.0) += sim;
        }
    }

    scores
        .into_iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(am, _)| am.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_stitch(
        id: &str,
        title: &str,
        adapter_model: Option<&str>,
        cost: f64,
        duration: i64,
        days_ago: i64,
    ) -> HistoricalStitch {
        HistoricalStitch {
            id: id.to_string(),
            title: title.to_string(),
            body: None,
            labels: vec![],
            adapter_model: adapter_model.map(String::from),
            cost_usd: cost,
            duration_seconds: duration,
            closed_at: Utc::now() - Duration::days(days_ago),
        }
    }

    #[test]
    fn test_predict_stitch_basic() {
        let historical = vec![
            make_stitch("st1", "fix bug", Some("claude:opus"), 1.5, 600, 10),
            make_stitch("st2", "fix bug", Some("claude:opus"), 2.0, 800, 5),
            make_stitch("st3", "fix crash", Some("codex:gpt4"), 3.0, 1200, 2),
        ];

        let result = predict_stitch("fix bug", None, &[], historical, 90);

        assert!(result.is_some());
        let pred = result.unwrap();

        // "fix bug" exact matches (2) + "fix crash" partial (1, just above 0.3 threshold
        // because both-empty labels contribute 0.1 to the combined score)
        assert_eq!(pred.similar_count, 3);
        assert!(pred.cost.p50 > 0.0);
        assert!(pred.duration.p50 > 0.0);
        // claude:opus has 2 exact matches weighted more than codex:gpt4's 1 partial
        assert_eq!(pred.likely_adapter_model, Some("claude:opus".to_string()));
    }

    #[test]
    fn test_predict_stitch_no_similar() {
        let historical = vec![
            make_stitch("st1", "add feature", Some("claude:opus"), 1.5, 600, 10),
        ];

        let result = predict_stitch("fix critical bug", None, &[], historical, 90);

        // "fix critical bug" is too different from "add feature" (< 0.3 similarity)
        assert!(result.is_none());
    }

    #[test]
    fn test_predict_stitch_empty_historical() {
        let result = predict_stitch("fix bug", None, &[], vec![], 90);
        assert!(result.is_none());
    }

    #[test]
    fn test_predict_stitch_old_data_filtered() {
        let historical = vec![
            make_stitch("st1", "fix bug", Some("claude:opus"), 1.5, 600, 100), // Too old
            make_stitch("st2", "fix bug", Some("claude:opus"), 2.0, 800, 5),   // Recent
        ];

        let result = predict_stitch("fix bug", None, &[], historical, 90);

        assert!(result.is_some());
        assert_eq!(result.unwrap().similar_count, 1);
    }

    #[test]
    fn test_percentiles_single_value() {
        let values = vec![5.0];
        let p = compute_percentiles(&values);
        assert_eq!(p.p50, 5.0);
        assert_eq!(p.p90, 5.0);
        assert_eq!(p.count, 1);
    }

    #[test]
    fn test_percentiles_multiple_values() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let p = compute_percentiles(&values);
        // p50 at index 5 (0.5 * 10 = 5)
        assert_eq!(p.p50, 6.0);
        // p90 at index 9 (0.9 * 10 = 9)
        assert_eq!(p.p90, 10.0);
    }

    #[test]
    fn test_most_common_adapter_model() {
        let historical = vec![
            make_stitch("st1", "fix bug", Some("claude:opus"), 1.0, 100, 10),
            make_stitch("st2", "fix bug", Some("claude:opus"), 1.0, 100, 10),
            make_stitch("st3", "fix bug", Some("codex:gpt4"), 1.0, 100, 10),
        ];

        let result = predict_stitch("fix bug", None, &[], historical, 90).unwrap();
        assert_eq!(result.likely_adapter_model, Some("claude:opus".to_string()));
    }

    #[test]
    fn test_most_common_adapter_model_none() {
        let historical = vec![
            make_stitch("st1", "fix bug", None, 1.0, 100, 10),
            make_stitch("st2", "fix bug", None, 1.0, 100, 10),
        ];

        let result = predict_stitch("fix bug", None, &[], historical, 90).unwrap();
        assert!(result.likely_adapter_model.is_none());
    }

    #[test]
    fn test_predict_with_labels() {
        let historical = [
            make_stitch("st1", "fix bug", Some("claude:opus"), 1.5, 600, 10),
        ];
        let mut stitch = historical[0].clone();
        stitch.labels = vec!["urgent".to_string()];

        let result = predict_stitch(
            "fix bug",
            None,
            &["urgent".to_string()],
            vec![stitch],
            90,
        );

        assert!(result.is_some());
    }
}
