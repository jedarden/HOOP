//! WebSocket endpoint for real-time worker updates
//!
//! Broadcasts worker state changes, heartbeats, and liveness transitions
//! to connected web UI clients.

use crate::heartbeats::{MonitorEvent, WorkerHeartbeat, WorkerLiveness};
use crate::sessions::{SessionEvent, SessionTailer};
use crate::{Bead, BeadStatus as DaemonBeadStatus, BeadType as DaemonBeadType, DaemonState, WorkerState};
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
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, warn};

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
        Self {
            role: m.role,
            content: m.content,
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
}

impl From<ParsedSession> for ConversationData {
    fn from(s: ParsedSession) -> Self {
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

/// WebSocket event sent to clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub struct WsEvent {
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
}

impl WsEvent {
    /// Create a worker update event
    fn worker_update(worker: WorkerData) -> Self {
        Self {
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
        }
    }

    /// Create a full worker snapshot event
    fn workers_snapshot(workers: Vec<WorkerData>) -> Self {
        Self {
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
        }
    }

    /// Create a beads snapshot event
    fn beads_snapshot(beads: Vec<BeadData>) -> Self {
        Self {
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
        }
    }

    /// Create a conversations snapshot event
    fn conversations_snapshot(conversations: Vec<ConversationData>) -> Self {
        Self {
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
        }
    }

    /// Create a conversation update event
    fn conversation_update(conversation: ConversationData) -> Self {
        Self {
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
        }
    }

    /// Create a streaming content event
    fn streaming_content(data: StreamingContentData) -> Self {
        Self {
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
        }
    }

    /// Create a projects snapshot event
    pub fn projects_snapshot(projects: Vec<ProjectCardData>) -> Self {
        Self {
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
        }
    }

    /// Create a config status event
    pub fn config_status(status: ConfigStatusData) -> Self {
        Self {
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
        }
    }

    /// Create a capacity snapshot event
    pub fn capacity_snapshot(capacity: Vec<crate::capacity::AccountCapacity>) -> Self {
        Self {
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
        }
    }

    /// Create a bead event update (single event from events.jsonl)
    pub fn bead_event_update(event: BeadEventData) -> Self {
        Self {
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
        }
    }

    /// Create a bead events snapshot (all events for a bead)
    pub fn bead_events_snapshot(events: Vec<BeadEventData>) -> Self {
        Self {
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
        }
    }
}

/// Shared worker registry
#[derive(Debug, Clone)]
pub struct WorkerRegistry {
    workers: Arc<RwLock<Vec<WorkerData>>>,
    conversations: Arc<RwLock<Vec<ConversationData>>>,
    /// Bead events from events.jsonl, keyed by bead_id
    bead_events: Arc<RwLock<std::collections::HashMap<String, Vec<BeadEventData>>>>,
    monitor: broadcast::Sender<MonitorEvent>,
    sessions: broadcast::Sender<SessionEvent>,
}

impl WorkerRegistry {
    pub fn new(monitor: broadcast::Sender<MonitorEvent>, sessions: broadcast::Sender<SessionEvent>) -> Self {
        Self {
            workers: Arc::new(RwLock::new(Vec::new())),
            conversations: Arc::new(RwLock::new(Vec::new())),
            bead_events: Arc::new(RwLock::new(std::collections::HashMap::new())),
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

    /// Update or insert a worker entry
    pub async fn update_worker(&self, heartbeat: crate::heartbeats::WorkerHeartbeat, liveness: crate::heartbeats::WorkerLiveness) {
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
    // Register this connection with the shutdown coordinator for tracking
    let _conn_token = state.shutdown.register_connection();

    let (mut sender, mut receiver) = socket.split();
    let registry = state.worker_registry.clone();
    let mut monitor_rx = registry.subscribe();
    let mut bead_rx = state.bead_tx.subscribe();
    let mut session_rx = registry.subscribe_sessions();
    let mut config_status_rx = state.config_status_tx.subscribe();
    let mut project_status_rx = state.project_status_tx.subscribe();
    let mut capacity_rx = state.capacity_tx.subscribe();
    let mut shutdown_rx = state.shutdown.subscribe();

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

    // Spawn task to forward monitor events to the WebSocket
    let registry_tx = registry.clone();
    let monitor_task = tokio::spawn(async move {
        while let Ok(event) = monitor_rx.recv().await {
            match event {
                MonitorEvent::Heartbeat(heartbeat) => {
                    // Get current liveness for this worker
                    let liveness = registry_tx
                        .workers
                        .read()
                        .await
                        .iter()
                        .find(|w| w.worker == heartbeat.worker)
                        .map(|w| w.liveness)
                        .unwrap_or(WorkerLiveness::Dead);

                    registry_tx.update_worker(heartbeat, liveness).await;

                    let worker = registry_tx
                        .workers
                        .read()
                        .await
                        .iter()
                        .find(|w| w.worker == heartbeat.worker)
                        .cloned();

                    if let Some(w) = worker {
                        if let Ok(json) = serde_json::to_string(&WsEvent::worker_update(w)) {
                            if sender.send(Message::Text(json)).await.is_err() {
                                break;
                            }
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
                        // Update the stored worker
                        registry_tx.update_worker(
                            WorkerHeartbeat {
                                ts: w.last_heartbeat,
                                worker: w.worker.clone(),
                                state: match &w.state {
                                    WorkerDisplayState::Executing { bead, adapter, .. } => {
                                        WorkerState::Executing {
                                            bead: bead.clone(),
                                            pid: 0, // PID not tracked here
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
                            if sender.send(Message::Text(json)).await.is_err() {
                                break;
                            }
                        }
                    }
                }
                MonitorEvent::Rotated => {
                    debug!("Log rotation detected, sending fresh snapshot");
                    let snapshot = registry_tx.snapshot().await;
                    if let Ok(json) = serde_json::to_string(&WsEvent::workers_snapshot(snapshot)) {
                        if sender.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                }
                MonitorEvent::Error(e) => {
                    warn!("Monitor error: {}", e);
                }
            }
        }
    });

    // Spawn task to forward bead events to the WebSocket
    let bead_task = tokio::spawn(async move {
        while let Ok(_bead) = bead_rx.recv().await {
            // Send full beads snapshot on any bead change
            // (In the future, could optimize to send only changed bead)
            let beads = state.beads.read().unwrap().clone();
            let bead_data: Vec<BeadData> = beads.iter().map(bead_to_data).collect();
            if let Ok(json) = serde_json::to_string(&WsEvent::beads_snapshot(bead_data)) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Spawn task to forward session events to the WebSocket
    let registry_for_sessions = registry.clone();
    let session_task = tokio::spawn(async move {
        while let Ok(event) = session_rx.recv().await {
            match event {
                SessionEvent::ConversationsUpdated { sessions } => {
                    // Update registry and send snapshot
                    registry_for_sessions.update_conversations(sessions.clone()).await;
                    let data: Vec<ConversationData> = sessions.into_iter().map(ConversationData::from).collect();
                    if let Ok(json) = serde_json::to_string(&WsEvent::conversations_snapshot(data)) {
                        if sender.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                }
                SessionEvent::SessionBound { id, file_path: _ } => {
                    // Send update for the bound session
                    let convos = registry_for_sessions.conversations_snapshot().await;
                    if let Some(convo) = convos.iter().find(|c| c.id == id) {
                        if let Ok(json) = serde_json::to_string(&WsEvent::conversation_update(convo.clone())) {
                            if sender.send(Message::Text(json)).await.is_err() {
                                break;
                            }
                        }
                    }
                }
                SessionEvent::Error(e) => {
                    warn!("Session error: {}", e);
                }
            }
        }
    });

    // Spawn task to forward config status events to the WebSocket
    let config_task = tokio::spawn(async move {
        while let Ok(status) = config_status_rx.recv().await {
            if let Ok(json) = serde_json::to_string(&WsEvent::config_status(status)) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Spawn task to forward project status events to the WebSocket
    let projects_for_update = state.projects.clone();
    let project_task = tokio::spawn(async move {
        while let Ok(project_status) = project_status_rx.recv().await {
            // Update the project in the projects store
            {
                let mut projects = projects_for_update.write().unwrap();
                if let Some(project) = projects.iter_mut().find(|p| p.name == project_status.name) {
                    *project = project_status.clone();
                }
            }

            // Send updated projects snapshot
            let projects = projects_for_update.read().unwrap().clone();
            if let Ok(json) = serde_json::to_string(&WsEvent::projects_snapshot(projects)) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Spawn task to forward capacity events to the WebSocket
    let capacity_task = tokio::spawn(async move {
        loop {
            match capacity_rx.recv().await {
                Ok(capacities) => {
                    if let Ok(json) = serde_json::to_string(&WsEvent::capacity_snapshot(capacities)) {
                        if sender.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    debug!("Capacity broadcast lagged by {}, continuing", n);
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    // Handle incoming messages (just ping/pong for now)
    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Ping(msg)) => {
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
    let shutdown_task = tokio::spawn(async move {
        use crate::shutdown::ShutdownPhase;
        while let Ok(phase) = shutdown_rx.recv().await {
            if phase == ShutdownPhase::NotifyClients {
                debug!("Shutdown: sending close frame to WebSocket client");
                // Send a close frame with normal closure (1000)
                let _ = sender.send(Message::Close(Some(axum::extract::ws::CloseFrame {
                    code: axum::extract::ws::close_code::NORMAL,
                    reason: "Server shutting down".into(),
                }))).await;
                break;
            }
        }
    });

    // Wait for any task to complete
    tokio::select! {
        _ = monitor_task => {},
        _ = bead_task => {},
        _ = session_task => {},
        _ = config_task => {},
        _ = project_task => {},
        _ = capacity_task => {},
        _ = recv_task => {},
    }

    debug!("WebSocket connection closed");
}
