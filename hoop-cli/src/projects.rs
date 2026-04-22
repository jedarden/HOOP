//! Project registry management
//!
//! Handles the `hoop projects` subcommands for managing the ~/.hoop/projects.yaml registry.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
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
    fs::create_dir_all(&home)
        .context("Failed to create ~/.hoop directory")?;
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
    /// Project name (derived from directory name)
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

        let contents = fs::read_to_string(&path)
            .context("Failed to read projects.yaml")?;

        let registry: Self = serde_yaml::from_str(&contents)
            .context("Failed to parse projects.yaml")?;

        Ok(registry)
    }

    /// Save the registry to disk
    pub fn save(&self) -> Result<()> {
        ensure_hoop_dir()?;
        let path = registry_path()?;

        let yaml = serde_yaml::to_string(self)
            .context("Failed to serialize registry")?;

        fs::write(&path, yaml)
            .context("Failed to write projects.yaml")?;

        Ok(())
    }

    /// Add a new project to the registry
    pub fn add(&mut self, path: PathBuf) -> Result<ProjectEntry> {
        // Resolve to absolute path
        let absolute_path = fs::canonicalize(&path)
            .with_context(|| format!("Path does not exist: {}", path.display()))?;

        // Check for .beads directory
        let beads_path = absolute_path.join(".beads");
        if !beads_path.exists() || !beads_path.is_dir() {
            anyhow::bail!(
                "Path does not contain a .beads/ directory: {}",
                absolute_path.display()
            );
        }

        // Extract project name from directory name
        let name = absolute_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Check for duplicate names
        if self.projects.iter().any(|p| p.name == name) {
            anyhow::bail!("Project '{}' already exists in registry", name);
        }

        // Check for duplicate paths
        if self.projects.iter().any(|p| p.path == absolute_path) {
            anyhow::bail!(
                "Path already registered as project '{}'",
                self.projects.iter().find(|p| p.path == absolute_path).unwrap().name
            );
        }

        let entry = ProjectEntry {
            name: name.clone(),
            path: absolute_path.clone(),
        };

        self.projects.push(entry);
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
    let entry = registry.add(absolute_path)?;
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
