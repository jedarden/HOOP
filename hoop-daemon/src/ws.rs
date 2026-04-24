//! WebSocket endpoint for real-time worker updates
//!
//! Broadcasts worker state changes, heartbeats, and liveness transitions
//! to connected web UI clients.

use crate::heartbeats::{MonitorEvent, WorkerHeartbeat, WorkerLiveness};
use crate::sessions::SessionEvent;
use crate::{Bead, DaemonState, WorkerState};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use futures_util::{stream::StreamExt, SinkExt};
use hoop_schema::{ParsedSession, ParsedSessionKind, ParsedSessionKindVariant1, ParsedSessionKindVariant2, ParsedSessionKindVariant3, ParsedSessionMessagesItem, ParsedSessionMessagesItemUsage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, warn};

// ---------------------------------------------------------------------------
// WS connection tracker (§16.8: every WS client in /debug/state)
// ---------------------------------------------------------------------------

/// Snapshot of a single connected WS client for /debug/state.
#[derive(Debug, Clone, Serialize)]
pub struct WsClientRecord {
    pub conn_id: u64,
    pub connected_at: DateTime<Utc>,
    pub connected_secs: i64,
}

/// Registry of currently connected WebSocket clients.
///
/// Each call to [`register`] returns a [`WsConnectionGuard`]; when the guard
/// is dropped the connection is automatically removed.
#[derive(Debug, Clone)]
pub struct WsConnectionTracker {
    next_id: Arc<AtomicU64>,
    clients: Arc<std::sync::RwLock<Vec<(u64, DateTime<Utc>)>>>,
}

impl WsConnectionTracker {
    pub fn new() -> Self {
        Self {
            next_id: Arc::new(AtomicU64::new(1)),
            clients: Arc::new(std::sync::RwLock::new(Vec::new())),
        }
    }

    /// Register a new WS connection. The returned guard must be held for the
    /// duration of the connection; dropping it deregisters the client.
    pub fn register(&self) -> WsConnectionGuard {
        let conn_id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let connected_at = Utc::now();
        self.clients.write().unwrap().push((conn_id, connected_at));
        WsConnectionGuard {
            conn_id,
            clients: self.clients.clone(),
        }
    }

    /// Return a snapshot of all currently connected clients.
    pub fn snapshot(&self) -> Vec<WsClientRecord> {
        let now = Utc::now();
        self.clients
            .read()
            .unwrap()
            .iter()
            .map(|(id, connected_at)| WsClientRecord {
                conn_id: *id,
                connected_at: *connected_at,
                connected_secs: (now - *connected_at).num_seconds().max(0),
            })
            .collect()
    }

    pub fn count(&self) -> usize {
        self.clients.read().unwrap().len()
    }
}

impl Default for WsConnectionTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII guard returned by [`WsConnectionTracker::register`].
/// Removes the connection from the tracker on drop.
pub struct WsConnectionGuard {
    conn_id: u64,
    clients: Arc<std::sync::RwLock<Vec<(u64, DateTime<Utc>)>>>,
}

impl Drop for WsConnectionGuard {
    fn drop(&mut self) {
        let mut lock = self.clients.write().unwrap();
        lock.retain(|(id, _)| *id != self.conn_id);
    }
}

/// Word-level timestamp for Whisper transcript sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptWordData {
    pub word: String,
    pub start: f64,
    pub end: f64,
}

/// Dictated note metadata sent to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictatedNoteData {
    pub stitch_id: String,
    pub audio_url: String,
    pub transcript: String,
    pub transcript_words: Vec<TranscriptWordData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_secs: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    pub recorded_at: String,
    pub transcription_status: String,
}

/// Worker data sent to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerData {
    pub worker: String,
    pub state: WorkerDisplayState,
    pub liveness: WorkerLiveness,
    pub last_heartbeat: DateTime<Utc>,
    pub heartbeat_age_secs: i64,
}

/// Bead data sent to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeadData {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: i64,
    pub issue_type: String,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: String,
    pub dependencies: Vec<String>,
}

/// Message usage sent to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageUsageData {
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
}

impl From<ParsedSessionMessagesItemUsage> for MessageUsageData {
    fn from(u: ParsedSessionMessagesItemUsage) -> Self {
        Self {
            input_tokens: u.input_tokens,
            output_tokens: u.output_tokens,
            cache_read_tokens: u.cache_read_tokens,
            cache_write_tokens: u.cache_write_tokens,
        }
    }
}

/// Session message sent to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessageData {
    pub role: String,
    pub content: serde_json::Value,
    pub usage: Option<MessageUsageData>,
    pub timestamp: Option<String>,
}

impl From<ParsedSessionMessagesItem> for SessionMessageData {
    fn from(m: ParsedSessionMessagesItem) -> Self {
        // §18.3: Apply read-side redaction before emitting to UI.
        // Raw CLI session files are never modified; only the projection is redacted.
        let content = crate::redaction::redact_json_value(m.content);
        Self {
            role: m.role,
            content,
            usage: m.usage.map(MessageUsageData::from),
            timestamp: m.timestamp.map(|t| t.to_rfc3339()),
        }
    }
}

/// Worker metadata for worker sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerMetadataData {
    pub worker: String,
    pub bead: String,
    pub strand: Option<String>,
}

/// Conversation data sent to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationData {
    pub id: String,
    pub session_id: String,
    pub provider: String,
    pub kind: String,
    pub worker_metadata: Option<WorkerMetadataData>,
    pub cwd: String,
    pub title: String,
    pub messages: Vec<SessionMessageData>,
    pub total_tokens: u64,
    pub created_at: String,
    pub updated_at: String,
    pub complete: bool,
    pub file_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dictated_note: Option<DictatedNoteData>,
}

/// Load dictated note data from fleet.db for a stitch
fn load_dictated_note(stitch_id: &str) -> Option<DictatedNoteData> {
    let db_path = crate::fleet::db_path();
    let conn = rusqlite::Connection::open(&db_path).ok()?;
    conn.pragma_update(None, "journal_mode", "WAL").ok()?;

    let result = conn.query_row(
        r#"
        SELECT dn.audio_filename, dn.transcript, dn.transcript_words,
               dn.duration_secs, dn.language, dn.recorded_at,
               COALESCE(dn.transcription_status, '"Pending"')
        FROM dictated_notes dn
        WHERE dn.stitch_id = ?1
        "#,
        rusqlite::params![stitch_id],
        |row| {
            let audio_filename: String = row.get(0)?;
            let transcript: String = row.get(1)?;
            let words_json: Option<String> = row.get(2)?;
            let duration_secs: Option<f64> = row.get(3)?;
            let language: Option<String> = row.get(4)?;
            let recorded_at: String = row.get(5)?;
            let status_str: String = row.get(6)?;
            Ok((audio_filename, transcript, words_json, duration_secs, language, recorded_at, status_str))
        },
    );

    match result {
        Ok((_audio_filename, transcript, words_json, duration_secs, language, recorded_at, status_str)) => {
            let transcript_words: Vec<TranscriptWordData> = words_json
                .and_then(|j| serde_json::from_str::<Vec<crate::dictated_notes::TranscriptWord>>(&j).ok())
                .unwrap_or_default()
                .into_iter()
                .map(|w| TranscriptWordData { word: w.word, start: w.start, end: w.end })
                .collect();

            let audio_url = format!("/api/dictated-notes/{}/audio", stitch_id);
            let transcription_status = match status_str.trim_matches('"') {
                "Completed" => "Completed".to_string(),
                "Failed" => "Failed".to_string(),
                _ => "Pending".to_string(),
            };

            Some(DictatedNoteData {
                stitch_id: stitch_id.to_string(),
                audio_url,
                transcript,
                transcript_words,
                duration_secs,
                language,
                recorded_at,
                transcription_status,
            })
        }
        Err(_) => None,
    }
}

impl From<ParsedSession> for ConversationData {
    fn from(s: ParsedSession) -> Self {
        let is_dictated = matches!(s.kind, ParsedSessionKind::Variant1(ParsedSessionKindVariant1::Dictated));

        let (worker_metadata, kind_str) = match &s.kind {
            ParsedSessionKind::Variant3(ParsedSessionKindVariant3::Operator) => (None, "operator".to_string()),
            ParsedSessionKind::Variant1(ParsedSessionKindVariant1::Dictated) => (None, "dictated".to_string()),
            ParsedSessionKind::Variant0 { worker, bead, strand } => (
                Some(WorkerMetadataData {
                    worker: worker.clone(),
                    bead: bead.clone(),
                    strand: strand.clone(),
                }),
                "worker".to_string(),
            ),
            ParsedSessionKind::Variant2(ParsedSessionKindVariant2::AdHoc) => (None, "ad-hoc".to_string()),
        };

        // For dictated sessions try session_id first, then id (in case they differ)
        let dictated_note = if is_dictated {
            load_dictated_note(&s.session_id)
                .or_else(|| load_dictated_note(&s.id))
        } else {
            None
        };

        Self {
            id: s.id,
            session_id: s.session_id,
            provider: s.provider,
            kind: kind_str,
            worker_metadata,
            cwd: s.cwd,
            title: s.title,
            messages: s.messages.into_iter().map(SessionMessageData::from).collect(),
            total_tokens: (s.total_usage.input_tokens + s.total_usage.output_tokens) as u64,
            created_at: s.created_at.to_rfc3339(),
            updated_at: s.updated_at.to_rfc3339(),
            complete: s.complete,
            file_path: s.file_path,
            dictated_note,
        }
    }
}

/// Streaming content update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingContentData {
    pub conversation_id: String,
    pub content: String,
    pub timestamp: u64,
}

/// Project card data sent to the frontend overview page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCardData {
    pub name: String,
    pub label: String,
    pub color: String,
    pub path: String,
    pub degraded: bool,
    /// Runtime state (e.g., "healthy", "failed", "error", "starting")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_state: Option<String>,
    /// Error message if runtime is in an error state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_error: Option<String>,
    /// Number of active (open) beads in this project
    pub bead_count: usize,
    /// Number of workers associated with this project
    pub worker_count: usize,
    /// Number of active stitches (open beads currently being worked)
    pub active_stitch_count: usize,
    /// Estimated cost today in USD
    pub cost_today: f64,
    /// Number of stuck (knot-state) workers
    pub stuck_count: usize,
    /// ISO 8601 timestamp of last activity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_activity: Option<String>,
}

/// Configuration error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigErrorData {
    /// Human-readable error message
    pub message: String,
    /// Line number where the error occurred (1-indexed)
    pub line: usize,
    /// Column number where the error occurred (1-indexed)
    pub col: usize,
}

/// Configuration status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigStatusData {
    /// True if the configuration is valid
    pub valid: bool,
    /// Error details if invalid
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ConfigErrorData>,
}

/// Worker state for display (combines WorkerState with additional info)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum WorkerDisplayState {
    Executing {
        bead: String,
        adapter: String,
        model: Option<String>,
    },
    Idle {
        last_strand: Option<String>,
    },
    Knot {
        reason: String,
    },
}

/// Bead event data from events.jsonl for debug panel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeadEventData {
    pub timestamp: String,
    pub event_type: String,
    pub bead_id: String,
    pub worker: String,
    pub line_number: Option<usize>,
    pub raw: String,
}

/// Morning brief event data sent to WS clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorningBriefData {
    pub id: String,
    pub headline: String,
    pub generated_at: String,
    pub draft_count: usize,
    pub status: String,
}

/// Stitch created event data sent after a bead is created via the submit flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StitchCreatedData {
    pub bead_id: String,
    pub title: String,
    pub project: String,
    pub stitch_id: Option<String>,
    pub source: String,
    pub actor: String,
    pub created_at: String,
}

/// Draft queue event data sent to WS clients when a draft is created, edited, approved, or rejected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftUpdateData {
    pub draft_id: String,
    pub project: String,
    pub title: String,
    pub kind: String,
    pub status: String,
    pub action: String,
    pub actor: String,
    pub created_by: String,
    pub version: i64,
    pub rejection_reason: Option<String>,
}

/// WebSocket event sent to clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WsEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker: Option<WorkerData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workers: Option<Vec<WorkerData>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub beads: Option<Vec<BeadData>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversations: Option<Vec<ConversationData>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation: Option<ConversationData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streaming: Option<StreamingContentData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projects: Option<Vec<ProjectCardData>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_status: Option<ConfigStatusData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity: Option<Vec<crate::capacity::AccountCapacity>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bead_event: Option<BeadEventData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bead_events: Option<Vec<BeadEventData>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stitch_created: Option<StitchCreatedData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_session: Option<crate::agent_session::AgentSessionEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub morning_brief: Option<MorningBriefData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft_update: Option<DraftUpdateData>,
}

impl WsEvent {
    /// Create a worker update event
    fn worker_update(worker: WorkerData) -> Self {
        Self {
            event_type: "worker_update".to_string(),
            worker: Some(worker),
            workers: None,
            beads: None,
            conversations: None,
            conversation: None,
            streaming: None,
            projects: None,
            config_status: None,
            capacity: None,
            bead_event: None,
            bead_events: None,
            stitch_created: None,
            agent_session: None,
            morning_brief: None,
            draft_update: None,
        }
    }

    /// Create a full worker snapshot event
    fn workers_snapshot(workers: Vec<WorkerData>) -> Self {
        Self {
            event_type: "workers_snapshot".to_string(),
            worker: None,
            workers: Some(workers),
            beads: None,
            conversations: None,
            conversation: None,
            streaming: None,
            projects: None,
            config_status: None,
            capacity: None,
            bead_event: None,
            bead_events: None,
            stitch_created: None,
            agent_session: None,
            morning_brief: None,
            draft_update: None,
        }
    }

    /// Create a beads snapshot event
    fn beads_snapshot(beads: Vec<BeadData>) -> Self {
        Self {
            event_type: "beads_snapshot".to_string(),
            worker: None,
            workers: None,
            beads: Some(beads),
            conversations: None,
            conversation: None,
            streaming: None,
            projects: None,
            config_status: None,
            capacity: None,
            bead_event: None,
            bead_events: None,
            stitch_created: None,
            agent_session: None,
            morning_brief: None,
            draft_update: None,
        }
    }

    /// Create a conversations snapshot event
    fn conversations_snapshot(conversations: Vec<ConversationData>) -> Self {
        Self {
            event_type: "conversations_snapshot".to_string(),
            worker: None,
            workers: None,
            beads: None,
            conversations: Some(conversations),
            conversation: None,
            streaming: None,
            projects: None,
            config_status: None,
            capacity: None,
            bead_event: None,
            bead_events: None,
            stitch_created: None,
            agent_session: None,
            morning_brief: None,
            draft_update: None,
        }
    }

    /// Create a conversation update event
    #[allow(dead_code)]
    fn conversation_update(conversation: ConversationData) -> Self {
        Self {
            event_type: "conversation_update".to_string(),
            worker: None,
            workers: None,
            beads: None,
            conversations: None,
            conversation: Some(conversation),
            streaming: None,
            projects: None,
            config_status: None,
            capacity: None,
            bead_event: None,
            bead_events: None,
            stitch_created: None,
            agent_session: None,
            morning_brief: None,
            draft_update: None,
        }
    }

    /// Create a streaming content event
    #[allow(dead_code)]
    fn streaming_content(data: StreamingContentData) -> Self {
        Self {
            event_type: "streaming_content".to_string(),
            worker: None,
            workers: None,
            beads: None,
            conversations: None,
            conversation: None,
            streaming: Some(data),
            projects: None,
            config_status: None,
            capacity: None,
            bead_event: None,
            bead_events: None,
            stitch_created: None,
            agent_session: None,
            morning_brief: None,
            draft_update: None,
        }
    }

    /// Create a projects snapshot event
    pub fn projects_snapshot(projects: Vec<ProjectCardData>) -> Self {
        Self {
            event_type: "projects_snapshot".to_string(),
            worker: None,
            workers: None,
            beads: None,
            conversations: None,
            conversation: None,
            streaming: None,
            projects: Some(projects),
            config_status: None,
            capacity: None,
            bead_event: None,
            bead_events: None,
            stitch_created: None,
            agent_session: None,
            morning_brief: None,
            draft_update: None,
        }
    }

    /// Create a config status event
    pub fn config_status(status: ConfigStatusData) -> Self {
        Self {
            event_type: "config_status".to_string(),
            worker: None,
            workers: None,
            beads: None,
            conversations: None,
            conversation: None,
            streaming: None,
            projects: None,
            config_status: Some(status),
            capacity: None,
            bead_event: None,
            bead_events: None,
            stitch_created: None,
            agent_session: None,
            morning_brief: None,
            draft_update: None,
        }
    }

    /// Create a capacity snapshot event
    pub fn capacity_snapshot(capacity: Vec<crate::capacity::AccountCapacity>) -> Self {
        Self {
            event_type: "capacity_snapshot".to_string(),
            worker: None,
            workers: None,
            beads: None,
            conversations: None,
            conversation: None,
            streaming: None,
            projects: None,
            config_status: None,
            capacity: Some(capacity),
            bead_event: None,
            bead_events: None,
            stitch_created: None,
            agent_session: None,
            morning_brief: None,
            draft_update: None,
        }
    }

    /// Create a bead event update (single event from events.jsonl)
    pub fn bead_event_update(event: BeadEventData) -> Self {
        Self {
            event_type: "bead_event".to_string(),
            worker: None,
            workers: None,
            beads: None,
            conversations: None,
            conversation: None,
            streaming: None,
            projects: None,
            config_status: None,
            capacity: None,
            bead_event: Some(event),
            bead_events: None,
            stitch_created: None,
            agent_session: None,
            morning_brief: None,
            draft_update: None,
        }
    }

    /// Create a bead events snapshot (all events for a bead)
    pub fn bead_events_snapshot(events: Vec<BeadEventData>) -> Self {
        Self {
            event_type: "bead_events".to_string(),
            worker: None,
            workers: None,
            beads: None,
            conversations: None,
            conversation: None,
            streaming: None,
            projects: None,
            config_status: None,
            capacity: None,
            bead_event: None,
            bead_events: Some(events),
            stitch_created: None,
            agent_session: None,
            morning_brief: None,
            draft_update: None,
        }
    }

    /// Create a stitch_created event emitted after successful bead creation
    pub fn stitch_created(data: StitchCreatedData) -> Self {
        Self {
            event_type: "stitch_created".to_string(),
            worker: None,
            workers: None,
            beads: None,
            conversations: None,
            conversation: None,
            streaming: None,
            projects: None,
            config_status: None,
            capacity: None,
            bead_event: None,
            bead_events: None,
            stitch_created: Some(data),
            agent_session: None,
            morning_brief: None,
            draft_update: None,
        }
    }

    /// Create an agent session event
    pub fn agent_session(event: crate::agent_session::AgentSessionEvent) -> Self {
        Self {
            event_type: "agent_session".to_string(),
            worker: None,
            workers: None,
            beads: None,
            conversations: None,
            conversation: None,
            streaming: None,
            projects: None,
            config_status: None,
            capacity: None,
            bead_event: None,
            bead_events: None,
            stitch_created: None,
            agent_session: Some(event),
            morning_brief: None,
            draft_update: None,
        }
    }

    /// Create a morning brief event
    pub fn morning_brief(data: MorningBriefData) -> Self {
        Self {
            event_type: "morning_brief".to_string(),
            worker: None,
            workers: None,
            beads: None,
            conversations: None,
            conversation: None,
            streaming: None,
            projects: None,
            config_status: None,
            capacity: None,
            bead_event: None,
            bead_events: None,
            stitch_created: None,
            agent_session: None,
            morning_brief: Some(data),
            draft_update: None,
        }
    }

    /// Create a draft_update event for the draft queue
    pub fn draft_update(data: DraftUpdateData) -> Self {
        Self {
            event_type: "draft_update".to_string(),
            worker: None,
            workers: None,
            beads: None,
            conversations: None,
            conversation: None,
            streaming: None,
            projects: None,
            config_status: None,
            capacity: None,
            bead_event: None,
            bead_events: None,
            stitch_created: None,
            agent_session: None,
            morning_brief: None,
            draft_update: Some(data),
        }
    }
}

/// Shared worker registry
#[derive(Debug, Clone)]
pub struct WorkerRegistry {
    workers: Arc<RwLock<Vec<WorkerData>>>,
    conversations: Arc<RwLock<Vec<ConversationData>>>,
    /// Bead events from events.jsonl, keyed by bead_id
    bead_events: Arc<RwLock<HashMap<String, Vec<BeadEventData>>>>,
    /// Latest observed OS PID per worker name (persists across state transitions).
    worker_pids: Arc<RwLock<HashMap<String, u32>>>,
    monitor: broadcast::Sender<MonitorEvent>,
    sessions: broadcast::Sender<SessionEvent>,
}

impl WorkerRegistry {
    pub fn new(monitor: broadcast::Sender<MonitorEvent>, sessions: broadcast::Sender<SessionEvent>) -> Self {
        Self {
            workers: Arc::new(RwLock::new(Vec::new())),
            conversations: Arc::new(RwLock::new(Vec::new())),
            bead_events: Arc::new(RwLock::new(HashMap::new())),
            worker_pids: Arc::new(RwLock::new(HashMap::new())),
            monitor,
            sessions,
        }
    }

    /// Get current snapshot of all workers
    pub async fn snapshot(&self) -> Vec<WorkerData> {
        self.workers.read().await.clone()
    }

    /// Get current snapshot of all conversations
    pub async fn conversations_snapshot(&self) -> Vec<ConversationData> {
        self.conversations.read().await.clone()
    }

    /// Update conversations with new batch
    pub async fn update_conversations(&self, sessions: Vec<ParsedSession>) {
        let mut convos = self.conversations.write().await;
        let new_data: Vec<ConversationData> = sessions.into_iter().map(ConversationData::from).collect();

        // Merge with existing: update existing, add new
        for new_convo in &new_data {
            if let Some(existing) = convos.iter_mut().find(|c| c.id == new_convo.id) {
                *existing = new_convo.clone();
            } else {
                convos.push(new_convo.clone());
            }
        }
    }

    /// Update a single conversation
    pub async fn update_conversation(&self, session: ParsedSession) {
        let mut convos = self.conversations.write().await;
        let data = ConversationData::from(session);

        if let Some(existing) = convos.iter_mut().find(|c| c.id == data.id) {
            *existing = data;
        } else {
            convos.push(data);
        }
    }

    /// Subscribe to monitor events
    pub fn subscribe(&self) -> broadcast::Receiver<MonitorEvent> {
        self.monitor.subscribe()
    }

    /// Subscribe to session events
    pub fn subscribe_sessions(&self) -> broadcast::Receiver<SessionEvent> {
        self.sessions.subscribe()
    }

    /// Update or insert a worker entry, tracking the latest observed PID.
    pub async fn update_worker(&self, heartbeat: crate::heartbeats::WorkerHeartbeat, liveness: crate::heartbeats::WorkerLiveness) {
        // Track PID when the worker is in Executing state.
        if let crate::WorkerState::Executing { pid, .. } = &heartbeat.state {
            if *pid > 0 {
                self.worker_pids.write().await.insert(heartbeat.worker.clone(), *pid);
            }
        }

        let mut workers = self.workers.write().await;
        let age = (chrono::Utc::now() - heartbeat.ts).num_seconds().max(0);
        let state = match &heartbeat.state {
            crate::WorkerState::Executing { bead, adapter, .. } => WorkerDisplayState::Executing {
                bead: bead.clone(),
                adapter: adapter.clone(),
                model: None,
            },
            crate::WorkerState::Idle { last_strand } => WorkerDisplayState::Idle {
                last_strand: last_strand.clone(),
            },
            crate::WorkerState::Knot { reason } => WorkerDisplayState::Knot {
                reason: reason.clone(),
            },
        };
        if let Some(existing) = workers.iter_mut().find(|w| w.worker == heartbeat.worker) {
            existing.state = state;
            existing.liveness = liveness;
            existing.last_heartbeat = heartbeat.ts;
            existing.heartbeat_age_secs = age;
        } else {
            workers.push(WorkerData {
                worker: heartbeat.worker,
                state,
                liveness,
                last_heartbeat: heartbeat.ts,
                heartbeat_age_secs: age,
            });
        }
    }

    /// Return a snapshot of latest observed PIDs, keyed by worker name.
    pub async fn worker_pids_snapshot(&self) -> HashMap<String, u32> {
        self.worker_pids.read().await.clone()
    }

    /// Get bead events for a specific bead
    pub async fn get_bead_events(&self, bead_id: &str) -> Vec<BeadEventData> {
        self.bead_events.read().await.get(bead_id).cloned().unwrap_or_default()
    }

    /// Get all bead events
    pub async fn all_bead_events(&self) -> std::collections::HashMap<String, Vec<BeadEventData>> {
        self.bead_events.read().await.clone()
    }

    /// Add a bead event (from events.jsonl)
    pub async fn add_bead_event(&self, event: BeadEventData) {
        let mut events = self.bead_events.write().await;
        events.entry(event.bead_id.clone()).or_default().push(event);
    }

    /// Add multiple bead events
    pub async fn add_bead_events(&self, new_events: Vec<BeadEventData>) {
        let mut events = self.bead_events.write().await;
        for event in new_events {
            events.entry(event.bead_id.clone()).or_default().push(event);
        }
    }
}

/// Convert Bead to BeadData for WebSocket
fn bead_to_data(bead: &Bead) -> BeadData {
    BeadData {
        id: bead.id.clone(),
        title: bead.title.clone(),
        status: format!("{:?}", bead.status).to_lowercase(),
        priority: bead.priority,
        issue_type: format!("{:?}", bead.issue_type).to_lowercase(),
        created_at: bead.created_at.to_rfc3339(),
        updated_at: bead.updated_at.to_rfc3339(),
        created_by: bead.created_by.clone(),
        dependencies: bead.dependencies.clone(),
    }
}

/// WebSocket upgrade handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<DaemonState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle a WebSocket connection
async fn handle_socket(socket: WebSocket, state: DaemonState) {
    // Register with the shutdown coordinator (for graceful drain) and with the
    // WS connection tracker (for /debug/state "every WS client" requirement).
    let _conn_token = state.shutdown.register_connection();
    let _ws_guard = state.ws_connection_tracker.register();

    let (mut sender, mut receiver) = socket.split();
    let registry = state.worker_registry.clone();
    let mut monitor_rx = registry.subscribe();
    let mut bead_rx = state.bead_tx.subscribe();
    let mut stitch_rx = state.stitch_tx.subscribe();
    let _session_rx = registry.subscribe_sessions();
    let mut config_status_rx = state.config_status_tx.subscribe();
    let mut project_status_rx = state.project_status_tx.subscribe();
    let mut capacity_rx = state.capacity_tx.subscribe();
    let mut brief_rx = state.brief_tx.subscribe();
    let mut draft_rx = state.draft_tx.subscribe();
    let mut shutdown_rx = state.shutdown.subscribe();

    // Create an mpsc channel as intermediary: all producer tasks send WsEvent strings here,
    // and a single forwarder task drains them to the WebSocket sender.
    let (ws_tx, mut ws_rx) = tokio::sync::mpsc::channel::<String>(256);

    // Send initial snapshots
    let worker_snapshot = registry.snapshot().await;
    if let Ok(json) = serde_json::to_string(&WsEvent::workers_snapshot(worker_snapshot)) {
        if sender.send(Message::Text(json)).await.is_err() {
            return;
        }
    }

    // Send beads snapshot
    let beads = state.beads.read().unwrap().clone();
    let bead_data: Vec<BeadData> = beads.iter().map(bead_to_data).collect();
    if let Ok(json) = serde_json::to_string(&WsEvent::beads_snapshot(bead_data)) {
        if sender.send(Message::Text(json)).await.is_err() {
            return;
        }
    }

    // Send conversations snapshot
    let convos = registry.conversations_snapshot().await;
    if !convos.is_empty() {
        if let Ok(json) = serde_json::to_string(&WsEvent::conversations_snapshot(convos)) {
            if sender.send(Message::Text(json)).await.is_err() {
                return;
            }
        }
    }

    // Send projects snapshot
    {
        let projects = state.projects.read().unwrap().clone();
        if !projects.is_empty() {
            if let Ok(json) = serde_json::to_string(&WsEvent::projects_snapshot(projects)) {
                if sender.send(Message::Text(json)).await.is_err() {
                    return;
                }
            }
        }
    }

    // Send initial config status (valid by default since daemon started successfully)
    let initial_config_status = ConfigStatusData {
        valid: true,
        error: None,
    };
    if let Ok(json) = serde_json::to_string(&WsEvent::config_status(initial_config_status)) {
        if sender.send(Message::Text(json)).await.is_err() {
            return;
        }
    }

    // Send initial agent session status if active
    if let Some(ref mgr) = state.agent_session_manager {
        let status = mgr.status().await;
        if status.active {
            if let Ok(json) = serde_json::to_string(&WsEvent::agent_session(
                crate::agent_session::AgentSessionEvent::SessionReattached {
                    session_id: status.session_id.unwrap_or_default(),
                    adapter: status.adapter.unwrap_or_default(),
                    model: status.model.unwrap_or_default(),
                },
            )) {
                if sender.send(Message::Text(json)).await.is_err() {
                    return;
                }
            }
        }
    }

    // Send latest morning brief snapshot (if one exists)
    if let Ok(Some(brief)) = crate::fleet::get_latest_morning_brief() {
        if let Ok(json) = serde_json::to_string(&WsEvent::morning_brief(MorningBriefData {
            id: brief.id,
            headline: brief.headline,
            generated_at: brief.generated_at,
            draft_count: brief.draft_ids.len(),
            status: brief.status,
        })) {
            if sender.send(Message::Text(json)).await.is_err() {
                return;
            }
        }
    }

    // Forwarder task: drains ws_rx mpsc → WebSocket sender
    let forwarder_task = tokio::spawn(async move {
        while let Some(json) = ws_rx.recv().await {
            if sender.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    // Spawn task to forward monitor events to the WebSocket
    let registry_tx = registry.clone();
    let ws_tx_monitor = ws_tx.clone();
    let monitor_task = tokio::spawn(async move {
        while let Ok(event) = monitor_rx.recv().await {
            match event {
                MonitorEvent::Heartbeat(heartbeat) => {
                    let worker_name = heartbeat.worker.clone();
                    // Get current liveness for this worker
                    let liveness = registry_tx
                        .workers
                        .read()
                        .await
                        .iter()
                        .find(|w| w.worker == worker_name)
                        .map(|w| w.liveness)
                        .unwrap_or(WorkerLiveness::Dead);

                    registry_tx.update_worker(heartbeat, liveness).await;

                    let worker = registry_tx
                        .workers
                        .read()
                        .await
                        .iter()
                        .find(|w| w.worker == worker_name)
                        .cloned();

                    if let Some(w) = worker {
                        if let Ok(json) = serde_json::to_string(&WsEvent::worker_update(w)) {
                            let _ = ws_tx_monitor.send(json).await;
                        }
                    }
                }
                MonitorEvent::LivenessChange(transition) => {
                    let worker = registry_tx
                        .workers
                        .read()
                        .await
                        .iter()
                        .find(|w| w.worker == transition.worker)
                        .cloned();

                    if let Some(mut w) = worker {
                        w.liveness = transition.new_state;
                        registry_tx.update_worker(
                            WorkerHeartbeat {
                                ts: w.last_heartbeat,
                                worker: w.worker.clone(),
                                state: match &w.state {
                                    WorkerDisplayState::Executing { bead, adapter, .. } => {
                                        WorkerState::Executing {
                                            bead: bead.clone(),
                                            pid: 0,
                                            adapter: adapter.clone(),
                                        }
                                    }
                                    WorkerDisplayState::Idle { last_strand } => {
                                        WorkerState::Idle {
                                            last_strand: last_strand.clone(),
                                        }
                                    }
                                    WorkerDisplayState::Knot { reason } => {
                                        WorkerState::Knot {
                                            reason: reason.clone(),
                                        }
                                    }
                                },
                            },
                            transition.new_state,
                        )
                        .await;

                        if let Ok(json) = serde_json::to_string(&WsEvent::worker_update(w)) {
                            let _ = ws_tx_monitor.send(json).await;
                        }
                    }
                }
                MonitorEvent::Rotated => {
                    debug!("Log rotation detected, sending fresh snapshot");
                    let snapshot = registry_tx.snapshot().await;
                    if let Ok(json) = serde_json::to_string(&WsEvent::workers_snapshot(snapshot)) {
                        let _ = ws_tx_monitor.send(json).await;
                    }
                }
                MonitorEvent::Error(e) => {
                    warn!("Monitor error: {}", e);
                }
            }
        }
    });

    // Spawn task to forward bead events to the WebSocket
    let ws_tx_beads = ws_tx.clone();
    let beads_store = state.beads.clone();
    let bead_task = tokio::spawn(async move {
        while let Ok(_bead) = bead_rx.recv().await {
            let beads = beads_store.read().unwrap().clone();
            let bead_data: Vec<BeadData> = beads.iter().map(bead_to_data).collect();
            if let Ok(json) = serde_json::to_string(&WsEvent::beads_snapshot(bead_data)) {
                let _ = ws_tx_beads.send(json).await;
            }
        }
    });

    // Spawn task to forward stitch_created events to the WebSocket
    let ws_tx_stitch = ws_tx.clone();
    let stitch_task = tokio::spawn(async move {
        while let Ok(stitch_data) = stitch_rx.recv().await {
            if let Ok(json) = serde_json::to_string(&WsEvent::stitch_created(stitch_data)) {
                let _ = ws_tx_stitch.send(json).await;
            }
        }
    });

    // Spawn task to forward session events to the WebSocket
    let _registry_for_sessions = registry.clone();
    let session_task = tokio::spawn(async move {
        // Session events are handled in lib.rs via the session_tx broadcast;
        // this task is a placeholder for future per-connection session handling.
    });

    // Spawn task to forward config status events to the WebSocket
    let ws_tx_config = ws_tx.clone();
    let config_task = tokio::spawn(async move {
        while let Ok(status) = config_status_rx.recv().await {
            if let Ok(json) = serde_json::to_string(&WsEvent::config_status(status)) {
                let _ = ws_tx_config.send(json).await;
            }
        }
    });

    // Spawn task to forward project status events to the WebSocket
    let projects_for_update = state.projects.clone();
    let ws_tx_projects = ws_tx.clone();
    let project_task = tokio::spawn(async move {
        while let Ok(project_status) = project_status_rx.recv().await {
            {
                let mut projects = projects_for_update.write().unwrap();
                if let Some(project) = projects.iter_mut().find(|p| p.name == project_status.name) {
                    *project = project_status.clone();
                }
            }

            let projects = projects_for_update.read().unwrap().clone();
            if let Ok(json) = serde_json::to_string(&WsEvent::projects_snapshot(projects)) {
                let _ = ws_tx_projects.send(json).await;
            }
        }
    });

    // Spawn task to forward capacity events to the WebSocket
    let ws_tx_capacity = ws_tx.clone();
    let capacity_task = tokio::spawn(async move {
        loop {
            match capacity_rx.recv().await {
                Ok(capacities) => {
                    if let Ok(json) = serde_json::to_string(&WsEvent::capacity_snapshot(capacities)) {
                        let _ = ws_tx_capacity.send(json).await;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    debug!("Capacity broadcast lagged by {}, continuing", n);
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    // Spawn task to forward agent session events to the WebSocket
    let ws_tx_agent = ws_tx.clone();
    let agent_session_manager = state.agent_session_manager.clone();
    let agent_task = tokio::spawn(async move {
        let Some(mgr) = agent_session_manager else { return };
        let mut agent_rx = mgr.subscribe();
        loop {
            match agent_rx.recv().await {
                Ok(event) => {
                    if let Ok(json) = serde_json::to_string(&WsEvent::agent_session(event)) {
                        let _ = ws_tx_agent.send(json).await;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    debug!("Agent session broadcast lagged by {}, continuing", n);
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    // Spawn task to forward draft queue events to the WebSocket
    let ws_tx_draft = ws_tx.clone();
    let draft_task = tokio::spawn(async move {
        loop {
            match draft_rx.recv().await {
                Ok(data) => {
                    if let Ok(json) = serde_json::to_string(&WsEvent::draft_update(data)) {
                        let _ = ws_tx_draft.send(json).await;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    debug!("Draft queue broadcast lagged by {}, continuing", n);
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    // Spawn task to forward morning brief events to the WebSocket
    let ws_tx_brief = ws_tx.clone();
    let brief_task = tokio::spawn(async move {
        loop {
            match brief_rx.recv().await {
                Ok(data) => {
                    if let Ok(json) = serde_json::to_string(&WsEvent::morning_brief(data)) {
                        let _ = ws_tx_brief.send(json).await;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    debug!("Morning brief broadcast lagged by {}, continuing", n);
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    // Handle incoming messages (just ping/pong for now)
    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Ping(_msg)) => {
                    // Pong response is handled automatically by axum
                    debug!("Received ping");
                }
                Ok(Message::Close(_)) => {
                    debug!("Client requested close");
                    break;
                }
                Err(e) => {
                    warn!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    // Handle shutdown: send close frame when NotifyClients phase is received
    let ws_tx_shutdown = ws_tx.clone();
    let shutdown_task = tokio::spawn(async move {
        use crate::shutdown::ShutdownPhase;
        while let Ok(phase) = shutdown_rx.recv().await {
            if phase == ShutdownPhase::NotifyClients {
                debug!("Shutdown: sending close frame to WebSocket client");
                // Signal the forwarder to stop by dropping the sender
                drop(ws_tx_shutdown);
                break;
            }
        }
    });

    // Wait for any task to complete
    tokio::select! {
        _ = forwarder_task => {},
        _ = monitor_task => {},
        _ = bead_task => {},
        _ = stitch_task => {},
        _ = session_task => {},
        _ = config_task => {},
        _ = project_task => {},
        _ = capacity_task => {},
        _ = agent_task => {},
        _ = brief_task => {},
        _ = draft_task => {},
        _ = recv_task => {},
        _ = shutdown_task => {},
    }

    debug!("WebSocket connection closed");
}
