//! WebSocket endpoint for real-time worker updates
//!
//! Broadcasts worker state changes, heartbeats, and liveness transitions
//! to connected web UI clients.

use crate::heartbeats::{MonitorEvent, WorkerHeartbeat, WorkerLiveness};
use crate::sessions::{SessionEvent, SessionTailer};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use futures_util::{stream::StreamExt, SinkExt};
use hoop_schema::{Bead, MessageUsage, ParsedSession, SessionKind, SessionMessage, WorkerState};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, warn};

use crate::DaemonState;

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
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
}

impl From<MessageUsage> for MessageUsageData {
    fn from(u: MessageUsage) -> Self {
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

impl From<SessionMessage> for SessionMessageData {
    fn from(m: SessionMessage) -> Self {
        Self {
            role: m.role,
            content: m.content,
            usage: m.usage.map(MessageUsageData::from),
            timestamp: m.timestamp,
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
            SessionKind::Operator => (None, "operator".to_string()),
            SessionKind::Dictated => (None, "dictated".to_string()),
            SessionKind::Worker { worker, bead, strand } => (
                Some(WorkerMetadataData {
                    worker: worker.clone(),
                    bead: bead.clone(),
                    strand: strand.clone(),
                }),
                "worker".to_string(),
            ),
            SessionKind::AdHoc => (None, "ad-hoc".to_string()),
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
            total_tokens: s.total_tokens(),
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
        }
    }
}

/// Shared worker registry
#[derive(Debug, Clone)]
pub struct WorkerRegistry {
    workers: Arc<RwLock<Vec<WorkerData>>>,
    conversations: Arc<RwLock<Vec<ConversationData>>>,
    monitor: broadcast::Sender<MonitorEvent>,
    sessions: broadcast::Sender<SessionEvent>,
}

impl WorkerRegistry {
    pub fn new(monitor: broadcast::Sender<MonitorEvent>, sessions: broadcast::Sender<SessionEvent>) -> Self {
        Self {
            workers: Arc::new(RwLock::new(Vec::new())),
            conversations: Arc::new(RwLock::new(Vec::new())),
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
    let (mut sender, mut receiver) = socket.split();
    let registry = state.worker_registry.clone();
    let mut monitor_rx = registry.subscribe();
    let mut bead_rx = state.bead_tx.subscribe();
    let mut session_rx = registry.subscribe_sessions();

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

    // Wait for any task to complete
    tokio::select! {
        _ = monitor_task => {},
        _ = bead_task => {},
        _ = session_task => {},
        _ = recv_task => {},
    }

    debug!("WebSocket connection closed");
}
