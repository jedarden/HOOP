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

/// NEEDLE event types written to .beads/events.jsonl
///
/// Events are append-only and authoritative. HOOP reads them to derive
/// worker liveness, bead state, and cost/capacity projections.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum NeedleEvent {
    /// Worker claimed a bead
    Claim {
        ts: String,
        worker: String,
        bead: String,
        strand: Option<String>,
    },
    /// Worker dispatched a bead to an adapter
    Dispatch {
        ts: String,
        worker: String,
        bead: String,
        strand: Option<String>,
        adapter: String,
        model: String,
    },
    /// Worker completed a bead
    Complete {
        ts: String,
        worker: String,
        bead: String,
        strand: Option<String>,
        outcome: String,
        duration_ms: Option<u64>,
        exit_code: Option<i32>,
    },
    /// Worker failed a bead
    Fail {
        ts: String,
        worker: String,
        bead: String,
        strand: Option<String>,
        reason: String,
    },
    /// Worker released a bead claim
    Release {
        ts: String,
        worker: String,
        bead: String,
        reason: Option<String>,
    },
    /// Worker heartbeat
    Heartbeat {
        ts: String,
        worker: String,
        #[serde(flatten)]
        state: WorkerState,
    },
    /// Worker timed out on a bead
    Timeout {
        ts: String,
        worker: String,
        bead: String,
        duration_ms: u64,
    },
    /// Worker crashed
    Crash {
        ts: String,
        worker: String,
        bead: Option<String>,
        signal: Option<i32>,
    },
    /// Unknown event type (preserves raw data)
    #[serde(other)]
    Unknown,
}

/// Worker state as reported in heartbeat events
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum WorkerState {
    /// Worker is actively executing a bead
    Executing {
        bead: String,
        pid: u32,
        adapter: String,
    },
    /// Worker is idle, waiting for work
    Idle {
        last_strand: Option<String>,
    },
    /// Worker is in a terminal knot state
    Knot {
        reason: String,
    },
}

/// Parsed event with metadata
#[derive(Debug, Clone)]
pub struct ParsedEvent {
    /// The raw event data
    pub event: NeedleEvent,
    /// The line number in the events.jsonl file
    pub line_number: usize,
    /// The raw JSON string (for unknown events)
    pub raw: String,
}

impl NeedleEvent {
    /// Get the event timestamp
    pub fn timestamp(&self) -> Option<&str> {
        match self {
            NeedleEvent::Claim { ts, .. } => Some(ts),
            NeedleEvent::Dispatch { ts, .. } => Some(ts),
            NeedleEvent::Complete { ts, .. } => Some(ts),
            NeedleEvent::Fail { ts, .. } => Some(ts),
            NeedleEvent::Release { ts, .. } => Some(ts),
            NeedleEvent::Heartbeat { ts, .. } => Some(ts),
            NeedleEvent::Timeout { ts, .. } => Some(ts),
            NeedleEvent::Crash { ts, .. } => Some(ts),
            NeedleEvent::Unknown => None,
        }
    }

    /// Get the worker name for this event
    pub fn worker(&self) -> Option<&str> {
        match self {
            NeedleEvent::Claim { worker, .. } => Some(worker),
            NeedleEvent::Dispatch { worker, .. } => Some(worker),
            NeedleEvent::Complete { worker, .. } => Some(worker),
            NeedleEvent::Fail { worker, .. } => Some(worker),
            NeedleEvent::Release { worker, .. } => Some(worker),
            NeedleEvent::Heartbeat { worker, .. } => Some(worker),
            NeedleEvent::Timeout { worker, .. } => Some(worker),
            NeedleEvent::Crash { worker, .. } => Some(worker),
            NeedleEvent::Unknown => None,
        }
    }

    /// Get the bead ID for this event (if applicable)
    pub fn bead(&self) -> Option<&str> {
        match self {
            NeedleEvent::Claim { bead, .. } => Some(bead),
            NeedleEvent::Dispatch { bead, .. } => Some(bead),
            NeedleEvent::Complete { bead, .. } => Some(bead),
            NeedleEvent::Fail { bead, .. } => Some(bead),
            NeedleEvent::Release { bead, .. } => Some(bead),
            NeedleEvent::Heartbeat { .. } => None,
            NeedleEvent::Timeout { bead, .. } => Some(bead),
            NeedleEvent::Crash { bead, .. } => bead.as_deref(),
            NeedleEvent::Unknown => None,
        }
    }
}
