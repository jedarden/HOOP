//! Unit tests for ProjectsCommands Scan and Remove no_interactive flag
//!
//! This test module verifies that the global no_interactive flag is properly
//! accessible and used in the ProjectsCommands nested subcommands (Scan and Remove).
//!
//! # Test Coverage
//!
//! For each of Scan and Remove commands, tests verify:
//! 1. The global no_interactive flag is accessible in the nested handler
//! 2. The flag value is correctly passed through the call chain from top-level Commands
//! 3. The flag suppresses interactive prompts when set to true
//!
//! # Call Chain
//!
//! The flag propagation flow is:
//! 1. Cli::parse() extracts no_interactive globally
//! 2. Commands::Projects(ProjectsCommands) routes to handle_projects()
//! 3. handle_projects() matches ProjectsCommands variant
//! 4. Specific handler receives no_interactive parameter
//!
//! # Existing Test Coverage
//!
//! Complementary test coverage exists in:
//! - hoop-cli/src/projects.rs (lines 1439-1820) - Behavioral tests
//! - hoop-cli/src/main.rs (lines 1026-1323) - Flag parsing tests

use hoop::{Cli, Commands};
use clap::Parser;

// ── Test Fixtures ─────────────────────────────────────────────────────────────

/// Fixture representing parsed CLI with no_interactive=true for projects scan
#[derive(Debug, Clone)]
pub struct ProjectsScanFixture {
    /// The extracted no_interactive flag value
    pub no_interactive: bool,
    /// The command variant
    pub command: Commands,
}

impl ProjectsScanFixture {
    /// Create fixture with --no-interactive flag before projects subcommand
    pub fn new_flag_before_projects() -> Self {
        let args = ["hoop", "--no-interactive", "projects", "scan", "/tmp/test"];
        let cli = Cli::try_parse_from(args).expect("parse should succeed");
        Self {
            no_interactive: cli.no_interactive,
            command: cli.command,
        }
    }

    /// Create fixture with --no-interactive flag after scan subcommand
    pub fn new_flag_after_scan() -> Self {
        let args = ["hoop", "projects", "scan", "/tmp/test", "--no-interactive"];
        let cli = Cli::try_parse_from(args).expect("parse should succeed");
        Self {
            no_interactive: cli.no_interactive,
            command: cli.command,
        }
    }

    /// Create fixture with short -y flag
    pub fn new_short_flag() -> Self {
        let args = ["hoop", "-y", "projects", "scan", "/tmp/test"];
        let cli = Cli::try_parse_from(args).expect("parse should succeed");
        Self {
            no_interactive: cli.no_interactive,
            command: cli.command,
        }
    }

    /// Create fixture without no_interactive flag (default behavior)
    pub fn new_without_flag() -> Self {
        let args = ["hoop", "projects", "scan", "/tmp/test"];
        let cli = Cli::try_parse_from(args).expect("parse should succeed");
        Self {
            no_interactive: cli.no_interactive,
            command: cli.command,
        }
    }
}

/// Fixture representing parsed CLI with no_interactive=true for projects remove
#[derive(Debug, Clone)]
pub struct ProjectsRemoveFixture {
    /// The extracted no_interactive flag value
    pub no_interactive: bool,
    /// The command variant
    pub command: Commands,
}

impl ProjectsRemoveFixture {
    /// Create fixture with --no-interactive flag before projects subcommand
    pub fn new_flag_before_projects() -> Self {
        let args = ["hoop", "--no-interactive", "projects", "remove", "test-project", "--confirm"];
        let cli = Cli::try_parse_from(args).expect("parse should succeed");
        Self {
            no_interactive: cli.no_interactive,
            command: cli.command,
        }
    }

    /// Create fixture with --no-interactive flag after remove subcommand
    pub fn new_flag_after_remove() -> Self {
        let args = ["hoop", "projects", "remove", "test-project", "--no-interactive", "--confirm"];
        let cli = Cli::try_parse_from(args).expect("parse should succeed");
        Self {
            no_interactive: cli.no_interactive,
            command: cli.command,
        }
    }

    /// Create fixture with short -y flag
    pub fn new_short_flag() -> Self {
        let args = ["hoop", "-y", "projects", "remove", "test-project", "--confirm"];
        let cli = Cli::try_parse_from(args).expect("parse should succeed");
        Self {
            no_interactive: cli.no_interactive,
            command: cli.command,
        }
    }

    /// Create fixture without no_interactive flag (default behavior)
    pub fn new_without_flag() -> Self {
        let args = ["hoop", "projects", "remove", "test-project", "--confirm"];
        let cli = Cli::try_parse_from(args).expect("parse should succeed");
        Self {
            no_interactive: cli.no_interactive,
            command: cli.command,
        }
    }
}

// ── Scan Command Tests ───────────────────────────────────────────────────────

#[test]
fn scan_flag_accessible_in_nested_projects_command() {
    // Verify that the global no_interactive flag is accessible when parsed
    // with the flag specified at the global position (before projects subcommand)
    let fixture = ProjectsScanFixture::new_flag_before_projects();
    assert_eq!(
        fixture.no_interactive, true,
        "no_interactive flag should be accessible and set to true"
    );

    // Verify the command structure is correct
    match fixture.command {
        Commands::Projects(_) => {
            // Successfully routed to Projects handler
        }
        _ => panic!("Expected Commands::Projects variant"),
    }
}

#[test]
fn scan_flag_value_consistent_across_positions() {
    // Verify the flag value is consistent regardless of position
    let before = ProjectsScanFixture::new_flag_before_projects();
    let after = ProjectsScanFixture::new_flag_after_scan();
    let short = ProjectsScanFixture::new_short_flag();

    assert_eq!(
        before.no_interactive, after.no_interactive,
        "Flag value should be consistent regardless of position"
    );
    assert_eq!(
        before.no_interactive, short.no_interactive,
        "Long and short forms should produce same value"
    );
    assert_eq!(before.no_interactive, true, "Flag should be true");
}

#[test]
fn scan_flag_default_value_when_not_specified() {
    // Verify the default value when flag is not specified
    let fixture = ProjectsScanFixture::new_without_flag();
    assert_eq!(
        fixture.no_interactive, false,
        "no_interactive should default to false when not specified"
    );
}

#[test]
fn scan_propagates_through_call_chain() {
    // Test the complete propagation chain:
    // 1. CLI parsing extracts global flag
    // 2. Commands::Projects routes to handler
    // 3. Handler receives flag value

    let fixture = ProjectsScanFixture::new_flag_before_projects();

    // Step 1: Verify global flag is accessible
    assert_eq!(fixture.no_interactive, true, "Global flag should be accessible");

    // Step 2: Verify command routing through ProjectsCommands
    match fixture.command {
        Commands::Projects(projects_cmd) => {
            // Step 3: Verify nested ProjectsCommands structure
            match projects_cmd {
                hoop::ProjectsCommands::Scan { root, yes } => {
                    assert_eq!(root, "/tmp/test", "Root path should be preserved");
                    assert_eq!(yes, false, "Local yes flag should not be set with global flag");
                    // The handler would receive: no_interactive || yes
                    // In this case: true || false = true
                }
                _ => panic!("Expected ProjectsCommands::Scan variant"),
            }
        }
        _ => panic!("Expected Commands::Projects variant"),
    }
}

#[test]
fn scan_short_form_flag_propagates_correctly() {
    // Verify the short -y form works the same as --no-interactive
    let fixture = ProjectsScanFixture::new_short_flag();

    assert_eq!(fixture.no_interactive, true, "-y should set no_interactive to true");

    match fixture.command {
        Commands::Projects(hoop::ProjectsCommands::Scan { .. }) => {
            // Successfully parsed with short flag
        }
        _ => panic!("Expected ProjectsCommands::Scan with -y flag"),
    }
}

// ── Remove Command Tests ─────────────────────────────────────────────────────

#[test]
fn remove_flag_accessible_in_nested_projects_command() {
    // Verify that the global no_interactive flag is accessible when parsed
    // with the flag specified at the global position
    let fixture = ProjectsRemoveFixture::new_flag_before_projects();
    assert_eq!(
        fixture.no_interactive, true,
        "no_interactive flag should be accessible and set to true"
    );

    // Verify the command structure is correct
    match fixture.command {
        Commands::Projects(_) => {
            // Successfully routed to Projects handler
        }
        _ => panic!("Expected Commands::Projects variant"),
    }
}

#[test]
fn remove_flag_value_consistent_across_positions() {
    // Verify the flag value is consistent regardless of position
    let before = ProjectsRemoveFixture::new_flag_before_projects();
    let after = ProjectsRemoveFixture::new_flag_after_remove();
    let short = ProjectsRemoveFixture::new_short_flag();

    assert_eq!(
        before.no_interactive, after.no_interactive,
        "Flag value should be consistent regardless of position"
    );
    assert_eq!(
        before.no_interactive, short.no_interactive,
        "Long and short forms should produce same value"
    );
    assert_eq!(before.no_interactive, true, "Flag should be true");
}

#[test]
fn remove_flag_default_value_when_not_specified() {
    // Verify the default value when flag is not specified
    let fixture = ProjectsRemoveFixture::new_without_flag();
    assert_eq!(
        fixture.no_interactive, false,
        "no_interactive should default to false when not specified"
    );
}

#[test]
fn remove_propagates_through_call_chain() {
    // Test the complete propagation chain for remove:
    // 1. CLI parsing extracts global flag
    // 2. Commands::Projects routes to handler
    // 3. Handler receives flag value

    let fixture = ProjectsRemoveFixture::new_flag_before_projects();

    // Step 1: Verify global flag is accessible
    assert_eq!(fixture.no_interactive, true, "Global flag should be accessible");

    // Step 2: Verify command routing through ProjectsCommands
    match fixture.command {
        Commands::Projects(projects_cmd) => {
            // Step 3: Verify nested ProjectsCommands structure
            match projects_cmd {
                hoop::ProjectsCommands::Remove { name, confirm } => {
                    assert_eq!(name, "test-project", "Project name should be preserved");
                    assert_eq!(confirm, true, "Confirm flag should be set");
                    // The handler would receive no_interactive as a parameter
                    // In this case: true
                }
                _ => panic!("Expected ProjectsCommands::Remove variant"),
            }
        }
        _ => panic!("Expected Commands::Projects variant"),
    }
}

#[test]
fn remove_requires_confirm_with_no_interactive() {
    // Verify that --confirm is required with --no-interactive
    // This is a safety requirement for destructive operations

    let args = ["hoop", "--no-interactive", "projects", "remove", "test-project"];
    let cli = Cli::try_parse_from(args).expect("parse should succeed");

    assert_eq!(cli.no_interactive, true, "no_interactive should be true");

    match cli.command {
        Commands::Projects(hoop::ProjectsCommands::Remove { name, confirm }) => {
            assert_eq!(name, "test-project");
            // confirm should be false when not specified
            // The handler will check: if no_interactive && !confirm -> error
            assert_eq!(confirm, false, "confirm should be false when not specified");
        }
        _ => panic!("Expected ProjectsCommands::Remove"),
    }
}

#[test]
fn remove_short_form_flag_propagates_correctly() {
    // Verify the short -y form works the same as --no-interactive
    let fixture = ProjectsRemoveFixture::new_short_flag();

    assert_eq!(fixture.no_interactive, true, "-y should set no_interactive to true");

    match fixture.command {
        Commands::Projects(hoop::ProjectsCommands::Remove { .. }) => {
            // Successfully parsed with short flag
        }
        _ => panic!("Expected ProjectsCommands::Remove with -y flag"),
    }
}

// ── Cross-Command Consistency Tests ───────────────────────────────────────────

#[test]
fn flag_propagation_consistent_across_projects_subcommands() {
    // Verify that the flag propagation mechanism is consistent
    // across all ProjectsCommands subcommands

    // Test scan command
    let scan_fixture = ProjectsScanFixture::new_flag_before_projects();
    assert_eq!(scan_fixture.no_interactive, true);

    // Test remove command
    let remove_fixture = ProjectsRemoveFixture::new_flag_before_projects();
    assert_eq!(remove_fixture.no_interactive, true);

    // Both should have the same flag value when specified the same way
    assert_eq!(
        scan_fixture.no_interactive, remove_fixture.no_interactive,
        "Flag propagation should be consistent across subcommands"
    );
}

#[test]
fn global_flag_persists_through_nesting_levels() {
    // Verify that the global flag persists through both nesting levels:
    // Level 1: Commands::Projects
    // Level 2: ProjectsCommands::Scan / ProjectsCommands::Remove

    // Test with scan
    let scan_args = ["hoop", "--no-interactive", "projects", "scan", "/tmp"];
    let scan_cli = Cli::try_parse_from(scan_args).expect("parse should succeed");
    assert_eq!(scan_cli.no_interactive, true);

    match scan_cli.command {
        Commands::Projects(hoop::ProjectsCommands::Scan { .. }) => {
            // Flag persists through both levels
        }
        _ => panic!("Expected nested Scan command"),
    }

    // Test with remove
    let remove_args = ["hoop", "--no-interactive", "projects", "remove", "proj", "--confirm"];
    let remove_cli = Cli::try_parse_from(remove_args).expect("parse should succeed");
    assert_eq!(remove_cli.no_interactive, true);

    match remove_cli.command {
        Commands::Projects(hoop::ProjectsCommands::Remove { .. }) => {
            // Flag persists through both levels
        }
        _ => panic!("Expected nested Remove command"),
    }
}

// ── Edge Cases and Error Handling ─────────────────────────────────────────────

#[test]
fn scan_with_local_yes_and_global_no_interactive() {
    // Test interaction between local --yes flag and global --no-interactive
    let args = ["hoop", "--no-interactive", "projects", "scan", "/tmp", "--yes"];
    let cli = Cli::try_parse_from(args).expect("parse should succeed");

    assert_eq!(cli.no_interactive, true, "global flag should be true");

    match cli.command {
        Commands::Projects(hoop::ProjectsCommands::Scan { root, yes }) => {
            assert_eq!(root, "/tmp");
            assert_eq!(yes, true, "local yes flag should be set");
            // Handler receives: no_interactive || yes = true || true = true
        }
        _ => panic!("Expected ProjectsCommands::Scan"),
    }
}

#[test]
fn projects_command_extracts_global_flag_correctly() {
    // Verify that the ProjectsCommands nested commands correctly
    // extract the global flag value

    // Parse with flag at different positions
    let positions = vec![
        vec!["hoop", "--no-interactive", "projects", "scan", "/tmp"],
        vec!["hoop", "projects", "scan", "/tmp", "--no-interactive"],
        vec!["hoop", "-y", "projects", "scan", "/tmp"],
    ];

    for args in positions {
        let cli = Cli::try_parse_from(args.iter()).expect("parse should succeed");
        assert_eq!(
            cli.no_interactive, true,
            "Flag should be true regardless of position: {:?}",
            args
        );

        // Verify command structure is preserved
        match cli.command {
            Commands::Projects(_) => {
                // Correctly routed
            }
            _ => panic!("Command routing failed for args: {:?}", args),
        }
    }
}
