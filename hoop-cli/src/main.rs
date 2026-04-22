//! HOOP CLI - The operator's interface to the daemon
//!
//! HOOP is the operator's pane of glass and conversational handle for a
//! single long-lived host that holds many repos, many NEEDLE fleets, and
//! many native-CLI conversations.

mod projects;

use clap::Parser;
use hoop_daemon::{audit, serve, Config as DaemonConfig};
use hoop_schema::{ControlRequest, ControlResponse};
use std::{fs, net::SocketAddr, path::PathBuf};
use tokio::io::AsyncBufReadExt;

#[derive(Parser, Debug)]
#[command(name = "hoop")]
#[command(about = "HOOP - The operator's pane of glass", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Run the daemon (web UI + WS + REST)
    Serve {
        /// Bind address (default: 127.0.0.1:3000)
        #[arg(short, long)]
        addr: Option<SocketAddr>,
        /// Skip br version compatibility check (dev override)
        #[arg(long)]
        allow_br_mismatch: bool,
    },
    /// Manage the project registry
    #[command(subcommand)]
    Projects(ProjectsCommands),
    /// Register a workspace
    #[command(arg_required_else_help = true)]
    Add {
        /// Path to the workspace
        path: String,
    },
    /// Auto-register every workspace with .beads/ under a root
    #[command(arg_required_else_help = true)]
    Scan {
        /// Root path to scan
        root: String,
        /// Auto-register all discoveries without prompting
        #[arg(short, long)]
        yes: bool,
    },
    /// List registered projects
    List,
    /// Remove a project
    #[command(arg_required_else_help = true)]
    Remove {
        /// Project name to remove
        name: String,
    },
    /// CLI overview of fleets / beads / cost
    #[command(arg_required_else_help = true)]
    Status {
        /// Optional project filter
        project: Option<String>,
    },
    /// Startup binary/env audit
    Audit {
        /// Output as JSON
        #[arg(short, long)]
        json: bool,
        /// Skip optional checks (Tailscale, systemd)
        #[arg(long)]
        strict: bool,
    },
    /// Attach to or start the human-interface agent conversation
    Agent,
    /// CLI shortcut to draft+submit a Stitch
    #[command(arg_required_else_help = true)]
    New {
        /// Target project
        project: String,
    },
    /// List open Stitches
    #[command(arg_required_else_help = true)]
    Stitch {
        /// Optional project filter
        project: Option<String>,
    },
}

#[derive(clap::Subcommand, Debug)]
enum ProjectsCommands {
    /// Add a project to the registry
    Add {
        /// Path to the workspace
        path: String,
    },
    /// Auto-register every directory with .beads/ under a root path
    Scan {
        /// Root path to scan
        root: String,
        /// Auto-register all discoveries without prompting
        #[arg(short, long)]
        yes: bool,
    },
    /// List registered projects
    List {
        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },
    /// Remove a project from the registry
    Remove {
        /// Project name to remove
        name: String,
    },
    /// Show details for a single project
    Show {
        /// Project name
        name: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Serve {
            addr,
            allow_br_mismatch,
        } => {
            let config = DaemonConfig {
                bind_addr: addr.unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 3000))),
                allow_br_mismatch,
                ..Default::default()
            };
            serve(config).await?
        }
        Commands::Projects(cmd) => {
            if let Err(e) = handle_projects(cmd) {
                eprintln!("hoop projects: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Add { path: _ } => {
            eprintln!("hoop add: not yet implemented");
            std::process::exit(1);
        }
        Commands::Scan { root, yes } => {
            if let Err(e) = projects::scan_projects(&root, yes) {
                eprintln!("hoop scan: {}", e);
                std::process::exit(1);
            }
        }
        Commands::List => {
            eprintln!("hoop list: not yet implemented");
            std::process::exit(1);
        }
        Commands::Remove { name: _ } => {
            eprintln!("hoop remove: not yet implemented");
            std::process::exit(1);
        }
        Commands::Status { project } => {
            if let Err(e) = handle_status(project).await {
                eprintln!("hoop status: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Audit { json, strict } => {
            // Load project paths from config if available
            let project_paths = load_project_paths()?;

            let config = audit::AuditConfig {
                project_paths,
                include_optional: !strict,
                ..Default::default()
            };

            let report = audit::run_audit(&config);

            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_report(&report);
            }

            if !report.success {
                std::process::exit(1);
            }
        }
        Commands::Agent => {
            eprintln!("hoop agent: not yet implemented");
            std::process::exit(1);
        }
        Commands::New { project: _ } => {
            eprintln!("hoop new: not yet implemented");
            std::process::exit(1);
        }
        Commands::Stitch { project: _ } => {
            eprintln!("hoop stitch: not yet implemented");
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Handle the `hoop projects` subcommands
fn handle_projects(cmd: ProjectsCommands) -> anyhow::Result<()> {
    match cmd {
        ProjectsCommands::Add { path } => {
            let entry = projects::add_project(&path)?;
            println!("Added project '{}': {}", entry.name, entry.path.display());
        }
        ProjectsCommands::Scan { root, yes } => {
            projects::scan_projects(&root, yes)?;
        }
        ProjectsCommands::List { json } => {
            let projects = projects::list_projects()?;

            if json {
                println!("{}", serde_json::to_string_pretty(&projects)?);
            } else {
                if projects.is_empty() {
                    println!("No projects registered");
                    println!("\nAdd a project with:");
                    println!("  hoop projects add <path>");
                } else {
                    println!("Registered projects:");
                    for proj in &projects {
                        println!("  {} - {}", proj.name, proj.path.display());
                    }
                }
            }
        }
        ProjectsCommands::Remove { name } => {
            let removed = projects::remove_project(&name)?;
            if removed {
                println!("Removed project '{}'", name);
                println!("Workspace data remains intact at its original location");
            } else {
                eprintln!("Project '{}' not found", name);
                std::process::exit(1);
            }
        }
        ProjectsCommands::Show { name } => {
            if let Some(proj) = projects::show_project(&name)? {
                println!("Project: {}", proj.name);
                println!("Path: {}", proj.path.display());

                let beads_path = proj.path.join(".beads");
                if beads_path.exists() {
                    println!("Status: Active (.beads/ present)");
                    if let Ok(entries) = std::fs::read_dir(beads_path.join("beads")) {
                        let count = entries.filter_map(Result::ok).count();
                        println!("Beads: {}", count);
                    }
                } else {
                    println!("Status: Inactive (.beads/ missing)");
                }
            } else {
                eprintln!("Project '{}' not found", name);
                std::process::exit(1);
            }
        }
    }
    Ok(())
}

/// Handle the `hoop status` command by connecting to the control socket
async fn handle_status(project: Option<String>) -> anyhow::Result<()> {
    let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.push(".hoop");
    let socket_path = home.join("control.sock");

    if !socket_path.exists() {
        anyhow::bail!(
            "Daemon not running (control socket not found at {})",
            socket_path.display()
        );
    }

    let mut socket = tokio::net::UnixStream::connect(&socket_path)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to control socket: {}", e))?;

    let request = ControlRequest::Status { project };
    let request_json = serde_json::to_string(&request)?;

    tokio::io::AsyncWriteExt::write_all(&mut socket, format!("{}\n", request_json).as_bytes())
        .await?;

    let mut reader = tokio::io::BufReader::new(&mut socket);
    let mut response_line = String::new();
    reader.read_line(&mut response_line).await?;

    if response_line.is_empty() {
        anyhow::bail!("No response from daemon");
    }

    let response: ControlResponse = serde_json::from_str(&response_line.trim())?;

    match response {
        ControlResponse::Status(status) => {
            print_status(&status);
            Ok(())
        }
        ControlResponse::Error { message } => {
            anyhow::bail!("Daemon error: {}", message);
        }
    }
}

/// Print status response in human-readable format
fn print_status(status: &hoop_schema::StatusResponse) {
    if !status.daemon_running {
        println!("Daemon: Not running");
        return;
    }

    let uptime = status.uptime_secs;
    let hours = uptime / 3600;
    let minutes = (uptime % 3600) / 60;
    let seconds = uptime % 60;

    println!("HOOP Daemon Status");
    println!("==================");
    println!("Running: Yes");
    println!("Uptime: {}h {}m {}s", hours, minutes, seconds);

    if !status.projects.is_empty() {
        println!("\nProjects:");
        for proj in &status.projects {
            println!("  - {} ({})", proj.name, proj.path);
            println!("    Active beads: {}", proj.active_beads);
            println!("    Workers: {}", proj.workers);
        }
    } else {
        println!("\nNo projects registered");
    }
}

/// Load project paths from ~/.hoop/projects.yaml if it exists
fn load_project_paths() -> anyhow::Result<Vec<PathBuf>> {
    let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.push(".hoop");
    home.push("projects.yaml");

    if !home.exists() {
        return Ok(Vec::new());
    }

    let contents = fs::read_to_string(&home)?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&contents)
        .map_err(|e| anyhow::anyhow!("Failed to parse projects.yaml: {}", e))?;

    let mut paths = Vec::new();

    if let Some(projects) = yaml.get("projects").and_then(|p| p.as_sequence()) {
        for project in projects {
            // Check for shorthand single-workspace form
            if let Some(path) = project.get("path").and_then(|p| p.as_str()) {
                paths.push(PathBuf::from(path));
            }
            // Check for multi-workspace form
            if let Some(workspaces) = project.get("workspaces").and_then(|w| w.as_sequence()) {
                for ws in workspaces {
                    if let Some(path) = ws.get("path").and_then(|p| p.as_str()) {
                        paths.push(PathBuf::from(path));
                    }
                }
            }
        }
    }

    Ok(paths)
}

/// Print audit report in human-readable format
fn print_report(report: &audit::AuditReport) {
    use audit::Severity;

    println!("HOOP Runtime Audit");
    println!("==================\n");

    for check in &report.checks {
        let icon = if check.passed {
            "\u{2705}" // ✅
        } else {
            match check.severity {
                Severity::Critical => "\u{274C}",        // ❌
                Severity::Warning => "\u{26A0}\u{FE0F}", // ⚠️
                Severity::Info => "\u{2139}",            // ℹ️
            }
        };

        println!("{} {}", icon, check.name);

        if check.passed {
            println!("   {}", check.description);
        } else {
            println!("   \u{001b}[31m{}\u{001b}[0m", check.description);
            if let Some(fix) = &check.fix_command {
                println!("   Fix: {}", fix);
            }
        }

        if let Some(detail) = &check.detail {
            println!("   ({})", detail);
        }

        println!();
    }

    // Summary
    let passed = report.checks.iter().filter(|c| c.passed).count();
    let total = report.checks.len();
    let critical = report.critical_failures().len();
    let warnings = report.warnings().len();

    println!("Summary: {}/{} checks passed", passed, total);

    if critical > 0 {
        println!("         {} critical failure(s)", critical);
    }
    if warnings > 0 {
        println!("         {} warning(s)", warnings);
    }

    if !report.success {
        println!("\n\u{001b}[31m\u{001b}[1mCritical failures detected. Fix these before starting the daemon.\u{001b}[0m");
        std::process::exit(1);
    } else if warnings > 0 {
        println!(
            "\n\u{001b}[33mWarnings detected. Daemon will start with degraded features.\u{001b}[0m"
        );
    } else {
        println!("\n\u{001b}[32m\u{001b}[1mAll checks passed!\u{001b}[0m");
    }
}
