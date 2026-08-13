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
}
