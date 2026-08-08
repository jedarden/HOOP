//! CLI Test Helpers for `no_interactive` Flag Testing
//!
//! This module provides high-level test helpers and patterns for testing the
//! `--no-interactive` flag across HOOP CLI commands. It complements the lower-level
//! utilities in `cli_test_utils.rs` by providing command-specific testing patterns
//! and flag parsing utilities.
//!
//! # Why Test `no_interactive` Flag at Different Positions?
//!
//! The `--no-interactive` flag is unique in HOOP's CLI design because it can be
//! specified at two different positions in the command line:
//!
//! 1. **Before the subcommand**: `hoop --no-interactive <command> [args]`
//! 2. **After the subcommand**: `hoop <command> [args] --no-interactive`
//!
//! This dual-position behavior is implemented via `clap`'s `global` flag setting,
//! which allows the flag to appear anywhere in the command line. Testing both
//! positions is critical because:
//!
//! - **User ergonomics**: Different users have different muscle memory for flag placement
//! - **CI/CD scripts**: Automated scripts may place flags differently based on generation patterns
//! - **Consistency**: Both positions must extract the same boolean value
//! - **Composability**: Subcommands with their own flags must not interfere with `no_interactive`
//!
//! # Flag Parsing Utilities
//!
//! This module provides utilities for parsing clap command structures with flags
//! at different positions:
//!
//! ## Available Functions
//!
//! - **`parse_flag_before_subcommand()`** - Parses commands with flag before the subcommand
//! - **`parse_flag_after_subcommand()`** - Parses commands with flag after the subcommand
//! - **`parse_nested_subcommand()`** - Parses nested subcommand structures (e.g., `projects remove`)
//! - **`extract_flag_value()`** - Convenience function to extract only the boolean flag value
//! - **`extract_subcommand()`** - Convenience function to extract only the subcommand name
//! - **`verify_flag_position_consistency()`** - Verifies flag parsing is consistent between positions
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! use cli_test_helpers::prelude::*;
//!
//! #[test]
//! fn test_scan_flag_positions() {
//!     // Parse with flag before subcommand
//!     let before = parse_flag_before_subcommand(&["--no-interactive", "scan", "/tmp"]).unwrap();
//!     assert_eq!(before.no_interactive, true);
//!     assert_eq!(before.subcommand, Some("scan".to_string()));
//!
//!     // Parse with flag after subcommand
//!     let after = parse_flag_after_subcommand(&["scan", "/tmp", "--no-interactive"]).unwrap();
//!     assert_eq!(after.no_interactive, true);
//!     assert_eq!(after.subcommand, Some("scan".to_string()));
//!
//!     // Verify consistency
//!     assert!(verify_flag_position_consistency(&["scan", "/tmp"]).is_ok());
//! }
//! ```
//!
//! # Testing Approach
//!
//! ## 1. Flag Position Testing
//!
//! For every command that supports `--no-interactive`, we verify:
//!
//! ```rust,ignore
//! # Before subcommand
//! hoop --no-interactive scan /tmp
//!
//! # After subcommand
//! hoop scan /tmp --no-interactive
//!
//! # Short flag variant
//! hoop -y scan /tmp
//! ```
//!
//! ## 2. Flag Extraction Verification
//!
//! We verify that the flag is correctly extracted from the parsed arguments and
//! that the boolean value is consistent regardless of position.
//!
//! ## 3. Prompt Suppression Testing
//!
//! When `--no-interactive` is true, commands must suppress all user prompts.
//! This includes:
//!
//! - Confirmation prompts ("Continue? [y/N]")
//! - Selection prompts ("Choose a workspace:")
//! - Input prompts ("Enter a name:")
//!
//! ## 4. Destructive Operation Testing
//!
//! Destructive operations (remove, delete, etc.) require an additional `--confirm`
//! flag when `--no-interactive` is set:
//!
//! ```rust,ignore
//! # Must fail - destructive without --confirm
//! hoop --no-interactive remove my-project
//!
//! # Must succeed - explicit --confirm
//! hoop --no-interactive remove my-project --confirm
//! ```
//!
//! # Clap Command Patterns We Test
//!
//! HOOP uses `clap` for CLI parsing with the following patterns:
//!
//! ## Top-Level Command Structure
//!
//! ```rust
//! #[derive(Parser, Debug)]
//! #[command(name = "hoop")]
//! #[command(about = "HOOP CLI")]
//! struct Cli {
//!     /// Run in non-interactive mode (no prompts)
//!     #[arg(short = 'y', long = "no-interactive", global = true)]
//!     no_interactive: bool,
//!
//!     #[command(subcommand)]
//!     command: Option<Commands>,
//! }
//! ```
//!
//! The `global = true` attribute is what enables the dual-position behavior.
//!
//! ## Subcommand Patterns
//!
//! Each subcommand (scan, init, remove, etc.) receives the `no_interactive` value
//! via propagation from the top-level `Cli` struct:
//!
//! ```rust
//! enum Commands {
//!     #[command(about = "Scan a project")]
//!     Scan {
//!         /// Whether to run in non-interactive mode
//!         #[arg(short = 'y', long = "no-interactive", global = true)]
//!         no_interactive: bool,
//!         path: String,
//!     },
//!     // ... other commands
//! }
//! ```
//!
//! # How This Helper Module Is Used
//!
//! This module provides two categories of helpers:
//!
//! ## 1. Command-Specific Test Builders
//!
//! Functions that build complete test cases for specific commands:
//!
//! ```rust,ignore
//! use cli_test_helpers::*;
//!
//! #[test]
//! fn test_scan_no_interactive() {
//!     let test = ScanNoInteractiveTest::new()
//!         .with_flag_before()
//!         .expect_prompt_suppression();
//!
//!     test.run();
//! }
//! ```
//!
//! ## 2. Reusable Test Patterns
//!
//! Macros and functions that generate common test patterns:
//!
//! ```rust,ignore
//! test_no_interactive_flag_positions!(scan, "scan", &["scan", "/tmp"]);
//! ```
//!
//! # Future Utility Sections
//!
//! This module is structured to grow with the following planned sections:
//!
//! - **Command-specific builders**: `ScanNoInteractiveTest`, `InitNoInteractiveTest`, etc.
//! - **Mock prompt factories**: Builders for creating mock prompt interfaces
//! - **Test fixtures**: Pre-built test workspaces and configs for common scenarios
//! - **Assertion helpers**: Custom assertions for `no_interactive` behavior
//! - **Integration test patterns**: End-to-end test patterns that run actual commands
//!
//! # Module Organization
//!
//! ```text
//! cli_test_helpers/
//! ├── mod.rs                    # This file - module overview and common patterns
//! ├── command_builders.rs       # Command-specific test builders (planned)
//! ├── mock_prompts.rs           # Mock prompt interfaces (planned)
//! ├── test_fixtures.rs          # Reusable test fixtures (planned)
//! └── assertions.rs             # Custom assertion helpers (planned)
//! ```
//!
//! # Examples
//!
//! ## Basic Flag Position Test
//!
//! ```rust,ignore
//! use cli_test_helpers::prelude::*;
//!
//! #[test]
//! fn test_scan_flag_positions() {
//!     // Test flag before subcommand
//!     let parsed_before = parse_with_flag_before(&["scan", "/tmp"]);
//!     assert!(parsed_before.is_ok());
//!     assert_eq!(parsed_before.unwrap().no_interactive, true);
//!
//!     // Test flag after subcommand
//!     let parsed_after = parse_with_flag_after(&["scan", "/tmp"]);
//!     assert!(parsed_after.is_ok());
//!     assert_eq!(parsed_after.unwrap().no_interactive, true);
//!
//!     // Verify consistency
//!     assert_eq!(
//!         parsed_before.unwrap().no_interactive,
//!         parsed_after.unwrap().no_interactive
//!     );
//! }
//! ```
//!
//! ## Prompt Suppression Test
//!
//! ```rust,ignore
//! use cli_test_helpers::prompt::*;
//!
//! #[test]
//! fn test_init_prompt_suppression() {
//!     let prompt = MockInitPrompt::new();
//!
//!     // With --no-interactive, prompt should be suppressed
//!     assert!(prompt.is_suppressed_when(no_interactive: true));
//!
//!     // Without --no-interactive, prompt should be shown
//!     assert!(!prompt.is_suppressed_when(no_interactive: false));
//! }
//! ```
//!
//! # See Also
//!
//! - `cli_test_utils.rs` - Low-level parsing and verification utilities
//! - `init_no_interactive_flag.rs` - Integration tests for the init command
//! - `no_interactive_flag_behavior.rs` - Comprehensive flag behavior tests

// ── Prelude Module (re-exports for convenience) ─────────────────────────────

/// Common imports for test helpers
///
/// Note: This prelude is intentionally minimal since cli_test_helpers is a new module.
/// Future expansions will add more re-exports as command-specific builders are implemented.
pub mod prelude {
    // Re-export the flag constants for easy access
    pub use super::{flags, commands};

    // Re-export the flag parsing utilities
    pub use super::{
        parse_flag_before_subcommand,
        parse_flag_after_subcommand,
        parse_nested_subcommand,
        extract_flag_value,
        extract_subcommand,
        verify_flag_position_consistency,
        FlagParseResult,
    };

    // Re-export the flag verification utilities
    pub use super::{
        assert_flag_is_true,
        assert_flag_is_false,
        assert_flag_value,
        assert_flag_propagation,
        compare_flag_values_at_levels,
        verify_default_flag_value,
    };

    // Re-export the placeholder builders (will be replaced with real implementations)
    pub use super::{command_builders, mock_prompts, test_fixtures, assertions};
}

// ── Test Constants ───────────────────────────────────────────────────────────

/// Common flag variants used in testing
pub mod flags {
    /// Long form: `--no-interactive`
    pub const LONG: &str = "--no-interactive";

    /// Short form: `-y`
    pub const SHORT: &str = "-y";

    /// All flag variants for comprehensive testing
    pub const ALL: &[&str] = &[LONG, SHORT];
}

/// Common command names for testing
pub mod commands {
    pub const SCAN: &str = "scan";
    pub const INIT: &str = "init";
    pub const REMOVE: &str = "remove";
    pub const STATUS: &str = "status";
    pub const LIST: &str = "list";
    pub const PROJECTS: &str = "projects";
    pub const PATTERNS: &str = "patterns";
    pub const ADD: &str = "add";
    pub const SHOW: &str = "show";

    /// All known top-level subcommands
    pub const TOP_LEVEL: &[&str] = &[SCAN, INIT, REMOVE, STATUS, LIST, PROJECTS, PATTERNS];

    /// All known nested subcommands
    pub const NESTED: &[&str] = &[REMOVE, ADD, SHOW];

    /// Check if a string is a known top-level subcommand
    pub fn is_top_level(cmd: &str) -> bool {
        TOP_LEVEL.contains(&cmd)
    }

    /// Check if a string is a known nested subcommand
    pub fn is_nested(cmd: &str) -> bool {
        NESTED.contains(&cmd)
    }
}

// ── Flag Parsing Utilities ─────────────────────────────────────────────────────

/// Result of parsing a clap command with flag position information
#[derive(Debug, Clone)]
pub struct FlagParseResult {
    /// Whether the no_interactive flag was present
    pub no_interactive: bool,
    /// The primary subcommand name (e.g., "scan", "projects")
    pub subcommand: Option<String>,
    /// Nested subcommand if present (e.g., "remove" in "projects remove")
    pub nested_subcommand: Option<String>,
    /// All arguments excluding the program name
    pub args: Vec<String>,
    /// Raw argument vector as provided
    pub raw_args: Vec<String>,
}

/// Parse a command with the flag positioned BEFORE the subcommand
///
/// This utility handles the pattern: `hoop --no-interactive <command> [args]`
///
/// # What This Handles
///
/// - Extracts the global `--no-interactive` flag from the top-level position
/// - Identifies the primary subcommand that follows
/// - Preserves nested subcommands and their arguments
/// - Returns structured data for test assertions
///
/// # Edge Cases
///
/// - Handles both `--no-interactive` and `-y` short form
/// - Gracefully handles missing subcommands (returns `None`)
/// - Preserves all remaining arguments for validation
/// - Handles commands with additional flags after the subcommand
///
/// # Examples
///
/// ```rust,ignore
/// use cli_test_helpers::*;
///
/// // Simple command
/// let result = parse_flag_before_subcommand(&["--no-interactive", "scan", "/tmp"]);
/// assert!(result.is_ok());
/// let parsed = result.unwrap();
/// assert_eq!(parsed.no_interactive, true);
/// assert_eq!(parsed.subcommand, Some("scan".to_string()));
///
/// // Nested subcommand
/// let result = parse_flag_before_subcommand(&["--no-interactive", "projects", "remove", "my-project", "--confirm"]);
/// assert!(result.is_ok());
/// let parsed = result.unwrap();
/// assert_eq!(parsed.no_interactive, true);
/// assert_eq!(parsed.subcommand, Some("projects".to_string()));
/// assert_eq!(parsed.nested_subcommand, Some("remove".to_string()));
///
/// // Short flag form
/// let result = parse_flag_before_subcommand(&["-y", "status", "--json"]);
/// assert!(result.is_ok());
/// let parsed = result.unwrap();
/// assert_eq!(parsed.no_interactive, true);
/// ```
pub fn parse_flag_before_subcommand(args: &[&str]) -> Result<FlagParseResult, String> {
    if args.is_empty() {
        return Err("No arguments provided".to_string());
    }

    let raw_args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    // Check for no_interactive flag in both forms at position 0
    let no_interactive = match args.get(0) {
        Some(&"--no-interactive" | &"-y") => true,
        _ => false,
    };

    // Collect all non-flag arguments in order
    let non_flag_args: Vec<&str> = args
        .iter()
        .skip(if no_interactive { 1 } else { 0 })
        .filter(|&&a| !a.starts_with('-'))
        .copied()
        .collect();

    // Find the primary subcommand (first known command)
    let subcommand = non_flag_args
        .iter()
        .find(|&&s| commands::is_top_level(s))
        .map(|s| s.to_string());

    // Find nested subcommand (second known command after the primary)
    let nested_subcommand = if subcommand.is_some() {
        non_flag_args
            .iter()
            .skip_while(|&&s| Some(s.to_string()) != subcommand)
            .skip(1)
            .find(|&&s| commands::is_nested(s))
            .map(|s| s.to_string())
    } else {
        None
    };

    // Collect all arguments excluding program name (which isn't in this slice)
    let all_args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    Ok(FlagParseResult {
        no_interactive,
        subcommand,
        nested_subcommand,
        args: all_args,
        raw_args,
    })
}

/// Parse a command with the flag positioned AFTER the subcommand
///
/// This utility handles the pattern: `hoop <command> [args] --no-interactive`
///
/// # What This Handles
///
/// - Extracts the global `--no-interactive` flag from the end position
/// - Identifies the primary subcommand that precedes it
/// - Preserves nested subcommands and their arguments
/// - Returns structured data for test assertions
///
/// # Edge Cases
///
/// - Handles both `--no-interactive` and `-y` short form
/// - Handles flags mixed with command arguments
/// - Gracefully handles missing subcommands (returns `None`)
/// - Supports multiple flags after the command (e.g., `scan /tmp --no-interactive --json`)
///
/// # Examples
///
/// ```rust,ignore
/// use cli_test_helpers::*;
///
/// // Simple command with flag at end
/// let result = parse_flag_after_subcommand(&["scan", "/tmp", "--no-interactive"]);
/// assert!(result.is_ok());
/// let parsed = result.unwrap();
/// assert_eq!(parsed.no_interactive, true);
/// assert_eq!(parsed.subcommand, Some("scan".to_string()));
///
/// // Nested subcommand with flag at end
/// let result = parse_flag_after_subcommand(&["projects", "remove", "my-project", "--confirm", "--no-interactive"]);
/// assert!(result.is_ok());
/// let parsed = result.unwrap();
/// assert_eq!(parsed.no_interactive, true);
/// assert_eq!(parsed.subcommand, Some("projects".to_string()));
/// assert_eq!(parsed.nested_subcommand, Some("remove".to_string()));
///
/// // Short flag form
/// let result = parse_flag_after_subcommand(&["status", "-y"]);
/// assert!(result.is_ok());
/// let parsed = result.unwrap();
/// assert_eq!(parsed.no_interactive, true);
/// ```
pub fn parse_flag_after_subcommand(args: &[&str]) -> Result<FlagParseResult, String> {
    if args.is_empty() {
        return Err("No arguments provided".to_string());
    }

    let raw_args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    // Check for no_interactive flag anywhere in the args
    let no_interactive = args
        .iter()
        .any(|&a| a == "--no-interactive" || a == "-y");

    // Collect all non-flag arguments in order
    let non_flag_args: Vec<&str> = args
        .iter()
        .filter(|&&a| !a.starts_with('-'))
        .copied()
        .collect();

    // Find the primary subcommand (first known command)
    let subcommand = non_flag_args
        .iter()
        .find(|&&s| commands::is_top_level(s))
        .map(|s| s.to_string());

    // Find nested subcommand (second known command after the primary)
    let nested_subcommand = if subcommand.is_some() {
        non_flag_args
            .iter()
            .skip_while(|&&s| Some(s.to_string()) != subcommand)
            .skip(1)
            .find(|&&s| commands::is_nested(s))
            .map(|s| s.to_string())
    } else {
        None
    };

    // Collect all arguments excluding program name
    let all_args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    Ok(FlagParseResult {
        no_interactive,
        subcommand,
        nested_subcommand,
        args: all_args,
        raw_args,
    })
}

/// Parse a command with nested subcommand structure
///
/// This utility handles commands with two-level subcommand structures like:
/// `hoop projects remove <name>` or `hoop patterns add <name>`
///
/// # What This Handles
///
/// - Correctly identifies two-level subcommand hierarchies
/// - Extracts the primary and nested subcommand names
/// - Handles `no_interactive` flag at any position (before, between, or after)
/// - Returns structured data for testing nested command structures
///
/// # Edge Cases
///
/// - Handles flag at any position (before primary, between primary/nested, after nested)
/// - Supports commands with only one level (nested_subcommand will be `None`)
/// - Handles additional flags mixed with subcommands
/// - Gracefully handles malformed command structures
///
/// # Examples
///
/// ```rust,ignore
/// use cli_test_helpers::*;
///
/// // Flag before primary subcommand
/// let result = parse_nested_subcommand(&["--no-interactive", "projects", "remove", "my-project", "--confirm"]);
/// assert!(result.is_ok());
/// let parsed = result.unwrap();
/// assert_eq!(parsed.no_interactive, true);
/// assert_eq!(parsed.subcommand, Some("projects".to_string()));
/// assert_eq!(parsed.nested_subcommand, Some("remove".to_string()));
///
/// // Flag after nested subcommand
/// let result = parse_nested_subcommand(&["patterns", "add", "my-pattern", "--no-interactive"]);
/// assert!(result.is_ok());
/// let parsed = result.unwrap();
/// assert_eq!(parsed.no_interactive, true);
/// assert_eq!(parsed.subcommand, Some("patterns".to_string()));
/// assert_eq!(parsed.nested_subcommand, Some("add".to_string()));
///
/// // Single-level command (no nesting)
/// let result = parse_nested_subcommand(&["scan", "/tmp", "-y"]);
/// assert!(result.is_ok());
/// let parsed = result.unwrap();
/// assert_eq!(parsed.no_interactive, true);
/// assert_eq!(parsed.subcommand, Some("scan".to_string()));
/// assert_eq!(parsed.nested_subcommand, None);
/// ```
pub fn parse_nested_subcommand(args: &[&str]) -> Result<FlagParseResult, String> {
    if args.is_empty() {
        return Err("No arguments provided".to_string());
    }

    let raw_args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    // Check for no_interactive flag anywhere in the args
    let no_interactive = args
        .iter()
        .any(|&a| a == "--no-interactive" || a == "-y");

    // Collect all non-flag arguments in order
    let non_flag_args: Vec<&str> = args
        .iter()
        .filter(|&&a| !a.starts_with('-'))
        .copied()
        .collect();

    // Find the primary subcommand (first known command)
    let subcommand = non_flag_args
        .iter()
        .find(|&&s| commands::is_top_level(s))
        .map(|s| s.to_string());

    // Find nested subcommand (second known command after the primary)
    let nested_subcommand = if subcommand.is_some() {
        non_flag_args
            .iter()
            .skip_while(|&&s| Some(s.to_string()) != subcommand)
            .skip(1)
            .find(|&&s| commands::is_nested(s))
            .map(|s| s.to_string())
    } else {
        None
    };

    // Collect all arguments excluding program name
    let all_args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    Ok(FlagParseResult {
        no_interactive,
        subcommand,
        nested_subcommand,
        args: all_args,
        raw_args,
    })
}

/// Extract only the boolean flag value from parsed arguments
///
/// This is a convenience function for tests that only need to verify
/// the flag value without examining the full command structure.
///
/// # Arguments
///
/// * `args` - Command line arguments as a slice of strings
///
/// # Returns
///
/// * `bool` - Whether the no_interactive flag was present
///
/// # Examples
///
/// ```rust,ignore
/// use cli_test_helpers::*;
///
/// // Flag present
/// let has_flag = extract_flag_value(&["scan", "/tmp", "--no-interactive"]);
/// assert_eq!(has_flag, true);
///
/// // Flag absent
/// let has_flag = extract_flag_value(&["scan", "/tmp"]);
/// assert_eq!(has_flag, false);
///
/// // Short form
/// let has_flag = extract_flag_value(&["-y", "scan", "/tmp"]);
/// assert_eq!(has_flag, true);
/// ```
pub fn extract_flag_value(args: &[&str]) -> bool {
    args.iter()
        .any(|&a| a == "--no-interactive" || a == "-y")
}

/// Extract the subcommand name from parsed arguments
///
/// This is a convenience function for tests that only need to verify
/// which subcommand was invoked without checking the flag or other details.
///
/// # Arguments
///
/// * `args` - Command line arguments as a slice of strings
///
/// # Returns
///
/// * `Option<String>` - The primary subcommand name, or None if not found
///
/// # Examples
///
/// ```rust,ignore
/// use cli_test_helpers::*;
///
/// let cmd = extract_subcommand(&["scan", "/tmp", "--no-interactive"]);
/// assert_eq!(cmd, Some("scan".to_string()));
///
/// let cmd = extract_subcommand(&["projects", "remove", "my-project"]);
/// assert_eq!(cmd, Some("projects".to_string()));
///
/// let cmd = extract_subcommand(&["--no-interactive"]);
/// assert_eq!(cmd, None);
/// ```
pub fn extract_subcommand(args: &[&str]) -> Option<String> {
    args.iter()
        .find(|&&a| !a.starts_with('-'))
        .map(|s| s.to_string())
}

/// Verify that flag parsing is consistent between two positions
///
/// This utility tests that the no_interactive flag value is extracted
/// consistently regardless of whether it appears before or after the subcommand.
///
/// # Arguments
///
/// * `command_args` - The base command arguments without the flag
///
/// # Returns
///
/// * `Result<(), String>` - Ok if both positions parse consistently, Err with details
///
/// # Examples
///
/// ```rust,ignore
/// use cli_test_helpers::*;
///
/// // Should succeed - both positions give same result
/// let result = verify_flag_position_consistency(&["scan", "/tmp"]);
/// assert!(result.is_ok());
///
/// // This will internally verify:
/// // 1. parse_flag_before_subcommand() returns no_interactive=true
/// // 2. parse_flag_after_subcommand() returns no_interactive=true
/// // 3. Both return the same subcommand name
/// // 4. Both return the same nested_subcommand (if any)
/// ```
pub fn verify_flag_position_consistency(command_args: &[&str]) -> Result<(), String> {
    // Parse with flag before subcommand
    let before_result = parse_flag_before_subcommand(
        &["--no-interactive"]
            .iter()
            .chain(command_args.iter())
            .copied()
            .collect::<Vec<_>>()
    )?;

    // Parse with flag after subcommand
    let after_result = parse_flag_after_subcommand(
        &command_args
            .iter()
            .chain(&["--no-interactive"])
            .copied()
            .collect::<Vec<_>>()
    )?;

    // Verify no_interactive is the same
    if before_result.no_interactive != after_result.no_interactive {
        return Err(format!(
            "Flag value differs: before={}, after={}",
            before_result.no_interactive, after_result.no_interactive
        ));
    }

    // Verify subcommand is the same
    if before_result.subcommand != after_result.subcommand {
        return Err(format!(
            "Subcommand differs: before={:?}, after={:?}",
            before_result.subcommand, after_result.subcommand
        ));
    }

    // Verify nested_subcommand is the same
    if before_result.nested_subcommand != after_result.nested_subcommand {
        return Err(format!(
            "Nested subcommand differs: before={:?}, after={:?}",
            before_result.nested_subcommand, after_result.nested_subcommand
        ));
    }

    Ok(())
}

// ── Flag Verification Utilities ─────────────────────────────────────────────────────

/// Assert that the no_interactive flag is true
///
/// This utility verifies that the parsed result has `no_interactive = true`.
/// It returns a detailed error message if the assertion fails.
///
/// # Arguments
///
/// * `parsed` - The FlagParseResult to check
///
/// # Returns
///
/// * `Result<(), String>` - Ok if flag is true, Err with details if not
///
/// # Examples
///
/// ```rust,ignore
/// use cli_test_helpers::*;
///
/// let result = parse_flag_before_subcommand(&["--no-interactive", "scan", "/tmp"]).unwrap();
/// assert!(assert_flag_is_true(&result).is_ok());
/// ```
pub fn assert_flag_is_true(parsed: &FlagParseResult) -> Result<(), String> {
    if !parsed.no_interactive {
        return Err(format!(
            "Expected no_interactive flag to be true, but it was false. Args: {:?}",
            parsed.raw_args
        ));
    }
    Ok(())
}

/// Assert that the no_interactive flag is false
///
/// This utility verifies that the parsed result has `no_interactive = false`.
/// It returns a detailed error message if the assertion fails.
///
/// # Arguments
///
/// * `parsed` - The FlagParseResult to check
///
/// # Returns
///
/// * `Result<(), String>` - Ok if flag is false, Err with details if not
///
/// # Examples
///
/// ```rust,ignore
/// use cli_test_helpers::*;
///
/// let result = parse_flag_before_subcommand(&["scan", "/tmp"]).unwrap();
/// assert!(assert_flag_is_false(&result).is_ok());
/// ```
pub fn assert_flag_is_false(parsed: &FlagParseResult) -> Result<(), String> {
    if parsed.no_interactive {
        return Err(format!(
            "Expected no_interactive flag to be false, but it was true. Args: {:?}",
            parsed.raw_args
        ));
    }
    Ok(())
}

/// Assert that the no_interactive flag has a specific expected value
///
/// This is a convenience function that combines `assert_flag_is_true` and
/// `assert_flag_is_false` into a single call.
///
/// # Arguments
///
/// * `parsed` - The FlagParseResult to check
/// * `expected` - The expected boolean value
///
/// # Returns
///
/// * `Result<(), String>` - Ok if flag matches expected, Err with details if not
///
/// # Examples
///
/// ```rust,ignore
/// use cli_test_helpers::*;
///
/// let result = parse_flag_before_subcommand(&["--no-interactive", "scan", "/tmp"]).unwrap();
/// assert!(assert_flag_value(&result, true).is_ok());
///
/// let result = parse_flag_before_subcommand(&["scan", "/tmp"]).unwrap();
/// assert!(assert_flag_value(&result, false).is_ok());
/// ```
pub fn assert_flag_value(parsed: &FlagParseResult, expected: bool) -> Result<(), String> {
    if expected {
        assert_flag_is_true(parsed)
    } else {
        assert_flag_is_false(parsed)
    }
}

/// Assert that the flag propagates correctly from top-level to subcommand
///
/// This utility verifies that when a flag is specified at the top level,
/// it correctly propagates to the subcommand level. It tests both positions
/// (before and after the subcommand) and verifies consistency.
///
/// # Arguments
///
/// * `command_args` - The base command arguments without the flag
///
/// # Returns
///
/// * `Result<(), String>` - Ok if propagation is correct, Err with details if not
///
/// # Examples
///
/// ```rust,ignore
/// use cli_test_helpers::*;
///
/// // Test that flag propagates correctly for scan command
/// let result = assert_flag_propagation(&["scan", "/tmp"]);
/// assert!(result.is_ok());
/// ```
pub fn assert_flag_propagation(command_args: &[&str]) -> Result<(), String> {
    // Parse with flag before subcommand (top-level position)
    let before_result = parse_flag_before_subcommand(
        &["--no-interactive"]
            .iter()
            .chain(command_args.iter())
            .copied()
            .collect::<Vec<_>>()
    )?;

    // Parse with flag after subcommand (subcommand-level position)
    let after_result = parse_flag_after_subcommand(
        &command_args
            .iter()
            .chain(&["--no-interactive"])
            .copied()
            .collect::<Vec<_>>()
    )?;

    // Both should have no_interactive=true
    assert_flag_is_true(&before_result)?;
    assert_flag_is_true(&after_result)?;

    // Verify the subcommands match
    if before_result.subcommand != after_result.subcommand {
        return Err(format!(
            "Flag propagation failed: subcommands differ. before={:?}, after={:?}",
            before_result.subcommand, after_result.subcommand
        ));
    }

    // Verify nested subcommands match (if present)
    if before_result.nested_subcommand != after_result.nested_subcommand {
        return Err(format!(
            "Flag propagation failed: nested subcommands differ. before={:?}, after={:?}",
            before_result.nested_subcommand, after_result.nested_subcommand
        ));
    }

    Ok(())
}

/// Compare flag values at different parsing levels
///
/// This utility parses the same command at different levels and verifies
/// that the flag value is consistent across all parsing approaches.
///
/// # Arguments
///
/// * `args` - Command line arguments as a slice of strings
///
/// # Returns
///
/// * `Result<(), String>` - Ok if all levels extract the same value, Err with details if not
///
/// # Examples
///
/// ```rust,ignore
/// use cli_test_helpers::*;
///
/// // Test that all parsing levels give consistent flag value
/// let result = compare_flag_values_at_levels(&["scan", "/tmp", "--no-interactive"]);
/// assert!(result.is_ok());
/// ```
pub fn compare_flag_values_at_levels(args: &[&str]) -> Result<(), String> {
    // Method 1: Extract directly using extract_flag_value
    let direct_flag = extract_flag_value(args);

    // Collect non-flag args (command and arguments)
    let non_flag_args: Vec<&str> = args
        .iter()
        .filter(|&&a| !a.starts_with('-'))
        .copied()
        .collect();

    // Method 2: Parse with flag_before_subcommand approach
    // This always expects flag at position 0, so we construct it that way
    let before_args = ["--no-interactive"]
        .iter()
        .chain(non_flag_args.iter())
        .copied()
        .collect::<Vec<_>>();
    let before_result = parse_flag_before_subcommand(&before_args);

    // Method 3: Parse with flag_after_subcommand approach
    // This finds flag anywhere, so we can construct it with flag at end
    let after_args = non_flag_args
        .iter()
        .chain(&["--no-interactive"])
        .copied()
        .collect::<Vec<_>>();
    let after_result = parse_flag_after_subcommand(&after_args);

    // Method 4: Parse nested subcommand (use original args as-is)
    let nested_result = parse_nested_subcommand(args);

    // Compare: direct and nested should match the original intent
    // before and after should both have flag=true (since we added it)
    let mut errors = Vec::new();

    // Check that before_subcommand found the flag we added
    if let Ok(before) = &before_result {
        if !before.no_interactive {
            errors.push(format!(
                "before_subcommand should have flag=true (we added it at position 0), got {}",
                before.no_interactive
            ));
        }
    } else {
        errors.push(format!("before_subcommand parsing failed: {:?}", before_result));
    }

    // Check that after_subcommand found the flag we added
    if let Ok(after) = &after_result {
        if !after.no_interactive {
            errors.push(format!(
                "after_subcommand should have flag=true (we added it at end), got {}",
                after.no_interactive
            ));
        }
    } else {
        errors.push(format!("after_subcommand parsing failed: {:?}", after_result));
    }

    // Check that direct extraction and nested agree on the original args
    if let Ok(nested) = &nested_result {
        if nested.no_interactive != direct_flag {
            errors.push(format!(
                "Direct extraction ({}) != nested_subcommand ({}) - these should agree on original args",
                direct_flag, nested.no_interactive
            ));
        }
    } else {
        errors.push(format!("nested_subcommand parsing failed: {:?}", nested_result));
    }

    if !errors.is_empty() {
        return Err(format!(
            "Flag value inconsistency across parsing levels:\n{}",
            errors.join("\n")
        ));
    }

    Ok(())
}

/// Verify that the flag defaults to false when not present
///
/// This utility tests the default behavior of the no_interactive flag,
/// ensuring that when the flag is not specified, it defaults to false.
///
/// # Arguments
///
/// * `command_args` - Command arguments without the no_interactive flag
///
/// # Returns
///
/// * `Result<(), String>` - Ok if flag defaults to false, Err with details if not
///
/// # Examples
///
/// ```rust,ignore
/// use cli_test_helpers::*;
///
/// // Test that flag defaults to false
/// let result = verify_default_flag_value(&["scan", "/tmp"]);
/// assert!(result.is_ok());
/// ```
pub fn verify_default_flag_value(command_args: &[&str]) -> Result<(), String> {
    // Parse without the flag
    let result = parse_flag_before_subcommand(command_args)?;

    // Verify it defaults to false
    assert_flag_is_false(&result)?;

    // Also verify with after_subcommand parsing
    let after_result = parse_flag_after_subcommand(command_args)?;
    assert_flag_is_false(&after_result)?;

    // Verify nested parsing also defaults to false
    let nested_result = parse_nested_subcommand(command_args)?;
    assert_flag_is_false(&nested_result)?;

    Ok(())
}

// ── Test Pattern Builders (planned section) ──────────────────────────────────

/// Placeholder for command-specific test builders
///
/// # Planned Implementation
///
/// This section will contain builders like:
///
/// ```rust,ignore
/// pub struct ScanNoInteractiveTest {
///     flag_position: FlagPosition,
///     expected_behavior: ExpectedBehavior,
///     test_workspace: TestWorkspace,
/// }
///
/// impl ScanNoInteractiveTest {
///     pub fn new() -> Self { /* ... */ }
///     pub fn with_flag_before(mut self) -> Self { /* ... */ }
///     pub fn with_flag_after(mut self) -> Self { /* ... */ }
///     pub fn expect_prompt_suppression(mut self) -> Self { /* ... */ }
///     pub fn run(self) -> TestResult { /* ... */ }
/// }
/// ```
pub mod command_builders {
    use super::*;

    /// Placeholder for command-specific test builders
    ///
    /// This will be implemented with builders for each command type:
    /// - ScanNoInteractiveTest
    /// - InitNoInteractiveTest
    /// - RemoveNoInteractiveTest
    /// - etc.
    pub struct CommandTestBuilder {
        _marker: std::marker::PhantomData<()>,
    }

    impl CommandTestBuilder {
        /// Create a new command test builder
        pub fn new() -> Self {
            Self {
                _marker: std::marker::PhantomData,
            }
        }
    }
}

// ── Mock Prompt Factories (planned section) ──────────────────────────────────

/// Placeholder for mock prompt interfaces
///
/// # Planned Implementation
///
/// This section will contain mock prompt implementations:
///
/// ```rust,ignore
/// pub trait MockPrompt {
///     fn would_prompt(&self, no_interactive: bool) -> bool;
///     fn prompt_text(&self) -> &str;
/// }
///
/// pub struct MockYesNoPrompt { /* ... */ }
/// pub struct MockSelectionPrompt { /* ... */ }
/// pub struct MockInputPrompt { /* ... */ }
/// ```
pub mod mock_prompts {
    /// Placeholder for mock prompt implementations
    ///
    /// This will be implemented with various mock prompt types:
    /// - MockYesNoPrompt
    /// - MockSelectionPrompt
    /// - MockInputPrompt
    /// - MockConfirmPrompt
    pub struct MockPromptFactory {
        _marker: std::marker::PhantomData<()>,
    }

    impl MockPromptFactory {
        /// Create a new mock prompt factory
        pub fn new() -> Self {
            Self {
                _marker: std::marker::PhantomData,
            }
        }
    }
}

// ── Test Fixtures (planned section) ───────────────────────────────────────────

/// Placeholder for reusable test fixtures
///
/// # Planned Implementation
///
/// This section will contain pre-built test fixtures:
///
/// ```rust,ignore
/// pub struct TestWorkspace {
///     pub path: PathBuf,
///     pub beads_dir: PathBuf,
///     pub config_dir: PathBuf,
/// }
///
/// pub fn create_minimal_workspace() -> TestWorkspace { /* ... */ }
/// pub fn create_workspace_with_beads() -> TestWorkspace { /* ... */ }
/// pub fn create_workspace_with_multiple_projects() -> TestWorkspace { /* ... */ }
/// ```
pub mod test_fixtures {
    /// Placeholder for reusable test fixtures
    ///
    /// This will be implemented with various fixture builders:
    /// - create_minimal_workspace()
    /// - create_workspace_with_beads()
    /// - create_workspace_with_multiple_projects()
    pub struct FixtureBuilder {
        _marker: std::marker::PhantomData<()>,
    }

    impl FixtureBuilder {
        /// Create a new fixture builder
        pub fn new() -> Self {
            Self {
                _marker: std::marker::PhantomData,
            }
        }
    }
}

// ── Custom Assertions (planned section) ───────────────────────────────────────

/// Placeholder for custom assertion helpers
///
/// # Planned Implementation
///
/// This section will contain custom assertions:
///
/// ```rust,ignore
/// pub fn assert_prompt_suppressed(result: &CommandResult);
/// pub fn assert_confirm_required(result: &CommandResult);
/// pub fn assert_no_user_interaction(result: &CommandResult);
/// pub fn assert_flag_extraction_consistent(before: &ParsedCli, after: &ParsedCli);
/// ```
pub mod assertions {
    /// Placeholder for custom assertion helpers
    ///
    /// This will be implemented with various assertion functions:
    /// - assert_prompt_suppressed()
    /// - assert_confirm_required()
    /// - assert_no_user_interaction()
    /// - assert_flag_extraction_consistent()
    pub struct AssertionHelper {
        _marker: std::marker::PhantomData<()>,
    }

    impl AssertionHelper {
        /// Create a new assertion helper
        pub fn new() -> Self {
            Self {
                _marker: std::marker::PhantomData,
            }
        }
    }
}

// ── Module Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flags_constants() {
        assert_eq!(flags::LONG, "--no-interactive");
        assert_eq!(flags::SHORT, "-y");
        assert_eq!(flags::ALL.len(), 2);
    }

    #[test]
    fn test_commands_constants() {
        assert_eq!(commands::SCAN, "scan");
        assert_eq!(commands::INIT, "init");
        assert_eq!(commands::REMOVE, "remove");
    }

    #[test]
    fn test_command_builder_placeholder() {
        let builder = command_builders::CommandTestBuilder::new();
        // Placeholder test - will be replaced when builder is implemented
        let _ = builder;
    }

    #[test]
    fn test_mock_prompt_factory_placeholder() {
        let factory = mock_prompts::MockPromptFactory::new();
        // Placeholder test - will be replaced when factory is implemented
        let _ = factory;
    }

    #[test]
    fn test_fixture_builder_placeholder() {
        let builder = test_fixtures::FixtureBuilder::new();
        // Placeholder test - will be replaced when builder is implemented
        let _ = builder;
    }

    #[test]
    fn test_assertion_helper_placeholder() {
        let helper = assertions::AssertionHelper::new();
        // Placeholder test - will be replaced when helper is implemented
        let _ = helper;
    }

    // ── Flag Parsing Utility Tests ─────────────────────────────────────────────

    #[test]
    fn test_parse_flag_before_subcommand_simple() {
        let result = parse_flag_before_subcommand(&["--no-interactive", "scan", "/tmp"]);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.no_interactive, true);
        assert_eq!(parsed.subcommand, Some("scan".to_string()));
        assert_eq!(parsed.nested_subcommand, None);
        assert_eq!(parsed.args.len(), 3);
    }

    #[test]
    fn test_parse_flag_before_subcommand_nested() {
        let result = parse_flag_before_subcommand(&[
            "--no-interactive",
            "projects",
            "remove",
            "my-project",
            "--confirm",
        ]);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.no_interactive, true);
        assert_eq!(parsed.subcommand, Some("projects".to_string()));
        assert_eq!(parsed.nested_subcommand, Some("remove".to_string()));
    }

    #[test]
    fn test_parse_flag_before_subcommand_short_form() {
        let result = parse_flag_before_subcommand(&["-y", "status", "--json"]);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.no_interactive, true);
        assert_eq!(parsed.subcommand, Some("status".to_string()));
    }

    #[test]
    fn test_parse_flag_before_subcommand_no_flag() {
        let result = parse_flag_before_subcommand(&["scan", "/tmp"]);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.no_interactive, false);
        assert_eq!(parsed.subcommand, Some("scan".to_string()));
    }

    #[test]
    fn test_parse_flag_after_subcommand_simple() {
        let result = parse_flag_after_subcommand(&["scan", "/tmp", "--no-interactive"]);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.no_interactive, true);
        assert_eq!(parsed.subcommand, Some("scan".to_string()));
        assert_eq!(parsed.nested_subcommand, None);
    }

    #[test]
    fn test_parse_flag_after_subcommand_nested() {
        let result = parse_flag_after_subcommand(&[
            "projects",
            "remove",
            "my-project",
            "--confirm",
            "--no-interactive",
        ]);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.no_interactive, true);
        assert_eq!(parsed.subcommand, Some("projects".to_string()));
        assert_eq!(parsed.nested_subcommand, Some("remove".to_string()));
    }

    #[test]
    fn test_parse_flag_after_subcommand_short_form() {
        let result = parse_flag_after_subcommand(&["status", "-y"]);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.no_interactive, true);
        assert_eq!(parsed.subcommand, Some("status".to_string()));
    }

    #[test]
    fn test_parse_flag_after_subcommand_no_flag() {
        let result = parse_flag_after_subcommand(&["scan", "/tmp"]);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.no_interactive, false);
        assert_eq!(parsed.subcommand, Some("scan".to_string()));
    }

    #[test]
    fn test_parse_nested_subcommand_flag_before() {
        let result = parse_nested_subcommand(&[
            "--no-interactive",
            "projects",
            "remove",
            "my-project",
            "--confirm",
        ]);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.no_interactive, true);
        assert_eq!(parsed.subcommand, Some("projects".to_string()));
        assert_eq!(parsed.nested_subcommand, Some("remove".to_string()));
    }

    #[test]
    fn test_parse_nested_subcommand_flag_after() {
        let result = parse_nested_subcommand(&["patterns", "add", "my-pattern", "--no-interactive"]);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.no_interactive, true);
        assert_eq!(parsed.subcommand, Some("patterns".to_string()));
        assert_eq!(parsed.nested_subcommand, Some("add".to_string()));
    }

    #[test]
    fn test_parse_nested_subcommand_single_level() {
        let result = parse_nested_subcommand(&["scan", "/tmp", "-y"]);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.no_interactive, true);
        assert_eq!(parsed.subcommand, Some("scan".to_string()));
        assert_eq!(parsed.nested_subcommand, None);
    }

    #[test]
    fn test_parse_nested_subcommand_no_flag() {
        let result = parse_nested_subcommand(&["status", "--json"]);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.no_interactive, false);
        assert_eq!(parsed.subcommand, Some("status".to_string()));
        assert_eq!(parsed.nested_subcommand, None);
    }

    #[test]
    fn test_extract_flag_value_present() {
        assert_eq!(extract_flag_value(&["scan", "/tmp", "--no-interactive"]), true);
        assert_eq!(extract_flag_value(&["-y", "scan", "/tmp"]), true);
        assert_eq!(extract_flag_value(&["scan", "--no-interactive", "/tmp"]), true);
    }

    #[test]
    fn test_extract_flag_value_absent() {
        assert_eq!(extract_flag_value(&["scan", "/tmp"]), false);
        assert_eq!(extract_flag_value(&["status", "--json"]), false);
        assert_eq!(extract_flag_value(&[]), false);
    }

    #[test]
    fn test_extract_subcommand_found() {
        assert_eq!(
            extract_subcommand(&["scan", "/tmp", "--no-interactive"]),
            Some("scan".to_string())
        );
        assert_eq!(
            extract_subcommand(&["projects", "remove", "my-project"]),
            Some("projects".to_string())
        );
        assert_eq!(extract_subcommand(&["--no-interactive", "status"]), Some("status".to_string()));
    }

    #[test]
    fn test_extract_subcommand_not_found() {
        assert_eq!(extract_subcommand(&["--no-interactive"]), None);
        assert_eq!(extract_subcommand(&["--json", "--verbose"]), None);
        assert_eq!(extract_subcommand(&[]), None);
    }

    #[test]
    fn test_verify_flag_position_consistency_simple() {
        let result = verify_flag_position_consistency(&["scan", "/tmp"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_flag_position_consistency_nested() {
        let result = verify_flag_position_consistency(&["projects", "remove", "my-project", "--confirm"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_flag_position_consistency_single_level() {
        let result = verify_flag_position_consistency(&["status"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_flag_before_subcommand_empty_args() {
        let result = parse_flag_before_subcommand(&[]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No arguments provided".to_string());
    }

    #[test]
    fn test_parse_flag_after_subcommand_empty_args() {
        let result = parse_flag_after_subcommand(&[]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No arguments provided".to_string());
    }

    #[test]
    fn test_parse_nested_subcommand_empty_args() {
        let result = parse_nested_subcommand(&[]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No arguments provided".to_string());
    }

    #[test]
    fn test_flag_parse_result_debug_clone() {
        let result = parse_flag_before_subcommand(&["--no-interactive", "scan", "/tmp"]).unwrap();

        // Test Clone trait
        let cloned = result.clone();
        assert_eq!(result.no_interactive, cloned.no_interactive);
        assert_eq!(result.subcommand, cloned.subcommand);

        // Test Debug trait (just ensure it compiles)
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("FlagParseResult"));
        assert!(debug_str.contains("no_interactive"));
    }

    // ── Flag Verification Utility Tests ─────────────────────────────────────────────

    #[test]
    fn test_assert_flag_is_true_success() {
        let result = parse_flag_before_subcommand(&["--no-interactive", "scan", "/tmp"]).unwrap();
        assert!(assert_flag_is_true(&result).is_ok());
    }

    #[test]
    fn test_assert_flag_is_true_failure() {
        let result = parse_flag_before_subcommand(&["scan", "/tmp"]).unwrap();
        let result = assert_flag_is_true(&result);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Expected no_interactive flag to be true"));
    }

    #[test]
    fn test_assert_flag_is_false_success() {
        let result = parse_flag_before_subcommand(&["scan", "/tmp"]).unwrap();
        assert!(assert_flag_is_false(&result).is_ok());
    }

    #[test]
    fn test_assert_flag_is_false_failure() {
        let result = parse_flag_before_subcommand(&["--no-interactive", "scan", "/tmp"]).unwrap();
        let result = assert_flag_is_false(&result);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Expected no_interactive flag to be false"));
    }

    #[test]
    fn test_assert_flag_value_true() {
        let result = parse_flag_before_subcommand(&["--no-interactive", "scan", "/tmp"]).unwrap();
        assert!(assert_flag_value(&result, true).is_ok());
    }

    #[test]
    fn test_assert_flag_value_false() {
        let result = parse_flag_before_subcommand(&["scan", "/tmp"]).unwrap();
        assert!(assert_flag_value(&result, false).is_ok());
    }

    #[test]
    fn test_assert_flag_value_mismatch() {
        let result = parse_flag_before_subcommand(&["--no-interactive", "scan", "/tmp"]).unwrap();
        let result = assert_flag_value(&result, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_assert_flag_propagation_simple() {
        let result = assert_flag_propagation(&["scan", "/tmp"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_assert_flag_propagation_nested() {
        let result = assert_flag_propagation(&["projects", "remove", "my-project", "--confirm"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_assert_flag_propagation_with_flags() {
        let result = assert_flag_propagation(&["status", "--json"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compare_flag_values_at_levels_with_flag() {
        let result = compare_flag_values_at_levels(&["scan", "/tmp", "--no-interactive"]);
        if let Err(e) = &result {
            println!("Error: {}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_compare_flag_values_at_levels_without_flag() {
        let result = compare_flag_values_at_levels(&["scan", "/tmp"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compare_flag_values_at_levels_short_flag() {
        let result = compare_flag_values_at_levels(&["-y", "scan", "/tmp"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compare_flag_values_at_levels_nested() {
        let result = compare_flag_values_at_levels(&["projects", "remove", "my-project", "--confirm", "--no-interactive"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_default_flag_value_simple() {
        let result = verify_default_flag_value(&["scan", "/tmp"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_default_flag_value_nested() {
        let result = verify_default_flag_value(&["projects", "remove", "my-project"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_default_flag_value_with_other_flags() {
        let result = verify_default_flag_value(&["status", "--json"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_default_flag_value_with_positional_args() {
        let result = verify_default_flag_value(&["init", "new-project", "/tmp/test"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_assert_flag_propagation_multiple_flags() {
        // Test with multiple flags to ensure no_interactive still propagates correctly
        let result = assert_flag_propagation(&["scan", "/tmp", "--verbose", "--json"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_assert_flag_value_with_short_form() {
        let result = parse_flag_before_subcommand(&["-y", "scan", "/tmp"]).unwrap();
        assert!(assert_flag_value(&result, true).is_ok());
    }

    #[test]
    fn test_compare_flag_values_at_levels_consistency_check() {
        // Test that all parsing methods agree on the flag value
        let args_with_flag = &["--no-interactive", "scan", "/tmp"];
        let result = compare_flag_values_at_levels(args_with_flag);
        assert!(result.is_ok());

        // Verify the flag is actually detected
        let direct = extract_flag_value(args_with_flag);
        assert_eq!(direct, true);
    }

    #[test]
    fn test_extract_and_assert_flag_consistency() {
        // Extract flag value directly
        let args = &["scan", "/tmp", "--no-interactive"];
        let extracted = extract_flag_value(args);
        assert_eq!(extracted, true);

        // Parse and assert the same value
        let parsed = parse_flag_after_subcommand(args).unwrap();
        assert!(assert_flag_value(&parsed, true).is_ok());
    }

    #[test]
    fn test_verify_default_flag_value_empty_command() {
        // Test with minimal command (edge case)
        let result = verify_default_flag_value(&["status"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_assert_flag_propagation_edge_case_only_flag() {
        // Edge case: command with only the flag
        let result = parse_flag_before_subcommand(&["--no-interactive"]);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert!(assert_flag_is_true(&parsed).is_ok());
        assert_eq!(parsed.subcommand, None);
    }
}
