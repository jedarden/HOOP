//! Similarity matching for finding historical Stitches
//!
//! Provides lexical similarity on title, body, and labels.
//! Phase 5 will add embedding-based similarity.

use std::collections::HashSet;

/// Tokenize text into lowercase word tokens, removing punctuation
pub fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split_whitespace()
        .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Compute Jaccard similarity between two token sets
fn jaccard_similarity(set1: &HashSet<String>, set2: &HashSet<String>) -> f64 {
    if set1.is_empty() && set2.is_empty() {
        return 1.0;
    }
    if set1.is_empty() || set2.is_empty() {
        return 0.0;
    }

    let intersection: HashSet<_> = set1.intersection(set2).cloned().collect();
    let union: HashSet<_> = set1.union(set2).cloned().collect();

    intersection.len() as f64 / union.len() as f64
}

/// Similarity score between two texts
#[derive(Debug, Clone)]
pub struct TextSimilarity {
    /// Jaccard similarity (0-1)
    pub jaccard: f64,
    /// Token overlap count
    pub overlap_count: usize,
}

/// Compute similarity between two texts
pub fn text_similarity(text1: &str, text2: &str) -> TextSimilarity {
    let tokens1: HashSet<_> = tokenize(text1).into_iter().collect();
    let tokens2: HashSet<_> = tokenize(text2).into_iter().collect();

    let overlap_count = tokens1.intersection(&tokens2).count();
    let jaccard = jaccard_similarity(&tokens1, &tokens2);

    TextSimilarity {
        jaccard,
        overlap_count,
    }
}

/// Combined similarity score for title + body + labels
#[derive(Debug, Clone)]
pub struct CombinedSimilarity {
    /// Overall similarity (0-1)
    pub score: f64,
    /// Title similarity
    pub title: TextSimilarity,
    /// Body/description similarity
    pub body: Option<TextSimilarity>,
    /// Label overlap count
    pub label_overlap: usize,
}

/// Compute combined similarity between a draft and historical Stitch
pub fn combined_similarity(
    draft_title: &str,
    draft_body: Option<&str>,
    draft_labels: &[String],
    historical_title: &str,
    historical_body: Option<&str>,
    historical_labels: &[String],
) -> CombinedSimilarity {
    // Title similarity is weighted highest
    let title_sim = text_similarity(draft_title, historical_title);

    // Body similarity if both have descriptions
    let body_sim = match (draft_body, historical_body) {
        (Some(db), Some(hb)) if !db.is_empty() && !hb.is_empty() => {
            Some(text_similarity(db, hb))
        }
        _ => None,
    };

    // Label overlap
    let draft_labels_set: HashSet<_> = draft_labels.iter().map(|l| l.to_lowercase()).collect();
    let historical_labels_set: HashSet<_> = historical_labels.iter().map(|l| l.to_lowercase()).collect();
    let label_overlap = draft_labels_set.intersection(&historical_labels_set).count();

    // Combined score: 60% title, 30% body, 10% labels
    let title_weight = 0.6;
    let body_weight = 0.3;
    let label_weight = 0.1;

    let body_score = body_sim.as_ref().map(|b| b.jaccard).unwrap_or(0.0);
    let label_score = if draft_labels_set.is_empty() && historical_labels_set.is_empty() {
        1.0 // Both having no labels is a match
    } else if draft_labels_set.is_empty() || historical_labels_set.is_empty() {
        0.0
    } else {
        label_overlap as f64 / draft_labels_set.len().max(historical_labels_set.len()) as f64
    };

    let score = title_sim.jaccard * title_weight
        + body_score * body_weight
        + label_score * label_weight;

    CombinedSimilarity {
        score,
        title: title_sim,
        body: body_sim,
        label_overlap,
    }
}

/// A historical Stitch with similarity information
#[derive(Debug, Clone)]
pub struct SimilarStitch {
    pub id: String,
    pub title: String,
    pub similarity: CombinedSimilarity,
}

/// Find similar Stitches from a list of historical Stitches
///
/// Returns Stitches sorted by similarity score (descending),
/// filtered by minimum similarity threshold.
pub fn find_similar_stitches(
    draft_title: &str,
    draft_body: Option<&str>,
    draft_labels: &[String],
    historical_stitches: impl IntoIterator<Item = (String, String, Option<String>, Vec<String>)>,
    min_similarity: f64,
    max_results: usize,
) -> Vec<SimilarStitch> {
    let mut results: Vec<SimilarStitch> = historical_stitches
        .into_iter()
        .map(|(id, title, body, labels)| {
            let similarity = combined_similarity(
                draft_title,
                draft_body,
                draft_labels,
                &title,
                body.as_deref(),
                &labels,
            );

            SimilarStitch {
                id,
                title,
                similarity,
            }
        })
        .filter(|s| s.similarity.score >= min_similarity)
        .collect();

    // Sort by score descending
    results.sort_by(|a, b| {
        b.similarity
            .score
            .partial_cmp(&a.similarity.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Limit results
    results.truncate(max_results);
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let tokens = tokenize("Hello world foo bar");
        assert_eq!(tokens, vec!["hello", "world", "foo", "bar"]);
    }

    #[test]
    fn test_tokenize_punctuation() {
        let tokens = tokenize("Hello, world! Foo: bar.");
        assert_eq!(tokens, vec!["hello", "world", "foo", "bar"]);
    }

    #[test]
    fn test_tokenize_empty() {
        let tokens = tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_jaccard_identical() {
        let set1: HashSet<_> = vec!["a", "b", "c"].into_iter().map(String::from).collect();
        let set2: HashSet<_> = vec!["a", "b", "c"].into_iter().map(String::from).collect();
        assert!((jaccard_similarity(&set1, &set2) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_jaccard_no_overlap() {
        let set1: HashSet<_> = vec!["a", "b"].into_iter().map(String::from).collect();
        let set2: HashSet<_> = vec!["c", "d"].into_iter().map(String::from).collect();
        assert_eq!(jaccard_similarity(&set1, &set2), 0.0);
    }

    #[test]
    fn test_jaccard_partial_overlap() {
        let set1: HashSet<_> = vec!["a", "b", "c"].into_iter().map(String::from).collect();
        let set2: HashSet<_> = vec!["b", "c", "d"].into_iter().map(String::from).collect();
        let sim = jaccard_similarity(&set1, &set2);
        // intersection = {b, c} = 2, union = {a, b, c, d} = 4
        assert!((sim - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_text_similarity_identical() {
        let sim = text_similarity("fix the bug", "fix the bug");
        assert!((sim.jaccard - 1.0).abs() < f64::EPSILON);
        assert_eq!(sim.overlap_count, 3);
    }

    #[test]
    fn test_text_similarity_partial() {
        let sim = text_similarity("fix the bug", "fix the crash");
        assert!(sim.jaccard > 0.0 && sim.jaccard < 1.0);
        assert_eq!(sim.overlap_count, 2); // "fix", "the"
    }

    #[test]
    fn test_combined_similarity_title_only() {
        let sim = combined_similarity(
            "fix bug",
            None,
            &[],
            "fix bug",
            Some("detailed description"),
            &["urgent".to_string()],
        );
        assert!((sim.score - 0.6).abs() < 0.01); // Only title matches
        assert_eq!(sim.label_overlap, 0);
        assert!(sim.body.is_none());
    }

    #[test]
    fn test_combined_similarity_with_labels() {
        let sim = combined_similarity(
            "fix bug",
            None,
            &["urgent".to_string()],
            "fix bug",
            None,
            &["urgent".to_string(), "backend".to_string()],
        );
        // Title: 0.6 + Labels: 0.1 * (1/max(1,2)) = 0.05 → total 0.65
        assert!((sim.score - 0.65).abs() < 0.01);
        assert_eq!(sim.label_overlap, 1);
    }

    #[test]
    fn test_find_similar_stitches() {
        let historical = vec![
            ("st1".to_string(), "fix crash bug".to_string(), None, vec!["bug".to_string()]),
            ("st2".to_string(), "add feature".to_string(), None, vec![]),
            ("st3".to_string(), "fix bug".to_string(), Some("detailed".to_string()), vec![]),
        ];

        let results = find_similar_stitches(
            "fix bug",
            None,
            &[],
            historical,
            0.3,
            10,
        );

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "st3"); // Closest match
        assert!(results[0].similarity.score > results[1].similarity.score);
    }

    #[test]
    fn test_find_similar_stitches_filters_by_threshold() {
        let historical = vec![
            ("st1".to_string(), "totally different".to_string(), None, vec![]),
            ("st2".to_string(), "fix bug".to_string(), None, vec![]),
        ];

        let results = find_similar_stitches(
            "fix bug",
            None,
            &[],
            historical,
            0.5, // High threshold
            10,
        );

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "st2");
    }

    #[test]
    fn test_find_similar_stitches_limits_results() {
        let historical = vec![
            ("st1".to_string(), "fix bug a".to_string(), None, vec![]),
            ("st2".to_string(), "fix bug b".to_string(), None, vec![]),
            ("st3".to_string(), "fix bug c".to_string(), None, vec![]),
        ];

        let results = find_similar_stitches(
            "fix bug",
            None,
            &[],
            historical,
            0.0, // No threshold
            2, // Limit to 2
        );

        assert_eq!(results.len(), 2);
    }
}
