//! Notes integration for HOOP — agent-readable markdown files (§22.4)
//!
//! Discovers notes from `~/.hoop/notes/<name>.md` (global) and
//! `<workspace>/.hoop/notes/<name>.md` (project-scoped).
//! Exposes `read_note` tool for agents.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Notes directory path
pub fn notes_dir() -> Result<PathBuf> {
    let mut path = dirs::home_dir().ok_or_else(|| anyhow!("Cannot determine home directory"))?;
    path.push(".hoop");
    path.push("notes");
    Ok(path)
}

/// A parsed note ready for reading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    /// Unique name (filename without .md)
    pub name: String,
    /// Title from frontmatter or derived from filename
    pub title: String,
    /// Description from frontmatter (if any)
    pub description: Option<String>,
    /// Tags from frontmatter (if any)
    pub tags: Option<Vec<String>>,
    /// Scope: global or project
    pub scope: NoteScope,
    /// Project name (if scope=project)
    pub project: Option<String>,
    /// Full markdown content
    pub body: String,
    /// File size in bytes
    pub size_bytes: u64,
}

/// Note scope
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NoteScope {
    /// Global note at ~/.hoop/notes/
    Global,
    /// Project-scoped note at <workspace>/.hoop/notes/
    Project,
}

/// Raw frontmatter parsed from YAML.
#[derive(Debug, Deserialize, Default)]
struct NoteFrontmatter {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
}

/// Parse a single note markdown file with optional YAML frontmatter.
fn parse_note_file(path: &Path, scope: NoteScope, project: Option<String>) -> Result<Note> {
    let content = fs::read_to_string(path)?;
    let metadata = fs::metadata(path)?;

    let (frontmatter, body) = split_frontmatter(&content);

    let fm: NoteFrontmatter = match frontmatter {
        Some(yaml) => serde_yaml::from_str(yaml)?,
        None => NoteFrontmatter::default(),
    };

    let filename_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let name = fm.name.unwrap_or_else(|| filename_name.clone());
    let title = fm.title.unwrap_or_else(|| filename_name.clone());

    Ok(Note {
        name,
        title,
        description: if fm.description.is_some() {
            fm.description
        } else {
            None
        },
        tags: if fm.tags.is_empty() {
            None
        } else {
            Some(fm.tags)
        },
        scope,
        project,
        body: body.trim().to_string(),
        size_bytes: metadata.len(),
    })
}

/// Split content into optional YAML frontmatter and body.
/// Frontmatter is enclosed between `---` delimiters.
fn split_frontmatter(content: &str) -> (Option<&str>, &str) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return (None, content);
    }
    // Find the closing ---
    let rest = &trimmed[3..];
    let rest = rest.trim_start_matches(['\r', '\n']);
    if let Some(end) = rest.find("\n---") {
        let yaml = &rest[..end];
        let body_start = &rest[end + 4..];
        let body_start = body_start.trim_start_matches(['\r', '\n']);
        (Some(yaml), body_start)
    } else {
        (None, content)
    }
}

/// Discover all global notes
pub fn discover_global_notes() -> Vec<Note> {
    let mut notes = Vec::new();

    let notes_dir = match notes_dir() {
        Ok(dir) => dir,
        Err(e) => {
            tracing::debug!("Failed to determine notes directory: {}", e);
            return notes;
        }
    };

    let Ok(entries) = fs::read_dir(&notes_dir) else {
        tracing::debug!(
            "Global notes directory not readable: {}",
            notes_dir.display()
        );
        return notes;
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();

        // Skip non-markdown files
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        match parse_note_file(&path, NoteScope::Global, None) {
            Ok(note) => {
                notes.push(note);
            }
            Err(e) => {
                tracing::warn!("Failed to parse note {}: {}", path.display(), e);
            }
        }
    }

    notes.sort_by(|a, b| a.name.cmp(&b.name));
    notes
}

/// Discover project-scoped notes for a specific project
pub fn discover_project_notes(project: &str, workspace: &Path) -> Vec<Note> {
    let mut notes = Vec::new();

    let project_notes_dir = workspace.join(".hoop").join("notes");

    if !project_notes_dir.exists() {
        // Project notes directory doesn't exist, that's fine
        return notes;
    }

    let Ok(entries) = fs::read_dir(&project_notes_dir) else {
        tracing::debug!(
            "Project notes directory not readable: {}",
            project_notes_dir.display()
        );
        return notes;
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();

        // Skip non-markdown files
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        match parse_note_file(&path, NoteScope::Project, Some(project.to_string())) {
            Ok(note) => {
                notes.push(note);
            }
            Err(e) => {
                tracing::warn!("Failed to parse project note {}: {}", path.display(), e);
            }
        }
    }

    notes.sort_by(|a, b| a.name.cmp(&b.name));
    notes
}

/// Read a note by name (global first, then all projects)
pub fn read_note_by_name(name: &str, projects: &HashMap<String, String>) -> Option<Note> {
    // Check global notes first
    for note in discover_global_notes() {
        if note.name == name {
            return Some(note);
        }
    }

    // Check all project notes
    for (project_name, workspace) in projects {
        for note in discover_project_notes(project_name, workspace.as_ref()) {
            if note.name == name {
                return Some(note);
            }
        }
    }

    None
}

/// List all available note names (for completion/validation)
pub fn list_note_names(projects: &HashMap<String, String>) -> Vec<String> {
    let mut names = Vec::new();

    // Global notes
    for note in discover_global_notes() {
        names.push(note.name);
    }

    // Project notes
    for (project_name, workspace) in projects {
        for note in discover_project_notes(project_name, workspace.as_ref()) {
            names.push(note.name);
        }
    }

    names.sort();
    names.dedup();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_frontmatter_with_yaml() {
        let content = "---\nname: test\ntitle: Test Note\n---\nBody text here";
        let (fm, body) = split_frontmatter(content);
        assert_eq!(fm, Some("name: test\ntitle: Test Note"));
        assert_eq!(body, "Body text here");
    }

    #[test]
    fn test_split_frontmatter_no_yaml() {
        let content = "Just a body\nNo frontmatter";
        let (fm, body) = split_frontmatter(content);
        assert!(fm.is_none());
        assert_eq!(body, content);
    }
}
