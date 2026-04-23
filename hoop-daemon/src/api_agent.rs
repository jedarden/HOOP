//! REST API endpoints for agent session lifecycle control.
//!
//! Routes:
//!   GET  /api/agent/status       — current session status
//!   POST /api/agent/spawn        — spawn a new session (or reattach)
//!   POST /api/agent/disable      — agent-off: cleanly disable
//!   POST /api/agent/switch       — switch adapter mid-stream
//!   POST /api/agent/turn         — send a user turn (streams events via WS)
//!   GET  /api/agent/sessions     — list recent sessions

use crate::agent_session::AgentAdapterConfig;
use crate::DaemonState;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

/// Build the agent API router.
pub fn router() -> Router<DaemonState> {
    Router::new()
        .route("/api/agent/status", get(get_status))
        .route("/api/agent/spawn", post(spawn_session))
        .route("/api/agent/disable", post(disable_agent))
        .route("/api/agent/switch", post(switch_adapter))
        .route("/api/agent/turn", post(send_turn))
        .route("/api/agent/sessions", get(list_sessions))
}

/// GET /api/agent/status
async fn get_status(
    State(state): State<DaemonState>,
) -> Json<crate::agent_session::AgentSessionStatus> {
    match &state.agent_session_manager {
        Some(mgr) => Json(mgr.status().await),
        None => Json(crate::agent_session::AgentSessionStatus {
            active: false,
            session_id: None,
            adapter: None,
            model: None,
            stitch_id: None,
            cost_usd: 0.0,
            input_tokens: 0,
            output_tokens: 0,
            turn_count: 0,
            created_at: None,
            last_activity_at: None,
            age_secs: None,
        }),
    }
}

/// POST /api/agent/spawn
async fn spawn_session(
    State(state): State<DaemonState>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let mgr = state
        .agent_session_manager
        .as_ref()
        .ok_or(axum::http::StatusCode::NOT_FOUND)?;

    match mgr.spawn().await {
        Ok(db_id) => Ok(Json(serde_json::json!({
            "status": "ok",
            "session_db_id": db_id,
        }))),
        Err(e) => {
            tracing::error!("Failed to spawn agent session: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// POST /api/agent/disable
async fn disable_agent(
    State(state): State<DaemonState>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let mgr = state
        .agent_session_manager
        .as_ref()
        .ok_or(axum::http::StatusCode::NOT_FOUND)?;

    match mgr.disable().await {
        Ok(()) => Ok(Json(serde_json::json!({
            "status": "ok",
            "message": "Agent disabled"
        }))),
        Err(e) => {
            tracing::error!("Failed to disable agent: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
struct SwitchRequest {
    adapter: String,
    model: Option<String>,
    #[serde(default)]
    anthropic_api_key: Option<String>,
    #[serde(default)]
    zai_base_url: Option<String>,
    #[serde(default)]
    zai_api_key: Option<String>,
}

/// POST /api/agent/switch
async fn switch_adapter(
    State(state): State<DaemonState>,
    Json(req): Json<SwitchRequest>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let mgr = state
        .agent_session_manager
        .as_ref()
        .ok_or(axum::http::StatusCode::NOT_FOUND)?;

    let new_config = AgentAdapterConfig {
        adapter: req.adapter,
        model: req.model.unwrap_or_else(|| "claude-opus-4-7".to_string()),
        anthropic_api_key: req.anthropic_api_key,
        zai_base_url: req.zai_base_url,
        zai_api_key: req.zai_api_key,
        rate_limit_rpm: None,
        cost_cap_usd: None,
    };

    match mgr.switch_adapter(new_config).await {
        Ok(db_id) => Ok(Json(serde_json::json!({
            "status": "ok",
            "session_db_id": db_id,
            "message": "Adapter switched, new session started"
        }))),
        Err(e) => {
            tracing::error!("Failed to switch adapter: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
struct TurnRequest {
    prompt: String,
}

/// POST /api/agent/turn
///
/// Sends a user turn to the active session. Events are streamed via the
/// WebSocket `agent_session` channel, not in the HTTP response. The response
/// confirms the turn was accepted.
async fn send_turn(
    State(state): State<DaemonState>,
    Json(req): Json<TurnRequest>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let mgr = state
        .agent_session_manager
        .as_ref()
        .ok_or(axum::http::StatusCode::NOT_FOUND)?;

    if !mgr.is_enabled() {
        return Err(axum::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    match mgr.send_turn(req.prompt, vec![]).await {
        Ok(mut stream) => {
            use futures_util::StreamExt;
            // Consume the stream, processing each event.
            // This runs synchronously to completion; for long-running turns
            // the WS channel gets events as they arrive.
            while let Some(item) = stream.next().await {
                match item {
                    Ok(event) => mgr.handle_event(&event).await,
                    Err(e) => {
                        tracing::warn!("Agent stream error: {}", e);
                        break;
                    }
                }
            }
            Ok(Json(serde_json::json!({
                "status": "ok",
                "message": "Turn completed"
            })))
        }
        Err(e) => {
            tracing::error!("Failed to send turn: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /api/agent/sessions
async fn list_sessions(
) -> Result<Json<Vec<crate::fleet::AgentSessionRow>>, axum::http::StatusCode> {
    match crate::fleet::list_agent_sessions(20) {
        Ok(rows) => Ok(Json(rows)),
        Err(e) => {
            tracing::error!("Failed to list agent sessions: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
