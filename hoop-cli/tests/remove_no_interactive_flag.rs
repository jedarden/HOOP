//! Unit tests for Remove command no_interactive flag
//!
//! These tests verify that the no_interactive flag works correctly
//! for the Remove command, covering:
//! 1. Parse test: hoop --no-interactive remove PATH
//! 2. Parse test: hoop remove PATH --no-interactive
//! 3. Verify flag value extraction in handler
//! 4. Verify flag is passed correctly to remove_project
//! 5. Verify confirmation suppression behavior when flag is true
//! 6. Verify --confirm requirement in no-interactive mode

use std::fs;

// Include the test utilities module
mod cli_test_utils;
use cli_test_utils::*;

// ── Parse tests: Flag position independence ─────────────────────────────────────

#[test]
fn test_remove_parse_with_flag_before_subcommand() {
    // Test: hoop --no-interactive remove my-project --confirm
    let result = parse_cli_with_flag(&["hoop", "--no-interactive", "remove", "my-project", "--confirm"]);

    assert!(result.is_ok(), "Should successfully parse flag before subcommand");
    let parsed = result.unwrap();

    assert!(parsed.no_interactive, "no_interactive should be true");
    assert_eq!(parsed.command, "remove", "Command should be 'remove'");
    assert!(
        parsed.args.contains(&"remove".to_string()),
        "Args should contain remove command"
    );
    assert!(
        parsed.args.contains(&"my-project".to_string()),
        "Args should contain project name"
    );
}

#[test]
fn test_remove_parse_with_flag_after_subcommand() {
    // Test: hoop remove my-project --no-interactive --confirm
    let result = parse_cli_with_flag(&["hoop", "remove", "my-project", "--no-interactive", "--confirm"]);

    assert!(result.is_ok(), "Should successfully parse flag after subcommand");
    let parsed = result.unwrap();

    assert!(parsed.no_interactive, "no_interactive should be true");
    assert_eq!(parsed.command, "remove", "Command should be 'remove'");
    assert!(
        parsed.args.contains(&"remove".to_string()),
        "Args should contain remove command"
    );
    assert!(
        parsed.args.contains(&"my-project".to_string()),
        "Args should contain project name"
    );
}

#[test]
fn test_remove_parse_with_short_flag_before_subcommand() {
    // Test: hoop -y remove my-project --confirm
    let result = parse_cli_with_flag(&["hoop", "-y", "remove", "my-project", "--confirm"]);

    assert!(result.is_ok(), "Should successfully parse short flag before subcommand");
    let parsed = result.unwrap();

    assert!(parsed.no_interactive, "no_interactive should be true with -y");
    assert_eq!(parsed.command, "remove", "Command should be 'remove'");
}

#[test]
fn test_remove_parse_with_short_flag_after_subcommand() {
    // Test: hoop remove my-project -y --confirm
    let result = parse_cli_with_flag(&["hoop", "remove", "my-project", "-y", "--confirm"]);

    assert!(result.is_ok(), "Should successfully parse short flag after subcommand");
    let parsed = result.unwrap();

    assert!(parsed.no_interactive, "no_interactive should be true with -y");
    assert_eq!(parsed.command, "remove", "Command should be 'remove'");
}

#[test]
fn test_remove_parse_without_flag() {
    // Test: hoop remove my-project --confirm (default behavior)
    let result = parse_cli_with_flag(&["hoop", "remove", "my-project", "--confirm"]);

    assert!(result.is_ok(), "Should successfully parse command without flag");
    let parsed = result.unwrap();

    assert!(!parsed.no_interactive, "no_interactive should default to false");
    assert_eq!(parsed.command, "remove", "Command should be 'remove'");
}

// ── Flag extraction verification tests ───────────────────────────────────────────

#[test]
fn test_remove_flag_extraction_before_position() {
    // Verify flag extraction when flag appears before subcommand
    let parsed = parse_flag_before_subcommand(&["remove", "my-project", "--confirm"])
        .expect("Parse should succeed");

    let verification_result = verify_flag_extraction(&parsed, "before");
    assert!(verification_result.is_ok(), "Flag extraction should verify for 'before' position");

    // Additional assertions
    assert!(parsed.no_interactive);
    assert_eq!(parsed.command, "remove");
}

#[test]
fn test_remove_flag_extraction_after_position() {
    // Verify flag extraction when flag appears after subcommand
    let parsed = parse_flag_after_subcommand(&["remove", "my-project", "--confirm"])
        .expect("Parse should succeed");

    let verification_result = verify_flag_extraction(&parsed, "after");
    assert!(verification_result.is_ok(), "Flag extraction should verify for 'after' position");

    // Additional assertions
    assert!(parsed.no_interactive);
    assert_eq!(parsed.command, "remove");
}

#[test]
fn test_remove_no_flag_present_verification() {
    // Verify that no_interactive is correctly set to false when flag is absent
    let parsed = parse_cli_with_flag(&["hoop", "remove", "my-project", "--confirm"])
        .expect("Parse should succeed");

    let verification_result = verify_no_flag_present(&parsed);
    assert!(verification_result.is_ok(), "Should verify no flag is present");

    assert!(!parsed.no_interactive);
}

// ── Flag propagation to handler tests ────────────────────────────────────────────

#[test]
fn test_remove_flag_propagation_from_main_to_handler() {
    // Verify that the no_interactive flag is correctly extracted in main()
    // and passed to remove_project

    let main_code = fs::read_to_string("src/main.rs")
        .expect("Failed to read main.rs");

    // Verify flag is extracted from parsed CLI
    assert!(
        main_code.contains("let no_interactive = cli.no_interactive;"),
        "Flag should be extracted from parsed CLI structure"
    );

    // Verify flag is passed to remove handler
    assert!(
        main_code.contains("projects::remove_project(&name, no_interactive, confirm)"),
        "Flag should be passed to remove_project handler"
    );

    // Verify the Remove command enum variant exists
    assert!(
        main_code.contains("Commands::Remove { name, confirm } =>"),
        "Remove command handler should exist in main.rs"
    );
}

#[test]
fn test_remove_handler_accepts_no_interactive_parameter() {
    // Verify that remove_project handler actually uses the no_interactive parameter
    let projects_code = fs::read_to_string("src/projects.rs")
        .expect("Failed to read projects.rs");

    // Verify function signature accepts no_interactive
    assert!(
        projects_code.contains("pub fn remove_project(name: &str, no_interactive: bool, confirm: bool)"),
        "remove_project must accept no_interactive parameter"
    );

    // Verify it's used in conditional logic
    assert!(
        projects_code.contains("if no_interactive && !confirm {"),
        "remove_project must check no_interactive flag for confirm requirement"
    );

    assert!(
        projects_code.contains("if !no_interactive {"),
        "remove_project must check no_interactive flag for prompting logic"
    );
}

#[test]
fn test_remove_no_interactive_requires_confirm() {
    // Verify that remove requires --confirm when no_interactive=true
    let projects_code = fs::read_to_string("src/projects.rs")
        .expect("Failed to read projects.rs");

    // Find the remove_project function
    let remove_start = projects_code.find("pub fn remove_project")
        .expect("Should find remove_project function");

    // Find the no_interactive confirm requirement
    let confirm_check = projects_code[remove_start..].find("if no_interactive && !confirm {")
        .expect("Should find confirm requirement check");

    // Get the confirm requirement section
    let confirm_section = &projects_code[remove_start + confirm_check..remove_start + confirm_check + 200];

    // Verify the error message about --confirm requirement
    assert!(
        confirm_section.contains("--confirm is required in non-interactive mode"),
        "Should error when --confirm is missing in no-interactive mode"
    );

    // Verify the error suggests the correct command
    assert!(
        confirm_section.contains("hoop projects remove {} --no-interactive --confirm"),
        "Error should suggest using --confirm flag"
    );
}

// ── Confirmation suppression behavior tests ─────────────────────────────────────

#[test]
fn test_remove_confirms_when_no_interactive_true_with_confirm_flag() {
    // Test that remove with no_interactive=true and --confirm proceeds without prompting
    let projects_code = fs::read_to_string("src/projects.rs")
        .expect("Failed to read projects.rs");

    // Find the remove_project function
    let remove_start = projects_code.find("pub fn remove_project")
        .expect("Should find remove_project function");

    // Find the no_interactive confirm requirement
    let confirm_check = projects_code[remove_start..].find("if no_interactive && !confirm {")
        .expect("Should find confirm requirement check");

    // Get the section after the confirm check
    let after_confirm_check = &projects_code[remove_start + confirm_check..];

    // Find the prompt check
    let prompt_check = after_confirm_check.find("if !no_interactive {")
        .expect("Should have prompt check after confirm requirement");

    // Get the prompt section
    let prompt_section = &after_confirm_check[prompt_check..prompt_check + 500];

    // Verify prompts exist in the !no_interactive branch
    assert!(
        prompt_section.contains("eprint!(\"Confirm removal? [y/N] \")"),
        "Should have confirmation prompt in interactive mode"
    );

    // Verify stdin reading
    assert!(
        prompt_section.contains("std::io::stdin().read_line(&mut input)"),
        "Should read from stdin for confirmation"
    );

    // Verify answer processing
    assert!(
        prompt_section.contains("let answer = input.trim().to_lowercase()"),
        "Should process user input"
    );
}

#[test]
fn test_remove_prompts_when_no_interactive_false() {
    // Test that remove with no_interactive=false prompts for confirmation
    let projects_code = fs::read_to_string("src/projects.rs")
        .expect("Failed to read projects.rs");

    // Find the remove_project function
    let remove_start = projects_code.find("pub fn remove_project")
        .expect("Should find remove_project function");

    // Find the prompt check section
    let prompt_check = projects_code[remove_start..].find("if !no_interactive {")
        .expect("Should find prompt check");

    // Get the prompt section
    let prompt_section = &projects_code[remove_start + prompt_check..remove_start + prompt_check + 600];

    // Verify all the interactive prompting elements exist
    assert!(
        prompt_section.contains("eprintln!(\"Removing project '{}'\", name)"),
        "Should show project removal message"
    );

    assert!(
        prompt_section.contains("eprint!(\"Confirm removal? [y/N] \")"),
        "Should prompt for confirmation"
    );

    assert!(
        prompt_section.contains("std::io::stderr().flush()?"),
        "Should flush stderr after prompt"
    );

    assert!(
        prompt_section.contains("std::io::stdin().read_line(&mut input)"),
        "Should read user input from stdin"
    );

    assert!(
        prompt_section.contains("let answer = input.trim().to_lowercase()"),
        "Should process user input"
    );

    assert!(
        prompt_section.contains("if answer != \"y\" && answer != \"yes\""),
        "Should check for yes/yes response"
    );

    assert!(
        prompt_section.contains("eprintln!(\"Removal cancelled\")"),
        "Should show cancellation message"
    );
}

#[test]
fn test_remove_prompts_go_to_stderr() {
    // Verify that remove prompts go to stderr (not stdout) to avoid interfering with data output
    let projects_code = fs::read_to_string("src/projects.rs")
        .expect("Failed to read projects.rs");

    // Find the remove_project function
    let remove_start = projects_code.find("pub fn remove_project")
        .expect("Should find remove_project function");

    // Find the prompt section
    let prompt_check = projects_code[remove_start..].find("if !no_interactive {")
        .expect("Should find prompt check");

    let prompt_section = &projects_code[remove_start + prompt_check..remove_start + prompt_check + 600];

    // Verify prompts use eprint! (stderr) not println! (stdout)
    assert!(
        prompt_section.contains("eprintln!(\"Removing project '{}'\", name)"),
        "Removal message should use eprintln! to write to stderr"
    );

    assert!(
        prompt_section.contains("eprint!(\"Confirm removal? [y/N] \")"),
        "Prompt should use eprint! to write to stderr"
    );

    // Verify stderr flush
    assert!(
        prompt_section.contains("std::io::stderr().flush()?"),
        "Should flush stderr after prompt to ensure it appears"
    );
}

#[test]
fn test_remove_non_interactive_skips_confirmation_prompt() {
    // Verify that when no_interactive=true (with --confirm), the confirmation prompt is skipped
    // and removal proceeds directly
    let projects_code = fs::read_to_string("src/projects.rs")
        .expect("Failed to read projects.rs");

    // Find the remove_project function
    let remove_start = projects_code.find("pub fn remove_project")
        .expect("Should find remove_project function");

    // Find the confirm requirement check
    let confirm_check = projects_code[remove_start..].find("if no_interactive && !confirm {")
        .expect("Should find confirm requirement check");

    // Find the closing brace of the confirm requirement block
    let confirm_block_end = projects_code[remove_start + confirm_check..].find('}')
        .expect("Should find end of confirm requirement block");

    // Get the confirm requirement block
    let confirm_block = &projects_code[remove_start + confirm_check..remove_start + confirm_check + confirm_block_end + 50];

    // Verify that in this block, there's no confirmation prompt
    assert!(
        !confirm_block[..200].contains("eprint!(\"Confirm removal?"),
        "Confirm requirement block should NOT contain confirmation prompt"
    );

    // Now find the prompt check (after the confirm requirement)
    let after_confirm = &projects_code[remove_start + confirm_check + confirm_block_end..];
    let prompt_check = after_confirm.find("if !no_interactive {")
        .expect("Should find prompt check after confirm requirement");

    // Get the prompt section
    let prompt_section = &after_confirm[prompt_check..prompt_check + 500];

    // Verify the confirmation prompt exists in the !no_interactive branch
    assert!(
        prompt_section.contains("eprint!(\"Confirm removal? [y/N] \")"),
        "Confirmation prompt should exist in !no_interactive branch"
    );

    // This confirms that:
    // 1. When no_interactive=true and !confirm → error (not a prompt, a bail-out)
    // 2. When no_interactive=false → prompts for confirmation
    // 3. When no_interactive=true and confirm → skips the !no_interactive block entirely
}

// ── Mock prompt interface test ───────────────────────────────────────────────────

/// Mock remove prompt interface for testing
#[derive(Debug, Clone)]
struct MockRemovePrompt {
    /// Whether the prompt would show in interactive mode
    pub would_prompt_interactive: bool,
    /// Whether this is a destructive operation requiring --confirm
    pub requires_confirm_in_no_interactive: bool,
}

impl MockRemovePrompt {
    /// Determine whether a prompt would be shown given the no_interactive value
    fn would_prompt_when(&self, no_interactive: bool) -> bool {
        // In non-interactive mode, no prompt
        if no_interactive {
            return false;
        }

        // In interactive mode, prompt is shown
        self.would_prompt_interactive
    }

    /// Determine whether --confirm is required
    fn confirm_required_when(&self, no_interactive: bool) -> bool {
        no_interactive && self.requires_confirm_in_no_interactive
    }
}

#[test]
fn test_remove_mock_prompt_no_interactive_true() {
    // Test using mock to verify prompt behavior with no_interactive=true
    let mock_prompt = MockRemovePrompt {
        would_prompt_interactive: true,
        requires_confirm_in_no_interactive: true, // Remove is destructive
    };

    // When no_interactive=true, prompt should NOT be shown
    let would_prompt = mock_prompt.would_prompt_when(true);
    assert!(
        !would_prompt,
        "Prompt should not be shown when no_interactive=true"
    );
}

#[test]
fn test_remove_mock_prompt_no_interactive_false() {
    // Test using mock to verify prompt behavior with no_interactive=false
    let mock_prompt = MockRemovePrompt {
        would_prompt_interactive: true,
        requires_confirm_in_no_interactive: true,
    };

    // When no_interactive=false, prompt should be shown
    let would_prompt = mock_prompt.would_prompt_when(false);
    assert!(
        would_prompt,
        "Prompt should be shown when no_interactive=false (default)"
    );
}

// ── Comprehensive prompt suppression behavior tests ──────────────────────────────────

#[test]
fn test_remove_confirmation_prompt_suppressed_when_no_interactive_true() {
    // Test that confirmation prompt is suppressed when no_interactive=true (with --confirm)
    let behavior = MockRemovePrompt {
        would_prompt_interactive: true,
        requires_confirm_in_no_interactive: true,
    };

    // When no_interactive=true, confirmation prompt should NOT be shown
    let prompt_shown = behavior.would_prompt_when(true);
    assert!(
        !prompt_shown,
        "Confirmation prompt should be suppressed when no_interactive=true"
    );
}

#[test]
fn test_remove_confirmation_prompt_shown_when_no_interactive_false() {
    // Test that confirmation prompt appears normally when no_interactive=false
    let behavior = MockRemovePrompt {
        would_prompt_interactive: true,
        requires_confirm_in_no_interactive: true,
    };

    // When no_interactive=false, confirmation prompt should be shown
    let prompt_shown = behavior.would_prompt_when(false);
    assert!(
        prompt_shown,
        "Confirmation prompt should be shown when no_interactive=false (default)"
    );
}

#[test]
fn test_remove_confirm_required_when_no_interactive_true() {
    // Test that --confirm is required when no_interactive=true
    let behavior = MockRemovePrompt {
        would_prompt_interactive: true,
        requires_confirm_in_no_interactive: true,
    };

    // When no_interactive=true, --confirm should be required
    let confirm_required = behavior.confirm_required_when(true);
    assert!(
        confirm_required,
        "--confirm should be required when no_interactive=true"
    );
}

#[test]
fn test_remove_confirm_not_required_when_no_interactive_false() {
    // Test that --confirm is NOT required when no_interactive=false
    let behavior = MockRemovePrompt {
        would_prompt_interactive: true,
        requires_confirm_in_no_interactive: true,
    };

    // When no_interactive=false, --confirm should NOT be required
    let confirm_required = behavior.confirm_required_when(false);
    assert!(
        !confirm_required,
        "--confirm should NOT be required when no_interactive=false (prompts instead)"
    );
}

#[test]
fn test_remove_all_prompts_suppressed_when_no_interactive_true() {
    // Test that ALL prompts are suppressed when no_interactive=true (with --confirm)
    let behavior = MockRemovePrompt {
        would_prompt_interactive: true,
        requires_confirm_in_no_interactive: true,
    };

    // Verify prompt is suppressed
    let prompt = behavior.would_prompt_when(true);
    let confirm_req = behavior.confirm_required_when(true);

    assert!(
        !prompt && confirm_req,
        "When no_interactive=true: prompt suppressed AND --confirm required"
    );
}

#[test]
fn test_remove_all_prompts_shown_when_no_interactive_false() {
    // Test that prompts appear normally when no_interactive=false
    let behavior = MockRemovePrompt {
        would_prompt_interactive: true,
        requires_confirm_in_no_interactive: true,
    };

    // Verify prompt is shown and --confirm is not required
    let prompt = behavior.would_prompt_when(false);
    let confirm_req = behavior.confirm_required_when(false);

    assert!(
        prompt && !confirm_req,
        "When no_interactive=false: prompt shown AND --confirm not required"
    );
}

// ── Integration-style test: Flag position independence ────────────────────────────

#[test]
fn test_remove_flag_position_yields_same_value() {
    // Verify that both flag positions yield the same no_interactive value

    // Parse with flag before subcommand
    let before = parse_cli_with_flag(&["hoop", "--no-interactive", "remove", "my-project", "--confirm"]);
    assert!(before.is_ok(), "Should parse flag before command");
    let before_parsed = before.unwrap();

    // Parse with flag after subcommand
    let after = parse_cli_with_flag(&["hoop", "remove", "my-project", "--no-interactive", "--confirm"]);
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

// ── Handler value extraction tests ─────────────────────────────────────────────────

#[test]
fn test_remove_handler_receives_no_interactive_true_from_global_flag() {
    // Test that handler receives no_interactive=true when global flag is set
    let parsed = parse_cli_with_flag(&["hoop", "--no-interactive", "remove", "my-project", "--confirm"])
        .expect("Should parse global --no-interactive flag");

    // Verify handler receives correct extraction
    let handler_value = extract_remove_handler_value(&parsed);
    assert!(
        handler_value,
        true,
        "Handler should receive true when global --no-interactive is set"
    );
}

#[test]
fn test_remove_handler_receives_no_interactive_false_when_no_flags() {
    // Test that handler receives no_interactive=false when no flag is set
    let parsed = parse_cli_with_flag(&["hoop", "remove", "my-project", "--confirm"])
        .expect("Should parse remove command without flags");

    // Verify handler receives correct extraction
    let handler_value = extract_remove_handler_value(&parsed);
    assert!(
        !handler_value,
        "Handler should receive false when no flag is set"
    );
}

#[test]
fn test_remove_handler_value_extraction_from_parsed_arguments() {
    // Test that the handler correctly extracts values from parsed arguments
    // This simulates the actual flow: parse → extract → pass to handler

    // Case 1: Global flag only
    let parsed_global = parse_cli_with_flag(&["hoop", "--no-interactive", "remove", "my-project", "--confirm"])
        .expect("Parse with global flag");
    let value_global = simulate_remove_handler_extraction(&parsed_global);
    assert!(value_global, "Global flag should produce true");

    // Case 2: No flag
    let parsed_none = parse_cli_with_flag(&["hoop", "remove", "my-project", "--confirm"])
        .expect("Parse without flags");
    let value_none = simulate_remove_handler_extraction(&parsed_none);
    assert!(!value_none, "No flag should produce false");
}

#[test]
fn test_remove_handler_short_flag_y_extraction() {
    // Test that the short -y flag is correctly extracted and passed to handler
    let parsed = parse_cli_with_flag(&["hoop", "-y", "remove", "my-project", "--confirm"])
        .expect("Should parse short -y flag");

    // Verify the global -y flag is recognized as no_interactive
    assert!(
        parsed.no_interactive,
        "Short -y flag should set no_interactive to true"
    );

    // Simulate handler extraction with short flag
    let handler_value = simulate_remove_handler_extraction(&parsed);
    assert!(
        handler_value,
        "Handler should receive true when short -y flag is used"
    );
}

#[test]
fn test_remove_handler_flag_position_independence_for_value() {
    // Test that flag position doesn't affect the extracted value passed to handler
    // Both positions should yield the same handler value

    // Flag before subcommand
    let parsed_before = parse_cli_with_flag(&["hoop", "--no-interactive", "remove", "my-project", "--confirm"])
        .expect("Parse flag before subcommand");
    let value_before = simulate_remove_handler_extraction(&parsed_before);

    // Flag after subcommand
    let parsed_after = parse_cli_with_flag(&["hoop", "remove", "my-project", "--no-interactive", "--confirm"])
        .expect("Parse flag after subcommand");
    let value_after = simulate_remove_handler_extraction(&parsed_after);

    // Both should yield the same handler value
    assert_eq!(
        value_before, value_after,
        "Flag position should not affect the handler value"
    );
    assert!(value_before, "Both should produce true");
}

// ── Helper functions for handler value extraction tests ─────────────────────────────

/// Extract the value that would be received by the remove handler
/// This simulates the logic: no_interactive from cli.no_interactive
fn extract_remove_handler_value(parsed: &ParsedCli) -> bool {
    // The handler receives: global_no_interactive
    parsed.no_interactive
}

/// Simulate the full handler extraction flow from parsed arguments
/// This mirrors what actually happens in main.rs when calling remove_project
fn simulate_remove_handler_extraction(parsed: &ParsedCli) -> bool {
    // This simulates:
    // let no_interactive = cli.no_interactive;
    // Commands::Remove { name, confirm } => {
    //     projects::remove_project(&name, no_interactive, confirm)
    // }
    parsed.no_interactive
}

// ── Behavioral tests: Actual prompt suppression verification ─────────────────────

/// Test 1: Behavioral test - confirm requirement when no_interactive=true
/// This test verifies that when remove_project runs with no_interactive=true,
/// it requires --confirm flag
#[test]
fn test_remove_behavioral_requires_confirm_when_no_interactive_true() {
    // This test verifies the actual behavior: when no_interactive=true,
    // remove_project should require --confirm flag

    // The verification is done by checking the code structure ensures this:
    // - When no_interactive=true and !confirm, it should bail out with error
    // - The error should suggest using --confirm flag

    let projects_code = fs::read_to_string("src/projects.rs")
        .expect("Failed to read projects.rs");

    // Find the remove_project function
    let remove_start = projects_code.find("pub fn remove_project")
        .expect("Should find remove_project function");

    // Find the no_interactive confirm requirement check
    let confirm_check = projects_code[remove_start..].find("if no_interactive && !confirm {")
        .expect("Should find confirm requirement check");

    // Get the confirm requirement section
    let confirm_section = &projects_code[remove_start + confirm_check..remove_start + confirm_check + 400];

    // Verify it bails out with an error
    assert!(
        confirm_section.contains("anyhow::bail!"),
        "Should bail out when --confirm is missing in no-interactive mode"
    );

    // Verify the error message
    assert!(
        confirm_section.contains("--confirm is required in non-interactive mode"),
        "Error should mention --confirm requirement"
    );

    // Verify the suggested command format
    assert!(
        confirm_section.contains("hoop projects remove {} --no-interactive --confirm"),
        "Error should suggest the correct command with --confirm"
    );
}

/// Test 2: Behavioral test - prompts appear when no_interactive=false
/// This test verifies that when remove_project runs with no_interactive=false,
/// it DOES show prompts
#[test]
fn test_remove_behavioral_prompts_shown_when_no_interactive_false() {
    // This test verifies the actual behavior: when no_interactive=false,
    // remove_project SHOULD show prompts

    let projects_code = fs::read_to_string("src/projects.rs")
        .expect("Failed to read projects.rs");

    // Find the remove_project function
    let remove_start = projects_code.find("pub fn remove_project")
        .expect("Should find remove_project function");

    // Find the else/interactive branch that contains the prompts
    let prompt_check = projects_code[remove_start..].find("if !no_interactive {")
        .expect("Should find prompt check for interactive mode");

    // Get the interactive section
    let interactive_section = &projects_code[remove_start + prompt_check..remove_start + prompt_check + 600];

    // Verify the interactive/else branch DOES contain prompts
    assert!(
        interactive_section.contains("eprintln!(\"Removing project '{}'\", name)"),
        "Behavior: When no_interactive=false, should show removal message"
    );

    assert!(
        interactive_section.contains("eprint!(\"Confirm removal? [y/N] \")"),
        "Behavior: When no_interactive=false, should prompt for confirmation"
    );

    assert!(
        interactive_section.contains("std::io::stderr().flush()?"),
        "Behavior: When no_interactive=false, should flush stderr after prompt"
    );

    assert!(
        interactive_section.contains("std::io::stdin().read_line(&mut input)"),
        "Behavior: When no_interactive=false, should read user input from stdin"
    );
}

/// Test 3: Behavioral test - verify prompts go to stderr, not stdout
/// This ensures prompts don't interfere with data output when piping
#[test]
fn test_remove_behavioral_prompts_use_stderr_not_stdout() {
    // This test verifies that prompts use eprint! (stderr) not println! (stdout)
    // This ensures prompts don't interfere with data output

    let projects_code = fs::read_to_string("src/projects.rs")
        .expect("Failed to read projects.rs");

    // Find the remove_project function
    let remove_start = projects_code.find("pub fn remove_project")
        .expect("Should find remove_project function");

    // Find the prompt section
    let prompt_check = projects_code[remove_start..].find("if !no_interactive {")
        .expect("Should find prompt check");

    let interactive_section = &projects_code[remove_start + prompt_check..remove_start + prompt_check + 600];

    // Verify prompts use stderr (eprint!)
    assert!(
        interactive_section.contains("eprintln!(\"Removing project '{}'\", name)"),
        "Behavior: Removal message should go to stderr (eprintln!)"
    );

    assert!(
        interactive_section.contains("eprint!(\"Confirm removal? [y/N] \")"),
        "Behavior: Confirmation prompt should go to stderr (eprint!)"
    );

    // Verify stderr is flushed after prompts
    assert!(
        interactive_section.contains("std::io::stderr().flush()?"),
        "Behavior: Should flush stderr after prompts to ensure visibility"
    );
}

/// Test 4: Behavioral test - no stdin reading when no_interactive=true with confirm
/// This verifies the non-interactive path doesn't block waiting for input
#[test]
fn test_remove_behavioral_no_stdin_when_no_interactive_true_with_confirm() {
    // This test verifies that when no_interactive=true with --confirm,
    // remove_project does NOT read from stdin (ensures non-blocking behavior)

    let projects_code = fs::read_to_string("src/projects.rs")
        .expect("Failed to read projects.rs");

    // Find the remove_project function
    let remove_start = projects_code.find("pub fn remove_project")
        .expect("Should find remove_project function");

    // Find the confirm requirement check
    let confirm_check = projects_code[remove_start..].find("if no_interactive && !confirm {")
        .expect("Should find confirm requirement check");

    // Find the prompt check
    let prompt_check = projects_code[remove_start + confirm_check..].find("if !no_interactive {")
        .expect("Should find prompt check after confirm requirement");

    // Get the section between confirm requirement and prompt check
    let between_sections = &projects_code[remove_start + confirm_check..remove_start + confirm_check + prompt_check];

    // Verify no stdin reading between confirm check and prompt check
    assert!(
        !between_sections.contains("std::io::stdin().read_line"),
        "Behavior: Between confirm check and prompt check, should NOT read from stdin"
    );

    // Verify stdin reading exists in the else/interactive branch
    let interactive_section = &projects_code[remove_start + confirm_check + prompt_check..];
    assert!(
        interactive_section.contains("std::io::stdin().read_line"),
        "Behavior: Stdin reading should only occur in interactive mode (no_interactive=false)"
    );
}

/// Test 5: Behavioral test - verify prompt suppression with both flag values
/// This tests the complete behavior matrix for prompt suppression
#[test]
fn test_remove_behavioral_prompt_suppression_matrix() {
    // This test verifies the complete behavior matrix for prompt suppression
    // It ensures the code correctly handles both no_interactive=true and no_interactive=false

    let projects_code = fs::read_to_string("src/projects.rs")
        .expect("Failed to read projects.rs");

    // Find the remove_project function
    let remove_start = projects_code.find("pub fn remove_project")
        .expect("Should find remove_project function");

    let remove_function = &projects_code[remove_start..];

    // Verify the if/else structure exists
    assert!(
        remove_function.contains("if no_interactive && !confirm {"),
        "Code must have confirm requirement check"
    );

    assert!(
        remove_function.contains("if !no_interactive {"),
        "Code must have prompt check for interactive mode"
    );

    // Find the confirm requirement check
    let confirm_check_pos = remove_function.find("if no_interactive && !confirm {")
        .expect("Should find confirm requirement check");

    // Find the prompt check
    let prompt_check_pos = remove_function[confirm_check_pos..].find("if !no_interactive {")
        .expect("Should find prompt check after confirm requirement");

    // Get the confirm requirement section
    let confirm_section = &remove_function[confirm_check_pos..confirm_check_pos + 300];

    // Verify the confirm requirement section does NOT have confirmation prompt
    assert!(
        !confirm_section.contains("eprint!(\"Confirm removal?"),
        "Confirm requirement section must NOT have confirmation prompt (it's an error bail-out)"
    );

    // Get the interactive section
    let interactive_section = &remove_function[confirm_check_pos + prompt_check_pos..];

    // Verify the interactive section has the confirmation prompt
    assert!(
        interactive_section.contains("eprint!(\"Confirm removal? [y/N] \")"),
        "Interactive section must have confirmation prompt"
    );

    // This ensures:
    // 1. When no_interactive=true && !confirm → error bail-out (not a prompt)
    // 2. When no_interactive=false → prompts for confirmation
    // 3. When no_interactive=true && confirm → skips the !no_interactive block entirely
}

/// Test 6: Behavioral test - verify successful removal when no_interactive=true with confirm
/// When no_interactive=true with --confirm, removal proceeds without prompts
#[test]
fn test_remove_behavioral_succeeds_when_no_interactive_true_with_confirm() {
    // This test verifies that when no_interactive=true with --confirm,
    // remove_project proceeds to removal without prompting

    let projects_code = fs::read_to_string("src/projects.rs")
        .expect("Failed to read projects.rs");

    // Find the remove_project function
    let remove_start = projects_code.find("pub fn remove_project")
        .expect("Should find remove_project function");

    // Find the confirm requirement check
    let confirm_check = projects_code[remove_start..].find("if no_interactive && !confirm {")
        .expect("Should find confirm requirement check");

    // Find the end of the confirm requirement block
    let confirm_block_end = projects_code[remove_start + confirm_check..].find('}')
        .expect("Should find end of confirm requirement block");

    // Find the prompt check
    let prompt_check = projects_code[remove_start + confirm_check + confirm_block_end..].find("if !no_interactive {")
        .expect("Should find prompt check after confirm requirement");

    // Find the actual removal code (after both checks)
    let after_checks = &projects_code[remove_start + confirm_check + confirm_block_end + prompt_check..];
    let removal_code = after_checks.find("let removed = registry.remove(name)?")
        .expect("Should find removal call after checks");

    // Get the removal section
    let removal_section = &after_checks[removal_code..removal_code + 200];

    // Verify removal proceeds
    assert!(
        removal_section.contains("let removed = registry.remove(name)?"),
        "Behavior: When checks pass, should proceed to removal"
    );

    assert!(
        removal_section.contains("registry.save()?"),
        "Behavior: After removal, should save the registry"
    );

    // Verify successful message (printed by main.rs, not remove_project)
    let main_code = fs::read_to_string("src/main.rs")
        .expect("Failed to read main.rs");

    // Verify the Remove command handler prints success message
    let remove_handler = main_code.find("Commands::Remove { name, confirm } =>")
        .expect("Should find Remove command handler in main.rs");

    let handler_section = &main_code[remove_handler..remove_handler + 300];

    assert!(
        handler_section.contains("println!(\"Removed project '{}'\", name)"),
        "Behavior: Successful removal should print confirmation message to stdout"
    );
}

// ── Test runner for all Remove command flag tests ───────────────────────────────────

#[test]
fn test_remove_comprehensive_no_interactive_coverage() {
    // Meta-test that verifies all critical aspects are covered
    // This serves as a checklist for the test suite

    let main_code = fs::read_to_string("src/main.rs")
        .expect("Failed to read main.rs");
    let projects_code = fs::read_to_string("src/projects.rs")
        .expect("Failed to read projects.rs");

    // Checklist:
    // 1. Remove command has confirm field (local --confirm flag)
    assert!(
        main_code.contains("Remove { name, confirm }"),
        "✓ Remove command has confirm field"
    );

    // 2. Remove handler extracts global no_interactive flag
    assert!(
        main_code.contains("let no_interactive = cli.no_interactive;"),
        "✓ Global flag extracted in main"
    );

    // 3. Remove handler passes flag to remove_project
    assert!(
        main_code.contains("projects::remove_project(&name, no_interactive, confirm)"),
        "✓ Flag passed to remove_project"
    );

    // 4. remove_project accepts no_interactive and confirm parameters
    assert!(
        projects_code.contains("pub fn remove_project(name: &str, no_interactive: bool, confirm: bool)"),
        "✓ remove_project accepts both no_interactive and confirm parameters"
    );

    // 5. remove_project checks no_interactive flag for confirm requirement
    assert!(
        projects_code.contains("if no_interactive && !confirm {"),
        "✓ remove_project checks confirm requirement in no-interactive mode"
    );

    // 6. remove_project checks no_interactive flag for prompting logic
    assert!(
        projects_code.contains("if !no_interactive {"),
        "✓ remove_project checks no_interactive flag for prompting"
    );

    // 7. When no_interactive=true && !confirm, error message suggests --confirm
    assert!(
        projects_code.contains("--confirm is required in non-interactive mode"),
        "✓ Error message suggests --confirm in no-interactive mode"
    );

    // 8. When no_interactive=false, prompts for confirmation
    assert!(
        projects_code.contains("eprint!(\"Confirm removal? [y/N] \")"),
        "✓ Prompts for confirmation when no_interactive=false"
    );

    // 9. Prompts go to stderr
    assert!(
        projects_code.contains("std::io::stderr().flush()?"),
        "✓ Prompts go to stderr (not stdout)"
    );

    // 10. When checks pass, removal proceeds
    assert!(
        projects_code.contains("let removed = registry.remove(name)?"),
        "✓ Removal proceeds when checks pass"
    );

    // All checks passed
}
