//! HOOP CLI library — provides the project registry and shared utilities.

pub mod projects;

pub use projects::{
    ProjectEntry, ProjectsRegistry,
    add_project, list_projects, remove_project, show_project, scan_projects, validate_workspace,
};
