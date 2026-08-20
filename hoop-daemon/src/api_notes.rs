//! Notes API — structured knowledge files the agent can read (§22.4)
//!
//! Notes at `~/.hoop/notes/<name>.md` (global) or `<project-workspace>/.hoop/notes/<name>.md`
//! (project-scoped) are plain markdown files the agent can read via its `read_note(name)` tool.
//!
//! ## Use Cases
//!
//! - Project glossaries
//! - Team conventions ("we always prefer A over B")
//! - Reference material the operator wants the agent to have at hand
//! - Operator's own running notes on long-term work
//!
//! ## Note Format
//!
//! Notes are plain markdown files with optional YAML frontmatter:
//!
//! ```markdown
//! ---
//! title: Project Glossary
//! description: Common terms used in this project
//! tags: [glossary, reference]
//! ---
//!
//! # Glossary
//!
//! - **Widget**: A thing that does stuff
//! - **Gadget**: A thing that processes widgets
//! ```
//!
//! ## API Endpoints
//!
//! - `GET /api/notes` — list all notes (global or project-scoped)
//! - `GET /api/notes/:name` — get a single note by name
//! - `GET /api/notes/global` — list global notes only
//! - `GET /api/notes/project/:project` — list project-scoped notes

use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path as StdPath, PathBuf};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// A parsed note ready for API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    /// Unique name (filename without .md, or explicit `name` in frontmatter)
    pub name: String,
    /// Title from frontmatter or derived from filename
    pub title: String,
    /// Description from frontmatter (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Tags from frontmatter (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Scope: global or project
    pub scope: NoteScope,
    /// Project name (if scope=project)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    /// Full markdown content
    pub body: String,
    /// File size in bytes
    pub size_bytes: u64,
    /// Last modified timestamp (RFC3339)
    pub modified: String,
}

/// Note scope
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NoteScope {
    /// Global note at ~/.hoop/notes/
    Global,
    /// Project-scoped note at <project-workspace>/.hoop/notes/
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

/// Shared note store behind an RwLock.
pub type NoteStore = Arc<std::sync::RwLock<NoteLibrary>>;

/// In-memory collection of loaded notes.
#[derive(Debug, Clone, Default)]
pub struct NoteLibrary {
    /// Global notes
    global_notes: HashMap<String, Note>,
    /// Project-scoped notes: project -> notes map
    project_notes: HashMap<String, HashMap<String, Note>>,
}

impl NoteLibrary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load all global notes from the notes directory.
    pub fn load_global(&mut self, notes_dir: &StdPath) -> Result<()> {
        let mut notes = HashMap::new();

        let Ok(entries) = std::fs::read_dir(notes_dir) else {
            debug!(
                "Global notes directory not readable: {}",
                notes_dir.display()
            );
            return Ok(());
        };

        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();

            // Skip non-markdown files
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }

            match parse_note_file(&path, NoteScope::Global, None) {
                Ok(note) => {
                    notes.insert(note.name.clone(), note);
                }
                Err(e) => {
                    warn!("Failed to parse note {}: {}", path.display(), e);
                }
            }
        }

        info!("Global notes library loaded {} note(s)", notes.len());
        self.global_notes = notes;
        Ok(())
    }

    /// Load project-scoped notes.
    pub fn load_project(&mut self, project_name: &str, project_notes_dir: &StdPath) -> Result<()> {
        let mut notes = HashMap::new();

        if !project_notes_dir.exists() {
            // Project notes directory doesn't exist, that's fine
            return Ok(());
        }

        let Ok(entries) = std::fs::read_dir(project_notes_dir) else {
            debug!(
                "Project notes directory not readable: {}",
                project_notes_dir.display()
            );
            return Ok(());
        };

        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();

            // Skip non-markdown files
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }

            match parse_note_file(&path, NoteScope::Project, Some(project_name.to_string())) {
                Ok(note) => {
                    notes.insert(note.name.clone(), note);
                }
                Err(e) => {
                    warn!("Failed to parse project note {}: {}", path.display(), e);
                }
            }
        }

        info!(
            "Project '{}' notes loaded {} note(s)",
            project_name,
            notes.len()
        );
        self.project_notes.insert(project_name.to_string(), notes);
        Ok(())
    }

    /// Return all notes (global + all projects).
    pub fn list_all(&self) -> Vec<Note> {
        let mut notes = self.global_notes.values().cloned().collect::<Vec<_>>();
        for project_notes in self.project_notes.values() {
            notes.extend(project_notes.values().cloned());
        }
        notes.sort_by(|a, b| a.name.cmp(&b.name));
        notes
    }

    /// Return global notes only.
    pub fn list_global(&self) -> Vec<Note> {
        let mut notes = self.global_notes.values().cloned().collect::<Vec<_>>();
        notes.sort_by(|a, b| a.name.cmp(&b.name));
        notes
    }

    /// Return project-scoped notes for a specific project.
    pub fn list_project(&self, project_name: &str) -> Vec<Note> {
        self.project_notes
            .get(project_name)
            .map(|notes| {
                let mut notes = notes.values().cloned().collect::<Vec<_>>();
                notes.sort_by(|a, b| a.name.cmp(&b.name));
                notes
            })
            .unwrap_or_default()
    }

    /// Get a single note by name (global first, then projects).
    pub fn get(&self, name: &str) -> Option<Note> {
        // Check global first
        if let Some(note) = self.global_notes.get(name) {
            return Some(note.clone());
        }
        // Check all projects
        for project_notes in self.project_notes.values() {
            if let Some(note) = project_notes.get(name) {
                return Some(note.clone());
            }
        }
        None
    }

    /// Get a global note by name.
    pub fn get_global(&self, name: &str) -> Option<Note> {
        self.global_notes.get(name).cloned()
    }

    /// Get a project-scoped note by name.
    pub fn get_project(&self, project_name: &str, name: &str) -> Option<Note> {
        self.project_notes
            .get(project_name)
            .and_then(|notes| notes.get(name).cloned())
    }
}

/// Parse a single note markdown file with optional YAML frontmatter.
fn parse_note_file(path: &StdPath, scope: NoteScope, project: Option<String>) -> Result<Note> {
    let content = std::fs::read_to_string(path)?;
    let metadata = std::fs::metadata(path)?;

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

    // Get modified time
    let modified = metadata
        .modified()
        .ok()
        .and_then(|t| {
            let secs_since_epoch = t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
            chrono::DateTime::from_timestamp(secs_since_epoch as i64, 0)
        })
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_default();

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
        modified,
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

/// Start file watcher for the global notes directory.
/// Returns the watcher (must be kept alive for watching to work).
pub fn start_global_watcher(notes_dir: PathBuf, store: NoteStore) -> RecommendedWatcher {
    let watcher_notes_dir = notes_dir.clone();
    let mut watcher =
        notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| match res {
            Ok(_event) => {
                debug!("Global notes directory changed, reloading");
                let mut lib = store.write().unwrap();
                if let Err(e) = lib.load_global(&watcher_notes_dir) {
                    warn!("Global notes reload failed: {}", e);
                }
            }
            Err(e) => {
                warn!("Global notes watch error: {}", e);
            }
        })
        .expect("failed to create notes file watcher");

    if notes_dir.exists() {
        watcher
            .watch(&notes_dir, RecursiveMode::NonRecursive)
            .unwrap_or_else(|e| warn!("Cannot watch global notes dir: {}", e));
    }

    info!("Global notes file watcher started");
    watcher
}

/// Ensure the global notes directory exists.
pub fn ensure_notes_dir(notes_dir: &StdPath) -> PathBuf {
    if !notes_dir.exists() {
        std::fs::create_dir_all(notes_dir).unwrap_or_else(|e| {
            warn!("Failed to create global notes directory: {}", e);
        });
    }

    // Seed example notes if directory is empty
    let has_files = std::fs::read_dir(notes_dir)
        .map(|mut d| d.next().is_some())
        .unwrap_or(false);

    if !has_files {
        seed_example_notes(notes_dir);
    }

    notes_dir.to_path_buf()
}

fn seed_example_notes(dir: &StdPath) {
    let examples = [
        (
            "team-conventions.md",
            r#"---
title: Team Conventions
description: How we work together
tags: [conventions, workflow]
---
# Team Conventions

## Code Review
- All PRs require at least one approval
- Use "Request Changes" for blocking issues
- Comments should be constructive and specific

## Testing
- Write tests for new features
- Run tests before committing
- Test coverage should not decrease

## Communication
- Use async communication for most things
- Sync only when complex discussion is needed
- Document decisions in the repo
"#,
        ),
        (
            "glossary.md",
            r#"---
title: Project Glossary
description: Common terms and acronyms
tags: [glossary, reference]
---
# Glossary

## Terms
- **HOOP**: Human-Operator-Oriented Platform — the control-plane daemon
- **NEEDLE**: The worker supervision system
- **Stitch**: A single conversation within a project
- **Bead**: NEEDLE's internal execution unit

## Acronyms
- **MCP**: Model Context Protocol
- **API**: Application Programming Interface
- **CLI**: Command Line Interface
"#,
        ),
    ];

    for (filename, content) in &examples {
        let path = dir.join(filename);
        if let Err(e) = crate::atomic_write::atomic_write_file_str(&path, content) {
            warn!("Failed to seed example note {}: {}", filename, e);
        }
    }
    info!("Seeded {} example notes", examples.len());
}

// ---------------------------------------------------------------------------
// REST API
// ---------------------------------------------------------------------------

/// GET /api/notes — list all notes (global + all projects)
async fn list_notes(State(state): State<crate::DaemonState>) -> Json<Vec<Note>> {
    let lib = state.note_library.read().unwrap();
    Json(lib.list_all())
}

/// GET /api/notes/global — list global notes only
async fn list_global_notes(State(state): State<crate::DaemonState>) -> Json<Vec<Note>> {
    let lib = state.note_library.read().unwrap();
    Json(lib.list_global())
}

/// GET /api/notes/project/:project — list project-scoped notes
async fn list_project_notes(
    Path(project): Path<String>,
    State(state): State<crate::DaemonState>,
) -> Json<Vec<Note>> {
    let lib = state.note_library.read().unwrap();
    Json(lib.list_project(&project))
}

/// GET /api/notes/:name — get a single note by name
async fn get_note(
    Path(name): Path<String>,
    State(state): State<crate::DaemonState>,
) -> Result<Json<Note>, (StatusCode, String)> {
    let lib = state.note_library.read().unwrap();
    lib.get(&name)
        .map(Json)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Note '{}' not found", name)))
}

pub fn router() -> Router<crate::DaemonState> {
    Router::new()
        .route("/api/notes", get(list_notes))
        .route("/api/notes/global", get(list_global_notes))
        .route("/api/notes/project/:project", get(list_project_notes))
        .route("/api/notes/:name", get(get_note))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

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

    #[test]
    fn test_parse_note_file_with_frontmatter() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("my-note.md");
        std::fs::write(
            &file_path,
            r#"---
name: test-note
title: Test Note
description: A test note
tags:
  - example
  - test
---
## Body
Test content
"#,
        )
        .unwrap();

        let note = parse_note_file(&file_path, NoteScope::Global, None).unwrap();
        assert_eq!(note.name, "test-note");
        assert_eq!(note.title, "Test Note");
        assert_eq!(note.description, Some("A test note".to_string()));
        assert_eq!(
            note.tags,
            Some(vec!["example".to_string(), "test".to_string()])
        );
        assert!(note.body.contains("Test content"));
    }

    #[test]
    fn test_parse_note_file_no_frontmatter() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("plain-note.md");
        std::fs::write(&file_path, "Just a body\n").unwrap();

        let note = parse_note_file(&file_path, NoteScope::Global, None).unwrap();
        assert_eq!(note.name, "plain-note");
        assert_eq!(note.title, "plain-note");
        assert!(note.description.is_none());
        assert!(note.tags.is_none());
        assert_eq!(note.body, "Just a body");
    }

    #[test]
    fn test_library_list_and_get() {
        let mut lib = NoteLibrary::new();
        lib.global_notes = {
            let mut m = HashMap::new();
            m.insert(
                "note1".to_string(),
                Note {
                    name: "note1".to_string(),
                    title: "First".to_string(),
                    description: None,
                    tags: None,
                    scope: NoteScope::Global,
                    project: None,
                    body: "Body 1".to_string(),
                    size_bytes: 100,
                    modified: "2024-01-01T00:00:00Z".to_string(),
                },
            );
            m.insert(
                "note2".to_string(),
                Note {
                    name: "note2".to_string(),
                    title: "Second".to_string(),
                    description: None,
                    tags: None,
                    scope: NoteScope::Global,
                    project: None,
                    body: "Body 2".to_string(),
                    size_bytes: 200,
                    modified: "2024-01-01T00:00:00Z".to_string(),
                },
            );
            m
        };

        assert_eq!(lib.list_all().len(), 2);
        assert!(lib.get("note1").is_some());
        assert!(lib.get("nonexistent").is_none());
    }

    #[test]
    fn test_library_scoped_notes() {
        let mut lib = NoteLibrary::new();
        lib.global_notes.insert(
            "global".to_string(),
            Note {
                name: "global".to_string(),
                title: "Global".to_string(),
                description: None,
                tags: None,
                scope: NoteScope::Global,
                project: None,
                body: "Global body".to_string(),
                size_bytes: 100,
                modified: "2024-01-01T00:00:00Z".to_string(),
            },
        );

        let mut proj_notes = HashMap::new();
        proj_notes.insert(
            "project".to_string(),
            Note {
                name: "project".to_string(),
                title: "Project".to_string(),
                description: None,
                tags: None,
                scope: NoteScope::Project,
                project: Some("myproject".to_string()),
                body: "Project body".to_string(),
                size_bytes: 100,
                modified: "2024-01-01T00:00:00Z".to_string(),
            },
        );
        lib.project_notes
            .insert("myproject".to_string(), proj_notes);

        assert_eq!(lib.list_all().len(), 2);
        assert_eq!(lib.list_global().len(), 1);
        assert_eq!(lib.list_project("myproject").len(), 1);
        assert_eq!(lib.list_project("other").len(), 0);
    }

    #[test]
    fn test_note_scope_serialization() {
        let global = NoteScope::Global;
        assert_eq!(serde_json::to_value(global).unwrap(), "global");

        let project = NoteScope::Project;
        assert_eq!(serde_json::to_value(project).unwrap(), "project");
    }
}
