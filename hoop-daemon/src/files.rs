//! Per-project file browser backend
//!
//! Lists directory contents lazily (one level at a time), respecting .gitignore
//! and .hoopignore files. Git status is derived from `git status --porcelain`.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
pub fn list_dir(project_root: &Path, rel_dir: &str) -> Result<Vec<FileEntry>> {
    let abs_dir = if rel_dir.is_empty() {
        project_root.to_path_buf()
    } else {
        project_root.join(rel_dir)
    };

    if !abs_dir.is_dir() {
        anyhow::bail!("not a directory: {}", abs_dir.display());
    }

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
