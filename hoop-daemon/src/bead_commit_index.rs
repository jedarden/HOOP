//! Bead-to-commit indexer: walks git log and maintains `bead_commits` in fleet.db.
//!
//! On startup performs a full walk of every registered workspace's git log, looking
//! for commits with a `Bead-Id:` trailer. Thereafter polls every 30 seconds and
//! indexes only commits newer than the stored HEAD cursor.
//!
//! ## Index schema (see migration 1.17.0 → 1.18.0 in fleet.rs)
//! - `bead_commits(bead_id, workspace, sha, ts)` — queryable by bead_id or sha
//! - `bead_commit_cursor(workspace, head_sha, indexed_at)` — tracks last-scanned HEAD

use anyhow::Result;
use rusqlite::{params, Connection};
use std::time::Duration;
use tokio::process::Command;
use tracing::{info, warn};

// ---------------------------------------------------------------------------
// Core async helpers
// ---------------------------------------------------------------------------

/// Get the current HEAD SHA for a workspace, returning Err if not a git repo.
async fn git_head(workspace: &str) -> Result<String> {
    let out = Command::new("git")
        .args(["-C", workspace, "rev-parse", "HEAD"])
        .output()
        .await?;
    if !out.status.success() {
        anyhow::bail!("git rev-parse HEAD failed in {}", workspace);
    }
    Ok(String::from_utf8(out.stdout)?.trim().to_string())
}

/// Parse git log output into `(sha, ts, bead_id)` triples.
///
/// Expected git format: `%H|%aI|%(trailers:key=Bead-Id,valueonly,separator=,)`
/// Lines without a Bead-Id trailer have an empty third field and are skipped.
fn parse_git_log(text: &str) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for line in text.lines() {
        let mut parts = line.splitn(3, '|');
        let sha = parts.next().unwrap_or("").trim();
        let ts = parts.next().unwrap_or("").trim();
        let bead_ids_str = parts.next().unwrap_or("").trim();
        if sha.is_empty() || ts.is_empty() || bead_ids_str.is_empty() {
            continue;
        }
        for bead_id in bead_ids_str.split(',') {
            let bead_id = bead_id.trim();
            if !bead_id.is_empty() {
                out.push((sha.to_string(), ts.to_string(), bead_id.to_string()));
            }
        }
    }
    out
}

/// Flush a batch of `(sha, ts, bead_id)` records plus an updated cursor into fleet.db.
fn flush_to_db(workspace: &str, commits: Vec<(String, String, String)>, head_sha: &str) -> Result<()> {
    let db_path = crate::fleet::db_path();
    let mut conn = Connection::open(&db_path)?;
    let tx = conn.transaction()?;
    for (sha, ts, bead_id) in &commits {
        tx.execute(
            "INSERT OR REPLACE INTO bead_commits (bead_id, workspace, sha, ts) VALUES (?1,?2,?3,?4)",
            params![bead_id, workspace, sha, ts],
        )?;
    }
    let now = chrono::Utc::now().to_rfc3339();
    tx.execute(
        "INSERT OR REPLACE INTO bead_commit_cursor (workspace, head_sha, indexed_at) VALUES (?1,?2,?3)",
        params![workspace, head_sha, now],
    )?;
    tx.commit()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Public indexing functions
// ---------------------------------------------------------------------------

/// Walk the full git log of `workspace` and upsert every bead-tagged commit.
///
/// Designed for startup: runs a single `git log` over the entire history.
/// A 10k-commit repo typically completes in 1–5 seconds.
pub async fn index_workspace(workspace: &str) -> Result<usize> {
    let ws = workspace.to_string();

    let out = Command::new("git")
        .args([
            "-C",
            &ws,
            "log",
            "--format=%H|%aI|%(trailers:key=Bead-Id,valueonly,separator=,)",
        ])
        .output()
        .await?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let s = stderr.trim();
        if !s.contains("not a git repository") && !s.contains("does not have any commits") {
            warn!("bead_commit_index: git log failed for {}: {}", ws, s);
        }
        return Ok(0);
    }

    let head_sha = git_head(&ws).await.unwrap_or_default();
    if head_sha.is_empty() {
        return Ok(0);
    }

    let text = String::from_utf8(out.stdout)?;
    let commits = parse_git_log(&text);
    let count = commits.len();

    let ws_db = ws.clone();
    let head_db = head_sha.clone();
    tokio::task::spawn_blocking(move || flush_to_db(&ws_db, commits, &head_db)).await??;

    Ok(count)
}

/// Incremental update: index only commits newer than the stored HEAD cursor.
///
/// Returns early (0) when HEAD has not changed since the last scan.
pub async fn incremental_index_workspace(workspace: &str) -> Result<usize> {
    let ws = workspace.to_string();

    let current_head = match git_head(&ws).await {
        Ok(h) => h,
        Err(_) => return Ok(0),
    };

    // Fetch stored cursor
    let ws_cursor = ws.clone();
    let cursor: Option<String> = tokio::task::spawn_blocking(move || -> Result<Option<String>> {
        let conn = Connection::open(crate::fleet::db_path())?;
        let result = conn
            .query_row(
                "SELECT head_sha FROM bead_commit_cursor WHERE workspace = ?1",
                params![ws_cursor],
                |row| row.get(0),
            )
            .ok();
        Ok(result)
    })
    .await??;

    if cursor.as_deref() == Some(current_head.as_str()) {
        return Ok(0); // nothing new
    }

    // No cursor → full walk
    let range = match &cursor {
        Some(prev) => format!("{}..HEAD", prev),
        None => return index_workspace(workspace).await,
    };

    let out = Command::new("git")
        .args([
            "-C",
            &ws,
            "log",
            &range,
            "--format=%H|%aI|%(trailers:key=Bead-Id,valueonly,separator=,)",
        ])
        .output()
        .await?;

    if !out.status.success() {
        warn!("bead_commit_index: incremental git log failed for {}", ws);
        return Ok(0);
    }

    let text = String::from_utf8(out.stdout)?;
    let commits = parse_git_log(&text);
    let count = commits.len();

    let ws_db = ws.clone();
    let head_db = current_head.clone();
    tokio::task::spawn_blocking(move || flush_to_db(&ws_db, commits, &head_db)).await??;

    Ok(count)
}

// ---------------------------------------------------------------------------
// Background task
// ---------------------------------------------------------------------------

/// Spawn the background indexer task.
///
/// - Full walk of each workspace runs immediately at startup.
/// - Incremental poll runs every 30 seconds thereafter.
pub fn spawn_indexer(workspaces: Vec<String>) {
    tokio::spawn(async move {
        let start = std::time::Instant::now();
        let mut total = 0usize;
        for ws in &workspaces {
            match index_workspace(ws).await {
                Ok(n) => total += n,
                Err(e) => warn!("bead_commit_index: full walk failed for {}: {}", ws, e),
            }
        }
        info!(
            "bead_commit_index: startup walk complete ({} bead-tagged commits, {:.1}s)",
            total,
            start.elapsed().as_secs_f64()
        );

        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            for ws in &workspaces {
                match incremental_index_workspace(ws).await {
                    Ok(n) if n > 0 => info!("bead_commit_index: {} new commits indexed in {}", n, ws),
                    Ok(_) => {}
                    Err(e) => warn!("bead_commit_index: incremental failed for {}: {}", ws, e),
                }
            }
        }
    });
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_git_log_extracts_bead_ids() {
        let input = "\
abc123|2024-01-15T10:30:00+00:00|hoop-ttb.3.35\n\
def456|2024-01-14T09:00:00+00:00|\n\
ghi789|2024-01-13T08:00:00+00:00|hoop-ttb.3.10,hoop-ttb.3.11\n\
";
        let results = parse_git_log(input);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0], ("abc123".to_string(), "2024-01-15T10:30:00+00:00".to_string(), "hoop-ttb.3.35".to_string()));
        // def456 is skipped (no bead id)
        assert_eq!(results[1].2, "hoop-ttb.3.10");
        assert_eq!(results[2].2, "hoop-ttb.3.11");
    }

    #[test]
    fn parse_git_log_skips_incomplete_lines() {
        let input = "onlysha\n||justpipes\n";
        let results = parse_git_log(input);
        assert!(results.is_empty());
    }

    #[test]
    fn parse_git_log_trims_whitespace() {
        let input = "sha1 | 2024-01-01T00:00:00+00:00 | hoop-ttb.1.1 \n";
        // splitn(3, '|') gives "sha1 ", " 2024-01-01T00:00:00+00:00 ", " hoop-ttb.1.1 "
        let results = parse_git_log(input);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "sha1");
        assert_eq!(results[0].2, "hoop-ttb.1.1");
    }
}
