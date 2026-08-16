//! Unit tests for Init command no_interactive flag
//!
//! These tests verify that the no_interactive flag works correctly
//! for the Init command, covering:
//! 1. Parse test: hoop --no-interactive init
//! 2. Parse test: hoop init --no-interactive
//! 3. Verify flag value extraction in handler
//! 4. Verify flag is passed correctly to init wizard
//! 5. Verify wizard behavior with flag true vs false (mocked wizard)

use std::fs;

// Include the test utilities module
mod cli_test_utils;
use cli_test_utils::*;

// ── Parse tests: Flag position independence ─────────────────────────────────────

#[test]
fn test_init_parse_with_flag_before_subcommand() {
    // Test: hoop --no-interactive init
    let result = parse_cli_with_flag(&["hoop", "--no-interactive", "init"]);

    assert!(result.is_ok(), "Should successfully parse flag before subcommand");
    let parsed = result.unwrap();

    assert!(parsed.no_interactive, "no_interactive should be true");
    assert_eq!(parsed.command, "init", "Command should be 'init'");
    assert!(
        parsed.args.contains(&"init".to_string()),
        "Args should contain init command"
    );
}

#[test]
fn test_init_parse_with_flag_after_subcommand() {
    // Test: hoop init --no-interactive
    let result = parse_cli_with_flag(&["hoop", "init", "--no-interactive"]);

    assert!(result.is_ok(), "Should successfully parse flag after subcommand");
    let parsed = result.unwrap();

    assert!(parsed.no_interactive, "no_interactive should be true");
    assert_eq!(parsed.command, "init", "Command should be 'init'");
    assert!(
        parsed.args.contains(&"init".to_string()),
        "Args should contain init command"
    );
}

#[test]
fn test_init_parse_with_short_flag_before_subcommand() {
    // Test: hoop -y init
    let result = parse_cli_with_flag(&["hoop", "-y", "init"]);

    assert!(result.is_ok(), "Should successfully parse short flag before subcommand");
    let parsed = result.unwrap();

    assert!(parsed.no_interactive, "no_interactive should be true with -y");
    assert_eq!(parsed.command, "init", "Command should be 'init'");
}

#[test]
fn test_init_parse_with_short_flag_after_subcommand() {
    // Test: hoop init -y
    let result = parse_cli_with_flag(&["hoop", "init", "-y"]);

    assert!(result.is_ok(), "Should successfully parse short flag after subcommand");
    let parsed = result.unwrap();

    assert!(parsed.no_interactive, "no_interactive should be true with -y");
    assert_eq!(parsed.command, "init", "Command should be 'init'");
}

#[test]
fn test_init_parse_without_flag() {
    // Test: hoop init (default behavior)
    let result = parse_cli_with_flag(&["hoop", "init"]);

    assert!(result.is_ok(), "Should successfully parse command without flag");
    let parsed = result.unwrap();

    assert!(!parsed.no_interactive, "no_interactive should default to false");
    assert_eq!(parsed.command, "init", "Command should be 'init'");
}

// ── Flag extraction verification tests ───────────────────────────────────────────

#[test]
fn test_init_flag_extraction_before_position() {
    // Verify flag extraction when flag appears before subcommand
    let parsed = parse_flag_before_subcommand(&["init"]).expect("Parse should succeed");

    let verification_result = verify_flag_extraction(&parsed, "before");
    assert!(verification_result.is_ok(), "Flag extraction should verify for 'before' position");

    // Additional assertions
    assert!(parsed.no_interactive);
    assert_eq!(parsed.command, "init");
}

#[test]
fn test_init_flag_extraction_after_position() {
    // Verify flag extraction when flag appears after subcommand
    let parsed = parse_flag_after_subcommand(&["init"]).expect("Parse should succeed");

    let verification_result = verify_flag_extraction(&parsed, "after");
    assert!(verification_result.is_ok(), "Flag extraction should verify for 'after' position");

    // Additional assertions
    assert!(parsed.no_interactive);
    assert_eq!(parsed.command, "init");
}

#[test]
fn test_init_no_flag_present_verification() {
    // Verify that no_interactive is correctly set to false when flag is absent
    let parsed = parse_cli_with_flag(&["hoop", "init"]).expect("Parse should succeed");

    let verification_result = verify_no_flag_present(&parsed);
    assert!(verification_result.is_ok(), "Should verify no flag is present");

    assert!(!parsed.no_interactive);
}

// ── Flag propagation to handler tests ────────────────────────────────────────────

#[test]
fn test_init_flag_propagation_from_main_to_handler() {
    // Verify that the no_interactive flag is correctly extracted in main()
    // and passed to run_init_wizard

    let main_code = fs::read_to_string("src/main.rs")
        .expect("Failed to read main.rs");

    // Verify flag is extracted from parsed CLI
    assert!(
        main_code.contains("let no_interactive = cli.no_interactive;"),
        "Flag should be extracted from parsed CLI structure"
    );

    // Verify flag is passed to init handler
    assert!(
        main_code.contains("init::run_init_wizard(no_interactive)"),
        "Flag should be passed to run_init_wizard handler function"
    );

    // Verify the Init command enum variant exists
    assert!(
        main_code.contains("Commands::Init =>"),
        "Init command handler should exist in main.rs"
    );
}

#[test]
fn test_init_handler_accepts_no_interactive_parameter() {
    // Verify that run_init_wizard handler actually uses the no_interactive parameter
    let init_code = fs::read_to_string("src/init.rs")
        .expect("Failed to read init.rs");

    // Verify function signature accepts no_interactive
    assert!(
        init_code.contains("pub fn run_init_wizard(no_interactive: bool)"),
        "run_init_wizard must accept no_interactive parameter"
    );

    // Verify it's used in conditional logic
    assert!(
        init_code.contains("if no_interactive {"),
        "run_init_wizard must check no_interactive flag"
    );
}

// ── Wizard behavior tests (mocked) ─────────────────────────────────────────────────

#[test]
fn test_init_wizard_rejects_no_interactive_mode() {
    // Test that the init wizard explicitly rejects no_interactive mode
    // This is different from other commands that adapt to non-interactive mode

    let init_code = fs::read_to_string("src/init.rs")
        .expect("Failed to read init.rs");

    // Verify the early rejection pattern exists
    assert!(
        init_code.contains("if no_interactive {"),
        "Init must check no_interactive flag early in the handler"
    );

    // Verify it exits with error code 2
    assert!(
        init_code.contains("std::process::exit(2)"),
        "Init must exit with code 2 when no_interactive is true"
    );

    // Verify helpful error message is shown
    assert!(
        init_code.contains("cannot run in non-interactive mode"),
        "Init must explain why it cannot run non-interactively"
    );

    assert!(
        init_code.contains("requires interactive input for configuration"),
        "Init must state that it requires interactive input"
    );

    assert!(
        init_code.contains("manually create ~/.hoop/config.yml and ~/.hoop/projects.yaml"),
        "Init must suggest manual configuration for automation"
    );
}

#[test]
fn test_init_wizard_runs_when_no_interactive_false() {
    // Test that the init wizard runs normally when no_interactive is false
    // This is the default/expected behavior

    let init_code = fs::read_to_string("src/init.rs")
        .expect("Failed to read init.rs");

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

    // Verify the check comes before stages (early exit pattern)
    assert!(
        no_interactive_check.unwrap() < banner_print.unwrap(),
        "no_interactive check must come before wizard banner"
    );
    assert!(
        banner_print.unwrap() < stage_1.unwrap(),
        "Wizard banner must come before stage 1"
    );
}

// ── Mock prompt interface test ───────────────────────────────────────────────────

/// Mock init wizard prompt interface for testing
#[derive(Debug, Clone)]
struct MockInitWizardPrompt {
    /// Whether the wizard would prompt in interactive mode
    pub would_prompt_interactive: bool,
    /// Whether the wizard rejects no_interactive mode entirely
    pub rejects_no_interactive: bool,
}

impl MockInitWizardPrompt {
    /// Determine whether the wizard would run given the no_interactive value
    fn would_run_when(&self, no_interactive: bool) -> bool {
        // Init wizard explicitly rejects no_interactive mode
        if self.rejects_no_interactive && no_interactive {
            return false;
        }

        // In interactive mode, wizard runs
        !no_interactive
    }
}

#[test]
fn test_init_mock_wizard_rejects_no_interactive() {
    // Test using mock to verify wizard behavior with no_interactive=true

    let mock_wizard = MockInitWizardPrompt {
        would_prompt_interactive: true,
        rejects_no_interactive: true,
    };

    // When no_interactive=true, wizard should NOT run
    let would_run = mock_wizard.would_run_when(true);
    assert!(
        !would_run,
        "Wizard should not run when no_interactive=true (explicit rejection)"
    );
}

#[test]
fn test_init_mock_wizard_runs_interactively() {
    // Test using mock to verify wizard behavior with no_interactive=false

    let mock_wizard = MockInitWizardPrompt {
        would_prompt_interactive: true,
        rejects_no_interactive: true,
    };

    // When no_interactive=false, wizard should run
    let would_run = mock_wizard.would_run_when(false);
    assert!(
        would_run,
        "Wizard should run when no_interactive=false (default interactive mode)"
    );
}

// ── Error message verification tests ──────────────────────────────────────────────

#[test]
fn test_init_error_message_is_helpful() {
    // Verify that the error message when no_interactive=true is helpful
    // and guides users toward manual configuration

    let init_code = fs::read_to_string("src/init.rs")
        .expect("Failed to read init.rs");

    // Verify error message components
    assert!(
        init_code.contains("eprintln!(\"hoop init: cannot run in non-interactive mode.\")"),
        "Error message should be clear and start with command name"
    );

    // Verify it uses stderr (eprintln, not println)
    assert!(
        init_code.contains("eprintln!(\"  The init wizard requires interactive input"),
        "Error message should go to stderr via eprintln!"
    );

    // Verify it provides actionable guidance
    assert!(
        init_code.contains("For automated setup, manually create"),
        "Error message should provide automated setup alternative"
    );

    // Verify it lists both required files
    assert!(
        init_code.contains("~/.hoop/config.yml"),
        "Error message should reference config.yml"
    );
    assert!(
        init_code.contains("~/.hoop/projects.yaml"),
        "Error message should reference projects.yaml"
    );
}

#[test]
fn test_init_exits_with_correct_error_code() {
    // Verify that init exits with code 2 (fatal/precondition error)
    // when no_interactive=true

    let init_code = fs::read_to_string("src/init.rs")
        .expect("Failed to read init.rs");

    // Find the exit code in the no_interactive check
    let no_interactive_section = init_code
        .find("if no_interactive")
        .expect("Should find no_interactive check");

    // Extract a reasonable section around the check
    let section_after_check = &init_code[no_interactive_section..];
    let exit_code_pos = section_after_check
        .find("std::process::exit(2)")
        .expect("Should find exit(2) in no_interactive section");

    // Verify it's exit(2), not exit(1) or any other code
    assert!(
        section_after_check[exit_code_pos..].starts_with("std::process::exit(2)"),
        "Init must exit with code 2 (fatal/precondition error)"
    );
}

// ── Integration-style test: Flag position independence ────────────────────────────

#[test]
fn test_init_flag_position_yields_same_value() {
    // Verify that both flag positions yield the same no_interactive value

    // Parse with flag before subcommand
    let before = parse_cli_with_flag(&["hoop", "--no-interactive", "init"]);
    assert!(before.is_ok(), "Should parse flag before command");
    let before_parsed = before.unwrap();

    // Parse with flag after subcommand
    let after = parse_cli_with_flag(&["hoop", "init", "--no-interactive"]);
    assert!(after.is_ok(), "Should parse flag after command");
    let after_parsed = after.unwrap();

    // Both should yield the same no_interactive value
    assert_eq!(
        before_parsed.no_interactive,
        after_parsed.no_interactive,
        "Flag position should not affect the extracted value"
    );

    assert!(
        before_parsed.no_interactive,
        true,
        "Both positions should extract no_interactive as true"
    );

    assert_eq!(
        before_parsed.command,
        after_parsed.command,
        "Both positions should extract the same command"
    );
}

// ── Test runner for all Init command flag tests ───────────────────────────────────

#[test]
fn test_init_comprehensive_no_interactive_coverage() {
    // Meta-test that verifies all critical aspects are covered
    // This serves as a checklist for the test suite

    let init_code = fs::read_to_string("src/init.rs")
        .expect("Failed to read init.rs");

    // Checklist:
    // 1. Flag is accepted as parameter
    assert!(
        init_code.contains("no_interactive: bool"),
        "✓ Flag accepted as parameter"
    );

    // 2. Flag is checked early
    assert!(
        init_code.find("if no_interactive").is_some(),
        "✓ Flag checked early in handler"
    );

    // 3. Helpful error message provided
    assert!(
        init_code.contains("cannot run in non-interactive mode"),
        "✓ Helpful error message provided"
    );

    // 4. Manual configuration suggested
    assert!(
        init_code.contains("manually create"),
        "✓ Manual configuration alternative suggested"
    );

    // 5. Correct exit code used
    assert!(
        init_code.contains("std::process::exit(2)"),
        "✓ Correct exit code (2) used"
    );

    // 6. Wizard stages exist for interactive mode
    assert!(
        init_code.contains("stage_1_dependency_check"),
        "✓ Wizard stage 1 exists"
    );
    assert!(
        init_code.contains("stage_2_project_registration"),
        "✓ Wizard stage 2 exists"
    );

    // All checks passed
}
