//! Pure function unit test suite
//!
//! Comprehensive unit tests for pure functions identified in plan §14.2:
//! - parsers (parse_jsonl_safe)
//! - tag extract (tag_join)
//! - canonicalization (embedding)
//! - cost math (cost)
//! - status derivation (stitch_status)
//! - ANSI stripping (ansi_strip)
//! - similarity metrics (similarity)
//! - sanitization (svg_sanitize, pdf_sanitize)
//! - prompt substitution (prompt_substitute)
//! - path security (path_security)
//!
//! This test suite:
//! - Focuses on pure functions (no I/O, no mutable state)
//! - Runs in <60s
//! - Provides >80% coverage on pure-function modules
//!
//! Run with: cargo test --test pure_functions

#[cfg(test)]
mod pure_function_tests {
    // Import pure function modules for testing
    use hoop_daemon::ansi_strip;
    use hoop_daemon::cost;
    use hoop_daemon::embedding;
    use hoop_daemon::parse_jsonl_safe;
    use hoop_daemon::pdf_sanitize;
    use hoop_daemon::prompt_substitute;
    use hoop_daemon::similarity;
    use hoop_daemon::stitch_status;
    use hoop_daemon::svg_sanitize;
    use hoop_daemon::tag_join;
    use hoop_schema::path_security;

    // ============================================================================
    // ANSI Stripping Tests
    // ============================================================================

    #[test]
    fn test_strip_basic_sgr() {
        assert_eq!(ansi_strip::strip_ansi("\x1b[0m"), "");
        assert_eq!(ansi_strip::strip_ansi("\x1b[31mRed\x1b[0m"), "Red");
    }

    #[test]
    fn test_strip_256_color() {
        assert_eq!(ansi_strip::strip_ansi("\x1b[38;5;123m"), "");
        assert_eq!(ansi_strip::strip_ansi("\x1b[48;5;255m"), "");
    }

    #[test]
    fn test_strip_rgb_color() {
        assert_eq!(ansi_strip::strip_ansi("\x1b[38;2;255;0;128m"), "");
        assert_eq!(ansi_strip::strip_ansi("\x1b[48;2;0;128;255m"), "");
    }

    #[test]
    fn test_preserve_normal_text() {
        assert_eq!(ansi_strip::strip_ansi("Just normal text"), "Just normal text");
        assert_eq!(ansi_strip::strip_ansi("Text with 🎉 emoji"), "Text with 🎉 emoji");
    }

    // ============================================================================
    // Cost Math Tests
    // ============================================================================

    #[test]
    fn test_extract_project() {
        assert_eq!(cost::CostAggregator::extract_project("/home/coding/HOOP"), "HOOP");
        assert_eq!(
            cost::CostAggregator::extract_project("/home/user/projects/my-project"),
            "my-project"
        );
    }

    #[test]
    fn test_worker_to_model() {
        assert_eq!(cost::CostAggregator::worker_to_model("alpha"), "opus");
        assert_eq!(cost::CostAggregator::worker_to_model("beta"), "sonnet");
        assert_eq!(cost::CostAggregator::worker_to_model("gamma"), "haiku");
    }

    #[test]
    fn test_extract_account_id_default() {
        assert_eq!(
            cost::CostAggregator::extract_account_id(
                "/home/user/.codex/sessions/abc.json",
                "codex"
            ),
            "default"
        );
    }

    #[test]
    fn test_extract_account_id_named() {
        assert_eq!(
            cost::CostAggregator::extract_account_id(
                "/home/user/.codex-work/sessions/abc.json",
                "codex"
            ),
            "work"
        );
    }

    // ============================================================================
    // Status Derivation Tests
    // ============================================================================

    #[test]
    fn test_purity_same_inputs_same_output() {
        use chrono::{Duration, Utc};
        use hoop_daemon::stitch_status::{
            BeadStatus, BeadType, LinkedBead, StitchActivity, StitchContext,
        };

        let now = Utc::now();
        let ctx = StitchContext {
            linked_beads: vec![],
            activity: StitchActivity {
                last_message_at: Some(now),
                last_streaming_at: None,
            },
            config: Default::default(),
        };

        let status1 = ctx.derive_status();
        let status2 = ctx.derive_status();
        assert_eq!(status1, status2);
    }

    #[test]
    fn test_claimed_bead_is_in_progress() {
        use chrono::Utc;
        use hoop_daemon::stitch_status::{
            BeadStatus, BeadType, LinkedBead, StitchActivity, StitchContext,
        };

        let ctx = StitchContext {
            linked_beads: vec![LinkedBead {
                id: "bd-1".to_string(),
                status: BeadStatus::Open,
                issue_type: BeadType::Task,
                claimed_by: Some("worker-alpha".to_string()),
                updated_at: Utc::now(),
            }],
            activity: StitchActivity {
                last_message_at: Some(Utc::now()),
                last_streaming_at: None,
            },
            config: Default::default(),
        };

        assert_eq!(ctx.derive_status(), stitch_status::StitchStatus::InProgress);
    }

    // ============================================================================
    // Tag Join Tests
    // ============================================================================

    #[test]
    fn test_worker_tag_full() {
        let result = tag_join::resolve("[needle:alpha:bd-abc123:pluck] Fix the login bug", None);
        assert!(result.binding.is_some());
        assert_eq!(result.binding.unwrap().worker, "alpha");
    }

    #[test]
    fn test_malformed_tag_too_few_parts() {
        let result = tag_join::resolve("[needle:alpha] Fix the bug", None);
        assert!(result.binding.is_none());
    }

    #[test]
    fn test_dictated_prefix() {
        let result = tag_join::resolve("[dictated] Voice note transcript", None);
        assert!(matches!(
            result.kind,
            hoop_schema::ParsedSessionKind::Variant1(
                hoop_schema::ParsedSessionKindVariant1::Dictated
            )
        ));
    }

    // ============================================================================
    // Canonicalization Tests
    // ============================================================================

    #[test]
    fn test_canonicalize() {
        assert_eq!(embedding::NgramEmbedder::canonicalize("auth"), "auth");
        assert_eq!(embedding::NgramEmbedder::canonicalize("authentication"), "auth");
        assert_eq!(embedding::NgramEmbedder::canonicalize("db"), "db");
        assert_eq!(embedding::NgramEmbedder::canonicalize("database"), "db");
    }

    #[test]
    fn test_canonicalize_empty() {
        assert_eq!(embedding::NgramEmbedder::canonicalize(""), "");
    }

    // ============================================================================
    // Similarity Tests
    // ============================================================================

    #[test]
    fn test_tokenize_simple() {
        let tokens = similarity::tokenize("Hello world foo bar");
        assert_eq!(tokens, vec!["hello", "world", "foo", "bar"]);
    }

    #[test]
    fn test_tokenize_punctuation() {
        let tokens = similarity::tokenize("Hello, world! Foo: bar.");
        assert_eq!(tokens, vec!["hello", "world", "foo", "bar"]);
    }

    #[test]
    fn test_text_similarity_identical() {
        let sim = similarity::text_similarity("fix the bug", "fix the bug");
        assert!((sim.jaccard - 1.0).abs() < f64::EPSILON);
        assert_eq!(sim.overlap_count, 3);
    }

    // ============================================================================
    // Path Security Tests
    // ============================================================================

    #[test]
    fn test_path_allowlist_contains_root() {
        use std::path::PathBuf;
        let root = PathBuf::from("/tmp/test");
        let al = path_security::PathAllowlist::from_roots(vec![root.clone()]);
        assert!(al.contains(&root));
        assert!(al.contains(&root.join("subdir")));
    }

    #[test]
    fn test_path_allowlist_rejects_outside() {
        use std::path::PathBuf;
        let root = PathBuf::from("/tmp/test1");
        let al = path_security::PathAllowlist::from_roots(vec![root]);
        let outside = PathBuf::from("/tmp/test2");
        assert!(!al.contains(&outside));
    }

    // ============================================================================
    // SVG Sanitize Tests
    // ============================================================================

    #[test]
    fn test_strips_script_element() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><script>alert(1)</script><rect/></svg>"#;
        let result = svg_sanitize::sanitize(svg.as_bytes()).expect("sanitize should not fail");
        assert!(result.record.was_modified);
        let out = String::from_utf8(result.safe_bytes).unwrap();
        assert!(!out.contains("script"));
    }

    #[test]
    fn test_strips_onclick_attr() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><rect onclick="alert(1)" width="100"/></svg>"#;
        let result = svg_sanitize::sanitize(svg.as_bytes()).expect("sanitize should not fail");
        assert!(result.record.was_modified);
        let out = String::from_utf8(result.safe_bytes).unwrap();
        assert!(!out.contains("onclick"));
    }

    // ============================================================================
    // PDF Sanitize Tests
    // ============================================================================

    #[test]
    fn test_detects_javascript_action() {
        let pdf = b"%PDF-1.4\n1 0 obj\n<< /Type /Action /S /JavaScript /JS (alert(1)) >>\nendobj\n%%EOF\n";
        let result = pdf_sanitize::sanitize(pdf).expect("sanitize should not fail");
        assert!(result.record.was_modified);
        let out = String::from_utf8(result.safe_bytes).unwrap();
        assert!(!out.contains("/JavaScript"));
    }

    #[test]
    fn test_legit_pdf_unchanged() {
        let pdf = b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n%%EOF\n";
        let result = pdf_sanitize::sanitize(pdf).expect("sanitize should not fail");
        assert!(!result.record.was_modified);
    }

    // ============================================================================
    // Prompt Substitute Tests
    // ============================================================================

    #[test]
    fn test_substitute_project() {
        let ctx = prompt_substitute::SubstitutionContext::new().project("myproject".to_string());
        let result = prompt_substitute::substitute("Working on {{project}}", &ctx).unwrap();
        assert_eq!(result, "Working on myproject");
    }

    #[test]
    fn test_substitute_unknown_variable() {
        let ctx = prompt_substitute::SubstitutionContext::new();
        let result = prompt_substitute::substitute("Unknown {{typo}}", &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_variables() {
        let vars = prompt_substitute::extract_variables("{{project}} and {{file}} and {{custom}}");
        assert_eq!(vars, vec!["custom", "file", "project"]);
    }

    // ============================================================================
    // Parser Tests (parse_jsonl_safe)
    // ============================================================================

    #[test]
    fn test_ndjson_reader_basic() {
        use hoop_daemon::parse_jsonl_safe::NdjsonReader;

        let mut reader = NdjsonReader::new();
        let lines = reader.feed("{\"a\":1}\n{\"b\":2}\n");
        assert_eq!(lines, vec!["{\"a\":1}", "{\"b\":2}"]);
    }

    #[test]
    fn test_ndjson_reader_line_spanning() {
        use hoop_daemon::parse_jsonl_safe::NdjsonReader;

        let mut reader = NdjsonReader::new();
        let parts = ["{\"ki", "ng\":", "\"val", "ue\"}", "\n"];
        let mut all = Vec::new();
        for part in parts {
            all.extend(reader.feed(part));
        }
        assert_eq!(all, vec![r#"{"king":"value"}"#]);
    }

    #[test]
    fn test_parse_line_valid_json() {
        use hoop_daemon::parse_jsonl_safe::{LineSource, parse_line, ParseResult};

        let source = LineSource {
            tag: "test",
            file_path: "/tmp/test.jsonl".into(),
            line_number: 1,
        };

        let result: ParseResult<serde_json::Value> = parse_line(r#"{"key": "value"}"#, &source);
        assert!(matches!(result, ParseResult::Ok(_)));
    }
}
