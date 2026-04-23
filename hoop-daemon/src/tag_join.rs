//! Tag-join resolver: extracts [needle:<worker>:<bead>:<strand>] prefix
//!
//! Every NEEDLE bead dispatch prefixes the first user message with this tag.
//! The resolver extracts it on parse and establishes the session -> bead mapping.
//!
//! - Well-formed tag -> Worker kind with binding
//! - Malformed tag -> logged at warn, treated as missing -> Ad-hoc
//! - Missing tag -> Ad-hoc (or Dictated if [dictated] prefix)
//! - Binding emitted as `TagJoinBound` event (dual-identity invariant §B1)
//!
//! Plan reference: §5.1 Data flows (tag-join arrow), §3 principle 4

use hoop_schema::{ParsedSessionKind, ParsedSessionKindVariant1, ParsedSessionKindVariant2};
use regex::Regex;
use std::sync::OnceLock;
use tracing::warn;

/// Result of tag-join resolution
#[derive(Debug, Clone)]
pub struct TagJoinResult {
    /// The parsed session kind (Worker, Dictated, AdHoc)
    pub kind: ParsedSessionKind,
    /// Binding info present when a well-formed needle tag was found
    pub binding: Option<TagBinding>,
}

/// A session-to-bead binding established via the needle tag.
///
/// When a worker session is discovered and its first message contains a
/// `[needle:<worker>:<bead>:<strand>]` prefix, this struct captures the
/// extracted components. The binding maps the CLI session to the bead being
/// processed, satisfying the dual-identity invariant (§B1).
#[derive(Debug, Clone)]
pub struct TagBinding {
    pub worker: String,
    pub bead: String,
    pub strand: Option<String>,
}

static NEEDLE_TAG_RE: OnceLock<Regex> = OnceLock::new();
static MALFORMED_NEEDLE_RE: OnceLock<Regex> = OnceLock::new();

fn needle_tag_re() -> &'static Regex {
    NEEDLE_TAG_RE.get_or_init(|| {
        Regex::new(r"^\[needle:([^:]+):([^:]+):([^:\]]*)\]")
            .expect("valid needle tag regex")
    })
}

fn malformed_needle_re() -> &'static Regex {
    MALFORMED_NEEDLE_RE.get_or_init(|| {
        Regex::new(r"^\[needle:[^\]]*\]")
            .expect("valid malformed needle regex")
    })
}

/// Resolve the tag-join for a session.
///
/// Examines content for the `[needle:<worker>:<bead>:<strand>]` prefix tag.
/// The resolver checks `first_user_content` (authoritative) first, then falls
/// back to `title` (which Claude Code derives from the first message).
///
/// # Arguments
/// * `title` - Session title
/// * `first_user_content` - Raw first user message content (if available)
///
/// # Returns
/// A `TagJoinResult` with the session kind and optional binding.
pub fn resolve(title: &str, first_user_content: Option<&str>) -> TagJoinResult {
    let sources = [first_user_content, Some(title)];

    // Try well-formed tag extraction from each source
    for content in sources.iter().copied().flatten() {
        if let Some(result) = try_extract_tag(content) {
            return result;
        }
    }

    // No well-formed tag found. Check for malformed needle tags.
    for content in sources.iter().copied().flatten() {
        if malformed_needle_re().is_match(content) {
            warn!(
                "Malformed needle tag in session, treating as missing: {}",
                content.chars().take(80).collect::<String>()
            );
            break;
        }
    }

    // Check for dictated prefix
    for content in sources.iter().copied().flatten() {
        if content.starts_with("[dictated]") {
            return TagJoinResult {
                kind: ParsedSessionKind::Variant1(ParsedSessionKindVariant1::Dictated),
                binding: None,
            };
        }
    }

    // Default: ad-hoc (no prefix, no binding)
    TagJoinResult {
        kind: ParsedSessionKind::Variant2(ParsedSessionKindVariant2::AdHoc),
        binding: None,
    }
}

fn try_extract_tag(content: &str) -> Option<TagJoinResult> {
    let captures = needle_tag_re().captures(content)?;
    let worker = captures.get(1)?.as_str().to_string();
    let bead = captures.get(2)?.as_str().to_string();
    let strand = captures
        .get(3)
        .map(|m| m.as_str().to_string())
        .filter(|s| !s.is_empty());

    Some(TagJoinResult {
        kind: ParsedSessionKind::Variant0 {
            worker: worker.clone(),
            bead: bead.clone(),
            strand: strand.clone(),
        },
        binding: Some(TagBinding {
            worker,
            bead,
            strand,
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Well-formed tag extraction ---

    #[test]
    fn test_worker_tag_full() {
        let result = resolve("[needle:alpha:bd-abc123:pluck] Fix the login bug", None);
        match result.kind {
            ParsedSessionKind::Variant0 { worker, bead, strand } => {
                assert_eq!(worker, "alpha");
                assert_eq!(bead, "bd-abc123");
                assert_eq!(strand.as_deref(), Some("pluck"));
            }
            _ => panic!("Expected Worker kind"),
        }
        let binding = result.binding.expect("should have binding");
        assert_eq!(binding.worker, "alpha");
        assert_eq!(binding.bead, "bd-abc123");
        assert_eq!(binding.strand.as_deref(), Some("pluck"));
    }

    #[test]
    fn test_worker_tag_empty_strand() {
        let result = resolve("[needle:bravo:bd-def456:] Some task", None);
        match result.kind {
            ParsedSessionKind::Variant0 { worker, bead, strand } => {
                assert_eq!(worker, "bravo");
                assert_eq!(bead, "bd-def456");
                assert!(strand.is_none());
            }
            _ => panic!("Expected Worker kind"),
        }
    }

    #[test]
    fn test_worker_tag_no_strand_value() {
        // Empty strand (no trailing content after colon)
        let result = resolve("[needle:charlie:bd-ghi789:]", None);
        match result.kind {
            ParsedSessionKind::Variant0 { strand, .. } => {
                assert!(strand.is_none());
            }
            _ => panic!("Expected Worker kind"),
        }
    }

    #[test]
    fn test_worker_tag_from_first_user_content() {
        // Tag in first user message, not in title
        let result = resolve(
            "Fix the login bug",
            Some("[needle:delta:bd-jkl012:mend] Fix the login bug in auth module"),
        );
        match result.kind {
            ParsedSessionKind::Variant0 { worker, bead, strand } => {
                assert_eq!(worker, "delta");
                assert_eq!(bead, "bd-jkl012");
                assert_eq!(strand.as_deref(), Some("mend"));
            }
            _ => panic!("Expected Worker kind"),
        }
    }

    #[test]
    fn test_worker_tag_prefers_first_user_content() {
        // Tag in first user message should take precedence over title
        let result = resolve(
            "[needle:wrong:bd-bad:pluck] Wrong tag",
            Some("[needle:echo:bd-mno345:explore] Correct tag from first message"),
        );
        match result.kind {
            ParsedSessionKind::Variant0 { worker, .. } => {
                assert_eq!(worker, "echo"); // Should use first_user_content, not title
            }
            _ => panic!("Expected Worker kind"),
        }
    }

    // --- All four adapters ---

    #[test]
    fn test_adapter_claude_code() {
        // Claude Code: title derived from first user message, tag in title
        let result = resolve(
            "[needle:alpha:bd-abc:pluck] Implement the new auth flow",
            None,
        );
        assert!(result.binding.is_some());
        assert_eq!(result.binding.unwrap().worker, "alpha");
    }

    #[test]
    fn test_adapter_codex() {
        // Codex (OpenAI): tag in first user message, title may differ
        let result = resolve(
            "Fix auth flow",
            Some("[needle:bravo:bd-def:explore] Fix auth flow in the login module"),
        );
        assert!(result.binding.is_some());
        assert_eq!(result.binding.unwrap().worker, "bravo");
    }

    #[test]
    fn test_adapter_opencode() {
        // OpenCode: tag in first user message
        let result = resolve(
            "Refactor config",
            Some("[needle:charlie:bd-ghi:mend] Refactor the configuration parsing"),
        );
        assert!(result.binding.is_some());
        assert_eq!(result.binding.unwrap().worker, "charlie");
    }

    #[test]
    fn test_adapter_gemini() {
        // Gemini: tag in first user message
        let result = resolve(
            "Add tests",
            Some("[needle:delta:bd-jkl:weave] Add integration tests for the API"),
        );
        assert!(result.binding.is_some());
        assert_eq!(result.binding.unwrap().worker, "delta");
    }

    // --- Malformed tags (logged at warn, treated as missing) ---

    #[test]
    fn test_malformed_tag_too_few_parts() {
        // [needle:alpha] - only 1 part, needs 3
        let result = resolve("[needle:alpha] Fix the bug", None);
        assert_eq!(
            result.kind,
            ParsedSessionKind::Variant2(ParsedSessionKindVariant2::AdHoc)
        );
        assert!(result.binding.is_none());
    }

    #[test]
    fn test_malformed_tag_two_parts() {
        // [needle:alpha:bd-123] - only 2 parts, needs 3
        let result = resolve("[needle:alpha:bd-123] Fix the bug", None);
        assert_eq!(
            result.kind,
            ParsedSessionKind::Variant2(ParsedSessionKindVariant2::AdHoc)
        );
        assert!(result.binding.is_none());
    }

    #[test]
    fn test_malformed_tag_too_many_parts() {
        // [needle:alpha:bd-123:pluck:extra] - 4 parts, should only have 3
        let result = resolve("[needle:alpha:bd-123:pluck:extra] Fix the bug", None);
        assert_eq!(
            result.kind,
            ParsedSessionKind::Variant2(ParsedSessionKindVariant2::AdHoc)
        );
        assert!(result.binding.is_none());
    }

    #[test]
    fn test_malformed_tag_empty_brackets() {
        // [needle:] - completely empty
        let result = resolve("[needle:] Do something", None);
        assert_eq!(
            result.kind,
            ParsedSessionKind::Variant2(ParsedSessionKindVariant2::AdHoc)
        );
        assert!(result.binding.is_none());
    }

    // --- Missing tags (ad-hoc classification) ---

    #[test]
    fn test_missing_tag_plain_title() {
        let result = resolve("Fix the login bug", None);
        assert_eq!(
            result.kind,
            ParsedSessionKind::Variant2(ParsedSessionKindVariant2::AdHoc)
        );
        assert!(result.binding.is_none());
    }

    #[test]
    fn test_missing_tag_empty_strings() {
        let result = resolve("", None);
        assert_eq!(
            result.kind,
            ParsedSessionKind::Variant2(ParsedSessionKindVariant2::AdHoc)
        );
        assert!(result.binding.is_none());
    }

    #[test]
    fn test_missing_tag_with_first_user_content() {
        let result = resolve("Some title", Some("Just a regular message without any tag"));
        assert_eq!(
            result.kind,
            ParsedSessionKind::Variant2(ParsedSessionKindVariant2::AdHoc)
        );
        assert!(result.binding.is_none());
    }

    // --- Dictated prefix ---

    #[test]
    fn test_dictated_prefix() {
        let result = resolve("[dictated] Voice note transcript", None);
        assert_eq!(
            result.kind,
            ParsedSessionKind::Variant1(ParsedSessionKindVariant1::Dictated)
        );
        assert!(result.binding.is_none());
    }

    // --- Edge cases ---

    #[test]
    fn test_tag_not_at_start() {
        // Tag must be at the beginning
        let result = resolve("Some text [needle:alpha:bd-abc:pluck] then more", None);
        assert_eq!(
            result.kind,
            ParsedSessionKind::Variant2(ParsedSessionKindVariant2::AdHoc)
        );
        assert!(result.binding.is_none());
    }

    #[test]
    fn test_tag_with_hyphenated_names() {
        let result = resolve(
            "[needle:worker-alpha:bd-abc-123:deep-explore] Complex task",
            None,
        );
        match result.kind {
            ParsedSessionKind::Variant0 { worker, bead, strand } => {
                assert_eq!(worker, "worker-alpha");
                assert_eq!(bead, "bd-abc-123");
                assert_eq!(strand.as_deref(), Some("deep-explore"));
            }
            _ => panic!("Expected Worker kind"),
        }
    }

    #[test]
    fn test_tag_fallback_to_title() {
        // First user content has no tag, but title does
        let result = resolve(
            "[needle:alpha:bd-abc:pluck] Task from title",
            Some("User message without tag"),
        );
        match result.kind {
            ParsedSessionKind::Variant0 { worker, .. } => {
                assert_eq!(worker, "alpha");
            }
            _ => panic!("Expected Worker kind"),
        }
    }

    #[test]
    fn test_malformed_in_first_user_content() {
        // Malformed tag in first user content, no tag in title
        let result = resolve(
            "Plain title",
            Some("[needle:alpha] Malformed in first message"),
        );
        assert_eq!(
            result.kind,
            ParsedSessionKind::Variant2(ParsedSessionKindVariant2::AdHoc)
        );
        assert!(result.binding.is_none());
    }
}
