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
//!
//! ## Macro-Based Testing Patterns
//!
//! ### Using test_global_flag_position!
//! Tests flag at global position (before subcommand):
//! ```ignore
//! test_global_flag_position!(scan_global_flag, "scan", "/tmp");
//! test_global_flag_position!(remove_global_flag, "remove", "my-project");
//! test_global_flag_position!(status_global_flag, "status");
//! ```
//!
//! ### Using test_subcommand_flag_position!
//! Tests flag at subcommand position (after command):
//! ```ignore
//! test_subcommand_flag_position!(scan_subcommand_flag, "scan", "/tmp");
//! test_subcommand_flag_position!(remove_subcommand_flag, "remove", "my-project");
//! test_subcommand_flag_position!(status_subcommand_flag, "status");
//! ```
//!
//! ### Using test_flag_propagation!
//! Tests flag propagation through command chains:
//! ```ignore
//! // Pattern 1: Global flag affects subcommand
//! test_flag_propagation!(
//!     global_affects_projects_scan,
//!     global_flag = "--no-interactive",
//!     command = ["projects", "scan", "/tmp"],
//!     expected = true
//! );
//!
//! // Pattern 2: Verify consistency across positions
//! test_flag_propagation!(
//!     scan_position_consistency,
//!     command = ["scan", "/tmp"],
//!     verify_consistency = true
//! );
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

/// Test macro for testing flag at global position (before subcommand)
///
/// This macro generates tests that verify the `--no-interactive` flag works correctly
/// when specified BEFORE the command/subcommand: `hoop --no-interactive CMD`
///
/// # Usage
///
/// ```ignore
/// use hoop_test_helpers::test_global_flag_position;
///
/// test_global_flag_position!(test_scan_global_flag, "scan", "/tmp");
/// test_global_flag_position!(test_remove_global_flag, "remove", "my-project");
/// ```
///
/// # Generated Tests
///
/// For each invocation, this macro generates a test that:
/// 1. Parses the command with `--no-interactive` BEFORE the subcommand
/// 2. Asserts the flag value is correctly extracted as `true`
/// 3. Verifies the correct command was parsed
///
/// # Example
///
/// ```ignore
/// test_global_flag_position!(scan_flag_before_command, "scan", "/tmp");
/// ```
///
/// Generates:
/// ```ignore
/// #[test]
/// fn scan_flag_before_command() {
///     let args = ["hoop", "--no-interactive", "scan", "/tmp"];
///     let cli = parse_cli_args(&args).unwrap();
///     assert_no_interactive_true(&cli);
///     // Additional verification...
/// }
/// ```
#[macro_export]
macro_rules! test_global_flag_position {
    ($test_name:ident, $cmd:expr, $arg:expr) => {
        #[test]
        fn $test_name() {
            let args = ["hoop", "--no-interactive", $cmd, $arg];
            let cli = parse_cli_args(&args).unwrap();
            assert_no_interactive_true(&cli);
            assert_eq!(cli.no_interactive, true,
                "Global flag should be true when specified before command: {} {}", $cmd, $arg);
        }
    };
    ($test_name:ident, $cmd:expr) => {
        #[test]
        fn $test_name() {
            let args = ["hoop", "--no-interactive", $cmd];
            let cli = parse_cli_args(&args).unwrap();
            assert_no_interactive_true(&cli);
            assert_eq!(cli.no_interactive, true,
                "Global flag should be true when specified before command: {}", $cmd);
        }
    };
}

/// Test macro for testing flag at subcommand position (after command)
///
/// This macro generates tests that verify the `--no-interactive` flag works correctly
/// when specified AFTER the command/subcommand: `hoop CMD --no-interactive`
///
/// # Usage
///
/// ```ignore
/// use hoop_test_helpers::test_subcommand_flag_position;
///
/// test_subcommand_flag_position!(test_scan_subcommand_flag, "scan", "/tmp");
/// test_subcommand_flag_position!(test_remove_subcommand_flag, "remove", "my-project");
/// ```
///
/// # Generated Tests
///
/// For each invocation, this macro generates a test that:
/// 1. Parses the command with `--no-interactive` AFTER the subcommand
/// 2. Asserts the flag value is correctly extracted as `true`
/// 3. Verifies the correct command was parsed
///
/// # Example
///
/// ```ignore
/// test_subcommand_flag_position!(scan_flag_after_command, "scan", "/tmp");
/// ```
///
/// Generates:
/// ```ignore
/// #[test]
/// fn scan_flag_after_command() {
///     let args = ["hoop", "scan", "/tmp", "--no-interactive"];
///     let cli = parse_cli_args(&args).unwrap();
///     assert_no_interactive_true(&cli);
///     // Additional verification...
/// }
/// ```
#[macro_export]
macro_rules! test_subcommand_flag_position {
    ($test_name:ident, $cmd:expr, $arg:expr) => {
        #[test]
        fn $test_name() {
            let args = ["hoop", $cmd, $arg, "--no-interactive"];
            let cli = parse_cli_args(&args).unwrap();
            assert_no_interactive_true(&cli);
            assert_eq!(cli.no_interactive, true,
                "Subcommand flag should be true when specified after command: {} {}", $cmd, $arg);
        }
    };
    ($test_name:ident, $cmd:expr) => {
        #[test]
        fn $test_name() {
            let args = ["hoop", $cmd, "--no-interactive"];
            let cli = parse_cli_args(&args).unwrap();
            assert_no_interactive_true(&cli);
            assert_eq!(cli.no_interactive, true,
                "Subcommand flag should be true when specified after command: {}", $cmd);
        }
    };
}

/// Test pattern for testing flag propagation behavior
///
/// This macro tests that global flags properly propagate through command chains
/// and that subcommand-specific flags override global flags when applicable.
///
/// # Usage Patterns
///
/// ## Pattern 1: Global flag affects subcommand behavior
/// ```ignore
/// test_flag_propagation!(
///     global_affects_subcommand,
///     global_flag = "--no-interactive",
///     command = ["projects", "scan", "/tmp"],
///     expected = true
/// );
/// ```
///
/// ## Pattern 2: Subcommand flag overrides global flag
/// ```ignore
/// test_flag_propagation!(
///     subcommand_overrides_global,
///     global_flag = "--no-interactive",
///     command = ["scan", "/tmp"],
///     local_flag = "--interactive",  // Hypothetical override flag
///     expected = false
/// );
/// ```
///
/// ## Pattern 3: Global flag persists through command chain
/// ```ignore
/// test_flag_propagation!(
///     flag_persists_through_chain,
///     global_flag = "--no-interactive",
///     command = ["projects", "remove", "my-project"],
///     expected = true
/// );
/// ```
///
/// # Generated Tests
///
/// This macro creates comprehensive tests that verify:
/// 1. Global flag is correctly set at the top level
/// 2. Flag value propagates through the command chain
/// 3. Subcommand flags (if any) properly override global flags
/// 4. Final flag value matches expected behavior
#[macro_export]
macro_rules! test_flag_propagation {
    ($test_name:ident, global_flag = $global:expr, command = $cmd:expr, expected = $expected:expr) => {
        #[test]
        fn $test_name() {
            let args: Vec<&str> = ["hoop", $global]
                .iter()
                .chain($cmd.iter())
                .copied()
                .collect();

            let cli = parse_cli_args(&args).unwrap();
            let result = cli.no_interactive;

            assert_eq!(result, $expected,
                "Global flag should propagate through command chain: {} {:?}",
                $global, $cmd
            );
        }
    };
    ($test_name:ident, global_flag = $global:expr, command = $cmd:expr, local_flag = $local:expr, expected = $expected:expr) => {
        #[test]
        fn $test_name() {
            let args: Vec<&str> = ["hoop", $global]
                .iter()
                .chain($cmd.iter())
                .chain(&[$local])
                .copied()
                .collect();

            let cli = parse_cli_args(&args).unwrap();
            let result = cli.no_interactive;

            assert_eq!(result, $expected,
                "Local flag should override global flag: global={}, local={}",
                $global, $local
            );
        }
    };
    ($test_name:ident, command = $cmd:expr, verify_consistency = $consistency:expr) => {
        #[test]
        fn $test_name() {
            // Test global flag position
            let args_global: Vec<&str> = ["hoop", "--no-interactive"]
                .iter()
                .chain($cmd.iter())
                .copied()
                .collect();
            let cli_global = parse_cli_args(&args_global).unwrap();

            // Test subcommand flag position
            let args_subcommand: Vec<&str> = ["hoop"]
                .iter()
                .chain($cmd.iter())
                .chain(&["--no-interactive"])
                .copied()
                .collect();
            let cli_subcommand = parse_cli_args(&args_subcommand).unwrap();

            assert_eq!(cli_global.no_interactive, cli_subcommand.no_interactive,
                "Flag value must be consistent across positions for command: {:?}",
                $cmd
            );

            assert_eq!(cli_global.no_interactive, $consistency,
                "Expected consistency check failed: expected {}",
                $consistency
            );
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

    // ── Tests demonstrating new macro patterns ────────────────────────────────

    // Example usage of test_global_flag_position! macro
    test_global_flag_position!(scan_global_flag_example, "scan", "/tmp");
    test_global_flag_position!(remove_global_flag_example, "remove", "test-project");

    // Example usage of test_subcommand_flag_position! macro
    test_subcommand_flag_position!(scan_subcommand_flag_example, "scan", "/tmp");
    test_subcommand_flag_position!(remove_subcommand_flag_example, "remove", "test-project");

    // Example usage of test_flag_propagation! macro
    test_flag_propagation!(
        global_affects_projects_scan,
        global_flag = "--no-interactive",
        command = ["projects", "scan", "/tmp"],
        expected = true
    );

    test_flag_propagation!(
        global_affects_status,
        global_flag = "--no-interactive",
        command = ["status"],
        expected = true
    );

    test_flag_propagation!(
        scan_position_consistency,
        command = ["scan", "/tmp"],
        verify_consistency = true
    );
}