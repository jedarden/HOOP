//! Project registry management
//!
//! Handles the `hoop projects` subcommands for managing the ~/.hoop/projects.yaml registry.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
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

/// Project registry structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectsRegistry {
    pub projects: Vec<ProjectEntry>,
}

/// A single project entry in the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectEntry {
    /// Project name (derived from directory name or operator-specified)
    pub name: String,
    /// Absolute path to the workspace
    pub path: PathBuf,
}

impl Default for ProjectsRegistry {
    fn default() -> Self {
        Self {
            projects: Vec::new(),
        }
    }
}

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

    /// Add a new project to the registry.
    ///
    /// If `name_override` is provided, it is used as the project name;
    /// otherwise the name is derived from the directory basename.
    pub fn add(&mut self, path: PathBuf, name_override: Option<&str>) -> Result<ProjectEntry> {
        let absolute_path = fs::canonicalize(&path)
            .with_context(|| format!("Path does not exist: {}", path.display()))?;

        let beads_path = absolute_path.join(".beads");
        if !beads_path.exists() || !beads_path.is_dir() {
            anyhow::bail!(
                "Path does not contain a .beads/ directory: {}",
                absolute_path.display()
            );
        }

        let name = name_override.map(|s| s.to_string()).unwrap_or_else(|| {
            absolute_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string()
        });

        if self.projects.iter().any(|p| p.name == name) {
            anyhow::bail!("Project '{}' already exists in registry", name);
        }

        if let Some(existing) = self.projects.iter().find(|p| p.path == absolute_path) {
            anyhow::bail!("Path already registered as project '{}'", existing.name);
        }

        let entry = ProjectEntry {
            name: name.clone(),
            path: absolute_path.clone(),
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

    /// Returns the set of canonical paths already registered
    fn registered_paths(&self) -> HashSet<PathBuf> {
        self.projects.iter().map(|p| p.path.clone()).collect()
    }
}

/// Validate that a path exists and contains a .beads directory
pub fn validate_workspace(path: &Path) -> Result<PathBuf> {
    let absolute_path = fs::canonicalize(path)
        .with_context(|| format!("Path does not exist: {}", path.display()))?;

    let beads_path = absolute_path.join(".beads");
    if !beads_path.exists() || !beads_path.is_dir() {
        anyhow::bail!(
            "Path does not contain a .beads/ directory: {}",
            absolute_path.display()
        );
    }

    Ok(absolute_path)
}

/// Add a project to the registry
pub fn add_project(path: &str) -> Result<ProjectEntry> {
    let path_buf = PathBuf::from(path);
    let absolute_path = validate_workspace(&path_buf)?;

    let mut registry = ProjectsRegistry::load()?;
    let entry = registry.add(absolute_path, None)?;
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
                    let effective = &entry.name;
                    if name_opt.is_some() {
                        println!("    Registered '{}' -> {}", effective, path.display());
                    } else {
                        println!("    Registered '{}' -> {}", effective, path.display());
                    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_load_empty() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let yaml_path = tmp.path().join("projects.yaml");
        // Empty file should parse as empty registry
        fs::write(&yaml_path, "projects: []\n").expect("write");
        // Can't easily test load() since it reads from ~/.hoop, but we can
        // test parse round-trips
        let registry: ProjectsRegistry =
            serde_yaml::from_str("projects: []\n").expect("parse empty");
        assert!(registry.projects.is_empty());
    }

    #[test]
    fn entry_round_trip() {
        let entry = ProjectEntry {
            name: "test-project".to_string(),
            path: PathBuf::from("/home/coding/test-project"),
        };
        let yaml = serde_yaml::to_string(&entry).expect("serialize");
        let parsed: ProjectEntry = serde_yaml::from_str(&yaml).expect("deserialize");
        assert_eq!(entry.name, parsed.name);
        assert_eq!(entry.path, parsed.path);
    }

    #[test]
    fn add_with_default_name() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("my-repo");
        fs::create_dir_all(repo.join(".beads")).expect("mkdir");

        let mut registry = ProjectsRegistry::default();
        let entry = registry.add(repo, None).expect("add");

        assert_eq!(entry.name, "my-repo");
        assert!(entry.path.is_absolute());
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
        registry.projects.push(ProjectEntry {
            name: "to-remove".to_string(),
            path: PathBuf::from("/tmp/to-remove"),
        });

        assert!(registry.remove("to-remove").expect("remove"));
        assert!(registry.projects.is_empty());
    }

    #[test]
    fn remove_nonexistent_project() {
        let mut registry = ProjectsRegistry::default();
        assert!(!registry.remove("nope").expect("remove"));
    }

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
            projects: vec![ProjectEntry {
                name: "already-here".to_string(),
                path: canonical,
            }],
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

    #[test]
    fn registered_paths_returns_all() {
        let registry = ProjectsRegistry {
            projects: vec![
                ProjectEntry {
                    name: "a".to_string(),
                    path: PathBuf::from("/tmp/a"),
                },
                ProjectEntry {
                    name: "b".to_string(),
                    path: PathBuf::from("/tmp/b"),
                },
            ],
        };

        let paths = registry.registered_paths();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&PathBuf::from("/tmp/a")));
        assert!(paths.contains(&PathBuf::from("/tmp/b")));
    }
}
