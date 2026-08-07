//! Integration and acceptance tests for HOOP
//!
//! This module organizes HOOP's test suite into:
//! - **CLI test helpers**: Reusable utilities for testing CLI commands
//! - **Acceptance tests**: End-to-end scenario tests (S1-S6)

mod cli_test_helpers;

pub use cli_test_helpers::{
    // Core parsing utilities
    parse_cli_args,
    parse_cmd_string,
    parse_both_positions,

    // Assertion helpers
    assert_no_interactive_true,
    assert_no_interactive_false,
    assert_position_independence,

    // Types
    CliResult,
};

// Re-export acceptance tests
pub mod acceptance;