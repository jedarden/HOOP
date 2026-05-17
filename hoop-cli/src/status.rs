//! HOOP status command - CLI overview of fleets / beads / cost
use anyhow::Result;
use hoop_schema::{ProjectsRegistry, ProjectsRegistryProjectsItem, WorkspaceView};
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Status output for JSON serialization
#[derive(Debug, Serialize)]
struct StatusOutput {
    projects: Vec<ProjectStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct ProjectStatus {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    workspaces: Vec<WorkspaceStatus>,
    total_beads: u64,
    open_beads: u64,
    claimed_beads: u64,
    closed_beads: u64,
}

#[derive(Debug, Serialize)]
struct WorkspaceStatus {
    path: String,
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    beads_summary: Option<BeadsSummary>,
}

#[derive(Debug, Serialize)]
struct BeadsSummary {
    total: u64,
    open: u64,
    claimed: u64,
    closed: u64,
}

/// Run the status command
pub fn run(project_filter: Option<String>, json: bool) -> Result<()> {
    let registry = load_projects()?;

    let filtered_projects: Vec<_> = if let Some(ref filter) = project_filter {
        registry.projects.into_iter()
            .filter(|p| p.name() == filter)
            .collect()
    } else {
        registry.projects
    };

    if filtered_projects.is_empty() {
        if let Some(filter) = project_filter {
            let error = format!("Project '{}' not found", filter);
            if json {
                let output = StatusOutput {
                    projects: vec![],
                    error: Some(error.clone()),
                };
                println!("{}", serde_json::to_string_pretty(&output)?);
                std::process::exit(2); // Fatal: project not found
            } else {
                eprintln!("{}", error);
                std::process::exit(2);
            }
        }
    }

    let mut project_statuses = Vec::new();

    for project in filtered_projects {
        let project_status = gather_project_status(&project)?;
        project_statuses.push(project_status);
    }

    if json {
        let output = StatusOutput {
            projects: project_statuses,
            error: None,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        print_human_readable(&project_statuses)?;
    }

    Ok(())
}

/// Load projects from ~/.hoop/projects.yaml
fn load_projects() -> Result<ProjectsRegistry> {
    let config_path = PathBuf::from(std::env::var("HOME").unwrap())
        .join(".hoop")
        .join("projects.yaml");

    if !config_path.exists() {
        // Return empty registry if no projects configured
        return Ok(ProjectsRegistry { projects: vec![] });
    }

    let content = std::fs::read_to_string(&config_path)?;
    let registry: ProjectsRegistry = serde_yaml::from_str(&content)?;
    Ok(registry)
}

/// Gather status information for a single project
fn gather_project_status(project: &ProjectsRegistryProjectsItem) -> Result<ProjectStatus> {
    let workspaces = project.workspace_views();
    let mut workspace_statuses = Vec::new();
    let mut total_beads = 0u64;
    let mut open_beads = 0u64;
    let mut claimed_beads = 0u64;
    let mut closed_beads = 0u64;

    for workspace_view in workspaces {
        let ws_status = gather_workspace_status(&workspace_view)?;

        if let Some(summary) = &ws_status.beads_summary {
            total_beads += summary.total;
            open_beads += summary.open;
            claimed_beads += summary.claimed;
            closed_beads += summary.closed;
        }

        workspace_statuses.push(ws_status);
    }

    // Extract label from project (it's an optional field)
    let label = match project {
        ProjectsRegistryProjectsItem::Variant0 { label, .. } => label.clone(),
        ProjectsRegistryProjectsItem::Variant1 { label, .. } => label.clone(),
    };

    Ok(ProjectStatus {
        name: project.name().to_string(),
        label,
        error: None,
        workspaces: workspace_statuses,
        total_beads,
        open_beads,
        claimed_beads,
        closed_beads,
    })
}

/// Gather status information for a single workspace
fn gather_workspace_status(workspace_view: &WorkspaceView) -> Result<WorkspaceStatus> {
    let beads_path = workspace_view.path.join(".beads");

    let beads_summary = if beads_path.exists() {
        match get_beads_summary(&beads_path) {
            Ok(summary) => Some(summary),
            Err(e) => {
                return Ok(WorkspaceStatus {
                    path: workspace_view.path.to_string_lossy().to_string(),
                    role: workspace_view.role.to_string(),
                    error: Some(format!("Failed to read beads: {}", e)),
                    beads_summary: None,
                });
            }
        }
    } else {
        None
    };

    Ok(WorkspaceStatus {
        path: workspace_view.path.to_string_lossy().to_string(),
        role: workspace_view.role.to_string(),
        error: None,
        beads_summary,
    })
}

/// Get beads summary by calling br list
fn get_beads_summary(beads_path: &Path) -> Result<BeadsSummary> {
    // Try to call br list --json
    let output = std::process::Command::new("br")
        .arg("list")
        .arg("--json")
        .current_dir(beads_path.parent().unwrap())
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let json = std::str::from_utf8(&output.stdout)?;
                if let Ok(beads) = serde_json::from_str::<Vec<serde_json::Value>>(json) {
                    let total = beads.len() as u64;
                    let open = beads.iter()
                        .filter(|b| b["status"] == "open")
                        .count() as u64;
                    let claimed = beads.iter()
                        .filter(|b| b["status"] == "claimed")
                        .count() as u64;
                    let closed = beads.iter()
                        .filter(|b| b["status"] == "closed")
                        .count() as u64;

                    return Ok(BeadsSummary {
                        total,
                        open,
                        claimed,
                        closed,
                    });
                }
            }

            // If br list fails, return empty summary
            Ok(BeadsSummary {
                total: 0,
                open: 0,
                claimed: 0,
                closed: 0,
            })
        }
        Err(_) => {
            // br command not found or failed
            Ok(BeadsSummary {
                total: 0,
                open: 0,
                claimed: 0,
                closed: 0,
            })
        }
    }
}

/// Print human-readable status output
fn print_human_readable(project_statuses: &[ProjectStatus]) -> Result<()> {
    if project_statuses.is_empty() {
        println!("No projects configured. Use 'hoop projects add' to register a workspace.");
        return Ok(());
    }

    for project in project_statuses {
        println!("{} ({})", project.name, project.label.as_ref().unwrap_or(&"unnamed".to_string()));
        println!("  Workspaces:");

        for workspace in &project.workspaces {
            if let Some(error) = &workspace.error {
                println!("    [{}] {} - ERROR: {}", workspace.role, workspace.path, error);
            } else if let Some(summary) = &workspace.beads_summary {
                println!("    [{}] {} - {} beads ({} open, {} claimed, {} closed)",
                    workspace.role, workspace.path, summary.total, summary.open, summary.claimed, summary.closed);
            } else {
                println!("    [{}] {} - no beads data", workspace.role, workspace.path);
            }
        }

        if project.workspaces.len() > 1 {
            println!("  Total: {} beads ({} open, {} claimed, {} closed)",
                project.total_beads, project.open_beads, project.claimed_beads, project.closed_beads);
        }

        println!();
    }

    Ok(())
}
