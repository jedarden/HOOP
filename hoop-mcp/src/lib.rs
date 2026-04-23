//! HOOP MCP Server library
//!
//! Core modules for the MCP server: protocol handling, tool implementations,
//! audit logging, br verb classification, and socket transport.

pub mod audit;
pub mod br_verbs;
pub mod protocol;
pub mod socket;
pub mod tools;
