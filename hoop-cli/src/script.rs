//! `hoop script` subcommand — operator-invoked scripts (§22.3)

use anyhow::{Context, Result};
use clap::Subcommand;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum ScriptCommands {
    /// Run a script by name
    Run {
        /// Script name to run
        name: String,
        /// Arguments to pass to the script
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// List all available scripts
    List {
        /// Filter by project (shows global + matching project scripts)
        #[arg(short, long)]
        project: Option<String>,
    },
    /// Show details for a specific script
    Show {
        /// Script name
        name: String,
    },
}

/// Script execution request
#[derive(Debug, Serialize)]
struct ScriptRunRequest {
    args: Vec<String>,
    project: Option<String>,
}

/// Script execution response
#[derive(Debug, Deserialize)]
struct ScriptRunResponse {
    script: String,
    exit_code: Option<i32>,
    timed_out: bool,
    stdout: String,
    stderr: String,
    duration_ms: u64,
    status: String,
}

/// Script entry from list API
#[derive(Debug, Deserialize)]
struct ScriptEntry {
    name: String,
    path: PathBuf,
    manifest: Option<ScriptManifest>,
    executable: bool,
}

/// Script manifest
#[derive(Debug, Deserialize)]
struct ScriptManifest {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    scope: String,
    #[serde(default)]
    projects: Vec<String>,
    #[serde(default)]
    timeout_secs: u64,
}

/// Get the daemon API URL from control socket or default
fn get_daemon_url() -> String {
    // Try to read from control socket first
    if let Ok(home) = std::env::var("HOME") {
        let socket_path = format!("{}/.hoop/control.sock", home);
        // For now, use default localhost URL
        // TODO: Implement Unix socket communication
    }
    "http://127.0.0.1:3000".to_string()
}

/// Run a script
pub async fn run_script(name: String, args: Vec<String>) -> Result<()> {
    let client = Client::new();
    let base_url = get_daemon_url();

    let url = format!("{}/api/scripts/{}/run", base_url, name);
    let req_body = ScriptRunRequest {
        args,
        project: None,
    };

    eprintln!("Running script: {}", name);

    let response = client
        .post(&url)
        .json(&req_body)
        .send()
        .await
        .context("Failed to send script run request")?;

    if !response.status().is_success() {
        let status = response.status();
        let error = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        anyhow::bail!("Script execution failed ({}): {}", status, error);
    }

    let result: ScriptRunResponse = response
        .json()
        .await
        .context("Failed to parse script response")?;

    // Print stdout
    if !result.stdout.is_empty() {
        print!("{}", result.stdout);
    }

    // Print stderr
    if !result.stderr.is_empty() {
        eprint!("{}", result.stderr);
    }

    // Print status summary
    eprintln!();
    eprintln!("Status: {}", result.status);
    eprintln!("Duration: {}ms", result.duration_ms);

    // Exit with script's exit code
    if let Some(code) = result.exit_code {
        if code != 0 {
            std::process::exit(code);
        }
    } else if result.timed_out {
        anyhow::bail!("Script timed out");
    }

    Ok(())
}

/// List available scripts
pub async fn list_scripts(project: Option<String>) -> Result<()> {
    let client = Client::new();
    let base_url = get_daemon_url();

    let mut url = format!("{}/api/scripts", base_url);
    if let Some(proj) = &project {
        url.push_str(&format!("?project={}", proj));
    }

    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to fetch scripts list")?;

    if !response.status().is_success() {
        let status = response.status();
        let error = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        anyhow::bail!("Failed to list scripts ({}): {}", status, error);
    }

    let scripts: Vec<ScriptEntry> = response
        .json()
        .await
        .context("Failed to parse scripts list")?;

    if scripts.is_empty() {
        println!("No scripts found");
        println!();
        println!("Scripts should be placed in ~/.hoop/scripts/ and be executable (+x).");
        println!("See hoop-cli(1) for details on script manifests.");
        return Ok(());
    }

    println!("Available scripts:");
    println!();

    for script in &scripts {
        let exec_marker = if script.executable {
            ""
        } else {
            " (not executable)"
        };
        println!("  {}{}", script.name, exec_marker);

        if let Some(manifest) = &script.manifest {
            if !manifest.description.is_empty() {
                println!("    Description: {}", manifest.description);
            }
            if manifest.scope == "project" && !manifest.projects.is_empty() {
                println!("    Projects: {}", manifest.projects.join(", "));
            }
        }
        println!();
    }

    Ok(())
}

/// Show script details
pub async fn show_script(name: String) -> Result<()> {
    let client = Client::new();
    let base_url = get_daemon_url();

    let url = format!("{}/api/scripts/{}", base_url, name);

    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to fetch script details")?;

    if !response.status().is_success() {
        let status = response.status();
        let error = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        anyhow::bail!("Failed to get script ({}): {}", status, error);
    }

    let script: ScriptEntry = response
        .json()
        .await
        .context("Failed to parse script details")?;

    println!("Script: {}", script.name);
    println!("Path: {}", script.path.display());
    println!("Executable: {}", script.executable);

    if let Some(manifest) = &script.manifest {
        println!();
        println!("Manifest:");
        if !manifest.description.is_empty() {
            println!("  Description: {}", manifest.description);
        }
        println!("  Scope: {}", manifest.scope);
        if manifest.scope == "project" && !manifest.projects.is_empty() {
            println!("  Projects: {}", manifest.projects.join(", "));
        }
        println!("  Timeout: {}s", manifest.timeout_secs);
    }

    Ok(())
}

/// Handle script subcommands
pub async fn handle_script(cmd: ScriptCommands) -> Result<()> {
    match cmd {
        ScriptCommands::Run { name, args } => run_script(name, args).await?,
        ScriptCommands::List { project } => list_scripts(project).await?,
        ScriptCommands::Show { name } => show_script(name).await?,
    }
    Ok(())
}
