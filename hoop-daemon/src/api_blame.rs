//! Stitch-provenance file blame endpoint.
//!
//! `GET /api/projects/:project/files/blame?path=<filepath>`
//!
//! Runs `git blame --porcelain` on the file, then enriches each line with
//! Stitch attribution via the `bead_commits` index and `stitch_beads` join.
//!
//! Lines whose commit SHA is recorded in `bead_commits` get a `bead_id`,
//! and if that bead is linked to a Stitch via `stitch_beads`, they also
//! get `stitch_id` and `stitch_title`.  Lines with no bead trailer fall
//! back to raw git blame data (author + summary).

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{files, fleet, id_validators, DaemonState};

// ---------------------------------------------------------------------------
// Response type
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct BlameLine {
    pub line_no: u32,
    pub sha: String,
    pub author: String,
    pub ts: String,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bead_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stitch_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stitch_title: Option<String>,
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<DaemonState> {
    Router::new().route("/api/projects/:project/files/blame", get(get_file_blame))
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct BlameQuery {
    path: String,
}

async fn get_file_blame(
    Path(project): Path<String>,
    Query(params): Query<BlameQuery>,
    State(state): State<DaemonState>,
) -> Result<Json<Vec<BlameLine>>, (StatusCode, String)> {
    if id_validators::validate_project_name(&project).is_err() {
        return Err((StatusCode::BAD_REQUEST, "invalid project name".into()));
    }

    let project_root = {
        let projects = state.projects.read().unwrap();
        projects
            .iter()
            .find(|p| p.name == project)
            .map(|p| std::path::PathBuf::from(&p.path))
            .ok_or((StatusCode::NOT_FOUND, "project not found".into()))?
    };

    let rel_path = params.path.clone();
    if !files::is_safe_rel_path(&rel_path) {
        return Err((StatusCode::FORBIDDEN, "unsafe path".into()));
    }

    let lines = tokio::task::spawn_blocking(move || run_blame(&project_root, &rel_path))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(lines))
}

// ---------------------------------------------------------------------------
// Core blame logic (runs in blocking thread)
// ---------------------------------------------------------------------------

fn run_blame(project_root: &std::path::Path, rel_path: &str) -> anyhow::Result<Vec<BlameLine>> {
    let output = std::process::Command::new("git")
        .args([
            "-C",
            &project_root.display().to_string(),
            "blame",
            "--porcelain",
            "--",
            rel_path,
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git blame failed: {}", stderr.trim());
    }

    let text = String::from_utf8(output.stdout)?;
    let raw_lines = parse_porcelain_blame(&text);

    // Collect unique SHAs from the blame output.
    let unique_shas: Vec<String> = {
        let mut seen = std::collections::HashSet::new();
        raw_lines
            .iter()
            .map(|l| l.sha.clone())
            .filter(|sha| seen.insert(sha.clone()))
            .collect()
    };

    // sha → bead_id (from bead_commits index)
    let sha_to_bead = lookup_sha_to_bead(&unique_shas)?;

    // bead_id → (stitch_id, stitch_title)
    let unique_bead_ids: Vec<String> = {
        let mut seen = std::collections::HashSet::new();
        sha_to_bead
            .values()
            .filter(|b| seen.insert((*b).clone()))
            .cloned()
            .collect()
    };
    let bead_to_stitch = lookup_bead_to_stitch(&unique_bead_ids)?;

    let result = raw_lines
        .into_iter()
        .map(|raw| {
            let bead_id = sha_to_bead.get(&raw.sha).cloned();
            let stitch_info = bead_id
                .as_ref()
                .and_then(|bid| bead_to_stitch.get(bid));
            BlameLine {
                line_no: raw.line_no,
                sha: raw.sha,
                author: raw.author,
                ts: raw.ts,
                summary: raw.summary,
                bead_id,
                stitch_id: stitch_info.map(|(id, _)| id.clone()),
                stitch_title: stitch_info.map(|(_, t)| t.clone()),
            }
        })
        .collect();

    Ok(result)
}

// ---------------------------------------------------------------------------
// Porcelain blame parser
// ---------------------------------------------------------------------------

struct RawBlameLine {
    line_no: u32,
    sha: String,
    author: String,
    ts: String,
    summary: String,
}

/// Parse `git blame --porcelain` output into per-line records.
///
/// Porcelain format: each group starts with a 40-char SHA header line, then
/// metadata lines (`author`, `author-time`, `summary`, …), then a `\t`-prefixed
/// line containing the actual file content.  The same SHA group is reused for
/// repeated lines from the same commit.
fn parse_porcelain_blame(text: &str) -> Vec<RawBlameLine> {
    // sha → (author, ts, summary) cache so repeated-commit lines inherit metadata.
    let mut meta: HashMap<String, (String, String, String)> = HashMap::new();
    let mut cur_sha = String::new();
    let mut cur_line_no: u32 = 0;
    let mut cur_author = String::new();
    let mut cur_ts = String::new();
    let mut cur_summary = String::new();
    let mut out = Vec::new();

    for line in text.lines() {
        if line.starts_with('\t') {
            // Actual file content line — emit a BlameLine.
            let entry = meta
                .entry(cur_sha.clone())
                .or_insert_with(|| (cur_author.clone(), cur_ts.clone(), cur_summary.clone()));
            out.push(RawBlameLine {
                line_no: cur_line_no,
                sha: cur_sha.clone(),
                author: entry.0.clone(),
                ts: entry.1.clone(),
                summary: entry.2.clone(),
            });
        } else if line.starts_with("author ") && !line.starts_with("author-") {
            cur_author = line[7..].to_string();
            meta.entry(cur_sha.clone())
                .and_modify(|e| { if e.0.is_empty() { e.0 = cur_author.clone(); } });
        } else if line.starts_with("author-time ") {
            if let Ok(unix) = line[12..].trim().parse::<i64>() {
                cur_ts = format_unix_ts(unix);
                meta.entry(cur_sha.clone())
                    .and_modify(|e| { if e.1.is_empty() { e.1 = cur_ts.clone(); } });
            }
        } else if line.starts_with("summary ") {
            cur_summary = line[8..].to_string();
            meta.entry(cur_sha.clone())
                .and_modify(|e| { if e.2.is_empty() { e.2 = cur_summary.clone(); } });
        } else {
            // Commit header: <40-char sha> <orig-lineno> <final-lineno> [<count>]
            let parts: Vec<&str> = line.splitn(4, ' ').collect();
            if parts.len() >= 3
                && parts[0].len() == 40
                && parts[0].chars().all(|c| c.is_ascii_hexdigit())
            {
                cur_sha = parts[0].to_string();
                cur_line_no = parts[2].parse().unwrap_or(0);
                // Inherit cached metadata for repeated-commit lines.
                if let Some(cached) = meta.get(&cur_sha) {
                    cur_author = cached.0.clone();
                    cur_ts = cached.1.clone();
                    cur_summary = cached.2.clone();
                } else {
                    // Fresh commit — reset accumulators.
                    cur_author = String::new();
                    cur_ts = String::new();
                    cur_summary = String::new();
                }
            }
        }
    }

    out
}

fn format_unix_ts(unix: i64) -> String {
    use chrono::TimeZone;
    match chrono::Utc.timestamp_opt(unix, 0) {
        chrono::LocalResult::Single(dt) => dt.to_rfc3339(),
        _ => unix.to_string(),
    }
}

// ---------------------------------------------------------------------------
// DB lookups
// ---------------------------------------------------------------------------

/// Map each SHA to its most-recent bead_id in the bead_commits index.
fn lookup_sha_to_bead(shas: &[String]) -> anyhow::Result<HashMap<String, String>> {
    if shas.is_empty() {
        return Ok(HashMap::new());
    }
    let conn = Connection::open(fleet::db_path())?;
    let mut out = HashMap::new();
    for sha in shas {
        let result: Option<String> = conn
            .query_row(
                "SELECT bead_id FROM bead_commits WHERE sha = ?1 ORDER BY ts DESC LIMIT 1",
                params![sha],
                |row| row.get(0),
            )
            .ok();
        if let Some(bead_id) = result {
            out.insert(sha.clone(), bead_id);
        }
    }
    Ok(out)
}

/// Map each bead_id to the most-recently-active Stitch that references it.
fn lookup_bead_to_stitch(
    bead_ids: &[String],
) -> anyhow::Result<HashMap<String, (String, String)>> {
    if bead_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let conn = Connection::open(fleet::db_path())?;
    let mut out = HashMap::new();
    for bead_id in bead_ids {
        let result: Option<(String, String)> = conn
            .query_row(
                r#"
                SELECT sb.stitch_id, s.title
                FROM stitch_beads sb
                JOIN stitches s ON sb.stitch_id = s.id
                WHERE sb.bead_id = ?1
                ORDER BY s.last_activity_at DESC
                LIMIT 1
                "#,
                params![bead_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();
        if let Some(pair) = result {
            out.insert(bead_id.clone(), pair);
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_porcelain() {
        // Two lines: first from a bead-tagged commit, second from a different commit.
        // SHAs must be exactly 40 hex chars.
        let sha1 = "abcdef0123456789abcdef0123456789abcdef01";
        let sha2 = "fedcba9876543210fedcba9876543210fedcba98";
        let input = format!(
            "{sha1} 1 1 1\n\
author Alice\n\
author-time 1705316000\n\
summary Add feature\n\
filename src/lib.rs\n\
\tfn hello() {{}}\n\
{sha2} 2 2 1\n\
author Bob\n\
author-time 1705320000\n\
summary Fix bug\n\
filename src/lib.rs\n\
\t    println!(\"hi\");\n"
        );
        let lines = parse_porcelain_blame(&input);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].line_no, 1);
        assert_eq!(lines[0].author, "Alice");
        assert_eq!(lines[0].summary, "Add feature");
        assert_eq!(lines[1].line_no, 2);
        assert_eq!(lines[1].author, "Bob");
    }

    #[test]
    fn parse_repeated_commit_inherits_meta() {
        let sha = "a".repeat(40);
        let input = format!(
            "{sha} 1 1 2\nauthor Carol\nauthor-time 1705316000\nsummary Init\nfilename f\n\tline1\n\
             {sha} 2 2\nfilename f\n\tline2\n",
        );
        let lines = parse_porcelain_blame(&input);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[1].author, "Carol");
        assert_eq!(lines[1].summary, "Init");
    }
}
