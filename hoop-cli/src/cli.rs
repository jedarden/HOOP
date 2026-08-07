//! CLI type definitions for testing and re-use
//!
//! This module exposes the CLI structure (Cli, Commands) for use in tests
//! and other parts of the codebase that need to parse or inspect command-line
//! arguments without executing the full CLI.

use clap::{Parser, Subcommand};
use std::net::SocketAddr;

/// Global CLI structure
///
/// This is the main entry point for command-line parsing. The `no_interactive`
/// flag is global (available to all subcommands) due to the `global = true`
/// attribute in clap.
#[derive(Parser, Debug)]
#[command(name = "hoop")]
#[command(about = "HOOP - The operator's pane of glass", long_about = None)]
pub struct Cli {
    /// Global flag to suppress all interactive prompts (alias: -y)
    ///
    /// When set to `true`, all interactive prompts are suppressed. This is
    /// essential for CI/CD pipelines, non-interactive environments, and
    /// batch operations.
    ///
    /// Because of `global = true`, this flag can be specified at any position:
    /// - Before the subcommand: `hoop --no-interactive scan /tmp`
    /// - After the subcommand: `hoop scan /tmp --no-interactive`
    /// - With short alias: `hoop -y scan /tmp`
    #[arg(short = 'y', long = "no-interactive", global = true)]
    pub no_interactive: bool,

    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands
///
/// Each variant represents a top-level command in the HOOP CLI.
#[derive(Subcommand, Debug)]
pub enum Commands {
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
        #[arg(long = "yes")]
        auto_confirm: bool,
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

    /// First-time setup wizard
    Init,
}

/// Projects subcommands
#[derive(Subcommand, Debug)]
pub enum ProjectsCommands {
    /// Add a project to the registry
    Add {
        /// Path to the workspace
        path: String,
    },

    /// Auto-register every directory with .beads/ under a root path
    Scan {
        /// Root path to scan
        root: String,
        /// Auto-confirm all prompts (non-interactive mode) [local --yes flag]
        #[arg(long, action = clap::ArgAction::SetTrue)]
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

/// Audit subcommands
#[derive(Subcommand, Debug)]
pub enum AuditCommands {
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
