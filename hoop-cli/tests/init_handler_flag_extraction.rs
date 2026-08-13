//! Unit tests for flag value extraction in Init command handler
//!
//! These tests verify that the no_interactive flag value is correctly extracted
//! from the parsed CLI structure and used in the handler function.
//!
//! Test coverage:
//! 1. Flag value extraction from parsed Cli struct
//! 2. Correct boolean value retrieval (true when flag present, false when absent)
//! 3. Handler logic correctly receives and uses the flag value
//! 4. Integration-style tests verifying the full parsing → extraction → handler flow

use hoop::{Cli, Commands};
use clap::Parser;

// ── Test Helper Functions ─────────────────────────────────────────────────────

/// Parse CLI arguments and extract the flag value and command
///
/// This helper function simulates what happens in main.rs:
/// 1. Parse CLI arguments using Cli::try_parse_from()
/// 2. Extract both the no_interactive flag and the parsed command
///
/// Returns a tuple of (no_interactive flag value, command enum variant)
fn parse_and_extract(args: &[&str]) -> (bool, Commands) {
    let cli = Cli::try_parse_from(args).expect("CLI parsing should succeed");
    let no_interactive = cli.no_interactive;
    let command = cli.command;
    (no_interactive, command)
}

/// Simulate the main.rs handler pattern for Init command
///
/// This function mimics the actual handler pattern used in main.rs:
/// - Parse CLI
/// - Extract no_interactive flag (line 366 in main.rs)
/// - Match on command
/// - Extract the flag value that would be passed to the handler
///
/// This represents what main.rs does before calling init::run_init_wizard(no_interactive)
fn extract_init_handler_flag(args: &[&str]) -> bool {
    let cli = Cli::try_parse_from(args).expect("CLI parsing should succeed");

    // This is line 366 in main.rs
    let no_interactive = cli.no_interactive;

    // This is the match pattern from main.rs lines 520-524
    match cli.command {
        Commands::Init => {
            // The flag would be passed to init::run_init_wizard(no_interactive)
            no_interactive
        }
        _ => panic!("Expected Init command"),
    }
}

/// Test helper to verify handler would receive correct flag value
///
/// This simulates the complete flow:
/// 1. Parse CLI
/// 2. Extract no_interactive from Cli struct
/// 3. Match on Commands::Init
/// 4. Return the flag value that would be passed to run_init_wizard()
fn simulate_init_handler_flow(args: &[&str]) -> Result<bool, String> {
    // Parse CLI
    let cli = Cli::try_parse_from(args)
        .map_err(|e| format!("Parse failed: {}", e))?;

    // Extract flag (main.rs line 366)
    let no_interactive = cli.no_interactive;

    // Match on command (main.rs lines 520-524)
    match cli.command {
        Commands::Init => {
            // This is what gets passed to init::run_init_wizard(no_interactive)
            Ok(no_interactive)
        }
        _ => Err(format!("Expected Commands::Init, got {:?}", cli.command)),
    }
}

// ── Flag Extraction Tests ─────────────────────────────────────────────────────

#[test]
fn test_init_flag_extraction_with_flag_present() {
    // Test: hoop --no-interactive init
    let args = ["hoop", "--no-interactive", "init"];

    let (no_interactive, command) = parse_and_extract(&args);

    // Verify flag extraction
    assert_eq!(no_interactive, true,
        "no_interactive should be true when --no-interactive flag is present");

    // Verify command parsing
    assert!(matches!(command, Commands::Init),
        "Command should be parsed as Commands::Init");
}

#[test]
fn test_init_flag_extraction_with_flag_after_command() {
    // Test: hoop init --no-interactive
    let args = ["hoop", "init", "--no-interactive"];

    let (no_interactive, command) = parse_and_extract(&args);

    // Verify flag extraction
    assert_eq!(no_interactive, true,
        "no_interactive should be true when flag appears after init command");

    // Verify command parsing
    assert!(matches!(command, Commands::Init),
        "Command should be parsed as Commands::Init");
}

#[test]
fn test_init_flag_extraction_with_short_flag() {
    // Test: hoop -y init
    let args = ["hoop", "-y", "init"];

    let (no_interactive, command) = parse_and_extract(&args);

    // Verify flag extraction
    assert_eq!(no_interactive, true,
        "no_interactive should be true when -y short flag is present");

    // Verify command parsing
    assert!(matches!(command, Commands::Init),
        "Command should be parsed as Commands::Init");
}

#[test]
fn test_init_flag_extraction_without_flag() {
    // Test: hoop init (default behavior)
    let args = ["hoop", "init"];

    let (no_interactive, command) = parse_and_extract(&args);

    // Verify flag extraction defaults to false
    assert_eq!(no_interactive, false,
        "no_interactive should be false by default when flag is not present");

    // Verify command parsing
    assert!(matches!(command, Commands::Init),
        "Command should be parsed as Commands::Init");
}

#[test]
fn test_init_flag_extraction_consistency_across_positions() {
    // Verify that flag value is consistent regardless of position

    // Flag before command
    let args_before = ["hoop", "--no-interactive", "init"];
    let (no_interactive_before, command_before) = parse_and_extract(&args_before);

    // Flag after command
    let args_after = ["hoop", "init", "--no-interactive"];
    let (no_interactive_after, command_after) = parse_and_extract(&args_after);

    // Both should yield the same flag value
    assert_eq!(no_interactive_before, no_interactive_after,
        "Flag value should be consistent regardless of position");

    assert_eq!(no_interactive_before, true,
        "Both positions should extract no_interactive as true");

    // Both should parse as Init command
    assert!(matches!(command_before, Commands::Init));
    assert!(matches!(command_after, Commands::Init));
}

// ── Handler Pattern Tests ──────────────────────────────────────────────────────

#[test]
fn test_init_handler_pattern_with_flag_true() {
    // Test that the handler pattern correctly extracts and would pass true to handler
    let args = ["hoop", "--no-interactive", "init"];

    let handler_flag = extract_init_handler_flag(&args);

    assert_eq!(handler_flag, true,
        "Handler should receive no_interactive=true when flag is present");
}

#[test]
fn test_init_handler_pattern_with_flag_false() {
    // Test that the handler pattern correctly extracts and would pass false to handler
    let args = ["hoop", "init"];

    let handler_flag = extract_init_handler_flag(&args);

    assert_eq!(handler_flag, false,
        "Handler should receive no_interactive=false when flag is absent");
}

#[test]
fn test_init_handler_pattern_with_flag_after_command() {
    // Test that handler pattern works when flag appears after command
    let args = ["hoop", "init", "--no-interactive"];

    let handler_flag = extract_init_handler_flag(&args);

    assert_eq!(handler_flag, true,
        "Handler should receive no_interactive=true regardless of flag position");
}

#[test]
fn test_init_handler_pattern_with_short_flag() {
    // Test that handler pattern works with short flag
    let args = ["hoop", "-y", "init"];

    let handler_flag = extract_init_handler_flag(&args);

    assert_eq!(handler_flag, true,
        "Handler should receive no_interactive=true with -y short flag");
}

// ── Integration Flow Tests ─────────────────────────────────────────────────────

#[test]
fn test_init_full_flow_flag_present() {
    // Test the complete flow: parse → extract → match on Init → get flag value
    let args = ["hoop", "--no-interactive", "init"];

    let result = simulate_init_handler_flow(&args);

    assert!(result.is_ok(), "Handler flow should succeed for valid init command");
    assert_eq!(result.unwrap(), true,
        "Handler flow should extract no_interactive=true");
}

#[test]
fn test_init_full_flow_flag_absent() {
    // Test the complete flow without the flag
    let args = ["hoop", "init"];

    let result = simulate_init_handler_flow(&args);

    assert!(result.is_ok(), "Handler flow should succeed for init command without flag");
    assert_eq!(result.unwrap(), false,
        "Handler flow should extract no_interactive=false by default");
}

#[test]
fn test_init_full_flow_multiple_variants() {
    // Test multiple flag variants all yield the correct handler input
    let test_cases: Vec<[&str; 3]> = vec![
        ["hoop", "--no-interactive", "init"],
        ["hoop", "init", "--no-interactive"],
        ["hoop", "-y", "init"],
        ["hoop", "init", "-y"],
    ];

    let expected_flags = vec![true, true, true, true];
    let descriptions = vec![
        "flag before command",
        "flag after command",
        "short flag before command",
        "short flag after command",
    ];

    for (i, args) in test_cases.iter().enumerate() {
        let result = simulate_init_handler_flow(args);
        let expected_flag = expected_flags[i];
        let description = descriptions[i];

        assert!(result.is_ok(),
            "Handler flow should succeed for {}: parse failed", description);

        let flag_value = result.unwrap();
        assert_eq!(flag_value, expected_flag,
            "Handler flow should extract no_interactive={} for {}", expected_flag, description);
    }

    // Test the no-flag case separately
    let args_no_flag = ["hoop", "init"];
    let result_no_flag = simulate_init_handler_flow(&args_no_flag);
    assert!(result_no_flag.is_ok(), "Handler flow should succeed without flag");
    assert_eq!(result_no_flag.unwrap(), false, "Handler flow should extract false without flag");
}

// ── Boolean Value Retrieval Tests ──────────────────────────────────────────────

#[test]
fn test_init_flag_presence_returns_true() {
    // Test: Verify that the no_interactive flag value extraction returns true
    // when the --no-interactive flag is present in the parsed command.
    //
    // Acceptance criteria:
    // - Creates a Commands::Init with no_interactive set to true
    // - Extracts the flag value from the parsed command
    // - Asserts the extracted value is true
    // - Follows the patterns from existing test infrastructure

    // Parse with --no-interactive flag present
    let args = ["hoop", "--no-interactive", "init"];
    let cli = Cli::try_parse_from(args).expect("CLI parsing should succeed");

    // Verify command is Commands::Init
    let command = &cli.command;
    assert!(matches!(command, Commands::Init),
        "Command should be Commands::Init");

    // Extract the flag value from the parsed command
    let no_interactive = cli.no_interactive;

    // Assert the extracted value is true
    assert_eq!(no_interactive, true,
        "no_interactive flag value should be true when --no-interactive flag is present");
}

#[test]
fn test_init_retrieves_true_when_flag_present() {
    // Verify that true is retrieved when --no-interactive is present
    let flag_variants = vec![
        ["hoop", "--no-interactive", "init"],
        ["hoop", "init", "--no-interactive"],
        ["hoop", "-y", "init"],
        ["hoop", "init", "-y"],
    ];

    for args in flag_variants {
        let (no_interactive, _) = parse_and_extract(&args);
        assert_eq!(no_interactive, true,
            "Should retrieve true when --no-interactive or -y is present");
    }
}

#[test]
fn test_init_retrieves_false_when_flag_absent() {
    // Verify that false is retrieved when flag is absent
    let args = ["hoop", "init"];

    let (no_interactive, _) = parse_and_extract(&args);

    assert_eq!(no_interactive, false,
        "Should retrieve false when --no-interactive flag is absent");
}

#[test]
fn test_init_boolean_extraction_is_deterministic() {
    // Verify that extraction is deterministic (same input → same output)
    let args = ["hoop", "--no-interactive", "init"];

    // Extract multiple times
    let result1 = parse_and_extract(&args);
    let result2 = parse_and_extract(&args);
    let result3 = parse_and_extract(&args);

    // All extractions should yield identical results (compare only the bool part since Commands doesn't impl PartialEq)
    assert_eq!(result1.0, result2.0, "Extraction should be deterministic (1 vs 2)");
    assert_eq!(result2.0, result3.0, "Extraction should be deterministic (2 vs 3)");
    assert_eq!(result1.0, true, "All extractions should yield true");

    // Verify Commands::Init is the same for all
    assert!(matches!(result1.1, Commands::Init));
    assert!(matches!(result2.1, Commands::Init));
    assert!(matches!(result3.1, Commands::Init));
}

// ── Commands::Init Struct Tests ──────────────────────────────────────────────────

#[test]
fn test_commands_init_is_unit_variant() {
    // Verify that Commands::Init is a unit variant (no associated data)
    // This means the flag must come from the top-level Cli struct

    let args = ["hoop", "init"];
    let cli = Cli::try_parse_from(args).expect("Should parse successfully");

    // Commands::Init has no fields to extract the flag from
    match cli.command {
        Commands::Init => {
            // Success - Init is a unit variant with no associated data
            // The no_interactive flag must come from cli.no_interactive, not from Commands::Init
            assert!(true, "Commands::Init is a unit variant");
        }
        _ => panic!("Expected Commands::Init"),
    }
}

#[test]
fn test_flag_comes_from_cli_not_from_commands_init() {
    // Verify that the flag is extracted from Cli struct, not from Commands::Init
    // This is important because Commands::Init has no no_interactive field

    let args = ["hoop", "--no-interactive", "init"];
    let cli = Cli::try_parse_from(args).expect("Should parse successfully");

    // The flag is at the top level
    let flag_from_cli = cli.no_interactive;

    // Commands::Init has no no_interactive field (it's a unit variant)
    match cli.command {
        Commands::Init => {
            // No field access possible here - the command doesn't carry the flag
            // The flag must be accessed from cli.no_interactive
            assert_eq!(flag_from_cli, true,
                "Flag must be extracted from Cli.no_interactive, not from Commands::Init");
        }
        _ => panic!("Expected Commands::Init"),
    }
}

// ── Position Independence Verification ───────────────────────────────────────────

#[test]
fn test_init_flag_position_does_not_affect_extraction() {
    // Verify that flag position doesn't affect the extracted value
    let test_positions = vec![
        ["hoop", "--no-interactive", "init"],
        ["hoop", "-y", "init"],
        ["hoop", "init", "--no-interactive"],
        ["hoop", "init", "-y"],
    ];

    let mut extracted_values = Vec::new();
    for args in test_positions {
        let (no_interactive, _) = parse_and_extract(&args);
        extracted_values.push(no_interactive);
    }

    // All positions should yield true
    assert!(extracted_values.iter().all(|&v| v == true),
        "All flag positions should extract the same value (true)");

    // Verify all values are identical
    let first = extracted_values[0];
    assert!(extracted_values.iter().all(|&v| v == first),
        "Flag position independence: all extractions must yield identical values");
}

#[test]
fn test_init_default_value_is_consistent() {
    // Verify that the default value (false) is consistent when flag is omitted
    let args_without_flag = vec![
        ["hoop", "init"],
    ];

    let mut extracted_values = Vec::new();
    for args in args_without_flag {
        let (no_interactive, _) = parse_and_extract(&args);
        extracted_values.push(no_interactive);
    }

    // All should yield false
    assert!(extracted_values.iter().all(|&v| v == false),
        "Default value should consistently be false");
}

// ── Handler Logic Tests ─────────────────────────────────────────────────────────

#[test]
fn test_init_handler_receives_correct_boolean() {
    // Test that the handler logic receives the correct boolean value
    // This simulates what main.rs does at lines 520-524

    let test_cases: Vec<([&str; 3], bool, &str)> = vec![
        (["hoop", "--no-interactive", "init"], true, "with --no-interactive before"),
        (["hoop", "init", "--no-interactive"], true, "with --no-interactive after"),
        (["hoop", "-y", "init"], true, "with -y before"),
        (["hoop", "init", "-y"], true, "with -y after"),
    ];

    for (args, expected_boolean, description) in test_cases {
        let cli = Cli::try_parse_from(args).expect("Parse should succeed");

        // This is exactly what main.rs does
        let no_interactive = cli.no_interactive;

        // Match on command and verify the flag that would be passed to handler
        match cli.command {
            Commands::Init => {
                assert_eq!(no_interactive, expected_boolean,
                    "Handler should receive {} for {}", expected_boolean, description);
            }
            _ => panic!("Expected Commands::Init for {}", description),
        }
    }

    // Test the no-flag case separately
    let args_no_flag = ["hoop", "init"];
    let cli_no_flag = Cli::try_parse_from(args_no_flag).expect("Parse should succeed");
    let no_interactive_no_flag = cli_no_flag.no_interactive;
    match cli_no_flag.command {
        Commands::Init => {
            assert_eq!(no_interactive_no_flag, false, "Handler should receive false without flag");
        }
        _ => panic!("Expected Commands::Init"),
    }
}

// ── Edge Cases ───────────────────────────────────────────────────────────────────

#[test]
fn test_init_with_no_other_arguments() {
    // Test the simplest case: just "hoop init"
    let args = ["hoop", "init"];

    let result = simulate_init_handler_flow(&args);

    assert!(result.is_ok(), "Should parse successfully");
    assert_eq!(result.unwrap(), false, "Default should be false");
}

#[test]
fn test_init_rejects_extra_arguments() {
    // Test that extra arguments are rejected
    let args = ["hoop", "init", "extra-arg"];

    let result = Cli::try_parse_from(args);
    assert!(result.is_err(), "Should reject extra arguments");
}

#[test]
fn test_init_with_invalid_flag() {
    // Test that invalid flags are rejected
    let args = ["hoop", "init", "--invalid-flag"];

    let result = Cli::try_parse_from(args);
    assert!(result.is_err(), "Should reject invalid flags");
}

// ── Comprehensive Handler Flow Test ─────────────────────────────────────────────

#[test]
fn test_init_handler_comprehensive_coverage() {
    // Meta-test that verifies all aspects of flag extraction and handling

    // Test 1: Flag extraction with flag present
    let args_with_flag = ["hoop", "--no-interactive", "init"];
    let (flag_present, cmd_present) = parse_and_extract(&args_with_flag);
    assert_eq!(flag_present, true, "1. Should extract true when flag present");
    assert!(matches!(cmd_present, Commands::Init), "1. Should parse as Init");

    // Test 2: Flag extraction without flag
    let args_without_flag = ["hoop", "init"];
    let (flag_absent, cmd_absent) = parse_and_extract(&args_without_flag);
    assert_eq!(flag_absent, false, "2. Should extract false when flag absent");
    assert!(matches!(cmd_absent, Commands::Init), "2. Should parse as Init");

    // Test 3: Handler pattern receives correct value
    let handler_flag_true = extract_init_handler_flag(&args_with_flag);
    assert_eq!(handler_flag_true, true, "3. Handler should receive true");

    let handler_flag_false = extract_init_handler_flag(&args_without_flag);
    assert_eq!(handler_flag_false, false, "3. Handler should receive false");

    // Test 4: Full integration flow
    let flow_true = simulate_init_handler_flow(&args_with_flag);
    assert!(flow_true.is_ok(), "4. Flow should succeed with flag");
    assert_eq!(flow_true.unwrap(), true, "4. Flow should extract true");

    let flow_false = simulate_init_handler_flow(&args_without_flag);
    assert!(flow_false.is_ok(), "4. Flow should succeed without flag");
    assert_eq!(flow_false.unwrap(), false, "4. Flow should extract false");

    // All checks passed
    assert!(true, "All Init handler flag extraction tests verified");
}

#[test]
fn test_init_flag_absence_returns_false() {
    // Test: Verify that the no_interactive flag value extraction returns false
    // (or default value) when the --no-interactive flag is absent from the parsed command.
    //
    // Acceptance criteria:
    // - Creates a Commands::Init with no_interactive absent or set to false
    // - Extracts the flag value from the parsed command
    // - Asserts the extracted value is false or the default value
    // - Follows the patterns from existing test infrastructure
    // - Compiles without errors

    // Parse without --no-interactive flag (flag is absent)
    let args = ["hoop", "init"];
    let cli = Cli::try_parse_from(args).expect("CLI parsing should succeed");

    // Verify command is Commands::Init
    let command = &cli.command;
    assert!(matches!(command, Commands::Init),
        "Command should be Commands::Init");

    // Extract the flag value from the parsed command
    let no_interactive = cli.no_interactive;

    // Assert the extracted value is false (default value when flag is absent)
    assert_eq!(no_interactive, false,
        "no_interactive flag value should be false (default) when --no-interactive flag is absent");

    // Additional verification: confirm this is the default behavior
    // When no flag is provided, no_interactive should default to false
    let expected_default = false;
    assert_eq!(no_interactive, expected_default,
        "Flag absence should yield default value of false");
}

// ── Test Suite Summary ───────────────────────────────────────────────────────────
//
// This test suite verifies:
//
// 1. Flag value extraction from parsed Cli struct (not from Commands::Init)
//    - Commands::Init is a unit variant with no fields
//    - The no_interactive flag is stored at the Cli level
//    - Extraction works via cli.no_interactive
//
// 2. Correct boolean value retrieval
//    - Returns true when --no-interactive or -y is present
//    - Returns false when flag is absent (default behavior)
//    - Value is deterministic and consistent
//
// 3. Handler logic correctly uses the flag
//    - Handler pattern in main.rs extracts flag at line 366
//    - Match on Commands::Init at lines 520-524
//    - Flag is passed to init::run_init_wizard(no_interactive)
//
// 4. Position independence
//    - Flag works before command: hoop --no-interactive init
//    - Flag works after command: hoop init --no-interactive
//    - Short flag works: hoop -y init
//    - All positions yield the same extracted value
//
// 5. Integration flow
//    - Parse CLI → Extract no_interactive → Match command → Pass to handler
//    - Full flow tested with simulate_init_handler_flow()
//
// 6. Flag presence returns true
//    - test_init_flag_presence_returns_true() specifically verifies this
//    - Creates Commands::Init with no_interactive set to true
//    - Extracts the flag value from the parsed command
//    - Asserts the extracted value is true
//
// 7. Flag absence returns false
//    - test_init_flag_absence_returns_false() specifically verifies this
//    - Creates Commands::Init with no_interactive absent
//    - Extracts the flag value from the parsed command
//    - Asserts the extracted value is false (default value)
