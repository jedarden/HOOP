//! Prompts API — reusable prompt library with parameter substitution (§22.5)
//!
//! Prompts at `~/.hoop/prompts/<name>.md` are reusable prompt bodies that can be
//! referenced by name (`@prompt:<name>`) and substituted with parameters like
//! `{{project}}`, `{{file}}`, `{{stitch}}`, `{{now}}`, plus operator-passed args.
//!
//! ## Prompt Format
//!
//! ```markdown
//! ---
//! name: fix-linting
//! description: "Fix a linting violation in a file"
//! args:
//!   - lint_type
//!   - severity
//! ---
//! ## Task
//! Fix {{lint_type}} linting error ({{severity}}) in {{file}}.
//!
//! ## Acceptance
//! - Linter passes for {{file}}
//! - No new violations introduced
//! ```
//!
//! ## API Endpoints
//!
//! - `GET /api/prompts` — list all prompts
//! - `GET /api/prompts/:name` — get a single prompt
//! - `POST /api/prompts/:name/substitute` — substitute variables in a prompt

use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path as StdPath, PathBuf};
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::prompt_substitute::substitute_with_args;

/// A parsed prompt ready for API responses.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Prompt {
    /// Unique name (filename without .md, or explicit `name` in frontmatter)
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Body template with {{var}} placeholders
    pub body: String,
    /// Variables extracted from the body
    pub variables: Vec<String>,
    /// Optional argument schema (for validation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
}

/// Substitution request
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SubstitutionRequest {
    /// Built-in variables
    #[serde(default)]
    pub project: Option<String>,
    #[serde(default)]
    pub file: Option<String>,
    #[serde(default)]
    pub stitch: Option<String>,
    /// Custom operator-passed arguments
    #[serde(default)]
    pub args: HashMap<String, serde_json::Value>,
}

/// Substitution response
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SubstitutionResponse {
    /// Substituted prompt body
    pub body: String,
    /// Variables that were substituted
    pub substituted: Vec<String>,
}

/// Raw frontmatter parsed from YAML.
#[derive(Debug, Deserialize, Default)]
struct PromptFrontmatter {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    args: Vec<String>,
}

/// Shared prompt store behind an RwLock.
pub type PromptStore = Arc<std::sync::RwLock<PromptLibrary>>;

/// In-memory collection of loaded prompts.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PromptLibrary {
    prompts: HashMap<String, Prompt>,
}

impl PromptLibrary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load all prompts from the prompts directory.
    pub fn load(&mut self, prompts_dir: &StdPath) -> Result<()> {
        let mut prompts = HashMap::new();

        let Ok(entries) = std::fs::read_dir(prompts_dir) else {
            debug!("Prompts directory not readable: {}", prompts_dir.display());
            return Ok(());
        };

        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();

            // Skip non-markdown files
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }

            match parse_prompt_file(&path) {
                Ok(prompt) => {
                    prompts.insert(prompt.name.clone(), prompt);
                }
                Err(e) => {
                    warn!("Failed to parse prompt {}: {}", path.display(), e);
                }
            }
        }

        info!("Prompt library loaded {} prompt(s)", prompts.len());
        self.prompts = prompts;
        Ok(())
    }

    /// Return all prompts.
    pub fn list(&self) -> Vec<Prompt> {
        self.prompts.values().cloned().collect()
    }

    /// Get a single prompt by name.
    pub fn get(&self, name: &str) -> Option<Prompt> {
        self.prompts.get(name).cloned()
    }
}

/// Parse a single prompt markdown file with YAML frontmatter.
fn parse_prompt_file(path: &StdPath) -> Result<Prompt> {
    let content = std::fs::read_to_string(path)?;
    let (frontmatter, body) = split_frontmatter(&content);

    let fm: PromptFrontmatter = match frontmatter {
        Some(yaml) => serde_yaml::from_str(yaml)?,
        None => PromptFrontmatter::default(),
    };

    let name = fm.name.unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
    });

    // Extract variables from the body
    let variables = crate::prompt_substitute::extract_variables(body);

    Ok(Prompt {
        name,
        description: fm.description.unwrap_or_default(),
        body: body.trim().to_string(),
        variables,
        args: if fm.args.is_empty() {
            None
        } else {
            Some(fm.args)
        },
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

/// Start file watcher for the prompts directory.
/// Returns the watcher (must be kept alive for watching to work).
pub fn start_watcher(prompts_dir: PathBuf, store: PromptStore) -> RecommendedWatcher {
    let watcher_prompts_dir = prompts_dir.clone();
    let mut watcher =
        notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| match res {
            Ok(_event) => {
                debug!("Prompt directory changed, reloading");
                let mut lib = store.write().unwrap();
                if let Err(e) = lib.load(&watcher_prompts_dir) {
                    warn!("Prompt reload failed: {}", e);
                }
            }
            Err(e) => {
                warn!("Prompt watch error: {}", e);
            }
        })
        .expect("failed to create prompt file watcher");

    if prompts_dir.exists() {
        watcher
            .watch(&prompts_dir, RecursiveMode::NonRecursive)
            .unwrap_or_else(|e| warn!("Cannot watch prompts dir: {}", e));
    }

    info!("Prompt file watcher started");
    watcher
}

/// Ensure the prompts directory exists.
pub fn ensure_prompts_dir(prompts_dir: &StdPath) -> PathBuf {
    if !prompts_dir.exists() {
        std::fs::create_dir_all(prompts_dir).unwrap_or_else(|e| {
            warn!("Failed to create prompts directory: {}", e);
        });
    }

    // Seed example prompts if directory is empty
    let has_files = std::fs::read_dir(prompts_dir)
        .map(|mut d| d.next().is_some())
        .unwrap_or(false);

    if !has_files {
        seed_example_prompts(prompts_dir);
    }

    prompts_dir.to_path_buf()
}

fn seed_example_prompts(dir: &StdPath) {
    let examples = [
        (
            "fix-linting.md",
            r#"---
name: fix-linting
description: "Fix a linting violation in a file"
args:
  - lint_type
  - severity
---
## Task
Fix {{lint_type}} linting error ({{severity}}) in {{file}}.

## Acceptance
- Linter passes for {{file}}
- No new violations introduced
"#,
        ),
        (
            "write-plan-stub.md",
            r#"---
name: write-plan-stub
description: "Create a plan.md stub for a project"
---
## Task
Write a plan.md stub for {{project}}.

## Outline
- Vision
- Requirements
- Architecture overview
- Implementation phases
- Testing strategy
"#,
        ),
        (
            "investigate-error.md",
            r#"---
name: investigate-error
description: "Investigate an error in a file"
args:
  - error_message
---
## Task
Investigate error in {{file}}: {{error_message}}

## Steps
1. Read the file to understand context
2. Identify the root cause
3. Propose a fix
4. Check for similar issues elsewhere
"#,
        ),
    ];

    for (filename, content) in &examples {
        let path = dir.join(filename);
        if let Err(e) = crate::atomic_write::atomic_write_file_str(&path, content) {
            warn!("Failed to seed example prompt {}: {}", filename, e);
        }
    }
    info!("Seeded {} example prompts", examples.len());
}

// ---------------------------------------------------------------------------
// REST API
// ---------------------------------------------------------------------------

/// GET /api/prompts — list all prompts
async fn list_prompts(State(state): State<crate::DaemonState>) -> Json<Vec<Prompt>> {
    let lib = state.prompt_library.read().unwrap();
    Json(lib.list())
}

/// GET /api/prompts/:name — get a single prompt by name
async fn get_prompt(
    Path(name): Path<String>,
    State(state): State<crate::DaemonState>,
) -> Result<Json<Prompt>, (StatusCode, String)> {
    let lib = state.prompt_library.read().unwrap();
    lib.get(&name).map(Json).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            format!("Prompt '{}' not found", name),
        )
    })
}

/// POST /api/prompts/:name/substitute — substitute variables in a prompt
async fn substitute_prompt(
    Path(name): Path<String>,
    State(state): State<crate::DaemonState>,
    Json(req): Json<SubstitutionRequest>,
) -> Result<Json<SubstitutionResponse>, (StatusCode, String)> {
    let lib = state.prompt_library.read().unwrap();
    let prompt = lib.get(&name).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            format!("Prompt '{}' not found", name),
        )
    })?;

    let args_json = serde_json::to_value(&req.args).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to serialize args: {}", e),
        )
    })?;

    match substitute_with_args(
        &prompt.body,
        req.project.as_deref(),
        req.file.as_deref(),
        req.stitch.as_deref(),
        &args_json,
    ) {
        Ok(body) => {
            let substituted = crate::prompt_substitute::extract_variables(&prompt.body);
            Ok(Json(SubstitutionResponse { body, substituted }))
        }
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            format!("Substitution failed: {}", e),
        )),
    }
}

pub fn router() -> Router<crate::DaemonState> {
    Router::new()
        .route("/api/prompts", get(list_prompts))
        .route("/api/prompts/{name}", get(get_prompt))
        .route("/api/prompts/{name}/substitute", post(substitute_prompt))
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
        let content = "---\nname: test\ndescription: hello\n---\nBody text here";
        let (fm, body) = split_frontmatter(content);
        assert_eq!(fm, Some("name: test\ndescription: hello"));
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
    fn test_parse_prompt_file_with_frontmatter() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("my-prompt.md");
        std::fs::write(
            &file_path,
            r#"---
name: test-prompt
description: A test prompt
args:
  - foo
  - bar
---
## Body
Test {{foo}} with {{bar}}
"#,
        )
        .unwrap();

        let prompt = parse_prompt_file(&file_path).unwrap();
        assert_eq!(prompt.name, "test-prompt");
        assert_eq!(prompt.description, "A test prompt");
        assert_eq!(
            prompt.args,
            Some(vec!["foo".to_string(), "bar".to_string()])
        );
        assert!(prompt.body.contains("Test {{foo}}"));
        assert_eq!(prompt.variables, vec!["bar", "foo"]); // sorted
    }

    #[test]
    fn test_parse_prompt_file_no_frontmatter() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("plain-prompt.md");
        std::fs::write(&file_path, "Just a body\n").unwrap();

        let prompt = parse_prompt_file(&file_path).unwrap();
        assert_eq!(prompt.name, "plain-prompt");
        assert_eq!(prompt.description, "");
        assert!(prompt.args.is_none());
        assert_eq!(prompt.body, "Just a body");
        assert!(prompt.variables.is_empty());
    }

    #[test]
    fn test_library_list_and_get() {
        let mut lib = PromptLibrary::new();
        lib.prompts = {
            let mut m = HashMap::new();
            m.insert(
                "prompt1".to_string(),
                Prompt {
                    name: "prompt1".to_string(),
                    description: "First".to_string(),
                    body: "Body 1".to_string(),
                    variables: vec![],
                    args: None,
                },
            );
            m.insert(
                "prompt2".to_string(),
                Prompt {
                    name: "prompt2".to_string(),
                    description: "Second".to_string(),
                    body: "Body 2".to_string(),
                    variables: vec![],
                    args: None,
                },
            );
            m
        };

        assert_eq!(lib.list().len(), 2);
        assert!(lib.get("prompt1").is_some());
        assert!(lib.get("nonexistent").is_none());
    }

    #[test]
    fn test_extract_variables_from_body() {
        let vars = crate::prompt_substitute::extract_variables("Test {{project}} and {{file}}");
        assert_eq!(vars, vec!["file", "project"]);
    }
}
