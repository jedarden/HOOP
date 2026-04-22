//! WebSocket endpoint for real-time worker updates
//!
//! Broadcasts worker state changes, heartbeats, and liveness transitions
//! to connected web UI clients.

use crate::heartbeats::{MonitorEvent, WorkerHeartbeat, WorkerLiveness};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use futures_util::{stream::StreamExt, SinkExt};
use hoop_schema::{Bead, WorkerState};
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
}

impl WsEvent {
    /// Create a worker update event
    fn worker_update(worker: WorkerData) -> Self {
        Self {
            worker: Some(worker),
            workers: None,
            beads: None,
        }
    }

    /// Create a full worker snapshot event
    fn workers_snapshot(workers: Vec<WorkerData>) -> Self {
        Self {
            worker: None,
            workers: Some(workers),
            beads: None,
        }
    }

    /// Create a beads snapshot event
    fn beads_snapshot(beads: Vec<BeadData>) -> Self {
        Self {
            worker: None,
            workers: None,
            beads: Some(beads),
        }
    }
}

/// Shared worker registry
#[derive(Debug, Clone)]
pub struct WorkerRegistry {
    workers: Arc<RwLock<Vec<WorkerData>>>,
    monitor: broadcast::Sender<MonitorEvent>,
}

impl WorkerRegistry {
    pub fn new(monitor: broadcast::Sender<MonitorEvent>) -> Self {
        Self {
            workers: Arc::new(RwLock::new(Vec::new())),
            monitor,
        }
    }

    /// Get current snapshot of all workers
    pub async fn snapshot(&self) -> Vec<WorkerData> {
        self.workers.read().await.clone()
    }

    /// Update a single worker's data
    pub async fn update_worker(&self, heartbeat: WorkerHeartbeat, liveness: WorkerLiveness) {
        let mut workers = self.workers.write().await;
        let now = Utc::now();
        let heartbeat_age = now.signed_duration_since(heartbeat.ts).num_seconds();

        let display_state = match &heartbeat.state {
            WorkerState::Executing { bead, adapter, .. } => WorkerDisplayState::Executing {
                bead: bead.clone(),
                adapter: adapter.clone(),
                model: None, // TODO: extract from dispatch events
            },
            WorkerState::Idle { last_strand } => WorkerDisplayState::Idle {
                last_strand: last_strand.clone(),
            },
            WorkerState::Knot { reason } => WorkerDisplayState::Knot {
                reason: reason.clone(),
            },
        };

        let worker_data = WorkerData {
            worker: heartbeat.worker.clone(),
            state: display_state,
            liveness,
            last_heartbeat: heartbeat.ts,
            heartbeat_age_secs: heartbeat_age,
        };

        // Update or insert
        if let Some(existing) = workers.iter_mut().find(|w| w.worker == heartbeat.worker) {
            *existing = worker_data;
        } else {
            workers.push(worker_data);
        }
    }

    /// Remove a worker (when it's been dead for a while)
    pub async fn remove_worker(&self, worker_name: &str) {
        let mut workers = self.workers.write().await;
        workers.retain(|w| w.worker != worker_name);
    }

    /// Subscribe to monitor events
    pub fn subscribe(&self) -> broadcast::Receiver<MonitorEvent> {
        self.monitor.subscribe()
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
        _ = recv_task => {},
    }

    debug!("WebSocket connection closed");
}
