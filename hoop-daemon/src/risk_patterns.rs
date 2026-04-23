//! Risk pattern matching from the Fix Lineage library
//!
//! Matches draft Stitches against known failure patterns
//! and recommends fixes based on historical data.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// A known failure pattern with fix recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskPattern {
    /// Unique pattern identifier
    pub id: String,
    /// Pattern name
    pub name: String,
    /// Description of the failure pattern
    pub description: String,
    /// Keywords that trigger this pattern (case-insensitive)
    pub keywords: Vec<String>,
    /// Label keywords that increase confidence
    pub label_keywords: Vec<String>,
    /// Recommended fix approach
    pub fix_recommendation: String,
    /// Severity level
    pub severity: RiskSeverity,
    /// Pattern category
    pub category: RiskCategory,
}

/// Severity of the risk pattern
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Category of the risk pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskCategory {
    Performance,
    Correctness,
    Security,
    Integration,
    CodeQuality,
    Infrastructure,
}

/// Match result for a risk pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMatch {
    /// The matched pattern
    pub pattern: RiskPattern,
    /// Confidence score (0-1)
    pub confidence: f64,
    /// Which keywords matched
    pub matched_keywords: Vec<String>,
    /// Which labels matched
    pub matched_labels: Vec<String>,
}

/// Fix Lineage library: stores and retrieves risk patterns
pub struct FixLineageLibrary {
    patterns: Vec<RiskPattern>,
    /// Keyword to pattern index for fast lookup
    keyword_index: HashMap<String, Vec<usize>>,
}

impl FixLineageLibrary {
    /// Create a new empty library
    pub fn new() -> Self {
        Self {
            patterns: vec![],
            keyword_index: HashMap::new(),
        }
    }

    /// Load patterns from a JSON file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let content = fs::read_to_string(path)?;
        let patterns: Vec<RiskPattern> = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        Ok(Self::from_patterns(patterns))
    }

    /// Create library from a list of patterns
    pub fn from_patterns(patterns: Vec<RiskPattern>) -> Self {
        let mut keyword_index: HashMap<String, Vec<usize>> = HashMap::new();

        for (idx, pattern) in patterns.iter().enumerate() {
            for keyword in &pattern.keywords {
                keyword_index
                    .entry(keyword.to_lowercase())
                    .or_insert_with(Vec::new)
                    .push(idx);
            }
        }

        Self {
            patterns,
            keyword_index,
        }
    }

    /// Match a draft against known risk patterns
    pub fn match_draft(
        &self,
        title: &str,
        body: Option<&str>,
        labels: &[String],
    ) -> Vec<RiskMatch> {
        let text = format!("{} {}", title, body.unwrap_or("")).to_lowercase();
        let labels_lower: Vec<String> = labels.iter().map(|l| l.to_lowercase()).collect();

        let mut pattern_scores: HashMap<usize, RiskMatchBuilder> = HashMap::new();

        // Match against text keywords
        for (keyword, pattern_indices) in &self.keyword_index {
            if text.contains(keyword) {
                for &idx in pattern_indices {
                    let builder = pattern_scores
                        .entry(idx)
                        .or_insert_with(|| RiskMatchBuilder::new(&self.patterns[idx]));
                    builder.add_keyword(keyword);
                }
            }
        }

        // Match against labels
        for label in &labels_lower {
            if let Some(pattern_indices) = self.keyword_index.get(label) {
                for &idx in pattern_indices {
                    let builder = pattern_scores
                        .entry(idx)
                        .or_insert_with(|| RiskMatchBuilder::new(&self.patterns[idx]));
                    builder.add_label(label);
                }
            }
        }

        // Build matches sorted by confidence
        let mut matches: Vec<RiskMatch> = pattern_scores
            .into_values()
            .map(|b| b.build(&labels_lower))
            .filter(|m| m.confidence > 0.0)
            .collect();

        matches.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        matches
    }

    /// Get all patterns
    pub fn patterns(&self) -> &[RiskPattern] {
        &self.patterns
    }

    /// Add a pattern to the library
    pub fn add_pattern(&mut self, pattern: RiskPattern) {
        let idx = self.patterns.len();
        for keyword in &pattern.keywords {
            self.keyword_index
                .entry(keyword.to_lowercase())
                .or_insert_with(Vec::new)
                .push(idx);
        }
        self.patterns.push(pattern);
    }
}

impl Default for FixLineageLibrary {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper for building risk matches
struct RiskMatchBuilder {
    pattern: RiskPattern,
    matched_keywords: Vec<String>,
    matched_labels: Vec<String>,
}

impl RiskMatchBuilder {
    fn new(pattern: &RiskPattern) -> Self {
        Self {
            pattern: pattern.clone(),
            matched_keywords: vec![],
            matched_labels: vec![],
        }
    }

    fn add_keyword(&mut self, keyword: &str) {
        if !self.matched_keywords.contains(&keyword.to_string()) {
            self.matched_keywords.push(keyword.to_string());
        }
    }

    fn add_label(&mut self, label: &str) {
        if !self.matched_labels.contains(&label.to_string()) {
            self.matched_labels.push(label.to_string());
        }
    }

    fn build(self, labels_lower: &[String]) -> RiskMatch {
        // Calculate confidence:
        // - Each keyword match adds 0.3
        // - Each label match adds 0.2
        // - Max confidence is 1.0
        let mut confidence = self.matched_keywords.len() as f64 * 0.3;
        confidence += self.matched_labels.len() as f64 * 0.2;
        confidence = confidence.min(1.0);

        RiskMatch {
            pattern: self.pattern,
            confidence,
            matched_keywords: self.matched_keywords,
            matched_labels: self.matched_labels,
        }
    }
}

/// Default risk patterns for common failure modes
pub fn default_risk_patterns() -> Vec<RiskPattern> {
    vec![
        RiskPattern {
            id: "large_codegen_stack_overflow".to_string(),
            name: "Large Codegen Stack Overflow".to_string(),
            description: "Large-scale code generation tasks often hit token limits or produce incomplete outputs".to_string(),
            keywords: vec![
                "codegen".to_string(),
                "generate".to_string(),
                "implement".to_string(),
                "large".to_string(),
                "refactor".to_string(),
            ],
            label_keywords: vec!["refactor".to_string(), "codegen".to_string()],
            fix_recommendation: "Break into smaller, focused beads. Scope to one file or module per bead.".to_string(),
            severity: RiskSeverity::High,
            category: RiskCategory::CodeQuality,
        },
        RiskPattern {
            id: "missing_test_coverage".to_string(),
            name: "Missing Test Coverage".to_string(),
            description: "New code without tests tends to break in production".to_string(),
            keywords: vec![
                "add".to_string(),
                "implement".to_string(),
                "feature".to_string(),
                "function".to_string(),
            ],
            label_keywords: vec!["feature".to_string()],
            fix_recommendation: "Include test writing in the bead scope or create a follow-up review bead for test coverage.".to_string(),
            severity: RiskSeverity::Medium,
            category: RiskCategory::CodeQuality,
        },
        RiskPattern {
            id: "race_condition_concurrency".to_string(),
            name: "Race Condition / Concurrency Issue".to_string(),
            description: "Concurrency bugs are notoriously difficult to reproduce and fix".to_string(),
            keywords: vec![
                "race".to_string(),
                "concurrent".to_string(),
                "async".to_string(),
                "thread".to_string(),
                "mutex".to_string(),
                "lock".to_string(),
            ],
            label_keywords: vec!["concurrency".to_string(), "async".to_string()],
            fix_recommendation: "Review with focus on shared state access patterns. Consider adding stress tests.".to_string(),
            severity: RiskSeverity::Critical,
            category: RiskCategory::Correctness,
        },
        RiskPattern {
            id: "performance_regression".to_string(),
            name: "Performance Regression".to_string(),
            description: "Changes that may impact performance need baseline measurement".to_string(),
            keywords: vec![
                "optimize".to_string(),
                "performance".to_string(),
                "slow".to_string(),
                "latency".to_string(),
            ],
            label_keywords: vec!["performance".to_string()],
            fix_recommendation: "Include benchmarking in the scope. Measure before and after.".to_string(),
            severity: RiskSeverity::Medium,
            category: RiskCategory::Performance,
        },
        RiskPattern {
            id: "breaking_change".to_string(),
            name: "Breaking Change".to_string(),
            description: "API or contract changes can break downstream consumers".to_string(),
            keywords: vec![
                "change".to_string(),
                "remove".to_string(),
                "api".to_string(),
                "interface".to_string(),
                "contract".to_string(),
            ],
            label_keywords: vec!["api".to_string(), "breaking".to_string()],
            fix_recommendation: "Identify all downstream consumers. Create a migration plan. Consider deprecation first.".to_string(),
            severity: RiskSeverity::High,
            category: RiskCategory::Integration,
        },
        RiskPattern {
            id: "database_migration".to_string(),
            name: "Database Migration Risk".to_string(),
            description: "Schema changes have high blast radius and rollback complexity".to_string(),
            keywords: vec![
                "migration".to_string(),
                "schema".to_string(),
                "database".to_string(),
                "sql".to_string(),
            ],
            label_keywords: vec!["database".to_string(), "migration".to_string()],
            fix_recommendation: "Test migration on a staging dataset first. Plan rollback procedure. Consider backfill strategy.".to_string(),
            severity: RiskSeverity::High,
            category: RiskCategory::Infrastructure,
        },
        RiskPattern {
            id: "dependency_update".to_string(),
            name: "Dependency Update Risk".to_string(),
            description: "Dependency updates can introduce subtle breakage".to_string(),
            keywords: vec![
                "update".to_string(),
                "upgrade".to_string(),
                "dependency".to_string(),
                "version".to_string(),
            ],
            label_keywords: vec!["dependencies".to_string()],
            fix_recommendation: "Pin specific versions. Run full test suite. Check for breaking changes in changelog.".to_string(),
            severity: RiskSeverity::Medium,
            category: RiskCategory::Integration,
        },
        RiskPattern {
            id: "file_overlap_conflict".to_string(),
            name: "File Overlap Conflict".to_string(),
            description: "Multiple beads touching the same files can cause conflicts".to_string(),
            keywords: vec![
                "fix".to_string(),
                "modify".to_string(),
                "change".to_string(),
            ],
            label_keywords: vec![],
            fix_recommendation: "Check for other active beads touching the same files. Consider sequencing or coordinating.".to_string(),
            severity: RiskSeverity::Medium,
            category: RiskCategory::CodeQuality,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_empty() {
        let lib = FixLineageLibrary::new();
        assert!(lib.patterns().is_empty());
    }

    #[test]
    fn test_library_from_patterns() {
        let patterns = default_risk_patterns();
        let lib = FixLineageLibrary::from_patterns(patterns);
        assert!(!lib.patterns().is_empty());
    }

    #[test]
    fn test_match_codegen_risk() {
        let lib = FixLineageLibrary::from_patterns(default_risk_patterns());
        let matches = lib.match_draft("Large codegen refactor", None, &[]);

        assert!(!matches.is_empty());
        let codegen_match = matches.iter().find(|m| m.pattern.id == "large_codegen_stack_overflow");
        assert!(codegen_match.is_some());
    }

    #[test]
    fn test_match_with_label() {
        let lib = FixLineageLibrary::from_patterns(default_risk_patterns());
        let matches = lib.match_draft("Add feature", None, &["feature".to_string()]);

        let test_match = matches.iter().find(|m| m.pattern.id == "missing_test_coverage");
        assert!(test_match.is_some());
    }

    #[test]
    fn test_match_confidence() {
        let lib = FixLineageLibrary::from_patterns(default_risk_patterns());
        let matches = lib.match_draft(
            "Large codegen refactor",
            Some("Need to generate lots of code"),
            &["codegen".to_string()],
        );

        let codegen_match = matches.iter().find(|m| m.pattern.id == "large_codegen_stack_overflow").unwrap();
        // Should have high confidence due to multiple keyword matches
        assert!(codegen_match.confidence > 0.5);
    }

    #[test]
    fn test_add_pattern() {
        let mut lib = FixLineageLibrary::new();
        lib.add_pattern(RiskPattern {
            id: "test_pattern".to_string(),
            name: "Test".to_string(),
            description: "Test".to_string(),
            keywords: vec!["test".to_string()],
            label_keywords: vec![],
            fix_recommendation: "Test fix".to_string(),
            severity: RiskSeverity::Low,
            category: RiskCategory::CodeQuality,
        });

        let matches = lib.match_draft("Test this", None, &[]);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].pattern.id, "test_pattern");
    }

    #[test]
    fn test_match_sorted_by_confidence() {
        let lib = FixLineageLibrary::from_patterns(default_risk_patterns());
        let matches = lib.match_draft("Large refactor", Some("Need to generate code"), &[]);

        // Check that matches are sorted by confidence (descending)
        for i in 1..matches.len() {
            assert!(matches[i - 1].confidence >= matches[i].confidence);
        }
    }

    #[test]
    fn test_default_patterns_exist() {
        let patterns = default_risk_patterns();
        assert!(!patterns.is_empty());

        // Check for expected patterns
        assert!(patterns.iter().any(|p| p.id == "large_codegen_stack_overflow"));
        assert!(patterns.iter().any(|p| p.id == "missing_test_coverage"));
        assert!(patterns.iter().any(|p| p.id == "race_condition_concurrency"));
    }
}
