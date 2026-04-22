//! HOOP schema definitions
//!
//! This crate provides the shared data types and schemas used across HOOP.
//! All records carry `schema_version: 1` for compatibility tracking.

use chrono::DateTime;

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

/// Session classification (from plan §1.6)
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SessionKind {
    /// Human ↔ agent chat (normal conversation)
    Operator,
    /// Voice note with Whisper transcript
    Dictated,
    /// NEEDLE worker's CLI session (tagged with `[needle:<worker>:<bead>:<strand>]`)
    Worker { worker: String, bead: String, strand: Option<String> },
    /// Direct CLI session without prefix tag
    AdHoc,
}

/// Token usage from a single message
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageUsage {
    /// Input tokens (prompt)
    pub input_tokens: u64,
    /// Output tokens (completion)
    pub output_tokens: u64,
    /// Cache read tokens (prompt cache hits)
    pub cache_read_tokens: u64,
    /// Cache write tokens (new cache entries)
    pub cache_write_tokens: u64,
}

/// A single message in a session
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionMessage {
    /// Role (user, assistant, system)
    pub role: String,
    /// Message content (may be text or structured for tool use)
    pub content: serde_json::Value,
    /// Token usage (present on assistant messages)
    pub usage: Option<MessageUsage>,
    /// Timestamp if available
    pub timestamp: Option<String>,
}

/// Parsed session from CLI adapter
#[derive(Debug, Clone)]
pub struct ParsedSession {
    /// Stable UI ID (UUID assigned by HOOP)
    pub id: String,
    /// Provider-native session ID (from CLI)
    pub session_id: String,
    /// Provider name (claude, codex, gemini, opencode)
    pub provider: String,
    /// Session classification
    pub kind: SessionKind,
    /// Working directory when session was created
    pub cwd: String,
    /// Session title (from first prompt or derived)
    pub title: String,
    /// Messages in the session
    pub messages: Vec<SessionMessage>,
    /// Total token usage across all messages
    pub total_usage: MessageUsage,
    /// Creation time
    pub created_at: DateTime<chrono::Utc>,
    /// Last update time
    pub updated_at: DateTime<chrono::Utc>,
    /// Whether the session is complete (process exited)
    pub complete: bool,
    /// Path to the session file on disk
    pub file_path: String,
}

impl ParsedSession {
    /// Get total tokens used (input + output + cache write)
    pub fn total_tokens(&self) -> u64 {
        self.total_usage.input_tokens
            + self.total_usage.output_tokens
            + self.total_usage.cache_write_tokens
    }
}

/// Bead status from issues.jsonl
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BeadStatus {
    Open,
    Closed,
}

/// Bead issue type from issues.jsonl
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BeadType {
    #[default]
    Task,
    Bug,
    Epic,
    #[serde(other)]
    Unknown,
}

/// A bead from issues.jsonl (for frontend display)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Bead {
    /// Bead ID (e.g., "hoop-ttb.1")
    pub id: String,
    /// Title
    pub title: String,
    /// Description (markdown content)
    #[serde(default)]
    pub description: String,
    /// Status
    pub status: BeadStatus,
    /// Priority (0 = highest)
    #[serde(default)]
    pub priority: i64,
    /// Issue type
    #[serde(default)]
    pub issue_type: BeadType,
    /// Created timestamp
    pub created_at: DateTime<chrono::Utc>,
    /// Created by
    #[serde(default)]
    pub created_by: String,
    /// Updated timestamp
    pub updated_at: DateTime<chrono::Utc>,
    /// Closed timestamp (if closed)
    pub closed_at: Option<DateTime<chrono::Utc>>,
    /// Close reason (if closed)
    pub close_reason: Option<String>,
    /// Source repo
    #[serde(default = "default_source_repo")]
    pub source_repo: String,
    /// Dependencies
    #[serde(default)]
    pub dependencies: Vec<String>,
}

fn default_source_repo() -> String {
    ".".to_string()
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
