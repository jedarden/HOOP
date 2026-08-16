//! Edge cases for the global `--no-interactive` flag using the real Clap parser.

mod clap_test_utils;

use clap_test_utils::{parse_cli, Commands, ProjectsCommands};

#[test]
fn duplicate_no_interactive_flags_use_the_last_occurrence() {
    // This is a positive boolean flag, so every occurrence sets the same value.
    // `args_override_self` makes the final occurrence win instead of reporting a
    // duplicate-argument conflict.
    let before_command = parse_cli(&["hoop", "--no-interactive", "--no-interactive", "status"])
        .expect("duplicate global flags should be accepted");
    assert!(before_command.no_interactive);

    let after_command = parse_cli(&["hoop", "status", "--no-interactive", "-y"])
        .expect("duplicate global aliases should be accepted");
    assert!(after_command.no_interactive);
}

#[test]
fn global_flag_survives_projects_remove_command_chain() {
    let cli = parse_cli(&[
        "hoop",
        "--no-interactive",
        "projects",
        "remove",
        "example-project",
        "--confirm",
    ])
    .expect("projects remove chain should parse");

    assert!(cli.no_interactive);
    match cli.command {
        Commands::Projects(ProjectsCommands::Remove { name, confirm }) => {
            assert_eq!(name, "example-project");
            assert!(confirm);
        }
        command => panic!("expected projects remove, got {command:?}"),
    }
}

#[test]
fn global_flag_coexists_with_command_confirmation_options() {
    let cli = parse_cli(&[
        "hoop",
        "projects",
        "scan",
        "/tmp/projects",
        "--yes",
        "--no-interactive",
    ])
    .expect("global and local confirmation flags should not conflict");

    assert!(cli.no_interactive);
    match cli.command {
        Commands::Projects(ProjectsCommands::Scan { root, yes }) => {
            assert_eq!(root, "/tmp/projects");
            assert!(yes);
        }
        command => panic!("expected projects scan, got {command:?}"),
    }
}

#[test]
fn omitted_global_flag_defaults_to_interactive_mode() {
    let cli = parse_cli(&["hoop", "projects", "remove", "example-project"])
        .expect("projects remove without global flag should parse");

    assert!(!cli.no_interactive);
    match cli.command {
        Commands::Projects(ProjectsCommands::Remove { name, confirm }) => {
            assert_eq!(name, "example-project");
            assert!(!confirm);
        }
        command => panic!("expected projects remove, got {command:?}"),
    }
}
