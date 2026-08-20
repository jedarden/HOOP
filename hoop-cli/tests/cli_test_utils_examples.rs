//! Example tests demonstrating the CLI test utilities
//!
//! This file provides comprehensive examples of how to use the cli_test_utils
//! module for testing the no_interactive flag across different commands.

mod cli_test_utils;

use cli_test_utils::*;
use tempfile::TempDir;

// ── Basic parsing tests ───────────────────────────────────────────────────────

#[test]
fn example_parse_scan_with_flag_before_command() {
    // Example 1: Parse with flag before the subcommand
    let result = parse_cli_with_flag(&["hoop", "--no-interactive", "scan", "/tmp"]);
    assert!(result.is_ok());

    let parsed = result.unwrap();
    assert!(parsed.no_interactive);
    assert_eq!(parsed.command, "scan");
    assert!(parsed.args.contains(&"/tmp".to_string()));
}

#[test]
fn example_parse_scan_with_flag_after_command() {
    // Example 2: Parse with flag after the subcommand
    let result = parse_cli_with_flag(&["hoop", "scan", "/tmp", "--no-interactive"]);
    assert!(result.is_ok());

    let parsed = result.unwrap();
    assert!(parsed.no_interactive);
    assert_eq!(parsed.command, "scan");
    assert!(parsed.args.contains(&"/tmp".to_string()));
}

#[test]
fn example_parse_remove_with_confirm() {
    // Example 3: Parse destructive command with confirm flag
    let result = parse_cli_with_flag(&[
        "hoop",
        "--no-interactive",
        "projects",
        "remove",
        "test-project",
        "--confirm",
    ]);
    assert!(result.is_ok());

    let parsed = result.unwrap();
    assert!(parsed.no_interactive);
    assert_eq!(parsed.command, "projects");
    assert!(parsed.args.contains(&"test-project".to_string()));
    assert!(parsed.args.contains(&"--confirm".to_string()));
}

// ── Helper function tests ───────────────────────────────────────────────────────

#[test]
fn example_use_flag_before_subcommand_helper() {
    // Example 4: Using the helper for flag before subcommand
    let result = parse_flag_before_subcommand(&["scan", "/tmp"]);
    assert!(result.is_ok());

    let parsed = result.unwrap();
    assert!(parsed.no_interactive);
    assert_eq!(parsed.command, "scan");
}

#[test]
fn example_use_flag_after_subcommand_helper() {
    // Example 5: Using the helper for flag after subcommand
    let result = parse_flag_after_subcommand(&["restore", "--from", "s3://bucket/key"]);
    assert!(result.is_ok());

    let parsed = result.unwrap();
    assert!(parsed.no_interactive);
    assert_eq!(parsed.command, "restore");
}

// ── Verification tests ────────────────────────────────────────────────────────

#[test]
fn example_verify_flag_extraction_before_position() {
    // Example 6: Verify flag was correctly extracted from "before" position
    let parsed = parse_flag_before_subcommand(&["remove", "test", "--confirm"]).unwrap();

    let verification = verify_flag_extraction(&parsed, "before");
    assert!(
        verification.is_ok(),
        "Verification should succeed: {:?}",
        verification
    );
}

#[test]
fn example_verify_flag_extraction_after_position() {
    // Example 7: Verify flag was correctly extracted from "after" position
    let parsed =
        parse_flag_after_subcommand(&["restore", "--from", "s3://b/k", "--confirm"]).unwrap();

    let verification = verify_flag_extraction(&parsed, "after");
    assert!(
        verification.is_ok(),
        "Verification should succeed: {:?}",
        verification
    );
}

#[test]
fn example_verify_no_flag_present() {
    // Example 8: Verify that flag is correctly detected as absent
    let parsed = parse_cli_with_flag(&["hoop", "scan", "/tmp"]).unwrap();

    let verification = verify_no_flag_present(&parsed);
    assert!(
        verification.is_ok(),
        "Should verify no flag is present: {:?}",
        verification
    );
}

#[test]
fn example_verify_no_flag_present_fails_when_flag_exists() {
    // Example 9: Demonstrate verification failure when flag is present
    let parsed = parse_cli_with_flag(&["hoop", "-y", "scan", "/tmp"]).unwrap();

    let verification = verify_no_flag_present(&parsed);
    assert!(
        verification.is_err(),
        "Should fail when flag is actually present"
    );
}

// ── Prompt suppression tests ───────────────────────────────────────────────────

#[test]
fn example_verify_prompt_is_suppressed_with_no_interactive() {
    // Example 10: Verify that a yes/no prompt is suppressed when no_interactive=true
    let prompt = MockYesNoPrompt {
        text: "Register discovered workspace?".to_string(),
        requires_confirm: false, // Safe operation
    };

    let verification = verify_prompt_suppressed(&prompt, true);
    assert!(
        verification.is_ok(),
        "Prompt should be suppressed: {:?}",
        verification
    );
}

#[test]
fn example_verify_prompt_is_shown_without_no_interactive() {
    // Example 11: Verify that a prompt is shown when no_interactive=false
    let prompt = MockYesNoPrompt {
        text: "Register discovered workspace?".to_string(),
        requires_confirm: false,
    };

    // When no_interactive=false, prompt should be shown
    assert!(
        prompt.would_prompt(false),
        "Prompt should be shown when no_interactive=false"
    );
}

#[test]
fn example_verify_destructive_requires_confirm() {
    // Example 12: Verify that destructive operations require --confirm flag
    let prompt = MockYesNoPrompt {
        text: "Remove project 'test'?".to_string(),
        requires_confirm: true, // Destructive operation
    };

    // Should fail: no_interactive=true but confirm=false
    let verification = verify_confirm_required(&prompt, true, false);
    assert!(
        verification.is_err(),
        "Should require --confirm when no_interactive=true for destructive operations"
    );

    // Should succeed: no_interactive=true and confirm=true
    let verification = verify_confirm_required(&prompt, true, true);
    assert!(verification.is_ok(), "Should pass with --confirm flag");

    // Should succeed: no_interactive=false (interactive mode)
    let verification = verify_confirm_required(&prompt, false, false);
    assert!(
        verification.is_ok(),
        "Should not require --confirm in interactive mode"
    );
}

// ── Batch testing tests ───────────────────────────────────────────────────────

#[test]
fn example_run_batch_flag_position_tests() {
    // Example 13: Run a batch of flag position tests
    let test_cases = vec![
        FlagPositionTestCase {
            description: "Scan with flag before command".to_string(),
            command: ["hoop", "--no-interactive", "scan", "/tmp"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            expected_result: true,
        },
        FlagPositionTestCase {
            description: "Scan with flag after command".to_string(),
            command: ["hoop", "scan", "/tmp", "--no-interactive"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            expected_result: true,
        },
        FlagPositionTestCase {
            description: "Scan without flag".to_string(),
            command: ["hoop", "scan", "/tmp"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            expected_result: false,
        },
        FlagPositionTestCase {
            description: "Remove with flag and confirm".to_string(),
            command: [
                "hoop",
                "--no-interactive",
                "remove",
                "test-project",
                "--confirm",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            expected_result: true,
        },
        FlagPositionTestCase {
            description: "Restore with short flag".to_string(),
            command: ["hoop", "-y", "restore", "--from", "s3://b/k", "--confirm"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            expected_result: true,
        },
    ];

    let (successes, failures) = run_flag_position_tests(test_cases);

    assert_eq!(successes.len(), 5, "All test cases should succeed");
    assert_eq!(failures.len(), 0, "No test cases should fail");

    println!("✓ All {} flag position tests passed", successes.len());
}

// ── Test fixture tests ───────────────────────────────────────────────────────

#[test]
fn example_create_test_workspace_fixture() {
    // Example 14: Create a temporary test workspace for integration tests
    let tmp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace = create_test_workspace(&tmp_dir, "test-project");

    assert!(workspace.exists(), "Workspace directory should exist");
    assert!(
        workspace.join(".beads").exists(),
        "Workspace should have .beads/ directory"
    );
}

#[test]
fn example_create_test_registry_fixture() {
    // Example 15: Create a temporary projects registry for integration tests
    let tmp_dir = TempDir::new().expect("Failed to create temp dir");
    let registry_path = create_test_registry(&tmp_dir);

    assert!(registry_path.exists(), "Registry file should exist");
    assert!(
        registry_path.parent().unwrap().ends_with(".hoop"),
        "Registry should be in .hoop/ directory"
    );

    // Verify the file content
    let content = std::fs::read_to_string(&registry_path).expect("Failed to read registry file");
    assert!(
        content.contains("projects: []"),
        "Registry should have empty projects list"
    );
}

// ── Integration test examples ───────────────────────────────────────────────────

#[test]
fn example_integration_test_scan_command() {
    // Example 16: Integration test combining fixtures and flag testing
    let tmp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace = create_test_workspace(&tmp_dir, "test-project");
    let _registry = create_test_registry(&tmp_dir);

    // Parse the scan command with no_interactive flag
    let result = parse_cli_with_flag(&[
        "hoop",
        "--no-interactive",
        "scan",
        tmp_dir.path().to_str().unwrap(),
    ]);
    assert!(result.is_ok(), "Should parse scan command successfully");

    let parsed = result.unwrap();
    assert!(parsed.no_interactive);
    assert_eq!(parsed.command, "scan");

    // Verify the workspace exists for the scan to discover
    assert!(workspace.join(".beads").exists());

    println!("✓ Integration test passed: scan command parsed correctly");
}

#[test]
fn example_integration_test_remove_command_with_confirm() {
    // Example 17: Integration test for destructive command with confirm requirement
    let prompt = MockYesNoPrompt {
        text: "Remove project 'test'?".to_string(),
        requires_confirm: true,
    };

    // Parse the remove command
    let parsed = parse_cli_with_flag(&[
        "hoop",
        "--no-interactive",
        "remove",
        "test-project",
        "--confirm",
    ])
    .expect("Should parse remove command successfully");

    assert!(parsed.no_interactive);
    assert!(parsed.args.contains(&"--confirm".to_string()));

    // Verify that the prompt requires confirm in no_interactive mode
    assert!(prompt.requires_confirm_in_no_interactive());

    // Verify that with --confirm flag, the operation would proceed
    let verification = verify_confirm_required(&prompt, true, true);
    assert!(verification.is_ok(), "Should succeed with --confirm flag");

    println!("✓ Integration test passed: remove command with confirm verified");
}

#[test]
fn example_complex_multi_command_test() {
    // Example 18: Complex test covering multiple commands and scenarios
    let test_cases = vec![
        // Safe operations (scan, list) - no confirm required
        (
            "scan - safe operation",
            vec!["hoop", "--no-interactive", "scan", "/tmp"],
            false, // confirm not required
        ),
        (
            "list - read-only operation",
            vec!["hoop", "--no-interactive", "list"],
            false, // confirm not required
        ),
        // Destructive operations - confirm required
        (
            "remove - destructive operation",
            vec!["hoop", "--no-interactive", "remove", "test", "--confirm"],
            true, // confirm required
        ),
        (
            "restore - destructive operation",
            vec!["hoop", "-y", "restore", "--from", "s3://b/k", "--confirm"],
            true, // confirm required
        ),
    ];

    let mut all_passed = true;
    for (description, args, confirm_required) in test_cases {
        let parsed = parse_cli_with_flag(&args);

        match parsed {
            Ok(p) => {
                if p.no_interactive {
                    // If confirm_required, verify --confirm is present
                    if confirm_required {
                        if !p.args.contains(&"--confirm".to_string()) {
                            eprintln!("✗ FAIL: {} missing --confirm flag", description);
                            all_passed = false;
                        } else {
                            println!("✓ PASS: {} has --confirm as required", description);
                        }
                    } else {
                        println!("✓ PASS: {} (no confirm required)", description);
                    }
                } else {
                    eprintln!("✗ FAIL: {} no_interactive not detected", description);
                    all_passed = false;
                }
            }
            Err(e) => {
                eprintln!("✗ FAIL: {} parse error: {}", description, e);
                all_passed = false;
            }
        }
    }

    assert!(all_passed, "All complex multi-command tests should pass");
}

// ── Error handling examples ───────────────────────────────────────────────────

#[test]
fn example_error_handling_empty_args() {
    // Example 19: Demonstrate error handling for empty arguments
    let result = parse_cli_with_flag(&[]);
    assert!(result.is_err(), "Should fail with empty args");

    let err = result.unwrap_err();
    assert!(
        err.contains("No arguments provided"),
        "Should have descriptive error message"
    );
}

#[test]
fn example_error_handling_invalid_position() {
    // Example 20: Demonstrate error handling for invalid expected_position
    let parsed = parse_cli_with_flag(&["hoop", "--no-interactive", "scan", "/tmp"])
        .expect("Should parse successfully");

    let result = verify_flag_extraction(&parsed, "invalid_position");
    assert!(
        result.is_err(),
        "Should fail with invalid expected_position"
    );
}

#[test]
fn example_error_handling_missing_confirm_for_destructive() {
    // Example 21: Demonstrate error when confirm is missing for destructive operation
    let prompt = MockYesNoPrompt {
        text: "Delete all data?".to_string(),
        requires_confirm: true,
    };

    // no_interactive=true but confirm=false - should fail
    let result = verify_confirm_required(&prompt, true, false);
    assert!(result.is_err(), "Should require --confirm flag");

    let err = result.unwrap_err();
    assert!(
        err.contains("--confirm"),
        "Error message should mention --confirm flag"
    );
}

#[test]
fn example_comprehensive_end_to_end_test() {
    // Example 22: Comprehensive end-to-end test combining all utilities
    let tmp_dir = TempDir::new().expect("Failed to create temp dir");
    let _workspace = create_test_workspace(&tmp_dir, "comprehensive-test");
    let _registry = create_test_registry(&tmp_dir);

    // Test 1: Parse with flag before subcommand
    let parsed_before = parse_flag_before_subcommand(&["scan", tmp_dir.path().to_str().unwrap()])
        .expect("Parse with flag before should succeed");
    assert!(parsed_before.no_interactive);

    // Test 2: Parse with flag after subcommand
    let parsed_after = parse_flag_after_subcommand(&["scan", tmp_dir.path().to_str().unwrap()])
        .expect("Parse with flag after should succeed");
    assert!(parsed_after.no_interactive);

    // Test 3: Verify flag extraction
    assert!(verify_flag_extraction(&parsed_before, "before").is_ok());
    assert!(verify_flag_extraction(&parsed_after, "after").is_ok());

    // Test 4: Test prompt suppression
    let safe_prompt = MockYesNoPrompt {
        text: "Continue?".to_string(),
        requires_confirm: false,
    };
    assert!(verify_prompt_suppressed(&safe_prompt, true).is_ok());

    // Test 5: Test destructive operation requirements
    let destructive_prompt = MockYesNoPrompt {
        text: "Delete?".to_string(),
        requires_confirm: true,
    };
    assert!(verify_confirm_required(&destructive_prompt, true, true).is_ok());
    assert!(verify_confirm_required(&destructive_prompt, true, false).is_err());

    println!("✓ Comprehensive end-to-end test passed");
}
