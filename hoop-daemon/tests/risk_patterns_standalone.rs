//! Standalone test for risk_patterns module
//! This file tests the risk_patterns functionality without requiring
//! the full daemon infrastructure that other integration tests need.

use hoop_daemon::risk_patterns::{
    default_risk_patterns, FixLineageLibrary, RiskCategory, RiskPattern, RiskSeverity,
};

#[test]
fn test_library_empty() {
    let lib = FixLineageLibrary::new();
    assert!(lib.patterns().is_empty());
}

#[test]
fn test_library_from_patterns() {
    let patterns = default_risk_patterns();
    let expected_count = patterns.len();
    let lib = FixLineageLibrary::from_patterns(patterns);

    assert_eq!(
        lib.patterns().len(),
        expected_count,
        "Library should contain all patterns passed to from_patterns()"
    );

    assert!(
        lib.patterns()
            .iter()
            .any(|p| p.id == "large_codegen_stack_overflow"),
        "Library should contain expected pattern IDs"
    );
}

#[test]
fn test_match_codegen_risk() {
    let lib = FixLineageLibrary::from_patterns(default_risk_patterns());
    let matches = lib.match_draft("Large codegen refactor", None, &[]);

    assert!(!matches.is_empty());
    let codegen_match = matches
        .iter()
        .find(|m| m.pattern.id == "large_codegen_stack_overflow");
    assert!(codegen_match.is_some());
}

#[test]
fn test_match_with_label() {
    let lib = FixLineageLibrary::from_patterns(default_risk_patterns());
    let matches = lib.match_draft("Add feature", None, &["feature".to_string()]);

    let test_match = matches
        .iter()
        .find(|m| m.pattern.id == "missing_test_coverage");
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

    let codegen_match = matches
        .iter()
        .find(|m| m.pattern.id == "large_codegen_stack_overflow")
        .unwrap();
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

    assert_eq!(
        matches.len(),
        1,
        "Should find exactly one match for 'test' keyword"
    );
    assert_eq!(
        matches[0].pattern.id, "test_pattern",
        "Matched pattern should have the expected ID"
    );
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
    assert!(patterns
        .iter()
        .any(|p| p.id == "large_codegen_stack_overflow"));
    assert!(patterns.iter().any(|p| p.id == "missing_test_coverage"));
}

#[test]
fn test_add_multiple_patterns() {
    let mut lib = FixLineageLibrary::new();

    lib.add_pattern(RiskPattern {
        id: "pattern1".to_string(),
        name: "Pattern 1".to_string(),
        description: "First pattern".to_string(),
        keywords: vec!["keyword1".to_string()],
        label_keywords: vec![],
        fix_recommendation: "Fix 1".to_string(),
        severity: RiskSeverity::Low,
        category: RiskCategory::CodeQuality,
    });

    lib.add_pattern(RiskPattern {
        id: "pattern2".to_string(),
        name: "Pattern 2".to_string(),
        description: "Second pattern".to_string(),
        keywords: vec!["keyword2".to_string()],
        label_keywords: vec![],
        fix_recommendation: "Fix 2".to_string(),
        severity: RiskSeverity::Medium,
        category: RiskCategory::Integration,
    });

    assert_eq!(
        lib.patterns().len(),
        2,
        "Library should contain exactly 2 patterns"
    );

    let matches1 = lib.match_draft("Test keyword1", None, &[]);
    assert_eq!(
        matches1.len(),
        1,
        "Should find exactly one match for keyword1"
    );
    assert_eq!(matches1[0].pattern.id, "pattern1");

    let matches2 = lib.match_draft("Test keyword2", None, &[]);
    assert_eq!(
        matches2.len(),
        1,
        "Should find exactly one match for keyword2"
    );
    assert_eq!(matches2[0].pattern.id, "pattern2");
}

#[test]
fn test_add_pattern_with_label_keywords() {
    let mut lib = FixLineageLibrary::new();

    lib.add_pattern(RiskPattern {
        id: "label_pattern".to_string(),
        name: "Label Pattern".to_string(),
        description: "Test label keywords".to_string(),
        keywords: vec![],
        label_keywords: vec!["bug".to_string(), "urgent".to_string()],
        fix_recommendation: "Fix label pattern".to_string(),
        severity: RiskSeverity::High,
        category: RiskCategory::CodeQuality,
    });

    // Should match via label keyword
    let matches = lib.match_draft("Test issue", None, &["bug".to_string()]);
    assert_eq!(matches.len(), 1, "Should find match via label keyword");
    assert_eq!(matches[0].pattern.id, "label_pattern");
}

#[test]
fn test_add_pattern_with_mixed_keywords() {
    let mut lib = FixLineageLibrary::new();

    lib.add_pattern(RiskPattern {
        id: "mixed_pattern".to_string(),
        name: "Mixed Keywords".to_string(),
        description: "Test mixed title and label keywords".to_string(),
        keywords: vec!["refactor".to_string(), "large".to_string()],
        label_keywords: vec!["feature".to_string()],
        fix_recommendation: "Fix mixed pattern".to_string(),
        severity: RiskSeverity::Medium,
        category: RiskCategory::CodeQuality,
    });

    // Should match via title keyword
    let matches_title = lib.match_draft("Large refactor needed", None, &[]);
    assert_eq!(
        matches_title.len(),
        1,
        "Should find match via title keyword"
    );

    // Should match via label keyword
    let matches_label = lib.match_draft("Test issue", None, &["feature".to_string()]);
    assert_eq!(
        matches_label.len(),
        1,
        "Should find match via label keyword"
    );

    // Should have higher confidence when both match
    let matches_both = lib.match_draft("Large refactor", None, &["feature".to_string()]);
    assert_eq!(
        matches_both.len(),
        1,
        "Should find match with both keywords"
    );
    assert!(
        matches_both[0].confidence > matches_title[0].confidence,
        "Combined match should have higher confidence"
    );
}
