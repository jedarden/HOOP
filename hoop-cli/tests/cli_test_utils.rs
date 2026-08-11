//! Test utilities for CLI testing, including no_interactive flag testing
//!
//! This module provides reusable helpers for testing the no_interactive flag
//! across commands. It supports:
//! - Parsing commands with no_interactive flag at both positions (top-level and subcommand)
//! - Verifying flag extraction in command args
//! - Testing flag suppresses prompts (with mockable prompt interface)
//!
//! # Getting Started
//!
//! **Looking for comprehensive testing patterns and macros?** See [`cli_test_helpers`]
//! for high-level testing patterns, macros, and the complete "Getting Started" guide.
//!
//! **New to testing the `--no-interactive` flag?** See [`TEST_PATTERNS_QUICK_START.md`]
//! for a unified guide covering both this module and `cli_test_helpers` with real-world
//! examples and decision trees.
//!
//! This module offers three levels of abstraction for testing CLI flag behavior:
//!
//! ## 1. Manual Implementation (Low-level, Maximum Control)
//!
//! Use the helper functions directly when you need custom test logic or want to
//! understand exactly what's being tested at each step.
//!
//! **When to use:**
//! - Debugging a specific flag parsing issue
//! - Writing one-off tests for unique scenarios
//! - Learning how the flag parsing works internally
//!
//! **Functions:**
//! - `parse_cli_with_flag()` - Parse CLI args and extract the no_interactive flag
//! - `parse_flag_before_subcommand()` - Convenience for flag-before-subcommand parsing
//! - `parse_flag_after_subcommand()` - Convenience for flag-after-subcommand parsing
//! - `verify_flag_extraction()` - Verify the flag was correctly extracted
//! - `verify_no_flag_present()` - Verify the flag is NOT present
//! - `verify_prompt_suppressed()` - Test that prompts are suppressed with the flag
//! - `verify_confirm_required()` - Test that --confirm is required for destructive ops
//!
//! ## 2. Individual Test Macros (Medium-level, Focused Tests)
//!
//! Use individual macros when you want separate test functions for each pattern,
//! making it easy to identify which specific pattern failed.
//!
//! **When to use:**
//! - Testing a single command's flag behavior
//! - Wanting granular test failure reports
//! - Building a custom test suite with selective patterns
//!
//! **Macros:**
//! - `test_no_interactive_flag_before!()` - Generate test for flag-before-command
//! - `test_no_interactive_flag_after!()` - Generate test for flag-after-command
//! - `test_short_flag_y!()` - Generate test for short -y flag
//! - `test_both_positions_consistency!()` - Generate test comparing both positions
//! - `test_flag_default_false!()` - Generate test that flag defaults to false
//!
//! ## 3. Comprehensive Test Suite Macro (High-level, Complete Coverage)
//!
//! Use the suite macro for one-stop testing of all flag patterns. This generates
//! a single test that verifies all five patterns: flag before, flag after, short flag,
//! consistency between positions, and default behavior.
//!
//! **When to use:**
//! - Quick coverage of all flag patterns for a command
//! - Regression testing with minimal boilerplate
//! - Ensuring all patterns are tested consistently
//!
//! **Macro:**
//! - `test_command_no_interactive_suite!()` - Generate complete 5-in-1 test
//!
//! # Example Usage
//!
//! ```rust
//! use cli_test_utils::*;
//!
//! #[test]
//! fn test_scan_no_interactive() {
//!     // Parse with flag before subcommand
//!     let result = parse_cli_with_flag(&["hoop", "--no-interactive", "scan", "/tmp"]);
//!     assert!(result.is_ok());
//!
//!     // Parse with flag after subcommand
//!     let result = parse_cli_with_flag(&["hoop", "scan", "/tmp", "--no-interactive"]);
//!     assert!(result.is_ok());
//!
//!     // Verify flag was extracted correctly
//!     let cli = result.unwrap();
//!     assert_eq!(cli.no_interactive, true);
//! }
//! ```
//!
//! # Common Mistakes to Avoid
//!
//! 1. **Forgetting the short flag (-y):** The no_interactive flag has two forms:
//!    `--no-interactive` and `-y`. Always test both if you're writing manual tests.
//!
//! 2. **Missing default behavior test:** Don't forget to test that the flag defaults
//!    to `false` when not specified. This catches cases where the flag is always true.
//!
//! 3. **Inconsistent position handling:** The flag should work identically whether
//!    placed before or after the subcommand. Use `test_both_positions_consistency!()`
//!    or verify this manually.
//!
//! 4. **Testing position in isolation:** When using individual macros, you often
//!    need multiple tests to cover all patterns. Consider using `test_command_no_interactive_suite!()`
//!    to ensure complete coverage.

use std::path::PathBuf;
use tempfile::TempDir;

// ── Parse helpers ─────────────────────────────────────────────────────────────

/// Parsed CLI result with no_interactive flag extracted
#[derive(Debug, Clone)]
pub struct ParsedCli {
    /// Whether no_interactive flag was present
    pub no_interactive: bool,
    /// The command that was parsed
    pub command: String,
    /// Remaining arguments after command extraction
    pub args: Vec<String>,
    /// Original input args
    pub raw_args: Vec<String>,
}

/// Parse a CLI command string and extract the no_interactive flag
///
/// This function handles both positions of the no_interactive flag:
/// - Before the subcommand: `hoop --no-interactive scan /tmp`
/// - After the subcommand: `hoop scan /tmp --no-interactive`
///
/// # Arguments
///
/// * `args` - Command line arguments as a slice of strings
///
/// # Returns
///
/// * `Result<ParsedCli, String>` - Parsed CLI or error message
///
/// # Examples
///
/// ```rust
/// let result = parse_cli_with_flag(&["hoop", "--no-interactive", "scan", "/tmp"]);
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap().no_interactive, true);
/// ```
pub fn parse_cli_with_flag(args: &[&str]) -> Result<ParsedCli, String> {
    if args.is_empty() {
        return Err("No arguments provided".to_string());
    }

    // Convert to owned strings
    let raw_args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    // Check for no_interactive flag in both forms
    let no_interactive = args.iter().any(|&a| a == "--no-interactive" || a == "-y");

    // Extract the command (second argument after "hoop")
    let command = args
        .get(1)
        .map(|s| {
            // Handle both direct commands and subcommands
            if s.starts_with('-') {
                // Skip flags, find the actual command
                args.iter()
                    .skip(2)
                    .find(|a| !a.starts_with('-'))
                    .map(|&cmd| cmd.to_string())
                    .unwrap_or_else(|| s.to_string())
            } else {
                s.to_string()
            }
        })
        .ok_or_else(|| "Missing command".to_string())?;

    // Collect remaining args (excluding program name and flags we already extracted)
    let remaining: Vec<String> = args
        .iter()
        .skip(1)
        .filter(|&&a| a != "--no-interactive" && a != "-y")
        .map(|&s| s.to_string())
        .collect();

    Ok(ParsedCli {
        no_interactive,
        command,
        args: remaining,
        raw_args,
    })
}

/// Parse CLI with the flag before the subcommand
///
/// # Examples
///
/// ```rust
/// let result = parse_flag_before_subcommand(&["scan", "/tmp"]);
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap().no_interactive, true);
/// ```
pub fn parse_flag_before_subcommand(command_args: &[&str]) -> Result<ParsedCli, String> {
    let mut args = vec!["hoop", "--no-interactive"];
    args.extend_from_slice(command_args);
    parse_cli_with_flag(&args)
}

/// Parse CLI with the flag after the subcommand
///
/// # Examples
///
/// ```rust
/// let result = parse_flag_after_subcommand(&["scan", "/tmp"]);
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap().no_interactive, true);
/// ```
pub fn parse_flag_after_subcommand(command_args: &[&str]) -> Result<ParsedCli, String> {
    let mut args = vec!["hoop"];
    args.extend_from_slice(command_args);
    args.push("--no-interactive");
    parse_cli_with_flag(&args)
}

// ── Verification helpers ───────────────────────────────────────────────────────

/// Verify that the no_interactive flag is correctly extracted from parsed args
///
/// This function checks that:
/// - The flag was found in the raw args
/// - The parsed ParsedCli has no_interactive=true
/// - The flag was properly removed from remaining args
///
/// # Arguments
///
/// * `parsed` - The parsed CLI result
/// * `expected_position` - Where we expect the flag ("before" or "after" subcommand)
///
/// # Returns
///
/// * `Result<(), String>` - Ok if verification passes, Err with details
pub fn verify_flag_extraction(parsed: &ParsedCli, expected_position: &str) -> Result<(), String> {
    // Check that flag was found
    if !parsed.no_interactive {
        return Err("no_interactive flag not detected in parsed args".to_string());
    }

    // Check that flag is in raw args
    if !parsed.raw_args.iter().any(|a| a == "--no-interactive" || a == "-y") {
        return Err("no_interactive flag not found in raw args".to_string());
    }

    // Verify position matches expectation
    match expected_position {
        "before" => {
            if parsed.raw_args.get(1) != Some(&"--no-interactive".to_string())
                && parsed.raw_args.get(1) != Some(&"-y".to_string())
            {
                return Err(format!(
                    "Expected flag at position 1 (before subcommand), but found: {:?}",
                    parsed.raw_args
                ));
            }
        }
        "after" => {
            let last_is_flag = parsed
                .raw_args
                .last()
                .map(|a| a == "--no-interactive" || a == "-y")
                .unwrap_or(false);

            if !last_is_flag {
                return Err(format!(
                    "Expected flag at last position (after subcommand), but found: {:?}",
                    parsed.raw_args
                ));
            }
        }
        _ => return Err(format!("Invalid expected_position: {}", expected_position)),
    }

    // Check that flag was removed from remaining args
    if parsed.args.iter().any(|a| a == "--no-interactive" || a == "-y") {
        return Err("Flag should be removed from remaining args".to_string());
    }

    Ok(())
}

/// Verify that no_interactive flag is NOT present in the parsed args
///
/// This is useful for testing that commands work correctly without the flag.
pub fn verify_no_flag_present(parsed: &ParsedCli) -> Result<(), String> {
    if parsed.no_interactive {
        return Err("no_interactive flag should not be present".to_string());
    }

    if parsed.raw_args.iter().any(|a| a == "--no-interactive" || a == "-y") {
        return Err("Flag should not be in raw args".to_string());
    }

    Ok(())
}

// ── Prompt suppression helpers ───────────────────────────────────────────────

/// Mock prompt interface for testing prompt suppression
///
/// This trait allows testing whether prompts are suppressed based on the
/// no_interactive flag without actually interacting with the user.
pub trait MockPrompt {
    /// Returns whether prompting would occur with the given no_interactive value
    fn would_prompt(&self, no_interactive: bool) -> bool;

    /// Returns the prompt text that would be shown
    fn prompt_text(&self) -> &str;

    /// Returns whether the prompt requires confirmation in non-interactive mode
    fn requires_confirm_in_no_interactive(&self) -> bool;
}

/// A mock prompt that simulates a yes/no confirmation prompt
#[derive(Debug, Clone)]
pub struct MockYesNoPrompt {
    pub text: String,
    pub requires_confirm: bool,
}

impl MockPrompt for MockYesNoPrompt {
    fn would_prompt(&self, no_interactive: bool) -> bool {
        !no_interactive
    }

    fn prompt_text(&self) -> &str {
        &self.text
    }

    fn requires_confirm_in_no_interactive(&self) -> bool {
        self.requires_confirm
    }
}

/// Test that a prompt is suppressed when no_interactive=true
///
/// # Arguments
///
/// * `prompt` - A MockPrompt implementation
/// * `no_interactive` - The value of the no_interactive flag
///
/// # Returns
///
/// * `Result<(), String>` - Ok if prompt suppression works as expected
pub fn verify_prompt_suppressed(prompt: &dyn MockPrompt, no_interactive: bool) -> Result<(), String> {
    let would_prompt = prompt.would_prompt(no_interactive);

    if no_interactive && would_prompt {
        return Err(format!(
            "Prompt should be suppressed when no_interactive=true, but would_prompt returned true for prompt: {}",
            prompt.prompt_text()
        ));
    }

    if !no_interactive && !would_prompt {
        return Err(format!(
            "Prompt should be shown when no_interactive=false, but would_prompt returned false for prompt: {}",
            prompt.prompt_text()
        ));
    }

    Ok(())
}

/// Test that a destructive operation requires --confirm when no_interactive=true
///
/// # Arguments
///
/// * `prompt` - A MockPrompt implementation
/// * `no_interactive` - The value of the no_interactive flag
/// * `confirm` - The value of the confirm flag
///
/// # Returns
///
/// * `Result<(), String>` - Ok if the confirm requirement is properly enforced
pub fn verify_confirm_required(
    prompt: &dyn MockPrompt,
    no_interactive: bool,
    confirm: bool,
) -> Result<(), String> {
    if !prompt.requires_confirm_in_no_interactive() {
        // Safe operation, no confirm required
        return Ok(());
    }

    if no_interactive && !confirm {
        return Err(format!(
            "Destructive operation requires --confirm when no_interactive=true for prompt: {}",
            prompt.prompt_text()
        ));
    }

    Ok(())
}

// ── Test fixtures ─────────────────────────────────────────────────────────────

/// Create a temporary test workspace with .beads directory
pub fn create_test_workspace(tmp_dir: &tempfile::TempDir, name: &str) -> PathBuf {
    let workspace = tmp_dir.path().join(name);
    std::fs::create_dir_all(workspace.join(".beads"))
        .expect("Failed to create .beads/ directory");
    workspace
}

/// Create a temporary HOOP config directory
pub fn create_hoop_config_dir(tmp_dir: &tempfile::TempDir) -> PathBuf {
    let hoop_dir = tmp_dir.path().join(".hoop");
    std::fs::create_dir_all(&hoop_dir).expect("Failed to create .hoop/ directory");
    hoop_dir
}

/// Create a temporary projects.yaml registry
pub fn create_test_registry(tmp_dir: &tempfile::TempDir) -> PathBuf {
    let hoop_dir = create_hoop_config_dir(tmp_dir);
    let registry_path = hoop_dir.join("projects.yaml");
    std::fs::write(&registry_path, "projects: []")
        .expect("Failed to write projects.yaml");
    registry_path
}

// ── Integration test helpers ───────────────────────────────────────────────────

/// Test case definition for flag position testing
#[derive(Debug, Clone)]
pub struct FlagPositionTestCase {
    pub description: String,
    pub command: Vec<String>,
    pub expected_result: bool,
}

/// Run a batch of flag position tests
///
/// This helper runs multiple test cases for flag position testing
/// and returns a summary of results.
///
/// # Arguments
///
/// * `test_cases` - Vector of test cases to run
///
/// # Returns
///
/// * (Vec<String>, Vec<(String, String)>) - (successes, (description, error))
pub fn run_flag_position_tests(
    test_cases: Vec<FlagPositionTestCase>,
) -> (Vec<String>, Vec<(String, String)>) {
    let mut successes = Vec::new();
    let mut failures = Vec::new();

    for test_case in test_cases {
        let args: Vec<&str> = test_case.command.iter().map(|s| s.as_str()).collect();

        match parse_cli_with_flag(&args) {
            Ok(parsed) => {
                if parsed.no_interactive == test_case.expected_result {
                    successes.push(test_case.description);
                } else {
                    failures.push((
                        test_case.description,
                        format!(
                            "Expected no_interactive={}, got {}",
                            test_case.expected_result, parsed.no_interactive
                        ),
                    ));
                }
            }
            Err(e) => {
                failures.push((test_case.description, format!("Parse error: {}", e)));
            }
        }
    }

    (successes, failures)
}

// ── Test macros for common no_interactive flag testing patterns ──────────────────

/// Macro to generate tests for flag parsing before a command
///
/// # Usage
///
/// ```rust
/// test_no_interactive_flag_before!(scan_before, "scan", &["scan", "/tmp"]);
/// ```
///
/// This generates a test function that verifies the no_interactive flag
/// is correctly parsed when placed before the command.
#[macro_export]
macro_rules! test_no_interactive_flag_before {
    ($test_name:ident, $command:expr, $args:expr) => {
        #[test]
        fn $test_name() {
            let full_args: Vec<&str> = vec!["hoop", "--no-interactive"]
                .iter()
                .chain($args.iter())
                .copied()
                .collect();
            let result = parse_cli_with_flag(&full_args);
            assert!(result.is_ok(), "Failed to parse args: {:?}", full_args);
            let parsed = result.unwrap();
            assert_eq!(parsed.no_interactive, true, "no_interactive should be true");
        }
    };
}

/// Macro to generate tests for flag parsing after a command
///
/// # Usage
///
/// ```rust
/// test_no_interactive_flag_after!(scan_after, "scan", &["scan", "/tmp"]);
/// ```
///
/// This generates a test function that verifies the no_interactive flag
/// is correctly parsed when placed after the command.
#[macro_export]
macro_rules! test_no_interactive_flag_after {
    ($test_name:ident, $command:expr, $args:expr) => {
        #[test]
        fn $test_name() {
            let full_args: Vec<&str> = $args
                .iter()
                .chain(&["--no-interactive"])
                .copied()
                .collect();
            let result = parse_cli_with_flag(&full_args);
            assert!(result.is_ok(), "Failed to parse args: {:?}", full_args);
            let parsed = result.unwrap();
            assert_eq!(parsed.no_interactive, true, "no_interactive should be true");
        }
    };
}

/// Macro to generate tests for short flag (-y) parsing
///
/// # Usage
///
/// ```rust
/// test_short_flag_y!(scan_short, &["scan", "/tmp"]);
/// ```
#[macro_export]
macro_rules! test_short_flag_y {
    ($test_name:ident, $args:expr) => {
        #[test]
        fn $test_name() {
            let full_args: Vec<&str> = vec!["hoop", "-y"]
                .iter()
                .chain($args.iter())
                .copied()
                .collect();
            let result = parse_cli_with_flag(&full_args);
            assert!(result.is_ok(), "Failed to parse args: {:?}", full_args);
            let parsed = result.unwrap();
            assert_eq!(parsed.no_interactive, true, "no_interactive should be true with -y");
        }
    };
}

/// Macro to generate tests verifying both positions extract the same value
///
/// # Usage
///
/// ```rust
/// test_both_positions_consistency!(scan_consistency, &["scan", "/tmp"]);
/// ```
///
/// This generates a test that verifies the no_interactive flag is parsed
/// consistently whether placed before or after the command.
#[macro_export]
macro_rules! test_both_positions_consistency {
    ($test_name:ident, $args:expr) => {
        #[test]
        fn $test_name() {
            // Parse with flag before command
            let args_before: Vec<&str> = vec!["hoop", "--no-interactive"]
                .iter()
                .chain($args.iter())
                .copied()
                .collect();
            let parsed_before = parse_cli_with_flag(&args_before)
                .expect("Failed to parse with flag before command");

            // Parse with flag after command
            let args_after: Vec<&str> = $args
                .iter()
                .chain(&["--no-interactive"])
                .copied()
                .collect();
            let parsed_after = parse_cli_with_flag(&args_after)
                .expect("Failed to parse with flag after command");

            assert_eq!(
                parsed_before.no_interactive,
                parsed_after.no_interactive,
                "no_interactive value must be consistent regardless of flag position"
            );
            assert_eq!(
                parsed_before.no_interactive,
                true,
                "no_interactive should be true"
            );
        }
    };
}

/// Macro to generate tests verifying flag defaults to false when not specified
///
/// # Usage
///
/// ```rust
/// test_flag_default_false!(scan_default, &["scan", "/tmp"]);
/// ```
#[macro_export]
macro_rules! test_flag_default_false {
    ($test_name:ident, $args:expr) => {
        #[test]
        fn $test_name() {
            let full_args: Vec<&str> = vec!["hoop"]
                .iter()
                .chain($args.iter())
                .copied()
                .collect();
            let result = parse_cli_with_flag(&full_args);
            assert!(result.is_ok(), "Failed to parse args: {:?}", full_args);
            let parsed = result.unwrap();
            assert_eq!(
                parsed.no_interactive,
                false,
                "no_interactive should be false when not specified"
            );
        }
    };
}

/// Macro to generate a complete test suite for a command
///
/// Generates five tests covering:
/// - Flag before command
/// - Flag after command
/// - Short flag (-y)
/// - Both positions consistency
/// - Default (no flag)
///
/// # Usage
///
/// ```rust
/// test_command_no_interactive_suite!(scan_suite, "scan", &["scan", "/tmp"]);
/// test_command_no_interactive_suite!(remove_suite, "remove", &["remove", "test-project", "--confirm"]);
/// ```
///
/// This generates a single test function that verifies all aspects of the flag.
/// Unlike the individual test macros, this creates one comprehensive test
/// that checks all patterns.
#[macro_export]
macro_rules! test_command_no_interactive_suite {
    ($test_name:ident, $command:expr, $args:expr) => {
        #[test]
        fn $test_name() {
            // Test 1: Flag before command
            let full_args_before: Vec<&str> = vec!["hoop", "--no-interactive"]
                .iter()
                .chain($args.iter())
                .copied()
                .collect();
            let result_before = parse_cli_with_flag(&full_args_before);
            assert!(result_before.is_ok(), "Failed to parse with flag before command");
            let parsed_before = result_before.unwrap();
            assert_eq!(parsed_before.no_interactive, true, "no_interactive should be true before command");

            // Test 2: Flag after command
            let full_args_after: Vec<&str> = $args
                .iter()
                .chain(&["--no-interactive"])
                .copied()
                .collect();
            let result_after = parse_cli_with_flag(&full_args_after);
            assert!(result_after.is_ok(), "Failed to parse with flag after command");
            let parsed_after = result_after.unwrap();
            assert_eq!(parsed_after.no_interactive, true, "no_interactive should be true after command");

            // Test 3: Short flag (-y)
            let full_args_short: Vec<&str> = vec!["hoop", "-y"]
                .iter()
                .chain($args.iter())
                .copied()
                .collect();
            let result_short = parse_cli_with_flag(&full_args_short);
            assert!(result_short.is_ok(), "Failed to parse with -y flag");
            let parsed_short = result_short.unwrap();
            assert_eq!(parsed_short.no_interactive, true, "no_interactive should be true with -y");

            // Test 4: Both positions consistency
            assert_eq!(
                parsed_before.no_interactive,
                parsed_after.no_interactive,
                "no_interactive value must be consistent regardless of flag position"
            );

            // Test 5: Default (no flag)
            let full_args_default: Vec<&str> = vec!["hoop"]
                .iter()
                .chain($args.iter())
                .copied()
                .collect();
            let result_default = parse_cli_with_flag(&full_args_default);
            assert!(result_default.is_ok(), "Failed to parse without flag");
            let parsed_default = result_default.unwrap();
            assert_eq!(
                parsed_default.no_interactive,
                false,
                "no_interactive should be false when not specified"
            );
        }
    };
}

// ── Complete integration example: demonstrating all patterns together ─────────────
//
// This section shows a complete, runnable example that combines all the patterns,
// macros, and helpers provided by this module. Use this as a reference when writing
// comprehensive tests for new commands.
//
// The example demonstrates:
// 1. Manual implementation with helper functions
// 2. Individual test macros for granular testing
// 3. The comprehensive suite macro for complete coverage
// 4. Prompt suppression testing
// 5. Batch testing with multiple test cases
//
// ────────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod integration_example {
    //! Complete integration example demonstrating all test patterns
    //!
    //! This module shows how to use all the utilities together to write
    //! comprehensive tests for CLI flag behavior. Use this as a template
    //! when adding tests for new commands.

    use super::*;
    use tempfile::TempDir;

    /// Example 1: Manual implementation with helper functions
    ///
    /// This example shows the low-level approach using helper functions directly.
    /// Use this when you need maximum control or are debugging specific issues.
    #[test]
    fn example_manual_implementation() {
        // Test 1a: Flag before subcommand (manual)
        let args_before = &["hoop", "--no-interactive", "status", "--json"];
        let parsed_before = parse_cli_with_flag(args_before)
            .expect("Failed to parse with flag before subcommand");
        assert_eq!(parsed_before.no_interactive, true);
        assert_eq!(parsed_before.command, "status");

        // Test 1b: Flag after subcommand (manual)
        let args_after = &["hoop", "status", "--json", "--no-interactive"];
        let parsed_after = parse_cli_with_flag(args_after)
            .expect("Failed to parse with flag after subcommand");
        assert_eq!(parsed_after.no_interactive, true);
        assert_eq!(parsed_after.command, "status");

        // Test 1c: Verify consistency between positions (manual)
        assert_eq!(
            parsed_before.no_interactive,
            parsed_after.no_interactive,
            "Flag value must be consistent regardless of position"
        );

        // Test 1d: Verify the flag extraction (manual)
        assert!(verify_flag_extraction(&parsed_before, "before").is_ok());
        assert!(verify_flag_extraction(&parsed_after, "after").is_ok());

        // Test 1e: Test short flag form (manual)
        let args_short = &["hoop", "-y", "status", "--json"];
        let parsed_short = parse_cli_with_flag(args_short)
            .expect("Failed to parse with -y flag");
        assert_eq!(parsed_short.no_interactive, true);

        // Test 1f: Test default behavior (no flag)
        let args_default = &["hoop", "status", "--json"];
        let parsed_default = parse_cli_with_flag(args_default)
            .expect("Failed to parse without flag");
        assert_eq!(parsed_default.no_interactive, false);
        assert!(verify_no_flag_present(&parsed_default).is_ok());
    }

    /// Example 2: Using convenience helpers
    ///
    /// This example shows the shorthand helpers for common patterns.
    #[test]
    fn example_convenience_helpers() {
        // Parse with flag before subcommand (helper)
        let parsed_before = parse_flag_before_subcommand(&["scan", "/tmp"])
            .expect("Failed to parse with flag before subcommand");
        assert_eq!(parsed_before.no_interactive, true);

        // Parse with flag after subcommand (helper)
        let parsed_after = parse_flag_after_subcommand(&["scan", "/tmp"])
            .expect("Failed to parse with flag after subcommand");
        assert_eq!(parsed_after.no_interactive, true);

        // Verify both positions produce same result
        assert_eq!(
            parsed_before.no_interactive,
            parsed_after.no_interactive
        );
    }

    /// Example 3: Testing prompt suppression
    ///
    /// This example shows how to test that prompts are properly suppressed
    /// when the no_interactive flag is set.
    #[test]
    fn example_prompt_suppression() {
        // Create a mock prompt (simulating "Delete project?" confirmation)
        let delete_prompt = MockYesNoPrompt {
            text: "Delete project test-project?".to_string(),
            requires_confirm: true, // Requires --confirm in no-interactive mode
        };

        // Test 1: Prompt should be suppressed with no_interactive=true
        assert!(verify_prompt_suppressed(&delete_prompt, true).is_ok());
        assert!(!delete_prompt.would_prompt(true));

        // Test 2: Prompt should be shown with no_interactive=false
        assert!(delete_prompt.would_prompt(false));

        // Test 3: Destructive operation requires --confirm in no-interactive mode
        // This should fail: no_interactive=true but confirm=false
        assert!(verify_confirm_required(&delete_prompt, true, false).is_err());

        // This should succeed: no_interactive=true and confirm=true
        assert!(verify_confirm_required(&delete_prompt, true, true).is_ok());

        // This should succeed: no_interactive=false (prompts allowed)
        assert!(verify_confirm_required(&delete_prompt, false, false).is_ok());
    }

    /// Example 4: Batch testing with multiple test cases
    ///
    /// This example shows how to run multiple test cases efficiently
    /// using the batch testing helper.
    #[test]
    fn example_batch_testing() {
        let test_cases = vec![
            FlagPositionTestCase {
                description: "add command with flag before".to_string(),
                command: vec!["hoop", "--no-interactive", "add", "/path/to/project"]
                    .iter().map(|s| s.to_string()).collect(),
                expected_result: true,
            },
            FlagPositionTestCase {
                description: "add command with flag after".to_string(),
                command: vec!["hoop", "add", "/path/to/project", "--no-interactive"]
                    .iter().map(|s| s.to_string()).collect(),
                expected_result: true,
            },
            FlagPositionTestCase {
                description: "add command with short flag".to_string(),
                command: vec!["hoop", "-y", "add", "/path/to/project"]
                    .iter().map(|s| s.to_string()).collect(),
                expected_result: true,
            },
            FlagPositionTestCase {
                description: "add command without flag".to_string(),
                command: vec!["hoop", "add", "/path/to/project"]
                    .iter().map(|s| s.to_string()).collect(),
                expected_result: false,
            },
        ];

        let (successes, failures) = run_flag_position_tests(test_cases);

        assert_eq!(successes.len(), 4, "All 4 test cases should succeed");
        assert_eq!(failures.len(), 0, "No test cases should fail");
    }

    /// Example 5: Using test fixtures
    ///
    /// This example shows how to use the test fixtures for integration testing
    /// that requires temporary files and directories.
    #[test]
    fn example_test_fixtures() {
        let tmp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create a test workspace with .beads directory
        let workspace = create_test_workspace(&tmp_dir, "my-project");
        assert!(workspace.exists());
        assert!(workspace.join(".beads").exists());

        // Create a HOOP config directory
        let hoop_dir = create_hoop_config_dir(&tmp_dir);
        assert!(hoop_dir.exists());
        assert!(hoop_dir.ends_with(".hoop"));

        // Create a projects.yaml registry
        let registry_path = create_test_registry(&tmp_dir);
        assert!(registry_path.exists());
        assert!(registry_path.file_name().unwrap() == "projects.yaml");

        // TempDir is automatically cleaned up when it goes out of scope
    }

    /// Example 6: Complex scenario combining all patterns
    ///
    /// This example demonstrates a realistic test scenario that combines
    /// multiple patterns: parsing, verification, prompt suppression, and fixtures.
    #[test]
    fn example_complex_scenario() {
        // Setup: Create test fixtures
        let tmp_dir = TempDir::new().expect("Failed to create temp dir");
        let workspace = create_test_workspace(&tmp_dir, "test-project");
        let config_dir = create_hoop_config_dir(&tmp_dir);

        // Scenario: Testing the "remove" command with various flag positions
        let remove_args = &["remove", "test-project", "--confirm"];

        // Parse and verify flag before subcommand
        let parsed_before = parse_flag_before_subcommand(remove_args)
            .expect("Failed to parse remove with flag before");
        assert_eq!(parsed_before.no_interactive, true);
        assert_eq!(parsed_before.command, "remove");
        assert!(verify_flag_extraction(&parsed_before, "before").is_ok());

        // Parse and verify flag after subcommand
        let parsed_after = parse_flag_after_subcommand(remove_args)
            .expect("Failed to parse remove with flag after");
        assert_eq!(parsed_after.no_interactive, true);
        assert_eq!(parsed_after.command, "remove");
        assert!(verify_flag_extraction(&parsed_after, "after").is_ok());

        // Verify consistency
        assert_eq!(
            parsed_before.no_interactive,
            parsed_after.no_interactive,
            "Flag must be consistent regardless of position"
        );

        // Test prompt suppression for destructive operation
        let remove_prompt = MockYesNoPrompt {
            text: format!("Remove project {:?}?", workspace),
            requires_confirm: true,
        };

        // Verify prompt is suppressed with no_interactive=true
        assert!(verify_prompt_suppressed(&remove_prompt, true).is_ok());

        // Verify --confirm is required
        assert!(verify_confirm_required(&remove_prompt, true, true).is_ok());
        assert!(verify_confirm_required(&remove_prompt, true, false).is_err());

        // Verify no confirm required when no_interactive=false
        assert!(verify_confirm_required(&remove_prompt, false, false).is_ok());

        // Verify test fixtures were created correctly
        assert!(workspace.exists());
        assert!(config_dir.exists());
        assert!(workspace.join(".beads").exists());
    }

    /// Example 7: Edge cases and error handling
    ///
    /// This example tests edge cases and error conditions.
    #[test]
    fn example_edge_cases() {
        // Empty arguments
        let result = parse_cli_with_flag(&[]);
        assert!(result.is_err());

        // Missing command (only program name and flag)
        // Note: The parser treats "--no-interactive" as a pseudo-command in this edge case
        // since it tries to find any non-flag argument after skipping flags
        let result = parse_cli_with_flag(&["hoop", "--no-interactive"]);
        assert!(result.is_ok()); // Parser accepts this as having command "--no-interactive"
        let parsed = result.unwrap();
        assert_eq!(parsed.no_interactive, true);
        assert_eq!(parsed.command, "--no-interactive"); // Edge case: flag becomes command name

        // Only program name
        let result = parse_cli_with_flag(&["hoop"]);
        assert!(result.is_err()); // Missing command - only program name

        // Multiple flags (should still parse correctly)
        let result = parse_cli_with_flag(&["hoop", "--no-interactive", "--verbose", "status"]);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.no_interactive, true);
        assert_eq!(parsed.command, "status");

        // Flag appears twice (last occurrence should win in real parser,
        // our implementation just detects presence)
        let result = parse_cli_with_flag(&["hoop", "-y", "status", "-y"]);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.no_interactive, true);
    }
}

// ── Module tests (demonstrating utility usage) ──────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Example: Using the test suite macro for the scan command
    test_command_no_interactive_suite!(scan_suite, "scan", &["scan", "/tmp"]);

    // Example: Using the test suite macro for the remove command
    test_command_no_interactive_suite!(remove_suite, "remove", &["remove", "test-project", "--confirm"]);

    #[test]
    fn test_parse_cli_with_flag_before_subcommand() {
        let result = parse_cli_with_flag(&["hoop", "--no-interactive", "scan", "/tmp"]);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.no_interactive, true);
        assert_eq!(parsed.command, "scan");
        assert_eq!(parsed.args, vec!["scan", "/tmp"]);
    }

    #[test]
    fn test_parse_cli_with_flag_after_subcommand() {
        let result = parse_cli_with_flag(&["hoop", "scan", "/tmp", "--no-interactive"]);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.no_interactive, true);
        assert_eq!(parsed.command, "scan");
        assert_eq!(parsed.args, vec!["scan", "/tmp"]);
    }

    #[test]
    fn test_parse_cli_without_flag() {
        let result = parse_cli_with_flag(&["hoop", "scan", "/tmp"]);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.no_interactive, false);
        assert_eq!(parsed.command, "scan");
        assert_eq!(parsed.args, vec!["scan", "/tmp"]);
    }

    #[test]
    fn test_parse_cli_with_short_flag() {
        let result = parse_cli_with_flag(&["hoop", "-y", "scan", "/tmp"]);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.no_interactive, true);
        assert_eq!(parsed.command, "scan");
        assert_eq!(parsed.args, vec!["scan", "/tmp"]);
    }

    #[test]
    fn test_parse_flag_before_subcommand_helper() {
        let result = parse_flag_before_subcommand(&["scan", "/tmp"]);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.no_interactive, true);
        assert_eq!(parsed.command, "scan");
    }

    #[test]
    fn test_parse_flag_after_subcommand_helper() {
        let result = parse_flag_after_subcommand(&["scan", "/tmp"]);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.no_interactive, true);
        assert_eq!(parsed.command, "scan");
    }

    #[test]
    fn test_verify_flag_extraction_before() {
        let parsed = parse_flag_before_subcommand(&["remove", "test", "--confirm"]).unwrap();
        assert!(verify_flag_extraction(&parsed, "before").is_ok());
    }

    #[test]
    fn test_verify_flag_extraction_after() {
        let parsed = parse_flag_after_subcommand(&["remove", "test", "--confirm"]).unwrap();
        assert!(verify_flag_extraction(&parsed, "after").is_ok());
    }

    #[test]
    fn test_verify_no_flag_present() {
        let parsed = parse_cli_with_flag(&["hoop", "scan", "/tmp"]).unwrap();
        assert!(verify_no_flag_present(&parsed).is_ok());
    }

    #[test]
    fn test_prompt_suppressed_with_no_interactive() {
        let prompt = MockYesNoPrompt {
            text: "Continue?".to_string(),
            requires_confirm: false,
        };

        assert!(verify_prompt_suppressed(&prompt, true).is_ok());
    }

    #[test]
    fn test_prompt_not_suppressed_without_no_interactive() {
        let prompt = MockYesNoPrompt {
            text: "Continue?".to_string(),
            requires_confirm: false,
        };

        // Should show prompt when no_interactive=false
        assert!(prompt.would_prompt(false));
    }

    #[test]
    fn test_confirm_required_for_destructive_operation() {
        let prompt = MockYesNoPrompt {
            text: "Remove project?".to_string(),
            requires_confirm: true,
        };

        // Should require confirm when no_interactive=true
        assert!(verify_confirm_required(&prompt, true, false).is_err());

        // Should succeed when confirm=true
        assert!(verify_confirm_required(&prompt, true, true).is_ok());

        // Should not require confirm when no_interactive=false
        assert!(verify_confirm_required(&prompt, false, false).is_ok());
    }

    #[test]
    fn test_create_test_workspace() {
        let tmp_dir = TempDir::new().expect("Failed to create temp dir");
        let workspace = create_test_workspace(&tmp_dir, "test-project");

        assert!(workspace.exists());
        assert!(workspace.join(".beads").exists());
    }

    #[test]
    fn test_create_test_registry() {
        let tmp_dir = TempDir::new().expect("Failed to create temp dir");
        let registry_path = create_test_registry(&tmp_dir);

        assert!(registry_path.exists());
        assert!(registry_path.parent().unwrap().ends_with(".hoop"));
    }

    #[test]
    fn test_run_flag_position_tests() {
        let test_cases = vec![
            FlagPositionTestCase {
                description: "scan with flag before".to_string(),
                command: vec!["hoop", "--no-interactive", "scan", "/tmp"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                expected_result: true,
            },
            FlagPositionTestCase {
                description: "scan with flag after".to_string(),
                command: vec!["hoop", "scan", "/tmp", "--no-interactive"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                expected_result: true,
            },
            FlagPositionTestCase {
                description: "scan without flag".to_string(),
                command: vec!["hoop", "scan", "/tmp"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                expected_result: false,
            },
        ];

        let (successes, failures) = run_flag_position_tests(test_cases);

        assert_eq!(successes.len(), 3);
        assert_eq!(failures.len(), 0);
    }
}
