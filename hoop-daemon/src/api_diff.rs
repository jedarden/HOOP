//! Git diff API — structured JSON diffs for working tree vs HEAD or any two refs.

use anyhow::{Context, Result};
use axum::{routing::get, Router};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

use crate::files::is_safe_rel_path;
use crate::DaemonState;

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DiffLineKind {
    Context,
    Add,
    Remove,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    /// Content of the line (without the leading +/-/space prefix).
    pub content: String,
    /// Line number in the old (left) file; None for pure additions.
    pub old_lineno: Option<u32>,
    /// Line number in the new (right) file; None for pure removals.
    pub new_lineno: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub old_start: u32,
    pub old_count: u32,
    pub new_start: u32,
    pub new_count: u32,
    /// Full @@ … @@ header line.
    pub header: String,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    pub old_path: String,
    pub new_path: String,
    pub is_new: bool,
    pub is_deleted: bool,
    pub is_binary: bool,
    pub added: usize,
    pub removed: usize,
    pub hunks: Vec<DiffHunk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResponse {
    pub files: Vec<FileDiff>,
    pub total_added: usize,
    pub total_removed: usize,
    /// True when the output was cut at max_lines.
    pub truncated: bool,
    /// The ref range that was diffed (e.g. "HEAD" or "abc123..HEAD").
    pub ref_range: String,
}

/// Merge-base SHA response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeBaseResponse {
    pub sha: Option<String>,
    pub upstream: String,
}

// ─── Query params ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct DiffQuery {
    #[serde(rename = "ref")]
    ref_: Option<String>,
    ref2: Option<String>,
    file: Option<String>,
    #[serde(default = "default_max_lines")]
    max_lines: usize,
}

fn default_max_lines() -> usize {
    10_000
}

#[derive(Debug, Deserialize)]
struct MergeBaseQuery {
    #[serde(default = "default_upstream")]
    upstream: String,
}

fn default_upstream() -> String {
    "main".to_string()
}

// ─── Diff parsing ─────────────────────────────────────────────────────────────

fn parse_hunk_range(s: &str) -> (u32, u32) {
    let s = s.trim_start_matches(['-', '+']);
    if let Some((a, b)) = s.split_once(',') {
        (a.parse().unwrap_or(1), b.parse().unwrap_or(0))
    } else {
        (s.parse().unwrap_or(1), 1)
    }
}

fn parse_diff_output(output: &str, max_lines: usize) -> (Vec<FileDiff>, bool) {
    let mut files: Vec<FileDiff> = Vec::new();
    let mut truncated = false;

    let mut current_file: Option<FileDiff> = None;
    let mut current_hunk: Option<DiffHunk> = None;
    let mut old_lineno = 0u32;
    let mut new_lineno = 0u32;
    let mut total_lines = 0usize;

    fn flush_hunk(current_hunk: &mut Option<DiffHunk>, current_file: &mut Option<FileDiff>) {
        if let (Some(h), Some(f)) = (current_hunk.take(), current_file.as_mut()) {
            f.hunks.push(h);
        }
    }
    fn flush_file(
        current_hunk: &mut Option<DiffHunk>,
        current_file: &mut Option<FileDiff>,
        files: &mut Vec<FileDiff>,
    ) {
        flush_hunk(current_hunk, current_file);
        if let Some(f) = current_file.take() {
            files.push(f);
        }
    }

    for line in output.lines() {
        if total_lines >= max_lines {
            flush_file(&mut current_hunk, &mut current_file, &mut files);
            truncated = true;
            break;
        }

        if line.starts_with("diff --git ") {
            flush_file(&mut current_hunk, &mut current_file, &mut files);
            current_file = Some(FileDiff {
                old_path: String::new(),
                new_path: String::new(),
                is_new: false,
                is_deleted: false,
                is_binary: false,
                added: 0,
                removed: 0,
                hunks: Vec::new(),
            });
            continue;
        }

        let Some(ref mut file) = current_file else {
            continue;
        };

        if line.starts_with("--- ") {
            let p = line[4..].trim_start_matches("a/");
            file.old_path = if p == "/dev/null" {
                String::new()
            } else {
                p.to_string()
            };
            continue;
        }
        if line.starts_with("+++ ") {
            let p = line[4..].trim_start_matches("b/");
            file.new_path = if p == "/dev/null" {
                String::new()
            } else {
                p.to_string()
            };
            if file.old_path.is_empty() {
                file.is_new = true;
            }
            if file.new_path.is_empty() {
                file.is_deleted = true;
            }
            continue;
        }
        if line.starts_with("Binary files") {
            file.is_binary = true;
            continue;
        }
        if line.starts_with("new file") {
            file.is_new = true;
            continue;
        }
        if line.starts_with("deleted file") {
            file.is_deleted = true;
            continue;
        }

        if line.starts_with("@@ ") {
            flush_hunk(&mut current_hunk, &mut current_file);
            let current_file = current_file.as_mut().unwrap();
            let mut parts = line.splitn(5, ' ');
            parts.next(); // "@@"
            let old_range = parts.next().unwrap_or("-0,0");
            let new_range = parts.next().unwrap_or("+0,0");
            let (old_start, old_count) = parse_hunk_range(old_range);
            let (new_start, new_count) = parse_hunk_range(new_range);
            old_lineno = old_start;
            new_lineno = new_start;
            current_hunk = Some(DiffHunk {
                old_start,
                old_count,
                new_start,
                new_count,
                header: line.to_string(),
                lines: Vec::new(),
            });
            let _ = current_file; // suppress unused warning
            continue;
        }

        let Some(ref mut hunk) = current_hunk else {
            continue;
        };

        let first_byte = line.as_bytes().first().copied();
        match first_byte {
            Some(b'+') => {
                let Some(ref mut file) = current_file else { continue };
                hunk.lines.push(DiffLine {
                    kind: DiffLineKind::Add,
                    content: line[1..].to_string(),
                    old_lineno: None,
                    new_lineno: Some(new_lineno),
                });
                file.added += 1;
                new_lineno += 1;
                total_lines += 1;
            }
            Some(b'-') => {
                let Some(ref mut file) = current_file else { continue };
                hunk.lines.push(DiffLine {
                    kind: DiffLineKind::Remove,
                    content: line[1..].to_string(),
                    old_lineno: Some(old_lineno),
                    new_lineno: None,
                });
                file.removed += 1;
                old_lineno += 1;
                total_lines += 1;
            }
            Some(b' ') => {
                hunk.lines.push(DiffLine {
                    kind: DiffLineKind::Context,
                    content: line[1..].to_string(),
                    old_lineno: Some(old_lineno),
                    new_lineno: Some(new_lineno),
                });
                old_lineno += 1;
                new_lineno += 1;
                total_lines += 1;
            }
            _ => {} // index, mode, etc.
        }
    }

    flush_file(&mut current_hunk, &mut current_file, &mut files);
    (files, truncated)
}

// ─── Git helpers ──────────────────────────────────────────────────────────────

fn run_git_diff(project_root: &Path, ref_range: &str, file: Option<&str>) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.arg("-C")
        .arg(project_root)
        .arg("diff")
        .arg("--no-color")
        .arg("--unified=3")
        .arg(ref_range);
    if let Some(f) = file {
        cmd.arg("--").arg(f);
    }
    let output = cmd.output().context("failed to run git diff")?;
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn run_git_merge_base(project_root: &Path, upstream: &str) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .arg("merge-base")
        .arg("HEAD")
        .arg(upstream)
        .output()
        .ok()?;
    if output.status.success() {
        let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if sha.is_empty() { None } else { Some(sha) }
    } else {
        None
    }
}

fn validate_ref_arg(r: &str) -> bool {
    !r.is_empty()
        && !r.contains(|c: char| {
            matches!(c, ';' | '|' | '&' | '$' | '`' | '\n' | '\r' | ' ')
        })
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

/// GET /api/projects/:project/diff
///
/// Query params:
///   ref  — git ref to diff working tree against (default: HEAD)
///   ref2 — if set, compare ref..ref2 (ref-to-ref mode; working tree not involved)
///   file — optional relative file path to narrow the diff
///   max_lines — safety cap on total lines returned (default: 10 000)
async fn get_project_diff(
    axum::extract::Path(project): axum::extract::Path<String>,
    axum::extract::Query(params): axum::extract::Query<DiffQuery>,
    axum::extract::State(state): axum::extract::State<DaemonState>,
) -> Result<axum::Json<DiffResponse>, (axum::http::StatusCode, String)> {
    use crate::id_validators;

    id_validators::validate_project_name(&project).map_err(id_validators::rejection)?;

    let project_root = {
        let projects = state.projects.read().unwrap();
        projects
            .iter()
            .find(|p| p.name == project)
            .map(|p| std::path::PathBuf::from(&p.path))
            .ok_or_else(|| {
                (
                    axum::http::StatusCode::NOT_FOUND,
                    "Project not found".to_string(),
                )
            })?
    };

    let ref1 = params.ref_.as_deref().unwrap_or("HEAD");
    if !validate_ref_arg(ref1) {
        return Err((axum::http::StatusCode::BAD_REQUEST, "Invalid ref".to_string()));
    }
    if let Some(r2) = &params.ref2 {
        if !validate_ref_arg(r2) {
            return Err((
                axum::http::StatusCode::BAD_REQUEST,
                "Invalid ref2".to_string(),
            ));
        }
    }
    if let Some(f) = &params.file {
        if !is_safe_rel_path(f) {
            return Err((
                axum::http::StatusCode::BAD_REQUEST,
                "Invalid file path".to_string(),
            ));
        }
    }

    let ref1 = ref1.to_string();
    let ref2 = params.ref2.clone();
    let file = params.file.clone();
    let max_lines = params.max_lines;

    let (diff_output, ref_range) =
        tokio::task::spawn_blocking(move || -> Result<(String, String)> {
            let range = match &ref2 {
                Some(r2) => format!("{}..{}", ref1, r2),
                None => ref1.clone(),
            };
            let out = run_git_diff(&project_root, &range, file.as_deref())?;
            Ok((out, range))
        })
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let (files, truncated) = parse_diff_output(&diff_output, max_lines);
    let total_added: usize = files.iter().map(|f| f.added).sum();
    let total_removed: usize = files.iter().map(|f| f.removed).sum();

    Ok(axum::Json(DiffResponse {
        files,
        total_added,
        total_removed,
        truncated,
        ref_range,
    }))
}

/// GET /api/projects/:project/diff/merge-base?upstream=main
///
/// Computes `git merge-base HEAD <upstream>` and returns the SHA.
/// Useful for three-way diff: the UI fetches this SHA, then requests
/// two diffs (merge-base..HEAD and HEAD..working).
async fn get_merge_base(
    axum::extract::Path(project): axum::extract::Path<String>,
    axum::extract::Query(params): axum::extract::Query<MergeBaseQuery>,
    axum::extract::State(state): axum::extract::State<DaemonState>,
) -> Result<axum::Json<MergeBaseResponse>, (axum::http::StatusCode, String)> {
    use crate::id_validators;

    id_validators::validate_project_name(&project).map_err(id_validators::rejection)?;

    if !validate_ref_arg(&params.upstream) {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            "Invalid upstream".to_string(),
        ));
    }

    let project_root = {
        let projects = state.projects.read().unwrap();
        projects
            .iter()
            .find(|p| p.name == project)
            .map(|p| std::path::PathBuf::from(&p.path))
            .ok_or_else(|| {
                (
                    axum::http::StatusCode::NOT_FOUND,
                    "Project not found".to_string(),
                )
            })?
    };

    let upstream = params.upstream.clone();
    let sha = tokio::task::spawn_blocking(move || {
        run_git_merge_base(&project_root, &upstream)
    })
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(axum::Json(MergeBaseResponse {
        sha,
        upstream: params.upstream,
    }))
}

// ─── Router ───────────────────────────────────────────────────────────────────

pub fn router() -> Router<DaemonState> {
    Router::new()
        .route("/api/projects/:project/diff", get(get_project_diff))
        .route(
            "/api/projects/:project/diff/merge-base",
            get(get_merge_base),
        )
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_DIFF: &str = r#"diff --git a/src/main.rs b/src/main.rs
index abc1234..def5678 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -10,7 +10,8 @@ fn main() {
     let x = 1;
-    println!("old");
+    println!("new");
+    println!("extra");
     let y = 2;
     let z = 3;
     let w = 4;
     let v = 5;
"#;

    #[test]
    fn test_parse_diff_basic() {
        let (files, truncated) = parse_diff_output(SAMPLE_DIFF, 10_000);
        assert!(!truncated);
        assert_eq!(files.len(), 1);
        let f = &files[0];
        assert_eq!(f.old_path, "src/main.rs");
        assert_eq!(f.new_path, "src/main.rs");
        assert_eq!(f.added, 2);
        assert_eq!(f.removed, 1);
        assert_eq!(f.hunks.len(), 1);

        let hunk = &f.hunks[0];
        assert_eq!(hunk.old_start, 10);
        assert_eq!(hunk.new_start, 10);

        let kinds: Vec<&DiffLineKind> = hunk.lines.iter().map(|l| &l.kind).collect();
        assert!(kinds.contains(&&DiffLineKind::Add));
        assert!(kinds.contains(&&DiffLineKind::Remove));
        assert!(kinds.contains(&&DiffLineKind::Context));
    }

    #[test]
    fn test_parse_diff_truncation() {
        let (files, truncated) = parse_diff_output(SAMPLE_DIFF, 2);
        assert!(truncated);
        // We should still get the partial result
        assert!(!files.is_empty());
    }

    #[test]
    fn test_parse_hunk_range() {
        assert_eq!(parse_hunk_range("-10,7"), (10, 7));
        assert_eq!(parse_hunk_range("+10,8"), (10, 8));
        assert_eq!(parse_hunk_range("-5"), (5, 1));
    }

    #[test]
    fn test_validate_ref_arg() {
        assert!(validate_ref_arg("HEAD"));
        assert!(validate_ref_arg("HEAD~1"));
        assert!(validate_ref_arg("main"));
        assert!(validate_ref_arg("abc123def"));
        assert!(!validate_ref_arg(""));
        assert!(!validate_ref_arg("HEAD; rm -rf /"));
        assert!(!validate_ref_arg("main|cat /etc/passwd"));
        assert!(!validate_ref_arg("HEAD\nnew line"));
    }

    #[test]
    fn test_parse_new_file_diff() {
        let diff = r#"diff --git a/new.txt b/new.txt
new file mode 100644
index 0000000..abc1234
--- /dev/null
+++ b/new.txt
@@ -0,0 +1,3 @@
+line 1
+line 2
+line 3
"#;
        let (files, _) = parse_diff_output(diff, 10_000);
        assert_eq!(files.len(), 1);
        assert!(files[0].is_new);
        assert_eq!(files[0].added, 3);
        assert_eq!(files[0].removed, 0);
    }

    #[test]
    fn test_parse_deleted_file_diff() {
        let diff = r#"diff --git a/old.txt b/old.txt
deleted file mode 100644
index abc1234..0000000
--- a/old.txt
+++ /dev/null
@@ -1,2 +0,0 @@
-line 1
-line 2
"#;
        let (files, _) = parse_diff_output(diff, 10_000);
        assert_eq!(files.len(), 1);
        assert!(files[0].is_deleted);
        assert_eq!(files[0].removed, 2);
    }

    #[test]
    fn test_line_numbers_tracked() {
        let (files, _) = parse_diff_output(SAMPLE_DIFF, 10_000);
        let hunk = &files[0].hunks[0];
        // First context line should be old_lineno=10, new_lineno=10
        let ctx = hunk.lines.iter().find(|l| l.kind == DiffLineKind::Context).unwrap();
        assert_eq!(ctx.old_lineno, Some(10));
        assert_eq!(ctx.new_lineno, Some(10));
        // Removed line has old_lineno but no new_lineno
        let rem = hunk.lines.iter().find(|l| l.kind == DiffLineKind::Remove).unwrap();
        assert!(rem.old_lineno.is_some());
        assert!(rem.new_lineno.is_none());
        // Added line has new_lineno but no old_lineno
        let add = hunk.lines.iter().find(|l| l.kind == DiffLineKind::Add).unwrap();
        assert!(add.old_lineno.is_none());
        assert!(add.new_lineno.is_some());
    }
}
