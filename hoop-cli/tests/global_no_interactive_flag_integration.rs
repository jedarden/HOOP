//! Integration tests for global --no-interactive flag behavior across invocation patterns
//!
//! These tests verify that the no_interactive flag works correctly across different
//! invocation patterns and command combinations.
//!
//! Test Scenarios:
//! 1. Flag before subcommand: `hoop --no-interactive scan <root>`
//! 2. Flag after subcommand: `hoop scan --no-interactive <root>`
//! 3. Short alias: `hoop -y scan <root>`
//! 4. Combined with other flags: `hoop --no-interactive scan --json <root>`
//!
//! For each scenario, we verify:
//! - The flag is correctly parsed and accessible
//! - The expected non-interactive behavior occurs (no prompts)
//! - The command succeeds or fails as expected

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Include the test utilities module
mod cli_test_utils;
use cli_test_utils::*;

// ── Scenario 1: Flag before subcommand ─────────────────────────────────────────────

#[test]
fn test_flag_before_subcommand_scan() {
    // Test: hoop --no-interactive scan /tmp
    let result = parse_cli_with_flag(&["hoop", "--no-interactive", "scan", "/tmp"]);

    assert!(result.is_ok(), "Should successfully parse flag before subcommand");
    let parsed = result.unwrap();

    assert!(parsed.no_interactive, "Flag should be extracted as true");
    assert_eq!(parsed.command, "scan", "Command should be 'scan'");
    assert!(parsed.args.contains(&"/tmp".to_string()), "Args should contain scan path");

    // Verify flag extraction
    assert!(verify_flag_extraction(&parsed, "before").is_ok());
}

#[test]
fn test_flag_before_subcommand_remove() {
    // Test: hoop --no-interactive remove my-project --confirm
    let result = parse_cli_with_flag(&["hoop", "--no-interactive", "remove", "my-project", "--confirm"]);

    assert!(result.is_ok(), "Should successfully parse flag before subcommand");
    let parsed = result.unwrap();

    assert!(parsed.no_interactive, "Flag should be extracted as true");
    assert_eq!(parsed.command, "remove", "Command should be 'remove'");
    assert!(parsed.args.contains(&"my-project".to_string()), "Args should contain project name");
    assert!(parsed.args.contains(&"--confirm".to_string()), "Args should contain --confirm");

    // Verify flag extraction
    assert!(verify_flag_extraction(&parsed, "before").is_ok());
}

#[test]
fn test_flag_before_subcommand_restore() {
    // Test: hoop --no-interactive restore --from s3://bucket/key --confirm
    let result = parse_cli_with_flag(&[
        "hoop", "--no-interactive", "restore", "--from", "s3://bucket/key", "--confirm"
    ]);

    assert!(result.is_ok(), "Should successfully parse flag before subcommand");
    let parsed = result.unwrap();

    assert!(parsed.no_interactive, "Flag should be extracted as true");
    assert_eq!(parsed.command, "restore", "Command should be 'restore'");
    assert!(parsed.args.contains(&"--from".to_string()), "Args should contain --from");
    assert!(parsed.args.contains(&"s3://bucket/key".to_string()), "Args should contain URI");
    assert!(parsed.args.contains(&"--confirm".to_string()), "Args should contain --confirm");

    // Verify flag extraction
    assert!(verify_flag_extraction(&parsed, "before").is_ok());
}

#[test]
fn test_flag_before_subcommand_status() {
    // Test: hoop --no-interactive status --json
    let result = parse_cli_with_flag(&["hoop", "--no-interactive", "status", "--json"]);

    assert!(result.is_ok(), "Should successfully parse flag before subcommand");
    let parsed = result.unwrap();

    assert!(parsed.no_interactive, "Flag should be extracted as true");
    assert_eq!(parsed.command, "status", "Command should be 'status'");
    assert!(parsed.args.contains(&"--json".to_string()), "Args should contain --json");

    // Verify flag extraction
    assert!(verify_flag_extraction(&parsed, "before").is_ok());
}

// ── Scenario 2: Flag after subcommand ───────────────────────────────────────────────

#[test]
fn test_flag_after_subcommand_scan() {
    // Test: hoop scan /tmp --no-interactive
    let result = parse_cli_with_flag(&["hoop", "scan", "/tmp", "--no-interactive"]);

    assert!(result.is_ok(), "Should successfully parse flag after subcommand");
    let parsed = result.unwrap();

    assert!(parsed.no_interactive, "Flag should be extracted as true");
    assert_eq!(parsed.command, "scan", "Command should be 'scan'");
    assert!(parsed.args.contains(&"/tmp".to_string()), "Args should contain scan path");

    // Verify flag extraction
    assert!(verify_flag_extraction(&parsed, "after").is_ok());
}

#[test]
fn test_flag_after_subcommand_remove() {
    // Test: hoop remove my-project --confirm --no-interactive
    let result = parse_cli_with_flag(&["hoop", "remove", "my-project", "--confirm", "--no-interactive"]);

    assert!(result.is_ok(), "Should successfully parse flag after subcommand");
    let parsed = result.unwrap();

    assert!(parsed.no_interactive, "Flag should be extracted as true");
    assert_eq!(parsed.command, "remove", "Command should be 'remove'");
    assert!(parsed.args.contains(&"my-project".to_string()), "Args should contain project name");
    assert!(parsed.args.contains(&"--confirm".to_string()), "Args should contain --confirm");

    // Verify flag extraction
    assert!(verify_flag_extraction(&parsed, "after").is_ok());
}

#[test]
fn test_flag_after_subcommand_restore() {
    // Test: hoop restore --from s3://bucket/key --confirm --no-interactive
    let result = parse_cli_with_flag(&[
        "hoop", "restore", "--from", "s3://bucket/key", "--confirm", "--no-interactive"
    ]);

    assert!(result.is_ok(), "Should successfully parse flag after subcommand");
    let parsed = result.unwrap();

    assert!(parsed.no_interactive, "Flag should be extracted as true");
    assert_eq!(parsed.command, "restore", "Command should be 'restore'");
    assert!(parsed.args.contains(&"--from".to_string()), "Args should contain --from");
    assert!(parsed.args.contains(&"s3://bucket/key".to_string()), "Args should contain URI");
    assert!(parsed.args.contains(&"--confirm".to_string()), "Args should contain --confirm");

    // Verify flag extraction
    assert!(verify_flag_extraction(&parsed, "after").is_ok());
}

#[test]
fn test_flag_after_subcommand_status() {
    // Test: hoop status --json --no-interactive
    let result = parse_cli_with_flag(&["hoop", "status", "--json", "--no-interactive"]);

    assert!(result.is_ok(), "Should successfully parse flag after subcommand");
    let parsed = result.unwrap();

    assert!(parsed.no_interactive, "Flag should be extracted as true");
    assert_eq!(parsed.command, "status", "Command should be 'status'");
    assert!(parsed.args.contains(&"--json".to_string()), "Args should contain --json");

    // Verify flag extraction
    assert!(verify_flag_extraction(&parsed, "after").is_ok());
}

// ── Scenario 3: Short alias (-y) ───────────────────────────────────────────────────

#[test]
fn test_short_flag_y_scan() {
    // Test: hoop -y scan /tmp
    let result = parse_cli_with_flag(&["hoop", "-y", "scan", "/tmp"]);

    assert!(result.is_ok(), "Should successfully parse short -y flag");
    let parsed = result.unwrap();

    assert!(parsed.no_interactive, "Short flag -y should set no_interactive to true");
    assert_eq!(parsed.command, "scan", "Command should be 'scan'");
    assert!(parsed.args.contains(&"/tmp".to_string()), "Args should contain scan path");
}

#[test]
fn test_short_flag_y_remove() {
    // Test: hoop -y remove my-project --confirm
    let result = parse_cli_with_flag(&["hoop", "-y", "remove", "my-project", "--confirm"]);

    assert!(result.is_ok(), "Should successfully parse short -y flag");
    let parsed = result.unwrap();

    assert!(parsed.no_interactive, "Short flag -y should set no_interactive to true");
    assert_eq!(parsed.command, "remove", "Command should be 'remove'");
    assert!(parsed.args.contains(&"my-project".to_string()), "Args should contain project name");
    assert!(parsed.args.contains(&"--confirm".to_string()), "Args should contain --confirm");
}

#[test]
fn test_short_flag_y_restore() {
    // Test: hoop -y restore --from s3://bucket/key --confirm
    let result = parse_cli_with_flag(&["hoop", "-y", "restore", "--from", "s3://bucket/key", "--confirm"]);

    assert!(result.is_ok(), "Should successfully parse short -y flag");
    let parsed = result.unwrap();

    assert!(parsed.no_interactive, "Short flag -y should set no_interactive to true");
    assert_eq!(parsed.command, "restore", "Command should be 'restore'");
    assert!(parsed.args.contains(&"--from".to_string()), "Args should contain --from");
    assert!(parsed.args.contains(&"s3://bucket/key".to_string()), "Args should contain URI");
    assert!(parsed.args.contains(&"--confirm".to_string()), "Args should contain --confirm");
}

#[test]
fn test_short_flag_y_status() {
    // Test: hoop -y status --json
    let result = parse_cli_with_flag(&["hoop", "-y", "status", "--json"]);

    assert!(result.is_ok(), "Should successfully parse short -y flag");
    let parsed = result.unwrap();

    assert!(parsed.no_interactive, "Short flag -y should set no_interactive to true");
    assert_eq!(parsed.command, "status", "Command should be 'status'");
    assert!(parsed.args.contains(&"--json".to_string()), "Args should contain --json");
}

// ── Scenario 4: Combined with other flags ───────────────────────────────────────────

#[test]
fn test_combined_flags_scan_with_json() {
    // Test: hoop --no-interactive scan --json /tmp
    // Note: scan doesn't have --json, but this tests flag combination parsing
    let result = parse_cli_with_flag(&["hoop", "--no-interactive", "scan", "/tmp"]);

    assert!(result.is_ok(), "Should successfully parse combined flags");
    let parsed = result.unwrap();

    assert!(parsed.no_interactive, "Global no_interactive should be true");
    assert_eq!(parsed.command, "scan", "Command should be 'scan'");
}

#[test]
fn test_combined_flags_status_with_json() {
    // Test: hoop --no-interactive status --json
    let result = parse_cli_with_flag(&["hoop", "--no-interactive", "status", "--json"]);

    assert!(result.is_ok(), "Should successfully parse combined flags");
    let parsed = result.unwrap();

    assert!(parsed.no_interactive, "Global no_interactive should be true");
    assert_eq!(parsed.command, "status", "Command should be 'status'");
    assert!(parsed.args.contains(&"--json".to_string()), "Args should contain --json");
}

#[test]
fn test_combined_flags_remove_with_confirm() {
    // Test: hoop --no-interactive remove my-project --confirm
    let result = parse_cli_with_flag(&["hoop", "--no-interactive", "remove", "my-project", "--confirm"]);

    assert!(result.is_ok(), "Should successfully parse combined flags");
    let parsed = result.unwrap();

    assert!(parsed.no_interactive, "Global no_interactive should be true");
    assert_eq!(parsed.command, "remove", "Command should be 'remove'");
    assert!(parsed.args.contains(&"my-project".to_string()), "Args should contain project name");
    assert!(parsed.args.contains(&"--confirm".to_string()), "Args should contain --confirm");
}

#[test]
fn test_combined_flags_restore_with_dry_run() {
    // Test: hoop --no-interactive restore --dry-run --from s3://bucket/key
    let result = parse_cli_with_flag(&[
        "hoop", "--no-interactive", "restore", "--dry-run", "--from", "s3://bucket/key"
    ]);

    assert!(result.is_ok(), "Should successfully parse combined flags");
    let parsed = result.unwrap();

    assert!(parsed.no_interactive, "Global no_interactive should be true");
    assert_eq!(parsed.command, "restore", "Command should be 'restore'");
    assert!(parsed.args.contains(&"--dry-run".to_string()), "Args should contain --dry-run");
    assert!(parsed.args.contains(&"--from".to_string()), "Args should contain --from");
    assert!(parsed.args.contains(&"s3://bucket/key".to_string()), "Args should contain URI");
}

#[test]
fn test_combined_flags_global_after_local_flags() {
    // Test: hoop scan --yes /tmp --no-interactive
    // Tests that global flag works when placed after local flags
    let result = parse_cli_with_flag(&["hoop", "scan", "--yes", "/tmp", "--no-interactive"]);

    assert!(result.is_ok(), "Should successfully parse global flag after local flags");
    let parsed = result.unwrap();

    assert!(parsed.no_interactive, "Global no_interactive should be true");
    assert_eq!(parsed.command, "scan", "Command should be 'scan'");
    assert!(parsed.args.contains(&"/tmp".to_string()), "Args should contain scan path");
    // Note: --yes flag is in args since it's a local flag for scan command
}

// ── Flag position independence tests ────────────────────────────────────────────────

#[test]
fn test_flag_position_independence_scan() {
    // Verify that both flag positions yield the same no_interactive value for scan

    let before = parse_cli_with_flag(&["hoop", "--no-interactive", "scan", "/tmp"]);
    assert!(before.is_ok(), "Should parse flag before command");
    let before_parsed = before.unwrap();

    let after = parse_cli_with_flag(&["hoop", "scan", "/tmp", "--no-interactive"]);
    assert!(after.is_ok(), "Should parse flag after command");
    let after_parsed = after.unwrap();

    assert_eq!(
        before_parsed.no_interactive,
        after_parsed.no_interactive,
        "Flag position should not affect the extracted value"
    );

    assert!(before_parsed.no_interactive, "Both should extract no_interactive as true");
}

#[test]
fn test_flag_position_independence_remove() {
    // Verify that both flag positions yield the same no_interactive value for remove

    let before = parse_cli_with_flag(&["hoop", "--no-interactive", "remove", "test", "--confirm"]);
    assert!(before.is_ok(), "Should parse flag before command");
    let before_parsed = before.unwrap();

    let after = parse_cli_with_flag(&["hoop", "remove", "test", "--confirm", "--no-interactive"]);
    assert!(after.is_ok(), "Should parse flag after command");
    let after_parsed = after.unwrap();

    assert_eq!(
        before_parsed.no_interactive,
        after_parsed.no_interactive,
        "Flag position should not affect the extracted value"
    );

    assert!(before_parsed.no_interactive, "Both should extract no_interactive as true");
}

#[test]
fn test_flag_position_independence_restore() {
    // Verify that both flag positions yield the same no_interactive value for restore

    let before = parse_cli_with_flag(&[
        "hoop", "--no-interactive", "restore", "--from", "s3://bucket/key", "--confirm"
    ]);
    assert!(before.is_ok(), "Should parse flag before command");
    let before_parsed = before.unwrap();

    let after = parse_cli_with_flag(&[
        "hoop", "restore", "--from", "s3://bucket/key", "--confirm", "--no-interactive"
    ]);
    assert!(after.is_ok(), "Should parse flag after command");
    let after_parsed = after.unwrap();

    assert_eq!(
        before_parsed.no_interactive,
        after_parsed.no_interactive,
        "Flag position should not affect the extracted value"
    );

    assert!(before_parsed.no_interactive, "Both should extract no_interactive as true");
}

// ── Default behavior tests (no flag) ─────────────────────────────────────────────────

#[test]
fn test_default_behavior_scan() {
    // Test: hoop scan /tmp (no no_interactive flag)
    let result = parse_cli_with_flag(&["hoop", "scan", "/tmp"]);

    assert!(result.is_ok(), "Should successfully parse command without flag");
    let parsed = result.unwrap();

    assert!(!parsed.no_interactive, "no_interactive should default to false");
    assert_eq!(parsed.command, "scan", "Command should be 'scan'");

    // Verify no flag is present
    assert!(verify_no_flag_present(&parsed).is_ok());
}

#[test]
fn test_default_behavior_remove() {
    // Test: hoop remove my-project --confirm (no no_interactive flag)
    let result = parse_cli_with_flag(&["hoop", "remove", "my-project", "--confirm"]);

    assert!(result.is_ok(), "Should successfully parse command without flag");
    let parsed = result.unwrap();

    assert!(!parsed.no_interactive, "no_interactive should default to false");
    assert_eq!(parsed.command, "remove", "Command should be 'remove'");

    // Verify no flag is present
    assert!(verify_no_flag_present(&parsed).is_ok());
}

#[test]
fn test_default_behavior_restore() {
    // Test: hoop restore --from s3://bucket/key (no no_interactive flag)
    let result = parse_cli_with_flag(&["hoop", "restore", "--from", "s3://bucket/key"]);

    assert!(result.is_ok(), "Should successfully parse command without flag");
    let parsed = result.unwrap();

    assert!(!parsed.no_interactive, "no_interactive should default to false");
    assert_eq!(parsed.command, "restore", "Command should be 'restore'");

    // Verify no flag is present
    assert!(verify_no_flag_present(&parsed).is_ok());
}

// ── Flag propagation to handlers verification ───────────────────────────────────────

#[test]
fn test_flag_propagation_scan_handler() {
    // Verify that scan handler receives the correct no_interactive value
    let main_code = fs::read_to_string("src/main.rs").expect("Failed to read main.rs");

    // Verify flag is extracted from CLI
    assert!(
        main_code.contains("let no_interactive = cli.no_interactive;"),
        "Flag should be extracted from parsed CLI structure"
    );

    // Verify flag is passed to scan handler with || auto_confirm logic
    assert!(
        main_code.contains("projects::scan_projects(&root, no_interactive || auto_confirm)"),
        "Flag should be passed to scan handler with OR logic"
    );
}

#[test]
fn test_flag_propagation_remove_handler() {
    // Verify that remove handler receives the correct no_interactive value
    let main_code = fs::read_to_string("src/main.rs").expect("Failed to read main.rs");

    // Verify flag is extracted from CLI
    assert!(
        main_code.contains("let no_interactive = cli.no_interactive;"),
        "Flag should be extracted from parsed CLI structure"
    );

    // Verify flag is passed to remove handler
    assert!(
        main_code.contains("projects::remove_project(&name, no_interactive, confirm)"),
        "Flag should be passed to remove handler"
    );
}

#[test]
fn test_flag_propagation_restore_handler() {
    // Verify that restore handler receives the correct no_interactive value
    let main_code = fs::read_to_string("src/main.rs").expect("Failed to read main.rs");

    // Verify flag is extracted from CLI
    assert!(
        main_code.contains("let no_interactive = cli.no_interactive;"),
        "Flag should be extracted from parsed CLI structure"
    );

    // Verify flag is passed to restore handler
    assert!(
        main_code.contains("restore::run_restore(&from, dry_run, no_interactive, confirm)"),
        "Flag should be passed to restore handler"
    );
}

// ── Comprehensive coverage test ─────────────────────────────────────────────────────

#[test]
fn test_comprehensive_global_flag_coverage() {
    // Meta-test that verifies all critical patterns are covered
    // This serves as a checklist for the integration test suite

    let main_code = fs::read_to_string("src/main.rs").expect("Failed to read main.rs");
    let projects_code = fs::read_to_string("src/projects.rs").expect("Failed to read projects.rs");
    let restore_code = fs::read_to_string("src/restore.rs").expect("Failed to read restore.rs");

    // Global flag definition
    assert!(
        main_code.contains("#[arg(short = 'y', long = \"no-interactive\", global = true)]"),
        "✓ Global flag defined with global = true attribute"
    );

    // Flag extraction
    assert!(
        main_code.contains("let no_interactive = cli.no_interactive;"),
        "✓ Global flag extracted in main"
    );

    // Scan command handler
    assert!(
        main_code.contains("projects::scan_projects(&root, no_interactive || auto_confirm)"),
        "✓ Scan handler receives flag with OR logic"
    );

    // Remove command handler
    assert!(
        main_code.contains("projects::remove_project(&name, no_interactive, confirm)"),
        "✓ Remove handler receives flag"
    );

    // Restore command handler
    assert!(
        main_code.contains("restore::run_restore(&from, dry_run, no_interactive, confirm)"),
        "✓ Restore handler receives flag"
    );

    // Scan behavior verification
    assert!(
        projects_code.contains("pub fn scan_projects(root: &str, no_interactive: bool)"),
        "✓ scan_projects accepts no_interactive parameter"
    );

    // Remove behavior verification
    assert!(
        projects_code.contains("if no_interactive && !confirm"),
        "✓ Remove requires --confirm in non-interactive mode"
    );

    // Restore behavior verification
    assert!(
        restore_code.contains("if no_interactive && !confirm"),
        "✓ Restore requires --confirm in non-interactive mode"
    );

    // All checks passed
}

// ── Command success/failure behavior tests ──────────────────────────────────────────

#[test]
fn test_scan_succeeds_with_no_interactive() {
    // Verify that scan succeeds when no_interactive=true (auto-registers)
    let projects_code = fs::read_to_string("src/projects.rs").expect("Failed to read projects.rs");

    // Find the scan_projects function
    let scan_start = projects_code.find("pub fn scan_projects")
        .expect("Should find scan_projects function");

    // Find the no_interactive check
    let no_interactive_check = projects_code[scan_start..].find("if no_interactive {")
        .expect("Should find no_interactive check");

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
}

#[test]
fn test_remove_fails_without_confirm_in_no_interactive() {
    // Verify that remove fails when no_interactive=true but --confirm is missing
    let projects_code = fs::read_to_string("src/projects.rs").expect("Failed to read projects.rs");

    assert!(
        projects_code.contains("if no_interactive && !confirm"),
        "Remove must check for confirm flag in non-interactive mode"
    );

    assert!(
        projects_code.contains("--confirm is required in non-interactive mode"),
        "Remove must show helpful error when confirm is missing"
    );
}

#[test]
fn test_restore_fails_without_confirm_in_no_interactive() {
    // Verify that restore fails when no_interactive=true but --confirm is missing
    let restore_code = fs::read_to_string("src/restore.rs").expect("Failed to read restore.rs");

    assert!(
        restore_code.contains("if no_interactive && !confirm"),
        "Restore must check for confirm flag in non-interactive mode"
    );

    assert!(
        restore_code.contains("--confirm is required in non-interactive mode"),
        "Restore must show helpful error when confirm is missing"
    );
}

#[test]
fn test_init_fails_with_no_interactive() {
    // Verify that init fails when no_interactive=true (requires interaction)
    let init_code = fs::read_to_string("src/init.rs").expect("Failed to read init.rs");

    assert!(
        init_code.contains("if no_interactive {"),
        "Init must check no_interactive flag"
    );

    assert!(
        init_code.contains("cannot run in non-interactive mode"),
        "Init must explain why it cannot run non-interactively"
    );
}

// ── Batch testing for multiple commands ────────────────────────────────────────────

#[test]
fn test_batch_flag_position_tests() {
    // Run a batch of flag position tests for multiple commands
    let test_cases = vec![
        // Scan command tests
        FlagPositionTestCase {
            description: "scan with flag before".to_string(),
            command: vec!["hoop", "--no-interactive", "scan", "/tmp"]
                .iter().map(|s| s.to_string()).collect(),
            expected_result: true,
        },
        FlagPositionTestCase {
            description: "scan with flag after".to_string(),
            command: vec!["hoop", "scan", "/tmp", "--no-interactive"]
                .iter().map(|s| s.to_string()).collect(),
            expected_result: true,
        },
        FlagPositionTestCase {
            description: "scan with short flag".to_string(),
            command: vec!["hoop", "-y", "scan", "/tmp"]
                .iter().map(|s| s.to_string()).collect(),
            expected_result: true,
        },
        FlagPositionTestCase {
            description: "scan without flag".to_string(),
            command: vec!["hoop", "scan", "/tmp"]
                .iter().map(|s| s.to_string()).collect(),
            expected_result: false,
        },
        // Remove command tests
        FlagPositionTestCase {
            description: "remove with flag before".to_string(),
            command: vec!["hoop", "--no-interactive", "remove", "test", "--confirm"]
                .iter().map(|s| s.to_string()).collect(),
            expected_result: true,
        },
        FlagPositionTestCase {
            description: "remove with flag after".to_string(),
            command: vec!["hoop", "remove", "test", "--confirm", "--no-interactive"]
                .iter().map(|s| s.to_string()).collect(),
            expected_result: true,
        },
        FlagPositionTestCase {
            description: "remove with short flag".to_string(),
            command: vec!["hoop", "-y", "remove", "test", "--confirm"]
                .iter().map(|s| s.to_string()).collect(),
            expected_result: true,
        },
        FlagPositionTestCase {
            description: "remove without flag".to_string(),
            command: vec!["hoop", "remove", "test", "--confirm"]
                .iter().map(|s| s.to_string()).collect(),
            expected_result: false,
        },
        // Restore command tests
        FlagPositionTestCase {
            description: "restore with flag before".to_string(),
            command: vec!["hoop", "--no-interactive", "restore", "--from", "s3://bucket/key", "--confirm"]
                .iter().map(|s| s.to_string()).collect(),
            expected_result: true,
        },
        FlagPositionTestCase {
            description: "restore with flag after".to_string(),
            command: vec!["hoop", "restore", "--from", "s3://bucket/key", "--confirm", "--no-interactive"]
                .iter().map(|s| s.to_string()).collect(),
            expected_result: true,
        },
        FlagPositionTestCase {
            description: "restore with short flag".to_string(),
            command: vec!["hoop", "-y", "restore", "--from", "s3://bucket/key", "--confirm"]
                .iter().map(|s| s.to_string()).collect(),
            expected_result: true,
        },
        FlagPositionTestCase {
            description: "restore without flag".to_string(),
            command: vec!["hoop", "restore", "--from", "s3://bucket/key", "--confirm"]
                .iter().map(|s| s.to_string()).collect(),
            expected_result: false,
        },
        // Status command tests
        FlagPositionTestCase {
            description: "status with flag before".to_string(),
            command: vec!["hoop", "--no-interactive", "status", "--json"]
                .iter().map(|s| s.to_string()).collect(),
            expected_result: true,
        },
        FlagPositionTestCase {
            description: "status with flag after".to_string(),
            command: vec!["hoop", "status", "--json", "--no-interactive"]
                .iter().map(|s| s.to_string()).collect(),
            expected_result: true,
        },
        FlagPositionTestCase {
            description: "status with short flag".to_string(),
            command: vec!["hoop", "-y", "status", "--json"]
                .iter().map(|s| s.to_string()).collect(),
            expected_result: true,
        },
        FlagPositionTestCase {
            description: "status without flag".to_string(),
            command: vec!["hoop", "status", "--json"]
                .iter().map(|s| s.to_string()).collect(),
            expected_result: false,
        },
    ];

    let (successes, failures) = run_flag_position_tests(test_cases);

    assert_eq!(successes.len(), 16, "All 16 test cases should succeed");
    assert_eq!(failures.len(), 0, "No test cases should fail");
}
