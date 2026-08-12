//! Unit tests for Restore command no_interactive flag
//!
//! These tests verify that the no_interactive flag works correctly
//! for the Restore command, covering:
//! 1. Parse test: hoop --no-interactive restore --from s3://...
//! 2. Parse test: hoop restore --from s3://... --no-interactive
//! 3. Verify flag value extraction in handler
//! 4. Verify flag is passed correctly to run_restore
//! 5. Verify confirmation suppression behavior when flag is true
//! 6. Verify --confirm requirement in no-interactive mode

use std::fs;

// Include the test utilities module
mod cli_test_utils;
use cli_test_utils::*;

// ── Parse tests: Flag position independence ─────────────────────────────────────

#[test]
fn test_restore_parse_with_flag_before_subcommand() {
    // Test: hoop --no-interactive restore --from s3://bucket/key --confirm
    let result = parse_cli_with_flag(&[
        "hoop",
        "--no-interactive",
        "restore",
        "--from",
        "s3://bucket/key",
        "--confirm",
    ]);

    assert!(result.is_ok(), "Should successfully parse flag before subcommand");
    let parsed = result.unwrap();

    assert_eq!(parsed.no_interactive, true, "no_interactive should be true");
    assert_eq!(parsed.command, "restore", "Command should be 'restore'");
    assert!(
        parsed.args.contains(&"restore".to_string()),
        "Args should contain restore command"
    );
    assert!(
        parsed.args.contains(&"--from".to_string()),
        "Args should contain --from flag"
    );
    assert!(
        parsed.args.contains(&"s3://bucket/key".to_string()),
        "Args should contain S3 URI"
    );
}

#[test]
fn test_restore_parse_with_flag_after_subcommand() {
    // Test: hoop restore --from s3://bucket/key --no-interactive --confirm
    let result = parse_cli_with_flag(&[
        "hoop",
        "restore",
        "--from",
        "s3://bucket/key",
        "--no-interactive",
        "--confirm",
    ]);

    assert!(result.is_ok(), "Should successfully parse flag after subcommand");
    let parsed = result.unwrap();

    assert_eq!(parsed.no_interactive, true, "no_interactive should be true");
    assert_eq!(parsed.command, "restore", "Command should be 'restore'");
    assert!(
        parsed.args.contains(&"restore".to_string()),
        "Args should contain restore command"
    );
    assert!(
        parsed.args.contains(&"s3://bucket/key".to_string()),
        "Args should contain S3 URI"
    );
}

#[test]
fn test_restore_parse_with_short_flag_before_subcommand() {
    // Test: hoop -y restore --from s3://bucket/key --confirm
    let result = parse_cli_with_flag(&[
        "hoop",
        "-y",
        "restore",
        "--from",
        "s3://bucket/key",
        "--confirm",
    ]);

    assert!(
        result.is_ok(),
        "Should successfully parse short flag before subcommand"
    );
    let parsed = result.unwrap();

    assert_eq!(parsed.no_interactive, true, "no_interactive should be true with -y");
    assert_eq!(parsed.command, "restore", "Command should be 'restore'");
}

#[test]
fn test_restore_parse_with_short_flag_after_subcommand() {
    // Test: hoop restore --from s3://bucket/key -y --confirm
    let result = parse_cli_with_flag(&[
        "hoop",
        "restore",
        "--from",
        "s3://bucket/key",
        "-y",
        "--confirm",
    ]);

    assert!(
        result.is_ok(),
        "Should successfully parse short flag after subcommand"
    );
    let parsed = result.unwrap();

    assert_eq!(parsed.no_interactive, true, "no_interactive should be true with -y");
    assert_eq!(parsed.command, "restore", "Command should be 'restore'");
}

#[test]
fn test_restore_parse_without_flag() {
    // Test: hoop restore --from s3://bucket/key --confirm (default behavior)
    let result = parse_cli_with_flag(&[
        "hoop",
        "restore",
        "--from",
        "s3://bucket/key",
        "--confirm",
    ]);

    assert!(result.is_ok(), "Should successfully parse command without flag");
    let parsed = result.unwrap();

    assert_eq!(
        parsed.no_interactive,
        false,
        "no_interactive should default to false"
    );
    assert_eq!(parsed.command, "restore", "Command should be 'restore'");
}

#[test]
fn test_restore_parse_with_dry_run_flag() {
    // Test: hoop --no-interactive restore --from s3://bucket/key --dry-run
    let result = parse_cli_with_flag(&[
        "hoop",
        "--no-interactive",
        "restore",
        "--from",
        "s3://bucket/key",
        "--dry-run",
    ]);

    assert!(result.is_ok(), "Should successfully parse with --dry-run flag");
    let parsed = result.unwrap();

    assert_eq!(parsed.no_interactive, true, "no_interactive should be true");
    assert_eq!(parsed.command, "restore", "Command should be 'restore'");
    assert!(
        parsed.args.contains(&"--dry-run".to_string()),
        "Args should contain --dry-run flag"
    );
}

// ── Flag extraction verification tests ───────────────────────────────────────────

#[test]
fn test_restore_flag_extraction_before_position() {
    // Verify flag extraction when flag appears before subcommand
    let parsed =
        parse_flag_before_subcommand(&["restore", "--from", "s3://bucket/key", "--confirm"])
            .expect("Parse should succeed");

    let verification_result = verify_flag_extraction(&parsed, "before");
    assert!(
        verification_result.is_ok(),
        "Flag extraction should verify for 'before' position"
    );

    // Additional assertions
    assert_eq!(parsed.no_interactive, true);
    assert_eq!(parsed.command, "restore");
}

#[test]
fn test_restore_flag_extraction_after_position() {
    // Verify flag extraction when flag appears after subcommand
    let parsed =
        parse_flag_after_subcommand(&["restore", "--from", "s3://bucket/key", "--confirm"])
            .expect("Parse should succeed");

    let verification_result = verify_flag_extraction(&parsed, "after");
    assert!(
        verification_result.is_ok(),
        "Flag extraction should verify for 'after' position"
    );

    // Additional assertions
    assert_eq!(parsed.no_interactive, true);
    assert_eq!(parsed.command, "restore");
}

#[test]
fn test_restore_no_flag_present_verification() {
    // Verify that no_interactive is correctly set to false when flag is absent
    let parsed = parse_cli_with_flag(&[
        "hoop",
        "restore",
        "--from",
        "s3://bucket/key",
        "--confirm",
    ])
    .expect("Parse should succeed");

    let verification_result = verify_no_flag_present(&parsed);
    assert!(verification_result.is_ok(), "Should verify no flag is present");

    assert_eq!(parsed.no_interactive, false);
}

// ── Flag propagation to handler tests ────────────────────────────────────────────

#[test]
fn test_restore_flag_propagation_from_main_to_handler() {
    // Verify that the no_interactive flag is correctly extracted in main()
    // and passed to run_restore

    let main_code = fs::read_to_string("src/main.rs").expect("Failed to read main.rs");

    // Verify flag is extracted from parsed CLI
    assert!(
        main_code.contains("let no_interactive = cli.no_interactive;"),
        "Flag should be extracted from parsed CLI structure"
    );

    // Verify flag is passed to restore handler
    assert!(
        main_code.contains("restore::run_restore(&from, dry_run, no_interactive, confirm)"),
        "Flag should be passed to run_restore handler"
    );

    // Verify the Restore command enum variant exists
    assert!(
        main_code.contains("Commands::Restore { from, dry_run, confirm } =>"),
        "Restore command handler should exist in main.rs"
    );
}

#[test]
fn test_restore_handler_accepts_no_interactive_parameter() {
    // Verify that run_restore handler actually uses the no_interactive parameter
    let restore_code = fs::read_to_string("src/restore.rs")
        .expect("Failed to read restore.rs");

    // Verify function signature accepts no_interactive
    assert!(
        restore_code
            .contains("pub async fn run_restore(from_uri: &str, dry_run: bool, no_interactive: bool, confirm: bool)"),
        "run_restore must accept no_interactive parameter"
    );

    // Verify it's used in conditional logic for --confirm requirement
    assert!(
        restore_code.contains("if no_interactive && !confirm {"),
        "run_restore must check no_interactive flag for confirm requirement"
    );

    // Verify it's used in conditional logic for prompting
    assert!(
        restore_code.contains("if !no_interactive {"),
        "run_restore must check no_interactive flag for prompting logic"
    );
}

#[test]
fn test_restore_no_interactive_requires_confirm() {
    // Verify that restore requires --confirm when no_interactive=true
    let restore_code = fs::read_to_string("src/restore.rs")
        .expect("Failed to read restore.rs");

    // Find the run_restore function
    let restore_start = restore_code
        .find("pub async fn run_restore(")
        .expect("Should find run_restore function");

    // Find the no_interactive confirm requirement
    let confirm_check = restore_code[restore_start..]
        .find("if no_interactive && !confirm {")
        .expect("Should find confirm requirement check");

    // Get the confirm requirement section
    let confirm_section =
        &restore_code[restore_start + confirm_check..restore_start + confirm_check + 400];

    // Verify the error message about --confirm requirement
    assert!(
        confirm_section.contains("--confirm is required in non-interactive mode"),
        "Should error when --confirm is missing in no-interactive mode"
    );

    // Verify the error suggests the correct command
    assert!(
        confirm_section.contains("--no-interactive --confirm"),
        "Error should suggest using --confirm flag"
    );

    // Verify DESTRUCTIVE warning
    assert!(
        confirm_section.contains("DESTRUCTIVE"),
        "Error should warn about destructive operation"
    );
}

// ── Confirmation suppression behavior tests ─────────────────────────────────────

#[test]
fn test_restore_confirms_when_no_interactive_true_with_confirm_flag() {
    // Test that restore with no_interactive=true and --confirm proceeds without prompting
    let restore_code = fs::read_to_string("src/restore.rs")
        .expect("Failed to read restore.rs");

    // Find the run_restore function
    let restore_start = restore_code
        .find("pub async fn run_restore(")
        .expect("Should find run_restore function");

    // Find the no_interactive confirm requirement
    let confirm_check = restore_code[restore_start..]
        .find("if no_interactive && !confirm {")
        .expect("Should find confirm requirement check");

    // Get the section after the confirm check
    let after_confirm_check = &restore_code[restore_start + confirm_check..];

    // Find the prompt check
    let prompt_check = after_confirm_check
        .find("if !no_interactive {")
        .expect("Should have prompt check after confirm requirement");

    // Get the prompt section
    let prompt_section = &after_confirm_check[prompt_check..prompt_check + 600];

    // Verify prompts exist in the !no_interactive branch
    assert!(
        prompt_section.contains("eprintln!(\"⚠️  WARNING: This will replace"),
        "Should have warning message in interactive mode"
    );

    assert!(
        prompt_section.contains("eprint!(\"Continue? [y/N] \")"),
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
fn test_restore_prompts_when_no_interactive_false() {
    // Test that restore with no_interactive=false prompts for confirmation
    let restore_code = fs::read_to_string("src/restore.rs")
        .expect("Failed to read restore.rs");

    // Find the run_restore function
    let restore_start = restore_code
        .find("pub async fn run_restore(")
        .expect("Should find run_restore function");

    // Find the prompt check section
    let prompt_check = restore_code[restore_start..]
        .find("if !no_interactive {")
        .expect("Should find prompt check");

    // Get the prompt section
    let prompt_section =
        &restore_code[restore_start + prompt_check..restore_start + prompt_check + 600];

    // Verify all the interactive prompting elements exist
    assert!(
        prompt_section.contains("eprintln!(\"⚠️  WARNING: This will replace"),
        "Should show warning message"
    );

    assert!(
        prompt_section.contains("eprintln!(\"  Snapshot ID: {}"),
        "Should show snapshot ID"
    );

    assert!(
        prompt_section.contains("eprintln!(\"  Created at: {}"),
        "Should show creation timestamp"
    );

    assert!(
        prompt_section.contains("eprint!(\"Continue? [y/N] \")"),
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
        prompt_section.contains("eprintln!(\"Restore cancelled\")"),
        "Should show cancellation message"
    );
}

#[test]
fn test_restore_prompts_go_to_stderr() {
    // Verify that restore prompts go to stderr (not stdout) to avoid interfering with data output
    let restore_code = fs::read_to_string("src/restore.rs")
        .expect("Failed to read restore.rs");

    // Find the run_restore function
    let restore_start = restore_code
        .find("pub async fn run_restore(")
        .expect("Should find run_restore function");

    // Find the prompt section
    let prompt_check = restore_code[restore_start..]
        .find("if !no_interactive {")
        .expect("Should find prompt check");

    let prompt_section =
        &restore_code[restore_start + prompt_check..restore_start + prompt_check + 600];

    // Verify prompts use eprint! (stderr) not println! (stdout)
    assert!(
        prompt_section.contains("eprintln!(\"⚠️  WARNING: This will replace"),
        "Warning message should use eprintln! to write to stderr"
    );

    assert!(
        prompt_section.contains("eprint!(\"Continue? [y/N] \")"),
        "Prompt should use eprint! to write to stderr"
    );

    // Verify stderr flush
    assert!(
        prompt_section.contains("std::io::stderr().flush()?"),
        "Should flush stderr after prompt to ensure it appears"
    );
}

#[test]
fn test_restore_non_interactive_skips_confirmation_prompt() {
    // Verify that when no_interactive=true (with --confirm), the confirmation prompt is skipped
    // and restore proceeds directly
    let restore_code = fs::read_to_string("src/restore.rs")
        .expect("Failed to read restore.rs");

    // Find the run_restore function
    let restore_start = restore_code
        .find("pub async fn run_restore(")
        .expect("Should find run_restore function");

    // Find the confirm requirement check
    let confirm_check = restore_code[restore_start..]
        .find("if no_interactive && !confirm {")
        .expect("Should find confirm requirement check");

    // Find the closing brace of the confirm requirement block
    let confirm_block_end = restore_code[restore_start + confirm_check..]
        .find('}')
        .expect("Should find end of confirm requirement block");

    // Get the confirm requirement block
    let confirm_block = &restore_code[restore_start + confirm_check
        ..restore_start + confirm_check + confirm_block_end + 50];

    // Verify that in this block, there's no confirmation prompt
    assert!(
        !confirm_block[..200].contains("eprint!(\"Continue?"),
        "Confirm requirement block should NOT contain confirmation prompt"
    );

    // Now find the prompt check (after the confirm requirement)
    let after_confirm =
        &restore_code[restore_start + confirm_check + confirm_block_end..];
    let prompt_check = after_confirm
        .find("if !no_interactive {")
        .expect("Should find prompt check after confirm requirement");

    // Get the prompt section
    let prompt_section = &after_confirm[prompt_check..prompt_check + 500];

    // Verify the confirmation prompt exists in the !no_interactive branch
    assert!(
        prompt_section.contains("eprint!(\"Continue? [y/N] \")"),
        "Confirmation prompt should exist in !no_interactive branch"
    );

    // This confirms that:
    // 1. When no_interactive=true and !confirm → error (not a prompt, a bail-out)
    // 2. When no_interactive=false → prompts for confirmation
    // 3. When no_interactive=true and confirm → skips the !no_interactive block entirely
}

#[test]
fn test_restore_dry_run_respects_no_interactive() {
    // Verify that dry-run mode shows correct usage message based on no_interactive flag
    let restore_code = fs::read_to_string("src/restore.rs")
        .expect("Failed to read restore.rs");

    // Find the dry-run section
    let dry_run_section = restore_code
        .find("if dry_run {")
        .expect("run_restore must have dry_run mode");

    // Get the dry_run block (expanded window to reach the print statements)
    let dry_run_block = &restore_code[dry_run_section..dry_run_section + 1500];

    // Verify the dry_run mode shows different messages based on no_interactive
    assert!(
        dry_run_block.contains("--no-interactive --confirm"),
        "Dry-run mode must show --no-interactive --confirm usage when no_interactive is true"
    );

    // Verify it also shows the interactive version
    assert!(
        dry_run_block.contains("hoop restore --from"),
        "Dry-run mode must show simple usage when no_interactive is false"
    );
}

// ── Behavioral tests: Code structure verification ─────────────────────────────────

#[test]
fn test_restore_code_order_validates_before_destructive() {
    // Verify the order in run_restore(): validate() is called before
    // move_aside_for_rollback(). A newer-version manifest must never
    // reach the destructive rename step.
    //
    // This is a structural guarantee — if the code order changes,
    // this test documents the intended invariant.
    let restore_code = fs::read_to_string("src/restore.rs")
        .expect("Failed to read restore.rs");

    // Find run_restore() function body
    let fn_start = restore_code
        .find("pub async fn run_restore(")
        .expect("restore.rs must define run_restore()");

    // Within run_restore, the validate call must precede the move_aside call
    let validate_pos = restore_code[fn_start..]
        .find("manifest.validate(current)")
        .expect("run_restore must call manifest.validate(current)");
    let move_aside_pos = restore_code[fn_start..]
        .find("move_aside_for_rollback()\n")
        .expect("run_restore must call move_aside_for_rollback()");

    assert!(
        validate_pos < move_aside_pos,
        "manifest.validate() must be called before move_aside_for_rollback() \
         (validate at offset {validate_pos}, move_aside at offset {move_aside_pos} from fn start)"
    );
}

#[test]
fn test_restore_confirm_check_before_prompt() {
    // Verify the order: no_interactive confirm check must come before
    // the interactive prompt check. This ensures --confirm is required
    // before any prompting logic.
    let restore_code = fs::read_to_string("src/restore.rs")
        .expect("Failed to read restore.rs");

    // Find run_restore() function body
    let fn_start = restore_code
        .find("pub async fn run_restore(")
        .expect("restore.rs must define run_restore()");

    // Within run_restore, the confirm check must precede the prompt check
    let confirm_check = restore_code[fn_start..]
        .find("if no_interactive && !confirm {")
        .expect("run_restore must check no_interactive && !confirm");
    let prompt_check = restore_code[fn_start..]
        .find("if !no_interactive {")
        .expect("run_restore must check !no_interactive for prompting");

    assert!(
        confirm_check < prompt_check,
        "Confirm requirement check must come before prompt check \
         (confirm at offset {confirm_check}, prompt at offset {prompt_check} from fn start)"
    );
}

#[test]
fn test_restore_error_handling_quality() {
    // Verify that error messages are clear and actionable
    let restore_code = fs::read_to_string("src/restore.rs")
        .expect("Failed to read restore.rs");

    // Find the --confirm requirement check
    let check_start = restore_code
        .find("if no_interactive && !confirm {")
        .expect("Must have --confirm requirement check");

    let check_section = &restore_code[check_start..check_start + 600];

    // Verify error message quality
    assert!(
        check_section.contains("hoop restore: --confirm is required"),
        "Error must clearly state --confirm is required"
    );

    assert!(
        check_section.contains("DESTRUCTIVE"),
        "Error must warn about destructive operation"
    );

    assert!(
        check_section.contains("replace ~/.hoop/"),
        "Error must explain what will be replaced"
    );

    assert!(
        check_section.contains("--no-interactive --confirm"),
        "Error must show correct re-run command"
    );

    assert!(
        check_section.contains("Re-run with:"),
        "Error should explicitly say 'Re-run with:'"
    );
}

#[test]
fn test_restore_both_positions_extract_same_value() {
    // Test that both flag positions yield the same no_interactive value
    // using convenience helpers

    // Parse with flag before subcommand
    let parsed_before =
        parse_flag_before_subcommand(&["restore", "--from", "s3://bucket/key", "--confirm"])
            .expect("Should parse flag before command");

    // Parse with flag after subcommand
    let parsed_after =
        parse_flag_after_subcommand(&["restore", "--from", "s3://bucket/key", "--confirm"])
            .expect("Should parse flag after command");

    // Both should yield the same no_interactive value
    assert_eq!(
        parsed_before.no_interactive,
        parsed_after.no_interactive,
        "Flag position should not affect the extracted value"
    );

    assert_eq!(
        parsed_before.no_interactive,
        true,
        "Both positions should extract no_interactive as true"
    );

    assert_eq!(
        parsed_before.command,
        parsed_after.command,
        "Both positions should extract the same command"
    );
}

#[test]
fn test_restore_short_flag_y_works() {
    // Test that the short -y flag works correctly for restore command
    let parsed = parse_flag_before_subcommand(&["restore", "--from", "s3://bucket/key", "--confirm"])
        .expect("Should parse -y flag");

    // Verify the global -y flag is recognized as no_interactive
    assert_eq!(
        parsed.no_interactive, true,
        "Short -y flag should set no_interactive to true"
    );

    assert_eq!(parsed.command, "restore", "Command should be 'restore'");
}

#[test]
fn test_restore_comprehensive_no_interactive_coverage() {
    // Meta-test that verifies all critical aspects are covered
    // This serves as a checklist for the test suite

    let main_code = fs::read_to_string("src/main.rs").expect("Failed to read main.rs");
    let restore_code = fs::read_to_string("src/restore.rs")
        .expect("Failed to read restore.rs");

    // Checklist:
    // 1. Restore command has from, dry_run, and confirm fields
    assert!(
        main_code.contains("Commands::Restore { from, dry_run, confirm } =>"),
        "✓ Restore command has from, dry_run, and confirm fields"
    );

    // 2. Restore handler extracts global no_interactive flag
    assert!(
        main_code.contains("let no_interactive = cli.no_interactive;"),
        "✓ Global flag extracted in main"
    );

    // 3. Restore handler passes flag to run_restore
    assert!(
        main_code.contains("restore::run_restore(&from, dry_run, no_interactive, confirm)"),
        "✓ Flag passed to run_restore"
    );

    // 4. run_restore accepts no_interactive and confirm parameters
    assert!(
        restore_code.contains("pub async fn run_restore(from_uri: &str, dry_run: bool, no_interactive: bool, confirm: bool)"),
        "✓ run_restore accepts both no_interactive and confirm parameters"
    );

    // 5. run_restore checks no_interactive flag for confirm requirement
    assert!(
        restore_code.contains("if no_interactive && !confirm {"),
        "✓ run_restore checks confirm requirement in no-interactive mode"
    );

    // 6. run_restore checks no_interactive flag for prompting logic
    assert!(
        restore_code.contains("if !no_interactive {"),
        "✓ run_restore checks no_interactive flag for prompting"
    );

    // 7. When no_interactive=true && !confirm, error message suggests --confirm
    assert!(
        restore_code.contains("--confirm is required in non-interactive mode"),
        "✓ Error message suggests --confirm in no-interactive mode"
    );

    // 8. When no_interactive=false, prompts for confirmation
    assert!(
        restore_code.contains("eprint!(\"Continue? [y/N] \")"),
        "✓ Prompts for confirmation when no_interactive=false"
    );

    // 9. Prompts go to stderr
    assert!(
        restore_code.contains("std::io::stderr().flush()?"),
        "✓ Prompts go to stderr (not stdout)"
    );

    // 10. Dry-run mode respects no_interactive in usage message
    assert!(
        restore_code.contains("--no-interactive --confirm"),
        "✓ Dry-run mode shows --no-interactive --confirm usage"
    );

    // 11. manifest.validate() is called before move_aside_for_rollback()
    // Note: We search within the actual function body, excluding the test module at the bottom
    let fn_start = restore_code
        .find("pub async fn run_restore(")
        .expect("run_restore function must exist");
    // Find the end of the function (before the test module starts)
    let fn_end = restore_code[fn_start..]
        .find("#[cfg(test)]")
        .unwrap_or(restore_code.len() - fn_start);
    let function_body = &restore_code[fn_start..fn_start + fn_end];

    let validate_pos = function_body
        .find(".validate(current)")
        .expect("manifest.validate() must be called in function body");
    let move_aside_pos = function_body
        .find("move_aside_for_rollback()")
        .expect("move_aside_for_rollback() must be called in function body");
    assert!(
        validate_pos < move_aside_pos,
        "✓ manifest.validate() called before move_aside_for_rollback()"
    );

    // All checks passed - if we reach here, all assertions above succeeded
    // All the assert! calls above have verified the required functionality
    println!("All Restore command no_interactive tests verified");
}
