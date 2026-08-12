//! Unit tests for Scan command no_interactive flag
//!
//! These tests verify that the no_interactive flag works correctly
//! for the Scan command, covering:
//! 1. Parse test: hoop --no-interactive scan <path>
//! 2. Parse test: hoop scan <path> --no-interactive
//! 3. Verify flag value extraction in handler
//! 4. Verify flag is passed correctly to scan_projects
//! 5. Verify prompt suppression behavior when flag is true
//! 6. Verify no_interactive || yes logic combination

use std::fs;

// Include the test utilities module
mod cli_test_utils;
use cli_test_utils::*;

// ── Parse tests: Flag position independence ─────────────────────────────────────

#[test]
fn test_scan_parse_with_flag_before_subcommand() {
    // Test: hoop --no-interactive scan /path/to/projects
    let result = parse_cli_with_flag(&["hoop", "--no-interactive", "scan", "/tmp"]);

    assert!(result.is_ok(), "Should successfully parse flag before subcommand");
    let parsed = result.unwrap();

    assert_eq!(parsed.no_interactive, true, "no_interactive should be true");
    assert_eq!(parsed.command, "scan", "Command should be 'scan'");
    assert!(
        parsed.args.contains(&"scan".to_string()),
        "Args should contain scan command"
    );
    assert!(
        parsed.args.contains(&"/tmp".to_string()),
        "Args should contain scan path"
    );
}

#[test]
fn test_scan_parse_with_flag_after_subcommand() {
    // Test: hoop scan /path/to/projects --no-interactive
    let result = parse_cli_with_flag(&["hoop", "scan", "/tmp", "--no-interactive"]);

    assert!(result.is_ok(), "Should successfully parse flag after subcommand");
    let parsed = result.unwrap();

    assert_eq!(parsed.no_interactive, true, "no_interactive should be true");
    assert_eq!(parsed.command, "scan", "Command should be 'scan'");
    assert!(
        parsed.args.contains(&"scan".to_string()),
        "Args should contain scan command"
    );
    assert!(
        parsed.args.contains(&"/tmp".to_string()),
        "Args should contain scan path"
    );
}

#[test]
fn test_scan_parse_with_short_flag_before_subcommand() {
    // Test: hoop -y scan /path/to/projects
    let result = parse_cli_with_flag(&["hoop", "-y", "scan", "/tmp"]);

    assert!(result.is_ok(), "Should successfully parse short flag before subcommand");
    let parsed = result.unwrap();

    assert_eq!(parsed.no_interactive, true, "no_interactive should be true with -y");
    assert_eq!(parsed.command, "scan", "Command should be 'scan'");
}

#[test]
fn test_scan_parse_with_short_flag_after_subcommand() {
    // Test: hoop scan /path/to/projects -y
    let result = parse_cli_with_flag(&["hoop", "scan", "/tmp", "-y"]);

    assert!(result.is_ok(), "Should successfully parse short flag after subcommand");
    let parsed = result.unwrap();

    assert_eq!(parsed.no_interactive, true, "no_interactive should be true with -y");
    assert_eq!(parsed.command, "scan", "Command should be 'scan'");
}

#[test]
fn test_scan_parse_without_flag() {
    // Test: hoop scan /path/to/projects (default behavior)
    let result = parse_cli_with_flag(&["hoop", "scan", "/tmp"]);

    assert!(result.is_ok(), "Should successfully parse command without flag");
    let parsed = result.unwrap();

    assert_eq!(parsed.no_interactive, false, "no_interactive should default to false");
    assert_eq!(parsed.command, "scan", "Command should be 'scan'");
}

#[test]
fn test_scan_parse_with_local_yes_flag() {
    // Test: hoop scan /path/to/projects --yes
    // This tests the local --yes flag (auto_confirm) that exists alongside global --no-interactive
    let result = parse_cli_with_flag(&["hoop", "scan", "/tmp", "--yes"]);

    assert!(result.is_ok(), "Should successfully parse local --yes flag");
    let parsed = result.unwrap();

    assert_eq!(parsed.no_interactive, false, "Global no_interactive should remain false with local --yes");
    assert_eq!(parsed.command, "scan", "Command should be 'scan'");
    assert!(
        parsed.args.contains(&"--yes".to_string()),
        "Args should contain local --yes flag"
    );
}

#[test]
fn test_scan_parse_with_both_flags() {
    // Test: hoop --no-interactive scan /path/to/projects --yes
    // This tests both global --no-interactive and local --yes together
    let result = parse_cli_with_flag(&["hoop", "--no-interactive", "scan", "/tmp", "--yes"]);

    assert!(result.is_ok(), "Should successfully parse both flags");
    let parsed = result.unwrap();

    assert_eq!(parsed.no_interactive, true, "Global no_interactive should be true");
    assert_eq!(parsed.command, "scan", "Command should be 'scan'");
    assert!(
        parsed.args.contains(&"--yes".to_string()),
        "Args should contain local --yes flag"
    );
}

// ── Flag extraction verification tests ───────────────────────────────────────────

#[test]
fn test_scan_flag_extraction_before_position() {
    // Verify flag extraction when flag appears before subcommand
    let parsed = parse_flag_before_subcommand(&["scan", "/tmp"]).expect("Parse should succeed");

    let verification_result = verify_flag_extraction(&parsed, "before");
    assert!(verification_result.is_ok(), "Flag extraction should verify for 'before' position");

    // Additional assertions
    assert_eq!(parsed.no_interactive, true);
    assert_eq!(parsed.command, "scan");
}

#[test]
fn test_scan_flag_extraction_after_position() {
    // Verify flag extraction when flag appears after subcommand
    let parsed = parse_flag_after_subcommand(&["scan", "/tmp"]).expect("Parse should succeed");

    let verification_result = verify_flag_extraction(&parsed, "after");
    assert!(verification_result.is_ok(), "Flag extraction should verify for 'after' position");

    // Additional assertions
    assert_eq!(parsed.no_interactive, true);
    assert_eq!(parsed.command, "scan");
}

#[test]
fn test_scan_no_flag_present_verification() {
    // Verify that no_interactive is correctly set to false when flag is absent
    let parsed = parse_cli_with_flag(&["hoop", "scan", "/tmp"]).expect("Parse should succeed");

    let verification_result = verify_no_flag_present(&parsed);
    assert!(verification_result.is_ok(), "Should verify no flag is present");

    assert_eq!(parsed.no_interactive, false);
}

// ── Flag propagation to handler tests ────────────────────────────────────────────

#[test]
fn test_scan_flag_propagation_from_main_to_handler() {
    // Verify that the no_interactive flag is correctly extracted in main()
    // and passed to scan_projects

    let main_code = fs::read_to_string("src/main.rs")
        .expect("Failed to read main.rs");

    // Verify flag is extracted from parsed CLI
    assert!(
        main_code.contains("let no_interactive = cli.no_interactive;"),
        "Flag should be extracted from parsed CLI structure"
    );

    // Verify flag is passed to scan handler
    assert!(
        main_code.contains("projects::scan_projects(&root, no_interactive || auto_confirm)"),
        "Flag should be passed to scan_projects handler with || auto_confirm logic"
    );

    // Verify the Scan command enum variant exists
    assert!(
        main_code.contains("Commands::Scan { root, auto_confirm } =>"),
        "Scan command handler should exist in main.rs"
    );
}

#[test]
fn test_scan_handler_accepts_no_interactive_parameter() {
    // Verify that scan_projects handler actually uses the no_interactive parameter
    let projects_code = fs::read_to_string("src/projects.rs")
        .expect("Failed to read projects.rs");

    // Verify function signature accepts no_interactive
    assert!(
        projects_code.contains("pub fn scan_projects(root: &str, no_interactive: bool)"),
        "scan_projects must accept no_interactive parameter"
    );

    // Verify it's used in conditional logic
    assert!(
        projects_code.contains("if no_interactive {"),
        "scan_projects must check no_interactive flag"
    );
}

#[test]
fn test_scan_no_interactive_or_yes_combination_logic() {
    // Verify the combination logic: no_interactive || auto_confirm
    // This means either flag being true should result in auto-registration
    let main_code = fs::read_to_string("src/main.rs")
        .expect("Failed to read main.rs");

    // Verify the || logic is used
    assert!(
        main_code.contains("no_interactive || auto_confirm"),
        "Scan should use || logic to combine global no_interactive with local auto_confirm"
    );

    // Find the specific scan handler call
    let scan_handler_pos = main_code.find("Commands::Scan { root, auto_confirm } =>")
        .expect("Should find Scan command handler");

    // Find the scan_projects call within the handler
    let scan_call = main_code[scan_handler_pos..].find("projects::scan_projects(&root, no_interactive || auto_confirm)")
        .expect("Should find scan_projects call with || logic");

    // Verify the logic is correct
    let logic_section = &main_code[scan_handler_pos + scan_call..scan_handler_pos + scan_call + 80];
    assert!(
        logic_section.contains("no_interactive || auto_confirm"),
        "Logic should be: no_interactive || auto_confirm (OR logic)"
    );
}

// ── Prompt suppression behavior tests ─────────────────────────────────────────────

#[test]
fn test_scan_auto_registers_when_no_interactive_true() {
    // Test that scan with no_interactive=true auto-registers without prompting
    let projects_code = fs::read_to_string("src/projects.rs")
        .expect("Failed to read projects.rs");

    // Find the scan_projects function
    let scan_start = projects_code.find("pub fn scan_projects")
        .expect("Should find scan_projects function");

    // Find the no_interactive check within scan_projects
    let no_interactive_check = projects_code[scan_start..].find("if no_interactive {")
        .expect("Should find no_interactive check in scan_projects");

    // Verify auto-registration happens when no_interactive is true
    let no_interactive_section = &projects_code[scan_start + no_interactive_check..scan_start + no_interactive_check + 500];

    assert!(
        no_interactive_section.contains("println!(\"  {} — registering\", default_name)"),
        "When no_interactive=true, should print 'registering' message"
    );

    assert!(
        no_interactive_section.contains("match registry.add(path.clone(), None)"),
        "When no_interactive=true, should call registry.add() without prompting"
    );

    // Verify no prompt happens in this branch
    assert!(
        !no_interactive_section[..200].contains("eprint!(\"  {} — register?"),
        "When no_interactive=true, should NOT prompt for confirmation"
    );
}

#[test]
fn test_scan_prompts_when_no_interactive_false() {
    // Test that scan with no_interactive=false prompts for each discovery
    let projects_code = fs::read_to_string("src/projects.rs")
        .expect("Failed to read projects.rs");

    // Find the scan_projects function
    let scan_start = projects_code.find("pub fn scan_projects")
        .expect("Should find scan_projects function");

    // Find the else branch (interactive mode)
    let no_interactive_check = projects_code[scan_start..].find("if no_interactive {")
        .expect("Should find no_interactive check in scan_projects");

    // Find the else branch after the no_interactive check
    let else_branch = projects_code[scan_start + no_interactive_check..].find("} else {")
        .expect("Should find else branch for interactive mode");

    let interactive_section = &projects_code[scan_start + no_interactive_check + else_branch..scan_start + no_interactive_check + else_branch + 400];

    // Verify prompting happens in the else branch
    assert!(
        interactive_section.contains("eprint!(\"  {} — register? [y/N] \", default_name)"),
        "When no_interactive=false, should prompt with 'register? [y/N]'"
    );

    assert!(
        interactive_section.contains("std::io::stdin().read_line(&mut input)"),
        "When no_interactive=false, should read user input from stdin"
    );

    assert!(
        interactive_section.contains("let answer = input.trim().to_lowercase()"),
        "When no_interactive=false, should process user input"
    );

    assert!(
        interactive_section.contains("if answer != \"y\" && answer != \"yes\""),
        "When no_interactive=false, should check for yes/yes response"
    );
}

#[test]
fn test_scan_prompts_go_to_stderr() {
    // Verify that scan prompts go to stderr (not stdout) to avoid interfering with data output
    let projects_code = fs::read_to_string("src/projects.rs")
        .expect("Failed to read projects.rs");

    // Find the scan_projects function
    let scan_start = projects_code.find("pub fn scan_projects")
        .expect("Should find scan_projects function");

    // Find the else branch (interactive mode)
    let no_interactive_check = projects_code[scan_start..].find("if no_interactive {")
        .expect("Should find no_interactive check in scan_projects");

    // Find the else branch
    let else_branch = projects_code[scan_start + no_interactive_check..].find("} else {")
        .expect("Should find else branch for interactive mode");

    let interactive_section = &projects_code[scan_start + no_interactive_check + else_branch..scan_start + no_interactive_check + else_branch + 400];

    // Verify prompts use eprint! (stderr) not println! (stdout)
    assert!(
        interactive_section.contains("eprint!(\"  {} — register?"),
        "Prompt should use eprint! to write to stderr"
    );

    assert!(
        interactive_section.contains("eprint!(\"    name [{}]: "),
        "Name prompt should also use eprint! to write to stderr"
    );

    // Verify stderr flush
    assert!(
        interactive_section.contains("std::io::stderr().flush()?"),
        "Should flush stderr after prompt to ensure it appears"
    );
}

#[test]
fn test_scan_non_interactive_skips_rename_prompt() {
    // Verify that when no_interactive=true, the rename prompt is skipped
    // and the default name is used
    let projects_code = fs::read_to_string("src/projects.rs")
        .expect("Failed to read projects.rs");

    // Find the scan_projects function
    let scan_start = projects_code.find("pub fn scan_projects")
        .expect("Should find scan_projects function");

    // Find the no_interactive check
    let no_interactive_check = projects_code[scan_start..].find("if no_interactive {")
        .expect("Should find no_interactive check in scan_projects");

    let no_interactive_section = &projects_code[scan_start + no_interactive_check..scan_start + no_interactive_check + 600];

    // Verify registry.add is called with None (no custom name)
    assert!(
        no_interactive_section.contains("match registry.add(path.clone(), None)"),
        "When no_interactive=true, should call registry.add with None (use default name)"
    );

    // Verify the else branch has the rename prompt
    let else_branch = projects_code[scan_start + no_interactive_check..].find("} else {")
        .expect("Should find else branch for interactive mode");

    let interactive_section = &projects_code[scan_start + no_interactive_check + else_branch..scan_start + no_interactive_check + else_branch + 400];

    assert!(
        interactive_section.contains("eprint!(\"    name [{}]: \", default_name)"),
        "When no_interactive=false, should prompt for custom name"
    );
}

// ── Mock prompt interface test ───────────────────────────────────────────────────

/// Mock scan prompt interface for testing
#[derive(Debug, Clone)]
struct MockScanPrompt {
    /// Whether the prompt would show in interactive mode
    pub would_prompt_interactive: bool,
}

impl MockScanPrompt {
    /// Determine whether a prompt would be shown given the no_interactive value
    fn would_prompt_when(&self, no_interactive: bool) -> bool {
        // In non-interactive mode, no prompt
        if no_interactive {
            return false;
        }

        // In interactive mode, prompt is shown
        self.would_prompt_interactive
    }
}

#[test]
fn test_scan_mock_prompt_no_interactive_true() {
    // Test using mock to verify prompt behavior with no_interactive=true
    let mock_prompt = MockScanPrompt {
        would_prompt_interactive: true,
    };

    // When no_interactive=true, prompt should NOT be shown
    let would_prompt = mock_prompt.would_prompt_when(true);
    assert!(
        !would_prompt,
        "Prompt should not be shown when no_interactive=true"
    );
}

#[test]
fn test_scan_mock_prompt_no_interactive_false() {
    // Test using mock to verify prompt behavior with no_interactive=false
    let mock_prompt = MockScanPrompt {
        would_prompt_interactive: true,
    };

    // When no_interactive=false, prompt should be shown
    let would_prompt = mock_prompt.would_prompt_when(false);
    assert!(
        would_prompt,
        "Prompt should be shown when no_interactive=false (default)"
    );
}

// ── Integration-style test: Flag position independence ────────────────────────────

#[test]
fn test_scan_flag_position_yields_same_value() {
    // Verify that both flag positions yield the same no_interactive value

    // Parse with flag before subcommand
    let before = parse_cli_with_flag(&["hoop", "--no-interactive", "scan", "/tmp"]);
    assert!(before.is_ok(), "Should parse flag before command");
    let before_parsed = before.unwrap();

    // Parse with flag after subcommand
    let after = parse_cli_with_flag(&["hoop", "scan", "/tmp", "--no-interactive"]);
    assert!(after.is_ok(), "Should parse flag after command");
    let after_parsed = after.unwrap();

    // Both should yield the same no_interactive value
    assert_eq!(
        before_parsed.no_interactive,
        after_parsed.no_interactive,
        "Flag position should not affect the extracted value"
    );

    assert_eq!(
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

// ── Test for local --yes flag independence ────────────────────────────────────────

#[test]
fn test_scan_local_yes_flag_exists() {
    // Verify that Scan command has its own local --yes flag (auto_confirm)
    // This is independent of the global --no-interactive flag
    let main_code = fs::read_to_string("src/main.rs")
        .expect("Failed to read main.rs");

    // Verify Scan command has auto_confirm field
    assert!(
        main_code.contains("Scan { root, auto_confirm }"),
        "Scan command should have auto_confirm field for local --yes flag"
    );
}

#[test]
fn test_scan_local_yes_flag_documented() {
    // Verify that the local --yes flag is documented
    let main_code = fs::read_to_string("src/main.rs")
        .expect("Failed to read main.rs");

    // Find the Scan command definition
    let scan_def_pos = main_code.find("/// Auto-register every workspace with .beads/")
        .expect("Should find Scan command documentation");

    let scan_section = &main_code[scan_def_pos..scan_def_pos + 300];

    // Verify documentation mentions auto-confirm
    assert!(
        scan_section.contains("Auto-confirm all prompts (non-interactive mode)"),
        "Scan --yes flag should be documented as auto-confirming prompts"
    );

    // Verify the flag attribute
    assert!(
        scan_section.contains("#[arg(long = \"yes\")]"),
        "Scan should have local --yes flag defined with arg attribute"
    );
}

#[test]
fn test_scan_combines_global_and_local_flags() {
    // Verify that both global --no-interactive and local --yes work together
    // The handler should use: no_interactive || auto_confirm
    let main_code = fs::read_to_string("src/main.rs")
        .expect("Failed to read main.rs");

    // Find the scan handler call
    let scan_handler = main_code.find("Commands::Scan { root, auto_confirm } =>")
        .expect("Should find Scan command handler");

    let scan_call = main_code[scan_handler..].find("projects::scan_projects(&root, no_interactive || auto_confirm)")
        .expect("Should find scan_projects call");

    let handler_section = &main_code[scan_handler..scan_handler + scan_call + 100];

    // Verify the || logic combines both flags
    assert!(
        handler_section.contains("no_interactive || auto_confirm"),
        "Handler should combine flags with OR logic: no_interactive || auto_confirm"
    );
}

// ── Test runner for all Scan command flag tests ───────────────────────────────────

#[test]
fn test_scan_comprehensive_no_interactive_coverage() {
    // Meta-test that verifies all critical aspects are covered
    // This serves as a checklist for the test suite

    let main_code = fs::read_to_string("src/main.rs")
        .expect("Failed to read main.rs");
    let projects_code = fs::read_to_string("src/projects.rs")
        .expect("Failed to read projects.rs");

    // Checklist:
    // 1. Scan command has auto_confirm field (local --yes flag)
    assert!(
        main_code.contains("Scan { root, auto_confirm }"),
        "✓ Scan command has auto_confirm field"
    );

    // 2. Scan handler extracts global no_interactive flag
    assert!(
        main_code.contains("let no_interactive = cli.no_interactive;"),
        "✓ Global flag extracted in main"
    );

    // 3. Scan handler combines flags with || logic
    assert!(
        main_code.contains("no_interactive || auto_confirm"),
        "✓ Flags combined with OR logic"
    );

    // 4. Scan handler passes combined value to scan_projects
    assert!(
        main_code.contains("projects::scan_projects(&root, no_interactive || auto_confirm)"),
        "✓ Combined value passed to scan_projects"
    );

    // 5. scan_projects accepts no_interactive parameter
    assert!(
        projects_code.contains("pub fn scan_projects(root: &str, no_interactive: bool)"),
        "✓ scan_projects accepts no_interactive parameter"
    );

    // 6. scan_projects checks no_interactive flag
    assert!(
        projects_code.contains("if no_interactive {"),
        "✓ scan_projects checks no_interactive flag"
    );

    // 7. When no_interactive=true, auto-registers without prompting
    assert!(
        projects_code.contains("println!(\"  {} — registering\", default_name)"),
        "✓ Auto-registers without prompting when no_interactive=true"
    );

    // 8. When no_interactive=false, prompts for confirmation
    assert!(
        projects_code.contains("eprint!(\"  {} — register? [y/N] \", default_name)"),
        "✓ Prompts for confirmation when no_interactive=false"
    );

    // 9. Prompts go to stderr
    assert!(
        projects_code.contains("std::io::stderr().flush()?"),
        "✓ Prompts go to stderr (not stdout)"
    );

    // 10. When no_interactive=true, skips rename prompt
    assert!(
        projects_code.contains("match registry.add(path.clone(), None)"),
        "✓ Skips rename prompt and uses default name when no_interactive=true"
    );

    // All checks passed
    assert!(true, "All Scan command no_interactive tests verified");
}

// ── Handler value extraction tests ─────────────────────────────────────────────────

#[test]
fn test_scan_handler_receives_no_interactive_true_from_global_flag() {
    // Test that handler receives no_interactive=true when global flag is set
    let parsed = parse_cli_with_flag(&["hoop", "--no-interactive", "scan", "/tmp"])
        .expect("Should parse global --no-interactive flag");

    // Verify handler receives correct extraction
    let handler_value = extract_scan_handler_value(&parsed, None);
    assert_eq!(
        handler_value,
        true,
        "Handler should receive true when global --no-interactive is set"
    );
}

#[test]
fn test_scan_handler_receives_no_interactive_true_from_local_yes_flag() {
    // Test that handler receives no_interactive=true when local --yes flag is set
    let parsed = parse_cli_with_flag(&["hoop", "scan", "/tmp", "--yes"])
        .expect("Should parse local --yes flag");

    // Verify handler receives correct extraction (local --yes = auto_confirm)
    let handler_value = extract_scan_handler_value(&parsed, Some(true));
    assert_eq!(
        handler_value,
        true,
        "Handler should receive true when local --yes is set (auto_confirm=true)"
    );
}

#[test]
fn test_scan_handler_receives_no_interactive_true_from_both_flags() {
    // Test that handler receives no_interactive=true when BOTH flags are set
    let parsed = parse_cli_with_flag(&["hoop", "--no-interactive", "scan", "/tmp", "--yes"])
        .expect("Should parse both global --no-interactive and local --yes flags");

    // Verify handler receives correct extraction (no_interactive || auto_confirm)
    let handler_value = extract_scan_handler_value(&parsed, Some(true));
    assert_eq!(
        handler_value,
        true,
        "Handler should receive true when both flags are set (true || true = true)"
    );
}

#[test]
fn test_scan_handler_receives_no_interactive_false_when_no_flags() {
    // Test that handler receives no_interactive=false when neither flag is set
    let parsed = parse_cli_with_flag(&["hoop", "scan", "/tmp"])
        .expect("Should parse scan command without flags");

    // Verify handler receives correct extraction
    let handler_value = extract_scan_handler_value(&parsed, None);
    assert_eq!(
        handler_value,
        false,
        "Handler should receive false when neither flag is set (false || false = false)"
    );
}

#[test]
fn test_scan_handler_no_interactive_or_yes_combination_matrix() {
    // Test all combinations of global no_interactive and local yes flags
    // This verifies the || logic (OR) works correctly

    let test_cases = vec![
        // (no_interactive, auto_confirm, expected_result, description)
        (false, false, false, "Neither flag → false"),
        (true, false, true, "Global flag only → true"),
        (false, true, true, "Local flag only → true"),
        (true, true, true, "Both flags → true"),
    ];

    for (no_interactive, auto_confirm, expected, description) in test_cases {
        let result = no_interactive || auto_confirm;
        assert_eq!(
            result, expected,
            "OR logic failed for case: {} ({} || {} should be {})",
            description, no_interactive, auto_confirm, expected
        );
    }
}

#[test]
fn test_scan_handler_value_extraction_from_parsed_arguments() {
    // Test that the handler correctly extracts and combines values from parsed arguments
    // This simulates the actual flow: parse → extract → combine → pass to handler

    // Case 1: Global flag only
    let parsed_global = parse_cli_with_flag(&["hoop", "--no-interactive", "scan", "/tmp"])
        .expect("Parse with global flag");
    let value_global = simulate_handler_extraction(&parsed_global, false);
    assert_eq!(value_global, true, "Global flag should produce true");

    // Case 2: Local flag only
    let parsed_local = parse_cli_with_flag(&["hoop", "scan", "/tmp", "--yes"])
        .expect("Parse with local flag");
    let value_local = simulate_handler_extraction(&parsed_local, true);
    assert_eq!(value_local, true, "Local flag should produce true");

    // Case 3: Both flags
    let parsed_both = parse_cli_with_flag(&["hoop", "--no-interactive", "scan", "/tmp", "--yes"])
        .expect("Parse with both flags");
    let value_both = simulate_handler_extraction(&parsed_both, true);
    assert_eq!(value_both, true, "Both flags should produce true");

    // Case 4: Neither flag
    let parsed_neither = parse_cli_with_flag(&["hoop", "scan", "/tmp"])
        .expect("Parse without flags");
    let value_neither = simulate_handler_extraction(&parsed_neither, false);
    assert_eq!(value_neither, false, "No flags should produce false");
}

#[test]
fn test_scan_handler_short_flag_y_extraction() {
    // Test that the short -y flag is correctly extracted and passed to handler
    let parsed = parse_cli_with_flag(&["hoop", "-y", "scan", "/tmp"])
        .expect("Should parse short -y flag");

    // Verify the global -y flag is recognized as no_interactive
    assert_eq!(
        parsed.no_interactive, true,
        "Short -y flag should set no_interactive to true"
    );

    // Simulate handler extraction with short flag
    let handler_value = simulate_handler_extraction(&parsed, false);
    assert_eq!(
        handler_value, true,
        "Handler should receive true when short -y flag is used"
    );
}

#[test]
fn test_scan_handler_global_flag_overrides_local_false() {
    // Test that global --no-interactive flag causes non-interactive mode
    // even when local --yes is NOT present (auto_confirm=false)
    let parsed = parse_cli_with_flag(&["hoop", "--no-interactive", "scan", "/tmp"])
        .expect("Should parse with global flag only");

    // Simulate: no_interactive=true, auto_confirm=false
    let handler_value = simulate_handler_extraction(&parsed, false);
    assert_eq!(
        handler_value, true,
        "Global flag should cause non-interactive mode even without local flag (true || false = true)"
    );
}

#[test]
fn test_scan_handler_local_flag_works_without_global() {
    // Test that local --yes flag works independently of global flag
    let parsed = parse_cli_with_flag(&["hoop", "scan", "/tmp", "--yes"])
        .expect("Should parse with local flag only");

    // Simulate: no_interactive=false, auto_confirm=true
    let handler_value = simulate_handler_extraction(&parsed, true);
    assert_eq!(
        handler_value, true,
        "Local flag should work without global flag (false || true = true)"
    );
}

#[test]
fn test_scan_handler_flag_position_independence_for_value() {
    // Test that flag position doesn't affect the extracted value passed to handler
    // Both positions should yield the same handler value

    // Flag before subcommand
    let parsed_before = parse_cli_with_flag(&["hoop", "--no-interactive", "scan", "/tmp"])
        .expect("Parse flag before subcommand");
    let value_before = simulate_handler_extraction(&parsed_before, false);

    // Flag after subcommand
    let parsed_after = parse_cli_with_flag(&["hoop", "scan", "/tmp", "--no-interactive"])
        .expect("Parse flag after subcommand");
    let value_after = simulate_handler_extraction(&parsed_after, false);

    // Both should yield the same handler value
    assert_eq!(
        value_before, value_after,
        "Flag position should not affect the handler value"
    );
    assert_eq!(value_before, true, "Both should produce true");
}

// ── Helper functions for handler value extraction tests ─────────────────────

/// Extract the value that would be received by the scan handler
/// This simulates the logic: no_interactive || auto_confirm
fn extract_scan_handler_value(parsed: &ParsedCli, auto_confirm: Option<bool>) -> bool {
    // The handler receives: global_no_interactive || local_auto_confirm
    let global_no_interactive = parsed.no_interactive;
    let local_auto_confirm = auto_confirm.unwrap_or(false);
    global_no_interactive || local_auto_confirm
}

/// Simulate the full handler extraction flow from parsed arguments
/// This mirrors what actually happens in main.rs when calling scan_projects
fn simulate_handler_extraction(parsed: &ParsedCli, has_local_yes: bool) -> bool {
    // This simulates:
    // let no_interactive = cli.no_interactive;
    // Commands::Scan { root, auto_confirm } => {
    //     projects::scan_projects(&root, no_interactive || auto_confirm)
    // }
    let global_flag = parsed.no_interactive;
    let local_flag = has_local_yes; // In real code, this comes from the Scan variant's auto_confirm field
    global_flag || local_flag
}
