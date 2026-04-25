//! Per-project file browser backend
//!
//! Lists directory contents lazily (one level at a time), respecting .gitignore
//! and .hoopignore files. Git status is derived from `git status --porcelain`.

use crate::path_security::{canonicalize_and_check, PathAllowlist};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;

/// Git working-tree status for a single node.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GitStatus {
    /// File is tracked and unchanged.
    Clean,
    /// File has unstaged or staged modifications.
    Modified,
    /// File is staged as new (added).
    Added,
    /// File has been deleted.
    Deleted,
    /// File is not tracked by git.
    Untracked,
    /// File was renamed.
    Renamed,
    /// Directory contains at least one dirty descendant.
    Dirty,
}

/// A single node in the file tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// Filename without path prefix.
    pub name: String,
    /// Path relative to the project root (forward-slash separated).
    pub path: String,
    pub is_dir: bool,
    /// File size in bytes; 0 for directories.
    pub size: u64,
    /// Modification time as Unix timestamp (seconds).
    pub mtime: i64,
    pub git_status: GitStatus,
}

/// Reject any relative path that contains `..` components (path traversal guard).
pub fn is_safe_rel_path(rel: &str) -> bool {
    PathBuf::from(rel)
        .components()
        .all(|c| !matches!(c, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
}

/// Run `git status --porcelain -- <filter>` and return a map from
/// repo-root-relative path to status.  Returns an empty map if the
/// directory is not inside a git repo or if git is unavailable.
fn git_status_map(project_root: &Path, rel_dir: &str) -> HashMap<String, GitStatus> {
    let filter = if rel_dir.is_empty() { "." } else { rel_dir };

    let output = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .arg("status")
        .arg("--porcelain")
        .arg("--")
        .arg(filter)
        .output();

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => return HashMap::new(),
    };

    let mut map = HashMap::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if line.len() < 3 {
            continue;
        }
        let xy = &line[..2];
        let rest = line[3..].trim();

        // Renames: "R old -> new" — use the destination path.
        let effective_path = if xy.starts_with('R') || xy.ends_with('R') {
            rest.split(" -> ").last().unwrap_or(rest)
        } else {
            rest
        };

        let index_char = xy.chars().next().unwrap_or(' ');
        let worktree_char = xy.chars().nth(1).unwrap_or(' ');

        let status = match index_char {
            'M' | 'U' => GitStatus::Modified,
            'A' => GitStatus::Added,
            'D' => GitStatus::Deleted,
            'R' => GitStatus::Renamed,
            '?' => GitStatus::Untracked,
            _ => match worktree_char {
                'M' => GitStatus::Modified,
                'D' => GitStatus::Deleted,
                _ => GitStatus::Clean,
            },
        };

        map.insert(effective_path.to_string(), status);
    }
    map
}

/// List the immediate children of `rel_dir` inside `project_root`.
///
/// Respects `.gitignore` and `.hoopignore` files in each directory.
/// Returns entries sorted: directories first, then alphabetically (case-insensitive).
///
/// Performs `canonicalize()` on the resolved directory and verifies it is
/// within the project workspace (§13 path-traversal hardening).  This catches
/// symlinks that point outside the project root even when `rel_dir` contains
/// no `..` components.
pub fn list_dir(project_root: &Path, rel_dir: &str) -> Result<Vec<FileEntry>> {
    let abs_dir = if rel_dir.is_empty() {
        project_root.to_path_buf()
    } else {
        project_root.join(rel_dir)
    };

    if !abs_dir.is_dir() {
        anyhow::bail!("not a directory");
    }

    // Build the allowlist from the project workspace root (pre-computed canonical).
    let allowlist = PathAllowlist::for_workspace(project_root)
        .context("failed to build path allowlist for project")?;

    // Realpath resolution + allowlist check — rejects symlink escapes (§13, §K2).
    canonicalize_and_check(&abs_dir, &allowlist)
        .map_err(|_| anyhow::anyhow!("directory not within project workspace"))?;

    let status_map = git_status_map(project_root, rel_dir);

    let walker = ignore::WalkBuilder::new(&abs_dir)
        .max_depth(Some(1))
        .hidden(false)
        .add_custom_ignore_filename(".hoopignore")
        .sort_by_file_path(std::path::Path::cmp)
        .build();

    let mut entries = Vec::new();

    for result in walker {
        let dir_entry = result.context("failed to read directory entry")?;

        // Depth 0 is the directory itself.
        if dir_entry.depth() == 0 {
            continue;
        }

        let path = dir_entry.path();

        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        let metadata = match path.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let is_dir = metadata.is_dir();
        let size = if is_dir { 0 } else { metadata.len() };

        let mtime = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        // Relative path from the project root (uses the OS separator, which on Linux is '/').
        let rel_path = path
            .strip_prefix(project_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        let git_status = if is_dir {
            let prefix = format!("{}/", rel_path);
            if status_map.keys().any(|k| k.starts_with(&prefix)) {
                GitStatus::Dirty
            } else {
                GitStatus::Clean
            }
        } else {
            status_map.get(&rel_path).cloned().unwrap_or(GitStatus::Clean)
        };

        entries.push(FileEntry {
            name,
            path: rel_path,
            is_dir,
            size,
            mtime,
            git_status,
        });
    }

    // Directories first, then alphabetical case-insensitive.
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(entries)
}

// ─── File Search ──────────────────────────────────────────────────────────────

/// First matching line from a content grep.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrepMatch {
    pub line_number: u64,
    /// The full matching line (trailing newline stripped).
    pub line: String,
    /// Byte offset of match start within `line`.
    pub match_start: usize,
    /// Byte offset of match end within `line`.
    pub match_end: usize,
}

/// A file-search result (flat, not tree-structured).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchResult {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub mtime: i64,
    pub git_status: GitStatus,
    /// Present when a content grep was performed.
    pub grep_match: Option<GrepMatch>,
}

/// Parse an extension filter string such as `"rs"`, `"ts,tsx"`, `"*.rs"`,
/// or `"*.{ts,tsx}"` into a list of bare extensions: `["rs"]`, `["ts","tsx"]`.
pub fn parse_ext_patterns(ext: &str) -> Vec<String> {
    let ext = ext.trim();
    if ext.is_empty() {
        return Vec::new();
    }

    // Strip leading `*.` or `.` from each token.
    let strip = |s: &str| s.trim().trim_start_matches('*').trim_start_matches('.').to_string();

    // Detect brace expansion: *.{ts,tsx} or {ts,tsx}
    if let (Some(open), Some(close)) = (ext.find('{'), ext.rfind('}')) {
        let inside = &ext[open + 1..close];
        return inside.split(',').map(|p| strip(p)).filter(|p| !p.is_empty()).collect();
    }

    // Plain comma-separated list.
    ext.split(',').map(|p| strip(p)).filter(|p| !p.is_empty()).collect()
}

/// Files changed between `ref_str` and the working tree/index.
fn get_modified_since(project_root: &Path, ref_str: &str) -> HashSet<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .arg("diff")
        .arg("--name-only")
        .arg(ref_str)
        .output();

    let mut changed = HashSet::new();
    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => return changed,
    };
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let t = line.trim();
        if !t.is_empty() {
            changed.insert(t.to_string());
        }
    }
    changed
}

/// Build a `FileSearchResult` from a relative path.
fn build_search_result(
    project_root: &Path,
    rel_path: &str,
    status_map: &HashMap<String, GitStatus>,
    grep_match: Option<GrepMatch>,
) -> Option<FileSearchResult> {
    let abs_path = project_root.join(rel_path);
    let metadata = abs_path.metadata().ok()?;
    if metadata.is_dir() {
        return None;
    }
    let name = abs_path.file_name()?.to_str()?.to_string();
    let size = metadata.len();
    let mtime = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let git_status = status_map.get(rel_path).cloned().unwrap_or(GitStatus::Clean);
    Some(FileSearchResult { path: rel_path.to_string(), name, size, mtime, git_status, grep_match })
}

const MAX_SEARCH_RESULTS: usize = 500;

/// Search files in a project applying any combination of extension, modified-since,
/// and content-grep filters.  Returns a flat, sorted list of matches.
pub fn search_files(
    project_root: &Path,
    ext_filter: &[String],
    modified_since: Option<&str>,
    grep_pattern: Option<&str>,
) -> Result<Vec<FileSearchResult>> {
    let allowlist = PathAllowlist::for_workspace(project_root)
        .context("failed to build path allowlist")?;
    canonicalize_and_check(project_root, &allowlist)
        .map_err(|_| anyhow::anyhow!("project root not within workspace"))?;

    let modified_set: Option<HashSet<String>> =
        modified_since.map(|r| get_modified_since(project_root, r));

    // Glob patterns for ripgrep  (e.g. ["*.rs", "*.ts"]).
    let glob_args: Vec<String> = ext_filter.iter().map(|e| format!("*.{}", e)).collect();

    if let Some(pattern) = grep_pattern {
        search_with_grep(project_root, pattern, &glob_args, modified_set.as_ref())
    } else {
        search_without_grep(project_root, ext_filter, modified_set.as_ref())
    }
}

fn search_with_grep(
    project_root: &Path,
    pattern: &str,
    glob_args: &[String],
    modified_set: Option<&HashSet<String>>,
) -> Result<Vec<FileSearchResult>> {
    let mut cmd = Command::new("rg");
    cmd.arg("--json").arg("--max-count").arg("1");
    for g in glob_args {
        cmd.arg("--glob").arg(g);
    }
    cmd.arg("--").arg(pattern).arg(project_root);

    let output = cmd.output().context("failed to run ripgrep")?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let status_map = git_status_map(project_root, "");
    let mut results: Vec<FileSearchResult> = Vec::new();

    for line in stdout.lines() {
        if results.len() >= MAX_SEARCH_RESULTS {
            break;
        }
        let value: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if value.get("type").and_then(|t| t.as_str()) != Some("match") {
            continue;
        }
        let data = match value.get("data") {
            Some(d) => d,
            None => continue,
        };

        // Absolute path from rg output.
        let abs_str = match data
            .get("path")
            .and_then(|p| p.get("text"))
            .and_then(|t| t.as_str())
        {
            Some(s) => s,
            None => continue,
        };
        let rel_path = match Path::new(abs_str).strip_prefix(project_root) {
            Ok(rel) => rel.to_string_lossy().into_owned(),
            Err(_) => continue,
        };

        if let Some(ms) = modified_set {
            if !ms.contains(&rel_path) {
                continue;
            }
        }

        let line_number = data.get("line_number").and_then(|n| n.as_u64()).unwrap_or(0);
        let line_text = data
            .get("lines")
            .and_then(|l| l.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .trim_end_matches('\n')
            .to_string();
        let (match_start, match_end) = data
            .get("submatches")
            .and_then(|sm| sm.as_array())
            .and_then(|arr| arr.first())
            .and_then(|m| {
                let s = m.get("start").and_then(|v| v.as_u64())? as usize;
                let e = m.get("end").and_then(|v| v.as_u64())? as usize;
                Some((s, e))
            })
            .unwrap_or((0, 0));

        let gm = GrepMatch { line_number, line: line_text, match_start, match_end };
        if let Some(r) = build_search_result(project_root, &rel_path, &status_map, Some(gm)) {
            results.push(r);
        }
    }

    results.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(results)
}

fn search_without_grep(
    project_root: &Path,
    ext_filter: &[String],
    modified_set: Option<&HashSet<String>>,
) -> Result<Vec<FileSearchResult>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .arg("ls-files")
        .arg("--cached")
        .arg("--others")
        .arg("--exclude-standard")
        .output()
        .context("git ls-files failed")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let status_map = git_status_map(project_root, "");
    let mut results: Vec<FileSearchResult> = Vec::new();

    for line in stdout.lines() {
        if results.len() >= MAX_SEARCH_RESULTS {
            break;
        }
        let rel_path = line.trim();
        if rel_path.is_empty() {
            continue;
        }

        // Extension filter.
        if !ext_filter.is_empty() {
            let ext = Path::new(rel_path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            if !ext_filter.iter().any(|f| f == ext) {
                continue;
            }
        }

        // Modified-since filter.
        if let Some(ms) = modified_set {
            if !ms.contains(rel_path) {
                continue;
            }
        }

        if let Some(r) = build_search_result(project_root, rel_path, &status_map, None) {
            results.push(r);
        }
    }

    results.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(results)
}
