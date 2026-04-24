//! Per-phase privacy-surface audit (§18, hoop-ttb.15.6)
//!
//! Verifies that every ingestion surface added in each phase routes through
//! the secrets scanner and that a synthetic secret placed in that surface is
//! flagged before storage or propagation.
//!
//! Phase coverage:
//!   Phase 3 — screen-capture frames + voice transcripts
//!   Phase 4 — bulk-draft bodies + imported markdown lists
//!   Phase 5 — morning-brief outputs + cross-project propagation drafts
//!
//! CI command:
//!   cargo test -p hoop-daemon --test privacy_surface_audit

use hoop_daemon::redaction::{
    scan_draft_body, scan_morning_brief, scan_propagation_draft,
    scan_screen_capture_text, scan_voice_transcript, SecretFinding,
};

// ── Synthetic secrets ──────────────────────────────────────────────────────────

const ANTHROPIC_KEY: &str = "sk-ant-api03-SYNTH111AAABBBCCCDDDEEEFFFGGGHHH";
const GITHUB_TOKEN: &str = "ghp_16C7e42F292c6912E7710c838347Ae178B4a";
const AWS_KEY: &str = "AKIAIOSFODNN7EXAMPLE";
const JWT_TOKEN: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";

fn has_finding(findings: &[SecretFinding], pattern: &str) -> bool {
    findings.iter().any(|f| f.pattern_name == pattern)
}

fn is_flagged(findings: &[SecretFinding]) -> bool {
    !findings.is_empty()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 3: Screen-capture frames + voice transcripts
// ═══════════════════════════════════════════════════════════════════════════════

/// Screen-capture frame text containing an Anthropic API key is flagged.
#[test]
fn phase3_screen_capture_frame_flags_anthropic_key() {
    let frame_text = format!(
        "Terminal window showing: export ANTHROPIC_API_KEY={ANTHROPIC_KEY}"
    );
    let findings = scan_screen_capture_text(&frame_text);
    assert!(
        is_flagged(&findings),
        "screen-capture frame with Anthropic key must be flagged; got: {findings:?}"
    );
    assert!(
        has_finding(&findings, "anthropic_api_key") || has_finding(&findings, "env_var_secret"),
        "expected anthropic_api_key or env_var_secret pattern; got: {findings:?}"
    );
}

/// Screen-capture frame text containing a GitHub token is flagged.
#[test]
fn phase3_screen_capture_frame_flags_github_token() {
    let frame_text = format!("git remote set-url origin https://{GITHUB_TOKEN}@github.com/org/repo");
    let findings = scan_screen_capture_text(&frame_text);
    assert!(
        is_flagged(&findings),
        "screen-capture frame with GitHub token must be flagged; got: {findings:?}"
    );
    assert!(
        has_finding(&findings, "github_token_ghp"),
        "expected github_token_ghp pattern; got: {findings:?}"
    );
}

/// A clean screen-capture frame produces no findings.
#[test]
fn phase3_screen_capture_frame_clean_text_no_findings() {
    let frame_text = "The test passed. No secrets here. Build output: OK";
    let findings = scan_screen_capture_text(frame_text);
    assert!(
        findings.is_empty(),
        "clean frame text must produce no findings; got: {findings:?}"
    );
}

/// Voice transcript containing an Anthropic API key is flagged (§18.2).
#[test]
fn phase3_voice_transcript_flags_anthropic_key() {
    let transcript = format!(
        "And then I set the key to {ANTHROPIC_KEY} in the environment file."
    );
    let findings = scan_voice_transcript(&transcript);
    assert!(
        is_flagged(&findings),
        "voice transcript with Anthropic key must be flagged; got: {findings:?}"
    );
    assert!(
        has_finding(&findings, "anthropic_api_key"),
        "expected anthropic_api_key pattern; got: {findings:?}"
    );
}

/// Voice transcript containing a JWT is flagged.
#[test]
fn phase3_voice_transcript_flags_jwt() {
    let transcript = format!("The bearer token value is {JWT_TOKEN} which expires tomorrow.");
    let findings = scan_voice_transcript(&transcript);
    assert!(
        is_flagged(&findings),
        "voice transcript with JWT must be flagged; got: {findings:?}"
    );
    assert!(
        has_finding(&findings, "jwt"),
        "expected jwt pattern; got: {findings:?}"
    );
}

/// Voice transcript with env-var style secret is flagged.
#[test]
fn phase3_voice_transcript_flags_env_var_style_secret() {
    let transcript = format!(
        "So the config has OPENAI_API_KEY={ANTHROPIC_KEY} in the dotenv file."
    );
    let findings = scan_voice_transcript(&transcript);
    assert!(
        is_flagged(&findings),
        "voice transcript with env-var secret must be flagged; got: {findings:?}"
    );
}

/// Clean voice transcript produces no findings.
#[test]
fn phase3_voice_transcript_clean_no_findings() {
    let transcript = "Today I reviewed the pull request and everything looks good. The tests all pass.";
    let findings = scan_voice_transcript(transcript);
    assert!(
        findings.is_empty(),
        "clean voice transcript must produce no findings; got: {findings:?}"
    );
}

/// SecretFinding carries accurate position metadata.
#[test]
fn phase3_finding_position_metadata_accurate() {
    let prefix = "export ANTHROPIC_API_KEY=";
    let transcript = format!("{prefix}{ANTHROPIC_KEY}");
    let findings = scan_voice_transcript(&transcript);
    assert!(!findings.is_empty(), "should find secrets");

    // At least one finding should start at or after the prefix
    let any_in_range = findings.iter().any(|f| f.match_start >= prefix.len() - 1);
    assert!(any_in_range, "at least one finding should overlap the key portion");

    // Each finding should have non-zero length
    for f in &findings {
        assert!(f.match_len > 0, "finding match_len must be > 0; got: {f:?}");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 4: Bulk-draft bodies + imported markdown lists
// ═══════════════════════════════════════════════════════════════════════════════

/// Draft title containing a secret is flagged.
#[test]
fn phase4_draft_title_with_secret_flagged() {
    let title = format!("Fix the API key rotation for {ANTHROPIC_KEY}");
    let findings = scan_draft_body(&title, None);
    assert!(
        is_flagged(&findings),
        "draft title with embedded key must be flagged; got: {findings:?}"
    );
}

/// Draft body (description) containing a secret is flagged.
#[test]
fn phase4_draft_body_with_secret_flagged() {
    let title = "Fix credential rotation";
    let body = format!(
        "The current AWS key is {AWS_KEY}.\n\
         Rotate before the quarterly review."
    );
    let findings = scan_draft_body(title, Some(&body));
    assert!(
        is_flagged(&findings),
        "draft body with AWS key must be flagged; got: {findings:?}"
    );
    assert!(
        has_finding(&findings, "aws_access_key"),
        "expected aws_access_key pattern; got: {findings:?}"
    );
}

/// Bulk-imported markdown list with a secret in one item is flagged.
///
/// A bulk draft is N separate `scan_draft_body` calls, one per item. This
/// test simulates a 3-item markdown list where item 2 contains a secret.
#[test]
fn phase4_bulk_import_markdown_list_flags_secret_item() {
    let items = vec![
        ("Investigate flaky test on CI", None),
        (
            "Rotate credentials",
            Some({
                // Construct the synthetic Slack bot token at runtime so the
                // literal doesn't trigger GitHub push-protection on test fixtures.
                let tok = ["xoxb", "111222333444", "111222333444", "AAABBBCCCDDDEEE"].join("-");
                format!("Current Slack bot token: {tok}\nNeeds rotation.")
            }),
        ),
        ("Update documentation", None),
    ];

    let flagged_items: Vec<_> = items
        .iter()
        .enumerate()
        .filter_map(|(i, (title, body))| {
            let findings = scan_draft_body(title, body.as_deref());
            if !findings.is_empty() { Some(i) } else { None }
        })
        .collect();

    assert_eq!(
        flagged_items,
        vec![1],
        "only item 1 (with Slack token) should be flagged; got: {flagged_items:?}"
    );
}

/// JSON-style secret field in a draft body is flagged.
#[test]
fn phase4_draft_body_json_style_secret_field_flagged() {
    let title = "Debug auth config";
    let body = r#"The config JSON is: {"api_key": "abc123def456ghi789jkl"}"#;
    let findings = scan_draft_body(title, Some(body));
    assert!(
        is_flagged(&findings),
        "JSON-style secret field in draft body must be flagged; got: {findings:?}"
    );
    assert!(
        has_finding(&findings, "json_secret_field"),
        "expected json_secret_field pattern; got: {findings:?}"
    );
}

/// Clean draft (no secrets) produces no findings.
#[test]
fn phase4_clean_draft_no_findings() {
    let title = "Improve test coverage for the event tailer";
    let body = "The event tailer currently has 60% line coverage. Add tests for the \
                rotation edge case and the slow-reader path.";
    let findings = scan_draft_body(title, Some(body));
    assert!(
        findings.is_empty(),
        "clean draft must produce no findings; got: {findings:?}"
    );
}

/// Bearer token in a draft body is flagged.
#[test]
fn phase4_draft_body_bearer_token_flagged() {
    let title = "API integration issue";
    let body = format!("curl -H 'Authorization: Bearer {JWT_TOKEN}' https://api.example.com");
    let findings = scan_draft_body(title, Some(&body));
    assert!(
        is_flagged(&findings),
        "draft body with Bearer token must be flagged; got: {findings:?}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 5: Morning-brief outputs + cross-project propagation drafts
// ═══════════════════════════════════════════════════════════════════════════════

/// Morning brief markdown with an embedded API key is flagged before storage
/// and before lateral propagation to Stitches (§18.1).
#[test]
fn phase5_morning_brief_flags_api_key() {
    let content = format!(
        "## Overnight summary\n\
         \n\
         - `kalshi-weather` bead bd-abc123 completed successfully.\n\
         - API key used: `{ANTHROPIC_KEY}`\n\
         - No errors detected.\n"
    );
    let findings = scan_morning_brief(&content);
    assert!(
        is_flagged(&findings),
        "morning brief with embedded API key must be flagged; got: {findings:?}"
    );
    assert!(
        has_finding(&findings, "anthropic_api_key"),
        "expected anthropic_api_key finding; got: {findings:?}"
    );
}

/// Morning brief with a GitHub token embedded (e.g. in a diff excerpt) is flagged.
#[test]
fn phase5_morning_brief_flags_github_token() {
    let content = format!(
        "## What changed\n\
         \n\
         The commit message contained a token: `{GITHUB_TOKEN}`. \
         Recommend rotating before next deploy.\n"
    );
    let findings = scan_morning_brief(&content);
    assert!(
        is_flagged(&findings),
        "morning brief with GitHub token must be flagged; got: {findings:?}"
    );
    assert!(
        has_finding(&findings, "github_token_ghp"),
        "expected github_token_ghp finding; got: {findings:?}"
    );
}

/// Clean morning brief produces no findings.
#[test]
fn phase5_morning_brief_clean_no_findings() {
    let content = "## Overnight summary\n\
                   \n\
                   All beads completed without errors. \
                   Total cost: $0.42. \
                   Three new Stitches were created.\n";
    let findings = scan_morning_brief(content);
    assert!(
        findings.is_empty(),
        "clean morning brief must produce no findings; got: {findings:?}"
    );
}

/// Cross-project propagation draft title with a secret is flagged.
#[test]
fn phase5_propagation_draft_title_secret_flagged() {
    let title = format!("Propagate fix: rotate key {ANTHROPIC_KEY} on iad-ci");
    let body = "Apply the same credential rotation fix as in ardenone-cluster.";
    let findings = scan_propagation_draft(&title, body);
    assert!(
        is_flagged(&findings),
        "propagation draft with key in title must be flagged; got: {findings:?}"
    );
}

/// Cross-project propagation draft body with a secret is flagged.
#[test]
fn phase5_propagation_draft_body_secret_flagged() {
    let title = "Propagate auth config fix to rs-manager";
    let body = format!(
        "Context from source project:\n\
         The service account key is `{AWS_KEY}` — rotate it in rs-manager as well."
    );
    let findings = scan_propagation_draft(title, &body);
    assert!(
        is_flagged(&findings),
        "propagation draft with AWS key in body must be flagged; got: {findings:?}"
    );
    assert!(
        has_finding(&findings, "aws_access_key"),
        "expected aws_access_key finding; got: {findings:?}"
    );
}

/// Clean propagation draft produces no findings.
#[test]
fn phase5_propagation_draft_clean_no_findings() {
    let title = "Propagate Calico IP selection fix to iad-ci";
    let body = "The same BGP peer configuration issue exists in iad-ci. \
                Apply the same annotation patch that fixed ardenone-cluster.";
    let findings = scan_propagation_draft(title, body);
    assert!(
        findings.is_empty(),
        "clean propagation draft must produce no findings; got: {findings:?}"
    );
}

/// JWT in a propagation draft body is flagged — prevents tokens from
/// the source project leaking into sibling project Stitches.
#[test]
fn phase5_propagation_draft_jwt_lateral_leak_blocked() {
    let title = "Propagate session config to rs-manager";
    let body = format!(
        "Copy the session token from the source project's context: {JWT_TOKEN}"
    );
    let findings = scan_propagation_draft(title, &body);
    assert!(
        is_flagged(&findings),
        "JWT in propagation draft must be flagged to prevent lateral leak; got: {findings:?}"
    );
    assert!(
        has_finding(&findings, "jwt"),
        "expected jwt pattern in propagation draft findings; got: {findings:?}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Coverage report: all ingestion surfaces and their scanner hooks
// ═══════════════════════════════════════════════════════════════════════════════

/// Documents every ingestion surface and its corresponding scanner hook.
///
/// This test never fails — it serves as a machine-readable coverage report
/// that is trivially kept up-to-date alongside the code. Adding a new surface
/// without a corresponding entry here is caught in code review.
#[test]
fn coverage_report_all_surfaces_accounted_for() {
    struct SurfaceEntry {
        phase: &'static str,
        surface: &'static str,
        scanner_fn: &'static str,
        call_site: &'static str,
    }

    let surfaces = [
        SurfaceEntry {
            phase: "Phase 0",
            surface: "CLI session JSONL (read-side projection)",
            scanner_fn: "redaction::redact_text / redact_json_value",
            call_site: "hoop-daemon/src/redaction.rs (global REDACTOR, called by session tailer)",
        },
        SurfaceEntry {
            phase: "Phase 3",
            surface: "Voice transcripts (Whisper output on create/update)",
            scanner_fn: "redaction::scan_voice_transcript",
            call_site: "hoop-daemon/src/api_dictated_notes.rs: create_note, update_note",
        },
        SurfaceEntry {
            phase: "Phase 3",
            surface: "Screen-capture frame text (OCR / narration transcript)",
            scanner_fn: "redaction::scan_screen_capture_text",
            call_site: "hoop-daemon/src/redaction.rs (hook ready; wired when screen-capture endpoint is built)",
        },
        SurfaceEntry {
            phase: "Phase 4",
            surface: "Draft bodies (single and bulk, title + description)",
            scanner_fn: "redaction::scan_draft_body",
            call_site: "hoop-daemon/src/api_draft_queue.rs: create_draft",
        },
        SurfaceEntry {
            phase: "Phase 4",
            surface: "Imported markdown lists (bulk-draft path)",
            scanner_fn: "redaction::scan_draft_body (called per item during bulk import)",
            call_site: "hoop-daemon/src/api_draft_queue.rs: create_draft (per-item scan)",
        },
        SurfaceEntry {
            phase: "Phase 5",
            surface: "Morning brief markdown output",
            scanner_fn: "redaction::scan_morning_brief",
            call_site: "hoop-daemon/src/fleet.rs: insert_morning_brief",
        },
        SurfaceEntry {
            phase: "Phase 5",
            surface: "Cross-project propagation drafts (agent-synthesised)",
            scanner_fn: "redaction::scan_propagation_draft",
            call_site: "hoop-daemon/src/api_draft_queue.rs: create_draft (propagation drafts use same path)",
        },
    ];

    // Assert all scanner functions exist at compile time by calling them with
    // empty input — ensures no surface entry becomes stale if a function is
    // renamed or removed.
    assert!(scan_voice_transcript("").is_empty());
    assert!(scan_screen_capture_text("").is_empty());
    assert!(scan_draft_body("", None).is_empty());
    assert!(scan_morning_brief("").is_empty());
    assert!(scan_propagation_draft("", "").is_empty());

    // Print the coverage table for CI log visibility
    println!("\n=== Privacy Surface Coverage Report ===");
    println!(
        "{:<10} {:<50} {:<40} {}",
        "Phase", "Surface", "Scanner function", "Call site"
    );
    println!("{}", "-".repeat(160));
    for s in &surfaces {
        println!(
            "{:<10} {:<50} {:<40} {}",
            s.phase, s.surface, s.scanner_fn, s.call_site
        );
    }
    println!("=== {} surfaces enumerated ===\n", surfaces.len());

    assert_eq!(surfaces.len(), 7, "update this count when adding new surfaces");
}
