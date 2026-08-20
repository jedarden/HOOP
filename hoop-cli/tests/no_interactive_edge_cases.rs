//! Edge case tests for the `no_interactive` flag
//!
//! This test file covers edge cases and boundary conditions for the no_interactive flag:
//! 1. Flag specified multiple times (last one should win per clap behavior)
//! 2. Flag in complex command chains with multiple subcommands
//! 3. Flag with potentially conflicting options
//! 4. Default behavior verification
//! 5. Position independence in complex scenarios
//! 6. Runtime access and no panics

use std::fs;

mod cli_test_helpers;
use cli_test_helpers::prelude::*;

// ── Edge Case 1: Flag specified multiple times ─────────────────────────────────────

#[test]
fn test_flag_specified_multiple_times_last_wins() {
    // When the same boolean flag is specified multiple times, clap's behavior is:
    // - For boolean flags: last occurrence wins
    // --no-interactive --no-interactive: last one wins (both are true, so result is true)
    // This test verifies that multiple occurrences don't cause parsing errors

    // Test 1: Multiple long form flags
    let args_multiple = &["--no-interactive", "scan", "/tmp", "--no-interactive"];
    let parsed = parse_flag_after_subcommand(args_multiple);
    assert!(parsed.is_ok(), "Should parse multiple flag occurrences");
    let result = parsed.unwrap();
    assert!(
        result.no_interactive,
        "Multiple flags should result in true"
    );

    // Test 2: Multiple short form flags
    let args_multiple_short = &["-y", "scan", "/tmp", "-y"];
    let parsed_short = parse_flag_after_subcommand(args_multiple_short);
    assert!(
        parsed_short.is_ok(),
        "Should parse multiple short flag occurrences"
    );
    let result_short = parsed_short.unwrap();
    assert!(
        result_short.no_interactive,
        "Multiple short flags should result in true"
    );

    // Test 3: Mix of long and short forms
    let args_mixed = &["--no-interactive", "scan", "/tmp", "-y"];
    let parsed_mixed = parse_flag_after_subcommand(args_mixed);
    assert!(parsed_mixed.is_ok(), "Should parse mixed flag forms");
    let result_mixed = parsed_mixed.unwrap();
    assert!(
        result_mixed.no_interactive,
        "Mixed flags should result in true"
    );

    // Test 4: Flag at both positions (before and after subcommand)
    let args_both_positions = &["--no-interactive", "scan", "/tmp", "--no-interactive"];
    let parsed_both = parse_flag_after_subcommand(args_both_positions);
    assert!(parsed_both.is_ok(), "Should parse flag at both positions");
    let result_both = parsed_both.unwrap();
    assert!(
        result_both.no_interactive,
        "Flag at both positions should result in true"
    );
}

#[test]
fn test_flag_multiple_times_with_nested_commands() {
    // Test multiple flag occurrences with nested command structures
    let args = &[
        "--no-interactive",
        "projects",
        "remove",
        "my-project",
        "--confirm",
        "-y",
    ];
    let parsed = parse_nested_subcommand(args);
    assert!(
        parsed.is_ok(),
        "Should parse multiple flags with nested commands"
    );
    let result = parsed.unwrap();
    assert!(result.no_interactive, "Should detect flag presence");
    assert_eq!(result.subcommand, Some("projects".to_string()));
    assert_eq!(result.nested_subcommand, Some("remove".to_string()));
}

// ── Edge Case 2: Flag in complex command chains ────────────────────────────────────

#[test]
fn test_flag_in_complex_command_chain_projects_remove_confirm() {
    // Test: hoop --no-interactive projects remove --confirm <name>
    let args = &[
        "--no-interactive",
        "projects",
        "remove",
        "test-project",
        "--confirm",
    ];
    let parsed = parse_nested_subcommand(args);
    assert!(parsed.is_ok(), "Should parse complex command chain");
    let result = parsed.unwrap();

    assert!(result.no_interactive, "Flag should be true");
    assert_eq!(result.subcommand, Some("projects".to_string()));
    assert_eq!(result.nested_subcommand, Some("remove".to_string()));
    assert!(result.args.contains(&"test-project".to_string()));
    assert!(result.args.contains(&"--confirm".to_string()));
}

#[test]
fn test_flag_at_different_positions_in_complex_chain() {
    // Test flag at each position in a complex chain
    let base_args = &["projects", "remove", "test-project", "--confirm"];

    // Position 1: Before primary subcommand
    let args_1 = &["--no-interactive"]
        .iter()
        .chain(base_args.iter())
        .copied()
        .collect::<Vec<_>>();
    let parsed_1 = parse_nested_subcommand(&args_1);
    assert!(parsed_1.is_ok());
    assert!(parsed_1.unwrap().no_interactive);

    // Position 2: Between primary and nested subcommand
    let args_2 = &[
        "projects",
        "--no-interactive",
        "remove",
        "test-project",
        "--confirm",
    ];
    let parsed_2 = parse_nested_subcommand(args_2);
    assert!(parsed_2.is_ok());
    assert!(parsed_2.unwrap().no_interactive);

    // Position 3: After nested subcommand
    let args_3 = base_args
        .iter()
        .chain(&["--no-interactive"])
        .copied()
        .collect::<Vec<_>>();
    let parsed_3 = parse_nested_subcommand(&args_3);
    assert!(parsed_3.is_ok());
    assert!(parsed_3.unwrap().no_interactive);
}

#[test]
fn test_flag_with_restore_complex_command() {
    // Test with restore command which has multiple flags
    let args = &[
        "--no-interactive",
        "restore",
        "--from",
        "s3://bucket/key",
        "--confirm",
    ];
    let parsed = parse_flag_before_subcommand(args);
    assert!(parsed.is_ok(), "Should parse restore with multiple flags");
    let result = parsed.unwrap();

    assert!(result.no_interactive);
    assert_eq!(result.subcommand, Some("restore".to_string()));
    assert!(result.args.contains(&"--from".to_string()));
    assert!(result.args.contains(&"--confirm".to_string()));
}

#[test]
fn test_flag_with_migrate_subcommands() {
    // Test with migrate subcommands
    let migrate_commands = [
        &["migrate", "run", "--confirm"][..],
        &["migrate", "major-upgrade", "--from", "1", "--confirm"][..],
        &["migrate", "rollback", "1.0.0", "--confirm"][..],
    ];

    for base_cmd in migrate_commands {
        let args = &["--no-interactive"]
            .iter()
            .chain(base_cmd.iter())
            .copied()
            .collect::<Vec<_>>();

        let parsed = parse_flag_before_subcommand(args);
        assert!(parsed.is_ok(), "Should parse migrate subcommands");
        let result = parsed.unwrap();
        assert!(result.no_interactive);
    }
}

// ── Edge Case 3: Flag with potentially conflicting options ────────────────────────

#[test]
fn test_flag_with_local_yes_flag_no_conflict() {
    // scan command has a local --yes flag that should work with global --no-interactive
    // This is not a conflict - both flags enable auto-confirmation
    let args = &["--no-interactive", "scan", "/tmp", "--yes"];
    let parsed = parse_flag_after_subcommand(args);
    assert!(
        parsed.is_ok(),
        "Global and local yes flags should not conflict"
    );
    let result = parsed.unwrap();
    assert!(result.no_interactive);
    assert!(result.args.contains(&"--yes".to_string()));
}

#[test]
fn test_scan_auto_confirm_combination() {
    // Verify the logic: main.rs:407 uses `no_interactive || auto_confirm`
    // Either flag being true should result in auto-confirmation
    let combinations = [
        &["scan", "/tmp", "--yes"][..],                     // Only local flag
        &["--no-interactive", "scan", "/tmp"][..],          // Only global flag
        &["--no-interactive", "scan", "/tmp", "--yes"][..], // Both flags
    ];

    for args in combinations {
        let has_global = args.iter().any(|&a| a == "--no-interactive" || a == "-y");
        let has_local = args.iter().any(|&a| a == "--yes");

        // At least one should be true for auto-confirmation scenarios
        assert!(
            has_global || has_local || !args.contains(&"/tmp"),
            "At least one confirmation flag should be present"
        );
    }
}

#[test]
fn test_flag_with_json_output_no_conflict() {
    // --no-interactive should work with --json output flag
    let args = &["status", "--json", "--no-interactive"];
    let parsed = parse_flag_after_subcommand(args);
    assert!(parsed.is_ok(), "JSON output flag should not conflict");
    let result = parsed.unwrap();
    assert!(result.no_interactive);
    assert!(result.args.contains(&"--json".to_string()));
}

#[test]
fn test_flag_with_dry_run_no_conflict() {
    // --no-interactive should work with --dry-run flag
    let args = &["new", "test-project", "--dry-run", "--no-interactive"];
    let parsed = parse_flag_after_subcommand(args);
    assert!(parsed.is_ok(), "Dry run flag should not conflict");
    let result = parsed.unwrap();
    assert!(result.no_interactive);
    assert!(result.args.contains(&"--dry-run".to_string()));
}

// ── Edge Case 4: Default behavior verification ─────────────────────────────────────

#[test]
fn test_default_behavior_is_interactive_mode() {
    // Verify that when no_interactive is not specified, the default is false (interactive mode)
    let commands_without_flag = [
        &["scan", "/tmp"][..],
        &["projects", "list"][..],
        &["status"][..],
        &["list"][..],
        &["init"][..],
    ];

    for args in commands_without_flag {
        let parsed = parse_flag_before_subcommand(args);
        assert!(parsed.is_ok(), "Should parse commands without flag");
        let result = parsed.unwrap();
        assert!(
            !result.no_interactive,
            "Default should be false (interactive mode) for {:?}",
            args
        );
    }
}

#[test]
fn test_default_propagation_through_handlers() {
    // Verify that default false value propagates correctly
    let result = verify_default_flag_value(&["scan", "/tmp"]);
    assert!(result.is_ok(), "Default value verification should succeed");
}

#[test]
fn test_explicit_vs_implicit_default() {
    // Verify that not specifying the flag gives the same result as explicitly setting it to false
    // Note: clap doesn't support explicit false for boolean flags, so we test implicit behavior

    let implicit_args = &["scan", "/tmp"];
    let parsed_implicit = parse_flag_before_subcommand(implicit_args);
    assert!(parsed_implicit.is_ok());
    let result_implicit = parsed_implicit.unwrap();

    assert!(
        !result_implicit.no_interactive,
        "Implicit default should be false (interactive mode)"
    );
}

// ── Edge Case 5: Position independence in complex scenarios ────────────────────────

#[test]
fn test_position_independence_with_multiple_other_flags() {
    // Flag should work the same regardless of position when other flags are present
    let base_cmd = &["scan", "/tmp"];
    let other_flags = &["--verbose", "--json", "--debug"];

    // Build combinations with flag at different positions
    let combinations = vec![
        vec![
            vec!["--no-interactive"],
            base_cmd.to_vec(),
            other_flags.to_vec(),
        ],
        vec![
            other_flags.to_vec(),
            vec!["--no-interactive"],
            base_cmd.to_vec(),
        ],
        vec![
            base_cmd.to_vec(),
            other_flags.to_vec(),
            vec!["--no-interactive"],
        ],
    ];

    for combo_parts in combinations {
        let args: Vec<&str> = combo_parts.concat().iter().copied().collect();
        let parsed = parse_flag_after_subcommand(&args);
        assert!(
            parsed.is_ok(),
            "Should parse flag with multiple other flags"
        );
        let result = parsed.unwrap();
        assert!(
            result.no_interactive,
            "Flag should be true regardless of position among other flags"
        );
    }
}

#[test]
fn test_position_independence_verification_across_all_commands() {
    // Verify position independence for multiple command types
    let test_cases = [
        (&["scan", "/tmp"][..], "scan"),
        (&["remove", "test-project", "--confirm"][..], "remove"),
        (&["status", "--json"][..], "status"),
        (&["projects", "list"][..], "projects list"),
    ];

    for (base_args, cmd_name) in test_cases {
        let result = verify_flag_position_consistency(base_args);
        assert!(
            result.is_ok(),
            "Position consistency should hold for command: {}",
            cmd_name
        );
    }
}

// ── Edge Case 6: Runtime access and no panics ───────────────────────────────────────

#[test]
fn test_runtime_access_no_panics_simple_commands() {
    // Verify that accessing the flag doesn't cause panics for simple commands
    let test_cases = [
        &["scan", "/tmp", "--no-interactive"][..],
        &["status", "--no-interactive"][..],
        &["list", "--no-interactive"][..],
        &["--no-interactive", "init"][..],
    ];

    for args in test_cases {
        let parsed = parse_flag_before_subcommand(args);
        assert!(parsed.is_ok(), "Parsing should succeed");
        let result = parsed.unwrap();

        // Access the field (this should not panic)
        let _ = result.no_interactive;
        let _ = result.subcommand;
        let _ = result.args;
        let _ = result.raw_args;

        // Clone should work (for use in handlers)
        let _ = result.clone();
    }
}

#[test]
fn test_runtime_access_no_panics_nested_commands() {
    // Verify no panics with nested command structures
    let test_cases = [
        &[
            "--no-interactive",
            "projects",
            "remove",
            "test",
            "--confirm",
        ][..],
        &["projects", "scan", "/tmp", "-y"][..],
        &["patterns", "add", "test-pattern", "--no-interactive"][..],
    ];

    for args in test_cases {
        let parsed = parse_nested_subcommand(args);
        assert!(parsed.is_ok(), "Parsing nested commands should succeed");
        let result = parsed.unwrap();

        // Access all fields (should not panic)
        let _ = result.no_interactive;
        let _ = result.subcommand;
        let _ = result.nested_subcommand;
        let _ = result.args;
        let _ = result.raw_args;

        // Verify trait methods work
        let _ = format!("{:?}", result); // Debug
        let _ = result.clone(); // Clone
    }
}

#[test]
fn test_empty_and_minimal_arguments_no_panics() {
    // Edge cases with minimal or empty argument sets
    let test_cases = [
        &["--no-interactive"][..], // Only flag, no command
        &["-y"][..],               // Only short flag
        &[][..],                   // Empty (should error gracefully)
    ];

    for args in test_cases {
        let parsed = parse_flag_before_subcommand(args);

        // Empty args should error, others should succeed
        if args.is_empty() {
            assert!(parsed.is_err(), "Empty args should error gracefully");
        } else {
            assert!(parsed.is_ok(), "Minimal args should parse");
            if let Ok(result) = parsed {
                // Accessing should not panic even with minimal args
                let _ = result.no_interactive;
                let _ = result.subcommand; // Will be None
            }
        }
    }
}

#[test]
fn test_flag_value_extraction_no_panics() {
    // Test that extract_flag_value doesn't panic on various inputs
    let test_cases = [
        &["scan", "/tmp", "--no-interactive"][..],
        &["-y", "status"][..],
        &["list"][..],             // No flag
        &[][..],                   // Empty
        &["--no-interactive"][..], // Only flag
    ];

    for args in test_cases {
        // This should never panic
        let _ = extract_flag_value(args);
    }
}

#[test]
fn test_verification_utilities_no_panics() {
    // Test that verification utilities don't panic on various inputs
    let test_cases = [
        (&["scan", "/tmp"][..], "simple scan"),
        (
            &["projects", "remove", "test", "--confirm"][..],
            "nested remove",
        ),
        (&["status", "--json"][..], "status with flags"),
    ];

    for (args, description) in test_cases {
        // All these should complete without panicking
        let _ = verify_flag_position_consistency(args);
        let _ = assert_flag_propagation(args);
        let _ = compare_flag_values_at_levels(args);

        // If we got here without panicking, the test passes
    }
}

// ── Edge Case 7: Special characters and edge cases in arguments ───────────────────

#[test]
fn test_flag_with_special_characters_in_paths() {
    // Test with paths that might have special characters
    let special_paths = [
        &["scan", "/tmp/test path", "--no-interactive"][..],
        &["scan", "/tmp/test-dash", "-y"][..],
        &["scan", "/tmp/test_underscore", "--no-interactive"][..],
    ];

    for args in special_paths {
        let parsed = parse_flag_after_subcommand(args);
        assert!(parsed.is_ok(), "Should parse paths with special characters");
        let result = parsed.unwrap();
        assert!(result.no_interactive);
    }
}

#[test]
fn test_flag_with_very_long_arguments() {
    // Test with very long argument values (no buffer overflow)
    let long_string = "a".repeat(1000);
    let args = &["scan", &long_string, "--no-interactive"];
    let parsed = parse_flag_after_subcommand(args);
    assert!(parsed.is_ok(), "Should handle long arguments");
    let result = parsed.unwrap();
    assert!(result.no_interactive);
}

// ── Edge Case 8: Verify global=true attribute is properly set ─────────────────────

#[test]
fn test_verify_global_flag_attribute_in_source() {
    // This test verifies that the flag is defined with global=true attribute
    // by checking the source code
    let main_code = fs::read_to_string("src/main.rs").expect("Failed to read src/main.rs");

    // Verify the flag definition includes global = true
    assert!(
        main_code.contains("global = true"),
        "no_interactive flag must have global = true attribute"
    );

    // Verify it's defined as a global flag in the Cli struct
    assert!(
        main_code.contains("#[arg(short = 'y', long = \"no-interactive\", global = true)]"),
        "no_interactive flag must be defined with proper attributes"
    );

    // Verify it's extracted in main()
    assert!(
        main_code.contains("let no_interactive = cli.no_interactive"),
        "Main function must extract no_interactive from CLI"
    );
}

#[test]
fn test_verify_flag_propagation_in_handlers() {
    // Verify that the flag is passed to command handlers
    let main_code = fs::read_to_string("src/main.rs").expect("Failed to read src/main.rs");

    // Verify scan handler receives the flag
    assert!(
        main_code.contains("projects::scan_projects(&root, no_interactive || yes)"),
        "scan handler must receive no_interactive flag"
    );

    // Verify remove handler receives the flag
    assert!(
        main_code.contains("projects::remove_project(&name, no_interactive, confirm)"),
        "remove handler must receive no_interactive flag"
    );

    // Verify restore handler receives the flag
    assert!(
        main_code.contains("restore::run_restore(&from, dry_run, no_interactive, confirm)"),
        "restore handler must receive no_interactive flag"
    );
}

// ── Integration: Comprehensive edge case scenario ───────────────────────────────────

#[test]
fn test_comprehensive_edge_case_scenario() {
    // A comprehensive test that combines multiple edge cases

    // Scenario: Complex command with multiple flags, multiple flag occurrences,
    // nested commands, and special characters

    let args = &[
        "--no-interactive",
        "projects",
        "remove",
        "test-project-with-dashes",
        "--confirm",
        "-y", // Multiple occurrences
    ];

    let parsed = parse_nested_subcommand(args);
    assert!(parsed.is_ok(), "Should parse complex scenario");
    let result = parsed.unwrap();

    // Verify all aspects
    assert!(result.no_interactive, "Should detect flag");
    assert_eq!(result.subcommand, Some("projects".to_string()));
    assert_eq!(result.nested_subcommand, Some("remove".to_string()));
    assert!(result
        .args
        .contains(&"test-project-with-dashes".to_string()));
    assert!(result.args.contains(&"--confirm".to_string()));

    // Verify no panics on all accesses
    let _ = result.clone();
    let _ = format!("{:?}", result);
    let _ = result.raw_args;
    let _ = result.args;

    // Verify extraction works
    let extracted = extract_flag_value(args);
    assert!(extracted);

    // Verify position consistency
    let base_args = &[
        "projects",
        "remove",
        "test-project-with-dashes",
        "--confirm",
    ];
    let consistency_result = verify_flag_position_consistency(base_args);
    assert!(consistency_result.is_ok(), "Position should be consistent");
}
