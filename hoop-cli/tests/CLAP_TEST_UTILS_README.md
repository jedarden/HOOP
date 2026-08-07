# Clap-Based Test Utilities

This directory contains clap-based test utilities for testing CLI commands with real clap parsing, ensuring tests match actual runtime behavior.

## Overview

The `clap_test_utils.rs` module provides comprehensive helpers for:

- **Real clap parsing**: Uses `Cli::try_parse_from()` for actual CLI behavior
- **Flag position testing**: Test `--no-interactive` at any position
- **Command extraction**: Access parsed commands and their arguments
- **Verification helpers**: Ensure flags are correctly extracted
- **Test macros**: Generate common test patterns for any command

## Key Difference from `cli_test_utils.rs`

- **`cli_test_utils.rs`**: Custom string-based parser (lightweight, doesn't validate against real CLI)
- **`clap_test_utils.rs`**: Uses actual clap parsing from the CLI (tests real behavior, validates argument structure)

**Use `clap_test_utils.rs` when:**
- You need to test actual CLI behavior
- You want to verify command-specific arguments are parsed correctly
- You need to access the full `Cli` structure

**Use `cli_test_utils.rs` when:**
- You only need to test flag extraction logic
- You want lightweight tests without full clap validation
- You're testing patterns that don't require full CLI parsing

## Core Structures

### Parsing Functions

**`parse_cli(args: &[&str]) -> Result<Cli, clap::Error>`**
Parse CLI arguments using actual clap parsing.

```rust
let cli = parse_cli(&["hoop", "--no-interactive", "scan", "/tmp"])?;
assert_eq!(cli.no_interactive, true);
```

**`parse_flag_before_subcommand(cmd_args: &[&str]) -> Result<Cli, clap::Error>`**
Parse with flag before subcommand (`hoop --no-interactive scan /tmp`).

```rust
let cli = parse_flag_before_subcommand(&["scan", "/tmp"])?;
assert_eq!(cli.no_interactive, true);
```

**`parse_flag_after_subcommand(cmd_args: &[&str]) -> Result<Cli, clap::Error>`**
Parse with flag after subcommand (`hoop scan /tmp --no-interactive`).

```rust
let cli = parse_flag_after_subcommand(&["scan", "/tmp"])?;
assert_eq!(cli.no_interactive, true);
```

**`parse_both_positions(flag_args: &[&str], cmd_args: &[&str]) -> (bool, bool)`**
Parse with flag in both positions, return both values.

```rust
let (before, after) = parse_both_positions(&["--no-interactive"], &["scan", "/tmp"]);
assert_eq!(before, after);
```

### Verification Functions

**`verify_no_interactive_value(cli: &Cli, expected: bool) -> Result<(), String>`**
Verify the parsed CLI has the expected no_interactive value.

```rust
let cli = parse_cli(&["hoop", "--no-interactive", "scan"])?;
assert!(verify_no_interactive_value(&cli, true).is_ok());
```

**`verify_flag_at_position(cmd_args: &[&str], position: &str, expected_value: bool) -> Result<(), String>`**
Verify flag is correctly parsed at a specific position.

```rust
assert!(verify_flag_at_position(&["scan", "/tmp"], "before", true).is_ok());
```

**`verify_position_independence(cmd_args: &[&str]) -> Result<(), String>`**
Verify both positions yield the same value.

```rust
assert!(verify_position_independence(&["scan", "/tmp"]).is_ok());
```

### Command Extraction

**`get_command(cli: &Cli) -> &Commands`**
Extract the command variant from parsed CLI.

```rust
let cli = parse_cli(&["hoop", "scan", "/tmp"])?;
let command = get_command(&cli);
```

**`try_get_scan_command(cli: &Cli) -> Option<&Commands>`**
Try to extract a specific command variant.

```rust
if let Some(Commands::Scan { root, .. }) = try_get_scan_command(&cli) {
    assert_eq!(root, "/tmp");
}
```

## Test Macros

### Individual Test Macros

**`test_flag_before!`**
Generate a test for flag parsing before the command.

```rust
test_flag_before!(scan_flag_before, &["scan", "/tmp"]);
```

**`test_flag_after!`**
Generate a test for flag parsing after the command.

```rust
test_flag_after!(scan_flag_after, &["scan", "/tmp"]);
```

**`test_short_flag!`**
Generate a test for short -y flag.

```rust
test_short_flag!(scan_short_y, &["scan", "/tmp"]);
```

**`test_position_independence!`**
Generate a test for position independence.

```rust
test_position_independence!(scan_consistency, &["scan", "/tmp"]);
```

**`test_flag_default!`**
Generate a test for default flag value (false).

```rust
test_flag_default!(scan_default, &["scan", "/tmp"]);
```

### Complete Test Suite Macro

**`test_command_no_interactive_suite!`**
Generate 5 tests covering all patterns:
1. Flag before command
2. Flag after command
3. Short flag -y
4. Position independence
5. Default (no flag)

```rust
test_command_no_interactive_suite!(scan, &["scan", "/tmp"]);
test_command_no_interactive_suite!(remove, &["remove", "test", "--confirm"]);
test_command_no_interactive_suite!(init, &["init"]);
```

## Batch Testing

**`ClapTestCase`**
Test case structure for batch testing.

```rust
ClapTestCase {
    description: "scan with flag before".to_string(),
    args: vec!["hoop", "--no-interactive", "scan", "/tmp"]
        .iter().map(|s| s.to_string()).collect(),
    expected_no_interactive: true,
    should_parse: true,
}
```

**`run_clap_tests(test_cases: Vec<ClapTestCase>) -> (Vec<String>, Vec<(String, String)>)`**
Run batch of tests and return success/failure summaries.

```rust
let test_cases = vec![
    // ... test cases
];
let (successes, failures) = run_clap_tests(test_cases);
assert_eq!(failures.len(), 0);
```

## Usage Examples

### Example 1: Testing a Single Command

```rust
use clap_test_utils::*;

#[test]
fn test_scan_no_interactive() {
    // Test flag before command
    let cli = parse_flag_before_subcommand(&["scan", "/tmp"]).unwrap();
    assert_eq!(cli.no_interactive, true);

    // Test flag after command
    let cli = parse_flag_after_subcommand(&["scan", "/tmp"]).unwrap();
    assert_eq!(cli.no_interactive, true);

    // Test position independence
    assert!(parse_both_positions_yield_same_value(
        &["scan", "/tmp"],
        &["--no-interactive"]
    ));

    // Test default value
    let cli = parse_cli(&["hoop", "scan", "/tmp"]).unwrap();
    assert_eq!(cli.no_interactive, false);
}
```

### Example 2: Using Test Macros

```rust
use clap_test_utils::*;

// Generate complete test suite for a command
test_command_no_interactive_suite!(scan, &["scan", "/tmp"]);
test_command_no_interactive_suite!(remove, &["remove", "test", "--confirm"]);
test_command_no_interactive_suite!(init, &["init"]);
```

### Example 3: Verifying Command-Specific Arguments

```rust
use clap_test_utils::*;

#[test]
fn test_scan_command_arguments() {
    let cli = parse_cli(&["hoop", "--no-interactive", "scan", "/tmp"]).unwrap();

    // Verify global flag
    assert_eq!(cli.no_interactive, true);

    // Verify command-specific arguments
    match cli.command {
        Commands::Scan { root, auto_confirm } => {
            assert_eq!(root, "/tmp");
            assert_eq!(auto_confirm, false);
        }
        _ => panic!("Expected Scan command"),
    }
}
```

### Example 4: Batch Testing Multiple Commands

```rust
use clap_test_utils::*;

#[test]
fn test_all_commands_no_interactive() {
    let test_cases = vec![
        ClapTestCase {
            description: "scan with flag".to_string(),
            args: vec!["hoop", "--no-interactive", "scan", "/tmp"]
                .iter().map(|s| s.to_string()).collect(),
            expected_no_interactive: true,
            should_parse: true,
        },
        ClapTestCase {
            description: "remove with flag".to_string(),
            args: vec!["hoop", "--no-interactive", "remove", "test", "--confirm"]
                .iter().map(|s| s.to_string()).collect(),
            expected_no_interactive: true,
            should_parse: true,
        },
        // ... more test cases
    ];

    let (successes, failures) = run_clap_tests(test_cases);
    assert_eq!(failures.len(), 0, "Some tests failed: {:?}", failures);
}
```

### Example 5: Testing Nested Subcommands

```rust
use clap_test_utils::*;

#[test]
fn test_projects_scan_subcommand() {
    let cli = parse_cli(&["hoop", "--no-interactive", "projects", "scan", "/tmp"]).unwrap();

    assert_eq!(cli.no_interactive, true);

    match cli.command {
        Commands::Projects(cmd) => match cmd {
            hoop_cli::main::ProjectsCommands::Scan { root, .. } => {
                assert_eq!(root, "/tmp");
            }
            _ => panic!("Expected Projects::Scan"),
        },
        _ => panic!("Expected Projects command"),
    }
}
```

## Running Tests

Run all tests including clap_test_utils:
```bash
cargo test --package hoop-cli --test clap_test_utils
```

Run specific test:
```bash
cargo test --package hoop-cli --test clap_test_utils test_scan_position_independence
```

Run with output:
```bash
cargo test --package hoop-cli --test clap_test_utils -- --nocapture
```

## Design Principles

1. **Real behavior**: Uses actual clap parsing, not mock parsers
2. **Position independence**: Tests verify flags work in any position
3. **Type safety**: Leverages Rust's type system for command matching
4. **Comprehensive coverage**: Macros test all flag positions and defaults
5. **Reusability**: Helpers work across different commands

## Integration with Other Test Utilities

This module complements `cli_test_utils.rs`:

- **Use both**: Test flag extraction with `cli_test_utils`, verify behavior with `clap_test_utils`
- **Different strengths**: `cli_test_utils` for lightweight pattern tests, `clap_test_utils` for comprehensive CLI testing
- **No conflicts**: Both can run together without interference

## Common Patterns

### Pattern 1: Flag Position Testing

```rust
#[test]
fn test_command_position_independence() {
    // Before
    let cli_before = parse_flag_before_subcommand(&["cmd", "arg"]).unwrap();
    assert_eq!(cli_before.no_interactive, true);

    // After
    let cli_after = parse_flag_after_subcommand(&["cmd", "arg"]).unwrap();
    assert_eq!(cli_after.no_interactive, true);

    // Consistency
    assert_eq!(cli_before.no_interactive, cli_after.no_interactive);
}
```

### Pattern 2: Command Argument Verification

```rust
#[test]
fn test_command_arguments() {
    let cli = parse_cli(&["hoop", "cmd", "arg1", "arg2"]).unwrap();

    match cli.command {
        Commands::MyCommand { arg1, arg2 } => {
            assert_eq!(arg1, "arg1");
            assert_eq!(arg2, "arg2");
        }
        _ => panic!("Unexpected command"),
    }
}
```

### Pattern 3: Global Flag Persistence

```rust
#[test]
fn test_global_flag_persists() {
    let cli = parse_cli(&["hoop", "--no-interactive", "subcmd", "subsubcmd", "arg"]).unwrap();
    assert_eq!(cli.no_interactive, true);
}
```

### Pattern 4: Default Behavior Testing

```rust
#[test]
fn test_default_behavior() {
    let cli = parse_cli(&["hoop", "cmd", "arg"]).unwrap();
    assert_eq!(cli.no_interactive, false);
}
```

## Tips and Best Practices

1. **Start with macros**: Use `test_command_no_interactive_suite!` for comprehensive coverage
2. **Test command args**: Always verify command-specific arguments are parsed correctly
3. **Check position independence**: Ensure flags work in both positions
4. **Test edge cases**: Test combinations like `--no-interactive --yes` for scan command
5. **Use batch testing**: Test multiple commands efficiently with `run_clap_tests`
6. **Verify defaults**: Always test that omitting the flag yields the default value

## Troubleshooting

**"Cannot find Cli or Commands in this scope"**
- Ensure you've imported: `use clap_test_utils::*;`
- These types are re-exported from `hoop_cli::main`

**"Expected function, found macro"**
- Macro invocations need proper semicolon
- Check macro syntax matches examples

**Tests fail with "Parse error"**
- Verify arguments match actual CLI structure
- Check for typos in command names or arguments
- Use `--nocapture` to see detailed error messages

**Position independence test fails**
- Ensure the flag is truly global (has `global = true` in clap definition)
- Check for conflicting local flags with same name
