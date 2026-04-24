//! Project registry management
//!
//! Handles the `hoop projects` subcommands for managing the ~/.hoop/projects.yaml registry.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::borrow::Cow;
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Path to the projects registry file
fn registry_path() -> Result<PathBuf> {
    let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.push(".hoop");
    home.push("projects.yaml");
    Ok(home)
}

/// Ensure the ~/.hoop directory exists
fn ensure_hoop_dir() -> Result<PathBuf> {
    let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.push(".hoop");
    fs::create_dir_all(&home).context("Failed to create ~/.hoop directory")?;
    Ok(home)
}

/// Workspace role within a project.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkspaceRole {
    #[default]
    Primary,
    Manifests,
    Source,
    Secrets,
    Docs,
}

impl fmt::Display for WorkspaceRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primary => write!(f, "primary"),
            Self::Manifests => write!(f, "manifests"),
            Self::Source => write!(f, "source"),
            Self::Secrets => write!(f, "secrets"),
            Self::Docs => write!(f, "docs"),
        }
    }
}

/// A single workspace within a project.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceEntry {
    /// Raw workspace path as provided by the operator (display-only)
    pub path: PathBuf,
    /// Realpath-resolved absolute path used for joins and dedup.
    /// Stored on write; reconciled on read.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_path: Option<PathBuf>,
    /// Workspace role
    pub role: WorkspaceRole,
}

/// Project registry structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectsRegistry {
    pub projects: Vec<ProjectEntry>,
}

/// A single project entry in the registry.
///
/// Supports two on-disk formats:
/// - **v0.1 shorthand** (single workspace, role defaults to `primary`):
///   ```yaml
///   - name: my-project
///     path: /home/coding/my-project
///   ```
/// - **v0.2 multi-workspace**:
///   ```yaml
///   - name: my-project
///     workspaces:
///       - path: /home/coding/my-project
///         role: primary
///       - path: /home/coding/my-project-manifests
///         role: manifests
///   ```
///
/// Both forms round-trip correctly. The shorthand serializes back as shorthand
/// when there is exactly one workspace with role `primary` and no label/color.
#[derive(Debug, Clone)]
pub struct ProjectEntry {
    /// Project name (derived from directory name or operator-specified)
    pub name: String,
    /// Optional label
    pub label: Option<String>,
    /// Optional color hex code (e.g. `#8A2BE2`)
    pub color: Option<String>,
    /// Workspaces — always at least one entry
    pub workspaces: Vec<WorkspaceEntry>,
}

impl ProjectEntry {
    /// Returns the primary workspace path (first workspace with role `Primary`,
    /// or the first workspace if none have that role).
    pub fn primary_path(&self) -> Option<&Path> {
        self.workspaces
            .iter()
            .find(|w| w.role == WorkspaceRole::Primary)
            .or_else(|| self.workspaces.first())
            .map(|w| w.path.as_path())
    }

    /// Iterate over all workspace raw paths (for display).
    pub fn all_paths(&self) -> impl Iterator<Item = &Path> {
        self.workspaces.iter().map(|w| w.path.as_path())
    }

    /// Iterate over all workspace canonical paths (for joins/dedup).
    /// Falls back to raw path when canonical_path is absent (legacy data).
    pub fn all_canonical_paths(&self) -> impl Iterator<Item = Cow<'_, Path>> {
        self.workspaces.iter().map(|w| {
            match &w.canonical_path {
                Some(cp) => Cow::Borrowed(cp.as_path()),
                None => Cow::Borrowed(w.path.as_path()),
            }
        })
    }

    /// Validate workspace invariants. Returns a list of warnings (non-fatal).
    /// Returns `Err` if the workspaces array is empty (hard error).
    #[allow(dead_code)]
    pub fn validate_workspaces(&self) -> Result<Vec<String>> {
        if self.workspaces.is_empty() {
            anyhow::bail!(
                "Project '{}': workspaces array cannot be empty",
                self.name
            );
        }

        let mut warnings = Vec::new();
        let mut seen: std::collections::HashMap<&WorkspaceRole, usize> =
            std::collections::HashMap::new();
        for (i, ws) in self.workspaces.iter().enumerate() {
            if let Some(&prev) = seen.get(&ws.role) {
                warnings.push(format!(
                    "Project '{}': duplicate role '{}' at workspace indices {} and {}",
                    self.name, ws.role, prev, i
                ));
            } else {
                seen.insert(&ws.role, i);
            }
        }
        Ok(warnings)
    }
}

// ── Serde implementations ───────────────────────────────────────────────────

impl Serialize for ProjectEntry {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;

        // Shorthand: single primary workspace with no label/color
        if self.workspaces.len() == 1
            && self.workspaces[0].role == WorkspaceRole::Primary
            && self.label.is_none()
            && self.color.is_none()
        {
            let n = 2 + usize::from(self.workspaces[0].canonical_path.is_some());
            let mut map = serializer.serialize_map(Some(n))?;
            map.serialize_entry("name", &self.name)?;
            map.serialize_entry("path", &self.workspaces[0].path)?;
            if let Some(cp) = &self.workspaces[0].canonical_path {
                map.serialize_entry("canonical_path", cp)?;
            }
            return map.end();
        }

        // Multi-workspace form
        let n = 1 // name
            + usize::from(self.label.is_some())
            + usize::from(self.color.is_some())
            + 1; // workspaces
        let mut map = serializer.serialize_map(Some(n))?;
        map.serialize_entry("name", &self.name)?;
        if let Some(label) = &self.label {
            map.serialize_entry("label", label)?;
        }
        if let Some(color) = &self.color {
            map.serialize_entry("color", color)?;
        }
        map.serialize_entry("workspaces", &self.workspaces)?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for ProjectEntry {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Raw {
            name: String,
            #[serde(default)]
            label: Option<String>,
            #[serde(default)]
            color: Option<String>,
            /// v0.1 shorthand — single workspace, role defaults to primary
            #[serde(default)]
            path: Option<PathBuf>,
            /// v0.1 shorthand — canonical path (optional)
            #[serde(default)]
            canonical_path: Option<PathBuf>,
            /// v0.2 multi-workspace
            #[serde(default)]
            workspaces: Option<Vec<WorkspaceEntry>>,
        }

        let raw = Raw::deserialize(deserializer)?;

        let workspaces = match (raw.path, raw.workspaces) {
            (Some(path), None) => vec![WorkspaceEntry {
                path,
                canonical_path: raw.canonical_path,
                role: WorkspaceRole::Primary,
            }],
            (None, Some(ws)) => {
                if ws.is_empty() {
                    return Err(serde::de::Error::custom(
                        "workspaces array cannot be empty; omit the field or provide at least one entry",
                    ));
                }
                ws
            }
            (Some(_), Some(_)) => {
                return Err(serde::de::Error::custom(
                    "project entry cannot have both 'path' and 'workspaces' fields",
                ));
            }
            (None, None) => {
                return Err(serde::de::Error::custom(
                    "project entry must have either 'path' (shorthand) or 'workspaces' field",
                ));
            }
        };

        Ok(ProjectEntry {
            name: raw.name,
            label: raw.label,
            color: raw.color,
            workspaces,
        })
    }
}

// ── Registry operations ─────────────────────────────────────────────────────

impl ProjectsRegistry {
    /// Load the registry from disk, creating a default one if it doesn't exist
    pub fn load() -> Result<Self> {
        let path = registry_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&path).context("Failed to read projects.yaml")?;

        let registry: Self =
            serde_yaml::from_str(&contents).context("Failed to parse projects.yaml")?;

        Ok(registry)
    }

    /// Save the registry to disk
    pub fn save(&self) -> Result<()> {
        ensure_hoop_dir()?;
        let path = registry_path()?;

        let yaml = serde_yaml::to_string(self).context("Failed to serialize registry")?;

        fs::write(&path, yaml).context("Failed to write projects.yaml")?;

        Ok(())
    }

    /// Add a new single-workspace project to the registry.
    ///
    /// If `name_override` is provided it is used as the project name;
    /// otherwise the name is derived from the directory basename.
    /// Stores both the raw input path and the canonical (realpath) form.
    pub fn add(&mut self, path: PathBuf, name_override: Option<&str>) -> Result<ProjectEntry> {
        let canonical = fs::canonicalize(&path)
            .with_context(|| format!("Path does not exist: {}", path.display()))?;

        let beads_path = canonical.join(".beads");
        if !beads_path.exists() || !beads_path.is_dir() {
            anyhow::bail!(
                "Path does not contain a .beads/ directory: {}",
                canonical.display()
            );
        }

        let name = name_override.map(|s| s.to_string()).unwrap_or_else(|| {
            canonical
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string()
        });

        if self.projects.iter().any(|p| p.name == name) {
            anyhow::bail!("Project '{}' already exists in registry", name);
        }

        // Dedup by canonical path (catches symlink aliases)
        if let Some(existing) = self
            .projects
            .iter()
            .find(|p| p.all_canonical_paths().any(|cp| cp.as_ref() == canonical))
        {
            // Warn if raw paths differ — indicates symlink or alternate mount
            if !existing.all_paths().any(|wp| wp == path) {
                eprintln!(
                    "warning: path '{}' resolves to the same canonical location as project '{}' ({})",
                    path.display(),
                    existing.name,
                    canonical.display()
                );
            }
            anyhow::bail!("Path already registered as project '{}'", existing.name);
        }

        // raw = original input (or canonical if they're the same)
        let raw = if path == canonical { canonical.clone() } else { path };

        let entry = ProjectEntry {
            name,
            label: None,
            color: None,
            workspaces: vec![WorkspaceEntry {
                path: raw,
                canonical_path: Some(canonical),
                role: WorkspaceRole::Primary,
            }],
        };

        self.projects.push(entry.clone());
        Ok(entry)
    }

    /// Remove a project by name
    pub fn remove(&mut self, name: &str) -> Result<bool> {
        let original_len = self.projects.len();
        self.projects.retain(|p| p.name != name);
        Ok(self.projects.len() < original_len)
    }

    /// Get a project by name
    pub fn get(&self, name: &str) -> Option<&ProjectEntry> {
        self.projects.iter().find(|p| p.name == name)
    }

    /// Returns the set of all canonical workspace paths already registered (across all projects).
    /// Uses canonical_path when available, falls back to raw path for legacy entries.
    fn registered_paths(&self) -> HashSet<PathBuf> {
        self.projects
            .iter()
            .flat_map(|p| p.all_canonical_paths().map(|cp| cp.to_path_buf()))
            .collect()
    }
}

// ── Public helpers ──────────────────────────────────────────────────────────

/// Validate that a path exists and contains a .beads directory.
/// Returns the canonical (realpath) version.
pub fn validate_workspace(path: &Path) -> Result<PathBuf> {
    let canonical = fs::canonicalize(path)
        .with_context(|| format!("Path does not exist: {}", path.display()))?;

    let beads_path = canonical.join(".beads");
    if !beads_path.exists() || !beads_path.is_dir() {
        anyhow::bail!(
            "Path does not contain a .beads/ directory: {}",
            canonical.display()
        );
    }

    Ok(canonical)
}

/// Add a project to the registry.
///
/// Passes the raw input path to `add()` so the original (possibly symlink)
/// path is preserved for display while the canonical path is used for joins.
pub fn add_project(path: &str) -> Result<ProjectEntry> {
    let raw_path = PathBuf::from(path);

    let mut registry = ProjectsRegistry::load()?;
    let entry = registry.add(raw_path, None)?;
    registry.save()?;

    Ok(entry)
}

/// List all projects in the registry
pub fn list_projects() -> Result<Vec<ProjectEntry>> {
    let registry = ProjectsRegistry::load()?;
    Ok(registry.projects)
}

/// Remove a project from the registry
pub fn remove_project(name: &str) -> Result<bool> {
    let mut registry = ProjectsRegistry::load()?;
    let removed = registry.remove(name)?;
    registry.save()?;
    Ok(removed)
}

/// Show details for a single project
pub fn show_project(name: &str) -> Result<Option<ProjectEntry>> {
    let registry = ProjectsRegistry::load()?;
    Ok(registry.get(name).cloned())
}

/// Recursively discover directories containing .beads/ under a root path.
///
/// Walks two levels deep: immediate children of the root, and one level
/// of nesting within each child. This avoids scanning deep into structures
/// like node_modules or target/ while catching common nested-repo layouts.
pub fn discover_bead_workspaces(root: &Path) -> Result<Vec<PathBuf>> {
    let mut results = Vec::new();
    let mut seen = HashSet::new();

    let entries = fs::read_dir(root)
        .with_context(|| format!("Failed to read directory: {}", root.display()))?;

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        if path.join(".beads").is_dir() {
            if let Ok(canonical) = fs::canonicalize(&path) {
                if seen.insert(canonical.clone()) {
                    results.push(canonical);
                }
            }
        }

        // Check one level deeper for nested repos
        if let Ok(sub_entries) = fs::read_dir(&path) {
            for sub_entry in sub_entries.filter_map(Result::ok) {
                let sub_path = sub_entry.path();
                if sub_path.is_dir() && sub_path.join(".beads").is_dir() {
                    if let Ok(canonical) = fs::canonicalize(&sub_path) {
                        if seen.insert(canonical.clone()) {
                            results.push(canonical);
                        }
                    }
                }
            }
        }
    }

    results.sort();
    Ok(results)
}

/// Scan a root directory for workspaces containing .beads/ and register them.
///
/// In interactive mode (auto_yes=false), the user is prompted y/n per discovery
/// and can optionally rename the project from the default (directory basename).
/// With auto_yes=true, all discoveries are registered without prompting.
/// Already-registered paths are skipped with a note.
///
/// Multi-workspace projects require manual merging via a separate command.
pub fn scan_projects(root: &str, auto_yes: bool) -> Result<()> {
    let root_path = PathBuf::from(root);
    if !root_path.exists() {
        anyhow::bail!("Root path does not exist: {}", root_path.display());
    }
    if !root_path.is_dir() {
        anyhow::bail!("Root path is not a directory: {}", root_path.display());
    }

    let mut registry = ProjectsRegistry::load()?;
    let discovered = discover_bead_workspaces(&root_path)?;

    if discovered.is_empty() {
        println!(
            "No directories with .beads/ found under {}",
            root_path.display()
        );
        return Ok(());
    }

    let registered = registry.registered_paths();
    let mut new_count = 0usize;
    let mut skipped_count = 0usize;

    println!(
        "Found {} director{} with .beads/ under {}\n",
        discovered.len(),
        if discovered.len() == 1 { "y" } else { "ies" },
        root_path.display()
    );

    for path in &discovered {
        let default_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        if registered.contains(path) {
            println!("  {} — already registered, skipping", default_name);
            skipped_count += 1;
            continue;
        }

        if auto_yes {
            println!("  {} — registering", default_name);
            match registry.add(path.clone(), None) {
                Ok(entry) => {
                    println!("    Registered '{}' -> {}", entry.name, path.display());
                    new_count += 1;
                }
                Err(e) => {
                    eprintln!("    Failed to register {}: {}", path.display(), e);
                }
            }
        } else {
            print!("  {} — register? [y/N] ", default_name);
            std::io::stdout().flush()?;

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let answer = input.trim().to_lowercase();

            if answer != "y" && answer != "yes" {
                continue;
            }

            // Offer rename
            print!("    name [{}]: ", default_name);
            std::io::stdout().flush()?;

            let mut name_input = String::new();
            std::io::stdin().read_line(&mut name_input)?;
            let custom_name = name_input.trim();

            let name_opt = if custom_name.is_empty() || custom_name == default_name {
                None
            } else {
                Some(custom_name)
            };

            match registry.add(path.clone(), name_opt) {
                Ok(entry) => {
                    println!("    Registered '{}' -> {}", entry.name, path.display());
                    new_count += 1;
                }
                Err(e) => {
                    eprintln!("    Failed to register {}: {}", path.display(), e);
                }
            }
        }
    }

    if new_count > 0 {
        registry.save()?;
    }

    println!();
    if new_count > 0 {
        println!(
            "Registered {} new project{}",
            new_count,
            if new_count == 1 { "" } else { "s" }
        );
    } else {
        println!("No new projects to register");
    }
    if skipped_count > 0 {
        println!(
            "Skipped {} already-registered path{}",
            skipped_count,
            if skipped_count == 1 { "" } else { "s" }
        );
    }

    Ok(())
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(name: &str, path: PathBuf) -> ProjectEntry {
        ProjectEntry {
            name: name.to_string(),
            label: None,
            color: None,
            workspaces: vec![WorkspaceEntry {
                path: path.clone(),
                canonical_path: Some(path),
                role: WorkspaceRole::Primary,
            }],
        }
    }

    fn make_entry_multi(name: &str, workspaces: Vec<WorkspaceEntry>) -> ProjectEntry {
        ProjectEntry {
            name: name.to_string(),
            label: None,
            color: None,
            workspaces,
        }
    }

    #[test]
    fn registry_load_empty() {
        let registry: ProjectsRegistry =
            serde_yaml::from_str("projects: []\n").expect("parse empty");
        assert!(registry.projects.is_empty());
    }

    // ── Shorthand round-trip ─────────────────────────────────────────────

    #[test]
    fn shorthand_round_trip() {
        let yaml = "name: test-project\npath: /home/coding/test-project\n";
        let parsed: ProjectEntry = serde_yaml::from_str(yaml).expect("deserialize shorthand");
        assert_eq!(parsed.name, "test-project");
        assert_eq!(parsed.workspaces.len(), 1);
        assert_eq!(parsed.workspaces[0].role, WorkspaceRole::Primary);
        assert_eq!(
            parsed.workspaces[0].path,
            PathBuf::from("/home/coding/test-project")
        );

        let reserialized = serde_yaml::to_string(&parsed).expect("serialize");
        assert!(reserialized.contains("path:"), "should re-serialize as shorthand");
        assert!(!reserialized.contains("workspaces:"), "should not emit workspaces key");
    }

    #[test]
    fn multi_workspace_round_trip() {
        let yaml = r#"
name: my-project
workspaces:
  - path: /home/coding/repo
    role: primary
  - path: /home/coding/manifests
    role: manifests
"#;
        let parsed: ProjectEntry = serde_yaml::from_str(yaml).expect("deserialize multi");
        assert_eq!(parsed.name, "my-project");
        assert_eq!(parsed.workspaces.len(), 2);
        assert_eq!(parsed.workspaces[0].role, WorkspaceRole::Primary);
        assert_eq!(parsed.workspaces[1].role, WorkspaceRole::Manifests);

        let reserialized = serde_yaml::to_string(&parsed).expect("serialize");
        assert!(reserialized.contains("workspaces:"), "should emit workspaces key");
        assert!(!reserialized.contains("\npath:"), "should not use shorthand form");
    }

    #[test]
    fn label_color_forces_multi_form() {
        let entry = ProjectEntry {
            name: "colored".to_string(),
            label: Some("My label".to_string()),
            color: Some("#FF0000".to_string()),
            workspaces: vec![WorkspaceEntry {
                path: PathBuf::from("/tmp/colored"),
                canonical_path: None,
                role: WorkspaceRole::Primary,
            }],
        };
        let yaml = serde_yaml::to_string(&entry).expect("serialize");
        assert!(yaml.contains("workspaces:"), "label/color forces multi-workspace form");
        assert!(yaml.contains("label:"));
        assert!(yaml.contains("color:"));

        let parsed: ProjectEntry = serde_yaml::from_str(&yaml).expect("deserialize");
        assert_eq!(parsed.label, Some("My label".to_string()));
        assert_eq!(parsed.color, Some("#FF0000".to_string()));
    }

    // ── Validation ───────────────────────────────────────────────────────

    #[test]
    fn empty_workspaces_rejected_at_deserialize() {
        let yaml = "name: bad\nworkspaces: []\n";
        let result: Result<ProjectEntry, _> = serde_yaml::from_str(yaml);
        assert!(result.is_err(), "empty workspaces array must be rejected");
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("empty"), "error should mention 'empty': {}", msg);
    }

    #[test]
    fn both_path_and_workspaces_rejected() {
        let yaml = r#"
name: bad
path: /tmp/bad
workspaces:
  - path: /tmp/bad
    role: primary
"#;
        let result: Result<ProjectEntry, _> = serde_yaml::from_str(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn neither_path_nor_workspaces_rejected() {
        let yaml = "name: bad\n";
        let result: Result<ProjectEntry, _> = serde_yaml::from_str(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn validate_workspaces_duplicate_roles_warns() {
        let entry = make_entry_multi(
            "dup-roles",
            vec![
                WorkspaceEntry {
                    path: PathBuf::from("/tmp/a"),
                    canonical_path: None,
                    role: WorkspaceRole::Primary,
                },
                WorkspaceEntry {
                    path: PathBuf::from("/tmp/b"),
                    canonical_path: None,
                    role: WorkspaceRole::Primary,
                },
            ],
        );
        let warnings = entry.validate_workspaces().expect("should not error");
        assert_eq!(warnings.len(), 1, "one duplicate role warning expected");
        assert!(warnings[0].contains("duplicate role"));
    }

    #[test]
    fn validate_workspaces_no_duplicates_ok() {
        let entry = make_entry_multi(
            "ok",
            vec![
                WorkspaceEntry {
                    path: PathBuf::from("/tmp/a"),
                    canonical_path: None,
                    role: WorkspaceRole::Primary,
                },
                WorkspaceEntry {
                    path: PathBuf::from("/tmp/b"),
                    canonical_path: None,
                    role: WorkspaceRole::Manifests,
                },
            ],
        );
        let warnings = entry.validate_workspaces().expect("should not error");
        assert!(warnings.is_empty());
    }

    #[test]
    fn validate_workspaces_empty_hard_error() {
        let entry = ProjectEntry {
            name: "empty".to_string(),
            label: None,
            color: None,
            workspaces: vec![],
        };
        assert!(entry.validate_workspaces().is_err());
    }

    // ── Registry operations ──────────────────────────────────────────────

    #[test]
    fn add_with_default_name() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("my-repo");
        fs::create_dir_all(repo.join(".beads")).expect("mkdir");

        let mut registry = ProjectsRegistry::default();
        let entry = registry.add(repo, None).expect("add");

        assert_eq!(entry.name, "my-repo");
        assert_eq!(entry.workspaces.len(), 1);
        assert_eq!(entry.workspaces[0].role, WorkspaceRole::Primary);
        assert!(entry.workspaces[0].path.is_absolute());
        assert_eq!(registry.projects.len(), 1);
    }

    #[test]
    fn add_with_custom_name() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("my-repo");
        fs::create_dir_all(repo.join(".beads")).expect("mkdir");

        let mut registry = ProjectsRegistry::default();
        let entry = registry.add(repo, Some("custom-name")).expect("add");

        assert_eq!(entry.name, "custom-name");
    }

    #[test]
    fn add_rejects_duplicate_name() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo_a = tmp.path().join("repo-a");
        let repo_b = tmp.path().join("repo-b");
        fs::create_dir_all(repo_a.join(".beads")).expect("mkdir");
        fs::create_dir_all(repo_b.join(".beads")).expect("mkdir");

        let mut registry = ProjectsRegistry::default();
        registry.add(repo_a, Some("same-name")).expect("add first");
        let result = registry.add(repo_b, Some("same-name"));

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn add_rejects_duplicate_path() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        fs::create_dir_all(repo.join(".beads")).expect("mkdir");

        let mut registry = ProjectsRegistry::default();
        registry.add(repo.clone(), None).expect("add first");
        let result = registry.add(repo, Some("other-name"));

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("already registered"));
    }

    #[test]
    fn add_rejects_missing_beads() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        fs::create_dir_all(&repo).expect("mkdir"); // no .beads/

        let mut registry = ProjectsRegistry::default();
        let result = registry.add(repo, None);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(".beads/"));
    }

    #[test]
    fn remove_existing_project() {
        let mut registry = ProjectsRegistry::default();
        registry.projects.push(make_entry("to-remove", PathBuf::from("/tmp/to-remove")));

        assert!(registry.remove("to-remove").expect("remove"));
        assert!(registry.projects.is_empty());
    }

    #[test]
    fn remove_nonexistent_project() {
        let mut registry = ProjectsRegistry::default();
        assert!(!registry.remove("nope").expect("remove"));
    }

    // ── registered_paths covers multi-workspace ──────────────────────────

    #[test]
    fn registered_paths_covers_all_workspaces() {
        let registry = ProjectsRegistry {
            projects: vec![
                make_entry("a", PathBuf::from("/tmp/a")),
                make_entry_multi(
                    "b",
                    vec![
                        WorkspaceEntry {
                            path: PathBuf::from("/tmp/b1"),
                            canonical_path: Some(PathBuf::from("/tmp/b1")),
                            role: WorkspaceRole::Primary,
                        },
                        WorkspaceEntry {
                            path: PathBuf::from("/tmp/b2"),
                            canonical_path: Some(PathBuf::from("/tmp/b2")),
                            role: WorkspaceRole::Manifests,
                        },
                    ],
                ),
            ],
        };

        let paths = registry.registered_paths();
        assert_eq!(paths.len(), 3);
        assert!(paths.contains(&PathBuf::from("/tmp/a")));
        assert!(paths.contains(&PathBuf::from("/tmp/b1")));
        assert!(paths.contains(&PathBuf::from("/tmp/b2")));
    }

    // ── Discovery ────────────────────────────────────────────────────────

    #[test]
    fn discover_finds_beads_workspaces() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo_a = tmp.path().join("repo-a");
        fs::create_dir_all(repo_a.join(".beads")).expect("mkdir");
        let repo_b = tmp.path().join("repo-b");
        fs::create_dir_all(&repo_b).expect("mkdir"); // no .beads/
        let repo_c = tmp.path().join("repo-c");
        fs::create_dir_all(repo_c.join(".beads")).expect("mkdir");

        let found = discover_bead_workspaces(tmp.path()).expect("scan");
        let names: Vec<&str> = found
            .iter()
            .filter_map(|p| p.file_name().and_then(|n| n.to_str()))
            .collect();

        assert_eq!(names.len(), 2);
        assert!(names.contains(&"repo-a"));
        assert!(names.contains(&"repo-c"));
    }

    #[test]
    fn discover_finds_nested_workspaces() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let parent = tmp.path().join("parent");
        let child = parent.join("child-repo");
        fs::create_dir_all(child.join(".beads")).expect("mkdir");

        let found = discover_bead_workspaces(tmp.path()).expect("scan");
        let names: Vec<&str> = found
            .iter()
            .filter_map(|p| p.file_name().and_then(|n| n.to_str()))
            .collect();

        assert!(names.contains(&"child-repo"));
    }

    #[test]
    fn discover_returns_empty_for_no_beads() {
        let tmp = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(tmp.path().join("repo-x")).expect("mkdir");
        fs::create_dir_all(tmp.path().join("repo-y")).expect("mkdir");

        let found = discover_bead_workspaces(tmp.path()).expect("scan");
        assert!(found.is_empty());
    }

    #[test]
    fn discover_deduplicates_canonical_paths() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("my-repo");
        fs::create_dir_all(repo.join(".beads")).expect("mkdir");

        let found = discover_bead_workspaces(tmp.path()).expect("scan");
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn scan_skips_already_registered() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("already-here");
        fs::create_dir_all(repo.join(".beads")).expect("mkdir");
        let canonical = fs::canonicalize(&repo).expect("canonicalize");

        let registry = ProjectsRegistry {
            projects: vec![make_entry("already-here", canonical)],
        };

        let registered = registry.registered_paths();
        let found = discover_bead_workspaces(tmp.path()).expect("scan");
        let new: Vec<_> = found.iter().filter(|p| !registered.contains(*p)).collect();

        assert!(new.is_empty(), "already-registered path should be filtered");
    }

    #[test]
    fn scan_auto_yes_registers_all() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo_a = tmp.path().join("repo-a");
        let repo_b = tmp.path().join("repo-b");
        fs::create_dir_all(repo_a.join(".beads")).expect("mkdir");
        fs::create_dir_all(repo_b.join(".beads")).expect("mkdir");

        let mut registry = ProjectsRegistry::default();
        let found = discover_bead_workspaces(tmp.path()).expect("scan");

        for path in &found {
            registry.add(path.clone(), None).expect("add");
        }

        assert_eq!(registry.projects.len(), 2);
    }

    // ── primary_path ─────────────────────────────────────────────────────

    #[test]
    fn primary_path_returns_primary_workspace() {
        let entry = make_entry_multi(
            "p",
            vec![
                WorkspaceEntry {
                    path: PathBuf::from("/tmp/manifests"),
                    canonical_path: None,
                    role: WorkspaceRole::Manifests,
                },
                WorkspaceEntry {
                    path: PathBuf::from("/tmp/primary"),
                    canonical_path: None,
                    role: WorkspaceRole::Primary,
                },
            ],
        );
        assert_eq!(entry.primary_path(), Some(Path::new("/tmp/primary")));
    }

    #[test]
    fn primary_path_falls_back_to_first() {
        let entry = make_entry_multi(
            "p",
            vec![WorkspaceEntry {
                path: PathBuf::from("/tmp/manifests"),
                canonical_path: None,
                role: WorkspaceRole::Manifests,
            }],
        );
        assert_eq!(entry.primary_path(), Some(Path::new("/tmp/manifests")));
    }

    // ── Canonicalization symlink fixtures ──────────────────────────────────

    #[test]
    fn add_via_symlink_preserves_raw_and_canonical() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let real = tmp.path().join("real-repo");
        fs::create_dir_all(real.join(".beads")).expect("mkdir");

        let link = tmp.path().join("link-repo");
        std::os::unix::fs::symlink(&real, &link).expect("symlink");

        let canonical = fs::canonicalize(&real).expect("canonicalize");
        assert_ne!(link, canonical, "symlink and canonical should differ");

        let mut registry = ProjectsRegistry::default();
        let entry = registry.add(link.clone(), None).expect("add via symlink");

        // raw = symlink path, canonical = real resolved path
        assert_eq!(entry.workspaces[0].path, link, "raw path should be the symlink");
        assert_eq!(
            entry.workspaces[0].canonical_path.as_ref(),
            Some(&canonical),
            "canonical_path should resolve to the real directory"
        );
    }

    #[test]
    fn add_via_symlink_rejects_duplicate_canonical() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let real = tmp.path().join("real-repo");
        fs::create_dir_all(real.join(".beads")).expect("mkdir");

        let link = tmp.path().join("link-repo");
        std::os::unix::fs::symlink(&real, &link).expect("symlink");

        let mut registry = ProjectsRegistry::default();
        registry.add(real.clone(), Some("via-real")).expect("add via real");

        let result = registry.add(link, Some("via-link"));
        assert!(result.is_err(), "should reject duplicate canonical path");
        assert!(
            result.unwrap_err().to_string().contains("already registered"),
            "error should reference the existing project"
        );
    }

    #[test]
    fn shorthand_round_trip_preserves_canonical_path() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let real = tmp.path().join("real-repo");
        fs::create_dir_all(real.join(".beads")).expect("mkdir");
        let canonical = fs::canonicalize(&real).expect("canonicalize");

        let link = tmp.path().join("link-repo");
        std::os::unix::fs::symlink(&real, &link).expect("symlink");

        // Simulate what add() produces: raw=symlink, canonical=real
        let entry = ProjectEntry {
            name: "symlink-proj".to_string(),
            label: None,
            color: None,
            workspaces: vec![WorkspaceEntry {
                path: link.clone(),
                canonical_path: Some(canonical.clone()),
                role: WorkspaceRole::Primary,
            }],
        };

        // Serialize as shorthand
        let yaml = serde_yaml::to_string(&entry).expect("serialize");
        assert!(yaml.contains("path:"), "should use shorthand form");
        assert!(yaml.contains("canonical_path:"), "should include canonical_path");
        assert!(!yaml.contains("workspaces:"), "should not use multi-workspace form");

        // Deserialize back
        let parsed: ProjectEntry = serde_yaml::from_str(&yaml).expect("deserialize");
        assert_eq!(parsed.workspaces[0].path, link, "raw path preserved");
        assert_eq!(
            parsed.workspaces[0].canonical_path.as_ref(),
            Some(&canonical),
            "canonical path preserved"
        );
    }

    #[test]
    fn shorthand_without_canonical_round_trips_cleanly() {
        let yaml = "name: test-project\npath: /home/coding/test-project\n";
        let parsed: ProjectEntry = serde_yaml::from_str(yaml).expect("deserialize");
        assert!(parsed.workspaces[0].canonical_path.is_none());

        let reserialized = serde_yaml::to_string(&parsed).expect("serialize");
        assert!(!reserialized.contains("canonical_path:"),
            "should not emit canonical_path when None");
    }

    #[test]
    fn add_direct_path_stores_both_equal() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("my-repo");
        fs::create_dir_all(repo.join(".beads")).expect("mkdir");
        let canonical = fs::canonicalize(&repo).expect("canonicalize");

        // When raw == canonical (no symlink), both should still be stored
        let mut registry = ProjectsRegistry::default();
        let entry = registry.add(canonical.clone(), None).expect("add");

        assert_eq!(entry.workspaces[0].path, canonical);
        assert_eq!(
            entry.workspaces[0].canonical_path.as_ref(),
            Some(&canonical)
        );
    }

    #[test]
    fn discover_symlink_dedup_across_dirs() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let real = tmp.path().join("real-repo");
        fs::create_dir_all(real.join(".beads")).expect("mkdir");

        // Create a subdirectory with a symlink to the same repo
        let sub = tmp.path().join("subdir");
        fs::create_dir_all(&sub).expect("mkdir");
        let link = sub.join("alias-repo");
        std::os::unix::fs::symlink(&real, &link).expect("symlink");

        let found = discover_bead_workspaces(tmp.path()).expect("scan");
        // Should find the real repo and the symlink alias — but deduplicated
        // by canonical path, so only one entry
        let canonical = fs::canonicalize(&real).expect("canonicalize");
        let canonical_count = found.iter().filter(|p| **p == canonical).count();
        assert_eq!(canonical_count, 1, "dedup by canonical should collapse symlink alias");
    }
}
