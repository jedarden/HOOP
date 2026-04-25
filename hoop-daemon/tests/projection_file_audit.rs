//! Static audit: forbid projection-shaped file writes.
//!
//! Plan §3.1: "Events are authoritative; projections are derived. Never caches
//! to disk what can be rebuilt." Plan §3.2: "Liveness = process, never file.
//! HOOP never writes a worker_status.json."
//!
//! See also: notes/orchestrator-problems-and-solutions.md §A3–A4 — the "51 wedged
//! workers" incident traced directly to a stale projection file outliving the
//! events that produced it.
//!
//! This test:
//!   1. Scans all production Rust source files in the workspace for any string
//!      matching a forbidden projection-filename pattern.
//!   2. Fails with a diagnostic if any non-allowlisted match is found.
//!   3. Includes synthetic violation tests that prove the scanner itself works —
//!      if these fail, the scanner is broken.
//!
//! CI command:
//!   cargo test -p hoop-daemon --test projection_file_audit

use regex::Regex;
use std::path::{Path, PathBuf};

// ── Forbidden patterns ───────────────────────────────────────────────────────

/// Filename patterns whose presence in production source constitutes a violation.
///
/// Each entry is a regex that is matched against every line of every `.rs` file
/// in the production source tree.
///
/// Patterns and rationale:
///   `_status\.json`    — captures `*_status.json` (worker_status.json,
///                        fleet_status.json, …).  Status is liveness; liveness
///                        is derived from heartbeats, never a file.
///   `live-[^/\\]*\.json` — captures `live-*.json` (live-workers.json, …).
///                        "Live" implies real-time state — the same concern.
///   `fleet_state\.`    — captures `fleet_state.*` (fleet_state.json,
///                        fleet_state.yaml, …).  Fleet state is a projection of
///                        bead events; writing it to disk produces the class of
///                        bug documented in §A4.
const FORBIDDEN_PATTERNS: &[&str] = &[
    r"_status\.json",
    r"live-[^/\\]*\.json",
    r"fleet_state\.",
];

// ── Allowlist ────────────────────────────────────────────────────────────────

/// An exception to the projection-file audit.
///
/// Adding an entry here requires a `rationale` that proves the matched string
/// does **not** correspond to a runtime filesystem write of the forbidden file.
/// Reviewers must verify the rationale is accurate before approving.
struct AllowlistEntry {
    /// Substring of the file path (relative to workspace root) that identifies
    /// which file(s) this entry covers.  Empty string matches all files.
    file_contains: &'static str,
    /// If non-empty, only exempts lines that also contain this substring.
    /// Empty string exempts all lines in the matched file.
    line_contains: &'static str,
    /// Why this exception does not violate the projection-file invariant.
    #[allow(dead_code)]
    rationale: &'static str,
}

/// Current exceptions — empty because no production source currently references
/// the forbidden projection filenames.
///
/// To add an exception, append an `AllowlistEntry` here with a `rationale` that
/// explains why the matched string does not write the forbidden file at runtime.
/// Every entry must be reviewed; the empty-rationale template below shows the form.
///
/// Template:
/// ```
/// AllowlistEntry {
///     file_contains: "src/path/to/file.rs",
///     line_contains: "substring_of_safe_line",
///     rationale: "This string appears in a log/error message warning that we \
///                 never write the file, not as an actual write target.",
/// },
/// ```
const ALLOWLIST: &[AllowlistEntry] = &[
    // ── No current exceptions ────────────────────────────────────────────────
    //
    // Schema note: `hoop-schema/schemas/project_config_status.json` is a JSON
    // Schema definition file; its `$id` URL appears only in generated code under
    // `target/` (which is excluded from scanning) and never in hand-written Rust
    // source.  No allowlist entry is needed for it.
];

// ── Scanner implementation ───────────────────────────────────────────────────

/// A line that matched a forbidden pattern and was not suppressed by the allowlist.
#[derive(Debug)]
struct Violation {
    path: PathBuf,
    line_number: usize,
    line: String,
    pattern: String,
}

/// Scan `content` (attributed to `path` for diagnostics) and return all
/// violations not suppressed by `allowlist`.
fn scan_content_with_path(
    path: &Path,
    content: &str,
    patterns: &[Regex],
    allowlist: &[AllowlistEntry],
) -> Vec<Violation> {
    let path_str = path.to_string_lossy();
    let mut violations = Vec::new();

    for (idx, line) in content.lines().enumerate() {
        for pattern in patterns {
            if !pattern.is_match(line) {
                continue;
            }
            let allowed = allowlist.iter().any(|entry| {
                let file_ok = entry.file_contains.is_empty()
                    || path_str.contains(entry.file_contains);
                let line_ok = entry.line_contains.is_empty()
                    || line.contains(entry.line_contains);
                file_ok && line_ok
            });
            if !allowed {
                violations.push(Violation {
                    path: path.to_path_buf(),
                    line_number: idx + 1,
                    line: line.to_string(),
                    pattern: pattern.as_str().to_string(),
                });
            }
        }
    }

    violations
}

/// Recursively collect all `.rs` files under `dir`.
/// If `dir` is itself a `.rs` file, return it as a single-element vec.
fn collect_rs_files(dir: &Path) -> Vec<PathBuf> {
    if dir.is_file() {
        return if dir.extension().map(|e| e == "rs").unwrap_or(false) {
            vec![dir.to_path_buf()]
        } else {
            vec![]
        };
    }
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(collect_rs_files(&path));
            } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
                files.push(path);
            }
        }
    }
    files
}

/// Paths to scan.  Only production source directories are included; `tests/`
/// is excluded because test fixtures legitimately contain synthetic violation
/// strings to prove the scanner works.
fn production_source_paths(workspace_root: &Path) -> Vec<PathBuf> {
    vec![
        workspace_root.join("hoop-daemon/src"),
        workspace_root.join("hoop-cli/src"),
        workspace_root.join("hoop-mcp/src"),
        workspace_root.join("hoop-schema/src"),
        workspace_root.join("hoop-schema/build.rs"),
    ]
}

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR points to hoop-daemon/ (the package running this test).
    // The workspace root is one level up.
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    Path::new(&manifest_dir)
        .parent()
        .expect("workspace root is the parent of hoop-daemon/")
        .to_path_buf()
}

// ── Main workspace scan ──────────────────────────────────────────────────────

/// Verifies that no production Rust source contains a projection-shaped filename.
///
/// A projection-shaped filename (e.g., `worker_status.json`, `live-fleet.json`,
/// `fleet_state.yaml`) indicates that the code might be materialising derived state
/// to disk rather than computing it on demand from authoritative event rows.
/// This is the root cause of the §A4 "51 wedged workers" incident.
///
/// Exceptions must be added to `ALLOWLIST` with a documented rationale.
#[test]
fn no_projection_file_writes_in_workspace() {
    let patterns: Vec<Regex> = FORBIDDEN_PATTERNS
        .iter()
        .map(|p| Regex::new(p).expect("valid regex"))
        .collect();

    let workspace = workspace_root();
    let mut all_violations: Vec<Violation> = Vec::new();

    for source_path in production_source_paths(&workspace) {
        if !source_path.exists() {
            continue;
        }
        for file in collect_rs_files(&source_path) {
            let content = match std::fs::read_to_string(&file) {
                Ok(c) => c,
                Err(e) => panic!("failed to read {}: {}", file.display(), e),
            };
            // Use the path relative to workspace root for cleaner diagnostics.
            let relative = file.strip_prefix(&workspace).unwrap_or(&file);
            let violations =
                scan_content_with_path(relative, &content, &patterns, ALLOWLIST);
            all_violations.extend(violations);
        }
    }

    if all_violations.is_empty() {
        return;
    }

    let mut msg = format!(
        "\n{} projection-file violation(s) found.\n\n\
         Plan §3.1: events are authoritative; projections are derived —\n\
         never cache to disk what can be rebuilt from events.\n\
         See: notes/orchestrator-problems-and-solutions.md §A3–A4 (51 wedged workers).\n\n",
        all_violations.len()
    );
    for v in &all_violations {
        msg.push_str(&format!(
            "  {}:{}: [pattern `{}`]\n    {}\n\n",
            v.path.display(),
            v.line_number,
            v.pattern,
            v.line.trim(),
        ));
    }
    msg.push_str(
        "To add an exception, append an AllowlistEntry to ALLOWLIST in\n\
         hoop-daemon/tests/projection_file_audit.rs with a documented rationale.\n",
    );
    panic!("{}", msg);
}

// ── Synthetic violation tests: prove the scanner catches each pattern ────────
//
// These tests are the "synthetic PR" requirement from the bead spec.  If any of
// these fail, the scanner itself is broken and no longer guards against
// projection-file writes.

/// Synthetic PR: adding a `worker_state.json` write must be caught.
/// This is the primary acceptance criterion from the bead spec.
#[test]
fn scanner_detects_worker_state_json_write() {
    let patterns = compile_patterns();
    let path = Path::new("synthetic_worker_state_pr.rs");
    let violating_code = r#"
        fn cache_worker_state(state: &WorkerState) -> anyhow::Result<()> {
            let json = serde_json::to_vec(state)?;
            std::fs::write("worker_state.json", &json)?;
            Ok(())
        }
    "#;

    let violations = scan_content_with_path(path, violating_code, &patterns, &[]);

    assert!(
        !violations.is_empty(),
        "scanner failed to detect worker_state.json write — scanner is broken"
    );
    assert!(
        violations.iter().any(|v| v.line.contains("worker_state.json")),
        "violation must be on the worker_state.json line, got: {:?}",
        violations.iter().map(|v| v.line.trim()).collect::<Vec<_>>()
    );
}

/// Any `*_status.json` write is forbidden — this covers `fleet_status.json`.
#[test]
fn scanner_detects_fleet_status_json_write() {
    let patterns = compile_patterns();
    let path = Path::new("synthetic_fleet_status.rs");
    let code = r#"std::fs::write("fleet_status.json", &encoded).unwrap();"#;
    let violations = scan_content_with_path(path, code, &patterns, &[]);
    assert!(!violations.is_empty(), "fleet_status.json write must be detected");
}

/// `live-*.json` writes are forbidden — this covers `live-workers.json`.
#[test]
fn scanner_detects_live_dash_json_write() {
    let patterns = compile_patterns();
    let path = Path::new("synthetic_live.rs");
    let code = r#"std::fs::write("live-workers.json", &bytes).unwrap();"#;
    let violations = scan_content_with_path(path, code, &patterns, &[]);
    assert!(!violations.is_empty(), "live-workers.json write must be detected");
}

/// `fleet_state.*` writes are forbidden — this covers `fleet_state.json`.
#[test]
fn scanner_detects_fleet_state_write() {
    let patterns = compile_patterns();
    let path = Path::new("synthetic_fleet_state.rs");
    let code = r#"std::fs::File::create("fleet_state.json").unwrap();"#;
    let violations = scan_content_with_path(path, code, &patterns, &[]);
    assert!(!violations.is_empty(), "fleet_state.json create must be detected");
}

/// `fleet_state.yaml` is also forbidden — pattern is extension-agnostic.
#[test]
fn scanner_detects_fleet_state_yaml_write() {
    let patterns = compile_patterns();
    let path = Path::new("synthetic_fleet_state_yaml.rs");
    let code = r#"std::fs::write("fleet_state.yaml", &serialized).unwrap();"#;
    let violations = scan_content_with_path(path, code, &patterns, &[]);
    assert!(!violations.is_empty(), "fleet_state.yaml write must be detected");
}

/// `live-fleet.json` — a variant of the live- pattern.
#[test]
fn scanner_detects_live_fleet_json() {
    let patterns = compile_patterns();
    let path = Path::new("synthetic_live_fleet.rs");
    let code = r#"std::fs::write("live-fleet.json", data).unwrap();"#;
    let violations = scan_content_with_path(path, code, &patterns, &[]);
    assert!(!violations.is_empty(), "live-fleet.json write must be detected");
}

// ── Negative tests: innocent filenames must not trigger ──────────────────────

/// Filenames that look superficially similar but are not forbidden.
#[test]
fn scanner_ignores_innocent_json_filenames() {
    let patterns = compile_patterns();
    let path = Path::new("innocent.rs");
    let innocent_code = r#"
        std::fs::write("workers.json", &bytes);        // no _status suffix
        std::fs::write("fleet.json", &bytes);           // not fleet_state.
        std::fs::write("usage.json", &cached);          // capacity JSONL cache
        std::fs::write("state.json", &val);             // no fleet_ prefix
        std::fs::write("live_workers.json", &val);      // underscore not dash
        std::fs::write("livefleet.json", &val);         // no dash after "live"
        std::fs::write("fleet_state_backup", &val);     // no dot after fleet_state... wait
    "#;
    // Note: "fleet_state_backup" does NOT match r"fleet_state\." because there
    // is no dot — it ends in "_backup", not ".anything".
    let violations = scan_content_with_path(path, innocent_code, &patterns, &[]);
    assert!(
        violations.is_empty(),
        "innocent filenames must not trigger; violations on: {:?}",
        violations.iter().map(|v| v.line.trim()).collect::<Vec<_>>()
    );
}

// ── Allowlist mechanics ──────────────────────────────────────────────────────

/// The allowlist must suppress violations on matched file+line pairs.
#[test]
fn allowlist_suppresses_matched_violation() {
    let patterns = compile_patterns();
    let local_allowlist = &[AllowlistEntry {
        file_contains: "src/special_helper.rs",
        line_contains: "worker_status.json",
        rationale: "Unit test for allowlist suppression: no runtime write occurs here.",
    }];

    let path = Path::new("src/special_helper.rs");
    // A comment explaining why we never write the file — the pattern fires on the
    // filename string, but the allowlist suppresses it.
    let code = r#"// Invariant: we never write worker_status.json (plan §3.2)"#;
    let violations = scan_content_with_path(path, code, &patterns, local_allowlist);
    assert!(
        violations.is_empty(),
        "allowlist must suppress the matched violation; got: {:?}",
        violations.iter().map(|v| v.line.trim()).collect::<Vec<_>>()
    );
}

/// The allowlist must NOT suppress violations on non-matching files.
#[test]
fn allowlist_does_not_suppress_unmatched_file() {
    let patterns = compile_patterns();
    let local_allowlist = &[AllowlistEntry {
        file_contains: "src/special_helper.rs",
        line_contains: "worker_status.json",
        rationale: "Unit test for allowlist scoping.",
    }];

    // Different file — allowlist should NOT suppress it.
    let path = Path::new("src/other_module.rs");
    let code = r#"std::fs::write("worker_status.json", &data).unwrap();"#;
    let violations = scan_content_with_path(path, code, &patterns, local_allowlist);
    assert!(
        !violations.is_empty(),
        "allowlist must not suppress violations in non-matching files"
    );
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn compile_patterns() -> Vec<Regex> {
    FORBIDDEN_PATTERNS
        .iter()
        .map(|p| Regex::new(p).expect("valid regex"))
        .collect()
}
