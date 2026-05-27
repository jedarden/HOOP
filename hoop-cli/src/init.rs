//! hoop init - First-time setup wizard
//!
//! Walks through five stages of initial setup:
//! 1. Dependency check (runs `hoop audit`)
//! 2. First project registration (offers `scan ~/` preview)
//! 3. Agent adapter setup (optional; Anthropic/Claude Code/ZAI)
//! 4. systemd install
//! 5. Health check + URL print
//!
//! Re-runnable and idempotent — each step can be skipped if already done.

use anyhow::{Context, Result};
use hoop_daemon::audit;
use serde::Deserialize;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

/// Default daemon bind address
const DEFAULT_BIND_ADDR: &str = "127.0.0.1:3000";

/// Tailscale status JSON structure
#[derive(Debug, Deserialize)]
struct TailscaleStatus {
    #[serde(default)]
    #[serde(rename = "Self")]
    tail_self: TailscaleSelf,
}

#[derive(Debug, Default, Deserialize)]
struct TailscaleSelf {
    #[serde(default)]
    DNSName: String,
}

/// Run the init wizard
pub fn run_init_wizard() -> Result<()> {
    print_wizard_banner();

    // Stage 1: Dependency check
    let audit_passed = stage_1_dependency_check()?;

    if !audit_passed {
        println!("\n⚠️  Critical dependencies are missing.");
        println!("    Please fix the issues above and run `hoop init` again.");
        println!("    You can re-run the audit anytime with: hoop audit check");
        std::process::exit(1);
    }

    // Stage 2: First project registration
    stage_2_project_registration()?;

    // Stage 3: Agent adapter setup (optional)
    stage_3_agent_setup()?;

    // Stage 4: systemd install (optional)
    stage_4_systemd_install()?;

    // Stage 5: Start daemon and health check
    stage_5_health_check()?;

    Ok(())
}

/// Print the wizard banner
fn print_wizard_banner() {
    println!();
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║                    HOOP Setup Wizard                         ║");
    println!("║                     First-Time Setup                         ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();
    println!("This wizard will guide you through setting up HOOP for the first time.");
    println!("Each step can be skipped if already configured.");
    println!();
}

/// Stage 1: Dependency check
///
/// Runs `hoop audit check` and displays results.
fn stage_1_dependency_check() -> Result<bool> {
    println!("─────────────────────────────────────────────────────────────────");
    println!("Stage 1: Dependency Check");
    println!("─────────────────────────────────────────────────────────────────");
    println!();

    // Load project paths from config if available
    let project_paths = load_project_paths()?;

    let config = audit::AuditConfig {
        project_paths,
        include_optional: true,
        ..Default::default()
    };

    let report = audit::run_audit(&config);

    // Print the report
    print_audit_report(&report);

    Ok(report.success)
}

/// Stage 2: First project registration
///
/// Offers to scan ~/ for bead workspaces and register the first project.
fn stage_2_project_registration() -> Result<()> {
    println!();
    println!("─────────────────────────────────────────────────────────────────");
    println!("Stage 2: Project Registration");
    println!("─────────────────────────────────────────────────────────────────");
    println!();

    // Check if projects already exist
    let existing_projects = crate::projects::list_projects()?;
    if !existing_projects.is_empty() {
        println!("✓ You already have {} project(s) registered.", existing_projects.len());
        println!("  Skipping project registration.");
        println!("  Manage projects with: hoop projects list/add/remove");
        return Ok(());
    }

    println!("No projects registered yet. Let's add your first project.");
    println!();

    // Offer to scan home directory
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    println!("I can scan your home directory ({}) for projects with .beads/ directories.", home.display());
    print!("Scan home directory? [Y/n]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let answer = input.trim().to_lowercase();

    if answer == "n" || answer == "no" {
        println!("Skipping project registration.");
        println!("You can add projects later with:");
        println!("  hoop projects add <path>");
        println!("  hoop projects scan <root>");
        return Ok(());
    }

    println!();
    println!("Scanning {} for projects...", home.display());
    println!("(This may take a moment for large directories)");
    println!();

    // Do a preview scan
    let discovered = crate::projects::discover_bead_workspaces(&home)?;

    if discovered.is_empty() {
        println!("No directories with .beads/ found under {}", home.display());
        println!();
        println!("If you have a project elsewhere, add it with:");
        println!("  hoop projects add <path>");
        return Ok(());
    }

    println!("Found {} director{} with .beads/:",
        discovered.len(),
        if discovered.len() == 1 { "y" } else { "ies" }
    );

    for (i, path) in discovered.iter().enumerate() {
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?");
        println!("  {}. {} → {}", i + 1, name, path.display());
    }
    println!();

    print!("Register all of these projects? [Y/n]: ");
    io::stdout().flush()?;

    input.clear();
    io::stdin().read_line(&mut input)?;
    let answer = input.trim().to_lowercase();

    if answer == "n" || answer == "no" {
        println!("Skipping batch registration.");
        println!("You can add projects individually with:");
        println!("  hoop projects add <path>");
        return Ok(());
    }

    println!();
    // Use scan_projects with auto_yes=true
    crate::projects::scan_projects(home.to_str().unwrap(), true)?;

    Ok(())
}

/// Stage 3: Agent adapter setup
///
/// Optionally configures the agent adapter (Claude Code, Anthropic API, or ZAI).
fn stage_3_agent_setup() -> Result<()> {
    println!();
    println!("─────────────────────────────────────────────────────────────────");
    println!("Stage 3: Agent Adapter Setup (Optional)");
    println!("─────────────────────────────────────────────────────────────────");
    println!();

    // Check if config exists and has agent configured
    let config_path = get_config_path();
    let agent_configured = config_path.exists()
        && agent_configured_in_file(&config_path)?;

    if agent_configured {
        println!("✓ Agent adapter already configured in {}", config_path.display());
        println!("  Skipping agent setup.");
        println!("  Change adapter anytime with: hoop config set agent.adapter <type>");
        return Ok(());
    }

    println!("HOOP can use different LLM providers for the human-interface agent.");
    println!();
    println!("Available adapters:");
    println!("  1. claude      - Claude Code CLI (default, requires claude CLI)");
    println!("  2. anthropic   - Anthropic API (requires API key)");
    println!("  3. zai         - ZAI proxy with GLM models");
    println!();
    print!("Select adapter [1-3, or Enter to skip]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let answer = input.trim();

    if answer.is_empty() {
        println!("Skipping agent setup. The default 'claude' adapter will be used.");
        return Ok(());
    }

    let adapter = match answer {
        "1" | "claude" => {
            println!();
            println!("✓ Selected: Claude Code CLI");
            println!("  Make sure the 'claude' CLI is installed and in your PATH.");
            println!("  Get it from: https://claude.ai/code");
            "claude"
        }
        "2" | "anthropic" => {
            println!();
            println!("✓ Selected: Anthropic API");
            println!("  You'll need to set your API key:");
            println!("  export HOOP_AGENT_ANTHROPIC_API_KEY=sk-ant-...");
            "anthropic"
        }
        "3" | "zai" => {
            println!();
            println!("✓ Selected: ZAI proxy");
            println!("  Configure your ZAI proxy URL and API key:");
            println!("  export HOOP_ZAI_BASE_URL=https://your-zai-proxy");
            println!("  export HOOP_ZAI_API_KEY=your-key");
            "zai"
        }
        _ => {
            println!("Invalid selection. Skipping agent setup.");
            return Ok(());
        }
    };

    // Update config file
    ensure_config_exists()?;
    append_to_config(&format!("  adapter: \"{}\"", adapter))?;

    println!();
    println!("✓ Agent adapter configured to: {}", adapter);

    Ok(())
}

/// Stage 4: systemd install
///
/// Optionally installs the systemd user service.
fn stage_4_systemd_install() -> Result<()> {
    println!();
    println!("─────────────────────────────────────────────────────────────────");
    println!("Stage 4: systemd Service Setup (Optional)");
    println!("─────────────────────────────────────────────────────────────────");
    println!();

    // Check if service file already exists
    let service_path = get_systemd_service_path();
    if service_path.exists() {
        println!("✓ systemd user service already installed at:");
        println!("  {}", service_path.display());
        println!("  Skipping systemd setup.");
        return Ok(());
    }

    println!("HOOP can run as a systemd user service for auto-start on login.");
    println!();
    print!("Install systemd user service? [Y/n]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let answer = input.trim().to_lowercase();

    if answer == "n" || answer == "no" {
        println!("Skipping systemd setup.");
        println!("You can start the daemon manually with: hoop serve");
        return Ok(());
    }

    println!();
    install_systemd_service()?;

    Ok(())
}

/// Stage 5: Health check
///
/// Starts the daemon and verifies it's healthy.
fn stage_5_health_check() -> Result<()> {
    println!();
    println!("─────────────────────────────────────────────────────────────────");
    println!("Stage 5: Health Check");
    println!("─────────────────────────────────────────────────────────────────");
    println!();

    // Check if daemon is already running
    let health_url = format!("http://{}/healthz", DEFAULT_BIND_ADDR);
    if check_url(&health_url) {
        println!("✓ HOOP daemon is already running and healthy!");
        println!();
        print_access_urls();
        return Ok(());
    }

    println!("Starting the HOOP daemon...");
    println!();

    // Start the daemon in the background
    let mut child = Command::new("hoop")
        .args(&["serve", "--addr", DEFAULT_BIND_ADDR])
        .spawn()
        .context("Failed to start hoop daemon. Is it installed and in PATH?")?;

    println!("Waiting for daemon to start...");
    println!("(This may take a few seconds)");

    // Poll for health
    for i in 0..30 {
        thread::sleep(Duration::from_millis(500));
        if check_url(&health_url) {
            println!();
            println!("✓ HOOP daemon is healthy and ready!");
            println!();
            print_access_urls();
            println!();
            println!("To stop the daemon, press Ctrl+C or run:");
            println!("  pkill -f 'hoop serve'");
            println!();
            println!("─────────────────────────────────────────────────────────────────");
            println!("✓ Setup complete! Welcome to HOOP.");
            println!("─────────────────────────────────────────────────────────────────");
            return Ok(());
        }

        // Show progress dots
        if i % 4 == 0 {
            print!(".");
            io::stdout().flush()?;
        }
    }

    println!();
    println!("⚠️  Daemon started but health check timed out after 15 seconds.");
    println!("  Check if it's running with: systemctl --user status hoop");
    println!("  Or view logs: journalctl --user -u hoop -f");

    // Try to clean up the child process
    let _ = child.kill();

    Ok(())
}

/// Print access URLs (localhost + Tailscale if available)
fn print_access_urls() {
    println!("Open in your browser:");

    // Always print localhost
    println!("  http://{}", DEFAULT_BIND_ADDR);

    // Try to get Tailscale hostname
    match get_tailscale_hostname() {
        Some(hostname) => {
            println!();
            println!("  Also accessible via Tailscale:");
            println!("  http://{}:3000", hostname);
            println!();
            println!("  (Make sure HOOP is bound to 0.0.0.0 for Tailscale access)");
        }
        None => {
            println!();
            println!("  Note: Tailscale not detected. To enable Tailscale access:");
            println!("  1. Install Tailscale and join your network");
            println!("  2. Edit ~/.hoop/config.yml and set server.bind_addr to 0.0.0.0:3000");
            println!("  3. Restart HOOP");
        }
    }
}

/// Get the Tailscale hostname (if available)
///
/// Runs `tailscale status --json` and extracts the DNSName field.
/// Returns None if Tailscale is not installed or not logged in.
fn get_tailscale_hostname() -> Option<String> {
    let output = Command::new("tailscale")
        .args(&["status", "--json"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let json_str = String::from_utf8(output.stdout).ok()?;
    let status: TailscaleStatus = serde_json::from_str(&json_str).ok()?;

    let hostname = status.tail_self.DNSName;
    if hostname.is_empty() {
        None
    } else {
        Some(hostname)
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Load project paths from ~/.hoop/projects.yaml
fn load_project_paths() -> Result<Vec<PathBuf>> {
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
            if let Some(path) = project.get("path").and_then(|p| p.as_str()) {
                paths.push(PathBuf::from(path));
            }
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
fn print_audit_report(report: &audit::AuditReport) {
    use audit::Severity;

    for check in &report.checks {
        let icon = if check.passed {
            "✓"
        } else {
            match check.severity {
                Severity::Critical => "✗",
                Severity::Warning => "⚠",
                Severity::Info => "ℹ",
            }
        };

        println!("{} {}", icon, check.name);

        if !check.passed {
            println!("   {}", check.description);
            if let Some(fix) = &check.fix_command {
                println!("   Fix: {}", fix);
            }
        }

        if let Some(detail) = &check.detail {
            println!("   ({})", detail);
        }
    }

    println!();
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
}

/// Get the config file path
fn get_config_path() -> PathBuf {
    let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.push(".hoop");
    home.push("config.yml");
    home
}

/// Get the systemd service file path
fn get_systemd_service_path() -> PathBuf {
    let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.push(".config");
    home.push("systemd");
    home.push("user");
    home.push("hoop.service");
    home
}

/// Check if agent is configured in config file
fn agent_configured_in_file(path: &PathBuf) -> Result<bool> {
    let contents = fs::read_to_string(path)?;
    Ok(contents.contains("agent:") && contents.contains("adapter:"))
}

/// Ensure config file exists with basic structure
fn ensure_config_exists() -> Result<()> {
    let config_path = get_config_path();
    let hoop_dir = config_path.parent().unwrap();

    fs::create_dir_all(hoop_dir).context("Failed to create .hoop directory")?;

    if !config_path.exists() {
        let default_config = format!(
            r#"# HOOP daemon configuration
# Generated by hoop init

schema_version: "1.0.0"

server:
  bind_addr: "{}"

agent:
"#,
            DEFAULT_BIND_ADDR
        );
        fs::write(&config_path, default_config)
            .context("Failed to write config file")?;
    }

    Ok(())
}

/// Append a line to the config file
fn append_to_config(line: &str) -> Result<()> {
    let config_path = get_config_path();
    let mut contents = fs::read_to_string(&config_path)?;
    contents.push_str(line);
    contents.push('\n');
    fs::write(&config_path, contents)?;
    Ok(())
}

/// Install the systemd user service
fn install_systemd_service() -> Result<()> {
    let service_path = get_systemd_service_path();
    let service_dir = service_path.parent().unwrap();

    fs::create_dir_all(service_dir)?;

    let hoop_path = std::env::current_exe()
        .context("Failed to get hoop binary path")?;
    let hoop_path_str = hoop_path
        .to_str()
        .context("Invalid hoop binary path")?;

    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let home_dir_str = home_dir
        .to_str()
        .context("Invalid home directory")?;

    let unit_content = format!(
        r#"[Unit]
Description=HOOP daemon - Control plane for NEEDLE fleets
After=network.target

[Service]
Type=simple
Restart=on-failure
RestartSec=5s
StartLimitBurst=5
StartLimitIntervalSec=5min
TimeoutStartSec=30
TimeoutStopSec=30
Environment="HOME={home_dir_str}"
Environment="PATH=/usr/local/bin:/usr/bin:/bin"
ExecStart={hoop_path_str} serve --addr {bind_addr}

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=hoop

# Security
NoNewPrivileges=true
PrivateTmp=true

[Install]
WantedBy=default.target
"#,
        bind_addr = DEFAULT_BIND_ADDR
    );

    fs::write(&service_path, unit_content)?;

    println!("✓ systemd user service installed to:");
    println!("  {}", service_path.display());
    println!();
    println!("To enable and start the service:");
    println!("  systemctl --user daemon-reload");
    println!("  systemctl --user enable hoop");
    println!("  systemctl --user start hoop");
    println!();
    println!("To view logs:");
    println!("  journalctl --user -u hoop -f");

    Ok(())
}

/// Check if a URL is reachable
fn check_url(url: &str) -> bool {
    let output = Command::new("curl")
        .args(&["-s", "-o", "/dev/null", "-w", "%{http_code}", "--max-time", "2", url])
        .output();

    match output {
        Ok(out) => {
            let status = String::from_utf8_lossy(&out.stdout);
            status.starts_with("2") || status.starts_with("3")
        }
        Err(_) => false,
    }
}
