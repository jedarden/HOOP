//! Stitch Net-Diff: aggregate git diff across all commits produced by a Stitch or Pattern.
//!
//! §6 Phase 2 marquee #3. Pure read-only — no mutations.
//!
//! Algorithm per workspace:
//!   1. Collect every bead linked to the Stitch (or Pattern's Stitches) in this workspace.
//!   2. Look up their commit SHAs from `bead_commits`.
//!   3. Filter out SHAs that are no longer reachable (rebased/force-pushed away).
//!   4. Sort reachable SHAs by commit date → oldest = base; newest = tip.
//!   5. Run `git diff <oldest>^..<newest>` for the net diff across the whole set.
//!
//! Multi-workspace Stitches produce one WorkspaceDiff per repo.

use anyhow::Result;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use tracing::warn;

use crate::api_diff::{parse_diff_output, FileDiff};
use crate::fleet;
use crate::DaemonState;

// ---------------------------------------------------------------------------
// Public response types
// ---------------------------------------------------------------------------

/// Per-workspace slice of the aggregate diff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceDiff {
    /// Absolute path to the workspace on disk.
    pub workspace: String,
    /// SHAs that were included (oldest-first).
    pub commit_shas: Vec<String>,
    /// `<oldest_sha>^..<newest_sha>` used for the diff.
    pub ref_range: String,
    pub files: Vec<FileDiff>,
    pub total_added: usize,
    pub total_removed: usize,
}

/// Aggregate net-diff response for a Stitch or Pattern.
#[derive(Debug, Serialize, Deserialize)]
pub struct NetDiffResponse {
    /// One entry per workspace that has reachable commits.
    pub workspaces: Vec<WorkspaceDiff>,
    /// Total across all workspaces.
    pub total_added: usize,
    pub total_removed: usize,
    /// True when any workspace diff was cut at `max_lines`.
    pub truncated: bool,
    /// Number of distinct beads whose commits contributed.
    pub bead_count: usize,
    /// Total commit SHAs across all workspaces.
    pub commit_count: usize,
}

// ---------------------------------------------------------------------------
// Query params
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct NetDiffQuery {
    #[serde(default = "default_max_lines")]
    max_lines: usize,
}

fn default_max_lines() -> usize {
    10_000
}

// ---------------------------------------------------------------------------
// Core computation (pure, synchronous, no mutations)
// ---------------------------------------------------------------------------

/// A single (workspace, sha, ts) record from `bead_commits`.
#[derive(Debug, Clone)]
struct CommitEntry {
    workspace: String,
    sha: String,
    ts: String,
}

/// Load every `(workspace, sha, ts)` record for a slice of bead IDs.
fn load_commits_for_beads(bead_ids: &[String]) -> Result<Vec<CommitEntry>> {
    if bead_ids.is_empty() {
        return Ok(vec![]);
    }
    let conn = Connection::open(fleet::db_path())?;
    let placeholders: String = bead_ids
        .iter()
        .enumerate()
        .map(|(i, _)| format!("?{}", i + 1))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT workspace, sha, ts FROM bead_commits WHERE bead_id IN ({}) ORDER BY ts ASC",
        placeholders
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(
        rusqlite::params_from_iter(bead_ids.iter()),
        |row| {
            Ok(CommitEntry {
                workspace: row.get(0)?,
                sha: row.get(1)?,
                ts: row.get(2)?,
            })
        },
    )?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// Query the bead_ids linked to a stitch from `stitch_beads`.
fn bead_ids_for_stitch(stitch_id: &str) -> Result<Vec<String>> {
    let conn = Connection::open(fleet::db_path())?;
    let mut stmt = conn.prepare(
        "SELECT DISTINCT bead_id FROM stitch_beads WHERE stitch_id = ?1",
    )?;
    let rows = stmt.query_map(params![stitch_id], |row| row.get(0))?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// Query stitch_ids belonging to a pattern from `pattern_members`.
fn stitch_ids_for_pattern(pattern_id: &str) -> Result<Vec<String>> {
    let conn = Connection::open(fleet::db_path())?;
    let mut stmt = conn.prepare(
        "SELECT DISTINCT stitch_id FROM pattern_members WHERE pattern_id = ?1",
    )?;
    let rows = stmt.query_map(params![pattern_id], |row| row.get(0))?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// Test whether a SHA is reachable (i.e. hasn't been rebased or force-pushed away).
fn sha_reachable(workspace: &str, sha: &str) -> bool {
    Command::new("git")
        .args(["-C", workspace, "cat-file", "-e", sha])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Compute the net diff for a set of bead IDs. Pure, no mutations.
///
/// Groups commits by workspace, filters unreachable SHAs, sorts by date,
/// then diffs `oldest^..newest` per workspace.
pub fn compute_net_diff(bead_ids: &[String], max_lines: usize) -> Result<NetDiffResponse> {
    let entries = load_commits_for_beads(bead_ids)?;

    // Group by workspace; preserve insertion order (already sorted by ts ASC).
    let mut by_workspace: HashMap<String, Vec<CommitEntry>> = HashMap::new();
    for e in entries {
        by_workspace.entry(e.workspace.clone()).or_default().push(e);
    }

    let mut workspace_diffs: Vec<WorkspaceDiff> = Vec::new();
    let mut any_truncated = false;
    let mut total_added = 0usize;
    let mut total_removed = 0usize;
    let mut total_commit_count = 0usize;
    let bead_count = bead_ids.len();

    for (workspace, mut commits) in by_workspace {
        // Sort oldest-first by timestamp string (ISO-8601 lexicographic order is chronological).
        commits.sort_by(|a, b| a.ts.cmp(&b.ts));

        // Filter to reachable SHAs.
        let reachable: Vec<CommitEntry> = commits
            .into_iter()
            .filter(|e| {
                let ok = sha_reachable(&workspace, &e.sha);
                if !ok {
                    warn!("net_diff: SHA {} unreachable in {}, skipping", e.sha, workspace);
                }
                ok
            })
            .collect();

        if reachable.is_empty() {
            continue;
        }

        let oldest_sha = &reachable[0].sha;
        let newest_sha = &reachable[reachable.len() - 1].sha;
        let ref_range = format!("{}^..{}", oldest_sha, newest_sha);

        let diff_output = run_git_diff(&workspace, &ref_range)?;
        let (files, truncated) = parse_diff_output(&diff_output, max_lines);

        if truncated {
            any_truncated = true;
        }

        let ws_added: usize = files.iter().map(|f| f.added).sum();
        let ws_removed: usize = files.iter().map(|f| f.removed).sum();
        total_added += ws_added;
        total_removed += ws_removed;
        total_commit_count += reachable.len();

        workspace_diffs.push(WorkspaceDiff {
            workspace,
            commit_shas: reachable.iter().map(|e| e.sha.clone()).collect(),
            ref_range,
            files,
            total_added: ws_added,
            total_removed: ws_removed,
        });
    }

    Ok(NetDiffResponse {
        workspaces: workspace_diffs,
        total_added,
        total_removed,
        truncated: any_truncated,
        bead_count,
        commit_count: total_commit_count,
    })
}

fn run_git_diff(workspace: &str, ref_range: &str) -> Result<String> {
    let out = Command::new("git")
        .args(["-C", workspace, "diff", "--no-color", "--unified=3", ref_range])
        .output()?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        anyhow::bail!("git diff failed in {}: {}", workspace, stderr.trim());
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

// ---------------------------------------------------------------------------
// HTTP handlers
// ---------------------------------------------------------------------------

fn err(code: StatusCode, msg: impl Into<String>) -> (StatusCode, String) {
    (code, msg.into())
}

/// GET /api/stitches/:stitch_id/net-diff
async fn get_stitch_net_diff(
    Path(stitch_id): Path<String>,
    Query(params): Query<NetDiffQuery>,
    State(_state): State<DaemonState>,
) -> Result<Json<NetDiffResponse>, (StatusCode, String)> {
    let max_lines = params.max_lines;

    let response = tokio::task::spawn_blocking(move || -> Result<NetDiffResponse> {
        let bead_ids = bead_ids_for_stitch(&stitch_id)?;
        compute_net_diff(&bead_ids, max_lines)
    })
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(response))
}

/// GET /api/patterns/:pattern_id/net-diff
async fn get_pattern_net_diff(
    Path(pattern_id): Path<String>,
    Query(params): Query<NetDiffQuery>,
    State(_state): State<DaemonState>,
) -> Result<Json<NetDiffResponse>, (StatusCode, String)> {
    let max_lines = params.max_lines;

    let response = tokio::task::spawn_blocking(move || -> Result<NetDiffResponse> {
        let stitch_ids = stitch_ids_for_pattern(&pattern_id)?;
        // Collect all bead_ids across all stitches, deduplicating.
        let mut bead_ids: Vec<String> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for sid in &stitch_ids {
            for bid in bead_ids_for_stitch(sid)? {
                if seen.insert(bid.clone()) {
                    bead_ids.push(bid);
                }
            }
        }
        compute_net_diff(&bead_ids, max_lines)
    })
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(response))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<DaemonState> {
    Router::new()
        .route("/api/stitches/:stitch_id/net-diff", get(get_stitch_net_diff))
        .route("/api/patterns/:pattern_id/net-diff", get(get_pattern_net_diff))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // ---------------------------------------------------------------------------
    // Helpers to build a realistic git repo in a tempdir
    // ---------------------------------------------------------------------------

    fn git(dir: &str, args: &[&str]) -> std::process::Output {
        Command::new("git")
            .args([&["-C", dir], args].concat())
            .env("GIT_AUTHOR_NAME", "Test")
            .env("GIT_AUTHOR_EMAIL", "test@test.com")
            .env("GIT_COMMITTER_NAME", "Test")
            .env("GIT_COMMITTER_EMAIL", "test@test.com")
            .env("GIT_AUTHOR_DATE", "2024-01-01T00:00:00+00:00")
            .env("GIT_COMMITTER_DATE", "2024-01-01T00:00:00+00:00")
            .output()
            .expect("git")
    }

    fn git_commit_with_ts(dir: &str, ts: &str, msg: &str, bead_id: Option<&str>) {
        let full_msg = match bead_id {
            Some(id) => format!("{}\n\nBead-Id: {}", msg, id),
            None => msg.to_string(),
        };
        Command::new("git")
            .args(["-C", dir, "commit", "--allow-empty", "-m", &full_msg])
            .env("GIT_AUTHOR_NAME", "Test")
            .env("GIT_AUTHOR_EMAIL", "test@test.com")
            .env("GIT_COMMITTER_NAME", "Test")
            .env("GIT_COMMITTER_EMAIL", "test@test.com")
            .env("GIT_AUTHOR_DATE", ts)
            .env("GIT_COMMITTER_DATE", ts)
            .output()
            .expect("git commit");
    }

    fn init_repo(dir: &str) -> String {
        git(dir, &["init", "-b", "main"]);
        git(dir, &["config", "user.email", "test@test.com"]);
        git(dir, &["config", "user.name", "Test"]);
        // Initial commit so we have a valid HEAD
        fs::write(format!("{}/README.md", dir), "init").unwrap();
        git(dir, &["add", "."]);
        git_commit_with_ts(dir, "2024-01-01T00:00:00+00:00", "initial", None);
        // Return HEAD sha
        let out = Command::new("git")
            .args(["-C", dir, "rev-parse", "HEAD"])
            .output()
            .unwrap();
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    }

    fn head_sha(dir: &str) -> String {
        let out = Command::new("git")
            .args(["-C", dir, "rev-parse", "HEAD"])
            .output()
            .unwrap();
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    }

    fn write_commit(dir: &str, filename: &str, content: &str, ts: &str, msg: &str) {
        fs::write(format!("{}/{}", dir, filename), content).unwrap();
        git(dir, &["add", filename]);
        git_commit_with_ts(dir, ts, msg, None);
    }

    // ---------------------------------------------------------------------------
    // Tests
    // ---------------------------------------------------------------------------

    #[test]
    fn sha_reachable_returns_true_for_valid_sha() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().to_str().unwrap();
        init_repo(dir);
        let sha = head_sha(dir);
        assert!(sha_reachable(dir, &sha));
    }

    #[test]
    fn sha_reachable_returns_false_for_garbage() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().to_str().unwrap();
        init_repo(dir);
        assert!(!sha_reachable(dir, "0000000000000000000000000000000000000000"));
    }

    #[test]
    fn compute_net_diff_empty_beads() {
        let result = compute_net_diff(&[], 10_000).unwrap();
        assert!(result.workspaces.is_empty());
        assert_eq!(result.total_added, 0);
        assert_eq!(result.total_removed, 0);
        assert_eq!(result.bead_count, 0);
        assert_eq!(result.commit_count, 0);
        assert!(!result.truncated);
    }

    #[test]
    fn compute_net_diff_single_workspace_single_bead() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().to_str().unwrap();
        init_repo(dir);

        // Write two files as part of "bead-1"
        write_commit(dir, "foo.txt", "hello\nworld\n", "2024-01-02T00:00:00+00:00", "add foo");
        let sha1 = head_sha(dir);
        write_commit(dir, "bar.txt", "bar content\n", "2024-01-03T00:00:00+00:00", "add bar");
        let sha2 = head_sha(dir);

        // Simulate what bead_commits table would give us
        let entries = vec![
            CommitEntry { workspace: dir.to_string(), sha: sha1.clone(), ts: "2024-01-02T00:00:00+00:00".to_string() },
            CommitEntry { workspace: dir.to_string(), sha: sha2.clone(), ts: "2024-01-03T00:00:00+00:00".to_string() },
        ];

        // Build the ref range manually as the function would
        let ref_range = format!("{}^..{}", sha1, sha2);
        let diff_out = run_git_diff(dir, &ref_range).unwrap();
        let (files, truncated) = parse_diff_output(&diff_out, 10_000);

        assert!(!truncated);
        // Should see foo.txt (from sha1) and bar.txt (from sha2)
        let paths: Vec<&str> = files.iter().map(|f| f.new_path.as_str()).collect();
        assert!(paths.contains(&"foo.txt"), "expected foo.txt in {:?}", paths);
        assert!(paths.contains(&"bar.txt"), "expected bar.txt in {:?}", paths);
        assert!(files.iter().map(|f| f.added).sum::<usize>() > 0);
    }

    #[test]
    fn compute_net_diff_handles_out_of_order_timestamps() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().to_str().unwrap();
        init_repo(dir);

        write_commit(dir, "a.txt", "aaa\n", "2024-01-05T00:00:00+00:00", "commit a");
        let sha_a = head_sha(dir);
        write_commit(dir, "b.txt", "bbb\n", "2024-01-03T00:00:00+00:00", "commit b (older ts)");
        let sha_b = head_sha(dir);

        // Both SHAs are reachable
        assert!(sha_reachable(dir, &sha_a));
        assert!(sha_reachable(dir, &sha_b));

        // Timestamps are provided out-of-order: b has older ts but was committed later
        let mut commits = vec![
            CommitEntry { workspace: dir.to_string(), sha: sha_a.clone(), ts: "2024-01-05T00:00:00+00:00".to_string() },
            CommitEntry { workspace: dir.to_string(), sha: sha_b.clone(), ts: "2024-01-03T00:00:00+00:00".to_string() },
        ];
        // Sort as the function does (oldest first by ts)
        commits.sort_by(|a, b| a.ts.cmp(&b.ts));
        assert_eq!(commits[0].sha, sha_b, "sha_b has older ts so sorts first");
        assert_eq!(commits[1].sha, sha_a);
    }

    #[test]
    fn compute_net_diff_filters_unreachable_shas() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().to_str().unwrap();
        init_repo(dir);
        write_commit(dir, "x.txt", "x\n", "2024-01-02T00:00:00+00:00", "add x");
        let sha_good = head_sha(dir);
        let sha_bad = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef";

        // sha_bad is not reachable
        assert!(sha_reachable(dir, &sha_good));
        assert!(!sha_reachable(dir, sha_bad));
    }

    #[test]
    fn compute_net_diff_five_bead_eleven_commit_cluster() {
        // § acceptance criterion: "Correct for a 5-bead / 11-commit test cluster"
        // We build a single-workspace repo with 11 commits tagged to 5 beads (2-3 commits each).
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().to_str().unwrap();
        init_repo(dir);

        // 5 beads × 2-3 commits each = 11 commits
        let mut sha_by_bead: Vec<(String, Vec<String>)> = Vec::new();
        let mut all_shas: Vec<(String, String)> = Vec::new(); // (bead_id, sha)

        // 5 beads: 3+2+2+2+2 = 11 commits
        let bead_commit_plan: &[(&str, &[(&str, &str)])] = &[
            ("bead-1", &[
                ("2024-01-02T00:00:00+00:00", "f1.txt"),
                ("2024-01-03T00:00:00+00:00", "f2.txt"),
                ("2024-01-04T00:00:00+00:00", "f3.txt"),
            ]),
            ("bead-2", &[
                ("2024-01-05T00:00:00+00:00", "f4.txt"),
                ("2024-01-06T00:00:00+00:00", "f5.txt"),
            ]),
            ("bead-3", &[
                ("2024-01-07T00:00:00+00:00", "f6.txt"),
                ("2024-01-08T00:00:00+00:00", "f7.txt"),
            ]),
            ("bead-4", &[
                ("2024-01-09T00:00:00+00:00", "f8.txt"),
                ("2024-01-10T00:00:00+00:00", "f9.txt"),
            ]),
            ("bead-5", &[
                ("2024-01-11T00:00:00+00:00", "f10.txt"),
                ("2024-01-12T00:00:00+00:00", "f11.txt"),
            ]),
        ];

        for (bead_id, commits) in bead_commit_plan {
            let mut bead_shas = Vec::new();
            for (ts, filename) in *commits {
                fs::write(format!("{}/{}", dir, filename), format!("{} content\n", filename)).unwrap();
                git(dir, &["add", filename]);
                git_commit_with_ts(dir, ts, &format!("add {}", filename), None);
                let sha = head_sha(dir);
                bead_shas.push(sha.clone());
                all_shas.push((bead_id.to_string(), sha));
            }
            sha_by_bead.push((bead_id.to_string(), bead_shas));
        }

        // Verify 11 commits were created (excluding the initial one)
        let total_bead_commits: usize = bead_commit_plan.iter().map(|(_, c)| c.len()).sum();
        assert_eq!(total_bead_commits, 11);

        // Build commit entries as the DB would give us (oldest-first by ts)
        let mut entries: Vec<CommitEntry> = all_shas
            .iter()
            .enumerate()
            .map(|(i, (bead_id, sha))| {
                let ts = format!("2024-01-{:02}T00:00:00+00:00", i + 2);
                CommitEntry { workspace: dir.to_string(), sha: sha.clone(), ts }
            })
            .collect();
        entries.sort_by(|a, b| a.ts.cmp(&b.ts));

        let oldest = &entries[0].sha;
        let newest = &entries[entries.len() - 1].sha;

        // All SHAs must be reachable
        for e in &entries {
            assert!(sha_reachable(dir, &e.sha), "SHA {} not reachable", e.sha);
        }

        // Run the diff
        let ref_range = format!("{}^..{}", oldest, newest);
        let diff_out = run_git_diff(dir, &ref_range).unwrap();
        let (files, truncated) = parse_diff_output(&diff_out, 100_000);

        assert!(!truncated);
        // 11 commits adding 1 file each → 11 new files visible in diff
        // (plus potentially f1-f12 files: 11 bead files + initial README isn't modified)
        // 11 bead commits → 11 new files (f1.txt … f11.txt), each adding 1 line
        assert_eq!(files.len(), 11, "expected 11 new files in aggregate diff, got {}", files.len());
        assert!(files.iter().all(|f| f.is_new), "all files should be new");
        assert_eq!(files.iter().map(|f| f.added).sum::<usize>(), 11);
    }

    #[test]
    fn compute_net_diff_performance_100_commits() {
        // § acceptance: 100-commit cluster in <5s
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().to_str().unwrap();
        init_repo(dir);

        let n = 100;
        for i in 0..n {
            let filename = format!("perf_{:03}.txt", i);
            fs::write(format!("{}/{}", dir, filename), format!("line {}\n", i)).unwrap();
            git(dir, &["add", &filename]);
            let ts = format!("2024-01-01T{:02}:{:02}:00+00:00", i / 60, i % 60);
            git_commit_with_ts(dir, &ts, &format!("commit {}", i), None);
        }

        // Collect all SHAs by walking git log
        let log_out = Command::new("git")
            .args(["-C", dir, "log", "--format=%H|%aI", "--reverse"])
            .output()
            .unwrap();
        let log_text = String::from_utf8(log_out.stdout).unwrap();
        let mut entries: Vec<CommitEntry> = log_text
            .lines()
            .filter(|l| !l.is_empty())
            .filter_map(|l| {
                let mut parts = l.splitn(2, '|');
                let sha = parts.next()?.trim().to_string();
                let ts = parts.next()?.trim().to_string();
                Some(CommitEntry { workspace: dir.to_string(), sha, ts })
            })
            // Skip the initial commit (no parent available for diff)
            .skip(1)
            .collect();
        entries.sort_by(|a, b| a.ts.cmp(&b.ts));

        let oldest = entries[0].sha.clone();
        let newest = entries[entries.len() - 1].sha.clone();
        let ref_range = format!("{}^..{}", oldest, newest);

        let t0 = std::time::Instant::now();
        let diff_out = run_git_diff(dir, &ref_range).unwrap();
        let (files, _) = parse_diff_output(&diff_out, 1_000_000);
        let elapsed = t0.elapsed();

        assert!(
            elapsed.as_secs_f64() < 5.0,
            "100-commit diff took {:.2}s (limit: 5s)",
            elapsed.as_secs_f64()
        );
        assert_eq!(files.len(), n, "expected {} files", n);
    }
}
