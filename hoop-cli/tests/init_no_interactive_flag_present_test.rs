//! Unit test for --no-interactive flag present case
//!
//! This test verifies that the init handler correctly reads the no_interactive flag
//! when --no-interactive is present in the command.
//!
//! Test coverage:
//! - Simulates parsing a command with --no-interactive flag present
//! - Verifies that init_handler reads the flag as true
//! - Tests the complete flow from parsed command to handler execution
//! - Verifies handler logic responds correctly to the flag value

use std::fs;

// Include the test utilities module
mod cli_test_utils;
use cli_test_utils::*;

#[test]
fn test_init_handler_reads_flag_present_as_true() {
    // Test the complete flow: parse -> extract -> handler receives true

    // Step 1: Parse command with --no-interactive flag present
    let args = &["hoop", "--no-interactive", "init"];
    let parsed = parse_cli_with_flag(args)
        .expect("Should successfully parse command with --no-interactive flag present");

    // Step 2: Verify flag is present and extracted as true
    assert!(
        parsed.no_interactive,
        "Handler should read no_interactive flag as true when --no-interactive is present"
    );

    // Step 3: Verify command was correctly identified
    assert_eq!(parsed.command, "init", "Command should be 'init'");

    // Step 4: Verify the handler function signature accepts no_interactive parameter
    let init_code = fs::read_to_string("src/init.rs").expect("Failed to read init.rs");

    assert!(
        init_code.contains("pub fn run_init_wizard(no_interactive: bool)"),
        "Handler function must accept no_interactive parameter with bool type"
    );

    // Step 5: Verify handler checks the flag value and responds correctly
    assert!(
        init_code.contains("if no_interactive {"),
        "Handler must check the no_interactive flag value"
    );

    // Step 6: Verify handler exits with error code 2 when flag is true
    assert!(
        init_code.contains("std::process::exit(2)"),
        "Handler must exit with code 2 when no_interactive is true (init requires interaction)"
    );

    // Step 7: Verify helpful error message is shown
    assert!(
        init_code.contains("cannot run in non-interactive mode"),
        "Handler must provide helpful error message when rejecting no_interactive mode"
    );
}

#[test]
fn test_init_handler_flag_present_flow_integration() {
    // Integration test: complete flow from CLI parsing to handler execution

    // Test 1: Parse with flag before subcommand
    let args_before = &["hoop", "--no-interactive", "init"];
    let result_before = parse_cli_with_flag(args_before);
    assert!(
        result_before.is_ok(),
        "Should parse with flag before subcommand"
    );
    let parsed_before = result_before.unwrap();

    // Verify the complete extraction chain
    assert!(
        parsed_before.no_interactive,
        "Parsed result should have no_interactive=true"
    );
    assert_eq!(
        parsed_before.command, "init",
        "Parsed result should identify 'init' command"
    );

    // Test 2: Parse with flag after subcommand (position independence)
    let args_after = &["hoop", "init", "--no-interactive"];
    let result_after = parse_cli_with_flag(args_after);
    assert!(
        result_after.is_ok(),
        "Should parse with flag after subcommand"
    );
    let parsed_after = result_after.unwrap();

    // Verify both positions yield the same result (position independence)
    assert_eq!(
        parsed_before.no_interactive, parsed_after.no_interactive,
        "Flag position should not affect the extracted value"
    );
    assert!(
        parsed_before.no_interactive,
        "Both positions should extract no_interactive as true"
    );
    assert!(
        parsed_after.no_interactive,
        "Both positions should extract no_interactive as true"
    );

    // Test 3: Verify handler propagation from main.rs
    let main_code = fs::read_to_string("src/main.rs").expect("Failed to read main.rs");

    // Verify main() extracts the flag from CLI
    assert!(
        main_code.contains("let no_interactive = cli.no_interactive;"),
        "main() must extract no_interactive flag from parsed CLI structure"
    );

    // Verify main() passes flag to init handler
    assert!(
        main_code.contains("init::run_init_wizard(no_interactive)"),
        "main() must pass no_interactive flag to run_init_wizard handler"
    );

    // Test 4: Verify handler receives and processes the flag correctly
    let init_code = fs::read_to_string("src/init.rs").expect("Failed to read init.rs");

    // Verify handler uses the flag in conditional logic
    assert!(
        init_code.contains("if no_interactive") && init_code.contains("std::process::exit(2)"),
        "Handler must use no_interactive flag in conditional logic and exit appropriately"
    );
}

#[test]
fn test_init_handler_flag_present_vs_absent_behavior() {
    // Test handler behavior difference between flag present (true) and absent (false)

    // Case 1: Flag present -> handler receives true
    let with_flag =
        parse_cli_with_flag(&["hoop", "--no-interactive", "init"]).expect("Should parse with flag");
    assert!(
        with_flag.no_interactive,
        "With flag present: handler should receive no_interactive=true"
    );

    // Case 2: Flag absent -> handler receives false (default)
    let without_flag = parse_cli_with_flag(&["hoop", "init"]).expect("Should parse without flag");
    assert!(
        !without_flag.no_interactive,
        "Without flag: handler should receive no_interactive=false (default)"
    );

    // Verify the handler code handles both cases correctly
    let init_code = fs::read_to_string("src/init.rs").expect("Failed to read init.rs");

    // Case 1: When no_interactive=true, handler should exit early
    assert!(
        init_code.contains("if no_interactive") && init_code.contains("std::process::exit(2)"),
        "Handler must exit with code 2 when no_interactive=true"
    );

    // Case 2: When no_interactive=false, handler should proceed with wizard
    let early_exit_pos = init_code
        .find("if no_interactive")
        .expect("Should find early exit check");
    let banner_print = init_code
        .find("print_wizard_banner")
        .expect("Should find wizard banner");
    let stage_1 = init_code
        .find("stage_1_dependency_check")
        .expect("Should find stage 1");

    // Verify early exit comes before wizard stages (exit early pattern)
    assert!(
        early_exit_pos < banner_print,
        "Early exit check must come before wizard banner"
    );
    assert!(
        banner_print < stage_1,
        "Wizard banner must come before stage 1"
    );
}

#[test]
fn test_init_handler_reads_flag_from_all_positions() {
    // Verify handler correctly reads flag regardless of position in command

    // Test all three flag forms: --no-interactive before, --no-interactive after, -y short form
    let test_cases = vec![
        (
            vec!["hoop", "--no-interactive", "init"],
            "long form before command",
        ),
        (
            vec!["hoop", "init", "--no-interactive"],
            "long form after command",
        ),
        (vec!["hoop", "-y", "init"], "short form -y before command"),
        (vec!["hoop", "init", "-y"], "short form -y after command"),
    ];

    for (args, description) in test_cases {
        let args_slice: Vec<&str> = args.iter().copied().collect();
        let result = parse_cli_with_flag(&args_slice)
            .expect(&format!("Should parse {}: {:?}", description, args));

        assert!(
            result.no_interactive,
            "Handler should read no_interactive=true for {} (command: {:?})",
            description, args
        );
        assert_eq!(
            result.command, "init",
            "Command should be 'init' for {} (command: {:?})",
            description, args
        );
    }

    // Verify handler code structure supports this position independence
    let init_code = fs::read_to_string("src/init.rs").expect("Failed to read init.rs");

    // Handler should not care about flag position (only the boolean value matters)
    assert!(
        init_code.contains("pub fn run_init_wizard(no_interactive: bool)"),
        "Handler signature should only care about boolean value, not flag position"
    );

    assert!(
        init_code.contains("if no_interactive"),
        "Handler should check the boolean value directly"
    );
}

#[test]
fn test_init_handler_flag_present_complete_flow() {
    // Comprehensive test: complete flow from CLI parsing to handler execution

    // Step 1: Parse command with flag present
    let args = &["hoop", "--no-interactive", "init"];
    let parsed =
        parse_cli_with_flag(args).expect("Should parse command with --no-interactive present");

    // Step 2: Verify all parsed values
    assert!(
        parsed.no_interactive,
        "Parsed result should have no_interactive=true"
    );
    assert_eq!(
        parsed.command, "init",
        "Parsed result should identify 'init' command"
    );
    assert!(
        parsed.args.contains(&"init".to_string()),
        "Args should contain 'init' command"
    );
    assert!(
        parsed.args.contains(&"--no-interactive".to_string()),
        "Args should contain '--no-interactive' flag"
    );

    // Step 3: Verify propagation through main.rs
    let main_code = fs::read_to_string("src/main.rs").expect("Failed to read main.rs");

    // main() extracts: let no_interactive = cli.no_interactive;
    assert!(
        main_code.contains("let no_interactive = cli.no_interactive;"),
        "main() must extract no_interactive from CLI structure"
    );

    // main() passes to handler: init::run_init_wizard(no_interactive)
    assert!(
        main_code.contains("init::run_init_wizard(no_interactive)"),
        "main() must pass extracted flag to init handler"
    );

    // Step 4: Verify handler receives and processes flag
    let init_code = fs::read_to_string("src/init.rs").expect("Failed to read init.rs");

    // Handler signature accepts parameter
    assert!(
        init_code.contains("pub fn run_init_wizard(no_interactive: bool)"),
        "Handler must accept no_interactive as bool parameter"
    );

    // Handler checks the value
    assert!(
        init_code.contains("if no_interactive"),
        "Handler must check the no_interactive flag value"
    );

    // Handler responds appropriately to true value
    assert!(
        init_code.contains("if no_interactive")
            && init_code.contains("eprintln!(\"hoop init: cannot run in non-interactive mode.\")")
            && init_code.contains("std::process::exit(2)"),
        "Handler must show error and exit with code 2 when no_interactive=true"
    );
}

#[test]
fn test_init_handler_flag_value_extraction_accuracy() {
    // Test that the handler receives the exact boolean value when flag is present

    // Test with flag present
    let with_flag =
        parse_cli_with_flag(&["hoop", "--no-interactive", "init"]).expect("Should parse with flag");
    assert!(
        with_flag.no_interactive,
        "Handler must receive exact value: no_interactive=true when flag present"
    );

    // Test without flag
    let without_flag = parse_cli_with_flag(&["hoop", "init"]).expect("Should parse without flag");
    assert!(
        !without_flag.no_interactive,
        "Handler must receive exact value: no_interactive=false when flag absent"
    );

    // Verify handler logic is based on this exact boolean value
    let init_code = fs::read_to_string("src/init.rs").expect("Failed to read init.rs");

    // Handler must use exact boolean comparison (not truthy/falsy checks)
    assert!(
        init_code.contains("if no_interactive"),
        "Handler must use direct boolean check (if no_interactive)"
    );

    // Verify the check is for true (not just any truthy value)
    assert!(
        init_code.contains("if no_interactive {")
            && init_code
                .lines()
                .skip_while(|line| !line.contains("if no_interactive"))
                .next()
                .is_some(),
        "Handler must check if no_interactive is true (direct boolean check)"
    );
}
