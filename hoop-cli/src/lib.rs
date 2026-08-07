//! HOOP CLI library — provides the project registry and shared utilities.

pub mod cli;
pub mod projects;

pub use cli::{AuditCommands, Cli, Commands, ProjectsCommands};
pub use projects::{
    add_project, list_projects, remove_project, scan_projects, show_project, validate_workspace,
    ProjectEntry, ProjectsRegistry,
};
