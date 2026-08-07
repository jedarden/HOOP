//! Clap-based test utilities for CLI testing, including no_interactive flag testing
//!
//! This module provides reusable helpers for testing CLI commands using actual clap parsing.
//! Unlike `cli_test_utils.rs` (which uses custom string parsing), this module uses the real
//! clap parser from the CLI, ensuring tests match actual runtime behavior.
//!
//! # What This Module Provides
//!
//! - **Real clap parsing**: Uses `Cli::try_parse_from()` to test actual CLI behavior
//! - **Flag position testing**: Test `--no-interactive` at any position in the command line
//! - **Command extraction**: Access parsed commands and their arguments
//! - **Verification helpers**: Ensure flags are correctly extracted and propagated
//! - **Test macros**: Generate common test patterns for any command
//!
//! # Example Usage
//!
//! ```rust
//! use clap_test_utils::*;
//!
//! #[test]
//! fn test_scan_no_interactive() {
//!     // Parse with flag before subcommand
//!     let cli = parse_cli(&["hoop", "--no-interactive", "scan", "/tmp"]).unwrap();
//!     assert_eq!(cli.no_interactive, true);
//!
//!     // Parse with flag after subcommand
//!     let cli = parse_cli(&["hoop", "scan", "/tmp", "--no-interactive"]).unwrap();
//!     assert_eq!(cli.no_interactive, true);
//!
//!     // Verify both positions yield the same value
//!     assert!(parse_both_positions_yield_same_value(
//!         &["scan", "/tmp"],
//!         &["--no-interactive"]
//!     ));
//! }
//! ```
//!
//! # Testing Philosophy
//!
//! 1. **Test real behavior**: Use actual clap parsing, not mock parsers
//! 2. **Position independence**: Verify flags work in any position
//! 3. **Flag propagation**: Ensure global flags are accessible to all subcommands
//! 4. **Consistency**: Same flag value regardless of position

use std::path::PathBuf;

// ── Import the actual CLI structure from lib ─────────────────────────────────────
//
// The CLI types (Cli, Commands, etc.) are now defined in src/cli.rs and
// re-exported through lib.rs, making them accessible to tests in the tests/
// directory.

pub use hoop_cli::{AuditCommands, Cli, Commands, ProjectsCommands};

// ── Core parsing helpers ───────────────────────────────────────────────────────────

/// Parse CLI arguments using actual clap parsing
///
/// This function uses `Cli::try_parse_from()` to parse command-line arguments
/// exactly as they would be parsed at runtime. This ensures tests match actual
/// CLI behavior.
///
/// # Arguments
///
/// * `args` - Command line arguments (including program name)
///
/// # Returns
///
/// * `Result<Cli, clap::Error>` - Parsed CLI structure or clap error
///
/// # Examples
///
/// ```rust
/// let cli = parse_cli(&["hoop", "--no-interactive", "scan", "/tmp"])?;
/// assert_eq!(cli.no_interactive, true);
/// ```
pub fn parse_cli(args: &[&str]) -> Result<Cli, clap::Error> {
    Cli::try_parse_from(args)
}

/// Parse CLI and extract the no_interactive flag value
///
/// Convenience function that parses CLI arguments and returns just the
/// no_interactive flag value.
///
/// # Examples
///
/// ```rust
/// let no_interactive = parse_no_interactive_flag(&["hoop", "--no-interactive", "scan"])?;
/// assert_eq!(no_interactive, true);
/// ```
pub fn parse_no_interactive_flag(args: &[&str]) -> Result<bool, clap::Error> {
    let cli = parse_cli(args)?;
    Ok(cli.no_interactive)
}

/// Test that flag parsing works in both positions for a command
///
/// Parses the command with the flag before and after the subcommand,
/// returning both values for comparison.
///
/// # Arguments
///
/// * `flag_args` - Flag arguments (e.g., `["--no-interactive"]`)
/// * `cmd_args` - Command arguments (e.g., `["scan", "/tmp"]`)
///
/// # Returns
///
/// * `(bool, bool)` - (no_interactive before, no_interactive after)
///
/// # Examples
///
/// ```rust
/// let (before, after) = parse_both_positions(
///     &["--no-interactive"],
///     &["scan", "/tmp"]
/// );
/// assert_eq!(before, after);
/// ```
pub fn parse_both_positions(flag_args: &[&str], cmd_args: &[&str]) -> (bool, bool) {
    // Parse with flag before subcommand: hoop --no-interactive scan /tmp
    let full_args_before: Vec<&str> = ["hoop"]
        .iter()
        .chain(flag_args.iter())
        .chain(cmd_args.iter())
        .copied()
        .collect();
    let cli_before = parse_cli(&full_args_before).unwrap();
    let no_interactive_before = cli_before.no_interactive;

    // Parse with flag after subcommand: hoop scan /tmp --no-interactive
    let full_args_after: Vec<&str> = ["hoop"]
        .iter()
        .chain(cmd_args.iter())
        .chain(flag_args.iter())
        .copied()
        .collect();
    let cli_after = parse_cli(&full_args_after).unwrap();
    let no_interactive_after = cli_after.no_interactive;

    (no_interactive_before, no_interactive_after)
}

/// Verify that both flag positions yield the same no_interactive value
///
/// This is a common test pattern: commands should behave identically
/// regardless of where the global flag appears on the command line.
///
/// # Arguments
///
/// * `cmd_args` - Command arguments (e.g., `["scan", "/tmp"]`)
/// * `flag_args` - Flag arguments (default: `["--no-interactive"]`)
///
/// # Returns
///
/// * `bool` - true if both positions yield the same value
///
/// # Examples
///
/// ```rust
/// assert!(parse_both_positions_yield_same_value(
///     &["scan", "/tmp"],
///     &["--no-interactive"]
/// ));
/// ```
pub fn parse_both_positions_yield_same_value(cmd_args: &[&str], flag_args: &[&str]) -> bool {
    let (before, after) = parse_both_positions(flag_args, cmd_args);
    before == after
}

/// Parse with flag before subcommand (e.g., `hoop --no-interactive scan /tmp`)
///
/// Convenience function that constructs the full argument list with
/// the flag before the command.
///
/// # Arguments
///
/// * `cmd_args` - Command arguments without program name or flag
///
/// # Returns
///
/// * `Result<Cli, clap::Error>` - Parsed CLI or error
pub fn parse_flag_before_subcommand(cmd_args: &[&str]) -> Result<Cli, clap::Error> {
    let full_args: Vec<&str> = ["hoop", "--no-interactive"]
        .iter()
        .chain(cmd_args.iter())
        .copied()
        .collect();
    parse_cli(&full_args)
}

/// Parse with flag after subcommand (e.g., `hoop scan /tmp --no-interactive`)
///
/// Convenience function that constructs the full argument list with
/// the flag after the command.
///
/// # Arguments
///
/// * `cmd_args` - Command arguments without program name or flag
///
/// # Returns
///
/// * `Result<Cli, clap::Error>` - Parsed CLI or error
pub fn parse_flag_after_subcommand(cmd_args: &[&str]) -> Result<Cli, clap::Error> {
    let mut full_args: Vec<&str> = ["hoop"].iter().chain(cmd_args.iter()).copied().collect();
    full_args.push("--no-interactive");
    parse_cli(&full_args)
}

/// Parse with short flag -y (e.g., `hoop -y scan /tmp`)
///
/// Convenience function that constructs the full argument list with
/// the short -y flag.
///
/// # Arguments
///
/// * `cmd_args` - Command arguments without program name or flag
///
/// # Returns
///
/// * `Result<Cli, clap::Error>` - Parsed CLI or error
pub fn parse_with_short_flag(cmd_args: &[&str]) -> Result<Cli, clap::Error> {
    let full_args: Vec<&str> = ["hoop", "-y"]
        .iter()
        .chain(cmd_args.iter())
        .copied()
        .collect();
    parse_cli(&full_args)
}

// ── Verification helpers ───────────────────────────────────────────────────────────

/// Verify that no_interactive is correctly extracted from parsed CLI
///
/// Checks that the parsed CLI has the expected no_interactive value.
///
/// # Arguments
///
/// * `cli` - Parsed CLI structure
/// * `expected` - Expected value of no_interactive
///
/// # Returns
///
/// * `Result<(), String>` - Ok if verification passes, Err with details
pub fn verify_no_interactive_value(cli: &Cli, expected: bool) -> Result<(), String> {
    if cli.no_interactive != expected {
        return Err(format!(
            "Expected no_interactive={}, got {}",
            expected, cli.no_interactive
        ));
    }
    Ok(())
}

/// Verify flag extraction at a specific position
///
/// Checks that the flag was correctly parsed when placed at a specific
/// position in the command line.
///
/// # Arguments
///
/// * `cmd_args` - Command arguments
/// * `position` - Expected position ("before" or "after")
/// * `expected_value` - Expected no_interactive value
///
/// # Returns
///
/// * `Result<(), String>` - Ok if verification passes
pub fn verify_flag_at_position(
    cmd_args: &[&str],
    position: &str,
    expected_value: bool,
) -> Result<(), String> {
    let cli = match position {
        "before" => parse_flag_before_subcommand(cmd_args),
        "after" => parse_flag_after_subcommand(cmd_args),
        _ => return Err(format!("Invalid position: {}", position)),
    }
    .map_err(|e| format!("Parse error: {}", e))?;

    verify_no_interactive_value(&cli, expected_value)
}

/// Verify that both positions yield the same value
///
/// Tests the position independence property: the flag value should
/// be the same regardless of where it appears in the command line.
///
/// # Arguments
///
/// * `cmd_args` - Command arguments
///
/// # Returns
///
/// * `Result<(), String>` - Ok if both positions yield same value
pub fn verify_position_independence(cmd_args: &[&str]) -> Result<(), String> {
    let (before, after) = parse_both_positions(&["--no-interactive"], cmd_args);

    if before != after {
        return Err(format!(
            "Position dependence detected: before={}, after={}",
            before, after
        ));
    }

    Ok(())
}

/// Verify that the flag defaults to false when not specified
///
/// Tests that the default behavior is interactive (no_interactive=false).
///
/// # Arguments
///
/// * `cmd_args` - Command arguments without flag
///
/// # Returns
///
/// * `Result<(), String>` - Ok if flag defaults to false
pub fn verify_flag_default_false(cmd_args: &[&str]) -> Result<(), String> {
    let mut full_args: Vec<&str> = ["hoop"].iter().chain(cmd_args.iter()).copied().collect();
    let cli = parse_cli(&full_args).map_err(|e| format!("Parse error: {}", e))?;

    verify_no_interactive_value(&cli, false)
}

// ── Command extraction helpers ─────────────────────────────────────────────────────

/// Extract the command variant from parsed CLI
///
/// Returns the command enum variant from the parsed CLI for inspection.
///
/// # Arguments
///
/// * `cli` - Parsed CLI structure
///
/// # Returns
///
/// * `&Commands` - Reference to the command variant
pub fn get_command(cli: &Cli) -> &Commands {
    &cli.command
}

/// Match on a specific command variant
///
/// Helper function that attempts to extract a specific command variant
/// from the parsed CLI. Returns None if the CLI parsed a different command.
///
/// # Examples
///
/// ```rust
/// let cli = parse_cli(&["hoop", "scan", "/tmp"])?;
/// if let Some(Commands::Scan { root, .. }) = try_get_scan_command(&cli) {
///     assert_eq!(root, "/tmp");
/// }
/// ```
pub fn try_get_scan_command(cli: &Cli) -> Option<&Commands> {
    match &cli.command {
        cmd @ Commands::Scan { .. } => Some(cmd),
        _ => None,
    }
}

pub fn try_get_remove_command(cli: &Cli) -> Option<&Commands> {
    match &cli.command {
        cmd @ Commands::Remove { .. } => Some(cmd),
        _ => None,
    }
}

pub fn try_get_restore_command(cli: &Cli) -> Option<&Commands> {
    match &cli.command {
        cmd @ Commands::Restore { .. } => Some(cmd),
        _ => None,
    }
}

pub fn try_get_init_command(cli: &Cli) -> Option<&Commands> {
    match &cli.command {
        Commands::Init => Some(&cli.command),
        _ => None,
    }
}

// ── Test macros for common patterns ───────────────────────────────────────────────

/// Macro to generate a test for flag parsing before a command
///
/// # Usage
///
/// ```rust
/// test_flag_before!(scan_flag_before, &["scan", "/tmp"]);
/// ```
#[macro_export]
macro_rules! test_flag_before {
    ($test_name:ident, $cmd_args:expr) => {
        #[test]
        fn $test_name() {
            let cli = $crate::clap_test_utils::parse_flag_before_subcommand($cmd_args)
                .expect("Should parse with flag before command");
            assert_eq!(
                cli.no_interactive, true,
                "no_interactive should be true with flag before command"
            );
        }
    };
}

/// Macro to generate a test for flag parsing after a command
///
/// # Usage
///
/// ```rust
/// test_flag_after!(scan_flag_after, &["scan", "/tmp"]);
/// ```
#[macro_export]
macro_rules! test_flag_after {
    ($test_name:ident, $cmd_args:expr) => {
        #[test]
        fn $test_name() {
            let cli = $crate::clap_test_utils::parse_flag_after_subcommand($cmd_args)
                .expect("Should parse with flag after command");
            assert_eq!(
                cli.no_interactive, true,
                "no_interactive should be true with flag after command"
            );
        }
    };
}

/// Macro to generate a test for short flag -y
///
/// # Usage
///
/// ```rust
/// test_short_flag!(scan_short_y, &["scan", "/tmp"]);
/// ```
#[macro_export]
macro_rules! test_short_flag {
    ($test_name:ident, $cmd_args:expr) => {
        #[test]
        fn $test_name() {
            let cli = $crate::clap_test_utils::parse_with_short_flag($cmd_args)
                .expect("Should parse with -y flag");
            assert_eq!(
                cli.no_interactive, true,
                "no_interactive should be true with -y flag"
            );
        }
    };
}

/// Macro to generate a test for position independence
///
/// # Usage
///
/// ```rust
/// test_position_independence!(scan_consistency, &["scan", "/tmp"]);
/// ```
#[macro_export]
macro_rules! test_position_independence {
    ($test_name:ident, $cmd_args:expr) => {
        #[test]
        fn $test_name() {
            assert!(
                $crate::clap_test_utils::parse_both_positions_yield_same_value(
                    $cmd_args,
                    &["--no-interactive"]
                ),
                "Flag value should be consistent regardless of position"
            );
        }
    };
}

/// Macro to generate a test for default flag value (false)
///
/// # Usage
///
/// ```rust
/// test_flag_default!(scan_default, &["scan", "/tmp"]);
/// ```
#[macro_export]
macro_rules! test_flag_default {
    ($test_name:ident, $cmd_args:expr) => {
        #[test]
        fn $test_name() {
            let mut full_args: Vec<&str> = ["hoop"].iter().chain($cmd_args.iter()).copied().collect();
            let cli = $crate::clap_test_utils::parse_cli(&full_args)
                .expect("Should parse without flag");
            assert_eq!(
                cli.no_interactive, false,
                "no_interactive should default to false"
            );
        }
    };
}

/// Macro to generate a complete test suite for a command
///
/// Generates 5 tests covering all flag positions and default behavior:
/// - Flag before command
/// - Flag after command
/// - Short flag -y
/// - Position independence
/// - Default (no flag)
///
/// # Usage
///
/// ```rust
/// test_command_no_interactive_suite!(scan, &["scan", "/tmp"]);
/// test_command_no_interactive_suite!(remove, &["remove", "my-project", "--confirm"]);
/// test_command_no_interactive_suite!(init, &["init"]);
/// ```
#[macro_export]
macro_rules! test_command_no_interactive_suite {
    ($prefix:ident, $cmd_args:expr) => {
        // Test 1: Flag before command
        #[test]
        fn concat_idents!($prefix, _flag_before_command)() {
            let cli = $crate::clap_test_utils::parse_flag_before_subcommand($cmd_args)
                .expect("Should parse with flag before command");
            assert_eq!(cli.no_interactive, true);
        }

        // Test 2: Flag after command
        #[test]
        fn concat_idents!($prefix, _flag_after_command)() {
            let cli = $crate::clap_test_utils::parse_flag_after_subcommand($cmd_args)
                .expect("Should parse with flag after command");
            assert_eq!(cli.no_interactive, true);
        }

        // Test 3: Short flag -y
        #[test]
        fn concat_idents!($prefix, _short_flag_y)() {
            let cli = $crate::clap_test_utils::parse_with_short_flag($cmd_args)
                .expect("Should parse with -y flag");
            assert_eq!(cli.no_interactive, true);
        }

        // Test 4: Position independence
        #[test]
        fn concat_idents!($prefix, _position_independence)() {
            assert!(
                $crate::clap_test_utils::parse_both_positions_yield_same_value(
                    $cmd_args,
                    &["--no-interactive"]
                ),
                "Flag value should be consistent regardless of position"
            );
        }

        // Test 5: Default (no flag)
        #[test]
        fn concat_idents!($prefix, _flag_default_false)() {
            let mut full_args: Vec<&str> = ["hoop"].iter().chain($cmd_args.iter()).copied().collect();
            let cli = $crate::clap_test_utils::parse_cli(&full_args)
                .expect("Should parse without flag");
            assert_eq!(cli.no_interactive, false);
        }
    };
}

// ── Integration test helpers ─────────────────────────────────────────────────────

/// Test case structure for batch testing
#[derive(Debug, Clone)]
pub struct ClapTestCase {
    pub description: String,
    pub args: Vec<String>,
    pub expected_no_interactive: bool,
    pub should_parse: bool,
}

/// Run a batch of clap parsing tests
///
/// Executes multiple test cases and returns success/failure summaries.
///
/// # Arguments
///
/// * `test_cases` - Vector of test cases to run
///
/// # Returns
///
/// * (Vec<String>, Vec<(String, String)>) - (successes, (description, error))
pub fn run_clap_tests(test_cases: Vec<ClapTestCase>) -> (Vec<String>, Vec<(String, String)>) {
    let mut successes = Vec::new();
    let mut failures = Vec::new();

    for test_case in test_cases {
        let args: Vec<&str> = test_case.args.iter().map(|s| s.as_str()).collect();

        match parse_cli(&args) {
            Ok(cli) => {
                if !test_case.should_parse {
                    failures.push((
                        test_case.description,
                        "Expected parse failure but succeeded".to_string(),
                    ));
                    continue;
                }

                if cli.no_interactive == test_case.expected_no_interactive {
                    successes.push(test_case.description);
                } else {
                    failures.push((
                        test_case.description,
                        format!(
                            "Expected no_interactive={}, got {}",
                            test_case.expected_no_interactive, cli.no_interactive
                        ),
                    ));
                }
            }
            Err(e) => {
                if test_case.should_parse {
                    failures.push((test_case.description, format!("Parse error: {}", e)));
                } else {
                    successes.push(test_case.description);
                }
            }
        }
    }

    (successes, failures)
}

// ── Module tests (demonstrating utility usage) ─────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Basic parsing tests ───────────────────────────────────────────────

    #[test]
    fn test_parse_scan_no_interactive_before() {
        let cli = parse_cli(&["hoop", "--no-interactive", "scan", "/tmp"]).unwrap();
        assert_eq!(cli.no_interactive, true);
    }

    #[test]
    fn test_parse_scan_no_interactive_after() {
        let cli = parse_cli(&["hoop", "scan", "/tmp", "--no-interactive"]).unwrap();
        assert_eq!(cli.no_interactive, true);
    }

    #[test]
    fn test_parse_scan_short_flag() {
        let cli = parse_cli(&["hoop", "-y", "scan", "/tmp"]).unwrap();
        assert_eq!(cli.no_interactive, true);
    }

    #[test]
    fn test_parse_scan_no_flag() {
        let cli = parse_cli(&["hoop", "scan", "/tmp"]).unwrap();
        assert_eq!(cli.no_interactive, false);
    }

    // ── Position independence tests ─────────────────────────────────────────

    #[test]
    fn test_scan_position_independence() {
        assert!(parse_both_positions_yield_same_value(
            &["scan", "/tmp"],
            &["--no-interactive"]
        ));
    }

    #[test]
    fn test_remove_position_independence() {
        assert!(parse_both_positions_yield_same_value(
            &["remove", "test-project", "--confirm"],
            &["--no-interactive"]
        ));
    }

    #[test]
    fn test_restore_position_independence() {
        assert!(parse_both_positions_yield_same_value(
            &["restore", "--from", "s3://bucket/key", "--confirm"],
            &["--no-interactive"]
        ));
    }

    // ── Verification helper tests ──────────────────────────────────────────

    #[test]
    fn test_verify_no_interactive_value_true() {
        let cli = parse_cli(&["hoop", "--no-interactive", "scan", "/tmp"]).unwrap();
        assert!(verify_no_interactive_value(&cli, true).is_ok());
    }

    #[test]
    fn test_verify_no_interactive_value_false() {
        let cli = parse_cli(&["hoop", "scan", "/tmp"]).unwrap();
        assert!(verify_no_interactive_value(&cli, false).is_ok());
    }

    #[test]
    fn test_verify_flag_at_position_before() {
        assert!(verify_flag_at_position(&["scan", "/tmp"], "before", true).is_ok());
    }

    #[test]
    fn test_verify_flag_at_position_after() {
        assert!(verify_flag_at_position(&["scan", "/tmp"], "after", true).is_ok());
    }

    #[test]
    fn test_verify_position_independence() {
        assert!(verify_position_independence(&["scan", "/tmp"]).is_ok());
    }

    #[test]
    fn test_verify_flag_default_false() {
        assert!(verify_flag_default_false(&["scan", "/tmp"]).is_ok());
    }

    // ── Helper function tests ───────────────────────────────────────────────

    #[test]
    fn test_parse_flag_before_subcommand() {
        let cli = parse_flag_before_subcommand(&["scan", "/tmp"]).unwrap();
        assert_eq!(cli.no_interactive, true);
    }

    #[test]
    fn test_parse_flag_after_subcommand() {
        let cli = parse_flag_after_subcommand(&["scan", "/tmp"]).unwrap();
        assert_eq!(cli.no_interactive, true);
    }

    #[test]
    fn test_parse_with_short_flag() {
        let cli = parse_with_short_flag(&["scan", "/tmp"]).unwrap();
        assert_eq!(cli.no_interactive, true);
    }

    #[test]
    fn test_parse_both_positions() {
        let (before, after) = parse_both_positions(&["--no-interactive"], &["scan", "/tmp"]);
        assert_eq!(before, true);
        assert_eq!(after, true);
    }

    // ── Command extraction tests ────────────────────────────────────────────

    #[test]
    fn test_get_command_scan() {
        let cli = parse_cli(&["hoop", "scan", "/tmp"]).unwrap();
        let command = get_command(&cli);
        match command {
            Commands::Scan { root, .. } => {
                assert_eq!(root, "/tmp");
            }
            _ => panic!("Expected Scan command"),
        }
    }

    #[test]
    fn test_try_get_scan_command() {
        let cli = parse_cli(&["hoop", "scan", "/tmp"]).unwrap();
        assert!(try_get_scan_command(&cli).is_some());

        let cli = parse_cli(&["hoop", "remove", "test", "--confirm"]).unwrap();
        assert!(try_get_scan_command(&cli).is_none());
    }

    #[test]
    fn test_try_get_remove_command() {
        let cli = parse_cli(&["hoop", "remove", "test", "--confirm"]).unwrap();
        assert!(try_get_remove_command(&cli).is_some());

        let cli = parse_cli(&["hoop", "scan", "/tmp"]).unwrap();
        assert!(try_get_remove_command(&cli).is_none());
    }

    #[test]
    fn test_try_get_init_command() {
        let cli = parse_cli(&["hoop", "init"]).unwrap();
        assert!(try_get_init_command(&cli).is_some());

        let cli = parse_cli(&["hoop", "scan", "/tmp"]).unwrap();
        assert!(try_get_init_command(&cli).is_none());
    }

    // ── Batch testing example ──────────────────────────────────────────────

    #[test]
    fn test_run_clap_tests() {
        let test_cases = vec![
            ClapTestCase {
                description: "scan with flag before".to_string(),
                args: vec!["hoop", "--no-interactive", "scan", "/tmp"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                expected_no_interactive: true,
                should_parse: true,
            },
            ClapTestCase {
                description: "scan with flag after".to_string(),
                args: vec!["hoop", "scan", "/tmp", "--no-interactive"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                expected_no_interactive: true,
                should_parse: true,
            },
            ClapTestCase {
                description: "scan without flag".to_string(),
                args: vec!["hoop", "scan", "/tmp"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                expected_no_interactive: false,
                should_parse: true,
            },
        ];

        let (successes, failures) = run_clap_tests(test_cases);
        assert_eq!(successes.len(), 3);
        assert_eq!(failures.len(), 0);
    }

    // ── Edge cases ─────────────────────────────────────────────────────────

    #[test]
    fn test_scan_with_local_yes_flag_and_global_no_interactive() {
        // When both flags are present, parse should succeed
        let cli = parse_cli(&["hoop", "--no-interactive", "scan", "/tmp", "--yes"]).unwrap();
        assert_eq!(cli.no_interactive, true);

        match cli.command {
            Commands::Scan { root, auto_confirm } => {
                assert_eq!(root, "/tmp");
                assert_eq!(auto_confirm, true);
            }
            _ => panic!("Expected Scan command"),
        }
    }

    #[test]
    fn test_projects_scan_subcommand_with_global_flag() {
        let cli = parse_cli(&["hoop", "--no-interactive", "projects", "scan", "/tmp"]).unwrap();
        assert_eq!(cli.no_interactive, true);

        match cli.command {
            Commands::Projects(cmd) => match cmd {
                hoop_cli::main::ProjectsCommands::Scan { root, .. } => {
                    assert_eq!(root, "/tmp");
                }
                _ => panic!("Expected Projects::Scan command"),
            },
            _ => panic!("Expected Projects command"),
        }
    }

    #[test]
    fn test_global_flag_persists_through_command_chain() {
        // Test that global flag is accessible even for nested subcommands
        let cli = parse_cli(&["hoop", "--no-interactive", "projects", "scan", "/tmp"]).unwrap();
        assert_eq!(cli.no_interactive, true);
    }

    #[test]
    fn test_explicit_false_flag_parsing() {
        // Test that not specifying the flag yields false
        let cli = parse_cli(&["hoop", "scan", "/tmp"]).unwrap();
        assert_eq!(cli.no_interactive, false);
    }
}

// ── Documentation examples ───────────────────────────────────────────────────────

/// # Example: Testing a Single Command
///
/// ```rust
/// use clap_test_utils::*;
///
/// #[test]
/// fn test_my_command_no_interactive() {
///     // Test flag before command
///     let cli = parse_flag_before_subcommand(&["my-command", "arg1"]).unwrap();
///     assert_eq!(cli.no_interactive, true);
///
///     // Test flag after command
///     let cli = parse_flag_after_subcommand(&["my-command", "arg1"]).unwrap();
///     assert_eq!(cli.no_interactive, true);
///
///     // Test position independence
///     assert!(parse_both_positions_yield_same_value(
///         &["my-command", "arg1"],
///         &["--no-interactive"]
///     ));
///
///     // Test default value
///     let cli = parse_cli(&["hoop", "my-command", "arg1"]).unwrap();
///     assert_eq!(cli.no_interactive, false);
/// }
/// ```
///
/// # Example: Using Test Macros
///
/// ```rust
/// use clap_test_utils::*;
///
/// // Generate complete test suite for a command
/// test_command_no_interactive_suite!(scan, &["scan", "/tmp"]);
/// test_command_no_interactive_suite!(remove, &["remove", "test", "--confirm"]);
/// test_command_no_interactive_suite!(init, &["init"]);
/// ```
///
/// # Example: Batch Testing Multiple Commands
///
/// ```rust
/// use clap_test_utils::*;
///
/// #[test]
/// fn test_all_commands_no_interactive() {
///     let test_cases = vec![
///         ClapTestCase {
///             description: "scan with flag".to_string(),
///             args: vec!["hoop", "--no-interactive", "scan", "/tmp"]
///                 .iter().map(|s| s.to_string()).collect(),
///             expected_no_interactive: true,
///             should_parse: true,
///         },
///         // ... more test cases
///     ];
///
///     let (successes, failures) = run_clap_tests(test_cases);
///     assert_eq!(failures.len(), 0, "Some tests failed: {:?}", failures);
/// }
/// ```
///
/// # Example: Verifying Command-Specific Arguments
///
/// ```rust
/// use clap_test_utils::*;
///
/// #[test]
/// fn test_scan_command_arguments() {
///     let cli = parse_cli(&["hoop", "--no-interactive", "scan", "/tmp"]).unwrap();
///
///     // Verify global flag
///     assert_eq!(cli.no_interactive, true);
///
///     // Verify command-specific arguments
///     match cli.command {
///         Commands::Scan { root, auto_confirm } => {
///             assert_eq!(root, "/tmp");
///             assert_eq!(auto_confirm, false);
///         }
///         _ => panic!("Expected Scan command"),
///     }
/// }
/// ```
