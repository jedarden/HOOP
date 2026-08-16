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

// ── Import the actual CLI structure from lib ─────────────────────────────────────
//
// The CLI types (Cli, Commands, etc.) are now defined in src/cli.rs and
// re-exported through lib.rs, making them accessible to tests in the tests/
// directory.

pub use hoop::{AuditCommands, Cli, Commands, ProjectsCommands};
use clap::Parser;

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

/// Parse a command with a global flag (flag before subcommand)
///
/// This is a general-purpose function that parses commands with flags that appear
/// BEFORE the subcommand, following the pattern: `hoop --flag CMD [args]`.
///
/// # Arguments
///
/// * `flag_args` - Flag arguments (e.g., `["--no-interactive"]` or `["--verbose", "-v"]`)
/// * `cmd_args` - Command arguments without program name (e.g., `["scan", "/tmp"]`)
///
/// # Returns
///
/// * `Result<Cli, clap::Error>` - Parsed CLI structure or clap error
///
/// # Examples
///
/// ```rust
/// // Parse: hoop --no-interactive scan /tmp
/// let cli = parse_command_with_global_flag(
///     &["--no-interactive"],
///     &["scan", "/tmp"]
/// )?;
///
/// // Parse: hoop --verbose --no-interactive scan /tmp
/// let cli = parse_command_with_global_flag(
///     &["--verbose", "--no-interactive"],
///     &["scan", "/tmp"]
/// )?;
///
/// // Parse: hoop -v -y scan /tmp
/// let cli = parse_command_with_global_flag(
///     &["-v", "-y"],
///     &["scan", "/tmp"]
/// )?;
/// ```
pub fn parse_command_with_global_flag(flag_args: &[&str], cmd_args: &[&str]) -> Result<Cli, clap::Error> {
    let full_args: Vec<&str> = ["hoop"]
        .iter()
        .chain(flag_args.iter())
        .chain(cmd_args.iter())
        .copied()
        .collect();
    parse_cli(&full_args)
}

/// Parse a command with a subcommand flag (flag after subcommand)
///
/// This is a general-purpose function that parses commands with flags that appear
/// AFTER the subcommand, following the pattern: `hoop CMD [args] --flag`.
///
/// # Arguments
///
/// * `cmd_args` - Command arguments without program name (e.g., `["scan", "/tmp"]`)
/// * `flag_args` - Flag arguments (e.g., `["--no-interactive"]` or `["--confirm"]`)
///
/// # Returns
///
/// * `Result<Cli, clap::Error>` - Parsed CLI structure or clap error
///
/// # Examples
///
/// ```rust
/// // Parse: hoop scan /tmp --no-interactive
/// let cli = parse_command_with_subcommand_flag(
///     &["scan", "/tmp"],
///     &["--no-interactive"]
/// )?;
///
/// // Parse: hoop scan /tmp --no-interactive --confirm
/// let cli = parse_command_with_subcommand_flag(
///     &["scan", "/tmp"],
///     &["--no-interactive", "--confirm"]
/// )?;
///
/// // Parse: hoop projects remove my-project --confirm
/// let cli = parse_command_with_subcommand_flag(
///     &["projects", "remove", "my-project"],
///     &["--confirm"]
/// )?;
/// ```
pub fn parse_command_with_subcommand_flag(cmd_args: &[&str], flag_args: &[&str]) -> Result<Cli, clap::Error> {
    let full_args: Vec<&str> = ["hoop"]
        .iter()
        .chain(cmd_args.iter())
        .chain(flag_args.iter())
        .copied()
        .collect();
    parse_cli(&full_args)
}

/// Extract command matches from parsed CLI output
///
/// This helper function provides a convenient way to extract and match on specific
/// command variants from the parsed CLI structure. It returns the command and
/// allows pattern matching to extract command-specific arguments.
///
/// # Arguments
///
/// * `cli` - Parsed CLI structure
///
/// # Returns
///
/// * `&Commands` - Reference to the command enum for pattern matching
///
/// # Examples
///
/// ```rust
/// let cli = parse_cli(&["hoop", "scan", "/tmp"])?;
/// let command = extract_command_matches(&cli);
///
/// match command {
///     Commands::Scan { root, auto_confirm } => {
///         assert_eq!(root, "/tmp");
///         println!("Scan command with root: {}", root);
///     }
///     Commands::Remove { name, confirm } => {
///         println!("Remove command for project: {}", name);
///     }
///     _ => println!("Other command"),
/// }
/// ```
pub fn extract_command_matches(cli: &Cli) -> &Commands {
    &cli.command
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

/// Verify that the global flag properly propagates to subcommands
///
/// This helper verifies that the no_interactive global flag is correctly
/// accessible and has the expected value when used with subcommands.
///
/// It tests:
/// 1. Global flag is accessible from subcommand context
/// 2. Flag value is correct when specified before subcommand
/// 3. Flag value is correct when specified after subcommand
/// 4. Flag propagates through nested subcommands (e.g., projects scan)
///
/// # Arguments
///
/// * `cmd_args` - Command arguments including subcommand (without program name or flag)
/// * `expected_value` - Expected value of no_interactive (default: true)
///
/// # Returns
///
/// * `Result<(), String>` - Ok if verification passes, Err with specific failure message
///
/// # Examples
///
/// ```rust
/// use clap_test_utils::*;
///
/// // Verify flag propagates to simple subcommand
/// assert!(verify_flag_propagation(&["scan", "/tmp"], true).is_ok());
///
/// // Verify flag propagates to nested subcommand
/// assert!(verify_flag_propagation(&["projects", "scan", "/tmp"], true).is_ok());
///
/// // Verify with flag before subcommand
/// let cli = parse_command_with_global_flag(&["--no-interactive"], &["scan", "/tmp"])?;
/// let flag_value = extract_no_interactive_flag(&cli);
/// assert_eq!(flag_value, true);
///
/// // Verify with flag after subcommand
/// let cli = parse_command_with_subcommand_flag(&["scan", "/tmp"], &["--no-interactive"])?;
/// let flag_value = extract_no_interactive_flag(&cli);
/// assert_eq!(flag_value, true);
/// ```
///
/// # Error Messages
///
/// This function provides clear error messages for different failure modes:
///
/// - "Failed to parse with flag before subcommand: {error}" - Parse error when flag before command
/// - "Failed to parse with flag after subcommand: {error}" - Parse error when flag after command
/// - "Flag value mismatch with flag before subcommand: expected={expected}, got={actual}" - Value mismatch
/// - "Flag value mismatch with flag after subcommand: expected={expected}, got={actual}" - Value mismatch
/// - "Flag propagation failed: both positions failed to yield expected value" - Both positions failed
///
/// # Implementation Details
///
/// This function tests both positions (before and after subcommand) because:
/// 1. Clap's global flag attribute (`global = true`) should make the flag work in both positions
/// 2. Some commands may have position-dependent parsing (this would be a bug)
/// 3. Testing both positions ensures consistent behavior regardless of how the user specifies the flag
///
/// The verification process:
/// 1. Parse with flag before subcommand: `hoop --no-interactive <cmd> <args>`
/// 2. Parse with flag after subcommand: `hoop <cmd> <args> --no-interactive`
/// 3. Extract flag value from both parses using `extract_no_interactive_flag()`
/// 4. Compare both values to expected
/// 5. Return Ok if both match, Err with details if either fails
pub fn verify_flag_propagation(cmd_args: &[&str], expected_value: bool) -> Result<(), String> {
    // Test 1: Flag before subcommand
    let cli_before = parse_command_with_global_flag(&["--no-interactive"], cmd_args)
        .map_err(|e| format!("Failed to parse with flag before subcommand: {}", e))?;

    let flag_before = extract_no_interactive_flag(&cli_before);
    if flag_before != expected_value {
        return Err(format!(
            "Flag value mismatch with flag before subcommand: expected={}, got={}",
            expected_value, flag_before
        ));
    }

    // Test 2: Flag after subcommand
    let cli_after = parse_command_with_subcommand_flag(cmd_args, &["--no-interactive"])
        .map_err(|e| format!("Failed to parse with flag after subcommand: {}", e))?;

    let flag_after = extract_no_interactive_flag(&cli_after);
    if flag_after != expected_value {
        return Err(format!(
            "Flag value mismatch with flag after subcommand: expected={}, got={}",
            expected_value, flag_after
        ));
    }

    // Both positions should yield the same value
    if flag_before != flag_after {
        return Err(format!(
            "Flag propagation failed: inconsistent values - before={}, after={}",
            flag_before, flag_after
        ));
    }

    Ok(())
}

// ── Command extraction helpers ─────────────────────────────────────────────────────

/// Extract the no_interactive flag value from parsed CLI
///
/// This function extracts the boolean value of the no_interactive flag from
/// a parsed CLI structure. It handles both global flag extraction (when specified
/// before or after the subcommand) and provides a consistent interface for
/// accessing the flag value.
///
/// The no_interactive flag is a global flag (with clap's `global = true` attribute),
/// meaning it is accessible at the top-level Cli struct regardless of which
/// subcommand is being executed. This function provides a clean abstraction for
/// extracting that value.
///
/// # Arguments
///
/// * `cli` - Parsed CLI structure
///
/// # Returns
///
/// * `bool` - The value of the no_interactive flag
///
/// # Examples
///
/// ```rust
/// use clap_test_utils::*;
///
/// // Extract from a parsed CLI with flag before subcommand
/// let cli = parse_cli(&["hoop", "--no-interactive", "scan", "/tmp"])?;
/// let no_interactive = extract_no_interactive_flag(&cli);
/// assert_eq!(no_interactive, true);
///
/// // Extract from a parsed CLI with flag after subcommand
/// let cli = parse_cli(&["hoop", "scan", "/tmp", "--no-interactive"])?;
/// let no_interactive = extract_no_interactive_flag(&cli);
/// assert_eq!(no_interactive, true);
///
/// // Extract from a parsed CLI without the flag (defaults to false)
/// let cli = parse_cli(&["hoop", "scan", "/tmp"])?;
/// let no_interactive = extract_no_interactive_flag(&cli);
/// assert_eq!(no_interactive, false);
///
/// // Extract from a nested subcommand (global flag propagates)
/// let cli = parse_cli(&["hoop", "--no-interactive", "projects", "scan", "/tmp"])?;
/// let no_interactive = extract_no_interactive_flag(&cli);
/// assert_eq!(no_interactive, true);
/// ```
///
/// # Implementation Notes
///
/// - The flag is stored at the top-level `Cli` struct, not on individual commands
/// - Due to clap's `global = true` attribute, the flag is accessible for all subcommands
/// - This function is a simple accessor that returns `cli.no_interactive`
/// - Use this instead of directly accessing `cli.no_interactive` for consistency
pub fn extract_no_interactive_flag(cli: &Cli) -> bool {
    cli.no_interactive
}

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
        assert!(cli.no_interactive);
    }

    #[test]
    fn test_parse_scan_no_interactive_after() {
        let cli = parse_cli(&["hoop", "scan", "/tmp", "--no-interactive"]).unwrap();
        assert!(cli.no_interactive);
    }

    #[test]
    fn test_parse_scan_short_flag() {
        let cli = parse_cli(&["hoop", "-y", "scan", "/tmp"]).unwrap();
        assert!(cli.no_interactive);
    }

    #[test]
    fn test_parse_scan_no_flag() {
        let cli = parse_cli(&["hoop", "scan", "/tmp"]).unwrap();
        assert!(!cli.no_interactive);
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
        assert!(cli.no_interactive);
    }

    #[test]
    fn test_parse_flag_after_subcommand() {
        let cli = parse_flag_after_subcommand(&["scan", "/tmp"]).unwrap();
        assert!(cli.no_interactive);
    }

    #[test]
    fn test_parse_with_short_flag() {
        let cli = parse_with_short_flag(&["scan", "/tmp"]).unwrap();
        assert!(cli.no_interactive);
    }

    #[test]
    fn test_parse_both_positions() {
        let (before, after) = parse_both_positions(&["--no-interactive"], &["scan", "/tmp"]);
        assert!(before);
        assert!(after);
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

    // ── General parsing function tests ─────────────────────────────────────────────

    #[test]
    fn test_parse_command_with_global_flag_single() {
        // Test: hoop --no-interactive scan /tmp
        let cli = parse_command_with_global_flag(
            &["--no-interactive"],
            &["scan", "/tmp"]
        ).unwrap();

        assert!(cli.no_interactive);
        match cli.command {
            Commands::Scan { root, .. } => {
                assert_eq!(root, "/tmp");
            }
            _ => panic!("Expected Scan command"),
        }
    }

    #[test]
    fn test_parse_command_with_global_flag_position_independence() {
        // Test that global flag position doesn't matter
        // Both before and after subcommand should work identically
        let before = parse_command_with_global_flag(
            &["--no-interactive"],
            &["scan", "/tmp"]
        ).unwrap();

        let after = parse_command_with_subcommand_flag(
            &["scan", "/tmp"],
            &["--no-interactive"]
        ).unwrap();

        assert_eq!(before.no_interactive, after.no_interactive);
        assert!(before.no_interactive);
    }

    #[test]
    fn test_parse_command_with_subcommand_flag_single() {
        // Test: hoop scan /tmp --no-interactive
        let cli = parse_command_with_subcommand_flag(
            &["scan", "/tmp"],
            &["--no-interactive"]
        ).unwrap();

        assert!(cli.no_interactive);
        match cli.command {
            Commands::Scan { root, .. } => {
                assert_eq!(root, "/tmp");
            }
            _ => panic!("Expected Scan command"),
        }
    }

    #[test]
    fn test_parse_command_with_subcommand_flag_multiple() {
        // Test: hoop scan /tmp --no-interactive --yes
        let cli = parse_command_with_subcommand_flag(
            &["scan", "/tmp"],
            &["--no-interactive", "--yes"]
        ).unwrap();

        assert!(cli.no_interactive);
        match cli.command {
            Commands::Scan { root, auto_confirm } => {
                assert_eq!(root, "/tmp");
                assert!(auto_confirm);
            }
            _ => panic!("Expected Scan command"),
        }
    }

    #[test]
    fn test_extract_command_matches_scan() {
        let cli = parse_cli(&["hoop", "scan", "/tmp"]).unwrap();
        let command = extract_command_matches(&cli);

        match command {
            Commands::Scan { root, auto_confirm } => {
                assert_eq!(root, "/tmp");
                assert!(!(*auto_confirm));
            }
            _ => panic!("Expected Scan command"),
        }
    }

    #[test]
    fn test_extract_command_matches_remove() {
        let cli = parse_cli(&["hoop", "remove", "test-project", "--confirm"]).unwrap();
        let command = extract_command_matches(&cli);

        match command {
            Commands::Remove { name, confirm } => {
                assert_eq!(name, "test-project");
                assert!(*confirm);
            }
            _ => panic!("Expected Remove command"),
        }
    }

    #[test]
    fn test_extract_command_matches_projects_subcommand() {
        let cli = parse_cli(&["hoop", "projects", "scan", "/tmp"]).unwrap();
        let command = extract_command_matches(&cli);

        match command {
            Commands::Projects(ProjectsCommands::Scan { root, .. }) => {
                assert_eq!(root, "/tmp");
            }
            _ => panic!("Expected Projects::Scan command"),
        }
    }

    #[test]
    fn test_general_flag_parsing_position_independence() {
        // Test that both global and subcommand flag positions work correctly
        let before = parse_command_with_global_flag(
            &["--no-interactive"],
            &["scan", "/tmp"]
        ).unwrap();

        let after = parse_command_with_subcommand_flag(
            &["scan", "/tmp"],
            &["--no-interactive"]
        ).unwrap();

        assert_eq!(before.no_interactive, after.no_interactive);
        assert!(before.no_interactive);
    }

    // ── Edge cases ─────────────────────────────────────────────────────────

    #[test]
    fn test_scan_with_local_yes_flag_and_global_no_interactive() {
        // When both flags are present, parse should succeed
        let cli = parse_cli(&["hoop", "--no-interactive", "scan", "/tmp", "--yes"]).unwrap();
        assert!(cli.no_interactive);

        match cli.command {
            Commands::Scan { root, auto_confirm } => {
                assert_eq!(root, "/tmp");
                assert!(auto_confirm);
            }
            _ => panic!("Expected Scan command"),
        }
    }

    #[test]
    fn test_projects_scan_subcommand_with_global_flag() {
        let cli = parse_cli(&["hoop", "--no-interactive", "projects", "scan", "/tmp"]).unwrap();
        assert!(cli.no_interactive);

        match cli.command {
            Commands::Projects(cmd) => match cmd {
                ProjectsCommands::Scan { root, .. } => {
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
        assert!(cli.no_interactive);
    }

    #[test]
    fn test_explicit_false_flag_parsing() {
        // Test that not specifying the flag yields false
        let cli = parse_cli(&["hoop", "scan", "/tmp"]).unwrap();
        assert!(!cli.no_interactive);
    }

    // ── New flag extraction and verification tests ─────────────────────────────

    #[test]
    fn test_extract_no_interactive_flag_true() {
        let cli = parse_cli(&["hoop", "--no-interactive", "scan", "/tmp"]).unwrap();
        let flag = extract_no_interactive_flag(&cli);
        assert!(flag);
    }

    #[test]
    fn test_extract_no_interactive_flag_false() {
        let cli = parse_cli(&["hoop", "scan", "/tmp"]).unwrap();
        let flag = extract_no_interactive_flag(&cli);
        assert!(!flag);
    }

    #[test]
    fn test_extract_no_interactive_flag_after_subcommand() {
        let cli = parse_cli(&["hoop", "scan", "/tmp", "--no-interactive"]).unwrap();
        let flag = extract_no_interactive_flag(&cli);
        assert!(flag);
    }

    #[test]
    fn test_extract_no_interactive_flag_nested_subcommand() {
        let cli = parse_cli(&["hoop", "--no-interactive", "projects", "scan", "/tmp"]).unwrap();
        let flag = extract_no_interactive_flag(&cli);
        assert!(flag);
    }

    #[test]
    fn test_verify_flag_propagation_simple_command() {
        // Verify flag propagates to simple subcommand
        assert!(verify_flag_propagation(&["scan", "/tmp"], true).is_ok());
    }

    #[test]
    fn test_verify_flag_propagation_nested_subcommand() {
        // Verify flag propagates to nested subcommand
        assert!(verify_flag_propagation(&["projects", "scan", "/tmp"], true).is_ok());
    }

    #[test]
    fn test_verify_flag_propagation_remove_command() {
        // Verify flag propagates to remove command with confirm flag
        assert!(verify_flag_propagation(&["remove", "test-project", "--confirm"], true).is_ok());
    }

    #[test]
    fn test_verify_flag_propagation_restore_command() {
        // Verify flag propagates to restore command with multiple args
        assert!(verify_flag_propagation(
            &["restore", "--from", "s3://bucket/key", "--confirm"],
            true
        ).is_ok());
    }

    #[test]
    fn test_verify_flag_propagation_init_command() {
        // Verify flag propagates to init command (even though init rejects it)
        assert!(verify_flag_propagation(&["init"], true).is_ok());
    }

    #[test]
    fn test_verify_flag_propagation_with_extraction() {
        // Integration test: parse, extract, and verify
        let cli_before = parse_command_with_global_flag(
            &["--no-interactive"],
            &["scan", "/tmp"]
        ).unwrap();

        let cli_after = parse_command_with_subcommand_flag(
            &["scan", "/tmp"],
            &["--no-interactive"]
        ).unwrap();

        // Extract flags
        let flag_before = extract_no_interactive_flag(&cli_before);
        let flag_after = extract_no_interactive_flag(&cli_after);

        // Verify both are true and equal
        assert!(flag_before);
        assert!(flag_after);
        assert_eq!(flag_before, flag_after);
    }

    #[test]
    fn test_extract_and_verify_workflow() {
        // Complete workflow: extract and verify in sequence
        let cli = parse_cli(&["hoop", "--no-interactive", "projects", "scan", "/tmp"]).unwrap();

        // Extract the flag
        let no_interactive = extract_no_interactive_flag(&cli);

        // Verify the extracted value is correct
        assert!(verify_no_interactive_value(&cli, no_interactive).is_ok());

        // Verify flag propagated correctly
        assert!(verify_flag_propagation(&["projects", "scan", "/tmp"], no_interactive).is_ok());
    }
}

// ── Documentation examples ───────────────────────────────────────────────────────
//
// These are example usage patterns that demonstrate how to use the utilities in this module.
// They are provided as reference for writing new tests.
//
// Example: Testing a Single Command
// -----------------------------------
// ```rust
// use clap_test_utils::*;
//
// #[test]
// fn test_my_command_no_interactive() {
//     // Test flag before command
//     let cli = parse_flag_before_subcommand(&["my-command", "arg1"]).unwrap();
//     assert_eq!(cli.no_interactive, true);
//
//     // Test flag after command
//     let cli = parse_flag_after_subcommand(&["my-command", "arg1"]).unwrap();
//     assert_eq!(cli.no_interactive, true);
//
//     // Test position independence
//     assert!(parse_both_positions_yield_same_value(
//         &["my-command", "arg1"],
//         &["--no-interactive"]
//     ));
//
//     // Test default value
//     let cli = parse_cli(&["hoop", "my-command", "arg1"]).unwrap();
//     assert_eq!(cli.no_interactive, false);
// }
// ```
//
// Example: Using Test Macros
// ---------------------------
// ```rust
// use clap_test_utils::*;
//
// // Generate complete test suite for a command
// test_command_no_interactive_suite!(scan, &["scan", "/tmp"]);
// test_command_no_interactive_suite!(remove, &["remove", "test", "--confirm"]);
// test_command_no_interactive_suite!(init, &["init"]);
// ```
//
// Example: Batch Testing Multiple Commands
// -------------------------------------------
// ```rust
// use clap_test_utils::*;
//
// #[test]
// fn test_all_commands_no_interactive() {
//     let test_cases = vec![
//         ClapTestCase {
//             description: "scan with flag".to_string(),
//             args: vec!["hoop", "--no-interactive", "scan", "/tmp"]
//                 .iter().map(|s| s.to_string()).collect(),
//             expected_no_interactive: true,
//             should_parse: true,
//         },
//         // ... more test cases
//     ];
//
//     let (successes, failures) = run_clap_tests(test_cases);
//     assert_eq!(failures.len(), 0, "Some tests failed: {:?}", failures);
// }
// ```
//
// Example: Verifying Command-Specific Arguments
// ------------------------------------------------
// ```rust
// use clap_test_utils::*;
//
// #[test]
// fn test_scan_command_arguments() {
//     let cli = parse_cli(&["hoop", "--no-interactive", "scan", "/tmp"]).unwrap();
//
//     // Verify global flag
//     assert_eq!(cli.no_interactive, true);
//
//     // Verify command-specific arguments
//     match cli.command {
//         Commands::Scan { root, auto_confirm } => {
//             assert_eq!(root, "/tmp");
//             assert_eq!(auto_confirm, false);
//         }
//         _ => panic!("Expected Scan command"),
//     }
// }
// ```
//
// Example: Using General-Purpose Flag Parsing Functions
// ------------------------------------------------------
// The new general-purpose functions can work with any flag, not just `--no-interactive`:
//
// ```rust
// use clap_test_utils::*;
//
// #[test]
// fn test_various_flag_positions() {
//     // Test with different flags
//     let verbose_before = parse_command_with_global_flag(
//         &["--verbose"],
//         &["scan", "/tmp"]
//     ).unwrap();
//
//     let confirm_after = parse_command_with_subcommand_flag(
//         &["remove", "test-project"],
//         &["--confirm"]
//     ).unwrap();
//
//     // Extract and match on commands
//     let command = extract_command_matches(&verbose_before);
//     match command {
//         Commands::Scan { root, .. } => {
//             assert_eq!(root, "/tmp");
//         }
//         _ => panic!("Expected Scan command"),
//     }
//
//     let command = extract_command_matches(&confirm_after);
//     match command {
//         Commands::Remove { name, confirm } => {
//             assert_eq!(name, "test-project");
//             assert_eq!(confirm, true);
//         }
//         _ => panic!("Expected Remove command"),
//     }
// }
// ```
//
// Example: Testing Position Independence with Any Flag
// ------------------------------------------------------
// ```rust
// use clap_test_utils::*;
//
// #[test]
// fn test_flag_position_independence_generic() {
//     // Test that flag position doesn't matter for parsing
//     let before = parse_command_with_global_flag(
//         &["--no-interactive"],
//         &["scan", "/tmp"]
//     ).unwrap();
//
//     let after = parse_command_with_subcommand_flag(
//         &["scan", "/tmp"],
//         &["--no-interactive"]
//     ).unwrap();
//
//     // Both should yield the same parsed result
//     assert_eq!(before.no_interactive, after.no_interactive);
// }
// ```
//
// Example: Complex Multi-Flag Scenarios
// --------------------------------------
// ```rust
// use clap_test_utils::*;
//
// #[test]
// fn test_multiple_flags_combinations() {
//     // Multiple flags before command
//     let cli1 = parse_command_with_global_flag(
//         &["--verbose", "--no-interactive"],
//         &["scan", "/tmp"]
//     ).unwrap();
//     assert_eq!(cli1.no_interactive, true);
//
//     // Multiple flags after command
//     let cli2 = parse_command_with_subcommand_flag(
//         &["restore", "--from", "s3://bucket/key"],
//         &["--no-interactive", "--confirm"]
//     ).unwrap();
//     assert_eq!(cli2.no_interactive, true);
//
//     // Short flags
//     let cli3 = parse_command_with_global_flag(
//         &["-v", "-y"],
//         &["scan", "/tmp"]
//     ).unwrap();
//     assert_eq!(cli3.no_interactive, true);
// }
// ```
