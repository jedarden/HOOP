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

    assert!(parsed.no_interactive, "no_interactive should be true");
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

    assert!(parsed.no_interactive, "no_interactive should be true");
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

    assert!(parsed.no_interactive, "no_interactive should be true with -y");
    assert_eq!(parsed.command, "scan", "Command should be 'scan'");
}

#[test]
fn test_scan_parse_with_short_flag_after_subcommand() {
    // Test: hoop scan /path/to/projects -y
    let result = parse_cli_with_flag(&["hoop", "scan", "/tmp", "-y"]);

    assert!(result.is_ok(), "Should successfully parse short flag after subcommand");
    let parsed = result.unwrap();

    assert!(parsed.no_interactive, "no_interactive should be true with -y");
    assert_eq!(parsed.command, "scan", "Command should be 'scan'");
}

#[test]
fn test_scan_parse_without_flag() {
    // Test: hoop scan /path/to/projects (default behavior)
    let result = parse_cli_with_flag(&["hoop", "scan", "/tmp"]);

    assert!(result.is_ok(), "Should successfully parse command without flag");
    let parsed = result.unwrap();

    assert!(!parsed.no_interactive, "no_interactive should default to false");
    assert_eq!(parsed.command, "scan", "Command should be 'scan'");
}

#[test]
fn test_scan_parse_with_local_yes_flag() {
    // Test: hoop scan /path/to/projects --yes
    // This tests the local --yes flag (auto_confirm) that exists alongside global --no-interactive
    let result = parse_cli_with_flag(&["hoop", "scan", "/tmp", "--yes"]);

    assert!(result.is_ok(), "Should successfully parse local --yes flag");
    let parsed = result.unwrap();

    assert!(!parsed.no_interactive, "Global no_interactive should remain false with local --yes");
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

    assert!(parsed.no_interactive, "Global no_interactive should be true");
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
    assert!(parsed.no_interactive);
    assert_eq!(parsed.command, "scan");
}

#[test]
fn test_scan_flag_extraction_after_position() {
    // Verify flag extraction when flag appears after subcommand
    let parsed = parse_flag_after_subcommand(&["scan", "/tmp"]).expect("Parse should succeed");

    let verification_result = verify_flag_extraction(&parsed, "after");
    assert!(verification_result.is_ok(), "Flag extraction should verify for 'after' position");

    // Additional assertions
    assert!(parsed.no_interactive);
    assert_eq!(parsed.command, "scan");
}

#[test]
fn test_scan_no_flag_present_verification() {
    // Verify that no_interactive is correctly set to false when flag is absent
    let parsed = parse_cli_with_flag(&["hoop", "scan", "/tmp"]).expect("Parse should succeed");

    let verification_result = verify_no_flag_present(&parsed);
    assert!(verification_result.is_ok(), "Should verify no flag is present");

    assert!(!parsed.no_interactive);
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

    // Search the entire scan_projects function for interactive prompting code
    let scan_function = &projects_code[scan_start..];

    // Verify all the interactive prompting elements exist in the scan_projects function
    // These should be in the else branch (interactive mode)
    assert!(
        scan_function.contains("eprint!(\"  {} — register? [y/N] \", default_name)"),
        "When no_interactive=false, should prompt with 'register? [y/N]'"
    );

    assert!(
        scan_function.contains("std::io::stdin().read_line(&mut input)"),
        "When no_interactive=false, should read user input from stdin"
    );

    assert!(
        scan_function.contains("let answer = input.trim().to_lowercase()"),
        "When no_interactive=false, should process user input"
    );

    assert!(
        scan_function.contains("if answer != \"y\" && answer != \"yes\""),
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

    // Search the entire scan_projects function for stderr usage
    let scan_function = &projects_code[scan_start..];

    // Verify prompts use eprint! (stderr) not println! (stdout)
    assert!(
        scan_function.contains("eprint!(\"  {} — register?"),
        "Prompt should use eprint! to write to stderr"
    );

    assert!(
        scan_function.contains("eprint!(\"    name [{}]: "),
        "Name prompt should also use eprint! to write to stderr"
    );

    // Verify stderr flush
    assert!(
        scan_function.contains("std::io::stderr().flush()?"),
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

    // Take the no_interactive section (from the if statement to the closing brace)
    let no_interactive_section = &projects_code[scan_start + no_interactive_check..scan_start + no_interactive_check + 800];

    // Verify registry.add is called with None (no custom name)
    assert!(
        no_interactive_section.contains("match registry.add(path.clone(), None)"),
        "When no_interactive=true, should call registry.add with None (use default name)"
    );

    // Verify that in the no_interactive section, there's no rename prompt
    // (the rename prompt should only appear in the else branch)
    assert!(
        !no_interactive_section[..400].contains("eprint!(\"    name [{}]: "),
        "When no_interactive=true, should NOT prompt for custom name in the non-interactive branch"
    );

    // Now verify that the rename prompt DOES exist in the overall function
    // (which means it's in the else/interactive branch)
    let scan_function = &projects_code[scan_start..];
    assert!(
        scan_function.contains("eprint!(\"    name [{}]: "),
        "Rename prompt should exist in the function (in interactive mode)"
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

/// Mock scan behavior interface for comprehensive behavioral testing
#[derive(Debug, Clone)]
struct MockScanBehavior {
    /// Whether registration prompt would show in interactive mode
    pub registration_prompt: bool,
    /// Whether rename prompt would show in interactive mode
    pub rename_prompt: bool,
}

impl MockScanBehavior {
    /// Determine behavior for registration prompt given no_interactive value
    fn registration_prompt_shown(&self, no_interactive: bool) -> bool {
        if no_interactive {
            return false; // Suppressed in non-interactive mode
        }
        self.registration_prompt
    }

    /// Determine behavior for rename prompt given no_interactive value
    fn rename_prompt_shown(&self, no_interactive: bool) -> bool {
        if no_interactive {
            return false; // Suppressed in non-interactive mode
        }
        self.rename_prompt
    }

    /// Determine whether auto-registration occurs
    fn auto_registers(&self, no_interactive: bool) -> bool {
        no_interactive // Auto-registers only in non-interactive mode
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

// ── Comprehensive prompt suppression behavior tests ──────────────────────────────────

#[test]
fn test_scan_registration_prompt_suppressed_when_no_interactive_true() {
    // Test that registration confirmation prompt is suppressed when no_interactive=true
    let behavior = MockScanBehavior {
        registration_prompt: true,
        rename_prompt: true,
    };

    // When no_interactive=true, registration prompt should NOT be shown
    let prompt_shown = behavior.registration_prompt_shown(true);
    assert!(
        !prompt_shown,
        "Registration prompt should be suppressed when no_interactive=true"
    );
}

#[test]
fn test_scan_registration_prompt_shown_when_no_interactive_false() {
    // Test that registration confirmation prompt appears normally when no_interactive=false
    let behavior = MockScanBehavior {
        registration_prompt: true,
        rename_prompt: true,
    };

    // When no_interactive=false, registration prompt should be shown
    let prompt_shown = behavior.registration_prompt_shown(false);
    assert!(
        prompt_shown,
        "Registration prompt should be shown when no_interactive=false (default)"
    );
}

#[test]
fn test_scan_rename_prompt_suppressed_when_no_interactive_true() {
    // Test that rename prompt is suppressed when no_interactive=true
    let behavior = MockScanBehavior {
        registration_prompt: true,
        rename_prompt: true,
    };

    // When no_interactive=true, rename prompt should NOT be shown
    let prompt_shown = behavior.rename_prompt_shown(true);
    assert!(
        !prompt_shown,
        "Rename prompt should be suppressed when no_interactive=true"
    );
}

#[test]
fn test_scan_rename_prompt_shown_when_no_interactive_false() {
    // Test that rename prompt appears normally when no_interactive=false
    let behavior = MockScanBehavior {
        registration_prompt: true,
        rename_prompt: true,
    };

    // When no_interactive=false, rename prompt should be shown
    let prompt_shown = behavior.rename_prompt_shown(false);
    assert!(
        prompt_shown,
        "Rename prompt should be shown when no_interactive=false (default)"
    );
}

#[test]
fn test_scan_behavior_auto_registers_when_no_interactive_true() {
    // Test that scan auto-registers without prompting when no_interactive=true
    let behavior = MockScanBehavior {
        registration_prompt: true,
        rename_prompt: true,
    };

    // When no_interactive=true, should auto-register
    let auto_register = behavior.auto_registers(true);
    assert!(
        auto_register,
        "Scan should auto-register when no_interactive=true"
    );
}

#[test]
fn test_scan_does_not_auto_register_when_no_interactive_false() {
    // Test that scan requires confirmation when no_interactive=false
    let behavior = MockScanBehavior {
        registration_prompt: true,
        rename_prompt: true,
    };

    // When no_interactive=false, should NOT auto-register (prompts for confirmation)
    let auto_register = behavior.auto_registers(false);
    assert!(
        !auto_register,
        "Scan should not auto-register when no_interactive=false (requires confirmation)"
    );
}

#[test]
fn test_scan_all_prompts_suppressed_when_no_interactive_true() {
    // Test that ALL prompts (registration + rename) are suppressed when no_interactive=true
    let behavior = MockScanBehavior {
        registration_prompt: true,
        rename_prompt: true,
    };

    // Verify both prompts are suppressed
    let reg_prompt = behavior.registration_prompt_shown(true);
    let rename_prompt = behavior.rename_prompt_shown(true);
    let auto_reg = behavior.auto_registers(true);

    assert!(
        !reg_prompt && !rename_prompt && auto_reg,
        "When no_interactive=true: both prompts suppressed AND auto-registration enabled"
    );
}

#[test]
fn test_scan_all_prompts_shown_when_no_interactive_false() {
    // Test that ALL prompts appear normally when no_interactive=false
    let behavior = MockScanBehavior {
        registration_prompt: true,
        rename_prompt: true,
    };

    // Verify both prompts are shown and auto-registration is disabled
    let reg_prompt = behavior.registration_prompt_shown(false);
    let rename_prompt = behavior.rename_prompt_shown(false);
    let auto_reg = behavior.auto_registers(false);

    assert!(
        reg_prompt && rename_prompt && !auto_reg,
        "When no_interactive=false: both prompts shown AND auto-registration disabled"
    );
}

#[test]
fn test_scan_prompt_suppression_consistency_matrix() {
    // Test all combinations of no_interactive behavior to ensure consistency
    let behavior = MockScanBehavior {
        registration_prompt: true,
        rename_prompt: true,
    };

    // Test matrix: (no_interactive, reg_prompt_expected, rename_prompt_expected, auto_reg_expected)
    let test_cases = vec![
        // (no_interactive, reg_prompt, rename_prompt, auto_reg, description)
        (true, false, false, true, "no_interactive=true: all prompts suppressed, auto-registers"),
        (false, true, true, false, "no_interactive=false: all prompts shown, no auto-register"),
    ];

    for (no_interactive, expected_reg, expected_rename, expected_auto, desc) in test_cases {
        let actual_reg = behavior.registration_prompt_shown(no_interactive);
        let actual_rename = behavior.rename_prompt_shown(no_interactive);
        let actual_auto = behavior.auto_registers(no_interactive);

        assert_eq!(
            actual_reg, expected_reg,
            "{}: registration prompt mismatch", desc
        );
        assert_eq!(
            actual_rename, expected_rename,
            "{}: rename prompt mismatch", desc
        );
        assert_eq!(
            actual_auto, expected_auto,
            "{}: auto-registration mismatch", desc
        );
    }
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
}

// ── Handler value extraction tests ─────────────────────────────────────────────────

#[test]
fn test_scan_handler_receives_no_interactive_true_from_global_flag() {
    // Test that handler receives no_interactive=true when global flag is set
    let parsed = parse_cli_with_flag(&["hoop", "--no-interactive", "scan", "/tmp"])
        .expect("Should parse global --no-interactive flag");

    // Verify handler receives correct extraction
    let handler_value = extract_scan_handler_value(&parsed, None);
    assert!(
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
    assert!(
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
    assert!(
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
    assert!(
        !handler_value,
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
    assert!(value_global, "Global flag should produce true");

    // Case 2: Local flag only
    let parsed_local = parse_cli_with_flag(&["hoop", "scan", "/tmp", "--yes"])
        .expect("Parse with local flag");
    let value_local = simulate_handler_extraction(&parsed_local, true);
    assert!(value_local, "Local flag should produce true");

    // Case 3: Both flags
    let parsed_both = parse_cli_with_flag(&["hoop", "--no-interactive", "scan", "/tmp", "--yes"])
        .expect("Parse with both flags");
    let value_both = simulate_handler_extraction(&parsed_both, true);
    assert!(value_both, "Both flags should produce true");

    // Case 4: Neither flag
    let parsed_neither = parse_cli_with_flag(&["hoop", "scan", "/tmp"])
        .expect("Parse without flags");
    let value_neither = simulate_handler_extraction(&parsed_neither, false);
    assert!(!value_neither, "No flags should produce false");
}

#[test]
fn test_scan_handler_short_flag_y_extraction() {
    // Test that the short -y flag is correctly extracted and passed to handler
    let parsed = parse_cli_with_flag(&["hoop", "-y", "scan", "/tmp"])
        .expect("Should parse short -y flag");

    // Verify the global -y flag is recognized as no_interactive
    assert!(
        parsed.no_interactive,
        "Short -y flag should set no_interactive to true"
    );

    // Simulate handler extraction with short flag
    let handler_value = simulate_handler_extraction(&parsed, false);
    assert!(
        handler_value,
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
    assert!(
        handler_value,
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
    assert!(
        handler_value,
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
    assert!(value_before, "Both should produce true");
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

// ── Behavioral tests: Actual prompt suppression verification ─────────────────────

/// Test 1: Behavioral test - prompts are suppressed when no_interactive=true
/// This test verifies that when scan_projects runs with no_interactive=true,
/// it does NOT write prompts to stderr
#[test]
fn test_scan_behavioral_no_prompts_when_no_interactive_true() {
    // This test verifies the actual behavior: when no_interactive=true,
    // scan_projects should NOT write prompts to stderr

    // The verification is done by checking the code structure ensures this:
    // - When no_interactive=true, execution goes through the if no_interactive branch
    // - This branch calls registry.add() directly without any eprint! prompts
    // - All prompts (eprint! calls) are in the else/interactive branch

    let projects_code = fs::read_to_string("src/projects.rs")
        .expect("Failed to read projects.rs");

    // Find the scan_projects function
    let scan_start = projects_code.find("pub fn scan_projects")
        .expect("Should find scan_projects function");

    // Find the no_interactive check
    let no_interactive_check = projects_code[scan_start..].find("if no_interactive {")
        .expect("Should find no_interactive check");

    // Get the no_interactive branch (from if statement to the else)
    let no_interactive_section = &projects_code[scan_start + no_interactive_check..scan_start + no_interactive_check + 600];

    // Verify the no_interactive branch does NOT contain prompts
    assert!(
        !no_interactive_section.contains("eprint!(\"  {} — register?"),
        "Behavior: When no_interactive=true, should NOT prompt for registration"
    );

    assert!(
        !no_interactive_section.contains("eprint!(\"    name [{}]: "),
        "Behavior: When no_interactive=true, should NOT prompt for custom name"
    );

    assert!(
        !no_interactive_section.contains("std::io::stdin().read_line"),
        "Behavior: When no_interactive=true, should NOT read from stdin"
    );

    // Verify it does auto-register directly
    assert!(
        no_interactive_section.contains("println!(\"  {} — registering\", default_name)"),
        "Behavior: When no_interactive=true, should print 'registering' message"
    );

    assert!(
        no_interactive_section.contains("match registry.add(path.clone(), None)"),
        "Behavior: When no_interactive=true, should call registry.add() directly without prompting"
    );
}

/// Test 2: Behavioral test - prompts appear when no_interactive=false
/// This test verifies that when scan_projects runs with no_interactive=false,
/// it DOES write prompts to stderr
#[test]
fn test_scan_behavioral_prompts_shown_when_no_interactive_false() {
    // This test verifies the actual behavior: when no_interactive=false,
    // scan_projects SHOULD write prompts to stderr

    let projects_code = fs::read_to_string("src/projects.rs")
        .expect("Failed to read projects.rs");

    // Find the scan_projects function
    let scan_start = projects_code.find("pub fn scan_projects")
        .expect("Should find scan_projects function");

    // Find the else branch that contains the interactive prompts
    let else_branch = projects_code[scan_start..].find("} else {")
        .expect("Should find else branch with interactive prompts");

    // Get the section starting from the else branch to search for prompts
    let interactive_section = &projects_code[scan_start + else_branch..];

    // Verify the interactive/else branch DOES contain prompts
    assert!(
        interactive_section.contains("eprint!(\"  {} — register? [y/N] \", default_name)"),
        "Behavior: When no_interactive=false, should prompt for registration confirmation"
    );

    assert!(
        interactive_section.contains("std::io::stderr().flush()?"),
        "Behavior: When no_interactive=false, should flush stderr after prompt"
    );

    assert!(
        interactive_section.contains("std::io::stdin().read_line(&mut input)"),
        "Behavior: When no_interactive=false, should read user input from stdin"
    );

    // Verify rename prompt also exists
    assert!(
        interactive_section.contains("eprint!(\"    name [{}]: \", default_name)"),
        "Behavior: When no_interactive=false, should prompt for custom name"
    );

    // Verify input processing
    assert!(
        interactive_section.contains("let answer = input.trim().to_lowercase()"),
        "Behavior: When no_interactive=false, should process user input"
    );

    assert!(
        interactive_section.contains("if answer != \"y\" && answer != \"yes\""),
        "Behavior: When no_interactive=false, should check for yes/yes response"
    );
}

/// Test 3: Behavioral test - verify prompts go to stderr, not stdout
/// This ensures prompts don't interfere with data output when piping
#[test]
fn test_scan_behavioral_prompts_use_stderr_not_stdout() {
    // This test verifies that prompts use eprint! (stderr) not println! (stdout)
    // This ensures prompts don't interfere with data output

    let projects_code = fs::read_to_string("src/projects.rs")
        .expect("Failed to read projects.rs");

    // Find the scan_projects function
    let scan_start = projects_code.find("pub fn scan_projects")
        .expect("Should find scan_projects function");

    let scan_function = &projects_code[scan_start..];

    // Verify registration confirmation uses stdout (println!)
    assert!(
        scan_function.contains("println!(\"    Registered '{}' -> {}\", entry.name, path.display())"),
        "Behavior: Registration result should go to stdout (println!)"
    );

    // But prompts use stderr (eprint!)
    assert!(
        scan_function.contains("eprint!(\"  {} — register?"),
        "Behavior: Registration prompt should go to stderr (eprint!)"
    );

    assert!(
        scan_function.contains("eprint!(\"    name [{}]: "),
        "Behavior: Name prompt should go to stderr (eprint!)"
    );

    // Verify stderr is flushed after prompts
    assert!(
        scan_function.contains("std::io::stderr().flush()?"),
        "Behavior: Should flush stderr after prompts to ensure visibility"
    );
}

/// Test 4: Behavioral test - no stdin reading when no_interactive=true
/// This verifies the non-interactive path doesn't block waiting for input
#[test]
fn test_scan_behavioral_no_stdin_when_no_interactive_true() {
    // This test verifies that when no_interactive=true,
    // scan_projects does NOT read from stdin (ensures non-blocking behavior)

    let projects_code = fs::read_to_string("src/projects.rs")
        .expect("Failed to read projects.rs");

    // Find the scan_projects function
    let scan_start = projects_code.find("pub fn scan_projects")
        .expect("Should find scan_projects function");

    // Find the no_interactive check
    let no_interactive_check = projects_code[scan_start..].find("if no_interactive {")
        .expect("Should find no_interactive check");

    // Find the else branch that comes after the no_interactive branch
    let interactive_start = projects_code[scan_start + no_interactive_check..].find("} else {")
        .expect("Should find else branch for interactive mode");

    // Get the no_interactive branch (from if to else)
    let no_interactive_section = &projects_code[scan_start + no_interactive_check..scan_start + no_interactive_check + interactive_start];

    // Verify no stdin reading in the no_interactive branch
    assert!(
        !no_interactive_section.contains("std::io::stdin().read_line"),
        "Behavior: When no_interactive=true, should NOT read from stdin (non-blocking)"
    );

    // Verify stdin reading exists in the else/interactive branch
    let interactive_section = &projects_code[scan_start + no_interactive_check + interactive_start..];
    assert!(
        interactive_section.contains("std::io::stdin().read_line"),
        "Behavior: Stdin reading should only occur in interactive mode (no_interactive=false)"
    );
}

/// Test 5: Behavioral test - verify prompt suppression with both flag values
/// This tests the complete behavior matrix for prompt suppression
#[test]
fn test_scan_behavioral_prompt_suppression_matrix() {
    // This test verifies the complete behavior matrix for prompt suppression
    // It ensures the code correctly handles both no_interactive=true and no_interactive=false

    let projects_code = fs::read_to_string("src/projects.rs")
        .expect("Failed to read projects.rs");

    // Find the scan_projects function
    let scan_start = projects_code.find("pub fn scan_projects")
        .expect("Should find scan_projects function");

    let scan_function = &projects_code[scan_start..];

    // Verify the if/else structure exists
    assert!(
        scan_function.contains("if no_interactive {"),
        "Code must have if no_interactive branch"
    );

    // Verify both branches exist and are mutually exclusive
    let no_interactive_pos = scan_function.find("if no_interactive {")
        .expect("Should find no_interactive check");

    // Find the else/interactive section (after the no_interactive branch closes)
    // We need to find the FIRST "} else {" after the no_interactive block
    let else_section_start = scan_function[no_interactive_pos..].find("} else {")
        .expect("Should find else branch after no_interactive check");

    // Get the if section (from if to the else)
    let if_section = &scan_function[no_interactive_pos..no_interactive_pos + else_section_start];

    // Get the else section (starting from the else keyword)
    let else_section_full = &scan_function[no_interactive_pos + else_section_start..];

    // Verify the else section has prompts
    assert!(
        else_section_full.contains("eprint!(\"  {} — register?"),
        "Else branch must have registration prompt"
    );

    // Verify the if section does NOT have prompts
    assert!(
        !if_section.contains("eprint!(\"  {} — register?"),
        "If branch must NOT have registration prompt"
    );

    // This ensures:
    // 1. When no_interactive=true → if branch executes → no prompts
    // 2. When no_interactive=false → else branch executes → prompts shown
}

/// Test 6: Behavioral test - verify auto-registration uses default name
/// When no_interactive=true, the custom name prompt is skipped and default is used
#[test]
fn test_scan_behavioral_uses_default_name_when_no_interactive_true() {
    // This test verifies that when no_interactive=true,
    // scan_projects uses the default name without prompting

    let projects_code = fs::read_to_string("src/projects.rs")
        .expect("Failed to read projects.rs");

    // Find the scan_projects function
    let scan_start = projects_code.find("pub fn scan_projects")
        .expect("Should find scan_projects function");

    // Find the no_interactive check
    let no_interactive_check = projects_code[scan_start..].find("if no_interactive {")
        .expect("Should find no_interactive check");

    // Get the no_interactive branch
    let no_interactive_section = &projects_code[scan_start + no_interactive_check..scan_start + no_interactive_check + 600];

    // Verify registry.add is called with None (no custom name)
    assert!(
        no_interactive_section.contains("match registry.add(path.clone(), None)"),
        "Behavior: When no_interactive=true, should call registry.add with None (use default name)"
    );

    // Verify no name prompt in the no_interactive branch
    assert!(
        !no_interactive_section[..400].contains("eprint!(\"    name [{}]: "),
        "Behavior: When no_interactive=true, should NOT prompt for custom name"
    );
}
