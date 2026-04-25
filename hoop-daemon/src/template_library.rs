//! Stitch template library — discovers markdown templates from disk
//!
//! Two scopes:
//!   Global:  `~/.hoop/templates/*.md`
//!   Project: `<project>/.hoop/templates/*.md`
//!
//! Each template is a markdown file with YAML frontmatter:
//!   ---
//!   name: review-PR
//!   description: "Review a pull request"
//!   kind: review
//!   priority: 1
//!   labels: [review, pr]
//!   fields:
//!     - key: pr_number
//!       label: "PR #"
//!       required: true
//!     - key: repo
//!       label: Repository
//!       required: false
//!   ---
//!   ## Task
//!   Review PR #{{pr_number}} in {{repo}}.
//!
//! Templates are loaded at startup and file-watched for hot-reload.

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

/// A parsed template ready for API responses.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StitchTemplate {
    /// Unique name (filename without .md, or explicit `name` in frontmatter)
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Scope: "global" or "project"
    pub scope: String,
    /// Default kind for the draft form
    pub kind: Option<String>,
    /// Default priority
    pub priority: Option<i64>,
    /// Default labels
    pub labels: Vec<String>,
    /// Default dependencies (bead IDs)
    pub default_beads: Vec<String>,
    /// Per-template required fields that surface as form inputs
    pub fields: Vec<TemplateField>,
    /// Body template with {{field}} placeholders
    pub body: String,
}

/// A field definition in a template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateField {
    /// Machine key used in {{key}} placeholders
    pub key: String,
    /// Human label for the form input
    pub label: String,
    /// Whether the field is required
    #[serde(default)]
    pub required: bool,
    /// Placeholder text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    /// Default value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

/// Raw frontmatter parsed from YAML.
#[derive(Debug, Deserialize, Default)]
struct TemplateFrontmatter {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    priority: Option<i64>,
    #[serde(default)]
    labels: Vec<String>,
    #[serde(default)]
    default_beads: Vec<String>,
    #[serde(default)]
    fields: Vec<TemplateField>,
}

/// Shared template store behind an RwLock.
pub type TemplateStore = Arc<std::sync::RwLock<TemplateLibrary>>;

/// In-memory collection of loaded templates.
#[derive(Debug, Clone, Default)]
pub struct TemplateLibrary {
    templates: Vec<StitchTemplate>,
}

impl TemplateLibrary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load all templates from global and (optionally) project template directories.
    pub fn load(&mut self, global_dir: &StdPath, project_dirs: &[PathBuf]) -> anyhow::Result<()> {
        let mut templates = Vec::new();

        if global_dir.exists() {
            load_from_dir(global_dir, "global", &mut templates)?;
        }

        for pd in project_dirs {
            let tmpl_dir = pd.join(".hoop").join("templates");
            if tmpl_dir.exists() {
                let project_name = pd
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                load_from_dir(&tmpl_dir, project_name, &mut templates)?;
            }
        }

        // Deduplicate by name (project overrides global with same name)
        let mut seen: HashMap<String, usize> = HashMap::new();
        for (i, t) in templates.iter().enumerate() {
            if let Some(&prev) = seen.get(&t.name) {
                // Project-scoped overrides global
                if templates[prev].scope == "global" && t.scope != "global" {
                    seen.insert(t.name.clone(), i);
                }
            } else {
                seen.insert(t.name.clone(), i);
            }
        }
        let deduped: Vec<StitchTemplate> = seen
            .into_values()
            .map(|i| templates[i].clone())
            .collect();

        info!("Template library loaded {} template(s)", deduped.len());
        self.templates = deduped;
        Ok(())
    }

    /// Return all templates, optionally filtered by scope.
    pub fn list(&self, scope: Option<&str>) -> Vec<StitchTemplate> {
        match scope {
            Some(s) => self.templates.iter().filter(|t| t.scope == s).cloned().collect(),
            None => self.templates.clone(),
        }
    }

    /// Get a single template by name.
    pub fn get(&self, name: &str) -> Option<StitchTemplate> {
        self.templates.iter().find(|t| t.name == name).cloned()
    }
}

/// Load all `.md` files from a directory, parsing frontmatter.
fn load_from_dir(dir: &StdPath, scope: &str, out: &mut Vec<StitchTemplate>) -> anyhow::Result<()> {
    let entries = std::fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        match parse_template_file(&path, scope) {
            Ok(tmpl) => out.push(tmpl),
            Err(e) => {
                warn!("Failed to parse template {}: {}", path.display(), e);
            }
        }
    }
    Ok(())
}

/// Parse a single template markdown file with YAML frontmatter.
fn parse_template_file(path: &StdPath, scope: &str) -> anyhow::Result<StitchTemplate> {
    let content = std::fs::read_to_string(path)?;
    let (frontmatter, body) = split_frontmatter(&content);

    let fm: TemplateFrontmatter = match frontmatter {
        Some(yaml) => serde_yaml::from_str(yaml)?,
        None => TemplateFrontmatter::default(),
    };

    let name = fm.name.unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
    });

    Ok(StitchTemplate {
        name,
        description: fm.description.unwrap_or_default(),
        scope: scope.to_string(),
        kind: fm.kind,
        priority: fm.priority,
        labels: fm.labels,
        default_beads: fm.default_beads,
        fields: fm.fields,
        body: body.trim().to_string(),
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

/// Start file watcher for the global templates directory.
/// Returns the watcher (must be kept alive for watching to work).
pub fn start_watcher(
    global_dir: PathBuf,
    store: TemplateStore,
    project_dirs: Vec<PathBuf>,
) -> RecommendedWatcher {
    let watcher_global_dir = global_dir.clone();
    let watcher_project_dirs = project_dirs.clone();
    let mut watcher =
        notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
            match res {
                Ok(_event) => {
                    debug!("Template directory changed, reloading");
                    let mut lib = store.write().unwrap();
                    if let Err(e) = lib.load(&watcher_global_dir, &watcher_project_dirs) {
                        warn!("Template reload failed: {}", e);
                    }
                }
                Err(e) => {
                    warn!("Template watch error: {}", e);
                }
            }
        })
        .expect("failed to create template file watcher");

    if global_dir.exists() {
        watcher
            .watch(&global_dir, RecursiveMode::Recursive)
            .unwrap_or_else(|e| warn!("Cannot watch global templates dir: {}", e));
    }

    // Watch project template dirs
    for pd in &project_dirs {
        let tmpl_dir = pd.join(".hoop").join("templates");
        if tmpl_dir.exists() {
            watcher
                .watch(&tmpl_dir, RecursiveMode::Recursive)
                .unwrap_or_else(|e| {
                    warn!("Cannot watch project templates dir {}: {}", tmpl_dir.display(), e)
                });
        }
    }

    info!("Template file watcher started");
    watcher
}

/// Ensure the global templates directory exists and seed example templates if empty.
pub fn ensure_global_templates_dir(global_dir: &StdPath) -> PathBuf {
    let dir = global_dir.join("templates");
    if !dir.exists() {
        std::fs::create_dir_all(&dir).unwrap_or_else(|e| {
            warn!("Failed to create global templates directory: {}", e);
        });
    }

    // Seed example templates if directory is empty
    let has_files = std::fs::read_dir(&dir)
        .map(|mut d| d.next().is_some())
        .unwrap_or(false);

    if !has_files {
        seed_example_templates(&dir);
    }

    dir
}

fn seed_example_templates(dir: &StdPath) {
    let examples = [
        (
            "review-PR.md",
            r#"---
name: review-PR
description: "Review a pull request"
kind: review
priority: 1
labels: [review, pr]
fields:
  - key: pr_number
    label: "PR #"
    required: true
    placeholder: "e.g. 1234"
  - key: repo
    label: Repository
    required: false
    placeholder: "e.g. jedarden/HOOP"
  - key: focus
    label: Focus areas
    required: false
    placeholder: "e.g. security, performance"
---
## Task
Review PR #{{pr_number}} in {{repo}}.

## Focus
{{focus}}

## Acceptance
- All CI checks pass
- No security regressions
- Code follows project conventions
"#,
        ),
        (
            "fix-from-incident.md",
            r#"---
name: fix-from-incident
description: "Fix a bug identified from an incident report"
kind: fix
priority: 0
labels: [fix, incident]
fields:
  - key: incident_id
    label: Incident ID
    required: true
    placeholder: "e.g. INC-2024-001"
  - key: summary
    label: Incident summary
    required: true
    placeholder: "Brief description of what happened"
  - key: root_cause
    label: Root cause
    required: false
    placeholder: "What caused the incident"
---
## Incident
{{incident_id}}: {{summary}}

## Root Cause
{{root_cause}}

## Fix Plan
- Identify affected code paths
- Write regression test(s) reproducing the failure
- Implement the fix
- Verify CI passes
"#,
        ),
        (
            "investigate-failure.md",
            r#"---
name: investigate-failure
description: "Investigate a production failure or anomaly"
kind: investigation
priority: 0
labels: [investigation, incident]
fields:
  - key: symptom
    label: Symptom
    required: true
    placeholder: "What was observed"
  - key: service
    label: Service/component
    required: false
    placeholder: "e.g. api-gateway, worker-pool"
  - key: started_at
    label: Started at
    required: false
    placeholder: "Approximate time the issue started"
  - key: impact
    label: Impact
    required: false
    placeholder: "User-visible impact, affected customers"
---
## Symptom
{{symptom}}

## Scope
- Service: {{service}}
- Started: {{started_at}}
- Impact: {{impact}}

## Investigation Plan
1. Check logs for errors around {{started_at}}
2. Check metrics for anomalies
3. Identify recent deployments
4. Narrow to root cause
5. Document findings and recommend fix
"#,
        ),
    ];

    for (filename, content) in &examples {
        let path = dir.join(filename);
        if let Err(e) = std::fs::write(&path, content) {
            warn!("Failed to seed example template {}: {}", filename, e);
        }
    }
    info!("Seeded {} example templates", examples.len());
}

// ---------------------------------------------------------------------------
// REST API
// ---------------------------------------------------------------------------

/// GET /api/templates — list all global templates
async fn list_global_templates(
    State(state): State<crate::DaemonState>,
) -> Json<Vec<StitchTemplate>> {
    let lib = state.template_library.read().unwrap();
    Json(lib.list(None))
}

/// GET /api/p/:project/templates — list templates (global + project-scoped)
async fn list_project_templates(
    Path(project): Path<String>,
    State(state): State<crate::DaemonState>,
) -> Result<Json<Vec<StitchTemplate>>, (StatusCode, String)> {
    crate::id_validators::validate_project_name(&project).map_err(crate::id_validators::rejection)?;
    let lib = state.template_library.read().unwrap();
    // Return all templates — project templates are already merged during load
    Ok(Json(lib.list(None)))
}

/// GET /api/templates/:name — get a single template by name
async fn get_template(
    Path(name): Path<String>,
    State(state): State<crate::DaemonState>,
) -> Result<Json<StitchTemplate>, (StatusCode, String)> {
    let lib = state.template_library.read().unwrap();
    lib.get(&name)
        .map(Json)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Template '{}' not found", name)))
}

pub fn router() -> Router<crate::DaemonState> {
    Router::new()
        .route("/api/templates", get(list_global_templates))
        .route("/api/templates/{name}", get(get_template))
        .route("/api/p/{project}/templates", get(list_project_templates))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_parse_template_file_with_frontmatter() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("my-template.md");
        std::fs::write(
            &file_path,
            r#"---
name: test-template
description: A test template
kind: fix
priority: 2
labels:
  - test
  - example
fields:
  - key: foo
    label: Foo Value
    required: true
  - key: bar
    label: Bar Value
    required: false
    default: baz
---
## Body
Fix {{foo}} with {{bar}}
"#,
        )
        .unwrap();

        let tmpl = parse_template_file(&file_path, "global").unwrap();
        assert_eq!(tmpl.name, "test-template");
        assert_eq!(tmpl.description, "A test template");
        assert_eq!(tmpl.kind, Some("fix".to_string()));
        assert_eq!(tmpl.priority, Some(2));
        assert_eq!(tmpl.labels, vec!["test", "example"]);
        assert_eq!(tmpl.fields.len(), 2);
        assert_eq!(tmpl.fields[0].key, "foo");
        assert!(tmpl.fields[0].required);
        assert_eq!(tmpl.fields[1].default, Some("baz".to_string()));
        assert!(tmpl.body.contains("Fix {{foo}}"));
        assert_eq!(tmpl.scope, "global");
    }

    #[test]
    fn test_parse_template_file_no_frontmatter() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("plain-template.md");
        std::fs::write(&file_path, "Just a body\n").unwrap();

        let tmpl = parse_template_file(&file_path, "project").unwrap();
        assert_eq!(tmpl.name, "plain-template");
        assert_eq!(tmpl.description, "");
        assert!(tmpl.kind.is_none());
        assert!(tmpl.fields.is_empty());
        assert_eq!(tmpl.body, "Just a body");
    }

    #[test]
    fn test_load_from_dir() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("a.md"),
            "---\nname: alpha\ndescription: first\n---\nAlpha body",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("b.md"),
            "---\nname: beta\ndescription: second\nkind: review\n---\nBeta body",
        )
        .unwrap();
        // Non-md file should be skipped
        std::fs::write(dir.path().join("ignore.txt"), "not a template").unwrap();

        let mut templates = vec![];
        load_from_dir(dir.path(), "global", &mut templates).unwrap();
        assert_eq!(templates.len(), 2);

        let names: Vec<&str> = templates.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"alpha"));
        assert!(names.contains(&"beta"));
    }

    #[test]
    fn test_library_list_and_get() {
        let mut lib = TemplateLibrary::new();
        lib.templates = vec![
            StitchTemplate {
                name: "global-tmpl".to_string(),
                scope: "global".to_string(),
                ..Default::default()
            },
            StitchTemplate {
                name: "proj-tmpl".to_string(),
                scope: "myproject".to_string(),
                ..Default::default()
            },
        ];

        assert_eq!(lib.list(None).len(), 2);
        assert_eq!(lib.list(Some("global")).len(), 1);
        assert!(lib.get("global-tmpl").is_some());
        assert!(lib.get("nonexistent").is_none());
    }

    #[test]
    fn test_dedup_project_overrides_global() {
        let dir_global = tempfile::tempdir().unwrap();
        let dir_project = tempfile::tempdir().unwrap();

        std::fs::write(
            dir_global.path().join("review-PR.md"),
            "---\nname: review-PR\ndescription: global version\n---\nGlobal body",
        )
        .unwrap();

        std::fs::create_dir_all(dir_project.path().join(".hoop").join("templates")).unwrap();
        std::fs::write(
            dir_project.path().join(".hoop").join("templates").join("review-PR.md"),
            "---\nname: review-PR\ndescription: project version\n---\nProject body",
        )
        .unwrap();

        let mut lib = TemplateLibrary::new();
        lib.load(dir_global.path(), &[dir_project.path().to_path_buf()]).unwrap();


        // Should have exactly one review-PR, the project-scoped one
        let tmpl = lib.get("review-PR").unwrap();
        assert_eq!(tmpl.description, "project version");
        assert_eq!(lib.list(None).len(), 1);
    }
}
