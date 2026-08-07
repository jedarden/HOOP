# CLI Test Utilities

This directory contains reusable test utilities for testing CLI behavior in the HOOP project, with a focus on testing the `--no-interactive` flag across different commands.

## Overview

The `cli_test_utils.rs` module provides comprehensive helpers for:

- **Parsing CLI commands** with the `--no-interactive` flag at any position
- **Verifying flag extraction** from parsed command arguments
- **Testing prompt suppression** using mockable prompt interfaces
- **Creating test fixtures** for integration testing
- **Batch testing** multiple command scenarios

## Main Module: `cli_test_utils.rs`

### Core Structures

#### `ParsedCli`
Represents a parsed CLI command with the `--no-interactive` flag extracted:

```rust
pub struct ParsedCli {
    pub no_interactive: bool,        // Whether the flag was present
    pub command: String,              // The command that was parsed
    pub args: Vec<String>,           // Remaining arguments
    pub raw_args: Vec<String>,        // Original input args
}
```

### Key Functions

#### Parsing Functions

**`parse_cli_with_flag(args: &[&str]) -> Result<ParsedCli, String>`**
Parse CLI commands and extract the `--no-interactive` flag from any position.

```rust
let result = parse_cli_with_flag(&["hoop", "--no-interactive", "scan", "/tmp"]);
assert!(result.is_ok());
```

**`parse_flag_before_subcommand(command_args: &[&str]) -> Result<ParsedCli, String>`**
Parse with flag before the subcommand (e.g., `hoop --no-interactive scan /tmp`).

```rust
let result = parse_flag_before_subcommand(&["scan", "/tmp"]);
assert!(result.is_ok());
```

**`parse_flag_after_subcommand(command_args: &[&str]) -> Result<ParsedCli, String>`**
Parse with flag after the subcommand (e.g., `hoop scan /tmp --no-interactive`).

```rust
let result = parse_flag_after_subcommand(&["scan", "/tmp"]);
assert!(result.is_ok());
```

#### Verification Functions

**`verify_flag_extraction(parsed: &ParsedCli, expected_position: &str) -> Result<(), String>`**
Verify that the flag was correctly extracted from the expected position.

```rust
let parsed = parse_flag_before_subcommand(&["remove", "test", "--confirm"])?;
assert!(verify_flag_extraction(&parsed, "before").is_ok());
```

**`verify_no_flag_present(parsed: &ParsedCli) -> Result<(), String>`**
Verify that the `--no-interactive` flag is correctly detected as absent.

```rust
let parsed = parse_cli_with_flag(&["hoop", "scan", "/tmp"])?;
assert!(verify_no_flag_present(&parsed).is_ok());
```

#### Prompt Testing Functions

**`verify_prompt_suppressed(prompt: &dyn MockPrompt, no_interactive: bool) -> Result<(), String>`**
Test that a prompt is suppressed when `--no-interactive` is true.

```rust
let prompt = MockYesNoPrompt {
    text: "Continue?".to_string(),
    requires_confirm: false,
};
assert!(verify_prompt_suppressed(&prompt, true).is_ok());
```

**`verify_confirm_required(prompt: &dyn MockPrompt, no_interactive: bool, confirm: bool) -> Result<(), String>`**
Test that destructive operations require the `--confirm` flag in non-interactive mode.

```rust
let prompt = MockYesNoPrompt {
    text: "Remove project?".to_string(),
    requires_confirm: true,
};

// Should fail: no_interactive=true but confirm=false
assert!(verify_confirm_required(&prompt, true, false).is_err());

// Should succeed: no_interactive=true and confirm=true
assert!(verify_confirm_required(&prompt, true, true).is_ok());
```

#### Test Fixtures

**`create_test_workspace(tmp_dir: &TempDir, name: &str) -> PathBuf`**
Create a temporary workspace with a `.beads` directory.

```rust
let tmp_dir = TempDir::new()?;
let workspace = create_test_workspace(&tmp_dir, "test-project");
assert!(workspace.join(".beads").exists());
```

**`create_test_registry(tmp_dir: &TempDir) -> PathBuf`**
Create a temporary `projects.yaml` registry.

```rust
let tmp_dir = TempDir::new()?;
let registry_path = create_test_registry(&tmp_dir);
assert!(registry_path.exists());
```

#### Batch Testing

**`run_flag_position_tests(test_cases: Vec<FlagPositionTestCase>) -> (Vec<String>, Vec<(String, String)>)`**
Run a batch of flag position tests and return success/failure summaries.

```rust
let test_cases = vec![
    FlagPositionTestCase {
        description: "Scan with flag before".to_string(),
        command: vec!["hoop", "--no-interactive", "scan", "/tmp"],
        expected_result: true,
    },
    // ... more test cases
];

let (successes, failures) = run_flag_position_tests(test_cases);
assert_eq!(failures.len(), 0);
```

## Examples Module: `cli_test_utils_examples.rs`

This module contains 22 comprehensive examples demonstrating how to use all the utilities:

1. **Basic parsing examples** - Parse commands with flags before/after subcommands
2. **Helper function examples** - Use the specialized helper functions
3. **Verification examples** - Verify flag extraction and absence
4. **Prompt suppression examples** - Test prompt behavior with flags
5. **Destructive operation examples** - Test `--confirm` requirements
6. **Batch testing examples** - Run multiple test cases efficiently
7. **Test fixture examples** - Create temporary workspaces and registries
8. **Integration test examples** - Combine fixtures and flag testing
9. **Error handling examples** - Test error conditions and edge cases
10. **Comprehensive end-to-end example** - Complete test combining all utilities

Run the examples with:
```bash
cargo test --package hoop --test cli_test_utils_examples -- --nocapture
```

## Usage Patterns

### Testing Flag Position Independence

```rust
#[test]
fn test_scan_with_flag_before() {
    let result = parse_flag_before_subcommand(&["scan", "/tmp"]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().no_interactive, true);
}

#[test]
fn test_scan_with_flag_after() {
    let result = parse_flag_after_subcommand(&["scan", "/tmp"]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().no_interactive, true);
}
```

### Testing Prompt Suppression

```rust
#[test]
fn test_safe_operation_auto_proceeds() {
    let prompt = MockYesNoPrompt {
        text: "Register workspace?".to_string(),
        requires_confirm: false, // Safe operation
    };

    // Should auto-proceed when no_interactive=true
    assert!(verify_prompt_suppressed(&prompt, true).is_ok());
}

#[test]
fn test_destructive_operation_requires_confirm() {
    let prompt = MockYesNoPrompt {
        text: "Delete project?".to_string(),
        requires_confirm: true, // Destructive operation
    };

    // Should require --confirm flag
    assert!(verify_confirm_required(&prompt, true, false).is_err());
    assert!(verify_confirm_required(&prompt, true, true).is_ok());
}
```

### Integration Testing with Fixtures

```rust
#[test]
fn test_scan_integration() {
    let tmp_dir = TempDir::new()?;
    let workspace = create_test_workspace(&tmp_dir, "test-project");
    let _registry = create_test_registry(&tmp_dir);

    let result = parse_cli_with_flag(&[
        "hoop", "--no-interactive", "scan",
        tmp_dir.path().to_str().unwrap()
    ]);

    assert!(result.is_ok());
    assert!(workspace.join(".beads").exists());
}
```

## Running Tests

Run all CLI tests including the utilities:
```bash
cargo test --package hoop
```

Run only the utilities module tests:
```bash
cargo test --package hoop --test cli_test_utils
```

Run the examples to see demonstrations:
```bash
cargo test --package hoop --test cli_test_utils_examples -- --nocapture
```

Run the existing behavioral tests:
```bash
cargo test --package hoop --test no_interactive_flag_behavior
```

## Design Principles

1. **Position Independence**: Test that `--no-interactive` works in both positions
2. **Mockable Prompts**: Test prompt logic without actual user interaction
3. **Safe vs Destructive**: Different patterns for safe vs destructive operations
4. **Comprehensive Coverage**: Test all major commands and edge cases
5. **Reusability**: Helpers can be used across different test files

## Relationship to Existing Tests

These utilities complement the existing `no_interactive_flag_behavior.rs` tests:
- **Existing tests**: Focus on code structure verification and patterns
- **New utilities**: Focus on parsing logic, verification helpers, and mockable interfaces
- **Both sets**: Can run together without conflicts and provide comprehensive coverage

The utilities are designed to be used by future test files that need to test the `--no-interactive` flag behavior across different commands.
