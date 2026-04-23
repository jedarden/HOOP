//! Agent session lifecycle: spawn / persist / attach-on-restart / resume-on-adapter-switch
//!
//! The `AgentSessionManager` owns the live agent session. It persists session
//! state to `fleet.db` on every mutation so that a `systemctl --user restart
//! hoop` can reattach. Adapter switch archives the old session (optionally as
//! a Stitch) and starts fresh with the Reflection Ledger carried forward.

use crate::agent_adapter::{
    self, AdapterKind, AgentAdapter, AgentEvent, AgentSession, Attachment, EventStream,
    SpawnConfig, SessionId,
};
use crate::fleet;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tracing::{info, warn};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Snapshot of the current (or absent) agent session, sent to WS clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSessionStatus {
    pub active: bool,
    pub session_id: Option<String>,
    pub adapter: Option<String>,
    pub model: Option<String>,
    pub stitch_id: Option<String>,
    pub cost_usd: f64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub turn_count: i64,
    pub created_at: Option<String>,
    pub last_activity_at: Option<String>,
    pub age_secs: Option<i64>,
}

/// Events broadcast to WS clients about the agent session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentSessionEvent {
    /// A new session was spawned.
    SessionSpawned {
        session_id: String,
        adapter: String,
        model: String,
    },
    /// Session reattached after daemon restart.
    SessionReattached {
        session_id: String,
        adapter: String,
        model: String,
    },
    /// A streaming text delta from the model.
    TextDelta {
        session_id: String,
        text: String,
    },
    /// A tool invocation.
    ToolUse {
        session_id: String,
        id: String,
        name: String,
        input: serde_json::Value,
    },
    /// Tool returned a result.
    ToolResult {
        session_id: String,
        id: String,
        output: serde_json::Value,
        is_error: bool,
    },
    /// A turn completed.
    TurnComplete {
        session_id: String,
        cost_usd: f64,
        input_tokens: i64,
        output_tokens: i64,
    },
    /// Session archived (adapter switch, agent-off, or error).
    SessionArchived {
        session_id: String,
        reason: String,
    },
    /// Error inside the adapter.
    Error {
        session_id: String,
        message: String,
    },
}

/// The internal state held behind the Mutex.
struct Inner {
    adapter: Box<dyn AgentAdapter>,
    adapter_kind: AdapterKind,
    session: Option<AgentSession>,
    db_row_id: Option<String>,
    config: AgentAdapterConfig,
}

impl std::fmt::Debug for Inner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inner")
            .field("adapter_kind", &self.adapter_kind)
            .field("session", &self.session)
            .field("db_row_id", &self.db_row_id)
            .field("config", &self.config)
            .finish()
    }
}

/// Config loaded from `~/.hoop/config.yml` agent section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAdapterConfig {
    pub adapter: String,
    pub model: String,
    #[serde(default)]
    pub anthropic_api_key: Option<String>,
    #[serde(default)]
    pub zai_base_url: Option<String>,
    #[serde(default)]
    pub zai_api_key: Option<String>,
    #[serde(default)]
    pub rate_limit_rpm: Option<u32>,
    #[serde(default)]
    pub cost_cap_usd: Option<f64>,
}

impl Default for AgentAdapterConfig {
    fn default() -> Self {
        Self {
            adapter: "claude".to_string(),
            model: "claude-opus-4-7".to_string(),
            anthropic_api_key: None,
            zai_base_url: None,
            zai_api_key: None,
            rate_limit_rpm: None,
            cost_cap_usd: None,
        }
    }
}

impl From<&AgentAdapterConfig> for agent_adapter::AgentAdapterConfig {
    fn from(c: &AgentAdapterConfig) -> Self {
        Self {
            adapter: c.adapter.clone(),
            model: c.model.clone(),
            anthropic_api_key: c.anthropic_api_key.clone(),
            zai_base_url: c.zai_base_url.clone(),
            zai_api_key: c.zai_api_key.clone(),
            rate_limit_rpm: c.rate_limit_rpm,
            cost_cap_usd: c.cost_cap_usd,
        }
    }
}

// ---------------------------------------------------------------------------
// Manager
// ---------------------------------------------------------------------------

/// Manages the lifecycle of the single human-interface agent session.
#[derive(Debug)]
pub struct AgentSessionManager {
    inner: Arc<Mutex<Inner>>,
    event_tx: broadcast::Sender<AgentSessionEvent>,
    enabled: std::sync::atomic::AtomicBool,
}

impl AgentSessionManager {
    /// Create a new manager. Attempts to reattach to an existing active
    /// session from fleet.db; if none, the manager starts idle (no session).
    pub async fn new(config: AgentAdapterConfig) -> Result<Self> {
        let adapter_config: agent_adapter::AgentAdapterConfig = (&config).into();
        let adapter_kind = AdapterKind::from_config(&config.adapter)
            .ok_or_else(|| anyhow::anyhow!("unknown agent adapter: {}", config.adapter))?;
        let adapter = agent_adapter::build_adapter(&adapter_config)?;

        // Try to reattach to an existing active session from fleet.db.
        let mut db_row_id = None;
        let mut session = None;

        if let Some(row) = fleet::load_active_agent_session()? {
            info!(
                "Found active agent session {} (adapter={}, model={}), reattaching",
                row.adapter_session_id, row.adapter, row.model
            );
            let sid = SessionId(row.adapter_session_id.clone());
            match adapter.resume_session(&sid).await {
                Ok(s) => {
                    session = Some(s);
                    db_row_id = Some(row.id.clone());
                    info!("Reattached to agent session {}", row.adapter_session_id);
                }
                Err(e) => {
                    warn!(
                        "Failed to reattach session {}: {}. Archiving.",
                        row.adapter_session_id, e
                    );
                    let _ = fleet::archive_agent_session(&row.id, "reattach_failed");
                }
            }
        }

        let event_tx = broadcast::channel::<AgentSessionEvent>(256).0;
        let enabled = std::sync::atomic::AtomicBool::new(true);

        Ok(Self {
            inner: Arc::new(Mutex::new(Inner {
                adapter,
                adapter_kind,
                session,
                db_row_id,
                config,
            })),
            event_tx,
            enabled,
        })
    }

    /// Subscribe to agent session events (for WS forwarding).
    pub fn subscribe(&self) -> broadcast::Receiver<AgentSessionEvent> {
        self.event_tx.subscribe()
    }

    /// Current status snapshot (for WS initial snapshot / REST status).
    pub async fn status(&self) -> AgentSessionStatus {
        let inner = self.inner.lock().await;
        match (&inner.session, &inner.db_row_id) {
            (Some(s), Some(db_id)) => {
                let row = fleet::load_active_agent_session()
                    .ok()
                    .flatten();
                let age_secs = row.as_ref().and_then(|r| {
                    chrono::DateTime::parse_from_rfc3339(&r.created_at).ok().map(|dt| {
                        (chrono::Utc::now() - dt.with_timezone(&chrono::Utc)).num_seconds()
                    })
                });
                let row_data = row.unwrap_or_else(|| fleet::AgentSessionRow {
                    id: db_id.clone(),
                    adapter_session_id: s.id.0.clone(),
                    adapter: inner.adapter_kind.as_str().to_string(),
                    model: s.model.clone(),
                    status: "active".to_string(),
                    stitch_id: None,
                    cost_usd: 0.0,
                    input_tokens: 0,
                    output_tokens: 0,
                    turn_count: 0,
                    created_at: chrono::Utc::now().to_rfc3339(),
                    last_activity_at: chrono::Utc::now().to_rfc3339(),
                    archived_at: None,
                    archived_reason: None,
                });
                AgentSessionStatus {
                    active: true,
                    session_id: Some(s.id.0.clone()),
                    adapter: Some(inner.adapter_kind.as_str().to_string()),
                    model: Some(s.model.clone()),
                    stitch_id: row_data.stitch_id,
                    cost_usd: row_data.cost_usd,
                    input_tokens: row_data.input_tokens,
                    output_tokens: row_data.output_tokens,
                    turn_count: row_data.turn_count,
                    created_at: Some(row_data.created_at),
                    last_activity_at: Some(row_data.last_activity_at),
                    age_secs,
                }
            }
            _ => AgentSessionStatus {
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
            },
        }
    }

    /// Spawn a new agent session. If one is already active, archives it first.
    pub async fn spawn(&self) -> Result<String> {
        let mut inner = self.inner.lock().await;

        // Archive any existing session.
        if let (Some(ref _session), Some(ref db_id)) = (&inner.session, &inner.db_row_id) {
            let _ = fleet::archive_agent_session(db_id, "superseded");
            let _ = self.event_tx.send(AgentSessionEvent::SessionArchived {
                session_id: db_id.clone(),
                reason: "superseded".to_string(),
            });
        }

        let spawn_config = SpawnConfig {
            model: inner.config.model.clone(),
            system_prompt: None,
            max_tokens: None,
            rate_limit_rpm: inner.config.rate_limit_rpm,
            cost_cap_usd: inner.config.cost_cap_usd,
            working_dir: None,
        };

        let session = inner.adapter.spawn_session(spawn_config).await?;
        let session_id = session.id.0.clone();
        let adapter_str = inner.adapter_kind.as_str().to_string();
        let model = session.model.clone();

        // Persist to fleet.db.
        let db_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let row = fleet::AgentSessionRow {
            id: db_id.clone(),
            adapter_session_id: session_id.clone(),
            adapter: adapter_str.clone(),
            model: model.clone(),
            status: "active".to_string(),
            stitch_id: None,
            cost_usd: 0.0,
            input_tokens: 0,
            output_tokens: 0,
            turn_count: 0,
            created_at: now,
            last_activity_at: chrono::Utc::now().to_rfc3339(),
            archived_at: None,
            archived_reason: None,
        };
        fleet::insert_agent_session(&row)?;

        inner.session = Some(session);
        inner.db_row_id = Some(db_id.clone());
        self.enabled.store(true, std::sync::atomic::Ordering::Relaxed);

        info!("Spawned new agent session {} (adapter={})", session_id, adapter_str);
        let _ = self.event_tx.send(AgentSessionEvent::SessionSpawned {
            session_id,
            adapter: adapter_str,
            model,
        });

        Ok(db_id)
    }

    /// Send a user turn and stream back events, persisting usage as we go.
    pub async fn send_turn(
        &self,
        prompt: String,
        attachments: Vec<Attachment>,
    ) -> Result<EventStream> {
        let inner = self.inner.lock().await;
        if !self.enabled.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(anyhow::anyhow!("Agent is disabled"));
        }
        let session = inner
            .session
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No active agent session"))?
            .clone();
        let adapter = &inner.adapter;

        let stream = adapter.send_turn(&session, &prompt, attachments).await?;
        Ok(stream)
    }

    /// Process a single event from the stream, updating DB and broadcasting.
    pub async fn handle_event(&self, event: &AgentEvent) {
        let inner = self.inner.lock().await;
        let session_id = match &inner.session {
            Some(s) => s.id.0.clone(),
            None => return,
        };
        let db_id = inner.db_row_id.clone();

        match event {
            AgentEvent::TextDelta { text } => {
                let _ = self.event_tx.send(AgentSessionEvent::TextDelta {
                    session_id: session_id.clone(),
                    text: text.clone(),
                });
            }
            AgentEvent::ToolUse { id, name, input } => {
                let _ = self.event_tx.send(AgentSessionEvent::ToolUse {
                    session_id: session_id.clone(),
                    id: id.clone(),
                    name: name.clone(),
                    input: input.clone(),
                });
            }
            AgentEvent::ToolResult { id, output, is_error } => {
                let _ = self.event_tx.send(AgentSessionEvent::ToolResult {
                    session_id: session_id.clone(),
                    id: id.clone(),
                    output: output.clone(),
                    is_error: *is_error,
                });
            }
            AgentEvent::TurnComplete { usage } => {
                if let (Some(ref db_id), Some(usage)) = (&db_id, usage) {
                    let cost = estimate_cost(
                        inner.adapter_kind,
                        &inner.config.model,
                        usage.input_tokens,
                        usage.output_tokens,
                        usage.cache_read_tokens,
                        usage.cache_write_tokens,
                    );
                    let _ = fleet::update_agent_session_usage(
                        db_id,
                        usage.input_tokens as i64,
                        usage.output_tokens as i64,
                        cost,
                    );
                    let _ = self.event_tx.send(AgentSessionEvent::TurnComplete {
                        session_id: session_id.clone(),
                        cost_usd: cost,
                        input_tokens: usage.input_tokens as i64,
                        output_tokens: usage.output_tokens as i64,
                    });
                }
            }
            AgentEvent::Error { message } => {
                let _ = self.event_tx.send(AgentSessionEvent::Error {
                    session_id: session_id.clone(),
                    message: message.clone(),
                });
            }
            AgentEvent::SessionEnded { reason } => {
                if let Some(ref db_id) = db_id {
                    let _ = fleet::archive_agent_session(db_id, reason);
                }
                let _ = self.event_tx.send(AgentSessionEvent::SessionArchived {
                    session_id: session_id.clone(),
                    reason: reason.clone(),
                });
            }
            AgentEvent::SessionStarted { .. } => {
                // Already handled at spawn time.
            }
        }
    }

    /// Switch adapter: archive old session, build new adapter, spawn fresh session.
    /// The old transcript is archived as a Stitch (if one was associated).
    pub async fn switch_adapter(&self, new_config: AgentAdapterConfig) -> Result<String> {
        let mut inner = self.inner.lock().await;

        // Archive old session.
        if let (Some(ref _session), Some(ref db_id)) = (&inner.session, &inner.db_row_id) {
            let _ = fleet::archive_agent_session(db_id, "switched");
            let _ = self.event_tx.send(AgentSessionEvent::SessionArchived {
                session_id: db_id.clone(),
                reason: "adapter_switch".to_string(),
            });
        }

        let adapter_kind = AdapterKind::from_config(&new_config.adapter)
            .ok_or_else(|| anyhow::anyhow!("unknown agent adapter: {}", new_config.adapter))?;
        let adapter_config: agent_adapter::AgentAdapterConfig = (&new_config).into();
        let adapter = agent_adapter::build_adapter(&adapter_config)?;

        let spawn_config = SpawnConfig {
            model: new_config.model.clone(),
            system_prompt: None,
            max_tokens: None,
            rate_limit_rpm: new_config.rate_limit_rpm,
            cost_cap_usd: new_config.cost_cap_usd,
            working_dir: None,
        };

        let session = adapter.spawn_session(spawn_config).await?;
        let session_id = session.id.0.clone();
        let adapter_str = adapter_kind.as_str().to_string();
        let model = session.model.clone();

        let db_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let row = fleet::AgentSessionRow {
            id: db_id.clone(),
            adapter_session_id: session_id.clone(),
            adapter: adapter_str.clone(),
            model: model.clone(),
            status: "active".to_string(),
            stitch_id: None,
            cost_usd: 0.0,
            input_tokens: 0,
            output_tokens: 0,
            turn_count: 0,
            created_at: now,
            last_activity_at: chrono::Utc::now().to_rfc3339(),
            archived_at: None,
            archived_reason: None,
        };
        fleet::insert_agent_session(&row)?;

        inner.adapter = adapter;
        inner.adapter_kind = adapter_kind;
        inner.session = Some(session);
        inner.db_row_id = Some(db_id.clone());
        inner.config = new_config;

        info!("Switched adapter, new session {} (adapter={})", session_id, adapter_str);
        let _ = self.event_tx.send(AgentSessionEvent::SessionSpawned {
            session_id,
            adapter: adapter_str,
            model,
        });

        Ok(db_id)
    }

    /// Disable the agent cleanly (agent-off). Archives session without leaving orphans.
    pub async fn disable(&self) -> Result<()> {
        let mut inner = self.inner.lock().await;
        self.enabled.store(false, std::sync::atomic::Ordering::Relaxed);

        if let (Some(ref session), Some(ref db_id)) = (&inner.session, &inner.db_row_id) {
            let _ = inner.adapter.close_session(session).await;
            let _ = fleet::archive_agent_session(db_id, "disabled");
            let _ = self.event_tx.send(AgentSessionEvent::SessionArchived {
                session_id: db_id.clone(),
                reason: "disabled".to_string(),
            });
            inner.session = None;
            inner.db_row_id = None;
            info!("Agent session disabled and archived");
        }

        Ok(())
    }

    /// Whether the agent is currently enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get the event sender (for wiring into the WS forwarder).
    pub fn event_sender(&self) -> broadcast::Sender<AgentSessionEvent> {
        self.event_tx.clone()
    }
}

/// Rough cost estimation. Real pricing comes from `~/.hoop/pricing.yml`, but
/// this provides a reasonable default so that the session manager can update
/// fleet.db without loading the full cost aggregator.
fn estimate_cost(
    adapter: AdapterKind,
    _model: &str,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: Option<u64>,
    cache_write_tokens: Option<u64>,
) -> f64 {
    let (input_price, output_price, cache_read_price, cache_write_price) = match adapter {
        AdapterKind::Claude | AdapterKind::Anthropic => {
            // Claude Opus 4 pricing (per 1M tokens)
            (15.0, 75.0, 1.875, 18.75)
        }
        AdapterKind::Zai => {
            // GLM pricing placeholder (per 1M tokens)
            (2.0, 8.0, 0.5, 4.0)
        }
    };

    let input = (input_tokens as f64 / 1_000_000.0) * input_price;
    let output = (output_tokens as f64 / 1_000_000.0) * output_price;
    let cache_read = cache_read_tokens
        .map(|t| (t as f64 / 1_000_000.0) * cache_read_price)
        .unwrap_or(0.0);
    let cache_write = cache_write_tokens
        .map(|t| (t as f64 / 1_000_000.0) * cache_write_price)
        .unwrap_or(0.0);

    input + output + cache_read + cache_write
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimate_cost_claude() {
        let cost = estimate_cost(
            AdapterKind::Claude,
            "claude-opus-4-7",
            1_000_000,
            100_000,
            Some(500_000),
            Some(200_000),
        );
        // input=15, output=7.5, cache_read=0.9375, cache_write=3.75
        assert!(cost > 25.0 && cost < 30.0, "cost was {}", cost);
    }

    #[test]
    fn estimate_cost_zai() {
        let cost = estimate_cost(
            AdapterKind::Zai,
            "glm-5",
            1_000_000,
            1_000_000,
            None,
            None,
        );
        // input=2, output=8
        assert!((cost - 10.0).abs() < 0.01, "cost was {}", cost);
    }

    #[test]
    fn agent_adapter_config_default() {
        let config = AgentAdapterConfig::default();
        assert_eq!(config.adapter, "claude");
        assert_eq!(config.model, "claude-opus-4-7");
    }

    #[test]
    fn agent_adapter_config_into() {
        let config = AgentAdapterConfig::default();
        let adapter_config: agent_adapter::AgentAdapterConfig = (&config).into();
        assert_eq!(adapter_config.adapter, "claude");
    }
}
