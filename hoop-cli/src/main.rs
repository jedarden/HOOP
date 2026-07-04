//! HOOP CLI - The operator's interface to the daemon
//!
//! HOOP is the operator's pane of glass and conversational handle for a
//! single long-lived host that holds many repos, many NEEDLE fleets, and
//! many native-CLI conversations.

mod backup;
mod config;
mod init;
mod new;
mod patterns;
mod projects;
mod restore;
mod risk_patterns;
mod script;
mod skills;
mod status;

use clap::Parser;
use hoop_daemon::{audit, fleet, serve, Config as DaemonConfig};
use serde::Serialize;
use std::{fs, net::SocketAddr, path::PathBuf};

#[derive(Parser, Debug)]
#[command(name = "hoop")]
#[command(about = "HOOP - The operator's pane of glass", long_about = None)]
struct Cli {
    /// Global flag to suppress all interactive prompts (alias: -y)
    ///
    /// The `global = true` attribute ensures this flag is available to all subcommands.
    /// It can be specified at any level: `hoop --no-interactive <subcommand>` or
    /// `hoop <subcommand> --no-interactive`. The flag value is extracted once at
    /// parse time (line 253) and passed to command handlers that need it.
    #[arg(short = 'y', long = 'no-interactive', global = true)]
    no_interactive: bool,

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
        /// Observer mode: read-only attach to primary daemon
        #[arg(long)]
        observer: bool,
        /// Primary daemon address (for observer mode, default: 127.0.0.1:3000)
        #[arg(long)]
        primary_addr: Option<SocketAddr>,
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
        /// Auto-confirm all prompts (non-interactive mode)
        #[arg(long, short = 'y')]
        yes: bool,
    },
    /// List registered projects
    List,
    /// Remove a project
    #[command(arg_required_else_help = true)]
    Remove {
        /// Project name to remove
        name: String,
        /// Required safety confirmation when in non-interactive mode
        #[arg(long)]
        confirm: bool,
    },
    /// CLI overview of fleets / beads / cost
    Status {
        /// Optional project filter
        project: Option<String>,
        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },
    /// Audit operations
    #[command(subcommand)]
    Audit(AuditCommands),
    /// Attach to or start the human-interface agent conversation
    Agent,
    /// CLI shortcut to draft+submit a Stitch
    #[command(arg_required_else_help = true)]
    New {
        /// Target project
        project: String,
        /// Validate and print the payload without submitting to the daemon
        #[arg(long)]
        dry_run: bool,
    },
    /// List open Stitches
    #[command(arg_required_else_help = true)]
    Stitch {
        /// Optional project filter
        project: Option<String>,
    },
    /// Install systemd user service
    InstallSystemd,
    /// Manage backups
    #[command(subcommand)]
    Backup(backup::BackupCommands),
    /// Restore from a prior snapshot (requires daemon stopped)
    #[command(arg_required_else_help = true)]
    Restore {
        /// S3 URI: s3://<bucket>/<prefix>/<snapshot-id>
        #[arg(long)]
        from: String,
        /// Validate and show what would be restored without making changes
        #[arg(long)]
        dry_run: bool,
        /// Required safety confirmation when in non-interactive mode
        #[arg(long)]
        confirm: bool,
    },
    /// Manage schema migrations
    #[command(subcommand)]
    Migrate(MigrateCommands),
    /// Manage and run scripts
    #[command(subcommand)]
    Script(script::ScriptCommands),
    /// Manage daemon configuration
    #[command(subcommand)]
    Config(config::ConfigCommands),
    /// Manage risk patterns
    #[command(subcommand)]
    RiskPatterns(risk_patterns::RiskPatternsCommands),
    /// Manage agent-invocable skills
    #[command(subcommand)]
    Skills(skills::SkillsCommands),
    /// Manage patterns (operator-curated groups of Stitches)
    #[command(subcommand)]
    Pattern(patterns::PatternCommands),
    /// First-time setup wizard
    Init,
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
        /// Auto-confirm all prompts (non-interactive mode)
        #[arg(long, short = 'y')]
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
        /// Required safety confirmation when in non-interactive mode
        #[arg(long)]
        confirm: bool,
    },
    /// Show details for a single project
    Show {
        /// Project name
        name: String,
    },
}

#[derive(clap::Subcommand, Debug)]
enum AuditCommands {
    /// Startup binary/env audit
    Check {
        /// Output as JSON
        #[arg(short, long)]
        json: bool,
        /// Skip optional checks (Tailscale, systemd)
        #[arg(long)]
        strict: bool,
    },
    /// Verify audit log hash chain integrity
    Verify {
        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },
}

#[derive(clap::Subcommand, Debug)]
enum MigrateCommands {
    /// Run pending migrations (minor version upgrades only)
    Run {
        /// Required safety confirmation
        #[arg(long)]
        confirm: bool,
    },
    /// Show migration status and pending migrations
    Status {
        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },
    /// Perform a major version upgrade (e.g., 1.x → 2.x)
    MajorUpgrade {
        /// Source major version to upgrade from (e.g., "1" for 1.x → 2.x)
        ///
        /// This is an explicit safety check: the command will only run if the
        /// current schema's major version matches. Use to avoid accidental
        /// upgrades on the wrong database.
        #[arg(long)]
        from: Option<u32>,
        /// Required safety confirmation
        #[arg(long)]
        confirm: bool,
    },
    /// Rollback to a previous minor version (not available for major upgrades)
    Rollback {
        /// Target version to rollback to
        version: String,
        /// Required safety confirmation
        #[arg(long)]
        confirm: bool,
    },
    /// Rebuild the percentile index from closed Stitches
    RebuildPercentileIndex,
}

#[derive(clap::Subcommand, Debug)]
enum ConfigCommands {
    /// Show configuration diff (running vs config.yml)
    Diff,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let no_interactive = cli.no_interactive;

    match cli.command {
        Commands::Serve {
            addr,
            observer,
            primary_addr,
            allow_br_mismatch,
        } => {
            let bind_addr = addr.unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 3000)));
            let primary_addr = primary_addr.unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 3000)));

            // In observer mode, default bind to a different port to avoid conflict
            let bind_addr = if observer && addr.is_none() {
                SocketAddr::from(([127, 0, 0, 1], 3001))
            } else {
                bind_addr
            };

            let config = DaemonConfig {
                bind_addr,
                observer_mode: observer,
                primary_addr,
                allow_br_mismatch,
                ..Default::default()
            };
            serve(config).await?
        }
        Commands::Projects(cmd) => {
            if let Err(e) = handle_projects(cmd, no_interactive) {
                eprintln!("hoop projects: {}", e);
                std::process::exit(exit_code_for_error(&e));
            }
        }
        Commands::Add { path: _ } => {
            eprintln!("hoop add: not yet implemented");
            std::process::exit(1);
        }
        Commands::Scan { root, yes } => {
            if let Err(e) = projects::scan_projects(&root, no_interactive || yes) {
                eprintln!("hoop scan: {}", e);
                std::process::exit(exit_code_for_error(&e));
            }
        }
        Commands::List => {
            eprintln!("hoop list: not yet implemented");
            std::process::exit(1);
        }
        Commands::Remove { name, confirm } => {
            let removed = projects::remove_project(&name, no_interactive, confirm)?;
            if removed {
                println!("Removed project '{}'", name);
                println!("Workspace data remains intact at its original location");
            } else {
                eprintln!("Project '{}' not found", name);
                std::process::exit(2);
            }
        }
        Commands::Status { project, json } => {
            if let Err(e) = status::run(project, json) {
                eprintln!("hoop status: {}", e);
                std::process::exit(exit_code_for_error(&e));
            }
        }
        Commands::Audit(cmd) => {
            if let Err(e) = handle_audit(cmd) {
                eprintln!("hoop audit: {}", e);
                std::process::exit(exit_code_for_error(&e));
            }
        }
        Commands::Agent => {
            eprintln!("hoop agent: not yet implemented");
            std::process::exit(1);
        }
        Commands::New { project, dry_run } => {
            if let Err(e) = new::run(&project, dry_run).await {
                eprintln!("hoop new: {}", e);
                std::process::exit(exit_code_for_error(&e));
            }
        }
        Commands::Stitch { project: _ } => {
            eprintln!("hoop stitch: not yet implemented");
            std::process::exit(1);
        }
        Commands::InstallSystemd => {
            if let Err(e) = install_systemd() {
                eprintln!("hoop install-systemd: {}", e);
                std::process::exit(exit_code_for_error(&e));
            }
        }
        Commands::Backup(cmd) => {
            if let Err(e) = backup::handle_backup(cmd).await {
                eprintln!("hoop backup: {}", e);
                std::process::exit(exit_code_for_error(&e));
            }
        }
        Commands::Restore { from, dry_run, confirm } => {
            if let Err(e) = restore::run_restore(&from, dry_run, no_interactive, confirm).await {
                eprintln!("hoop restore: {}", e);
                std::process::exit(exit_code_for_error(&e));
            }
        }
        Commands::Migrate(cmd) => {
            if let Err(e) = handle_migrate(cmd) {
                eprintln!("hoop migrate: {}", e);
                std::process::exit(exit_code_for_error(&e));
            }
        }
        Commands::Script(cmd) => {
            if let Err(e) = script::handle_script(cmd).await {
                eprintln!("hoop script: {}", e);
                std::process::exit(exit_code_for_error(&e));
            }
        }
        Commands::Config(cmd) => {
            if let Err(e) = config::handle_config(cmd).await {
                eprintln!("hoop config: {}", e);
                std::process::exit(exit_code_for_error(&e));
            }
        }
        Commands::RiskPatterns(cmd) => {
            if let Err(e) = risk_patterns::handle_risk_patterns(cmd).await {
                eprintln!("hoop risk-patterns: {}", e);
                std::process::exit(exit_code_for_error(&e));
            }
        }
        Commands::Skills(cmd) => {
            if let Err(e) = skills::handle_skills(cmd).await {
                eprintln!("hoop skills: {}", e);
                std::process::exit(exit_code_for_error(&e));
            }
        }
        Commands::Pattern(cmd) => {
            if let Err(e) = patterns::handle_patterns(cmd).await {
                eprintln!("hoop pattern: {}", e);
                std::process::exit(exit_code_for_error(&e));
            }
        }
        Commands::Init => {
            if let Err(e) = init::run_init_wizard(no_interactive) {
                eprintln!("hoop init: {}", e);
                std::process::exit(exit_code_for_error(&e));
            }
        }
    }

    Ok(())
}

/// Determine appropriate exit code for an error
/// Returns 0 for success (should not be called on success), 1 for partial failure, 2 for fatal
fn exit_code_for_error(e: &anyhow::Error) -> i32 {
    // Check for specific fatal/precondition errors
    let msg = e.to_string().to_lowercase();
    if msg.contains("not found")
        || msg.contains("does not exist")
        || msg.contains("required")
        || msg.contains("--confirm is required")
        || msg.contains("precondition")
    {
        return 2; // Fatal / precondition not met
    }
    1 // Partial failure
}

/// Handle the `hoop projects` subcommands
fn handle_projects(cmd: ProjectsCommands, no_interactive: bool) -> anyhow::Result<()> {
    match cmd {
        ProjectsCommands::Add { path } => {
            let entry = projects::add_project(&path)?;
            let ws_path = entry
                .primary_path()
                .unwrap_or_else(|| std::path::Path::new("?"));
            println!("Added project '{}': {}", entry.name, ws_path.display());
        }
        ProjectsCommands::Scan { root, yes } => {
            projects::scan_projects(&root, no_interactive || yes)?;
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
                        let ws_path = proj
                            .primary_path()
                            .unwrap_or_else(|| std::path::Path::new("?"));
                        println!("  {} - {}", proj.name, ws_path.display());
                    }
                }
            }
        }
        ProjectsCommands::Remove { name, confirm } => {
            let removed = projects::remove_project(&name, no_interactive, confirm)?;
            if removed {
                println!("Removed project '{}'", name);
                println!("Workspace data remains intact at its original location");
            } else {
                eprintln!("Project '{}' not found", name);
                std::process::exit(2);
            }
        }
        ProjectsCommands::Show { name } => {
            if let Some(proj) = projects::show_project(&name)? {
                println!("Project: {}", proj.name);
                if let Some(ws_path) = proj.primary_path() {
                    println!("Path: {}", ws_path.display());
                    let beads_path = ws_path.join(".beads");
                    if beads_path.exists() {
                        println!("Status: Active (.beads/ present)");
                        if let Ok(entries) = std::fs::read_dir(beads_path.join("beads")) {
                            let count = entries.filter_map(Result::ok).count();
                            println!("Beads: {}", count);
                        }
                    } else {
                        println!("Status: Inactive (.beads/ missing)");
                    }
                }
                if proj.workspaces.len() > 1 {
                    println!("Workspaces:");
                    for ws in &proj.workspaces {
                        println!("  {} ({})", ws.path.display(), ws.role);
                    }
                }
            } else {
                eprintln!("Project '{}' not found", name);
                std::process::exit(2);
            }
        }
    }
    Ok(())
}

/// Handle the `hoop audit` subcommands
fn handle_audit(cmd: AuditCommands) -> anyhow::Result<()> {
    match cmd {
        AuditCommands::Check { json, strict } => {
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
        AuditCommands::Verify { json } => match fleet::verify_hash_chain() {
            Ok(()) => {
                let final_hash = fleet::get_final_audit_hash()?;
                if json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "status": "ok",
                            "message": "Audit log hash chain is intact",
                            "final_hash": final_hash
                        })
                    );
                } else {
                    println!("Audit log hash chain is intact");
                    println!("Final hash: {}", final_hash);
                }
            }
            Err(e) => {
                if json {
                    eprintln!(
                        "{}",
                        serde_json::json!({
                            "status": "error",
                            "message": e.to_string()
                        })
                    );
                } else {
                    eprintln!("Hash chain verification failed: {}", e);
                }
                std::process::exit(1);
            }
        },
    }
    Ok(())
}

/// Handle the `hoop migrate` subcommands
fn handle_migrate(cmd: MigrateCommands) -> anyhow::Result<()> {
    use hoop_daemon::{fleet, migrations};

    match cmd {
        MigrateCommands::Run { confirm } => {
            if !confirm {
                eprintln!("hoop migrate run: --confirm is required.");
                eprintln!("  This will apply pending minor version migrations.");
                eprintln!("  Re-run with --confirm once you have verified you have a current backup.");
                std::process::exit(2);
            }

            // Open the database
            let db_path = fleet::db_path();
            let conn = &mut rusqlite::Connection::open(&db_path)?;

            // Get the current schema version
            let current_version = fleet::get_schema_version(conn)?;

            // Check if this is a major version gate
            if let Err(e) = fleet::check_schema_major_gate(&current_version, fleet::SCHEMA_VERSION) {
                eprintln!("Major upgrade required: {}", e);
                eprintln!("  Run: hoop migrate major-upgrade --confirm");
                std::process::exit(2);
            }

            // Run pending migrations
            migrations::run_pending_migrations(conn, &migrations::get_migration_registry(), &current_version)?;

            println!("Migration complete. Schema version is now {}.", fleet::SCHEMA_VERSION);
            println!("You can now start the daemon with `hoop serve`.");
        }
        MigrateCommands::Status { json } => {
            let db_path = fleet::db_path();
            let conn = rusqlite::Connection::open(&db_path)?;
            let registry = migrations::get_migration_registry();
            let status = migrations::get_migration_status(&conn, &registry)?;

            if json {
                println!("{}", serde_json::to_string_pretty(&status)?);
            } else {
                println!("Schema version: {}", status.current_version);
                println!("Binary version: {}", fleet::SCHEMA_VERSION);

                if status.pending_migrations.is_empty() {
                    println!("\nNo pending migrations.");
                } else {
                    println!("\nPending migrations:");
                    for pending in &status.pending_migrations {
                        let rollback = if pending.can_rollback { " (rollbackable)" } else { "" };
                        println!("  {} → {}{}", status.current_version, pending.version, rollback);
                        println!("    {}", pending.description);
                    }
                }

                if !status.can_rollback_to.is_empty() {
                    println!("\nCan rollback to: {}", status.can_rollback_to.join(", "));
                }
            }
        }
        MigrateCommands::MajorUpgrade { from, confirm } => {
            if !confirm {
                eprintln!("hoop migrate major-upgrade: --confirm is required.");
                eprintln!("  This will perform a major version upgrade (e.g., 1.x → 2.x).");
                eprintln!("  Re-run with --confirm once you have verified you have a current backup.");
                std::process::exit(2);
            }

            // If --from is provided, verify the current schema major version matches
            if let Some(expected_major) = from {
                let db_path = fleet::db_path();
                let conn = rusqlite::Connection::open(&db_path)?;
                let current_version = fleet::get_schema_version(&conn)?;
                let current_major = current_version
                    .split('.')
                    .next()
                    .and_then(|v| v.parse::<u32>().ok());

                if current_major != Some(expected_major) {
                    eprintln!("hoop migrate major-upgrade: --from {} does not match current schema version {}",
                        expected_major, current_version);
                    eprintln!("  This safety check prevents accidental upgrades on the wrong database.");
                    eprintln!("  Omit --from to skip this check, or verify you're targeting the correct database.");
                    std::process::exit(2);
                }
            }

            if let Err(e) = fleet::run_major_upgrade() {
                eprintln!("hoop migrate major-upgrade: {}", e);
                std::process::exit(1);
            }

            println!("Major upgrade complete. Schema version is now {}.", fleet::SCHEMA_VERSION);
            println!("You can now start the daemon with `hoop serve`.");
        }
        MigrateCommands::Rollback { version, confirm } => {
            if !confirm {
                eprintln!("hoop migrate rollback: --confirm is required.");
                eprintln!("  This will rollback schema to version {}.", version);
                eprintln!("  Re-run with --confirm once you have verified you have a current backup.");
                std::process::exit(2);
            }

            let db_path = fleet::db_path();
            let conn = &mut rusqlite::Connection::open(&db_path)?;
            let current_version = fleet::get_schema_version(conn)?;
            let registry = migrations::get_migration_registry();

            if !registry.can_rollback(&version) {
                eprintln!("Cannot rollback to version {}.", version);
                eprintln!("  Either the migration does not exist or does not support rollback.");
                eprintln!("  Major version upgrades cannot be rolled back.");
                std::process::exit(2);
            }

            migrations::rollback_migration(conn, &registry, &version, &current_version)?;

            println!("Rollback complete. Schema version is now {}.", version);
        }
        MigrateCommands::RebuildPercentileIndex => {
            let db_path = fleet::db_path();
            let conn = &mut rusqlite::Connection::open(&db_path)?;

            // Check if the percentile index table exists
            let table_exists: bool = conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='stitch_percentile_index'",
                [],
                |row| row.get(0),
            ).unwrap_or(0) > 0;

            if !table_exists {
                eprintln!("hoop migrate rebuild-percentile-index: percentile index table does not exist.");
                eprintln!("  Run pending migrations first: hoop migrate run --confirm");
                std::process::exit(1);
            }

            println!("Rebuilding percentile index from closed Stitches...");
            if let Err(e) = hoop_daemon::stitch_percentile_index::rebuild_index(conn) {
                eprintln!("Failed to rebuild percentile index: {}", e);
                std::process::exit(1);
            }

            let bucket_count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM stitch_percentile_index",
                [],
                |row| row.get(0),
            ).unwrap_or(0);

            println!("Percentile index rebuilt successfully.");
            println!("Total buckets: {}", bucket_count);
        }
    }

    Ok(())
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
        println!("\u{001b}[32m\u{001b}[1mAll checks passed!\u{001b}[0m");
    }
}

/// Install the systemd user service for HOOP
fn install_systemd() -> anyhow::Result<()> {
    let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.push(".config");
    home.push("systemd");
    home.push("user");

    // Create the directory if it doesn't exist
    fs::create_dir_all(&home)?;

    let service_path = home.join("hoop.service");

    // Get the hoop binary path
    let hoop_path = std::env::current_exe()?;
    let hoop_path_str = hoop_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid hoop binary path"))?;

    // Get the username
    let _username = std::env::var("USER").unwrap_or_else(|_| "user".to_string());

    // Get the home directory for environment variables
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let home_dir_str = home_dir
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid home directory"))?;

    // Create the systemd unit file content
    let unit_content = format!(
        r#"[Unit]
Description=HOOP daemon - Control plane for NEEDLE fleets
After=network.target tailscale.service

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
ExecStart={hoop_path_str} serve --addr 127.0.0.1:3000

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=hoop

# Security
NoNewPrivileges=true
PrivateTmp=true

[Install]
WantedBy=default.target
"#
    );

    // Write the service file
    fs::write(&service_path, unit_content)?;

    println!("Installed systemd user service to:");
    println!("  {}", service_path.display());
    println!();
    println!("To enable and start the service:");
    println!("  systemctl --user daemon-reload");
    println!("  systemctl --user enable hoop");
    println!("  systemctl --user start hoop");
    println!();
    println!("To view logs:");
    println!("  journalctl --user -u hoop -f");
    println!();
    println!("To check status:");
    println!("  systemctl --user status hoop");

    Ok(())
}
