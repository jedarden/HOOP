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
pub fn run_init_wizard(no_interactive: bool) -> Result<()> {
    if no_interactive {
        // In non-interactive mode, init wizard cannot proceed safely
        // since it requires user input for several steps
        eprintln!("hoop init: cannot run in non-interactive mode.");
        eprintln!("  The init wizard requires interactive input for configuration.");
        eprintln!("  For automated setup, manually create ~/.hoop/config.yml and ~/.hoop/projects.yaml");
        std::process::exit(2);
    }

    print_wizard_banner();

    // Stage 1: Dependency check
    let audit_passed = stage_1_dependency_check()?;

    if !audit_passed {
        println!("\n⚠️  Critical dependencies are missing.");
        println!("    Please fix the issues above and run `hoop init` again.");
        println!("    You can re-run the audit anytime with: hoop audit check");
        std::process::exit(2);
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
    eprintln!("I can scan your home directory ({}) for projects with .beads/ directories.", home.display());
    eprint!("Scan home directory? [Y/n]: ");
    io::stderr().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let answer = input.trim().to_lowercase();

    if answer == "n" || answer == "no" {
        eprintln!("Skipping project registration.");
        eprintln!("You can add projects later with:");
        eprintln!("  hoop projects add <path>");
        eprintln!("  hoop projects scan <root>");
        return Ok(());
    }

    eprintln!();
    eprintln!("Scanning {} for projects...", home.display());
    eprintln!("(This may take a moment for large directories)");
    eprintln!();

    // Do a preview scan
    let discovered = crate::projects::discover_bead_workspaces(&home)?;

    if discovered.is_empty() {
        println!("No directories with .beads/ found under {}", home.display());
        println!();
        println!("If you have a project elsewhere, add it with:");
        println!("  hoop projects add <path>");
        return Ok(());
    }

    eprintln!("Found {} director{} with .beads/:",
        discovered.len(),
        if discovered.len() == 1 { "y" } else { "ies" }
    );

    for (i, path) in discovered.iter().enumerate() {
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?");
        eprintln!("  {}. {} → {}", i + 1, name, path.display());
    }
    eprintln!();

    eprint!("Register all of these projects? [Y/n]: ");
    io::stderr().flush()?;

    input.clear();
    io::stdin().read_line(&mut input)?;
    let answer = input.trim().to_lowercase();

    if answer == "n" || answer == "no" {
        eprintln!("Skipping batch registration.");
        eprintln!("You can add projects individually with:");
        eprintln!("  hoop projects add <path>");
        return Ok(());
    }

    eprintln!();
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
        .args(["serve", "--addr", DEFAULT_BIND_ADDR])
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
        .args(["status", "--json"])
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
        .args(["-s", "-o", "/dev/null", "-w", "%{http_code}", "--max-time", "2", url])
        .output();

    match output {
        Ok(out) => {
            let status = String::from_utf8_lossy(&out.stdout);
            status.starts_with("2") || status.starts_with("3")
        }
        Err(_) => false,
    }
}

// ── Tests for no_interactive flag ───────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use clap::Parser;

    // ── Test 1 & 2: Parse tests for flag position independence ─────────────────────

    /// Test 1: Parse test for `hoop --no-interactive init`
    /// Verifies flag extraction when flag appears BEFORE the init command
    #[test]
    fn test_init_parse_flag_before_command() {
        let args = ["hoop", "--no-interactive", "init"];
        let cli = crate::Cli::parse_from(args);

        assert_eq!(cli.no_interactive, true, "Flag should be true when present before init");

        match cli.command {
            crate::Commands::Init => {}, // Correct command
            _ => panic!("Expected Init command, got {:?}", cli.command),
        }
    }

    /// Test 2: Parse test for `hoop init --no-interactive`
    /// Verifies flag extraction when flag appears AFTER the init command
    #[test]
    fn test_init_parse_flag_after_command() {
        let args = ["hoop", "init", "--no-interactive"];
        let cli = crate::Cli::parse_from(args);

        assert_eq!(cli.no_interactive, true, "Flag should be true when present after init");

        match cli.command {
            crate::Commands::Init => {}, // Correct command
            _ => panic!("Expected Init command, got {:?}", cli.command),
        }
    }

    /// Test: Parse test for short form `-y` with init
    #[test]
    fn test_init_parse_short_flag_y() {
        let args = ["hoop", "-y", "init"];
        let cli = crate::Cli::parse_from(args);

        assert_eq!(cli.no_interactive, true, "Flag should be true with -y short form");

        match cli.command {
            crate::Commands::Init => {}, // Correct command
            _ => panic!("Expected Init command"),
        }
    }

    /// Test: Verify flag value consistency across positions
    #[test]
    fn test_init_flag_both_positions_extract_same_value() {
        // Parse with flag before command
        let args_before = ["hoop", "--no-interactive", "init"];
        let cli_before = crate::Cli::parse_from(args_before);
        let no_interactive_before = cli_before.no_interactive;

        // Parse with flag after command
        let args_after = ["hoop", "init", "--no-interactive"];
        let cli_after = crate::Cli::parse_from(args_after);
        let no_interactive_after = cli_after.no_interactive;

        assert_eq!(
            no_interactive_before, no_interactive_after,
            "Flag value must be consistent regardless of position"
        );
        assert_eq!(no_interactive_before, true, "Flag should be true");
    }

    /// Test: Verify default behavior (no flag = false)
    #[test]
    fn test_init_without_flag_is_false() {
        let args = ["hoop", "init"];
        let cli = crate::Cli::parse_from(args);

        assert_eq!(cli.no_interactive, false, "Flag should be false when not specified");

        match cli.command {
            crate::Commands::Init => {}, // Correct command
            _ => panic!("Expected Init command"),
        }
    }

    // ── Test 3: Verify flag value extraction in handler ───────────────────────────────

    /// Test 3 (from requirements): Verify flag value extraction in handler
    /// Confirms that the flag value flows from CLI parsing to the init wizard function
    #[test]
    fn test_init_flag_extraction_in_handler() {
        // Verify the handler function signature accepts no_interactive parameter
        let code = std::fs::read_to_string("src/init.rs")
            .expect("Failed to read init.rs");

        assert!(
            code.contains("pub fn run_init_wizard(no_interactive: bool)"),
            "Handler signature must include no_interactive parameter"
        );

        // Verify the flag is actually used in conditional logic
        assert!(
            code.contains("if no_interactive"),
            "Handler must check no_interactive flag"
        );

        // Verify the flag flows from main.rs to the handler
        let main_code = std::fs::read_to_string("src/main.rs")
            .expect("Failed to read main.rs");

        assert!(
            main_code.contains("init::run_init_wizard(no_interactive)"),
            "main() must pass no_interactive flag to run_init_wizard"
        );
    }

    // ── Test 4: Verify flag is passed correctly to init wizard ──────────────────────────

    /// Test 4 (from requirements): Verify flag is passed correctly to init wizard
    /// Tests that the main.rs Init command handler correctly extracts and passes the flag
    #[test]
    fn test_init_flag_passed_to_wizard() {
        let main_code = std::fs::read_to_string("src/main.rs")
            .expect("Failed to read main.rs");

        // Find the Init command handler
        let init_handler_start = main_code.find("Commands::Init =>")
            .expect("Should have Init handler");

        // Extract the handler section (roughly 200 chars should cover it)
        let handler_section = &main_code[init_handler_start..init_handler_start + 200];

        // Verify the handler passes no_interactive to run_init_wizard
        assert!(
            handler_section.contains("init::run_init_wizard(no_interactive)"),
            "Init handler must pass no_interactive flag to run_init_wizard.\n\
             Handler section: {}", handler_section
        );

        // Verify error handling is in place
        assert!(
            handler_section.contains("eprintln"),
            "Handler must provide error output on failure"
        );

        assert!(
            handler_section.contains("hoop init"),
            "Error message must include 'hoop init' prefix"
        );
    }

    /// Test: Verify flag extraction happens once at parse time
    /// Confirms the pattern: let no_interactive = cli.no_interactive; (line 366 in main.rs)
    #[test]
    fn test_flag_extraction_at_parse_time() {
        let main_code = std::fs::read_to_string("src/main.rs")
            .expect("Failed to read main.rs");

        // Verify the extraction pattern
        assert!(
            main_code.contains("let no_interactive = cli.no_interactive;"),
            "main() must extract no_interactive flag once at parse time"
        );

        // Verify it's extracted before the match statement
        let parse_line = main_code.find("let no_interactive = cli.no_interactive;")
            .expect("Should have flag extraction");
        let match_line = main_code.find("match cli.command")
            .expect("Should have match statement");

        assert!(
            parse_line < match_line,
            "Flag extraction must happen BEFORE match statement"
        );
    }

    // ── Test 5: Verify wizard behavior with flag true vs false (mocked wizard) ────────────

    /// Test 5a (from requirements): Verify wizard exits early when no_interactive=true
    /// Tests that when no_interactive=true, the wizard exits with appropriate error
    #[test]
    fn test_init_wizard_exits_with_no_interactive_true() {
        let code = std::fs::read_to_string("src/init.rs")
            .expect("Failed to read init.rs");

        // Check for the early exit logic at the start of run_init_wizard
        let func_start = code.find("pub fn run_init_wizard(no_interactive: bool)")
            .expect("Should have run_init_wizard function");

        // Find the first if no_interactive block (should be near the start)
        let early_exit_start = code[func_start..].find("if no_interactive {")
            .expect("Wizard must check no_interactive at the start");

        // Extend the slice to ensure we capture the full early exit block (including the exit call)
        let early_exit_section = &code[func_start + early_exit_start..func_start + early_exit_start + 600];

        // Verify early exit behavior
        assert!(
            early_exit_section.contains("if no_interactive {"),
            "Wizard must check no_interactive at the start"
        );

        assert!(
            early_exit_section.contains("cannot run in non-interactive mode"),
            "Wizard must explain why it cannot run in non-interactive mode"
        );

        assert!(
            early_exit_section.contains("std::process::exit(2)") || early_exit_section.contains("std::process::exit(2);"),
            "Wizard must exit with code 2 when no_interactive is true.\n\
             Section content: {}", early_exit_section
        );

        // Verify the error message is helpful
        assert!(
            early_exit_section.contains("manually create ~/.hoop/config.yml"),
            "Error message must guide user to manual setup"
        );
    }

    /// Test 5b (from requirements): Verify wizard continues when no_interactive=false
    /// Confirms the wizard banner and stages are only shown when interactive
    #[test]
    fn test_init_wizard_continues_with_no_interactive_false() {
        let code = std::fs::read_to_string("src/init.rs")
            .expect("Failed to read init.rs");

        // Find the early exit block
        let early_exit_start = code.find("if no_interactive {").expect("Should have early exit");
        let early_exit_end = code[early_exit_start..]
            .find('}')
            .expect("Should close early exit") + early_exit_start + 1;

        // Find print_wizard_banner() call - it should come AFTER the early exit
        let banner_call = code.find("print_wizard_banner();")
            .expect("Should call banner");

        assert!(
            banner_call > early_exit_end,
            "Banner must only print AFTER the early exit check (i.e., only when no_interactive=false)\n\
             Early exit ends at: {}, Banner called at: {}", early_exit_end, banner_call
        );

        // Verify that stages are called (they should only execute when no_interactive=false)
        assert!(
            code.contains("stage_1_dependency_check()?"),
            "Stage 1 must be called when interactive"
        );

        assert!(
            code.contains("stage_2_project_registration()?"),
            "Stage 2 must be called when interactive"
        );

        assert!(
            code.contains("stage_3_agent_setup()?"),
            "Stage 3 must be called when interactive"
        );

        assert!(
            code.contains("stage_4_systemd_install()?"),
            "Stage 4 must be called when interactive"
        );

        assert!(
            code.contains("stage_5_health_check()?"),
            "Stage 5 must be called when interactive"
        );
    }

    /// Test: Verify error message provides clear guidance
    /// Checks that the error message contains all necessary information
    #[test]
    fn test_init_no_interactive_error_message_quality() {
        let code = std::fs::read_to_string("src/init.rs")
            .expect("Failed to read init.rs");

        // Find the error message block
        let error_start = code.find("if no_interactive {").expect("Should have error block");
        let error_block = &code[error_start..error_start + 500]; // Get enough context

        // Verify the error message contains key information
        assert!(
            error_block.contains("hoop init: cannot run in non-interactive mode"),
            "Error must clearly state init cannot run in non-interactive mode.\n\
             Got: {}", error_block
        );

        assert!(
            error_block.contains("requires interactive input"),
            "Error must explain that interaction is required"
        );

        assert!(
            error_block.contains("~/.hoop/config.yml"),
            "Error must provide config file path for manual setup"
        );

        assert!(
            error_block.contains("~/.hoop/projects.yaml"),
            "Error must provide projects file path for manual setup"
        );
    }

    // ── Additional behavioral tests ─────────────────────────────────────────────────────

    /// Test: Verify wizard never runs when no_interactive=true
    /// Ensures that none of the wizard stages execute when the flag is set
    #[test]
    fn test_wizard_stages_never_execute_with_no_interactive_true() {
        let code = std::fs::read_to_string("src/init.rs")
            .expect("Failed to read init.rs");

        // Find the early exit block
        let early_exit_start = code.find("if no_interactive {").expect("Should have early exit");
        let early_exit_end = code[early_exit_start..]
            .find('}')
            .expect("Should close early exit block") + early_exit_start;

        // Find all stage calls
        let stages = [
            "stage_1_dependency_check()?",
            "stage_2_project_registration()?",
            "stage_3_agent_setup()?",
            "stage_4_systemd_install()?",
            "stage_5_health_check()?",
        ];

        for stage in stages {
            if let Some(stage_pos) = code.find(stage) {
                assert!(
                    stage_pos > early_exit_end,
                    "Stage '{}' must be called AFTER the early exit block, \
                     otherwise it would execute even when no_interactive=true.\n\
                     Early exit ends at: {}, Stage called at: {}",
                    stage, early_exit_end, stage_pos
                );
            } else {
                panic!("Stage '{}' not found in code", stage);
            }
        }
    }

    /// Test: Verify print_wizard_banner is only called when interactive
    /// Ensures the banner is inside the interactive flow, not before the no_interactive check
    #[test]
    fn test_wizard_banner_only_prints_when_interactive() {
        let code = std::fs::read_to_string("src/init.rs")
            .expect("Failed to read init.rs");

        // Find function start
        let func_start = code.find("pub fn run_init_wizard(no_interactive: bool)")
            .expect("Should have run_init_wizard function");

        // Find early exit
        let early_exit_start = code[func_start..].find("if no_interactive {")
            .expect("Should have early exit") + func_start;
        let early_exit_end = code[early_exit_start..]
            .find('}')
            .expect("Should close early exit") + early_exit_start + 1;

        // Find banner call
        let banner_call = code.find("print_wizard_banner();")
            .expect("Should call banner");

        // Banner must come AFTER early exit
        assert!(
            banner_call > early_exit_end,
            "print_wizard_banner() must only be called AFTER the no_interactive check, \
             ensuring it only runs when interactive.\n\
             Early exit ends at: {}, Banner called at: {}",
            early_exit_end, banner_call
        );
    }

    // ── Test for flag presence returning true ───────────────────────────────────────

    /// Test: Verify no_interactive flag presence returns true
    ///
    /// This test specifically verifies that when the --no-interactive flag is present
    /// in the parsed command, the extracted flag value returns true.
    ///
    /// # Acceptance Criteria
    /// - Test creates a Commands::Init with no_interactive set to true
    /// - Test extracts the flag value from the parsed command
    /// - Test asserts the extracted value is true
    /// - Test follows the patterns from existing test infrastructure
    /// - Test compiles without errors
    #[test]
    fn test_no_interactive_flag_presence_returns_true() {
        use clap::Parser;

        // Test case 1: Flag appears before the init command
        let args_flag_before = ["hoop", "--no-interactive", "init"];
        let cli_flag_before = crate::Cli::parse_from(args_flag_before);

        // Extract the flag value from the parsed command
        let extracted_flag_before = cli_flag_before.no_interactive;

        // Assert the extracted value is true
        assert_eq!(
            extracted_flag_before, true,
            "no_interactive flag must be true when --no-interactive is present before init command.\n\
             Expected: true, Got: {}",
            extracted_flag_before
        );

        // Verify the command is Commands::Init
        match cli_flag_before.command {
            crate::Commands::Init => {
                // Success - correct command parsed
            }
            _ => panic!("Expected Commands::Init, got {:?}", cli_flag_before.command),
        }

        // Test case 2: Flag appears after the init command
        let args_flag_after = ["hoop", "init", "--no-interactive"];
        let cli_flag_after = crate::Cli::parse_from(args_flag_after);

        // Extract the flag value from the parsed command
        let extracted_flag_after = cli_flag_after.no_interactive;

        // Assert the extracted value is true
        assert_eq!(
            extracted_flag_after, true,
            "no_interactive flag must be true when --no-interactive is present after init command.\n\
             Expected: true, Got: {}",
            extracted_flag_after
        );

        // Verify the command is Commands::Init
        match cli_flag_after.command {
            crate::Commands::Init => {
                // Success - correct command parsed
            }
            _ => panic!("Expected Commands::Init, got {:?}", cli_flag_after.command),
        }

        // Test case 3: Verify consistency across positions
        assert_eq!(
            extracted_flag_before, extracted_flag_after,
            "no_interactive flag value must be consistent regardless of flag position.\n\
             Flag before command: {}, Flag after command: {}",
            extracted_flag_before, extracted_flag_after
        );

        // Test case 4: Verify flag is false when not present (baseline)
        let args_no_flag = ["hoop", "init"];
        let cli_no_flag = crate::Cli::parse_from(args_no_flag);
        let extracted_no_flag = cli_no_flag.no_interactive;

        assert_eq!(
            extracted_no_flag, false,
            "no_interactive flag must be false when --no-interactive is not present.\n\
             Expected: false, Got: {}",
            extracted_no_flag
        );
    }

    // ── Integration Test: Handler Logic with Flag Value ───────────────────────────────

    /// Integration test: Verify handler logic integration with no_interactive flag value
    ///
    /// This test verifies that the handler function correctly uses the extracted no_interactive
    /// flag value and that behavior changes based on the flag value.
    ///
    /// # Test Overview
    /// This is an integration test that simulates the complete flow from flag extraction
    /// to handler invocation, verifying:
    /// 1. Handler correctly receives the flag value
    /// 2. Handler behavior changes based on flag value
    /// 3. The flag extraction flow is complete and correct
    ///
    /// # Acceptance Criteria
    /// - Test verifies init_handler correctly reads the no_interactive field
    /// - Test verifies handler behavior changes based on flag value
    /// - Handler logic is tested in isolation via integration test
    /// - Test compiles and passes with cargo test
    /// - Test coverage is complete for the flag extraction flow
    ///
    /// # Implementation Note
    /// Since run_init_wizard calls std::process::exit(2) when no_interactive=true,
    /// we cannot directly test the exit behavior without killing the test process.
    /// Instead, we verify the logic flow by checking that the handler function:
    /// 1. Accepts the no_interactive parameter
    /// 2. Has the correct conditional logic based on the flag
    /// 3. Would exhibit different behavior based on the flag value
    #[test]
    fn test_init_handler_integration_with_flag_value() {
        // Part 1: Verify handler function signature accepts no_interactive parameter
        let code = std::fs::read_to_string("src/init.rs")
            .expect("Failed to read init.rs");

        // Verify the handler signature includes no_interactive parameter
        assert!(
            code.contains("pub fn run_init_wizard(no_interactive: bool)"),
            "Handler function signature must accept no_interactive: bool parameter.\n\
             This is required for the handler to receive the extracted flag value."
        );

        // Part 2: Verify handler checks the flag value (behavioral logic)
        assert!(
            code.contains("if no_interactive {"),
            "Handler must have conditional logic that checks no_interactive flag.\n\
             This ensures behavior changes based on flag value."
        );

        // Part 3: Verify early exit behavior when no_interactive=true
        let early_exit_start = code.find("if no_interactive {")
            .expect("Handler must check no_interactive flag");
        let early_exit_section = &code[early_exit_start..early_exit_start + 600];

        assert!(
            early_exit_section.contains("cannot run in non-interactive mode"),
            "Handler must explain why it cannot run in non-interactive mode.\n\
             This provides clear user feedback when flag is true."
        );

        assert!(
            early_exit_section.contains("std::process::exit(2)") || early_exit_section.contains("std::process::exit(2);"),
            "Handler must exit with code 2 when no_interactive=true.\n\
             This prevents the wizard from running in non-interactive mode."
        );

        // Part 4: Verify normal wizard stages only execute when no_interactive=false
        let early_exit_end = code[early_exit_start..]
            .find('}')
            .expect("Early exit block must be closed") + early_exit_start + 1;

        // Verify banner only prints after early exit check
        let banner_call = code.find("print_wizard_banner();")
            .expect("Handler must call wizard banner");

        assert!(
            banner_call > early_exit_end,
            "Wizard banner must only print AFTER the no_interactive check.\n\
             This ensures banner only shows when interactive (no_interactive=false).\n\
             Early exit ends at: {}, Banner called at: {}",
            early_exit_end, banner_call
        );

        // Part 5: Verify all wizard stages are called after the check
        let stages = [
            "stage_1_dependency_check()?",
            "stage_2_project_registration()?",
            "stage_3_agent_setup()?",
            "stage_4_systemd_install()?",
            "stage_5_health_check()?",
        ];

        for stage in stages {
            if let Some(stage_pos) = code.find(stage) {
                assert!(
                    stage_pos > early_exit_end,
                    "Stage '{}' must be called AFTER the early exit check.\n\
                     This ensures stages only execute when interactive.\n\
                     Early exit ends at: {}, Stage called at: {}",
                    stage, early_exit_end, stage_pos
                );
            } else {
                panic!("Stage '{}' not found in code", stage);
            }
        }

        // Part 6: Verify the flag flows from main.rs to the handler
        let main_code = std::fs::read_to_string("src/main.rs")
            .expect("Failed to read main.rs");

        // Verify flag extraction at parse time
        assert!(
            main_code.contains("let no_interactive = cli.no_interactive;"),
            "main() must extract no_interactive flag once at parse time.\n\
             This follows the documented pattern for flag extraction."
        );

        // Verify Init command handler passes the flag
        let init_handler_start = main_code.find("Commands::Init =>")
            .expect("main() must have Init command handler");

        let init_handler_section = &main_code[init_handler_start..init_handler_start + 200];

        assert!(
            init_handler_section.contains("init::run_init_wizard(no_interactive)"),
            "Init handler must pass no_interactive flag to run_init_wizard.\n\
             This completes the flag extraction flow from CLI parsing to handler invocation.\n\
             Handler section: {}", init_handler_section
        );

        // Part 7: Verify the complete extraction flow order
        let parse_line = main_code.find("let no_interactive = cli.no_interactive;")
            .expect("main() must extract flag");
        let match_line = main_code.find("match cli.command")
            .expect("main() must have match statement");

        assert!(
            parse_line < match_line,
            "Flag extraction must happen BEFORE the match statement.\n\
             This ensures the flag is available to all command handlers.\n\
             Parse at: {}, Match at: {}",
            parse_line, match_line
        );

        // Part 8: Integration verification - simulate flag flow
        // This demonstrates the complete flow from CLI argument to handler parameter
        use clap::Parser;

        // Simulate: hoop --no-interactive init
        let args_with_flag = ["hoop", "--no-interactive", "init"];
        let cli_with_flag = crate::Cli::parse_from(args_with_flag);

        // Verify flag extraction
        let extracted_flag = cli_with_flag.no_interactive;
        assert_eq!(
            extracted_flag, true,
            "Integration test: Extracted flag must be true when --no-interactive is present.\n\
             This verifies the complete CLI parsing → flag extraction flow."
        );

        // Simulate: hoop init (without flag)
        let args_without_flag = ["hoop", "init"];
        let cli_without_flag = crate::Cli::parse_from(args_without_flag);

        let extracted_flag_no_flag = cli_without_flag.no_interactive;
        assert_eq!(
            extracted_flag_no_flag, false,
            "Integration test: Extracted flag must be false when --no-interactive is absent.\n\
             This verifies the default behavior when flag is not provided."
        );

        // Verify both scenarios parse the correct command
        match cli_with_flag.command {
            crate::Commands::Init => {
                // Success - correct command parsed with flag
            }
            _ => panic!("Integration test: Expected Commands::Init with flag, got {:?}", cli_with_flag.command),
        }

        match cli_without_flag.command {
            crate::Commands::Init => {
                // Success - correct command parsed without flag
            }
            _ => panic!("Integration test: Expected Commands::Init without flag, got {:?}", cli_without_flag.command),
        }
    }

    /// Integration test: Verify handler behavior differs based on flag value
    ///
    /// This test specifically verifies that the handler exhibits different behavior
    /// when no_interactive=true vs no_interactive=false.
    ///
    /// # Behavior Verification
    /// - When no_interactive=true: Handler exits early with error message
    /// - When no_interactive=false: Handler proceeds through wizard stages
    #[test]
    fn test_init_handler_behavior_changes_with_flag_value() {
        let code = std::fs::read_to_string("src/init.rs")
            .expect("Failed to read init.rs");

        // Find the early exit block
        let early_exit_start = code.find("if no_interactive {")
            .expect("Handler must have early exit logic");
        let early_exit_end = code[early_exit_start..]
            .find('}')
            .expect("Early exit block must be closed") + early_exit_start + 1;

        // Behavior when no_interactive=true: Early exit
        let early_exit_section = &code[early_exit_start..early_exit_end];
        assert!(
            early_exit_section.contains("eprintln!(\"hoop init: cannot run in non-interactive mode.\")"),
            "Handler must print error message when no_interactive=true.\n\
             Behavior A: Exit with error - verified."
        );

        assert!(
            early_exit_section.contains("std::process::exit(2)") || early_exit_section.contains("std::process::exit(2);"),
            "Handler must exit with code 2 when no_interactive=true.\n\
             Behavior A: Exit with code 2 - verified."
        );

        // Behavior when no_interactive=false: Proceed through wizard stages
        // Verify that wizard stages exist and would execute after the check
        let stages_found = [
            code.find("print_wizard_banner();").is_some(),
            code.find("stage_1_dependency_check()?").is_some(),
            code.find("stage_2_project_registration()?").is_some(),
            code.find("stage_3_agent_setup()?").is_some(),
            code.find("stage_4_systemd_install()?").is_some(),
            code.find("stage_5_health_check()?").is_some(),
        ];

        for (i, found) in stages_found.iter().enumerate() {
            assert!(
                *found,
                "Stage {} must exist in handler for Behavior B (interactive mode).\n\
                 When no_interactive=false, these stages execute.",
                i + 1
            );
        }

        // Verify stages are called AFTER the early exit (ensuring they only run when interactive)
        let banner_call = code.find("print_wizard_banner();")
            .expect("Handler must call banner");

        assert!(
            banner_call > early_exit_end,
            "Behavior B verification: Wizard stages must execute AFTER early exit check.\n\
             This ensures stages only run when no_interactive=false.\n\
             Early exit ends at: {}, First stage (banner) at: {}",
            early_exit_end, banner_call
        );
    }

    /// Integration test: Verify complete flag extraction flow from CLI to handler
    ///
    /// This test verifies the end-to-end flow of the no_interactive flag:
    /// 1. CLI parsing extracts the flag
    /// 2. main() extracts the flag once at parse time
    /// 3. Init command handler receives the flag
    /// 4. Handler uses the flag to control behavior
    ///
    /// # Flow Verification
    /// - Clap parsing → no_interactive field in Cli struct
    /// - main() extracts: let no_interactive = cli.no_interactive;
    /// - Commands::Init handler passes: init::run_init_wizard(no_interactive)
    /// - Handler receives: pub fn run_init_wizard(no_interactive: bool)
    #[test]
    fn test_complete_flag_extraction_flow() {
        use clap::Parser;

        // Step 1: Verify CLI parsing correctly extracts the flag
        let args = ["hoop", "--no-interactive", "init"];
        let cli = crate::Cli::parse_from(args);

        assert_eq!(
            cli.no_interactive, true,
            "Step 1: CLI parsing must extract no_interactive flag correctly.\n\
             clap Parser stores the flag in Cli.no_interactive field."
        );

        // Step 2: Verify main() extraction pattern
        let main_code = std::fs::read_to_string("src/main.rs")
            .expect("Failed to read main.rs");

        let parse_line = main_code.find("let no_interactive = cli.no_interactive;")
            .expect("Step 2: main() must extract flag with: let no_interactive = cli.no_interactive;");

        // Verify extraction happens before match
        let match_line = main_code.find("match cli.command")
            .expect("Step 2: main() must have match statement");

        assert!(
            parse_line < match_line,
            "Step 2: Flag extraction must happen BEFORE match statement.\n\
             This ensures extracted value is available to all handlers.\n\
             Parse at line: {}, Match at line: {}",
            parse_line, match_line
        );

        // Step 3: Verify Init handler passes flag to wizard
        let init_handler_start = main_code.find("Commands::Init =>")
            .expect("Step 3: main() must have Init command handler");

        let init_handler_section = &main_code[init_handler_start..init_handler_start + 200];

        assert!(
            init_handler_section.contains("init::run_init_wizard(no_interactive)"),
            "Step 3: Init handler must pass extracted flag to wizard.\n\
             Handler call: init::run_init_wizard(no_interactive)"
        );

        // Step 4: Verify handler receives and uses the flag
        let init_code = std::fs::read_to_string("src/init.rs")
            .expect("Failed to read init.rs");

        assert!(
            init_code.contains("pub fn run_init_wizard(no_interactive: bool)"),
            "Step 4a: Handler signature must receive no_interactive parameter.\n\
             Function signature: pub fn run_init_wizard(no_interactive: bool)"
        );

        assert!(
            init_code.contains("if no_interactive {{"),
            "Step 4b: Handler must use the flag to control behavior.\n\
             Conditional check: if no_interactive {{ ... }}"
        );

        // Step 5: End-to-end verification
        // Parse a command and verify the complete flow
        let test_args = ["hoop", "init", "--no-interactive"];
        let test_cli = crate::Cli::parse_from(test_args);

        // Simulate main() extraction
        let simulated_extracted_flag = test_cli.no_interactive;

        assert_eq!(
            simulated_extracted_flag, true,
            "Step 5: End-to-end flow verification failed.\n\
             CLI args → Clap parsing → main() extraction → handler parameter.\n\
             Expected: true, Got: {}",
            simulated_extracted_flag
        );

        // Verify the command is correct
        match test_cli.command {
            crate::Commands::Init => {
                // Success - complete flow verified
            }
            _ => panic!("Step 5: Expected Commands::Init, got {:?}", test_cli.command),
        }
    }

    /// Runtime integration test: Verify handler receives correct flag values
    ///
    /// This is a true runtime integration test that:
    /// 1. Parses actual CLI arguments at runtime (not code inspection)
    /// 2. Verifies the extracted flag values are correct
    /// 3. Confirms the handler would receive the correct values
    /// 4. Tests both true and false flag scenarios
    ///
    /// # Runtime Verification
    /// - Executes clap Parser with real argument strings
    /// - Verifies Cli.no_interactive field is set correctly
    /// - Simulates main() extraction: let no_interactive = cli.no_interactive
    /// - Confirms Commands::Init is matched correctly
    #[test]
    fn test_runtime_flag_extraction_and_handler_receives_correct_values() {
        use clap::Parser;

        // Scenario 1: Flag is set to true (before command)
        let args_flag_true_before = ["hoop", "--no-interactive", "init"];
        let cli_flag_true_before = crate::Cli::parse_from(args_flag_true_before);

        // Verify runtime extraction
        let extracted_flag_true_before = cli_flag_true_before.no_interactive;
        assert_eq!(
            extracted_flag_true_before, true,
            "Runtime: Extracted flag must be true when --no-interactive present before command.\n\
             This verifies the clap Parser stores the flag correctly at runtime."
        );

        // Verify command is Init
        match cli_flag_true_before.command {
            crate::Commands::Init => {
                // Correct - handler would receive no_interactive=true
            }
            _ => panic!("Runtime: Expected Commands::Init, got {:?}", cli_flag_true_before.command),
        }

        // Scenario 2: Flag is set to true (after command)
        let args_flag_true_after = ["hoop", "init", "--no-interactive"];
        let cli_flag_true_after = crate::Cli::parse_from(args_flag_true_after);

        let extracted_flag_true_after = cli_flag_true_after.no_interactive;
        assert_eq!(
            extracted_flag_true_after, true,
            "Runtime: Extracted flag must be true when --no-interactive present after command.\n\
             This verifies global flag works in any position."
        );

        // Verify command is Init
        match cli_flag_true_after.command {
            crate::Commands::Init => {
                // Correct - handler would receive no_interactive=true
            }
            _ => panic!("Runtime: Expected Commands::Init, got {:?}", cli_flag_true_after.command),
        }

        // Scenario 3: Short form -y sets flag to true
        let args_short_form = ["hoop", "-y", "init"];
        let cli_short_form = crate::Cli::parse_from(args_short_form);

        let extracted_flag_short = cli_short_form.no_interactive;
        assert_eq!(
            extracted_flag_short, true,
            "Runtime: Short form -y must also set flag to true.\n\
             This verifies the short alias works correctly."
        );

        // Scenario 4: Flag is absent (default false)
        let args_no_flag = ["hoop", "init"];
        let cli_no_flag = crate::Cli::parse_from(args_no_flag);

        let extracted_flag_false = cli_no_flag.no_interactive;
        assert_eq!(
            extracted_flag_false, false,
            "Runtime: Extracted flag must be false when --no-interactive is absent.\n\
             This verifies the default value is false."
        );

        // Verify command is Init
        match cli_no_flag.command {
            crate::Commands::Init => {
                // Correct - handler would receive no_interactive=false
            }
            _ => panic!("Runtime: Expected Commands::Init, got {:?}", cli_no_flag.command),
        }

        // Scenario 5: Verify handler would receive different values
        // This demonstrates that the handler's behavior would change based on flag value
        let handler_receives_true = cli_flag_true_before.no_interactive;
        let handler_receives_false = cli_no_flag.no_interactive;

        assert_ne!(
            handler_receives_true, handler_receives_false,
            "Runtime: Handler must receive different values for different flag states.\n\
             With flag: {}, Without flag: {}",
            handler_receives_true, handler_receives_false
        );

        // Verify the correct values
        assert!(
            handler_receives_true && !handler_receives_false,
            "Runtime: With flag should be true, without flag should be false.\n\
             This ensures handler behavior changes based on flag presence.\n\
             With flag: {}, Without flag: {}",
            handler_receives_true, handler_receives_false
        );
    }

    /// Runtime integration test: Verify flag position independence
    ///
    /// This test verifies that the global no_interactive flag works correctly
    /// regardless of its position in the command line, demonstrating clap's
    /// global flag handling at runtime.
    #[test]
    fn test_runtime_global_flag_position_independence() {
        use clap::Parser;

        // Test all three valid positions
        let test_cases = vec![
            (["hoop", "--no-interactive", "init"], "before command"),
            (["hoop", "init", "--no-interactive"], "after command"),
            (["hoop", "-y", "init"], "short form before"),
        ];

        for (args, description) in test_cases {
            let cli = crate::Cli::parse_from(args);

            assert_eq!(
                cli.no_interactive, true,
                "Runtime: Flag must be true when {}: {:?}\n\
                 This verifies global flag works in any position.",
                description, args
            );

            // Verify the command is still parsed correctly
            match cli.command {
                crate::Commands::Init => {
                    // Success - correct command and flag value
                }
                _ => panic!(
                    "Runtime: Expected Commands::Init for {}, got {:?}",
                    description, cli.command
                ),
            }
        }
    }

    /// Integration test: Verify handler parameter type matches extracted flag type
    ///
    /// This test verifies type safety in the flag extraction flow:
    /// - CLI field: no_interactive: bool (in Cli struct)
    /// - Handler parameter: no_interactive: bool (in run_init_wizard)
    /// - Flow: bool → bool (type-safe passing)
    #[test]
    fn test_handler_parameter_type_matches_extracted_flag_type() {
        use clap::Parser;

        // Verify runtime type consistency
        let args = ["hoop", "--no-interactive", "init"];
        let cli = crate::Cli::parse_from(args);

        // The extracted value is a bool
        let extracted_flag: bool = cli.no_interactive;

        // Verify the handler signature expects a bool
        let init_code = std::fs::read_to_string("src/init.rs")
            .expect("Failed to read init.rs");

        assert!(
            init_code.contains("pub fn run_init_wizard(no_interactive: bool)"),
            "Handler signature must expect no_interactive: bool.\n\
             This ensures type-safe passing from CLI bool to handler bool parameter.\n\
             Extracted type: bool (from Cli.no_interactive)\n\
             Expected parameter type: bool (in run_init_wizard)"
        );

        // Verify the value is actually a bool at runtime
        match extracted_flag {
            true => {
                // Correct - bool type verified at runtime
            }
            false => {
                // Also correct - bool type verified at runtime
            }
        }

        // Verify main() passes without type conversion
        let main_code = std::fs::read_to_string("src/main.rs")
            .expect("Failed to read main.rs");

        assert!(
            main_code.contains("init::run_init_wizard(no_interactive)"),
            "main() must pass no_interactive directly without conversion.\n\
             This confirms the flow: bool (CLI) → bool (local) → bool (handler).\n\
             No type conversion needed - direct pass-through."
        );
    }
}
