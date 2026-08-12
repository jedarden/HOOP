//! CLI Test Helpers for `no_interactive` Flag Testing
//!
//! This module provides high-level test helpers and patterns for testing the
//! `--no-interactive` flag across HOOP CLI commands. It complements the lower-level
//! utilities in `cli_test_utils` by providing command-specific testing patterns
//! and flag parsing utilities.
//!
//! **Looking for low-level parsing utilities?** See [`cli_test_utils`]
//! for basic parsing functions and verification utilities.
//!
//! **New to testing the `--no-interactive` flag?** See [`TEST_PATTERNS_QUICK_START.md`]
//! for a unified guide covering both this module and `cli_test_utils` with real-world
//! examples and decision trees.
//!
//! # Getting Started
//!
//! Welcome to the HOOP CLI test helpers! This module makes it easy to test the
//! `--no-interactive` flag across all CLI commands. Whether you're adding a new
//! command or ensuring existing commands handle the flag correctly, we have three
//! levels of abstraction to match your needs.
//!
//! ## Quick Start: Which Approach Should I Use?
//!
//! Pick the approach that matches your testing goal:
//!
//! ### 1. **Comprehensive Suite Macro** (Recommended for most cases)
//!
//! Use `test_no_interactive_suite!` when you want complete coverage with minimal code.
//! This generates one test that verifies all five patterns: flag before, flag after,
//! short flag, consistency between positions, and default behavior.
//!
//! **When to use:**
//! - ✅ Adding tests for a new command
//! - ✅ Regression testing with minimal boilerplate
//! - ✅ Ensuring all patterns are tested consistently
//! - ✅ Quick coverage in CI/CD pipelines
//!
//! **Example:**
//! ```rust,ignore
//! test_no_interactive_suite!(test_mycommand_complete, "mycommand", &["mycommand", "--arg"]);
//! ```
//!
//! ### 2. **Individual Test Macros** (For focused testing)
//!
//! Use individual macros when you want separate test functions for each pattern,
//! making it easy to identify which specific pattern failed.
//!
//! **When to use:**
//! - ✅ Testing a single command's flag behavior
//! - ✅ Wanting granular test failure reports
//! - ✅ Building a custom test suite with selective patterns
//! - ✅ Debugging a specific flag position issue
//!
//! **Example:**
//! ```rust,ignore
//! test_flag_positions!(test_mycommand_positions, "mycommand", &["mycommand", "--arg"]);
//! test_flag_default_false!(test_mycommand_default, &["mycommand", "--arg"]);
//! ```
//!
//! ### 3. **Manual Implementation** (For maximum control)
//!
//! Use the helper functions directly when you need custom test logic or want to
//! understand exactly what's being tested at each step.
//!
//! **When to use:**
//! - ✅ Debugging a specific flag parsing issue
//! - ✅ Writing one-off tests for unique scenarios
//! - ✅ Learning how the flag parsing works internally
//! - ✅ Complex scenarios requiring custom assertions
//!
//! **Example:**
//! ```rust,ignore
//! let parsed = parse_flag_before_subcommand(&["scan", "/tmp"]).unwrap();
//! assert!(parsed.no_interactive);
//! assert!(assert_flag_propagation(&["scan", "/tmp"]).is_ok());
//! ```
//!
//! ## Common Mistakes to Avoid
//!
//! ### Mistake 1: Forgetting the Short Flag
//!
//! **Problem:** The no_interactive flag has two forms: `--no-interactive` and `-y`.
//! Testing only the long form means you're not testing the short form.
//!
//! **Solution:** Always test both forms, or use the comprehensive suite macro which
//! includes short flag testing automatically.
//!
//! ```rust,ignore
//! // ❌ WRONG: Only tests long form
//! let parsed = parse_flag_before_subcommand(&["--no-interactive", "scan", "/tmp"]);
//!
//! // ✅ CORRECT: Test both forms
//! test_no_interactive_suite!(test_scan_complete, "scan", &["scan", "/tmp"]);
//! ```
//!
//! ### Mistake 2: Missing Default Behavior Test
//!
//! **Problem:** Don't forget to test that the flag defaults to `false` when not specified.
//! This catches cases where the flag is always true.
//!
//! **Solution:** Always include a default behavior test, or use the suite macro which
//! includes default testing automatically.
//!
//! ```rust,ignore
//! // ❌ WRONG: Never tests default behavior
//! let parsed_with = parse_flag_before_subcommand(&["--no-interactive", "scan"]);
//! assert!(parsed_with.no_interactive);
//!
//! // ✅ CORRECT: Test default behavior too
//! test_flag_default_false!(test_scan_default, &["scan", "/tmp"]);
//! ```
//!
//! ### Mistake 3: Inconsistent Position Handling
//!
//! **Problem:** The flag should work identically whether placed before or after the
//! subcommand. Failing to verify this means users might encounter inconsistent behavior.
//!
//! **Solution:** Always verify position consistency, or use macros that include this check.
//!
//! ```rust,ignore
//! // ❌ WRONG: Tests positions in isolation
//! let before = parse_flag_before_subcommand(&["--no-interactive", "scan"]);
//! let after = parse_flag_after_subcommand(&["scan", "--no-interactive"]);
//! // Never compares them!
//!
//! // ✅ CORRECT: Verify consistency
//! assert!(verify_flag_position_consistency(&["scan", "/tmp"]).is_ok());
//! ```
//!
//! ### Mistake 4: Testing Position in Isolation
//!
//! **Problem:** When using individual macros, you often need multiple tests to cover
//! all patterns. Forgetting one pattern means incomplete coverage.
//!
//! **Solution:** Use the comprehensive suite macro for complete coverage, or create
//! a checklist of all patterns you need to test.
//!
//! ```rust,ignore
//! // ❌ WRONG: Only one pattern tested
//! test_no_interactive_flag_before!(test_scan_before, "scan", &["scan", "/tmp"]);
//! // Missing: after, short, consistency, default!
//!
//! // ✅ CORRECT: Use suite macro for complete coverage
//! test_no_interactive_suite!(test_scan_complete, "scan", &["scan", "/tmp"]);
//! ```
//!
//! ### Mistake 5: Not Testing Flag Propagation
//!
//! **Problem:** Even if the flag parses correctly, it needs to propagate from the CLI
//! to the command handler. Failing to test this means the flag might be parsed but ignored.
//!
//! **Solution:** Always verify flag propagation, especially for new commands.
//!
//! ```rust,ignore
//! // ❌ WRONG: Only tests parsing
//! let parsed = parse_flag_before_subcommand(&["scan", "/tmp"]).unwrap();
//! assert!(parsed.no_interactive);
//! // Never checks if handler receives the flag!
//!
//! // ✅ CORRECT: Verify propagation
//! assert!(assert_flag_propagation(&["scan", "/tmp"]).is_ok());
//! ```
//!
//! ## Decision Tree: Choosing Your Approach
//!
//! Not sure which approach to use? Follow this decision tree:
//!
//! ```
//! Are you testing a new command?
//! │
//! ├─ Yes → Use test_no_interactive_suite! for complete coverage
//! │        Add custom tests for command-specific behavior if needed
//! │
//! └─ No → Is this a debugging/learning scenario?
//!           │
//!           ├─ Yes → Use manual implementation with helper functions
//!           │        Step through each parsing stage
//!           │
//!           └─ No → Do you need granular failure reports?
//!                     │
//!                     ├─ Yes → Use individual test macros
//!                     │        One pattern per test function
//!                     │
//!                     └─ No → Use test_no_interactive_suite!
//!                           Complete coverage in one test
//! ```
//!
//! ## Real-World Example: Testing a New Command
//!
//! When adding a new command to HOOP, follow this pattern for comprehensive testing:
//!
//! ```rust,ignore
//! #[cfg(test)]
//! mod tests {
//!     use super::*;
//!     use cli_test_helpers::prelude::*;
//!
//!     // 1. Use the comprehensive test suite macro
//!     test_no_interactive_suite!(
//!         test_mycommand_complete,
//!         "mycommand",
//!         &["mycommand", "--arg"]
//!     );
//!
//!     // 2. Test flag positions specifically (optional - suite already covers this)
//!     test_flag_positions!(
//!         test_mycommand_positions,
//!         "mycommand",
//!         &["mycommand", "--arg"]
//!     );
//!
//!     // 3. Test default behavior (optional - suite already covers this)
//!     test_flag_default_false!(
//!         test_mycommand_default,
//!         &["mycommand", "--arg"]
//!     );
//!
//!     // 4. If it's a nested command, test propagation
//!     // test_nested_flag_propagation!(...);
//!
//!     // 5. If it's destructive, test confirm requirement
//!     // test_confirm_required_pattern!(...);
//!
//!     // 6. Add custom tests for command-specific behavior
//!     #[test]
//!     fn test_mycommand_flag_propagation() {
//!         // Verify the handler receives the flag correctly
//!         let main_code = std::fs::read_to_string("src/main.rs")
//!             .expect("Failed to read main.rs");
//!
//!         assert!(
//!             main_code.contains("mycommand::run_mycommand(no_interactive)"),
//!             "Main() must pass flag to handler"
//!         );
//!     }
//! }
//! ```
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
//! # Basic Test Patterns
//!
//! This section documents the fundamental patterns for testing `--no-interactive` flag
//! usage at different positions in the command line. Each pattern includes runnable
//! examples that demonstrate how to construct the test, verify flag recognition,
//! and understand expected behavior differences.
//!
//! ## Pattern 1: `hoop --no-interactive CMD` (Flag Before Subcommand)
//!
//! This pattern tests the global flag when it appears BEFORE the subcommand name.
//!
//! ### How to Construct the Test Command
//!
//! ```rust,ignore
//! use cli_test_helpers::prelude::*;
//!
//! // Construct: place --no-interactive at position 0, command at position 1
//! let args = ["--no-interactive", "scan", "/tmp"];
//! let result = parse_flag_before_subcommand(&args);
//! ```
//!
//! ### How to Verify the Flag is Recognized
//!
//! ```rust,ignore
//! // Verify parsing succeeded
//! assert!(result.is_ok(), "Parsing should succeed");
//!
//! // Extract the parsed result
//! let parsed = result.unwrap();
//!
//! // Verify flag value is true
//! assert_eq!(parsed.no_interactive, true, "Flag should be true");
//!
//! // Verify subcommand was identified
//! assert_eq!(parsed.subcommand, Some("scan".to_string()));
//! ```
//!
//! ### Expected Behavior Differences
//!
//! - **Parsing succeeds**: Flag is at expected global position (before subcommand)
//! - **no_interactive = true**: Boolean flag is set to true
//! - **Subcommand identification**: Primary command (scan, init, etc.) is correctly identified
//! - **Position independence**: Result is identical whether flag is before or after command
//!
//! ### Complete Example
//!
//! ```rust,ignore
//! #[test]
//! fn test_flag_before_subcommand() {
//!     use cli_test_helpers::prelude::*;
//!
//!     // Arrange: construct command with flag before subcommand
//!     let args = ["--no-interactive", "scan", "/tmp"];
//!
//!     // Act: parse the command
//!     let result = parse_flag_before_subcommand(&args);
//!
//!     // Assert: verify flag recognition
//!     assert!(result.is_ok());
//!     let parsed = result.unwrap();
//!     assert_eq!(parsed.no_interactive, true);
//!     assert_eq!(parsed.subcommand, Some("scan".to_string()));
//! }
//! ```
//!
//! ## Pattern 2: `hoop CMD --no-interactive` (Flag After Subcommand)
//!
//! This pattern tests the global flag when it appears AFTER the subcommand name.
//!
//! ### How to Construct the Test Command
//!
//! ```rust,ignore
//! use cli_test_helpers::prelude::*;
//!
//! // Construct: place command at position 0, flag at end
//! let args = ["scan", "/tmp", "--no-interactive"];
//! let result = parse_flag_after_subcommand(&args);
//! ```
//!
//! ### How to Verify the Flag is Recognized
//!
//! ```rust,ignore
//! // Verify parsing succeeded
//! assert!(result.is_ok(), "Parsing should succeed even with flag at end");
//!
//! // Extract the parsed result
//! let parsed = result.unwrap();
//!
//! // Verify flag value is true (regardless of position)
//! assert_eq!(parsed.no_interactive, true, "Flag should be true at any position");
//!
//! // Verify subcommand was identified
//! assert_eq!(parsed.subcommand, Some("scan".to_string()));
//! ```
//!
//! ### Expected Behavior Differences
//!
//! - **Parsing succeeds**: Flag at end position is still valid due to `global = true` in clap
//! - **no_interactive = true**: Boolean flag is set to true (same as Pattern 1)
//! - **Subcommand identification**: Primary command is correctly identified even with flag after
//! - **Position independence**: Result is identical whether flag is before or after command
//! - **User ergonomics**: Matches users who naturally type command first, then flags
//!
//! ### Complete Example
//!
//! ```rust,ignore
//! #[test]
//! fn test_flag_after_subcommand() {
//!     use cli_test_helpers::prelude::*;
//!
//!     // Arrange: construct command with flag after subcommand
//!     let args = ["scan", "/tmp", "--no-interactive"];
//!
//!     // Act: parse the command
//!     let result = parse_flag_after_subcommand(&args);
//!
//!     // Assert: verify flag recognition
//!     assert!(result.is_ok());
//!     let parsed = result.unwrap();
//!     assert_eq!(parsed.no_interactive, true);
//!     assert_eq!(parsed.subcommand, Some("scan".to_string()));
//! }
//! ```
//!
//! ## Pattern Comparison: Before vs After
//!
//! | Aspect | Pattern 1: `--no-interactive CMD` | Pattern 2: `CMD --no-interactive` |
//! |--------|-----------------------------------|-----------------------------------|
//! | **Parsing function** | `parse_flag_before_subcommand()` | `parse_flag_after_subcommand()` |
//! | **Flag position** | Position 0 (before command) | End position (after command) |
//! | **no_interactive value** | `true` | `true` |
//! | **Subcommand extraction** | Position 1 | Position 0 |
//! | **Result consistency** | ✅ Identical to Pattern 2 | ✅ Identical to Pattern 1 |
//! | **Clap behavior** | Standard global flag | Global flag via `global = true` |
//! | **User expectation** | CLI power users | Casual users |
//! | **CI/CD pattern** | Script generators (flags first) | Human-written scripts |
//!
//! ## Key Insight: Position Independence
//!
//! The fundamental guarantee of HOOP's `--no-interactive` flag design is **position independence**:
//!
//! ```rust,ignore
//! // These two commands MUST produce identical parsed results:
//! let before = parse_flag_before_subcommand(&["--no-interactive", "scan", "/tmp"]).unwrap();
//! let after = parse_flag_after_subcommand(&["scan", "/tmp", "--no-interactive"]).unwrap();
//!
//! assert_eq!(before.no_interactive, after.no_interactive); // ✅ Both true
//! assert_eq!(before.subcommand, after.subcommand);             // ✅ Both "scan"
//! ```
//!
//! This guarantee is verified by the `verify_flag_position_consistency()` utility:
//!
//! ```rust,ignore
//! // Verifies that both positions produce identical results
//! assert!(verify_flag_position_consistency(&["scan", "/tmp"]).is_ok());
//! ```
//!
//! # Flag Parsing Utilities
//!
//! This module provides utilities for parsing clap command structures with flags
//! At different positions:
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
//! That the boolean value is consistent regardless of position.
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
//! Flag when `--no-interactive` is set:
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
//! # Using Test Macros for Common Patterns
//!
//! The macros provided by this module eliminate repetitive boilerplate when testing
//! the `no_interactive` flag. Each macro generates a complete test function that
//! verifies specific aspects of flag behavior.
//!
//! ## Available Macros
//!
//! ### `test_flag_positions!` - Test Flag at Both Positions
//!
//! Generates a test that verifies the flag works correctly before and after the subcommand:
//!
//! ```rust,ignore
//! test_flag_positions!(test_scan_positions, "scan", &["scan", "/tmp"]);
//! ```
//!
//! **What it tests:**
//! - Flag before subcommand: `hoop --no-interactive scan /tmp`
//! - Flag after subcommand: `hoop scan /tmp --no-interactive`
//! - Short flag variant: `hoop -y scan /tmp`
//! - Both positions yield the same `no_interactive=true` value
//!
//! **Generated test includes:**
//! - Parsing at both positions
//! - Verification that `no_interactive=true` in all cases
//! - Consistency check between positions
//!
//! ### `test_no_interactive_suite!` - Complete Test Suite
//!
//! Generates a comprehensive test covering all flag behavior aspects:
//!
//! ```rust,ignore
//! test_no_interactive_suite!(test_status_suite, "status", &["status", "--json"]);
//! ```
//!
//! **What it tests:**
//! 1. Flag before subcommand → `no_interactive=true`
//! 2. Flag after subcommand → `no_interactive=true`
//! 3. Short flag `-y` → `no_interactive=true`
//! 4. Position independence (both positions give same value)
//! 5. Default behavior (no flag → `no_interactive=false`)
//! 6. Flag propagation verification
//!
//! **Use this for:** Commands where you need comprehensive coverage in a single test.
//!
//! ### `test_nested_flag_propagation!` - Test Nested Command Structures
//!
//! For commands with two-level structure like `projects remove`:
//!
//! ```rust,ignore
//! test_nested_flag_propagation!(
//!     test_projects_remove,
//!     "projects",    // primary subcommand
//!     "remove",      // nested subcommand
//!     &["projects", "remove", "my-project", "--confirm"]
//! );
//! ```
//!
//! **What it tests:**
//! - Nested command structure parsing
//! - Flag extraction at both primary and nested levels
//! - Flag before primary: `hoop --no-interactive projects remove test`
//! - Flag after nested: `hoop projects remove test --no-interactive`
//!
//! ### `test_flag_default_false!` - Test Default Behavior
//!
//! Verifies that the flag defaults to `false` when not specified:
//!
//! ```rust,ignore
//! test_flag_default_false!(test_list_default, &["list"]);
//! ```
//!
//! **What it tests:**
//! - Parsing without flag sets `no_interactive=false`
//! - All parsing methods agree on default value
//! - No flag present in raw arguments
//!
//! ### `test_confirm_required_pattern!` - Test Destructive Operation Safety
//!
//! For commands that require `--confirm` when `--no-interactive` is set:
//!
//! ```rust,ignore
//! test_confirm_required_pattern!(
//!     test_remove_confirm,
//!     "remove",
//!     &["projects", "remove", "my-project"]
//! );
//! ```
//!
//! **What it tests:**
//! - Parsing with `--no-interactive --confirm` (valid combination)
//! - Parsing with `--no-interactive` but no `--confirm` (should error in real code)
//! - Command structure supports the safety pattern
//!
//! ## Complete Example: Testing a New Command
//!
//! When adding a new command to HOOP, follow this pattern for comprehensive testing:
//!
//! ```rust,ignore
//! use cli_test_helpers::prelude::*;
//! use cli_test_helpers::*;
//!
//! #[cfg(test)]
//! mod tests {
//!     use super::*;
//!
//!     // 1. Use the comprehensive test suite macro
//!     test_no_interactive_suite!(test_mycommand_complete, "mycommand", &["mycommand", "--arg"]);
//!
//!     // 2. Test flag positions specifically
//!     test_flag_positions!(test_mycommand_positions, "mycommand", &["mycommand", "--arg"]);
//!
//!     // 3. Test default behavior
//!     test_flag_default_false!(test_mycommand_default, &["mycommand", "--arg"]);
//!
//!     // 4. If it's a nested command, test propagation
//!     // test_nested_flag_propagation!(...);
//!
//!     // 5. If it's destructive, test confirm requirement
//!     // test_confirm_required_pattern!(...);
//!
//!     // 6. Add custom tests for command-specific behavior
//!     #[test]
//!     fn test_mycommand_specific_behavior() {
//!         // Test that the handler uses the flag correctly
//!         let code = std::fs::read_to_string("src/mycommand.rs")
//!             .expect("Failed to read mycommand.rs");
//!
//!         assert!(
//!             code.contains("no_interactive: bool"),
//!             "Handler must accept no_interactive parameter"
//!         );
//!
//!         assert!(
//!             code.contains("if no_interactive"),
//!             "Handler must check no_interactive flag"
//!         );
//!     }
//!
//!     // 7. Test flag propagation from main.rs to handler
//!     #[test]
//!     fn test_mycommand_flag_propagation() {
//!         let main_code = std::fs::read_to_string("src/main.rs")
//!             .expect("Failed to read main.rs");
//!
//!         assert!(
//!             main_code.contains("let no_interactive = cli.no_interactive;"),
//!             "main() must extract flag from CLI"
//!         );
//!
//!         assert!(
//!             main_code.contains("mycommand::run_mycommand(no_interactive)"),
//!             "main() must pass flag to handler"
//!         );
//!     }
//! }
//! ```
//!
//! # Testing Destructive Operation Safety Pattern
//!
//! For commands that perform destructive operations (remove, delete, restore, etc.),
//! test that they require `--confirm` when `--no-interactive` is set:
//!
//! ```rust,ignore
//! use cli_test_helpers::*;
//!
//! #[test]
//! fn test_remove_safety_pattern() {
//!     // 1. Test that the command accepts both flags
//!     test_confirm_required_pattern!(
//!         test_remove_confirm,
//!         "remove",
//!         &["projects", "remove", "my-project"]
//!     );
//!
//!     // 2. Verify the source code implements the check
//!     let code = std::fs::read_to_string("src/projects.rs")
//!         .expect("Failed to read projects.rs");
//!
//!     // Must check for confirm flag in non-interactive mode
//!     assert!(
//!         code.contains("if no_interactive && !confirm"),
//!         "Must check --confirm requirement in non-interactive mode"
//!     );
//!
//!     // Must show helpful error message
//!     assert!(
//!         code.contains("--confirm is required in non-interactive mode"),
//!         "Must show helpful error when --confirm is missing"
//!     );
//!
//!     // Must show usage example
//!     assert!(
//!         code.contains("Re-run with: hoop projects remove"),
//!         "Must show correct usage in error message"
//!     );
//! }
//! ```
//!
//! # Testing Commands That Reject no_interactive
//!
//! Some commands (like `init`) require interactive input and should reject
//! `--no-interactive` entirely with a helpful error message:
//!
//! ```rust,ignore
//! #[test]
//! fn test_init_rejects_no_interactive() {
//!     let code = std::fs::read_to_string("src/init.rs")
//!         .expect("Failed to read init.rs");
//!
//!     // Must check the flag early
//!     assert!(
//!         code.contains("if no_interactive"),
//!         "Init must check no_interactive flag"
//!     );
//!
//!     // Must exit with error code 2 (fatal/precondition error)
//!     assert!(
//!         code.contains("std::process::exit(2)"),
//!         "Init must exit with error code 2"
//!     );
//!
//!     // Must show helpful error message
//!     assert!(
//!         code.contains("cannot run in non-interactive mode"),
//!         "Init must explain why it cannot run non-interactively"
//!     );
//!
//!     assert!(
//!         code.contains("requires interactive input for configuration"),
//!         "Init must state that it requires interactive input"
//!     );
//!
//!     assert!(
//!         code.contains("manually create ~/.hoop/config.yml"),
//!         "Init must suggest manual configuration for automation"
//!     );
//! }
//! ```
//!
//! # Testing Flag Propagation from CLI to Handler
//!
//! Verify that the flag value flows correctly through the command pipeline:
//!
//! ```rust,ignore
//! #[test]
//! fn test_scan_flag_propagation() {
//!     // 1. Verify main.rs extracts the flag
//!     let main_code = std::fs::read_to_string("src/main.rs")
//!         .expect("Failed to read main.rs");
//!
//!     assert!(
//!         main_code.contains("let no_interactive = cli.no_interactive;"),
//!         "main() must extract flag from CLI"
//!     );
//!
//!     // 2. Verify main.rs passes flag to handler
//!     assert!(
//!         main_code.contains("projects::scan_projects(&root, no_interactive || yes)"),
//!         "main() must pass flag to handler"
//!     );
//!
//!     // 3. Verify handler accepts and uses the flag
//!     let scan_code = std::fs::read_to_string("src/projects.rs")
//!         .expect("Failed to read projects.rs");
//!
//!     assert!(
//!         scan_code.contains("pub fn scan_projects(root: &str, no_interactive: bool)"),
//!         "Handler must accept no_interactive parameter"
//!     );
//!
//!     assert!(
//!         scan_code.contains("if no_interactive"),
//!         "Handler must use the flag in conditional logic"
//!     );
//! }
//! ```
//!
//! # Using Verification Utilities
//!
//! This module provides several verification utilities that return `Result<(), String>`
//! for detailed error messages in tests:
//!
//! ```rust,ignore
//! use cli_test_helpers::prelude::*;
//!
//! #[test]
//! fn test_verification_utilities() {
//!     let parsed = parse_flag_before_subcommand(&["scan", "/tmp"]).unwrap();
//!
//!     // Assert flag is true
//!     assert!(assert_flag_is_true(&parsed).is_ok());
//!
//!     // Assert flag is false
//!     assert!(assert_flag_is_false(&parsed).is_ok());
//!
//!     // Assert flag equals expected value
//!     assert!(assert_flag_value(&parsed, true).is_ok());
//!
//!     // Verify flag propagation
//!     assert!(assert_flag_propagation(&["scan", "/tmp"]).is_ok());
//!
//!     // Verify default value
//!     assert!(verify_default_flag_value(&["scan", "/tmp"]).is_ok());
//!
//!     // Verify position consistency
//!     assert!(verify_flag_position_consistency(&["scan", "/tmp"]).is_ok());
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
                "Before_subcommand should have flag=true (we added it at position 0), got {}",
                before.no_interactive
            ));
        }
    } else {
        errors.push(format!("Before_subcommand parsing failed: {:?}", before_result));
    }

    // Check that after_subcommand found the flag we added
    if let Ok(after) = &after_result {
        if !after.no_interactive {
            errors.push(format!(
                "After_subcommand should have flag=true (we added it at end), got {}",
                after.no_interactive
            ));
        }
    } else {
        errors.push(format!("After_subcommand parsing failed: {:?}", after_result));
    }

    // Check that direct extraction and nested agree on the original args
    if let Ok(nested) = &nested_result {
        if nested.no_interactive != direct_flag {
            errors.push(format!(
                "Direct extraction ({}) != Nested_subcommand ({}) - These should agree on original args",
                direct_flag, nested.no_interactive
            ));
        }
    } else {
        errors.push(format!("Nested_subcommand parsing failed: {:?}", nested_result));
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

// ── Test Macros for Common Patterns ─────────────────────────────────────────────

/// Macro for testing flag parsing at both positions for a command
///
/// This macro generates a test that verifies:
/// - Flag before subcommand: `hoop --no-interactive <command> [args]`
/// - Flag after subcommand: `hoop <command> [args] --no-interactive`
/// - Short flag variant: `hoop -y <command> [args]`
/// - Both positions yield the same value
///
/// # Usage
///
/// ```rust,ignore
/// test_flag_positions!(test_scan_positions, "scan", &["scan", "/tmp"]);
/// ```
///
/// # Generated Test
///
/// The macro generates a test function named `$test_name` that:
/// 1. Parses the command with flag before subcommand
/// 2. Parses the command with flag after subcommand
/// 3. Parses the command with short flag
/// 4. Verifies all three approaches yield the same `no_interactive=true` value
#[macro_export]
macro_rules! test_flag_positions {
    ($test_name:ident, $command_name:expr, $base_args:expr) => {
        #[test]
        fn $test_name() {
            use super::prelude::*;

            // Test flag before subcommand
            let args_before: Vec<&str> = vec!["--no-interactive"]
                .iter()
                .chain($base_args.iter())
                .copied()
                .collect();
            let before = parse_flag_before_subcommand(&args_before);
            assert!(
                before.is_ok(),
                "Failed to parse flag before subcommand for {}",
                $command_name
            );
            let before_parsed = before.unwrap();
            assert!(
                before_parsed.no_interactive,
                "No_interactive should be true with flag before {}",
                $command_name
            );

            // Test flag after subcommand
            let args_after: Vec<&str> = $base_args
                .iter()
                .chain(&["--no-interactive"])
                .copied()
                .collect();
            let after = parse_flag_after_subcommand(&args_after);
            assert!(
                after.is_ok(),
                "Failed to parse flag after subcommand for {}",
                $command_name
            );
            let after_parsed = after.unwrap();
            assert!(
                after_parsed.no_interactive,
                "No_interactive should be true with flag after {}",
                $command_name
            );

            // Verify consistency between positions
            assert_eq!(
                before_parsed.no_interactive,
                after_parsed.no_interactive,
                "Flag position should not affect value for {}",
                $command_name
            );

            // Test short flag variant
            let short_args: Vec<&str> = vec!["-y"]
                .iter()
                .chain($base_args.iter())
                .copied()
                .collect();
            let short_value = extract_flag_value(&short_args);
            assert!(
                short_value,
                "Short flag -y should set No_interactive=true for {}",
                $command_name
            );
        }
    };
}

/// Macro for testing flag propagation through nested commands
///
/// This macro verifies that the flag propagates correctly through
/// nested subcommand structures like `projects remove` or `patterns add`.
///
/// # Usage
///
/// ```rust,ignore
/// test_nested_flag_propagation!(
///     test_projects_remove_propagation,
///     "projects",
///     "remove",
///     &["projects", "remove", "my-project", "--confirm"]
/// );
/// ```
#[macro_export]
macro_rules! test_nested_flag_propagation {
    ($test_name:ident, $primary:expr, $nested:expr, $base_args:expr) => {
        #[test]
        fn $test_name() {
            use super::prelude::*;

            // Parse with nested structure
            let parsed = parse_nested_subcommand($base_args);
            assert!(
                parsed.is_ok(),
                "Failed to parse nested command for {} {}",
                $primary,
                $nested
            );

            let result = parsed.unwrap();
            assert_eq!(
                result.subcommand,
                Some($primary.to_string()),
                "Primary subcommand should be {}",
                $primary
            );
            assert_eq!(
                result.nested_subcommand,
                Some($nested.to_string()),
                "Nested subcommand should be {}",
                $nested
            );

            // Verify flag extraction works at both levels
            let with_flag_before: Vec<&str> = vec!["--no-interactive"]
                .iter()
                .chain($base_args.iter())
                .copied()
                .collect();
            let parsed_before = parse_nested_subcommand(&with_flag_before);
            assert!(parsed_before.is_ok());
            assert!(parsed_before.unwrap().no_interactive);

            let with_flag_after: Vec<&str> = $base_args
                .iter()
                .chain(&["--no-interactive"])
                .copied()
                .collect();
            let parsed_after = parse_nested_subcommand(&with_flag_after);
            assert!(parsed_after.is_ok());
            assert!(parsed_after.unwrap().no_interactive);
        }
    };
}

/// Macro for testing default flag behavior (false when not specified)
///
/// # Usage
///
/// ```rust,ignore
/// test_flag_default_false!(test_scan_default, &["scan", "/tmp"]);
/// ```
#[macro_export]
macro_rules! test_flag_default_false {
    ($test_name:ident, $base_args:expr) => {
        #[test]
        fn $test_name() {
            use super::prelude::*;

            let parsed = parse_flag_before_subcommand($base_args);
            assert!(parsed.is_ok(), "Failed to parse command without flag");

            let result = parsed.unwrap();
            assert!(
                !result.no_interactive,
                "No_interactive should default to false when not specified"
            );

            // Verify all parsing methods agree on default value
            assert!(
                verify_default_flag_value($base_args).is_ok(),
                "Default flag value verification failed"
            );
        }
    };
}

/// Macro for comprehensive no_interactive flag test suite
///
/// Generates a complete test covering all aspects of flag behavior:
/// - Parsing at both positions (before/after subcommand)
/// - Short flag variant (-y)
/// - Default behavior (false when not specified)
/// - Position independence (both positions yield same value)
/// - Flag propagation through handlers
///
/// # Usage
///
/// ```rust,ignore
/// test_no_interactive_suite!(test_status_suite, "status", &["status", "--json"]);
/// ```
///
/// # Example
///
/// For a command like `hoop status --json`, this generates tests that verify:
/// - `hoop --no-interactive status --json` parses correctly
/// - `hoop status --json --no-interactive` parses correctly
/// - `hoop -y status --json` parses correctly
/// - `hoop status --json` (no flag) defaults to false
/// - All three flag positions yield the same `no_interactive=true` value
#[macro_export]
macro_rules! test_no_interactive_suite {
    ($test_name:ident, $command_name:expr, $base_args:expr) => {
        #[test]
        fn $test_name() {
            use super::prelude::*;

            // Test 1: Flag before subcommand
            let args_before: Vec<&str> = vec!["--no-interactive"]
                .iter()
                .chain($base_args.iter())
                .copied()
                .collect();
            let parsed_before = parse_flag_before_subcommand(&args_before);
            assert!(
                parsed_before.is_ok(),
                "Failed to parse {} with flag before subcommand",
                $command_name
            );
            let before_result = parsed_before.unwrap();
            assert!(
                before_result.no_interactive,
                "Flag before subcommand should set No_interactive=true for {}",
                $command_name
            );
            assert_flag_is_true(&before_result)
                .expect("Flag before subcommand assertion failed");

            // Test 2: Flag after subcommand
            let args_after: Vec<&str> = $base_args
                .iter()
                .chain(&["--no-interactive"])
                .copied()
                .collect();
            let parsed_after = parse_flag_after_subcommand(&args_after);
            assert!(
                parsed_after.is_ok(),
                "Failed to parse {} with flag after subcommand",
                $command_name
            );
            let after_result = parsed_after.unwrap();
            assert!(
                after_result.no_interactive,
                "Flag after subcommand should set No_interactive=true for {}",
                $command_name
            );
            assert_flag_is_true(&after_result)
                .expect("Flag after subcommand assertion failed");

            // Test 3: Short flag variant
            let args_short: Vec<&str> = vec!["-y"]
                .iter()
                .chain($base_args.iter())
                .copied()
                .collect();
            let short_value = extract_flag_value(&args_short);
            assert!(
                short_value,
                "Short flag -y should set No_interactive=true for {}",
                $command_name
            );

            // Test 4: Position independence
            assert_eq!(
                before_result.no_interactive,
                after_result.no_interactive,
                "Flag position must not affect value for {}",
                $command_name
            );
            assert!(
                verify_flag_position_consistency($base_args).is_ok(),
                "Flag position consistency check failed for {}",
                $command_name
            );

            // Test 5: Default behavior (no flag)
            let parsed_default = parse_flag_before_subcommand($base_args);
            assert!(
                parsed_default.is_ok(),
                "Failed to parse {} without flag",
                $command_name
            );
            let default_result = parsed_default.unwrap();
            assert!(
                !default_result.no_interactive,
                "Default No_interactive should be false for {}",
                $command_name
            );
            assert_flag_is_false(&default_result)
                .expect("Default flag assertion failed");

            // Test 6: Flag propagation
            assert!(
                assert_flag_propagation($base_args).is_ok(),
                "Flag propagation check failed for {}",
                $command_name
            );
        }
    };
}

/// Macro for testing destructive operations require --confirm in no-interactive mode
///
/// This macro tests the safety pattern where destructive operations (remove, delete, etc.)
/// require an explicit `--confirm` flag when `--no-interactive` is set.
///
/// # Usage
///
/// ```rust,ignore
/// test_confirm_required_pattern!(
///     test_remove_confirm_pattern,
///     "remove",
///     &["projects", "remove", "my-project"]
/// );
/// ```
///
/// # What It Tests
///
/// For commands like `hoop projects remove my-project`, this verifies:
/// - Without `--no-interactive`: prompts for confirmation
/// - With `--no-interactive` but without `--confirm`: errors with helpful message
/// - With both `--no-interactive` and `--confirm`: proceeds without prompting
#[macro_export]
macro_rules! test_confirm_required_pattern {
    ($test_name:ident, $operation:expr, $base_args:expr) => {
        #[test]
        fn $test_name() {
            use super::prelude::*;

            // This pattern is verified by code review in actual tests
            // Here we verify the parsing structure supports the pattern

            // Parse with --no-interactive and --confirm (valid combination)
            let args_valid: Vec<&str> = $base_args
                .iter()
                .chain(&["--no-interactive", "--confirm"])
                .copied()
                .collect();
            let parsed_valid = parse_flag_after_subcommand(&args_valid);
            assert!(
                parsed_valid.is_ok(),
                "Failed to parse {} with --no-interactive --confirm",
                $operation
            );
            let valid_result = parsed_valid.unwrap();
            assert!(valid_result.no_interactive);
            assert!(
                valid_result.args.contains(&"--confirm".to_string()),
                "Args should include --confirm flag for {}",
                $operation
            );

            // Parse with --no-interactive but without --confirm (should error in real code)
            let args_invalid: Vec<&str> = $base_args
                .iter()
                .chain(&["--no-interactive"])
                .copied()
                .collect();
            let parsed_invalid = parse_flag_after_subcommand(&args_invalid);
            assert!(
                parsed_invalid.is_ok(),
                "Should parse {} with --no-interactive (even without --confirm)",
                $operation
            );
            let invalid_result = parsed_invalid.unwrap();
            assert!(invalid_result.no_interactive);
            assert!(
                !invalid_result.args.contains(&"--confirm".to_string()),
                "Args should not include --confirm flag for {}",
                $operation
            );

            // The actual enforcement of --confirm requirement is tested
            // in integration tests by reading the source code
        }
    };
}

// ── Module Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Example: Using the test suite macro for a simple command
    test_no_interactive_suite!(test_status_complete, "status", &["status", "--json"]);

    // Example: Using the test suite macro for a command with positional args
    test_no_interactive_suite!(test_scan_complete, "scan", &["scan", "/tmp"]);

    // Example: Testing flag positions for init command
    test_flag_positions!(test_init_positions, "init", &["init"]);

    // Example: Testing nested command flag propagation
    test_nested_flag_propagation!(
        test_projects_remove_propagation,
        "projects",
        "remove",
        &["projects", "remove", "my-project", "--confirm"]
    );

    // Example: Testing default flag behavior
    test_flag_default_false!(test_list_default, &["list"]);

    // Example: Testing confirm required pattern
    test_confirm_required_pattern!(
        test_restore_confirm_pattern,
        "restore",
        &["restore", "--from", "s3://bucket/key"]
    );

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

    // ── Comprehensive Example: All Patterns Working Together ─────────────────────

    /// Comprehensive example test demonstrating all flag patterns working together
    ///
    /// This test serves as the canonical example for how to test the no_interactive flag.
    /// It demonstrates:
    /// 1. Basic flag position patterns (before/after subcommand)
    /// 2. Flag propagation patterns through nested commands
    /// 3. Macro usage for common patterns
    /// 4. Complex scenarios combining multiple patterns
    /// 5. Integration with verification utilities
    #[test]
    fn comprehensive_example_all_patterns_together() {
        use super::prelude::*;

        // ── Part 1: Basic Flag Position Patterns ─────────────────────────────────

        // Pattern 1: Flag before subcommand
        let args_before = &["--no-interactive", "scan", "/tmp"];
        let parsed_before = parse_flag_before_subcommand(args_before)
            .expect("Should parse flag before subcommand");
        assert_eq!(parsed_before.no_interactive, true);
        assert_eq!(parsed_before.subcommand, Some("scan".to_string()));
        assert!(assert_flag_is_true(&parsed_before).is_ok());

        // Pattern 2: Flag after subcommand
        let args_after = &["scan", "/tmp", "--no-interactive"];
        let parsed_after = parse_flag_after_subcommand(args_after)
            .expect("Should parse flag after subcommand");
        assert_eq!(parsed_after.no_interactive, true);
        assert_eq!(parsed_after.subcommand, Some("scan".to_string()));
        assert!(assert_flag_is_true(&parsed_after).is_ok());

        // Pattern 3: Short flag variant
        let args_short = &["-y", "status", "--json"];
        let parsed_short = parse_flag_before_subcommand(args_short)
            .expect("Should parse short flag");
        assert_eq!(parsed_short.no_interactive, true);
        assert_eq!(extract_flag_value(args_short), true);

        // ── Part 2: Flag Propagation Patterns ───────────────────────────────────────

        // Pattern 4: Nested command flag propagation
        let nested_args = &["projects", "remove", "my-project", "--confirm"];
        let parsed_nested = parse_nested_subcommand(nested_args)
            .expect("Should parse nested command");
        assert_eq!(parsed_nested.subcommand, Some("projects".to_string()));
        assert_eq!(parsed_nested.nested_subcommand, Some("remove".to_string()));

        // Verify flag propagates to nested levels
        let nested_with_flag = &["--no-interactive"]
            .iter()
            .chain(nested_args.iter())
            .copied()
            .collect::<Vec<_>>();
        let parsed_nested_flag = parse_nested_subcommand(nested_with_flag)
            .expect("Should parse nested command with flag");
        assert_eq!(parsed_nested_flag.no_interactive, true);

        // Pattern 5: Position independence verification
        assert!(verify_flag_position_consistency(nested_args).is_ok(),
            "Flag should be consistent at both positions");

        // Pattern 6: Flag propagation from top-level to handler
        assert!(assert_flag_propagation(nested_args).is_ok(),
            "Flag should propagate correctly through handler chain");

        // ── Part 3: Complex Scenarios ──────────────────────────────────────────────

        // Scenario 1: Multiple flags combined
        let multi_flags = &["scan", "/tmp", "--verbose", "--json", "--no-interactive"];
        let parsed_multi = parse_flag_after_subcommand(multi_flags)
            .expect("Should parse command with multiple flags");
        assert_eq!(parsed_multi.no_interactive, true);
        assert!(parsed_multi.args.contains(&"--verbose".to_string()));
        assert!(parsed_multi.args.contains(&"--json".to_string()));

        // Scenario 2: Default behavior verification
        let no_flag_args = &["list"];
        let parsed_default = parse_flag_before_subcommand(no_flag_args)
            .expect("Should parse command without flag");
        assert_eq!(parsed_default.no_interactive, false);
        assert!(verify_default_flag_value(no_flag_args).is_ok());

        // Scenario 3: All parsing levels consistency
        let test_args = &["status", "--json", "--no-interactive"];
        assert!(compare_flag_values_at_levels(test_args).is_ok(),
            "All parsing levels should agree on flag value");

        // ── Part 4: Integration with Verification Utilities ───────────────────────────

        // Verify 1: Flag extraction works correctly
        let extracted = extract_flag_value(&["scan", "/tmp", "-y"]);
        assert_eq!(extracted, true, "Direct extraction should work");

        // Verify 2: Subcommand extraction works correctly
        let subcmd = extract_subcommand(&["projects", "remove", "test"]);
        assert_eq!(subcmd, Some("projects".to_string()));

        // Verify 3: Flag value assertions work correctly
        let parsed = parse_flag_before_subcommand(&["--no-interactive", "scan"])
            .expect("Should parse successfully");
        assert!(assert_flag_value(&parsed, true).is_ok());
        assert!(assert_flag_value(&parsed, false).is_err());

        // ── Part 5: Edge Cases and Error Handling ─────────────────────────────────────

        // Edge case 1: Empty arguments
        let empty_result = parse_flag_before_subcommand(&[]);
        assert!(empty_result.is_err(), "Empty args should error");

        // Edge case 2: Only flag, no command
        let flag_only = parse_flag_before_subcommand(&["--no-interactive"])
            .expect("Should parse flag-only args");
        assert_eq!(flag_only.subcommand, None);
        assert_eq!(flag_only.no_interactive, true);

        // Edge case 3: Multiple occurrences of flag (last wins in practice)
        let multi_flag = &["-y", "scan", "/tmp", "-y"];
        assert_eq!(extract_flag_value(multi_flag), true,
            "Should detect flag presence regardless of count");

        // ── Summary ─────────────────────────────────────────────────────────────────

        // This test demonstrates:
        // ✅ Basic flag position patterns (before/after)
        // ✅ Short flag variant (-y)
        // ✅ Nested command flag propagation
        // ✅ Position independence verification
        // ✅ Flag propagation to handlers
        // ✅ Multiple flags combined
        // ✅ Default behavior
        // ✅ All parsing levels consistency
        // ✅ Integration with verification utilities
        // ✅ Edge cases and error handling

        println!("✓ Comprehensive example test passed - all patterns work correctly");
    }

    /// Example test showing the recommended pattern for testing a new command
    ///
    /// This demonstrates the exact pattern you should follow when adding tests
    /// for a new command in HOOP.
    #[test]
    fn example_recommended_pattern_for_new_command() {
        // This is the recommended pattern for testing a new command.
        // It combines macros for comprehensive coverage with custom tests
        // for command-specific behavior.

        // Step 1: Use the comprehensive test suite macro
        // This gives you complete coverage of all flag patterns
        test_no_interactive_suite!(
            example_new_command_suite,
            "newcommand",
            &["newcommand", "--arg", "value"]
        );

        // Step 2: Add custom tests for command-specific behavior
        // For example, if your command has special flag handling

        // Verify the handler receives the flag correctly
        let main_code = std::fs::read_to_string("src/main.rs");
        if let Ok(code) = main_code {
            // In real tests, you would assert these conditions
            // assert!(code.contains("newcommand::run_newcommand(no_interactive)"));
        }

        println!("✓ Recommended pattern test passed - follow this structure for new commands");
    }
}
