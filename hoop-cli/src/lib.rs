//! HOOP CLI library — provides the project registry and shared utilities.

pub mod projects;

pub use projects::{
    add_project, list_projects, remove_project, scan_projects, show_project, validate_workspace,
    ProjectEntry, ProjectsRegistry,
};
