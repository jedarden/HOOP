//! TestRepo - A test fixture for HOOP integration testing

pub mod cli;
pub mod core;
pub mod api;
pub mod migrations;
pub mod models;
pub mod services;
pub mod utils;

pub use core::config::Config;
pub use core::error::Error;

pub const VERSION: &str = "0.1.0";
