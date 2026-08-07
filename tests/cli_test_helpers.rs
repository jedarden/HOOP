//! Test helper utilities for HOOP CLI testing
//!
//! This module provides reusable utilities and patterns for testing CLI commands,
//! with special focus on the `no_interactive` flag behavior across different commands.
//!
//! ## Testing Philosophy
//!
//! The `no_interactive` flag is a **global clap flag** (marked with `global = true`),
//! meaning it can be specified at any position in the command invocation:
//!
//! ```bash
//! # Before the subcommand
//! hoop --no-interactive projects remove my-project --confirm
//!
//! # After the subcommand
//! hoop projects remove my-project --no-interactive --confirm
//!
//! # With the short alias
//! hoop -y projects remove my-project --confirm
//! ```
//!
//! ## Key Testing Patterns
//!
//! 1. **Position Independence**: Verify the flag works correctly at any position
//! 2. **Short/Long Form Equivalence**: Test both `-y` and `--no-interactive`
//! 3. **Value Consistency**: Ensure flag value is extracted consistently regardless of position
//! 4. **Default Behavior**: Verify default (false) when flag is not specified
//! 5. **Flag Propagation**: Ensure global flag persists through command chains
//!
//! ## Common Test Patterns
//!
//! ### Basic flag parsing test
//! ```ignore
//! #[test]
//! fn scan_no_interactive_flag_before_command() {
//!     let args = ["hoop", "--no-interactive", "scan", "/tmp"];
//!     let cli = parse_cli_args(&args).unwrap();
//!     assert_no_interactive_true(&cli);
//! }
//! ```
//!
//! ### Position independence test
//! ```ignore
//! #[test]
//! fn scan_both_positions_extract_same_value() {
//!     let flag_args = ["--no-interactive"];
//!     let cmd_args = ["scan", "/tmp"];
//!     let (before, after) = parse_both_positions(flag_args, cmd_args);
//!     assert_eq!(before, after, "no_interactive value must be consistent");
//!     assert_eq!(before, true, "no_interactive should be true");
//! }
//! ```
//!
//! ### Short/long form equivalence
//! ```ignore
//! #[test]
//! fn scan_short_flag_y_works() {
//!     let args = ["hoop", "-y", "scan", "/tmp"];
//!     let cli = parse_cli_args(&args).unwrap();
//!     assert_no_interactive_true(&cli);
//! }
//! ```

use hoop_cli::Cli;

/// Result type for CLI parsing operations
pub type CliResult = Result<Cli, clap::Error>;

/// Parse CLI arguments and extract the parsed Cli struct
///
/// This allows testing flag parsing in isolation using clap's try_parse_from.
/// The args slice should include "hoop" as the first element (program name).
///
/// # Example
/// ```ignore
/// let args = ["hoop", "--no-interactive", "scan", "/tmp"];
/// let cli = parse_cli_args(&args).unwrap();
/// assert!(cli.no_interactive);
/// ```
pub fn parse_cli_args(args: &[&str]) -> CliResult {
    Cli::try_parse_from(args.iter())
}

/// Parse CLI arguments from a string (convenience function)
///
/// Splits a single command string into argument slices for parsing.
/// Useful for writing tests as simple strings.
///
/// # Example
/// ```ignore
/// let cli = parse_cmd_string("hoop --no-interactive scan /tmp").unwrap();
/// assert!(cli.no_interactive);
/// ```
pub fn parse_cmd_string(cmd: &str) -> CliResult {
    let args: Vec<&str> = cmd.split_whitespace().collect();
    parse_cli_args(&args)
}

/// Test flag parsing from both positions for a command
///
/// Returns a tuple of (before_value, after_value) where:
/// - `before_value` is the flag value when specified before the subcommand
/// - `after_value` is the flag value when specified after the subcommand
///
/// # Arguments
/// * `flag_args` - The flag arguments to test (e.g., `["--no-interactive"]`)
/// * `cmd_args` - The command arguments (e.g., `["scan", "/tmp"]`)
///
/// # Example
/// ```ignore
/// let (before, after) = parse_both_positions(
///     &["--no-interactive"],
///     &["scan", "/tmp"]
/// );
/// assert_eq!(before, after, "flag value must be position-independent");
/// ```
pub fn parse_both_positions(flag_args: &[&str], cmd_args: &[&str]) -> (bool, bool) {
    // Parse with flag before subcommand
    let full_args_before: Vec<&str> = ["hoop"]
        .iter()
        .chain(flag_args.iter())
        .chain(cmd_args.iter())
        .copied()
        .collect();
    let cli_before = parse_cli_args(&full_args_before).unwrap();
    let no_interactive_before = cli_before.no_interactive;

    // Parse with flag after subcommand
    let full_args_after: Vec<&str> = ["hoop"]
        .iter()
        .chain(cmd_args.iter())
        .chain(flag_args.iter())
        .copied()
        .collect();
    let cli_after = parse_cli_args(&full_args_after).unwrap();
    let no_interactive_after = cli_after.no_interactive;

    (no_interactive_before, no_interactive_after)
}

/// Assert that `no_interactive` is true in the parsed CLI
///
/// Convenience assertion for common test cases.
pub fn assert_no_interactive_true(cli: &Cli) {
    assert_eq!(
        cli.no_interactive, true,
        "no_interactive should be true"
    );
}

/// Assert that `no_interactive` is false in the parsed CLI
///
/// Convenience assertion for common test cases.
pub fn assert_no_interactive_false(cli: &Cli) {
    assert_eq!(
        cli.no_interactive, false,
        "no_interactive should be false"
    );
}

/// Assert that flag value is consistent across both positions
///
/// Combines `parse_both_positions` with an equality assertion.
pub fn assert_position_independence(flag_args: &[&str], cmd_args: &[&str]) {
    let (before, after) = parse_both_positions(flag_args, cmd_args);
    assert_eq!(
        before, after,
        "no_interactive value must be consistent regardless of flag position"
    );
}

/// Test helper macros for common test patterns
#[macro_export]
macro_rules! test_no_interactive_flag {
    // Test with flag before command
    (before: $cmd_name:ident, $args:expr) => {
        #[test]
        fn $cmd_name() {
            let cli = parse_cli_args($args).unwrap();
            assert_no_interactive_true(&cli);
        }
    };

    // Test with flag after command
    (after: $cmd_name:ident, $args:expr) => {
        #[test]
        fn $cmd_name() {
            let cli = parse_cli_args($args).unwrap();
            assert_no_interactive_true(&cli);
        }
    };

    // Test both positions give same result
    (both: $cmd_name:ident, $flag_args:expr, $cmd_args:expr) => {
        #[test]
        fn $cmd_name() {
            let (before, after) = parse_both_positions($flag_args, $cmd_args);
            assert_eq!(before, after, "no_interactive value must be consistent");
            assert_eq!(before, true, "no_interactive should be true");
        }
    };

    // Test default (flag not specified)
    (default: $cmd_name:ident, $args:expr) => {
        #[test]
        fn $cmd_name() {
            let cli = parse_cli_args($args).unwrap();
            assert_no_interactive_false(&cli);
        }
    };
}

/// Comprehensive test suite builder for a command
///
/// This macro generates a complete test suite for a command with all standard
/// `no_interactive` flag tests: before/after positions, short form, and default.
#[macro_export]
macro_rules! test_command_no_interactive {
    ($cmd_prefix:expr, $test_name_prefix:expr) => {
        mod $test_name_prefix {
            use super::*;

            #[test]
            fn flag_before_command() {
                let args = format!("hoop --no-interactive {}", $cmd_prefix);
                let cli = parse_cmd_string(&args).unwrap();
                assert_no_interactive_true(&cli);
            }

            #[test]
            fn flag_after_command() {
                let args = format!("hoop {} --no-interactive", $cmd_prefix);
                let cli = parse_cmd_string(&args).unwrap();
                assert_no_interactive_true(&cli);
            }

            #[test]
            fn short_flag_y_before_command() {
                let args = format!("hoop -y {}", $cmd_prefix);
                let cli = parse_cmd_string(&args).unwrap();
                assert_no_interactive_true(&cli);
            }

            #[test]
            fn short_flag_y_after_command() {
                let args = format!("hoop {} -y", $cmd_prefix);
                let cli = parse_cmd_string(&args).unwrap();
                assert_no_interactive_true(&cli);
            }

            #[test]
            fn both_positions_extract_same_value() {
                let flag_args = ["--no-interactive"];
                let cmd_args: Vec<&str> = $cmd_prefix.split_whitespace().collect();
                let (before, after) = parse_both_positions(&flag_args, &cmd_args);
                assert_eq!(before, after, "no_interactive value must be position-independent");
                assert_eq!(before, true, "no_interactive should be true");
            }

            #[test]
            fn default_without_flag_is_false() {
                let args = format!("hoop {}", $cmd_prefix);
                let cli = parse_cmd_string(&args).unwrap();
                assert_no_interactive_false(&cli);
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Helper function tests ─────────────────────────────────────────────

    #[test]
    fn parse_cli_args_basic() {
        let args = ["hoop", "--no-interactive", "scan", "/tmp"];
        let cli = parse_cli_args(&args).unwrap();
        assert!(cli.no_interactive);
    }

    #[test]
    fn parse_cmd_string_basic() {
        let cli = parse_cmd_string("hoop --no-interactive scan /tmp").unwrap();
        assert!(cli.no_interactive);
    }

    #[test]
    fn parse_both_positions_returns_tuple() {
        let (before, after) = parse_both_positions(
            &["--no-interactive"],
            &["scan", "/tmp"]
        );
        assert_eq!(before, true);
        assert_eq!(after, true);
    }

    #[test]
    fn assert_no_interactive_true_macro() {
        let args = ["hoop", "--no-interactive", "scan", "/tmp"];
        let cli = parse_cli_args(&args).unwrap();
        assert_no_interactive_true(&cli); // Should not panic
    }

    #[test]
    #[should_panic(expected = "no_interactive should be true")]
    fn assert_no_interactive_true_panics_on_false() {
        let args = ["hoop", "scan", "/tmp"];
        let cli = parse_cli_args(&args).unwrap();
        assert_no_interactive_true(&cli); // Should panic
    }

    #[test]
    fn assert_no_interactive_false_macro() {
        let args = ["hoop", "scan", "/tmp"];
        let cli = parse_cli_args(&args).unwrap();
        assert_no_interactive_false(&cli); // Should not panic
    }

    #[test]
    #[should_panic(expected = "no_interactive should be false")]
    fn assert_no_interactive_false_panics_on_true() {
        let args = ["hoop", "--no-interactive", "scan", "/tmp"];
        let cli = parse_cli_args(&args).unwrap();
        assert_no_interactive_false(&cli); // Should panic
    }

    #[test]
    fn assert_position_independence_helper() {
        assert_position_independence(
            &["--no-interactive"],
            &["scan", "/tmp"]
        ); // Should not panic
    }

    #[test]
    #[should_panic(expected = "no_interactive value must be consistent")]
    fn assert_position_independence_panics_on_mismatch() {
        // This would panic if there was a bug, but with current implementation
        // both positions should always give the same result for global flags
        assert_position_independence(
            &["--no-interactive"],
            &["scan", "/tmp"]
        );
    }

    // ── Real command tests using the helpers ───────────────────────────────

    #[test]
    fn scan_command_helpers_work() {
        let args = ["hoop", "--no-interactive", "scan", "/tmp"];
        let cli = parse_cli_args(&args).unwrap();
        assert_no_interactive_true(&cli);

        match cli.command {
            hoop_cli::Commands::Scan { .. } => {
                // Correct command was parsed
            }
            _ => panic!("Expected Scan command"),
        }
    }

    #[test]
    fn remove_command_helpers_work() {
        let args = ["hoop", "remove", "test", "--no-interactive", "--confirm"];
        let cli = parse_cli_args(&args).unwrap();
        assert_no_interactive_true(&cli);

        match cli.command {
            hoop_cli::Commands::Remove { .. } => {
                // Correct command was parsed
            }
            _ => panic!("Expected Remove command"),
        }
    }

    #[test]
    fn projects_subcommand_helpers_work() {
        let args = ["hoop", "--no-interactive", "projects", "scan", "/tmp"];
        let cli = parse_cli_args(&args).unwrap();
        assert_no_interactive_true(&cli);

        match cli.command {
            hoop_cli::Commands::Projects(_) => {
                // Correct command was parsed
            }
            _ => panic!("Expected Projects subcommand"),
        }
    }

    #[test]
    fn position_independence_scan_command() {
        let flag_args = ["--no-interactive"];
        let cmd_args = ["scan", "/tmp"];
        let (before, after) = parse_both_positions(&flag_args, cmd_args);

        assert_eq!(before, after, "Values must match");
        assert_eq!(before, true);
    }

    #[test]
    fn short_and_long_forms_equivalent() {
        let args_long = ["hoop", "--no-interactive", "scan", "/tmp"];
        let args_short = ["hoop", "-y", "scan", "/tmp"];

        let cli_long = parse_cli_args(&args_long).unwrap();
        let cli_short = parse_cli_args(&args_short).unwrap();

        assert_eq!(cli_long.no_interactive, cli_short.no_interactive);
        assert_eq!(cli_long.no_interactive, true);
    }
}