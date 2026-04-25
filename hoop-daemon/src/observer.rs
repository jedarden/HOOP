//! Observer mode - read-only attach to primary daemon
//!
//! The observer runs a second HTTP/WebSocket server on a different port,
//! forwarding read requests to the primary daemon and broadcasting events
//! to its own WebSocket clients.

use axum::extract::{State, WebSocketUpgrade, ws::WebSocket};
use crate::ws::WsEvent;
use anyhow::Result;
use futures_util::{stream::StreamExt, SinkExt};
use serde_json::Value;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, RwLock};
use tokio_tungstenite::{tungstenite::Message as TungsteniteMessage, WebSocketStream};
use tracing::{debug, error, info, warn};

/// Observer client that connects to primary daemon's WebSocket endpoint
pub struct ObserverClient {
    primary_addr: SocketAddr,
    event_tx: broadcast::Sender<WsEvent>,
    beads: Arc<RwLock<Vec<crate::Bead>>>,
    workers: Arc<RwLock<Vec<crate::ws::WorkerData>>>,
    projects: Arc<RwLock<Vec<crate::ws::ProjectCardData>>>,
}

impl ObserverClient {
    /// Create a new observer client
    pub fn new(
        primary_addr: SocketAddr,
        event_tx: broadcast::Sender<WsEvent>,
        beads: Arc<RwLock<Vec<crate::Bead>>>,
        workers: Arc<RwLock<Vec<crate::ws::WorkerData>>>,
        projects: Arc<RwLock<Vec<crate::ws::ProjectCardData>>>,
    ) -> Self {
        Self {
            primary_addr,
            event_tx,
            beads,
            workers,
            projects,
        }
    }

    /// Connect to primary daemon and start forwarding events
    pub async fn run(self) -> Result<()> {
        let primary_url = format!("ws://{}/ws", self.primary_addr);
        info!("Observer connecting to primary at {}", primary_url);

        let (ws_stream, _) = tokio_tungstenite::connect_async(&primary_url).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // Subscribe to all events (global + per-project)
        let subscribe_msg = r#"{"type":"subscribe","topic":"global"}"#;
        ws_sender.send(Message::Text(subscribe_msg.to_string())).await?;
        debug!("Observer subscribed to global events");

        // Forward events from primary to observer's clients
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    debug!("Observer received event from primary: {}", text.chars().take(100).collect::<String>());

                    // Parse the event and update local state
                    if let Ok(event) = serde_json::from_str::<serde_json::Value>(&text) {
                        if let Some(event_type) = event.get("type").and_then(|v| v.as_str()) {
                            match event_type {
                                "init" => {
                                    // Handle initial snapshot
                                    if let Some(workers) = event.get("workers").and_then(|v| v.as_array()) {
                                        if let Ok(parsed) = serde_json::from_value::<Vec<crate::ws::WorkerData>>(
                                            serde_json::json!(workers)
                                        ) {
                                            *self.workers.write().await = parsed;
                                            info!("Observer initialized with {} workers", self.workers.read().await.len());
                                        }
                                    }
                                    if let Some(beads) = event.get("beads").and_then(|v| v.as_array()) {
                                        if let Ok(parsed) = serde_json::from_value::<Vec<crate::Bead>>(
                                            serde_json::json!(beads)
                                        ) {
                                            *self.beads.write().await = parsed;
                                            info!("Observer initialized with {} beads", self.beads.read().await.len());
                                        }
                                    }
                                    if let Some(projects) = event.get("projects").and_then(|v| v.as_array()) {
                                        if let Ok(parsed) = serde_json::from_value::<Vec<crate::ws::ProjectCardData>>(
                                            serde_json::json!(projects)
                                        ) {
                                            *self.projects.write().await = parsed;
                                            info!("Observer initialized with {} projects", self.projects.read().await.len());
                                        }
                                    }
                                }
                                "workers_snapshot" => {
                                    if let Some(workers) = event.get("workers").and_then(|v| v.as_array()) {
                                        if let Ok(parsed) = serde_json::from_value::<Vec<crate::ws::WorkerData>>(
                                            serde_json::json!(workers)
                                        ) {
                                            *self.workers.write().await = parsed;
                                        }
                                    }
                                }
                                "beads_snapshot" => {
                                    if let Some(beads) = event.get("beads").and_then(|v| v.as_array()) {
                                        if let Ok(parsed) = serde_json::from_value::<Vec<crate::Bead>>(
                                            serde_json::json!(beads)
                                        ) {
                                            *self.beads.write().await = parsed;
                                        }
                                    }
                                }
                                "projects_snapshot" => {
                                    if let Some(projects) = event.get("projects").and_then(|v| v.as_array()) {
                                        if let Ok(parsed) = serde_json::from_value::<Vec<crate::ws::ProjectCardData>>(
                                            serde_json::json!(projects)
                                        ) {
                                            *self.projects.write().await = parsed;
                                        }
                                    }
                                }
                                "worker_update" => {
                                    if let Some(worker) = event.get("worker") {
                                        if let Ok(parsed) = serde_json::from_value::<crate::ws::WorkerData>(
                                            serde_json::json!(worker)
                                        ) {
                                            let mut workers = self.workers.write().await;
                                            if let Some(existing) = workers.iter_mut().find(|w| w.worker == parsed.worker) {
                                                *existing = parsed;
                                            } else {
                                                workers.push(parsed);
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    // Forward all other events to observer clients
                                    if let Ok(ws_event) = serde_json::from_str::<WsEvent>(&text) {
                                            let _ = self.event_tx.send(ws_event);
                                    }
                                }
                            }

                            // Always forward the raw event to observer clients
                            if let Ok(ws_event) = serde_json::from_str::<WsEvent>(&text) {
                                let _ = self.event_tx.send(ws_event);
                            }
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    warn!("Primary closed WebSocket connection");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error from primary: {}", e);
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

/// HTTP client for making read requests to primary daemon
#[derive(Clone)]
pub struct ObserverHttpClient {
    pub primary_addr: SocketAddr,
    pub client: reqwest::Client,
}

impl ObserverHttpClient {
    /// Create a new HTTP client for observer mode
    pub fn new(primary_addr: SocketAddr) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self {
            primary_addr,
            client,
        }
    }

    /// Proxy a GET request to the primary daemon
    pub async fn proxy_get(&self, path: &str) -> Result<reqwest::Response> {
        let url = format!("http://{}{}", self.primary_addr, path);
        Ok(self.client.get(&url).send().await?)
    }

    /// Get beads from primary daemon
    pub async fn get_beads(&self) -> Result<Vec<crate::Bead>> {
        let url = format!("http://{}/api/beads", self.primary_addr);
        let resp = self.client.get(&url).send().await?;
        Ok(resp.json().await?)
    }

    /// Get project cards from primary daemon
    pub async fn get_projects(&self) -> Result<Vec<crate::ws::ProjectCardData>> {
        let url = format!("http://{}/api/dashboard/cross-project?range=today", self.primary_addr);
        let resp = self.client.get(&url).send().await?;

        // Parse the dashboard response to extract project data
        let dashboard: Value = resp.json().await?;

        // For now, return empty vector - projects come through WebSocket
        Ok(vec![])
    }

    /// Get bead events from primary daemon
    pub async fn get_bead_events(&self, bead_id: &str) -> Result<Vec<crate::ws::BeadEventData>> {
        let url = format!("http://{}/api/beads/{}/events", self.primary_addr, bead_id);
        let resp = self.client.get(&url).send().await?;
        Ok(resp.json().await?)
    }

    /// Get cost buckets from primary daemon
    pub async fn get_cost_buckets(&self) -> Result<Vec<crate::cost::CostBucket>> {
        let url = format!("http://{}/api/cost/buckets", self.primary_addr);
        let resp = self.client.get(&url).send().await?;
        Ok(resp.json().await?)
    }

    /// Get cost buckets by project from primary daemon
    pub async fn get_cost_buckets_by_project(&self, project: &str) -> Result<Vec<crate::cost::CostBucket>> {
        let url = format!("http://{}/api/cost/buckets/{}", self.primary_addr, project);
        let resp = self.client.get(&url).send().await?;
        Ok(resp.json().await?)
    }

    /// Get workers snapshot from primary daemon
    pub async fn get_workers(&self) -> Result<Vec<crate::ws::WorkerData>> {
        // This will come through the WebSocket connection
        Ok(vec![])
    }

    /// Get conversations from primary daemon
    pub async fn get_conversations(&self) -> Result<Vec<crate::ws::ConversationData>> {
        // This will come through the WebSocket connection
        Ok(vec![])
    }
}

/// Observer state shared across request handlers
#[derive(Debug, Clone)]
pub struct ObserverState {
    pub primary_addr: SocketAddr,
    pub http_client: ObserverHttpClient,
    pub beads: Arc<RwLock<Vec<crate::Bead>>>,
    pub workers: Arc<RwLock<Vec<crate::ws::WorkerData>>>,
    pub projects: Arc<RwLock<Vec<crate::ws::ProjectCardData>>>,
    pub event_tx: broadcast::Sender<WsEvent>,
    pub started_at: std::time::Instant,
}

/// WebSocket handler for observer mode
pub async fn observer_ws_handler(
    ws: WebSocket,
    state: ObserverState,
) {
    let (mut ws_sender, mut ws_receiver) = ws.split();

    // Send initial state
    let init_subs = vec!["global".to_string()];
    let workers = state.workers.read().await.clone();
    let beads = state.beads.read().await.clone();
    let projects = state.projects.read().await.clone();

    let _ = ws_sender.send(Message::Text(serde_json::to_string(&WsEvent::init(init_subs)).unwrap())).await;
    let _ = ws_sender.send(Message::Text(serde_json::to_string(&WsEvent::workers_snapshot(workers)).unwrap())).await;
    let _ = ws_sender.send(Message::Text(serde_json::to_string(&WsEvent::beads_snapshot_from_beads(&beads)).unwrap())).await;
    let _ = ws_sender.send(Message::Text(serde_json::to_string(&WsEvent::projects_snapshot(projects)).unwrap())).await;

    // Subscribe to events
    let mut event_rx = state.event_tx.subscribe();

    // Handle incoming messages and broadcast events
    loop {
        tokio::select! {
            // Forward events from primary to observer client
            result = event_rx.recv() => {
                match result {
                    Ok(event) => {
                        if let Ok(json) = serde_json::to_string(&event) {
                            if ws_sender.send(Message::Text(json)).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        warn!("Observer WS client lagged, skipping events");
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
            // Handle client messages (subscribe/unsubscribe)
            result = ws_receiver.next() => {
                match result {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(msg) = serde_json::from_str::<crate::ws::ClientMessage>(&text) {
                            match msg {
                                crate::ws::ClientMessage::Subscribe { .. } | crate::ws::ClientMessage::Unsubscribe { .. } => {
                                    // In observer mode, we just forward subscriptions to the primary
                                    // For now, we ignore them since we already subscribe to global
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | Some(Err(_)) => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Serve the observer mode HTTP/WebSocket server
pub async fn serve_observer(config: crate::Config) -> Result<()> {
    log_rotation::init_logging();

    info!("HOOP observer mode starting");
    info!("Connecting to primary daemon at {}", config.primary_addr);
    info!("Observer UI will be available at http://{}", config.bind_addr);

    // Initialize shared state
    let beads: Arc<RwLock<Vec<crate::Bead>>> = Arc::new(RwLock::new(Vec::new()));
    let workers: Arc<RwLock<Vec<crate::ws::WorkerData>>> = Arc::new(RwLock::new(Vec::new()));
    let projects: Arc<RwLock<Vec<crate::ws::ProjectCardData>>> = Arc::new(RwLock::new(Vec::new()));

    let (event_tx, _) = broadcast::channel::<WsEvent>(256);

    let http_client = ObserverHttpClient::new(config.primary_addr);

    let state = ObserverState {
        primary_addr: config.primary_addr,
        http_client: http_client.clone(),
        beads: beads.clone(),
        workers: workers.clone(),
        projects: projects.clone(),
        event_tx: event_tx.clone(),
        started_at: std::time::Instant::now(),
    };

    // Start the observer client (connects to primary and forwards events)
    let observer_client = ObserverClient::new(
        config.primary_addr,
        event_tx.clone(),
        beads.clone(),
        workers.clone(),
        projects.clone(),
    );

    tokio::spawn(async move {
        if let Err(e) = observer_client.run().await {
            error!("Observer client error: {}", e);
        }
    });

    // Build the observer router
    let app = observer_router()
        .with_state(state.clone())
        .into_make_service_with_connect_info::<SocketAddr>();

    // Start the HTTP server
    let listener = TcpListener::bind(config.bind_addr).await?;
    info!("HOOP observer listening on http://{}", config.bind_addr);

    // Set up signal handling
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>;

    // Run the server with graceful shutdown
    tokio::select! {
        result = axum::serve(listener, app) => {
            result.map_err(|e| anyhow::anyhow!(e))
        }
        _ = ctrl_c => {
            info!("Received SIGINT (Ctrl-C), shutting down observer");
            Ok(())
        }
        _ = terminate => {
            info!("Received SIGTERM, shutting down observer");
            Ok(())
        }
    }
}

/// Build the observer router with read-only endpoints
pub fn observer_router() -> axum::Router<ObserverState> {
    use axum::{routing::get, Router};

    Router::new()
        .route("/healthz", get(observer_healthz))
        .route("/api/beads", get(observer_get_beads))
        .route("/api/cost/buckets", get(observer_get_cost_buckets))
        .route("/api/cost/buckets/:project", get(observer_get_cost_buckets_by_project))
        .route("/api/dashboard/cross-project", get(observer_get_dashboard))
        .route("/api/beads/:bead_id/events", get(observer_get_bead_events))
        .route("/ws", get(observer_ws_upgrade))
        .nest_service("/assets", hoop_ui::AssetsHandler::router())
        .fallback_service(hoop_ui::AssetsHandler::router())
        .layer(tower_http::trace::TraceLayer::new_for_http())
}

/// Health check for observer
async fn observer_healthz() -> axum::Json<hoop_schema::HealthResponse> {
    axum::Json(hoop_schema::HealthResponse::ok())
}

/// Get beads (from local cache, updated via WebSocket)
async fn observer_get_beads(
    State(state): State<ObserverState>,
) -> axum::Json<Vec<crate::Bead>> {
    let beads = state.beads.read().await.clone();
    axum::Json(beads)
}

/// Get cost buckets (proxied to primary)
async fn observer_get_cost_buckets(
    State(state): State<ObserverState>,
) -> Result<axum::Json<Vec<crate::cost::CostBucket>>, (axum::http::StatusCode, String)> {
    state
        .http_client
        .get_cost_buckets()
        .await
        .map(axum::Json)
        .map_err(|e| (axum::http::StatusCode::BAD_GATEWAY, e.to_string()))
}

/// Get cost buckets by project (proxied to primary)
async fn observer_get_cost_buckets_by_project(
    axum::extract::Path(project): axum::extract::Path<String>,
    State(state): State<ObserverState>,
) -> Result<axum::Json<Vec<crate::cost::CostBucket>>, (axum::http::StatusCode, String)> {
    state
        .http_client
        .get_cost_buckets_by_project(&project)
        .await
        .map(axum::Json)
        .map_err(|e| (axum::http::StatusCode::BAD_GATEWAY, e.to_string()))
}

/// Get dashboard data (proxied to primary)
async fn observer_get_dashboard(
    State(state): State<ObserverState>,
) -> Result<axum::Json<crate::CrossProjectDashboardResponse>, (axum::http::StatusCode, String)> {
    let url = format!("http://{}/api/dashboard/cross-project?range=today", state.primary_addr);
    let resp = state
        .http_client
        .client
        .get(&url)
        .send()
        .await
        .map_err(|e| (axum::http::StatusCode::BAD_GATEWAY, e.to_string()))?;

    let json: crate::CrossProjectDashboardResponse = resp
        .json()
        .await
        .map_err(|e| (axum::http::StatusCode::BAD_GATEWAY, e.to_string()))?;

    Ok(axum::Json(json))
}

/// Get bead events (proxied to primary)
async fn observer_get_bead_events(
    axum::extract::Path(bead_id): axum::extract::Path<String>,
    State(state): State<ObserverState>,
) -> Result<axum::Json<Vec<crate::ws::BeadEventData>>, (axum::http::StatusCode, String)> {
    state
        .http_client
        .get_bead_events(&bead_id)
        .await
        .map(axum::Json)
        .map_err(|e| (axum::http::StatusCode::BAD_GATEWAY, e.to_string()))
}

/// WebSocket upgrade handler for observer
async fn observer_ws_upgrade(
    ws: axum::extract::WebSocketUpgrade,
    State(state): State<ObserverState>,
) -> axum::response::Response {
    ws.on_upgrade(|socket| observer_ws_handler(socket, state))
}
