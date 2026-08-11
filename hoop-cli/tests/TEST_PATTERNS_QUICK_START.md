# HOOP CLI Test Patterns — Quick Start Guide

This guide provides a unified overview of testing the `--no-interactive` flag across HOOP CLI commands. It ties together the utilities from `cli_test_helpers.rs` and `cli_test_utils.rs` into a cohesive reference for contributors.

## Table of Contents

1. [Quick Reference](#quick-reference)
2. [Choosing Your Approach](#choosing-your-approach)
3. [Real-World Scenarios](#real-world-scenarios)
4. [Common Mistakes to Avoid](#common-mistakes-to-avoid)
5. [Complete Example Tests](#complete-example-tests)
6. [Module Reference](#module-reference)

---

## Quick Reference

### Three Levels of Abstraction

```
┌─────────────────────────────────────────────────────────────┐
│ Level 3: Comprehensive Suite Macro (Recommended)            │
│ test_no_interactive_suite! — One test, complete coverage    │
├─────────────────────────────────────────────────────────────┤
│ Level 2: Individual Test Macros (Focused testing)           │
│ test_flag_positions!, test_flag_default_false!, etc.        │
├─────────────────────────────────────────────────────────────┤
│ Level 1: Manual Implementation (Maximum control)          │
│ parse_flag_before_subcommand(), verify_flag_extraction(),   │
│ and other helper functions                                  │
└─────────────────────────────────────────────────────────────┘
```

### When to Use Each Level

| Level | Use Case | Time to Write | Maintenance |
|-------|----------|---------------|-------------|
| **Level 3** | New commands, regression testing, CI/CD | ~30 seconds | Low (macro handles changes) |
| **Level 2** | Debugging specific patterns, granular reports | ~1-2 minutes | Medium (update individual tests) |
| **Level 1** | Custom test logic, learning internals | ~5-10 minutes | High (manual updates needed) |

### One-Line Quick Start

```rust
// For most cases, this is all you need:
test_no_interactive_suite!(test_mycommand_complete, "mycommand", &["mycommand", "--arg"]);
```

---

## Choosing Your Approach

Use this decision tree to pick the right testing approach:

```
Are you testing a new command?
│
├─ Yes → Use test_no_interactive_suite! (Level 3)
│        Add custom tests for command-specific behavior if needed
│
└─ No → Is this a debugging/learning scenario?
           │
           ├─ Yes → Use manual implementation (Level 1)
           │        Step through each parsing stage
           │
           └─ No → Do you need granular failure reports?
                     │
                     ├─ Yes → Use individual macros (Level 2)
                     │        One pattern per test function
                     │
                     └─ No → Use test_no_interactive_suite! (Level 3)
                           Complete coverage in one test
```

### Decision Examples

**Scenario 1: Adding a new `analytics` command**
```rust
#[test]
fn test_analytics_complete() {
    test_no_interactive_suite!(
        test_analytics_complete,
        "analytics",
        &["analytics", "--format", "json"]
    );
}
```

**Scenario 2: Debugging why flag parsing fails for `status`**
```rust
#[test]
fn test_status_debug_flag_parsing() {
    // Manual implementation to see each step
    let args_before = &["--no-interactive", "status", "--json"];
    let parsed_before = parse_flag_before_subcommand(args_before)
        .expect("DEBUG: Failed to parse flag before subcommand");
    
    println!("DEBUG: parsed_before = {:?}", parsed_before);
    println!("DEBUG: no_interactive = {}", parsed_before.no_interactive);
    println!("DEBUG: subcommand = {:?}", parsed_before.subcommand);
    
    // Add more debug assertions as needed
    assert!(assert_flag_is_true(&parsed_before).is_ok());
}
```

**Scenario 3: Testing only flag positions for multiple commands**
```rust
#[test]
fn test_multiple_commands_flag_positions() {
    test_flag_positions!(test_status_positions, "status", &["status", "--json"]);
    test_flag_positions!(test_list_positions, "list", &["list"]);
    test_flag_positions!(test_scan_positions, "scan", &["scan", "/tmp"]);
}
```

---

## Real-World Scenarios

### Scenario 1: Testing a New Safe Command (e.g., `analytics`)

**Requirements:**
- Flag must work before and after the command
- Short flag `-y` must work
- Default behavior must be `false`
- No `--confirm` required (safe operation)

**Solution:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use cli_test_helpers::prelude::*;

    // Use the comprehensive suite macro
    test_no_interactive_suite!(
        test_analytics_complete,
        "analytics",
        &["analytics", "--format", "json"]
    );

    // Optional: Add custom test for format flag interaction
    #[test]
    fn test_analytics_format_flag_with_no_interactive() {
        let parsed = parse_flag_before_subcommand(&[
            "--no-interactive",
            "analytics",
            "--format",
            "json"
        ]).expect("Should parse successfully");

        assert!(parsed.args.contains(&"--format".to_string()));
        assert!(parsed.args.contains(&"json".to_string()));
    }
}
```

### Scenario 2: Testing a Destructive Command (e.g., `projects remove`)

**Requirements:**
- All flag position requirements from Scenario 1
- Must require `--confirm` when `--no-interactive` is set
- Must show helpful error if `--confirm` is missing

**Solution:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use cli_test_helpers::prelude::*;

    // Test flag positions (covers basic parsing)
    test_flag_positions!(
        test_remove_positions,
        "remove",
        &["projects", "remove", "my-project"]
    );

    // Test default behavior
    test_flag_default_false!(
        test_remove_default,
        &["projects", "remove", "my-project"]
    );

    // Test the confirm requirement pattern
    test_confirm_required_pattern!(
        test_remove_confirm_pattern,
        "remove",
        &["projects", "remove", "my-project"]
    );

    // Custom test: Verify source code implements the check
    #[test]
    fn test_remove_confirm_implementation() {
        let code = std::fs::read_to_string("src/projects.rs")
            .expect("Failed to read projects.rs");

        // Must check for confirm flag in non-interactive mode
        assert!(
            code.contains("if no_interactive && !confirm"),
            "Must check --confirm requirement in non-interactive mode"
        );

        // Must show helpful error message
        assert!(
            code.contains("--confirm is required"),
            "Must show helpful error when --confirm is missing"
        );
    }
}
```

### Scenario 3: Testing a Nested Command (e.g., `projects add`, `patterns add`)

**Requirements:**
- Flag must work at all positions (before primary, between primary/nested, after nested)
- Nested subcommand must be correctly identified
- Flag must propagate to the nested handler

**Solution:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use cli_test_helpers::prelude::*;

    // Test nested command flag propagation
    test_nested_flag_propagation!(
        test_projects_add_propagation,
        "projects",
        "add",
        &["projects", "add", "/path/to/project"]
    );

    // Test flag positions for the nested command
    test_flag_positions!(
        test_projects_add_positions,
        "add",
        &["projects", "add", "/path/to/project"]
    );

    // Custom test: Verify all three positions work
    #[test]
    fn test_projects_add_all_flag_positions() {
        // Position 1: Before primary subcommand
        let parsed_1 = parse_nested_subcommand(&[
            "--no-interactive",
            "projects",
            "add",
            "/path/to/project"
        ]).expect("Should parse with flag before primary");
        assert_eq!(parsed_1.no_interactive, true);
        assert_eq!(parsed_1.subcommand, Some("projects".to_string()));
        assert_eq!(parsed_1.nested_subcommand, Some("add".to_string()));

        // Position 2: Between primary and nested
        let parsed_2 = parse_nested_subcommand(&[
            "projects",
            "--no-interactive",
            "add",
            "/path/to/project"
        ]).expect("Should parse with flag between commands");
        assert_eq!(parsed_2.no_interactive, true);

        // Position 3: After nested subcommand
        let parsed_3 = parse_nested_subcommand(&[
            "projects",
            "add",
            "/path/to/project",
            "--no-interactive"
        ]).expect("Should parse with flag after nested");
        assert_eq!(parsed_3.no_interactive, true);

        // All positions should produce the same result
        assert_eq!(parsed_1.no_interactive, parsed_2.no_interactive);
        assert_eq!(parsed_2.no_interactive, parsed_3.no_interactive);
    }
}
```

### Scenario 4: Testing a Command That Rejects `no_interactive` (e.g., `init`)

**Requirements:**
- Command must detect `--no-interactive` flag
- Must exit with error code 2 (fatal/precondition error)
- Must explain why it cannot run non-interactively
- Must suggest manual configuration for automation

**Solution:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_rejects_no_interactive() {
        let code = std::fs::read_to_string("src/init.rs")
            .expect("Failed to read init.rs");

        // Must check the flag early
        assert!(
            code.contains("if no_interactive"),
            "Init must check no_interactive flag"
        );

        // Must exit with error code 2 (fatal/precondition error)
        assert!(
            code.contains("std::process::exit(2)"),
            "Init must exit with error code 2"
        );

        // Must show helpful error message
        assert!(
            code.contains("cannot run in non-interactive mode"),
            "Init must explain why it cannot run non-interactively"
        );

        assert!(
            code.contains("requires interactive input"),
            "Init must state that it requires interactive input"
        );

        assert!(
            code.contains("manually create ~/.hoop/config.yml"),
            "Init must suggest manual configuration for automation"
        );
    }
}
```

### Scenario 5: Complex Multi-Command Test Suite

**Requirements:**
- Test multiple commands in a single test file
- Run all tests with one `cargo test` command
- Share common test fixtures across tests

**Solution:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use cli_test_helpers::prelude::*;
    use cli_test_utils::*;
    use tempfile::TempDir;

    // Shared test fixture
    fn setup_test_workspace() -> TempDir {
        let tmp_dir = TempDir::new().expect("Failed to create temp dir");
        let _workspace = create_test_workspace(&tmp_dir, "test-project");
        let _registry = create_test_registry(&tmp_dir);
        tmp_dir
    }

    // Test all safe commands
    #[test]
    fn test_all_safe_commands_flag_positions() {
        let _tmp = setup_test_workspace();

        test_flag_positions!(test_status, "status", &["status", "--json"]);
        test_flag_positions!(test_list, "list", &["list"]);
        test_flag_positions!(test_scan, "scan", &["scan", "/tmp"]);
    }

    // Test all destructive commands
    #[test]
    fn test_all_destructive_commands_confirm_required() {
        test_confirm_required_pattern!(
            test_remove_confirm,
            "remove",
            &["projects", "remove", "test-project"]
        );

        test_confirm_required_pattern!(
            test_restore_confirm,
            "restore",
            &["restore", "--from", "s3://bucket/key"]
        );
    }

    // Test all nested commands
    #[test]
    fn test_all_nested_commands_flag_propagation() {
        test_nested_flag_propagation!(
            test_projects_add,
            "projects",
            "add",
            &["projects", "add", "/path/to/project"]
        );

        test_nested_flag_propagation!(
            test_patterns_add,
            "patterns",
            "add",
            &["patterns", "add", "pattern-name"]
        );
    }
}
```

---

## Common Mistakes to Avoid

### Mistake 1: Forgetting the Short Flag

**Problem:** Testing only `--no-interactive`, not `-y`

```rust
// ❌ WRONG: Only tests long form
let parsed = parse_flag_before_subcommand(&["--no-interactive", "scan"]);
assert!(parsed.no_interactive);
// Never tests -y!
```

**Solution:** Always test both forms, or use the suite macro

```rust
// ✅ CORRECT: Use suite macro (tests both automatically)
test_no_interactive_suite!(test_scan_complete, "scan", &["scan", "/tmp"]);

// ✅ CORRECT: Test both manually
let parsed_long = parse_flag_before_subcommand(&["--no-interactive", "scan"]);
let parsed_short = parse_flag_before_subcommand(&["-y", "scan"]);
assert!(parsed_long.unwrap().no_interactive);
assert!(parsed_short.unwrap().no_interactive);
```

### Mistake 2: Missing Default Behavior Test

**Problem:** Never testing that the flag defaults to `false`

```rust
// ❌ WRONG: Only tests with flag present
let parsed = parse_flag_before_subcommand(&["--no-interactive", "scan"]);
assert!(parsed.unwrap().no_interactive);
// What if flag is ALWAYS true? This test wouldn't catch it.
```

**Solution:** Always include a default behavior test

```rust
// ✅ CORRECT: Test default behavior
test_flag_default_false!(test_scan_default, &["scan", "/tmp"]);

// ✅ CORRECT: Test manually
let parsed_default = parse_flag_before_subcommand(&["scan", "/tmp"]);
assert!(!parsed_default.unwrap().no_interactive);
```

### Mistake 3: Inconsistent Position Handling

**Problem:** Testing positions in isolation, never comparing them

```rust
// ❌ WRONG: Tests positions separately, never compares
let before = parse_flag_before_subcommand(&["--no-interactive", "scan"]);
let after = parse_flag_after_subcommand(&["scan", "--no-interactive"]);
// Never checks if they produce the same value!
```

**Solution:** Always verify consistency

```rust
// ✅ CORRECT: Use suite macro (includes consistency check)
test_no_interactive_suite!(test_scan_complete, "scan", &["scan", "/tmp"]);

// ✅ CORRECT: Verify consistency manually
assert!(verify_flag_position_consistency(&["scan", "/tmp"]).is_ok());
```

### Mistake 4: Not Testing Flag Propagation

**Problem:** Testing parsing, but not verifying the flag reaches the handler

```rust
// ❌ WRONG: Only tests parsing
let parsed = parse_flag_before_subcommand(&["scan", "/tmp"]).unwrap();
assert!(parsed.no_interactive);
// Never checks if handler actually receives the flag!
```

**Solution:** Always verify flag propagation

```rust
// ✅ CORRECT: Use suite macro (includes propagation check)
test_no_interactive_suite!(test_scan_complete, "scan", &["scan", "/tmp"]);

// ✅ CORRECT: Verify propagation manually
assert!(assert_flag_propagation(&["scan", "/tmp"]).is_ok());

// ✅ CORRECT: Check source code
let main_code = std::fs::read_to_string("src/main.rs").unwrap();
assert!(main_code.contains("scan::run_scan(no_interactive)"));
```

### Mistake 5: Testing Only One Pattern

**Problem:** Using individual macros but forgetting some patterns

```rust
// ❌ WRONG: Only one pattern tested
test_no_interactive_flag_before!(test_scan_before, "scan", &["scan", "/tmp"]);
// Missing: after, short, consistency, default!
```

**Solution:** Use the suite macro, or create a checklist

```rust
// ✅ CORRECT: Use suite macro for complete coverage
test_no_interactive_suite!(test_scan_complete, "scan", &["scan", "/tmp"]);

// ✅ CORRECT: Checklist approach
test_flag_positions!(test_scan_positions, "scan", &["scan", "/tmp"]);
test_flag_default_false!(test_scan_default, &["scan", "/tmp"]);
// Add other patterns as needed
```

### Mistake 6: Not Testing Edge Cases

**Problem:** Only testing happy path, no error handling

```rust
// ❌ WRONG: Only tests valid inputs
let parsed = parse_flag_before_subcommand(&["--no-interactive", "scan"]);
assert!(parsed.is_ok());
// What about empty args? Missing command? Multiple flags?
```

**Solution:** Always include edge case tests

```rust
// ✅ CORRECT: Test edge cases
#[test]
fn test_edge_cases() {
    // Empty arguments
    assert!(parse_flag_before_subcommand(&[]).is_err());

    // Only flag, no command
    let parsed = parse_flag_before_subcommand(&["--no-interactive"]).unwrap();
    assert_eq!(parsed.subcommand, None);

    // Multiple flags
    let parsed = parse_flag_before_subcommand(&[
        "--no-interactive",
        "scan",
        "/tmp",
        "--verbose",
        "--json"
    ]).unwrap();
    assert!(parsed.args.contains(&"--verbose".to_string()));
    assert!(parsed.args.contains(&"--json".to_string()));
}
```

---

## Complete Example Tests

### Example 1: Minimal Complete Test (All Patterns, One Test)

This is the recommended pattern for most cases. One test function, complete coverage.

```rust
#[cfg(test)]
mod tests {
    use cli_test_helpers::prelude::*;

    test_no_interactive_suite!(
        test_mycommand_complete,
        "mycommand",
        &["mycommand", "--arg", "value"]
    );
}
```

**What this tests:**
- ✅ Flag before subcommand: `hoop --no-interactive mycommand --arg value`
- ✅ Flag after subcommand: `hoop mycommand --arg value --no-interactive`
- ✅ Short flag variant: `hoop -y mycommand --arg value`
- ✅ Position independence: Both positions give same value
- ✅ Default behavior: `hoop mycommand --arg value` → no_interactive=false
- ✅ Flag propagation: Flag reaches the handler correctly

### Example 2: Manual Implementation (All Patterns, Step by Step)

For learning or debugging, this shows each step explicitly.

```rust
#[cfg(test)]
mod tests {
    use cli_test_helpers::prelude::*;

    #[test]
    fn test_mycommand_manual_complete() {
        // ── Test 1: Flag before subcommand ────────────────────────────────
        let args_before = &["--no-interactive", "mycommand", "--arg", "value"];
        let parsed_before = parse_flag_before_subcommand(args_before)
            .expect("Should parse flag before subcommand");

        assert_eq!(
            parsed_before.no_interactive,
            true,
            "Flag should be true when before subcommand"
        );
        assert_eq!(
            parsed_before.subcommand,
            Some("mycommand".to_string()),
            "Should identify subcommand correctly"
        );
        assert!(
            assert_flag_is_true(&parsed_before).is_ok(),
            "Flag assertion should pass"
        );

        // ── Test 2: Flag after subcommand ─────────────────────────────────
        let args_after = &["mycommand", "--arg", "value", "--no-interactive"];
        let parsed_after = parse_flag_after_subcommand(args_after)
            .expect("Should parse flag after subcommand");

        assert_eq!(
            parsed_after.no_interactive,
            true,
            "Flag should be true when after subcommand"
        );
        assert_eq!(
            parsed_after.subcommand,
            Some("mycommand".to_string()),
            "Should identify subcommand correctly"
        );
        assert!(
            assert_flag_is_true(&parsed_after).is_ok(),
            "Flag assertion should pass"
        );

        // ── Test 3: Short flag variant ─────────────────────────────────────
        let args_short = &["-y", "mycommand", "--arg", "value"];
        let parsed_short = parse_flag_before_subcommand(args_short)
            .expect("Should parse short flag");

        assert_eq!(
            parsed_short.no_interactive,
            true,
            "Short flag should set no_interactive to true"
        );

        let short_value = extract_flag_value(args_short);
        assert_eq!(
            short_value, true,
            "Direct extraction should find short flag"
        );

        // ── Test 4: Position independence ───────────────────────────────────
        assert_eq!(
            parsed_before.no_interactive,
            parsed_after.no_interactive,
            "Flag value must be consistent regardless of position"
        );

        let base_args = &["mycommand", "--arg", "value"];
        assert!(
            verify_flag_position_consistency(base_args).is_ok(),
            "Position consistency verification should pass"
        );

        // ── Test 5: Default behavior (no flag) ────────────────────────────
        let args_default = &["mycommand", "--arg", "value"];
        let parsed_default = parse_flag_before_subcommand(args_default)
            .expect("Should parse without flag");

        assert_eq!(
            parsed_default.no_interactive,
            false,
            "Flag should default to false when not specified"
        );
        assert!(
            assert_flag_is_false(&parsed_default).is_ok(),
            "Default flag assertion should pass"
        );
        assert!(
            verify_default_flag_value(args_default).is_ok(),
            "Default value verification should pass"
        );

        // ── Test 6: Flag propagation ────────────────────────────────────────
        assert!(
            assert_flag_propagation(base_args).is_ok(),
            "Flag should propagate correctly through handler chain"
        );

        // ── Summary ───────────────────────────────────────────────────────────
        println!("✓ All manual tests passed - mycommand handles flag correctly");
    }
}
```

### Example 3: Integration Test with Fixtures

For commands that need actual files/directories to test.

```rust
#[cfg(test)]
mod tests {
    use cli_test_helpers::prelude::*;
    use cli_test_utils::*;
    use tempfile::TempDir;

    #[test]
    fn test_scan_integration_complete() {
        // ── Setup: Create test workspace ────────────────────────────────────
        let tmp_dir = TempDir::new().expect("Failed to create temp dir");
        let workspace = create_test_workspace(&tmp_dir, "test-project");
        let registry_path = create_test_registry(&tmp_dir);

        assert!(workspace.exists(), "Workspace should exist");
        assert!(workspace.join(".beads").exists(), ".beads directory should exist");
        assert!(registry_path.exists(), "Registry file should exist");

        // ── Test 1: Parse with flag before subcommand ───────────────────────
        let scan_path = tmp_dir.path().to_str().unwrap();
        let args_before = &["--no-interactive", "scan", scan_path];
        let parsed_before = parse_flag_before_subcommand(args_before)
            .expect("Should parse scan with flag before");

        assert_eq!(parsed_before.no_interactive, true);
        assert_eq!(parsed_before.subcommand, Some("scan".to_string()));

        // ── Test 2: Parse with flag after subcommand ─────────────────────────
        let args_after = &["scan", scan_path, "--no-interactive"];
        let parsed_after = parse_flag_after_subcommand(args_after)
            .expect("Should parse scan with flag after");

        assert_eq!(parsed_after.no_interactive, true);
        assert_eq!(parsed_after.subcommand, Some("scan".to_string()));

        // ── Test 3: Verify consistency ───────────────────────────────────────
        assert_eq!(
            parsed_before.no_interactive,
            parsed_after.no_interactive,
            "Flag must be consistent at both positions"
        );

        // ── Test 4: Test short flag ───────────────────────────────────────────
        let args_short = &["-y", "scan", scan_path];
        let parsed_short = parse_flag_before_subcommand(args_short)
            .expect("Should parse scan with short flag");

        assert_eq!(parsed_short.no_interactive, true);

        // ── Test 5: Test default behavior ─────────────────────────────────────
        let args_default = &["scan", scan_path];
        let parsed_default = parse_flag_before_subcommand(args_default)
            .expect("Should parse scan without flag");

        assert_eq!(parsed_default.no_interactive, false);

        // ── Test 6: Test prompt suppression (safe operation) ─────────────────
        let prompt = MockYesNoPrompt {
            text: "Register discovered workspace?".to_string(),
            requires_confirm: false, // Safe operation
        };

        assert!(
            verify_prompt_suppressed(&prompt, true).is_ok(),
            "Prompt should be suppressed with no_interactive=true"
        );

        // ── Summary ───────────────────────────────────────────────────────────
        println!("✓ Integration test passed - scan command works correctly with fixtures");
    }
}
```

### Example 4: Complex Multi-Command Scenario

For testing multiple commands with shared setup and teardown.

```rust
#[cfg(test)]
mod tests {
    use cli_test_helpers::prelude::*;
    use cli_test_utils::*;
    use tempfile::TempDir;

    // Shared fixture setup
    fn setup() -> TempDir {
        let tmp_dir = TempDir::new().expect("Failed to create temp dir");
        let _workspace = create_test_workspace(&tmp_dir, "multi-test");
        let _registry = create_test_registry(&tmp_dir);
        tmp_dir
    }

    // Test all safe commands with flag positions
    #[test]
    fn test_safe_commands_flag_positions() {
        let _tmp = setup();

        // Test multiple safe commands
        let test_cases = vec![
            ("status", &["status", "--json"] as &[&str]),
            ("list", &["list"]),
            ("scan", &["scan", "/tmp"]),
        ];

        for (name, args) in test_cases {
            println!("Testing {} command...", name);

            // Test flag before
            let args_before = ["--no-interactive"].iter().chain(args.iter()).copied().collect::<Vec<_>>();
            let parsed_before = parse_flag_before_subcommand(&args_before)
                .expect(&format!("{} should parse with flag before", name));
            assert!(parsed_before.no_interactive);

            // Test flag after
            let args_after = args.iter().chain(&["--no-interactive"]).copied().collect::<Vec<_>>();
            let parsed_after = parse_flag_after_subcommand(&args_after)
                .expect(&format!("{} should parse with flag after", name));
            assert!(parsed_after.no_interactive);

            println!("✓ {} passed", name);
        }
    }

    // Test all destructive commands with confirm requirement
    #[test]
    fn test_destructive_commands_confirm_required() {
        let _tmp = setup();

        // Test multiple destructive commands
        let test_cases = vec![
            ("remove", &["projects", "remove", "test-project", "--confirm"] as &[&str]),
            ("restore", &["restore", "--from", "s3://bucket/key", "--confirm"]),
        ];

        for (name, args) in test_cases {
            println!("Testing {} command...", name);

            // Parse with both flags (valid combination)
            let args_valid = args.iter().chain(&["--no-interactive"]).copied().collect::<Vec<_>>();
            let parsed_valid = parse_flag_after_subcommand(&args_valid)
                .expect(&format!("{} should parse with --no-interactive --confirm", name));
            assert!(parsed_valid.no_interactive);
            assert!(parsed_valid.args.contains(&"--confirm".to_string()));

            // Parse without --confirm (should parse, but will error in real code)
            let args_invalid: Vec<&str> = args.iter().filter(|&&a| a != "--confirm").copied().collect();
            let args_with_flag = args_invalid.iter().chain(&["--no-interactive"]).copied().collect::<Vec<_>>();
            let parsed_invalid = parse_flag_after_subcommand(&args_with_flag)
                .expect(&format!("{} should parse (even without --confirm)", name));
            assert!(parsed_invalid.no_interactive);
            assert!(!parsed_invalid.args.contains(&"--confirm".to_string()));

            println!("✓ {} passed", name);
        }
    }

    // Test all nested commands with flag propagation
    #[test]
    fn test_nested_commands_flag_propagation() {
        let _tmp = setup();

        // Test multiple nested commands
        let test_cases = vec![
            ("projects", "add", &["projects", "add", "/path/to/project"] as &[&str]),
            ("patterns", "add", &["patterns", "add", "pattern-name"]),
        ];

        for (primary, nested, args) in test_cases {
            println!("Testing {} {} nested command...", primary, nested);

            // Parse with flag before primary
            let args_before = ["--no-interactive"].iter().chain(args.iter()).copied().collect::<Vec<_>>();
            let parsed_before = parse_nested_subcommand(&args_before)
                .expect(&format!("{} {} should parse with flag before primary", primary, nested));
            assert_eq!(parsed_before.subcommand, Some(primary.to_string()));
            assert_eq!(parsed_before.nested_subcommand, Some(nested.to_string()));
            assert!(parsed_before.no_interactive);

            // Parse with flag after nested
            let args_after = args.iter().chain(&["--no-interactive"]).copied().collect::<Vec<_>>();
            let parsed_after = parse_nested_subcommand(&args_after)
                .expect(&format!("{} {} should parse with flag after nested", primary, nested));
            assert_eq!(parsed_after.subcommand, Some(primary.to_string()));
            assert_eq!(parsed_after.nested_subcommand, Some(nested.to_string()));
            assert!(parsed_after.no_interactive);

            println!("✓ {} {} passed", primary, nested);
        }
    }
}
```

---

## Module Reference

### cli_test_helpers.rs (High-Level Patterns)

**Purpose:** Command-specific testing patterns and comprehensive test macros

**Key Exports:**
```rust
use cli_test_helpers::prelude::*;

// Parsing utilities
parse_flag_before_subcommand(&["scan", "/tmp"])
parse_flag_after_subcommand(&["scan", "/tmp"])
parse_nested_subcommand(&["projects", "remove", "test"])
extract_flag_value(&["scan", "-y"])
extract_subcommand(&["scan", "/tmp"])

// Verification utilities
assert_flag_is_true(&parsed)
assert_flag_is_false(&parsed)
assert_flag_value(&parsed, true)
assert_flag_propagation(&["scan", "/tmp"])
verify_flag_position_consistency(&["scan", "/tmp"])
verify_default_flag_value(&["scan", "/tmp"])

// Macros (recommended for most cases)
test_no_interactive_suite!(test_name, "command", &["command", "--arg"])
test_flag_positions!(test_name, "command", &["command", "--arg"])
test_flag_default_false!(test_name, &["command", "--arg"])
test_nested_flag_propagation!(test_name, "primary", "nested", &["primary", "nested"])
test_confirm_required_pattern!(test_name, "operation", &["operation", "--arg"])
```

**When to use:**
- ✅ Adding tests for a new command
- ✅ Regression testing with minimal boilerplate
- ✅ Quick coverage in CI/CD pipelines
- ✅ Testing flag propagation through handlers

### cli_test_utils.rs (Low-Level Utilities)

**Purpose:** Basic parsing functions and verification utilities

**Key Exports:**
```rust
use cli_test_utils::*;

// Parsing utilities
parse_cli_with_flag(&["hoop", "--no-interactive", "scan", "/tmp"])
parse_flag_before_subcommand(&["scan", "/tmp"])
parse_flag_after_subcommand(&["scan", "/tmp"])

// Verification utilities
verify_flag_extraction(&parsed, "before")
verify_no_flag_present(&parsed)
verify_prompt_suppressed(&prompt, true)
verify_confirm_required(&prompt, true, true)

// Test fixtures
create_test_workspace(&tmp_dir, "project-name")
create_test_workspace(&tmp_dir, "project-name")
create_hoop_config_dir(&tmp_dir)
create_test_registry(&tmp_dir)

// Batch testing
run_flag_position_tests(test_cases)

// Macros
test_no_interactive_flag_before!(test_name, "command", &["command", "--arg"])
test_no_interactive_flag_after!(test_name, "command", &["command", "--arg"])
test_short_flag_y!(test_name, &["command", "--arg"])
test_both_positions_consistency!(test_name, &["command", "--arg"])
test_flag_default_false!(test_name, &["command", "--arg"])
test_command_no_interactive_suite!(test_name, "command", &["command", "--arg"])
```

**When to use:**
- ✅ Debugging a specific flag parsing issue
- ✅ Writing one-off tests for unique scenarios
- ✅ Learning how the flag parsing works internally
- ✅ Complex scenarios requiring custom assertions

### Choosing Between Modules

| Use Case | Use This Module | Why |
|----------|----------------|-----|
| New command test suite | `cli_test_helpers` | Comprehensive macros, less boilerplate |
| Debugging flag parsing | `cli_test_utils` | Low-level control, see each step |
| Testing prompt behavior | `cli_test_utils` | Mock prompt interfaces |
| Testing flag propagation | `cli_test_helpers` | Built-in propagation verification |
| Custom test logic | `cli_test_utils` | More flexibility in assertions |
| Integration tests with fixtures | `cli_test_utils` | Test fixtures for workspaces/configs |

---

## Additional Resources

### Running Tests

```bash
# Run all CLI tests
cargo test --package hoop-cli --tests

# Run specific test file
cargo test --package hoop-cli --test cli_test_helpers

# Run specific test
cargo test --package hoop-cli test_mycommand_complete

# Run tests with output
cargo test --package hoop-cli --tests -- --nocapture

# Run tests in verbose mode
cargo test --package hoop-cli --tests -- --verbose
```

### Viewing Documentation

```bash
# Generate and open documentation
cargo doc --package hoop-cli --open
```

### Related Documentation

- `CLI_TEST_UTILS_README.md` — Low-level utilities reference
- `CLAP_TEST_UTILS_README.md` — Clap-specific testing patterns
- `no_interactive_flag_audit.md` — Flag behavior audit and patterns
- `SCAN_NO_INTERACTIVE_TEST_COVERAGE.md` — Example test coverage for `scan` command

### Example Test Files

- `cli_test_helpers.rs` — High-level patterns and macros (see module tests)
- `cli_test_utils.rs` — Low-level utilities (see module tests)
- `cli_test_utils_examples.rs` — Comprehensive example tests
- `no_interactive_flag_behavior.rs` — Integration tests for flag behavior

---

## Summary

This quick start guide provides:

✅ **Quick reference** for all testing levels and patterns
✅ **Decision tree** for choosing the right approach
✅ **Real-world scenarios** for common testing situations
✅ **Common mistakes** and how to avoid them
✅ **Complete examples** from minimal to comprehensive
✅ **Module reference** for both `cli_test_helpers` and `cli_test_utils`

For most cases, use the **comprehensive suite macro**:

```rust
test_no_interactive_suite!(test_mycommand_complete, "mycommand", &["mycommand", "--arg"]);
```

This one line gives you complete coverage of all flag patterns in a single test function.
