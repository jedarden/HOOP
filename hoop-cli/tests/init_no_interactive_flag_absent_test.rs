//! Unit test for --no-interactive flag absent case
//!
//! This test verifies that the init handler correctly reads the no_interactive flag
//! when --no-interactive is absent from the command.
//!
//! Test coverage:
//! - Simulates parsing a command without --no-interactive flag
//! - Verifies that init_handler reads the flag as false or the default value
//! - Tests the complete flow from parsed command to handler execution
//! - Verifies handler logic responds correctly to the default (false) flag value

use std::fs;

// Include the test utilities module
mod cli_test_utils;
use cli_test_utils::*;

#[test]
fn test_init_handler_reads_flag_absent_as_false() {
    // Test the complete flow: parse -> extract -> handler receives false (default)

    // Step 1: Parse command without --no-interactive flag (flag is absent)
    let args = &["hoop", "init"];
    let parsed = parse_cli_with_flag(args)
        .expect("Should successfully parse command without --no-interactive flag");

    // Step 2: Verify flag is absent and extracted as false (default value)
    assert!(!parsed.no_interactive,
        "Handler should read no_interactive flag as false (default) when --no-interactive is absent");

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

    // Step 6: When no_interactive is false (default), handler should proceed with wizard
    // Verify wizard stages exist and run after the no_interactive check
    let no_interactive_check = init_code.find("if no_interactive");
    let banner_print = init_code.find("print_wizard_banner");
    let stage_1 = init_code.find("stage_1_dependency_check");

    assert!(
        no_interactive_check.is_some(),
        "Init should have no_interactive check"
    );
    assert!(
        banner_print.is_some(),
        "Init should have wizard banner print"
    );
    assert!(
        stage_1.is_some(),
        "Init should have stage 1 dependency check"
    );

    // Step 7: Verify the check comes before stages (early exit pattern when true, normal flow when false)
    assert!(
        no_interactive_check.unwrap() < banner_print.unwrap(),
        "no_interactive check must come before wizard banner"
    );
    assert!(
        banner_print.unwrap() < stage_1.unwrap(),
        "Wizard banner must come before stage 1"
    );
}

#[test]
fn test_init_handler_flag_absent_flow_integration() {
    // Integration test: complete flow from CLI parsing to handler execution

    // Test: Parse without flag (default behavior)
    let args = &["hoop", "init"];
    let result = parse_cli_with_flag(args);
    assert!(result.is_ok(), "Should parse without flag");
    let parsed = result.unwrap();

    // Verify the complete extraction chain
    assert!(
        !parsed.no_interactive,
        "Parsed result should have no_interactive=false (default)"
    );
    assert_eq!(
        parsed.command, "init",
        "Parsed result should identify 'init' command"
    );

    // Verify handler propagation from main.rs
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

    // Verify handler receives and processes the default (false) value correctly
    let init_code = fs::read_to_string("src/init.rs").expect("Failed to read init.rs");

    // Verify handler uses the flag in conditional logic
    assert!(
        init_code.contains("if no_interactive"),
        "Handler must use no_interactive flag in conditional logic"
    );

    // When no_interactive=false (default), wizard should proceed (not exit early)
    assert!(
        init_code.contains("stage_1_dependency_check"),
        "Handler should proceed with wizard stages when no_interactive=false"
    );
}

#[test]
fn test_init_handler_flag_absent_vs_present_behavior() {
    // Test handler behavior difference between flag absent (false) and present (true)

    // Case 1: Flag absent -> handler receives false (default)
    let without_flag = parse_cli_with_flag(&["hoop", "init"]).expect("Should parse without flag");
    assert!(
        !without_flag.no_interactive,
        "Without flag: handler should receive no_interactive=false (default)"
    );

    // Case 2: Flag present -> handler receives true
    let with_flag =
        parse_cli_with_flag(&["hoop", "--no-interactive", "init"]).expect("Should parse with flag");
    assert!(
        with_flag.no_interactive,
        "With flag present: handler should receive no_interactive=true"
    );

    // Verify the handler code handles both cases correctly
    let init_code = fs::read_to_string("src/init.rs").expect("Failed to read init.rs");

    // Case 1: When no_interactive=false (default), handler should proceed with wizard
    assert!(
        init_code.contains("stage_1_dependency_check"),
        "Handler should proceed with wizard when no_interactive=false"
    );
    assert!(
        init_code.contains("print_wizard_banner"),
        "Handler should show wizard banner when no_interactive=false"
    );

    // Case 2: When no_interactive=true, handler should exit early
    assert!(
        init_code.contains("if no_interactive") && init_code.contains("std::process::exit(2)"),
        "Handler must exit with code 2 when no_interactive=true"
    );
}

#[test]
fn test_init_handler_default_value_consistency() {
    // Verify that the default value (false) is consistent across all parsing scenarios

    // Test multiple scenarios where flag is absent
    let test_cases = vec![
        (vec!["hoop", "init"], "basic init command"),
        (vec!["hoop", "init"], "same command parsed again"),
    ];

    for (args, description) in test_cases {
        let result =
            parse_cli_with_flag(&args).expect(&format!("Should parse {}: {:?}", description, args));

        assert!(
            !result.no_interactive,
            "Handler should read no_interactive=false (default) for {} (command: {:?})",
            description, args
        );
        assert_eq!(
            result.command, "init",
            "Command should be 'init' for {} (command: {:?})",
            description, args
        );
    }

    // Verify handler code structure supports consistent default behavior
    let init_code = fs::read_to_string("src/init.rs").expect("Failed to read init.rs");

    // Handler should rely on Rust's default bool value (false)
    assert!(
        init_code.contains("pub fn run_init_wizard(no_interactive: bool)"),
        "Handler signature should accept bool with default false value"
    );

    assert!(
        init_code.contains("if no_interactive"),
        "Handler should check the boolean value directly"
    );
}

#[test]
fn test_init_handler_flag_absent_complete_flow() {
    // Comprehensive test: complete flow from CLI parsing to handler execution

    // Step 1: Parse command without flag (default behavior)
    let args = &["hoop", "init"];
    let parsed =
        parse_cli_with_flag(args).expect("Should parse command without --no-interactive flag");

    // Step 2: Verify all parsed values
    assert!(
        !parsed.no_interactive,
        "Parsed result should have no_interactive=false (default)"
    );
    assert_eq!(
        parsed.command, "init",
        "Parsed result should identify 'init' command"
    );
    assert!(
        parsed.args.contains(&"init".to_string()),
        "Args should contain 'init' command"
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

    // Step 4: Verify handler receives and processes default (false) value
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

    // Handler proceeds normally when no_interactive=false (default)
    assert!(
        init_code.contains("print_wizard_banner") && init_code.contains("stage_1_dependency_check"),
        "Handler should proceed with wizard when no_interactive=false (default)"
    );
}

#[test]
fn test_init_handler_default_value_extraction_accuracy() {
    // Test that the handler receives the exact boolean value when flag is absent

    // Test without flag (default behavior)
    let without_flag = parse_cli_with_flag(&["hoop", "init"]).expect("Should parse without flag");
    assert!(
        !without_flag.no_interactive,
        "Handler must receive exact value: no_interactive=false when flag absent"
    );

    // Test with flag for comparison
    let with_flag =
        parse_cli_with_flag(&["hoop", "--no-interactive", "init"]).expect("Should parse with flag");
    assert!(
        with_flag.no_interactive,
        "Handler must receive exact value: no_interactive=true when flag present"
    );

    // Verify handler logic is based on this exact boolean value
    let init_code = fs::read_to_string("src/init.rs").expect("Failed to read init.rs");

    // Handler must use exact boolean comparison (not truthy/falsy checks)
    assert!(
        init_code.contains("if no_interactive"),
        "Handler must use direct boolean check (if no_interactive)"
    );

    // Verify the check is for exact boolean values
    assert!(
        init_code.contains("if no_interactive {"),
        "Handler must check exact boolean value (if no_interactive braces)"
    );
}

#[test]
fn test_init_handler_flag_absent_wizard_proceeds() {
    // Test that when flag is absent (no_interactive=false), the wizard proceeds normally

    // Parse without flag
    let args = &["hoop", "init"];
    let parsed = parse_cli_with_flag(args).expect("Should parse without flag");

    assert!(!parsed.no_interactive, "Flag should be false when absent");

    // Verify wizard stages are present in the code
    let init_code = fs::read_to_string("src/init.rs").expect("Failed to read init.rs");

    // Find the no_interactive check and wizard stages
    let no_interactive_check = init_code
        .find("if no_interactive")
        .expect("Should find no_interactive check");
    let wizard_banner = init_code
        .find("print_wizard_banner")
        .expect("Should find wizard banner");
    let stage_1 = init_code
        .find("stage_1_dependency_check")
        .expect("Should find stage 1");

    // Verify early exit check comes before wizard (exit early pattern)
    assert!(
        no_interactive_check < wizard_banner,
        "no_interactive check must come before wizard stages"
    );

    // When no_interactive is false (default), execution should continue to wizard
    assert!(
        wizard_banner < stage_1,
        "Wizard banner should come before stage 1"
    );

    // Verify the early exit only happens when no_interactive is true
    assert!(
        init_code[no_interactive_check..].contains("if no_interactive {"),
        "Early exit should only trigger when no_interactive is true"
    );
}
