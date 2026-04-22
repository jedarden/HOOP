//! HOOP schema definitions
//!
//! This crate provides the shared data types and schemas used across HOOP.
//! All records carry `schema_version: 1` for compatibility tracking.

pub mod version {
    /// Current schema version following SemVer (X.Y.Z)
    pub const SCHEMA_VERSION: &str = "0.1.0";
}

/// Base trait for all schema records
pub trait SchemaRecord {
    /// Returns the schema version for this record type
    fn schema_version(&self) -> &'static str {
        version::SCHEMA_VERSION
    }
}

/// Health check response
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: &'static str,
}

impl HealthResponse {
    pub fn ok() -> Self {
        Self {
            status: "ok".to_string(),
            version: version::SCHEMA_VERSION,
        }
    }
}

/// Control socket request type
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ControlRequest {
    /// Get daemon status
    Status { project: Option<String> },
}

/// Control socket response type
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ControlResponse {
    /// Status response
    Status(StatusResponse),
    /// Error response
    Error { message: String },
}

/// Status response data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StatusResponse {
    pub daemon_running: bool,
    pub uptime_secs: u64,
    pub projects: Vec<ProjectStatus>,
}

/// Status of a single project
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectStatus {
    pub name: String,
    pub path: String,
    pub active_beads: usize,
    pub workers: usize,
}
