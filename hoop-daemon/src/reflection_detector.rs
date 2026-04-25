//! Reflection detector — scans closed operator Stitches for repeated patterns.
//!
//! After each closed operator Stitch, scans recent operator Stitches (configurable
//! window, default 30 days) for:
//! - Repeated corrections (same correction, 3+ instances)
//! - Repeated preferences (same preference stated 3+ times)
//! - Repeated negatives ("don't do X", 3+ times)
//! - Repeated approvals of non-obvious choices (same acceptance signal, 3+ times)
//!
//! Detected patterns become `status=proposed` rows in the Reflection Ledger.
//!
//! Plan reference: §6 Phase 5 marquee #12

use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::fleet;

/// Configuration for the reflection detector.
#[derive(Debug, Clone)]
pub struct ReflectionDetectorConfig {
    /// How far back to scan for operator Stitches (days).
    pub scan_window_days: i64,
    /// Minimum occurrences to consider a pattern (default: 3).
    pub min_occurrences: usize,
    /// Minimum Jaccard similarity for grouping user messages (default: 0.45).
    pub similarity_threshold: f64,
    /// Maximum messages per Stitch to analyze (avoids processing huge transcripts).
    pub max_messages_per_stitch: usize,
    /// Maximum Stitches to scan in one run (performance cap).
    pub max_stitches_to_scan: usize,
}

impl Default for ReflectionDetectorConfig {
    fn default() -> Self {
        Self {
            scan_window_days: 30,
            min_occurrences: 3,
            similarity_threshold: 0.45,
            max_messages_per_stitch: 200,
            max_stitches_to_scan: 500,
        }
    }
}

/// A user message extracted from an operator Stitch, with metadata for grouping.
#[derive(Debug, Clone)]
struct UserMessage {
    /// Normalized content (lowercased, punctuation stripped).
    normalized: String,
    /// The original stitch_id this message came from.
    stitch_id: String,
    /// The pattern category (correction, preference, negative, approval).
    category: PatternCategory,
}

/// Categories of repeatable patterns in operator messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PatternCategory {
    Correction,
    Preference,
    Negative,
    Approval,
}

impl PatternCategory {
    fn as_str(&self) -> &'static str {
        match self {
            PatternCategory::Correction => "correction",
            PatternCategory::Preference => "preference",
            PatternCategory::Negative => "negative",
            PatternCategory::Approval => "approval",
        }
    }
}

/// A detected pattern ready to be proposed to the Reflection Ledger.
#[derive(Debug, Clone)]
pub struct DetectedPattern {
    /// Human-readable summary of the repeated pattern.
    pub rule: String,
    /// Why this rule was proposed.
    pub reason: String,
    /// Stitch IDs that contributed to this pattern.
    pub source_stitches: Vec<String>,
    /// Pattern category.
    pub category: PatternCategory,
}

/// A lightweight row returned by fleet queries for operator stitch messages.
#[derive(Debug, Clone)]
pub struct OperatorMessage {
    pub stitch_id: String,
    pub role: String,
    pub content: String,
}

/// Run the reflection detector: scan recent operator Stitches and propose patterns.
///
/// This is the main entry point. It:
/// 1. Fetches recent operator stitch messages from fleet.db
/// 2. Classifies user messages into pattern categories
/// 3. Groups similar messages together
/// 4. For groups with 3+ occurrences, proposes a reflection entry
///
/// Returns the number of new patterns proposed.
pub fn run_detection(config: &ReflectionDetectorConfig) -> anyhow::Result<usize> {
    let cutoff = chrono::Utc::now() - chrono::Duration::days(config.scan_window_days);
    let cutoff_str = cutoff.to_rfc3339();

    // 1. Fetch recent operator stitch messages
    let messages = fleet::query_operator_stitch_messages(
        &cutoff_str,
        config.max_stitches_to_scan,
        config.max_messages_per_stitch,
    )?;

    if messages.is_empty() {
        debug!("Reflection detector: no operator stitch messages found");
        return Ok(0);
    }

    info!(
        "Reflection detector: scanning {} messages from operator Stitches",
        messages.len()
    );

    // 2. Extract and classify user messages
    let user_messages = extract_user_messages(&messages);
    if user_messages.is_empty() {
        debug!("Reflection detector: no classifiable user messages found");
        return Ok(0);
    }

    // 3. Group similar messages within each category
    let patterns = group_and_detect(user_messages, config);

    if patterns.is_empty() {
        debug!("Reflection detector: no repeated patterns detected");
        return Ok(0);
    }

    info!("Reflection detector: found {} candidate patterns", patterns.len());

    // 4. Insert proposed patterns into the Reflection Ledger
    let mut proposed = 0;
    for pattern in &patterns {
        match fleet::propose_reflection_entry(
            &pattern.rule,
            &pattern.reason,
            "global",
            &pattern.source_stitches,
        ) {
            Ok(id) => {
                info!(
                    "Reflection detector: proposed {} pattern: {} (id={})",
                    pattern.category.as_str(),
                    &pattern.rule[..pattern.rule.len().min(80)],
                    id
                );
                proposed += 1;
            }
            Err(e) => {
                warn!("Reflection detector: failed to propose pattern: {}", e);
            }
        }
    }

    Ok(proposed)
}

/// Extract classifiable user messages from raw stitch messages.
fn extract_user_messages(messages: &[OperatorMessage]) -> Vec<UserMessage> {
    let mut result = Vec::new();

    for msg in messages {
        if msg.role != "user" {
            continue;
        }

        let content = msg.content.trim();
        if content.len() < 10 {
            continue; // Skip very short messages
        }

        // Try to classify the message
        if let Some(category) = classify_message(content) {
            let normalized = normalize_for_comparison(content);
            if !normalized.is_empty() {
                result.push(UserMessage {
                    normalized,
                    stitch_id: msg.stitch_id.clone(),
                    category,
                });
            }
        }
    }

    result
}

/// Classify a user message into a pattern category, or None if not a directive.
fn classify_message(content: &str) -> Option<PatternCategory> {
    let lower = content.to_lowercase();

    // Correction signals: "no,", "actually,", "wrong,", "that's not", "fix that",
    // "incorrect", "I said", "I meant", "not what I asked"
    if is_correction(&lower) {
        return Some(PatternCategory::Correction);
    }

    // Negative signals: "don't", "never", "stop", "avoid", "no need to",
    // "don't ever", "refrain from"
    if is_negative(&lower) {
        return Some(PatternCategory::Negative);
    }

    // Preference signals: "I prefer", "I like", "use X instead", "always use",
    // "I want", "make sure", "ensure that", "please use"
    if is_preference(&lower) {
        return Some(PatternCategory::Preference);
    }

    // Approval signals for non-obvious choices: "yes that's right",
    // "correct", "perfect", "exactly", "good choice", "yes do that",
    // "looks good"
    if is_approval(&lower) {
        return Some(PatternCategory::Approval);
    }

    None
}

/// Detect correction signals in a message.
fn is_correction(lower: &str) -> bool {
    let correction_prefixes = [
        "no, ",
        "no ",
        "actually, ",
        "actually ",
        "wrong",
        "that's not",
        "thats not",
        "fix that",
        "incorrect",
        "i said ",
        "i meant ",
        "not what i asked",
        "that is wrong",
        "that's wrong",
        "you're wrong",
        "you are wrong",
        "you misunderstood",
        "misunderstood",
    ];

    // Check if the message starts with or contains a correction signal
    for prefix in &correction_prefixes {
        if lower.starts_with(prefix) {
            return true;
        }
    }

    // Also check for correction patterns anywhere (but must be the primary intent)
    let correction_anywhere = [
        "that's not correct",
        "that is not correct",
        "this is wrong",
        "is not what i wanted",
        "isn't what i wanted",
    ];
    for pattern in &correction_anywhere {
        if lower.contains(pattern) {
            return true;
        }
    }

    false
}

/// Detect negative directive signals in a message.
fn is_negative(lower: &str) -> bool {
    let negative_patterns = [
        "don't ",
        "do not ",
        "don't ever",
        "never ",
        "never use ",
        "never do ",
        "stop ",
        "stop using",
        "avoid ",
        "avoid using",
        "no need to ",
        "refrain from",
        "please don't",
        "please do not",
        "shouldn't ",
        "should not ",
        "must not ",
        "don't need to ",
        "don't include",
        "don't add",
        "don't create",
        "don't write",
        "do not include",
        "do not add",
        "do not create",
        "do not write",
    ];

    for pattern in &negative_patterns {
        if lower.contains(pattern) {
            return true;
        }
    }

    false
}

/// Detect preference directive signals in a message.
fn is_preference(lower: &str) -> bool {
    let preference_patterns = [
        "i prefer ",
        "i'd prefer ",
        "i like ",
        "i'd like ",
        "use ",
        " instead",
        "always use ",
        "always do ",
        "i want ",
        "make sure ",
        "ensure that ",
        "ensure ",
        "please use ",
        "please do ",
        "i always ",
        "i typically ",
        "my preference ",
        "preferred way",
        "i'd rather ",
    ];

    for pattern in &preference_patterns {
        if lower.contains(pattern) {
            return true;
        }
    }

    false
}

/// Detect approval/confirmation signals for non-obvious choices.
fn is_approval(lower: &str) -> bool {
    // Short approval messages that validate a non-obvious choice
    let approval_patterns = [
        "yes that's right",
        "yes thats right",
        "yes, that's right",
        "that's correct",
        "thats correct",
        "exactly right",
        "yes exactly",
        "yes, exactly",
        "perfect",
        "yes do that",
        "good choice",
        "yes please",
        "yes, please",
        "go ahead",
        "yes, go ahead",
        "correct",
        "that works",
        "that'll work",
        "sounds good",
        "looks good",
        "yes that works",
    ];

    // Approval signals work best on shorter messages (< 200 chars)
    // to avoid false positives from long messages that happen to contain
    // approval phrases.
    if lower.len() > 200 {
        return false;
    }

    for pattern in &approval_patterns {
        if lower.contains(pattern) || lower.trim() == pattern {
            return true;
        }
    }

    // Exact match for very short approvals
    let short_approvals = ["yes", "correct", "right", "yep", "yup", "ok"];
    let trimmed = lower.trim();
    if trimmed.len() <= 10 {
        for word in &short_approvals {
            if trimmed == *word {
                return true;
            }
        }
    }

    false
}

/// Normalize text for comparison: lowercase, remove punctuation, collapse whitespace.
fn normalize_for_comparison(text: &str) -> String {
    let lower = text.to_lowercase();
    let words: Vec<&str> = lower
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|w| !w.is_empty())
        .collect();
    words.join(" ")
}

/// Compute Jaccard similarity between two normalized texts.
fn jaccard_similarity(text1: &str, text2: &str) -> f64 {
    use std::collections::HashSet;

    let set1: HashSet<&str> = text1.split_whitespace().collect();
    let set2: HashSet<&str> = text2.split_whitespace().collect();

    if set1.is_empty() && set2.is_empty() {
        return 1.0;
    }
    if set1.is_empty() || set2.is_empty() {
        return 0.0;
    }

    let intersection: HashSet<&&str> = set1.intersection(&set2).collect();
    let union: HashSet<&&str> = set1.union(&set2).collect();

    intersection.len() as f64 / union.len() as f64
}

/// Group similar user messages and detect repeated patterns.
fn group_and_detect(
    messages: Vec<UserMessage>,
    config: &ReflectionDetectorConfig,
) -> Vec<DetectedPattern> {
    // Group messages by category first
    let mut by_category: HashMap<PatternCategory, Vec<UserMessage>> = HashMap::new();
    for msg in messages {
        by_category.entry(msg.category).or_default().push(msg);
    }

    let mut patterns = Vec::new();

    for (category, category_messages) in by_category {
        let category_patterns = detect_patterns_in_category(
            category_messages,
            category,
            config.min_occurrences,
            config.similarity_threshold,
        );
        patterns.extend(category_patterns);
    }

    patterns
}

/// Detect repeated patterns within a single category.
fn detect_patterns_in_category(
    messages: Vec<UserMessage>,
    category: PatternCategory,
    min_occurrences: usize,
    similarity_threshold: f64,
) -> Vec<DetectedPattern> {
    if messages.len() < min_occurrences {
        return Vec::new();
    }

    // Cluster messages by similarity using a simple greedy approach
    let mut clusters: Vec<Vec<&UserMessage>> = Vec::new();
    let mut assigned = vec![false; messages.len()];

    for (i, msg) in messages.iter().enumerate() {
        if assigned[i] {
            continue;
        }

        let mut cluster = vec![msg];
        assigned[i] = true;

        // Find all messages similar to this one
        for (j, other) in messages.iter().enumerate() {
            if assigned[j] {
                continue;
            }
            if jaccard_similarity(&msg.normalized, &other.normalized) >= similarity_threshold {
                cluster.push(other);
                assigned[j] = true;
            }
        }

        clusters.push(cluster);
    }

    // Extract patterns from clusters that meet the threshold
    let mut patterns = Vec::new();
    for cluster in clusters {
        if cluster.len() < min_occurrences {
            continue;
        }

        // Use the longest message in the cluster as the representative rule
        let representative = cluster
            .iter()
            .max_by_key(|m| m.normalized.len())
            .unwrap();

        // Summarize the rule from the representative message
        let rule = summarize_rule(&representative.normalized, category);
        if rule.is_empty() {
            continue;
        }

        let source_stitches: Vec<String> = cluster
            .iter()
            .map(|m| m.stitch_id.clone())
            .collect();

        let reason = format!(
            "Detected {} '{}' pattern across {} operator Stitches in the last 30 days. \
             Similar messages were grouped by lexical similarity.",
            cluster.len(),
            category.as_str(),
            source_stitches.len(),
        );

        patterns.push(DetectedPattern {
            rule,
            reason,
            source_stitches,
            category,
        });
    }

    patterns
}

/// Summarize a normalized message into a concise rule for the Reflection Ledger.
fn summarize_rule(normalized: &str, category: PatternCategory) -> String {
    // For very short messages, use as-is
    if normalized.len() <= 120 {
        return normalized.to_string();
    }

    // For longer messages, extract the core directive
    let lower = normalized;

    match category {
        PatternCategory::Correction => {
            // Extract what was corrected
            extract_directive(lower, "correction")
        }
        PatternCategory::Preference => {
            extract_directive(lower, "preference")
        }
        PatternCategory::Negative => {
            extract_directive(lower, "negative")
        }
        PatternCategory::Approval => {
            // Approval messages are usually short; truncate if needed
            let truncated: String = lower.chars().take(120).collect();
            truncated
        }
    }
}

/// Extract the core directive from a message, truncating to a reasonable length.
fn extract_directive(text: &str, kind: &str) -> String {
    // Try to find the first sentence or clause
    let sentence_end = text.find('.').unwrap_or(text.len()).min(120);
    let clause_end = text.find(',').unwrap_or(text.len()).min(sentence_end);

    let end = if clause_end > 20 && clause_end < sentence_end {
        clause_end
    } else {
        sentence_end
    };

    let result = text[..end].trim();
    if result.is_empty() {
        format!("[{} directive]", kind)
    } else {
        result.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_correction() {
        assert_eq!(classify_message("No, use tabs not spaces"), Some(PatternCategory::Correction));
        assert_eq!(classify_message("Actually, I meant the other file"), Some(PatternCategory::Correction));
        assert_eq!(classify_message("That's not what I asked for"), Some(PatternCategory::Correction));
        assert_eq!(classify_message("Fix that bug in the parser"), Some(PatternCategory::Correction));
        assert_eq!(classify_message("I said the config file, not the source"), Some(PatternCategory::Correction));
    }

    #[test]
    fn test_classify_negative() {
        assert_eq!(classify_message("Don't use unwrap() in production code"), Some(PatternCategory::Negative));
        assert_eq!(classify_message("Never commit directly to main"), Some(PatternCategory::Negative));
        assert_eq!(classify_message("Stop using println for logging"), Some(PatternCategory::Negative));
        assert_eq!(classify_message("Avoid cloning large structs"), Some(PatternCategory::Negative));
    }

    #[test]
    fn test_classify_preference() {
        assert_eq!(classify_message("I prefer early returns over nested if-else"), Some(PatternCategory::Preference));
        assert_eq!(classify_message("Use Result instead of unwrap"), Some(PatternCategory::Preference));
        assert_eq!(classify_message("Always use structured logging"), Some(PatternCategory::Preference));
        assert_eq!(classify_message("Make sure the tests pass before committing"), Some(PatternCategory::Preference));
    }

    #[test]
    fn test_classify_approval() {
        assert_eq!(classify_message("Yes that's right"), Some(PatternCategory::Approval));
        assert_eq!(classify_message("Perfect"), Some(PatternCategory::Approval));
        assert_eq!(classify_message("That works"), Some(PatternCategory::Approval));
        assert_eq!(classify_message("correct"), Some(PatternCategory::Approval));
    }

    #[test]
    fn test_classify_not_directive() {
        // Regular conversational messages should not be classified
        assert_eq!(classify_message("What does this function do?"), None);
        assert_eq!(classify_message("Can you explain the architecture?"), None);
        assert_eq!(classify_message("Show me the code"), None);
        assert_eq!(classify_message("Hello"), None);
    }

    #[test]
    fn test_normalize_for_comparison() {
        assert_eq!(normalize_for_comparison("Use Tabs, NOT Spaces!"), "use tabs not spaces");
        assert_eq!(normalize_for_comparison("  multiple   spaces  "), "multiple spaces");
    }

    #[test]
    fn test_jaccard_identical() {
        let sim = jaccard_similarity("don't use unwrap", "don't use unwrap");
        assert!((sim - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_jaccard_similar() {
        let sim = jaccard_similarity("don't use unwrap in production", "don't use unwrap ever");
        assert!(sim > 0.3, "Similar messages should have >0.3 similarity, got {}", sim);
    }

    #[test]
    fn test_jaccard_dissimilar() {
        let sim = jaccard_similarity("don't use unwrap", "hello world");
        assert!(sim < 0.2);
    }

    #[test]
    fn test_detect_repeated_correction() {
        let config = ReflectionDetectorConfig {
            min_occurrences: 3,
            similarity_threshold: 0.45,
            ..Default::default()
        };

        let messages = vec![
            UserMessage {
                normalized: "don't use unwrap in production code".to_string(),
                stitch_id: "st-1".to_string(),
                category: PatternCategory::Negative,
            },
            UserMessage {
                normalized: "don't use unwrap ever".to_string(),
                stitch_id: "st-2".to_string(),
                category: PatternCategory::Negative,
            },
            UserMessage {
                normalized: "don't use unwrap please".to_string(),
                stitch_id: "st-3".to_string(),
                category: PatternCategory::Negative,
            },
            // Unrelated message
            UserMessage {
                normalized: "use structured logging always".to_string(),
                stitch_id: "st-4".to_string(),
                category: PatternCategory::Preference,
            },
        ];

        let patterns = detect_patterns_in_category(
            messages,
            PatternCategory::Negative,
            config.min_occurrences,
            config.similarity_threshold,
        );

        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].category, PatternCategory::Negative);
        assert_eq!(patterns[0].source_stitches.len(), 3);
    }

    #[test]
    fn test_no_pattern_below_threshold() {
        let config = ReflectionDetectorConfig {
            min_occurrences: 3,
            similarity_threshold: 0.45,
            ..Default::default()
        };

        let messages = vec![
            UserMessage {
                normalized: "don't use unwrap".to_string(),
                stitch_id: "st-1".to_string(),
                category: PatternCategory::Negative,
            },
            UserMessage {
                normalized: "don't use unwrap".to_string(),
                stitch_id: "st-2".to_string(),
                category: PatternCategory::Negative,
            },
            // Only 2 occurrences — not enough
        ];

        let patterns = detect_patterns_in_category(
            messages,
            PatternCategory::Negative,
            config.min_occurrences,
            config.similarity_threshold,
        );

        assert!(patterns.is_empty());
    }

    #[test]
    fn test_group_and_detect_multiple_categories() {
        let config = ReflectionDetectorConfig {
            min_occurrences: 3,
            similarity_threshold: 0.45,
            ..Default::default()
        };

        let mut messages = Vec::new();

        // 3 similar negative messages
        for i in 0..3 {
            messages.push(UserMessage {
                normalized: format!("don't use unwrap in module {}", i),
                stitch_id: format!("st-neg-{}", i),
                category: PatternCategory::Negative,
            });
        }

        // 3 similar preference messages
        for i in 0..3 {
            messages.push(UserMessage {
                normalized: format!("use structured logging instead of println {}", i),
                stitch_id: format!("st-pref-{}", i),
                category: PatternCategory::Preference,
            });
        }

        let patterns = group_and_detect(messages, &config);
        assert_eq!(patterns.len(), 2);
    }

    #[test]
    fn test_synthetic_repeated_instructions() {
        // Test against the acceptance criteria fixtures:
        // "Tested against synthetic repeated-instruction fixtures"
        let config = ReflectionDetectorConfig {
            min_occurrences: 3,
            similarity_threshold: 0.40,
            ..Default::default()
        };

        let raw_messages = vec![
            OperatorMessage {
                stitch_id: "st-001".into(),
                role: "user".into(),
                content: "Don't use unwrap() in production code".into(),
            },
            OperatorMessage {
                stitch_id: "st-002".into(),
                role: "user".into(),
                content: "Please don't use unwrap() anywhere in the codebase".into(),
            },
            OperatorMessage {
                stitch_id: "st-003".into(),
                role: "user".into(),
                content: "Don't use unwrap() — use proper error handling".into(),
            },
            OperatorMessage {
                stitch_id: "st-004".into(),
                role: "user".into(),
                content: "I prefer you use Result instead of unwrap".into(),
            },
            OperatorMessage {
                stitch_id: "st-005".into(),
                role: "user".into(),
                content: "Always use Result instead of unwrap in this project".into(),
            },
            OperatorMessage {
                stitch_id: "st-006".into(),
                role: "user".into(),
                content: "I always want Result types, not unwrap calls".into(),
            },
            OperatorMessage {
                stitch_id: "st-007".into(),
                role: "assistant".into(),
                content: "I'll use Result instead of unwrap".into(),
            },
            OperatorMessage {
                stitch_id: "st-008".into(),
                role: "user".into(),
                content: "What does this function do?".into(), // Not a directive
            },
        ];

        let user_messages = extract_user_messages(&raw_messages);
        assert!(user_messages.len() >= 5, "Should extract at least 5 classifiable messages");

        let patterns = group_and_detect(user_messages, &config);
        assert!(
            patterns.len() >= 1,
            "Should detect at least 1 repeated pattern from synthetic fixtures, got {}",
            patterns.len()
        );

        // At least one pattern should involve 3+ source stitches
        let has_multi_source = patterns.iter().any(|p| p.source_stitches.len() >= 3);
        assert!(has_multi_source, "At least one pattern should span 3+ Stitches");
    }

    #[test]
    fn test_approval_length_filter() {
        // Long messages should not be classified as approvals even if they contain approval words
        let long_msg = "I was thinking about the architecture and we should consider using a microservice approach \
                       where each service handles its own domain. That works well for scalability. \
                       Let me elaborate on the specific services I have in mind...";

        assert_eq!(classify_message(long_msg), None);
    }

    #[test]
    fn test_summarize_rule_short() {
        let rule = summarize_rule("don't use unwrap", PatternCategory::Negative);
        assert_eq!(rule, "don't use unwrap");
    }

    #[test]
    fn test_summarize_rule_long() {
        let long = "don't use unwrap in production code because it will panic if the value is none, \
                    and we need to handle errors gracefully using result types and the question mark operator";
        let rule = summarize_rule(long, PatternCategory::Negative);
        // Should be truncated to something reasonable
        assert!(rule.len() < long.len());
        assert!(!rule.is_empty());
    }
}
