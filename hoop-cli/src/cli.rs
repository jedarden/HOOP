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
#[command(args_override_self = true)]
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
#[derive(Subcommand, Debug, Clone)]
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
#[derive(Subcommand, Debug, Clone)]
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
#[derive(Subcommand, Debug, Clone)]
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

// ── Tests for no_interactive flag in Remove command ─────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Test Helper Functions ─────────────────────────────────────────────────────

    /// Helper function to parse CLI arguments and extract the parsed Cli struct
    ///
    /// This is the foundational helper for all clap parser tests. It wraps
    /// `Cli::try_parse_from` and returns a `Result` type that can be used with
    /// `expect()` for successful parses or `assert!(result.is_err())` for failures.
    ///
    /// # Usage
    ///
    /// ```rust
    /// let cli = parse_args(&["hoop", "init"]).expect("should parse successfully");
    /// assert!(cli.command.is_init());
    /// ```
    fn parse_args(args: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(args)
    }

    /// Helper function to build Init command arguments with optional flags
    ///
    /// Constructs a vector of command-line arguments for the Init command,
    /// optionally including the `--no-interactive` flag. This helper reduces
    /// boilerplate when testing multiple flag configurations.
    ///
    /// # Arguments
    ///
    /// * `no_interactive` - If true, includes `--no-interactive` flag
    ///
    /// # Returns
    ///
    /// A vector of string slices representing the full command line
    ///
    /// # Usage
    ///
    /// ```rust
    /// let args = build_init_args(true);
    /// assert_eq!(args, vec!["hoop", "--no-interactive", "init"]);
    /// ```
    #[allow(dead_code)]
    fn build_init_args(no_interactive: bool) -> Vec<&'static str> {
        let mut args = vec!["hoop"];
        if no_interactive {
            args.push("--no-interactive");
        }
        args.push("init");
        args
    }

    /// Helper function to parse Init command and extract both flag and command
    ///
    /// Provides a convenient way to get both the `no_interactive` flag value
    /// and the parsed `Commands` enum in a single call. This is useful for
    /// tests that need to verify both the flag value and the command variant.
    ///
    /// # Arguments
    ///
    /// * `args` - Slice of command-line arguments (should start with "hoop")
    ///
    /// # Returns
    ///
    /// `Result<(bool, Commands), clap::Error>` - Tuple of (no_interactive flag, command)
    ///
    /// # Usage
    ///
    /// ```rust
    /// let (no_interactive, command) = parse_init_command(&["hoop", "--no-interactive", "init"])
    ///     .expect("should parse successfully");
    /// assert_eq!(no_interactive, true);
    /// assert!(matches!(command, Commands::Init));
    /// ```
    #[allow(dead_code)]
    fn parse_init_command(args: &[&str]) -> Result<(bool, Commands), clap::Error> {
        let cli = Cli::try_parse_from(args)?;
        Ok((cli.no_interactive, cli.command))
    }

    /// Helper function to build generic command arguments with optional no_interactive flag
    ///
    /// A more flexible version of `build_init_args` that works with any command.
    /// Use this when testing commands other than Init.
    ///
    /// # Arguments
    ///
    /// * `command` - The command string (e.g., "init", "remove", "scan")
    /// * `no_interactive` - If true, includes `--no-interactive` flag
    ///
    /// # Returns
    ///
    /// A vector of string slices representing the full command line
    ///
    /// # Usage
    ///
    /// ```rust
    /// let args = build_command_args("scan", true);
    /// assert_eq!(args, vec!["hoop", "--no-interactive", "scan"]);
    /// ```
    #[allow(dead_code)]
    fn build_command_args(command: &'static str, no_interactive: bool) -> Vec<&'static str> {
        let mut args = vec!["hoop"];
        if no_interactive {
            args.push("--no-interactive");
        }
        args.push(command);
        args
    }

    // ── Top-level Remove command tests ───────────────────────────────────────

    // ── Top-level Remove command tests ───────────────────────────────────────

    #[test]
    fn test_remove_no_interactive_flag_before_command() {
        let args = [
            "hoop",
            "--no-interactive",
            "remove",
            "my-project",
            "--confirm",
        ];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            cli.no_interactive,
            "no_interactive should be true when flag appears before command"
        );

        // Verify the command was parsed correctly
        match cli.command {
            Commands::Remove { name, confirm } => {
                assert_eq!(name, "my-project");
                assert!(confirm);
            }
            _ => panic!("expected Remove command"),
        }
    }

    #[test]
    fn test_remove_no_interactive_flag_after_command() {
        let args = [
            "hoop",
            "remove",
            "my-project",
            "--no-interactive",
            "--confirm",
        ];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            cli.no_interactive,
            "no_interactive should be true when flag appears after command"
        );

        match cli.command {
            Commands::Remove { name, confirm } => {
                assert_eq!(name, "my-project");
                assert!(confirm);
            }
            _ => panic!("expected Remove command"),
        }
    }

    #[test]
    fn test_remove_short_flag_y_before_command() {
        let args = ["hoop", "-y", "remove", "my-project", "--confirm"];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            cli.no_interactive,
            "no_interactive should be true with -y short flag"
        );

        match cli.command {
            Commands::Remove { name, confirm } => {
                assert_eq!(name, "my-project");
                assert!(confirm);
            }
            _ => panic!("expected Remove command"),
        }
    }

    #[test]
    fn test_remove_short_flag_y_after_command() {
        let args = ["hoop", "remove", "my-project", "-y", "--confirm"];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            cli.no_interactive,
            "no_interactive should be true with -y short flag after command"
        );

        match cli.command {
            Commands::Remove { name, confirm } => {
                assert_eq!(name, "my-project");
                assert!(confirm);
            }
            _ => panic!("expected Remove command"),
        }
    }

    #[test]
    fn test_remove_without_no_interactive_flag_is_false() {
        let args = ["hoop", "remove", "my-project", "--confirm"];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            !cli.no_interactive,
            "no_interactive should be false when flag is not provided"
        );

        match cli.command {
            Commands::Remove { name, confirm } => {
                assert_eq!(name, "my-project");
                assert!(confirm);
            }
            _ => panic!("expected Remove command"),
        }
    }

    #[test]
    fn test_remove_no_interactive_flag_extraction_consistency() {
        // Test that flag parsing is consistent regardless of position
        let args_before = [
            "hoop",
            "--no-interactive",
            "remove",
            "test-project",
            "--confirm",
        ];
        let args_after = [
            "hoop",
            "remove",
            "test-project",
            "--no-interactive",
            "--confirm",
        ];

        let cli_before = parse_args(&args_before).expect("should parse successfully");
        let cli_after = parse_args(&args_after).expect("should parse successfully");

        assert_eq!(
            cli_before.no_interactive, cli_after.no_interactive,
            "no_interactive value must be consistent regardless of flag position"
        );
        assert!(cli_before.no_interactive, "no_interactive should be true");
    }

    #[test]
    fn test_remove_command_with_all_flags() {
        let args = [
            "hoop",
            "--no-interactive",
            "remove",
            "my-project",
            "--confirm",
        ];
        let cli = parse_args(&args).expect("should parse successfully");

        assert!(cli.no_interactive);

        match cli.command {
            Commands::Remove { name, confirm } => {
                assert_eq!(name, "my-project");
                assert!(confirm);
            }
            _ => panic!("expected Remove command"),
        }
    }

    // ── Projects::Remove subcommand tests ──────────────────────────────────────

    #[test]
    fn test_projects_remove_no_interactive_flag_before_subcommand() {
        let args = [
            "hoop",
            "--no-interactive",
            "projects",
            "remove",
            "my-project",
            "--confirm",
        ];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            cli.no_interactive,
            "no_interactive should be true when flag appears before projects subcommand"
        );

        match cli.command {
            Commands::Projects(ProjectsCommands::Remove { name, confirm }) => {
                assert_eq!(name, "my-project");
                assert!(confirm);
            }
            _ => panic!("expected Projects::Remove command"),
        }
    }

    #[test]
    fn test_projects_remove_no_interactive_flag_after_subcommand() {
        let args = [
            "hoop",
            "projects",
            "remove",
            "my-project",
            "--no-interactive",
            "--confirm",
        ];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            cli.no_interactive,
            "no_interactive should be true when flag appears after projects remove subcommand"
        );

        match cli.command {
            Commands::Projects(ProjectsCommands::Remove { name, confirm }) => {
                assert_eq!(name, "my-project");
                assert!(confirm);
            }
            _ => panic!("expected Projects::Remove command"),
        }
    }

    #[test]
    fn test_projects_remove_short_flag_y_before_subcommand() {
        let args = [
            "hoop",
            "-y",
            "projects",
            "remove",
            "my-project",
            "--confirm",
        ];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            cli.no_interactive,
            "no_interactive should be true with -y short flag before projects subcommand"
        );

        match cli.command {
            Commands::Projects(ProjectsCommands::Remove { name, confirm }) => {
                assert_eq!(name, "my-project");
                assert!(confirm);
            }
            _ => panic!("expected Projects::Remove command"),
        }
    }

    #[test]
    fn test_projects_remove_without_no_interactive_flag_is_false() {
        let args = ["hoop", "projects", "remove", "my-project", "--confirm"];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            !cli.no_interactive,
            "no_interactive should be false when flag is not provided"
        );

        match cli.command {
            Commands::Projects(ProjectsCommands::Remove { name, confirm }) => {
                assert_eq!(name, "my-project");
                assert!(confirm);
            }
            _ => panic!("expected Projects::Remove command"),
        }
    }

    #[test]
    fn test_projects_remove_no_interactive_flag_extraction_consistency() {
        // Test that flag parsing is consistent for projects remove regardless of position
        let args_before = [
            "hoop",
            "--no-interactive",
            "projects",
            "remove",
            "test-project",
            "--confirm",
        ];
        let args_after = [
            "hoop",
            "projects",
            "remove",
            "test-project",
            "--no-interactive",
            "--confirm",
        ];

        let cli_before = parse_args(&args_before).expect("should parse successfully");
        let cli_after = parse_args(&args_after).expect("should parse successfully");

        assert_eq!(cli_before.no_interactive, cli_after.no_interactive,
                   "no_interactive value must be consistent for projects remove regardless of flag position");
        assert!(cli_before.no_interactive, "no_interactive should be true");
    }

    #[test]
    fn test_projects_remove_confirm_flag_extraction() {
        let args = [
            "hoop",
            "--no-interactive",
            "projects",
            "remove",
            "my-project",
            "--confirm",
        ];
        let cli = parse_args(&args).expect("should parse successfully");

        match cli.command {
            Commands::Projects(ProjectsCommands::Remove { name, confirm }) => {
                assert_eq!(
                    name, "my-project",
                    "project name should be extracted correctly"
                );
                assert!(confirm, "confirm flag should be extracted correctly");
            }
            _ => panic!("expected Projects::Remove command"),
        }
    }

    #[test]
    fn test_projects_remove_without_confirm_flag() {
        let args = [
            "hoop",
            "--no-interactive",
            "projects",
            "remove",
            "my-project",
        ];
        let cli = parse_args(&args).expect("should parse successfully");

        match cli.command {
            Commands::Projects(ProjectsCommands::Remove { name, confirm }) => {
                assert_eq!(
                    name, "my-project",
                    "project name should be extracted correctly"
                );
                assert!(!confirm, "confirm flag should be false when not provided");
            }
            _ => panic!("expected Projects::Remove command"),
        }
    }

    // ── Global flag persistence tests ─────────────────────────────────────────

    #[test]
    fn test_no_interactive_persists_through_projects_command_chain() {
        let args = [
            "hoop",
            "--no-interactive",
            "projects",
            "remove",
            "test-project",
            "--confirm",
        ];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            cli.no_interactive,
            "global no_interactive flag should persist through the entire command chain"
        );
    }

    #[test]
    fn test_global_flag_works_with_nested_subcommands() {
        let args = [
            "hoop",
            "--no-interactive",
            "projects",
            "remove",
            "test-project",
            "--confirm",
        ];
        let cli = parse_args(&args).expect("should parse successfully");

        // Verify the flag is accessible at the top level
        assert!(cli.no_interactive);

        // Verify nested command structure is correct
        match cli.command {
            Commands::Projects(ProjectsCommands::Remove { .. }) => {
                // Success - command parsed correctly
            }
            _ => panic!("expected Projects::Remove command"),
        }
    }

    // ── Error handling tests ─────────────────────────────────────────────────

    #[test]
    fn test_remove_command_requires_project_name() {
        let args = ["hoop", "--no-interactive", "remove"];
        let result = parse_args(&args);
        assert!(result.is_err(), "should fail when project name is missing");
    }

    #[test]
    fn test_projects_remove_command_requires_project_name() {
        let args = ["hoop", "--no-interactive", "projects", "remove"];
        let result = parse_args(&args);
        assert!(
            result.is_err(),
            "should fail when project name is missing for projects remove"
        );
    }

    #[test]
    fn test_remove_command_parsing_with_invalid_flag() {
        let args = [
            "hoop",
            "--no-interactive",
            "remove",
            "my-project",
            "--invalid-flag",
        ];
        let result = parse_args(&args);
        assert!(result.is_err(), "should fail with invalid flag");
    }

    // ── Edge cases ───────────────────────────────────────────────────────────

    #[test]
    fn test_remove_with_special_characters_in_project_name() {
        let project_name = "my-project-123";
        let args = [
            "hoop",
            "--no-interactive",
            "remove",
            project_name,
            "--confirm",
        ];
        let cli = parse_args(&args).expect("should parse successfully");

        match cli.command {
            Commands::Remove { name, .. } => {
                assert_eq!(
                    name, project_name,
                    "project name with special characters should be parsed correctly"
                );
            }
            _ => panic!("expected Remove command"),
        }
    }

    #[test]
    fn test_projects_remove_with_special_characters_in_project_name() {
        let project_name = "test-project_456";
        let args = [
            "hoop",
            "--no-interactive",
            "projects",
            "remove",
            project_name,
            "--confirm",
        ];
        let cli = parse_args(&args).expect("should parse successfully");

        match cli.command {
            Commands::Projects(ProjectsCommands::Remove { name, .. }) => {
                assert_eq!(name, project_name, "project name with special characters should be parsed correctly for projects remove");
            }
            _ => panic!("expected Projects::Remove command"),
        }
    }

    #[test]
    fn test_remove_command_flag_order_independence() {
        // Test different flag orderings
        let test_cases = vec![
            ["hoop", "--no-interactive", "remove", "proj", "--confirm"],
            ["hoop", "remove", "--no-interactive", "proj", "--confirm"],
            ["hoop", "remove", "proj", "--no-interactive", "--confirm"],
            ["hoop", "remove", "proj", "--confirm", "--no-interactive"],
        ];

        for args in test_cases {
            let cli = parse_args(&args).expect("should parse successfully with any flag ordering");
            assert!(
                cli.no_interactive,
                "no_interactive should be true regardless of flag order"
            );

            match &cli.command {
                Commands::Remove { name, confirm } => {
                    assert_eq!(name, "proj");
                    assert!(*confirm);
                }
                _ => panic!("expected Remove command"),
            }
        }
    }

    #[test]
    fn test_projects_remove_command_flag_order_independence() {
        // Test different flag orderings for projects remove
        let test_cases = vec![
            [
                "hoop",
                "--no-interactive",
                "projects",
                "remove",
                "proj",
                "--confirm",
            ],
            [
                "hoop",
                "projects",
                "--no-interactive",
                "remove",
                "proj",
                "--confirm",
            ],
            [
                "hoop",
                "projects",
                "remove",
                "proj",
                "--no-interactive",
                "--confirm",
            ],
            [
                "hoop",
                "projects",
                "remove",
                "proj",
                "--confirm",
                "--no-interactive",
            ],
        ];

        for args in test_cases {
            let cli = parse_args(&args).expect("should parse successfully with any flag ordering");
            assert!(
                cli.no_interactive,
                "no_interactive should be true regardless of flag order"
            );

            match &cli.command {
                Commands::Projects(ProjectsCommands::Remove { name, confirm }) => {
                    assert_eq!(name, "proj");
                    assert!(*confirm);
                }
                _ => panic!("expected Projects::Remove command"),
            }
        }
    }

    #[test]
    fn test_remove_default_no_interactive_value() {
        let args = ["hoop", "remove", "my-project", "--confirm"];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            !cli.no_interactive,
            "no_interactive should default to false"
        );
    }

    #[test]
    fn test_projects_remove_default_no_interactive_value() {
        let args = ["hoop", "projects", "remove", "my-project", "--confirm"];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            !cli.no_interactive,
            "no_interactive should default to false for projects remove"
        );
    }

    // ── Restore command tests ───────────────────────────────────────────────────

    #[test]
    fn test_restore_no_interactive_flag_before_command() {
        let args = [
            "hoop",
            "--no-interactive",
            "restore",
            "--from",
            "s3://bucket/path/snap",
            "--confirm",
        ];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            cli.no_interactive,
            "no_interactive should be true when flag appears before restore command"
        );

        match cli.command {
            Commands::Restore {
                from,
                dry_run,
                confirm,
            } => {
                assert_eq!(from, "s3://bucket/path/snap");
                assert!(!dry_run);
                assert!(confirm);
            }
            _ => panic!("expected Restore command"),
        }
    }

    #[test]
    fn test_restore_no_interactive_flag_after_command() {
        let args = [
            "hoop",
            "restore",
            "--from",
            "s3://bucket/path/snap",
            "--no-interactive",
            "--confirm",
        ];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            cli.no_interactive,
            "no_interactive should be true when flag appears after restore command"
        );

        match cli.command {
            Commands::Restore {
                from,
                dry_run,
                confirm,
            } => {
                assert_eq!(from, "s3://bucket/path/snap");
                assert!(!dry_run);
                assert!(confirm);
            }
            _ => panic!("expected Restore command"),
        }
    }

    #[test]
    fn test_restore_short_flag_y_before_command() {
        let args = [
            "hoop",
            "-y",
            "restore",
            "--from",
            "s3://bucket/path/snap",
            "--confirm",
        ];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            cli.no_interactive,
            "no_interactive should be true with -y short flag"
        );

        match cli.command {
            Commands::Restore {
                from,
                dry_run,
                confirm,
            } => {
                assert_eq!(from, "s3://bucket/path/snap");
                assert!(!dry_run);
                assert!(confirm);
            }
            _ => panic!("expected Restore command"),
        }
    }

    #[test]
    fn test_restore_short_flag_y_after_command() {
        let args = [
            "hoop",
            "restore",
            "-y",
            "--from",
            "s3://bucket/path/snap",
            "--confirm",
        ];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            cli.no_interactive,
            "no_interactive should be true with -y short flag after restore command"
        );

        match cli.command {
            Commands::Restore {
                from,
                dry_run,
                confirm,
            } => {
                assert_eq!(from, "s3://bucket/path/snap");
                assert!(!dry_run);
                assert!(confirm);
            }
            _ => panic!("expected Restore command"),
        }
    }

    #[test]
    fn test_restore_without_no_interactive_flag_is_false() {
        let args = [
            "hoop",
            "restore",
            "--from",
            "s3://bucket/path/snap",
            "--confirm",
        ];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            !cli.no_interactive,
            "no_interactive should be false when flag is not provided"
        );

        match cli.command {
            Commands::Restore {
                from,
                dry_run,
                confirm,
            } => {
                assert_eq!(from, "s3://bucket/path/snap");
                assert!(!dry_run);
                assert!(confirm);
            }
            _ => panic!("expected Restore command"),
        }
    }

    #[test]
    fn test_restore_no_interactive_flag_extraction_consistency() {
        // Test that flag parsing is consistent regardless of position
        let args_before = [
            "hoop",
            "--no-interactive",
            "restore",
            "--from",
            "s3://bucket/snap",
            "--confirm",
        ];
        let args_after = [
            "hoop",
            "restore",
            "--from",
            "s3://bucket/snap",
            "--no-interactive",
            "--confirm",
        ];

        let cli_before = parse_args(&args_before).expect("should parse successfully");
        let cli_after = parse_args(&args_after).expect("should parse successfully");

        assert_eq!(
            cli_before.no_interactive, cli_after.no_interactive,
            "no_interactive value must be consistent regardless of flag position"
        );
        assert!(cli_before.no_interactive, "no_interactive should be true");
    }

    #[test]
    fn test_restore_command_with_all_flags() {
        let args = [
            "hoop",
            "--no-interactive",
            "restore",
            "--from",
            "s3://bucket/path/snap",
            "--dry-run",
            "--confirm",
        ];
        let cli = parse_args(&args).expect("should parse successfully");

        assert!(cli.no_interactive);

        match cli.command {
            Commands::Restore {
                from,
                dry_run,
                confirm,
            } => {
                assert_eq!(from, "s3://bucket/path/snap");
                assert!(dry_run);
                assert!(confirm);
            }
            _ => panic!("expected Restore command"),
        }
    }

    #[test]
    fn test_restore_with_dry_run_flag() {
        let args = [
            "hoop",
            "restore",
            "--from",
            "s3://bucket/path/snap",
            "--dry-run",
        ];
        let cli = parse_args(&args).expect("should parse successfully");

        assert!(
            !cli.no_interactive,
            "no_interactive should be false when not provided"
        );

        match cli.command {
            Commands::Restore {
                from,
                dry_run,
                confirm,
            } => {
                assert_eq!(from, "s3://bucket/path/snap");
                assert!(dry_run);
                assert!(!confirm);
            }
            _ => panic!("expected Restore command"),
        }
    }

    #[test]
    fn test_restore_command_flag_order_independence() {
        // Test different flag orderings
        let test_cases = vec![
            [
                "hoop",
                "--no-interactive",
                "restore",
                "--from",
                "s3://b/s",
                "--confirm",
            ],
            [
                "hoop",
                "restore",
                "--no-interactive",
                "--from",
                "s3://b/s",
                "--confirm",
            ],
            [
                "hoop",
                "restore",
                "--from",
                "s3://b/s",
                "--no-interactive",
                "--confirm",
            ],
            [
                "hoop",
                "restore",
                "--from",
                "s3://b/s",
                "--confirm",
                "--no-interactive",
            ],
        ];

        for args in test_cases {
            let cli = parse_args(&args).expect("should parse successfully with any flag ordering");
            assert!(
                cli.no_interactive,
                "no_interactive should be true regardless of flag order"
            );

            match &cli.command {
                Commands::Restore { from, confirm, .. } => {
                    assert_eq!(*from, "s3://b/s");
                    assert!(*confirm);
                }
                _ => panic!("expected Restore command"),
            }
        }
    }

    #[test]
    fn test_restore_default_no_interactive_value() {
        let args = [
            "hoop",
            "restore",
            "--from",
            "s3://bucket/path/snap",
            "--confirm",
        ];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            !cli.no_interactive,
            "no_interactive should default to false"
        );
    }

    #[test]
    fn test_restore_command_requires_from_flag() {
        let args = ["hoop", "--no-interactive", "restore", "--confirm"];
        let result = parse_args(&args);
        assert!(result.is_err(), "should fail when --from flag is missing");
    }

    #[test]
    fn test_restore_command_parsing_with_invalid_flag() {
        let args = [
            "hoop",
            "--no-interactive",
            "restore",
            "--from",
            "s3://bucket/snap",
            "--invalid-flag",
        ];
        let result = parse_args(&args);
        assert!(result.is_err(), "should fail with invalid flag");
    }

    #[test]
    fn test_restore_with_complex_s3_uri() {
        let s3_uri = "s3://my-bucket/backups/snapshot-2024-01-15T10:30:00Z";
        let args = [
            "hoop",
            "--no-interactive",
            "restore",
            "--from",
            s3_uri,
            "--confirm",
        ];
        let cli = parse_args(&args).expect("should parse successfully");

        match cli.command {
            Commands::Restore { from, .. } => {
                assert_eq!(from, s3_uri, "complex S3 URI should be parsed correctly");
            }
            _ => panic!("expected Restore command"),
        }
    }

    // ── Init command tests ─────────────────────────────────────────────────────

    #[test]
    fn test_init_no_interactive_flag_before_command() {
        let args = ["hoop", "--no-interactive", "init"];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            cli.no_interactive,
            "no_interactive should be true when flag appears before init"
        );

        match cli.command {
            Commands::Init => {
                // Success - command parsed correctly
            }
            _ => panic!("expected Init command, got {:?}", cli.command),
        }
    }

    #[test]
    fn test_init_no_interactive_flag_after_command() {
        let args = ["hoop", "init", "--no-interactive"];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            cli.no_interactive,
            "no_interactive should be true when flag appears after init"
        );

        match cli.command {
            Commands::Init => {
                // Success - command parsed correctly
            }
            _ => panic!("expected Init command, got {:?}", cli.command),
        }
    }

    #[test]
    fn test_init_short_flag_y_before_command() {
        let args = ["hoop", "-y", "init"];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            cli.no_interactive,
            "no_interactive should be true with -y short flag"
        );

        match cli.command {
            Commands::Init => {
                // Success - command parsed correctly
            }
            _ => panic!("expected Init command"),
        }
    }

    #[test]
    fn test_init_short_flag_y_after_command() {
        let args = ["hoop", "init", "-y"];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            cli.no_interactive,
            "no_interactive should be true with -y short flag after init"
        );

        match cli.command {
            Commands::Init => {
                // Success - command parsed correctly
            }
            _ => panic!("expected Init command"),
        }
    }

    #[test]
    fn test_init_without_no_interactive_flag_is_false() {
        let args = ["hoop", "init"];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            !cli.no_interactive,
            "no_interactive should be false when flag is not provided"
        );

        match cli.command {
            Commands::Init => {
                // Success - command parsed correctly
            }
            _ => panic!("expected Init command"),
        }
    }

    #[test]
    fn test_init_no_interactive_flag_extraction_consistency() {
        // Test that flag parsing is consistent regardless of position
        let args_before = ["hoop", "--no-interactive", "init"];
        let args_after = ["hoop", "init", "--no-interactive"];

        let cli_before = parse_args(&args_before).expect("should parse successfully");
        let cli_after = parse_args(&args_after).expect("should parse successfully");

        assert_eq!(
            cli_before.no_interactive, cli_after.no_interactive,
            "no_interactive value must be consistent regardless of flag position"
        );
        assert!(cli_before.no_interactive, "no_interactive should be true");
    }

    #[test]
    fn test_init_command_parsing_with_invalid_flag() {
        let args = ["hoop", "--no-interactive", "init", "--invalid-flag"];
        let result = parse_args(&args);
        assert!(result.is_err(), "should fail with invalid flag");
    }

    #[test]
    fn test_init_command_flag_order_independence() {
        // Test different flag orderings
        let test_cases = vec![["hoop", "--no-interactive", "init"], ["hoop", "-y", "init"]];

        for args in test_cases {
            let cli = parse_args(&args).expect("should parse successfully with any flag ordering");
            assert!(
                cli.no_interactive,
                "no_interactive should be true regardless of flag order"
            );

            match cli.command {
                Commands::Init => {
                    // Success
                }
                _ => panic!("expected Init command"),
            }
        }
    }

    #[test]
    fn test_init_default_no_interactive_value() {
        let args = ["hoop", "init"];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            !cli.no_interactive,
            "no_interactive should default to false"
        );
    }

    #[test]
    fn test_init_command_takes_no_arguments() {
        let args = ["hoop", "init", "extra-argument"];
        let result = parse_args(&args);
        assert!(
            result.is_err(),
            "should fail when extra arguments are provided"
        );
    }

    // ── Helper function tests for Init command parsing ─────────────────────────

    /// Test helper function to verify Init command parsing
    #[test]
    fn test_parse_init_command_helper() {
        /// Helper function to parse Init command and extract both flag and command
        fn parse_init_command(args: &[&str]) -> Result<(bool, Commands), clap::Error> {
            let cli = Cli::try_parse_from(args)?;
            Ok((cli.no_interactive, cli.command))
        }

        // Test 1: Flag before command
        let (no_interactive, command) = parse_init_command(&["hoop", "--no-interactive", "init"])
            .expect("should parse successfully");
        assert!(no_interactive);
        assert!(matches!(command, Commands::Init));

        // Test 2: Flag after command
        let (no_interactive, command) = parse_init_command(&["hoop", "init", "--no-interactive"])
            .expect("should parse successfully");
        assert!(no_interactive);
        assert!(matches!(command, Commands::Init));

        // Test 3: No flag
        let (no_interactive, command) =
            parse_init_command(&["hoop", "init"]).expect("should parse successfully");
        assert!(!no_interactive);
        assert!(matches!(command, Commands::Init));
    }

    /// Test helper function for building test arguments
    #[test]
    fn test_build_init_args_helper() {
        /// Helper function to build Init command arguments with optional flags
        fn build_init_args(no_interactive: bool) -> Vec<&'static str> {
            let mut args = vec!["hoop"];
            if no_interactive {
                args.push("--no-interactive");
            }
            args.push("init");
            args
        }

        // Test with no_interactive=true
        let args_with_flag = build_init_args(true);
        let cli = parse_args(&args_with_flag).expect("should parse successfully");
        assert!(cli.no_interactive);
        assert!(matches!(cli.command, Commands::Init));

        // Test with no_interactive=false
        let args_without_flag = build_init_args(false);
        let cli = parse_args(&args_without_flag).expect("should parse successfully");
        assert!(!cli.no_interactive);
        assert!(matches!(cli.command, Commands::Init));
    }

    // ── Scan command tests ───────────────────────────────────────────────────────

    #[test]
    fn test_scan_no_interactive_flag_before_command() {
        let args = ["hoop", "--no-interactive", "scan", "/tmp/projects"];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            cli.no_interactive,
            "no_interactive should be true when flag appears before scan command"
        );

        match cli.command {
            Commands::Scan { root, auto_confirm } => {
                assert_eq!(root, "/tmp/projects");
                assert!(!auto_confirm);
            }
            _ => panic!("expected Scan command, got {:?}", cli.command),
        }
    }

    #[test]
    fn test_scan_no_interactive_flag_after_command() {
        let args = ["hoop", "scan", "/tmp/projects", "--no-interactive"];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            cli.no_interactive,
            "no_interactive should be true when flag appears after scan command"
        );

        match cli.command {
            Commands::Scan { root, auto_confirm } => {
                assert_eq!(root, "/tmp/projects");
                assert!(!auto_confirm);
            }
            _ => panic!("expected Scan command, got {:?}", cli.command),
        }
    }

    #[test]
    fn test_scan_short_flag_y_before_command() {
        let args = ["hoop", "-y", "scan", "/tmp/projects"];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            cli.no_interactive,
            "no_interactive should be true with -y short flag"
        );

        match cli.command {
            Commands::Scan { root, auto_confirm } => {
                assert_eq!(root, "/tmp/projects");
                assert!(!auto_confirm);
            }
            _ => panic!("expected Scan command"),
        }
    }

    #[test]
    fn test_scan_short_flag_y_after_command() {
        let args = ["hoop", "scan", "/tmp/projects", "-y"];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            cli.no_interactive,
            "no_interactive should be true with -y short flag after scan command"
        );

        match cli.command {
            Commands::Scan { root, auto_confirm } => {
                assert_eq!(root, "/tmp/projects");
                assert!(!auto_confirm);
            }
            _ => panic!("expected Scan command"),
        }
    }

    #[test]
    fn test_scan_without_no_interactive_flag_is_false() {
        let args = ["hoop", "scan", "/tmp/projects"];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            !cli.no_interactive,
            "no_interactive should be false when flag is not provided"
        );

        match cli.command {
            Commands::Scan { root, auto_confirm } => {
                assert_eq!(root, "/tmp/projects");
                assert!(!auto_confirm);
            }
            _ => panic!("expected Scan command"),
        }
    }

    #[test]
    fn test_scan_with_local_yes_flag() {
        let args = ["hoop", "scan", "/tmp/projects", "--yes"];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            !cli.no_interactive,
            "Global no_interactive should be false with local --yes"
        );

        match cli.command {
            Commands::Scan { root, auto_confirm } => {
                assert_eq!(root, "/tmp/projects");
                assert!(
                    auto_confirm,
                    "local --yes flag should set auto_confirm to true"
                );
            }
            _ => panic!("expected Scan command"),
        }
    }

    #[test]
    fn test_scan_with_both_global_and_local_flags() {
        let args = ["hoop", "--no-interactive", "scan", "/tmp/projects", "--yes"];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(cli.no_interactive, "Global no_interactive should be true");

        match cli.command {
            Commands::Scan { root, auto_confirm } => {
                assert_eq!(root, "/tmp/projects");
                assert!(auto_confirm, "local --yes flag should be true");
            }
            _ => panic!("expected Scan command"),
        }
    }

    #[test]
    fn test_scan_no_interactive_flag_extraction_consistency() {
        // Test that flag parsing is consistent regardless of position
        let args_before = ["hoop", "--no-interactive", "scan", "/tmp/projects"];
        let args_after = ["hoop", "scan", "/tmp/projects", "--no-interactive"];

        let cli_before = parse_args(&args_before).expect("should parse successfully");
        let cli_after = parse_args(&args_after).expect("should parse successfully");

        assert_eq!(
            cli_before.no_interactive, cli_after.no_interactive,
            "no_interactive value must be consistent regardless of flag position"
        );
        assert!(cli_before.no_interactive, "no_interactive should be true");
    }

    #[test]
    fn test_scan_command_flag_order_independence() {
        // Test different flag orderings
        let test_cases = vec![
            ["hoop", "--no-interactive", "scan", "/tmp/projects"],
            ["hoop", "scan", "/tmp/projects", "--no-interactive"],
            ["hoop", "-y", "scan", "/tmp/projects"],
            ["hoop", "scan", "/tmp/projects", "-y"],
        ];

        for args in test_cases {
            let cli = parse_args(&args).expect("should parse successfully with any flag ordering");
            assert!(
                cli.no_interactive,
                "no_interactive should be true regardless of flag order"
            );

            match &cli.command {
                Commands::Scan { root, .. } => {
                    assert_eq!(*root, "/tmp/projects");
                }
                _ => panic!("expected Scan command"),
            }
        }
    }

    #[test]
    fn test_scan_default_no_interactive_value() {
        let args = ["hoop", "scan", "/tmp/projects"];
        let cli = parse_args(&args).expect("should parse successfully");
        assert!(
            !cli.no_interactive,
            "no_interactive should default to false"
        );
    }

    #[test]
    fn test_scan_command_requires_root_argument() {
        let args = ["hoop", "--no-interactive", "scan"];
        let result = parse_args(&args);
        assert!(result.is_err(), "should fail when root argument is missing");
    }

    #[test]
    fn test_scan_command_parsing_with_invalid_flag() {
        let args = [
            "hoop",
            "--no-interactive",
            "scan",
            "/tmp/projects",
            "--invalid-flag",
        ];
        let result = parse_args(&args);
        assert!(result.is_err(), "should fail with invalid flag");
    }

    #[test]
    fn test_scan_with_complex_root_path() {
        let root_path = "/var/data/projects/2024/January";
        let args = ["hoop", "--no-interactive", "scan", root_path];
        let cli = parse_args(&args).expect("should parse successfully");

        match cli.command {
            Commands::Scan { root, .. } => {
                assert_eq!(
                    root, root_path,
                    "complex root path should be parsed correctly"
                );
            }
            _ => panic!("expected Scan command"),
        }
    }

    #[test]
    fn test_scan_no_interactive_or_yes_combination_logic() {
        // This test documents the expected behavior: both flags should work independently
        // The actual combination logic (no_interactive || auto_confirm) is tested in integration tests

        // Case 1: Only global flag
        let cli_global = parse_args(&["hoop", "--no-interactive", "scan", "/tmp"])
            .expect("should parse with global flag only");
        assert!(cli_global.no_interactive);
        if let Commands::Scan { auto_confirm, .. } = cli_global.command {
            assert!(
                !auto_confirm,
                "local auto_confirm should be false with only global flag"
            );
        } else {
            panic!("Expected Scan command");
        }

        // Case 2: Only local flag
        let cli_local = parse_args(&["hoop", "scan", "/tmp", "--yes"])
            .expect("should parse with local flag only");
        assert!(!cli_local.no_interactive);
        if let Commands::Scan { auto_confirm, .. } = cli_local.command {
            assert!(
                auto_confirm,
                "local auto_confirm should be true with --yes flag"
            );
        } else {
            panic!("Expected Scan command");
        }

        // Case 3: Both flags
        let cli_both = parse_args(&["hoop", "--no-interactive", "scan", "/tmp", "--yes"])
            .expect("should parse with both flags");
        assert!(cli_both.no_interactive);
        if let Commands::Scan { auto_confirm, .. } = cli_both.command {
            assert!(
                auto_confirm,
                "local auto_confirm should be true with --yes flag"
            );
        } else {
            panic!("Expected Scan command");
        }
    }
}
