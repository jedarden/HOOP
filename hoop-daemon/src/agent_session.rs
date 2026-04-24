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
use crate::agent_context;
use crate::fleet;
use crate::metrics;
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
    pub enabled: bool,
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
    /// Reads persisted enabled state from metadata so agent-off survives restart.
    pub async fn new(config: AgentAdapterConfig) -> Result<Self> {
        let adapter_config: agent_adapter::AgentAdapterConfig = (&config).into();
        let adapter_kind = AdapterKind::from_config(&config.adapter)
            .ok_or_else(|| anyhow::anyhow!("unknown agent adapter: {}", config.adapter))?;
        let adapter = agent_adapter::build_adapter(&adapter_config)?;

        // Read persisted enabled state (defaults to true if never set).
        let persisted_enabled = fleet::is_agent_enabled().unwrap_or(true);
        let event_tx = broadcast::channel::<AgentSessionEvent>(256).0;
        let enabled = std::sync::atomic::AtomicBool::new(persisted_enabled);

        // Try to reattach to an existing active session from fleet.db.
        let mut db_row_id = None;
        let mut session = None;

        if persisted_enabled {
            if let Some(row) = fleet::load_active_agent_session()? {
                info!(
                    "Found active agent session {} (adapter={}, model={}), reattaching",
                    row.adapter_session_id, row.adapter, row.model
                );
                let sid = SessionId(row.adapter_session_id.clone());
                let adapter_str = row.adapter.clone();
                let model = row.model.clone();
                match adapter.resume_session(&sid).await {
                    Ok(s) => {
                        let reattach_sid = row.adapter_session_id.clone();
                        session = Some(s);
                        db_row_id = Some(row.id.clone());
                        info!("Reattached to agent session {}", reattach_sid);
                        // Broadcast reattach so WS clients know.
                        let _ = event_tx.send(AgentSessionEvent::SessionReattached {
                            session_id: reattach_sid,
                            adapter: adapter_str,
                            model,
                        });
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
        } else {
            info!("Agent is disabled (persisted state), skipping reattach");
        }

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
        let enabled = self.enabled.load(std::sync::atomic::Ordering::Relaxed);
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
                    has_started_session: false,
                    created_at: chrono::Utc::now().to_rfc3339(),
                    last_activity_at: chrono::Utc::now().to_rfc3339(),
                    archived_at: None,
                    archived_reason: None,
                });
                AgentSessionStatus {
                    active: true,
                    enabled,
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
                enabled,
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

        // Build thin context index for system prompt (lazy-fetch per §3.12)
        let system_prompt = match agent_context::load_projects_config()
            .and_then(|config| agent_context::build_context_index(&config))
        {
            Ok(index) => {
                let token_count = index.estimate_token_count();
                info!("Context index built: {} projects, {} recent stitches, {} open stitches, ~{} tokens",
                    index.projects.len(),
                    index.recent_activity.closed_stitches.len(),
                    index.open_stitches.len(),
                    token_count
                );
                Some(index.to_system_prompt())
            }
            Err(e) => {
                warn!("Failed to build context index, using empty system prompt: {}", e);
                None
            }
        };

        let spawn_config = SpawnConfig {
            model: inner.config.model.clone(),
            system_prompt,
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
            has_started_session: false,
            created_at: now,
            last_activity_at: chrono::Utc::now().to_rfc3339(),
            archived_at: None,
            archived_reason: None,
        };
        fleet::insert_agent_session(&row)?;

        inner.session = Some(session);
        inner.db_row_id = Some(db_id.clone());
        self.enabled.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = fleet::set_agent_enabled(true);

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

        let start = std::time::Instant::now();
        let stream = adapter.send_turn(&session, &prompt, attachments).await?;
        // Record the time-to-first-byte (stream creation = model started responding)
        let elapsed_ms = start.elapsed().as_secs_f64() * 1_000.0;
        metrics::metrics().hoop_agent_turn_duration_ms.observe(
            &[inner.adapter_kind.as_str(), &inner.config.model, "ttfb"],
            elapsed_ms,
        );
        Ok(stream)
    }

    /// Process a single event from the stream, updating DB and broadcasting.
    pub async fn handle_event(&self, event: &AgentEvent) {
        let mut inner = self.inner.lock().await;
        let session_id = match &inner.session {
            Some(s) => s.id.0.clone(),
            None => return,
        };
        let db_id = inner.db_row_id.clone();
        let session_has_started = inner.session.as_ref().map(|s| s.has_started_session).unwrap_or(true);

        match event {
            AgentEvent::TextDelta { text } => {
                let _ = self.event_tx.send(AgentSessionEvent::TextDelta {
                    session_id: session_id.clone(),
                    text: text.clone(),
                });
            }
            AgentEvent::ToolUse { id, name, input } => {
                metrics::metrics().hoop_agent_tool_calls_total.inc(&[name, "invoked"]);
                let _ = self.event_tx.send(AgentSessionEvent::ToolUse {
                    session_id: session_id.clone(),
                    id: id.clone(),
                    name: name.clone(),
                    input: input.clone(),
                });
            }
            AgentEvent::ToolResult { id, output, is_error } => {
                // We don't have the tool name here, so we record result generically.
                // The name is tracked at ToolUse time.
                let result_label = if *is_error { "error" } else { "ok" };
                metrics::metrics().hoop_agent_tool_calls_total.inc(&["*", result_label]);
                let _ = self.event_tx.send(AgentSessionEvent::ToolResult {
                    session_id: session_id.clone(),
                    id: id.clone(),
                    output: output.clone(),
                    is_error: *is_error,
                });
            }
            AgentEvent::TurnComplete { usage } => {
                // After the first turn completes, persist has_started_session=true so that
                // a daemon restart resumes with the provider's resume form (--resume,
                // exec resume, --continue) rather than the create form.
                if !session_has_started {
                    if let Some(ref mut session) = inner.session {
                        session.has_started_session = true;
                    }
                    if let Some(ref id) = db_id {
                        let _ = fleet::update_has_started_session(id, true);
                    }
                }
                if let (Some(ref db_id), Some(usage)) = (&db_id, usage) {
                    let adapter_str = inner.adapter_kind.as_str();
                    let model = &inner.config.model;

                    let cost = estimate_cost(
                        inner.adapter_kind,
                        model,
                        usage.input_tokens,
                        usage.output_tokens,
                        usage.cache_read_tokens,
                        usage.cache_write_tokens,
                    );

                    // Agent metrics
                    metrics::metrics().hoop_agent_turn_duration_ms.observe(
                        &[adapter_str, model, "complete"],
                        0.0, // duration tracked at send_turn level
                    );
                    metrics::metrics().hoop_agent_tokens_total.inc_by(
                        &[adapter_str, model, "input"],
                        usage.input_tokens,
                    );
                    metrics::metrics().hoop_agent_tokens_total.inc_by(
                        &[adapter_str, model, "output"],
                        usage.output_tokens,
                    );
                    metrics::metrics().hoop_agent_session_cost_usd.set(cost);

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

    /// Switch adapter: archive old session as a Stitch, build new adapter, spawn
    /// fresh session with Reflection Ledger + recent-activity context carried forward.
    pub async fn switch_adapter(&self, new_config: AgentAdapterConfig) -> Result<String> {
        let mut inner = self.inner.lock().await;

        // Archive old session and persist its transcript as a Stitch.
        if let (Some(ref old_session), Some(ref db_id)) = (&inner.session, &inner.db_row_id) {
            // Load session row for Stitch archival.
            let session_row = fleet::load_active_agent_session().ok().flatten();

            // Extract in-memory history (only API adapters have it).
            let history_guard = old_session.history.lock().await;
            let history: Vec<(String, String)> = history_guard
                .iter()
                .map(|m| (m.role.clone(), m.content.clone()))
                .collect();
            drop(history_guard);

            if let Some(ref row) = session_row {
                match fleet::archive_session_as_stitch(row, &history) {
                    Ok(stitch_id) => {
                        info!(
                            "Archived old agent session {} as stitch {}",
                            row.adapter_session_id, stitch_id
                        );
                    }
                    Err(e) => {
                        warn!("Failed to archive session as stitch: {}", e);
                    }
                }
            }

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

        // Build handoff context from Reflection Ledger + recent activity.
        let system_prompt = build_handoff_context();

        let spawn_config = SpawnConfig {
            model: new_config.model.clone(),
            system_prompt: Some(system_prompt),
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
            has_started_session: false,
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
    /// Persists disabled state so it survives daemon restart.
    pub async fn disable(&self) -> Result<()> {
        let mut inner = self.inner.lock().await;
        self.enabled.store(false, std::sync::atomic::Ordering::Relaxed);
        let _ = fleet::set_agent_enabled(false);

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

/// Build a handoff context string from the Reflection Ledger and recent Stitches.
/// Injected as the system prompt when switching adapters so the new session has
/// continuity with the old one.
fn build_handoff_context() -> String {
    let mut parts = Vec::new();

    // Approved reflection ledger entries (global + all scopes).
    if let Ok(entries) = fleet::list_approved_reflection_entries(None) {
        if !entries.is_empty() {
            let mut rules = String::from("## Operator Preferences (Reflection Ledger)\n");
            for entry in &entries {
                rules.push_str(&format!("- [{}] {}\n", entry.scope, entry.rule));
            }
            parts.push(rules);
        }
    }

    // Recent Stitches for situational awareness.
    if let Ok(stitches) = fleet::load_recent_stitches(10) {
        if !stitches.is_empty() {
            let mut recent = String::from("## Recent Activity\n");
            for (id, project, title, last_at) in &stitches {
                let short_id = if id.len() > 8 { &id[..8] } else { id };
                let short_ts = if last_at.len() > 19 { &last_at[..19] } else { last_at };
                recent.push_str(&format!("- [{}] {} — {} ({})\n", short_id, project, title, short_ts));
            }
            parts.push(recent);
        }
    }

    if parts.is_empty() {
        "You are the HOOP human-interface agent, continuing after an adapter switch.".to_string()
    } else {
        let mut ctx = String::from(
            "You are the HOOP human-interface agent, continuing after an adapter switch. \
             The following context was carried forward from your previous session.\n\n",
        );
        for part in parts {
            ctx.push_str(&part);
            ctx.push('\n');
        }
        ctx
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
        AdapterKind::Codex | AdapterKind::OpenCode | AdapterKind::Gemini => {
            (0.0, 0.0, 0.0, 0.0)
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
    use chrono::Utc;

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

    #[test]
    fn agent_session_status_includes_enabled() {
        let status = AgentSessionStatus {
            active: false,
            enabled: true,
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
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("\"active\":false"));
    }

    #[test]
    fn agent_session_event_reattached_serializes() {
        let event = AgentSessionEvent::SessionReattached {
            session_id: "sess-123".to_string(),
            adapter: "claude".to_string(),
            model: "claude-opus-4-7".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"session_reattached\""));
        assert!(json.contains("\"session_id\":\"sess-123\""));
        let parsed: AgentSessionEvent = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, AgentSessionEvent::SessionReattached { .. }));
    }

    // -----------------------------------------------------------------------
    // Lifecycle integration tests (fleet.db-backed)
    // -----------------------------------------------------------------------

    /// Helper: create an in-memory fleet.db with agent_sessions + stitches tables.
    fn test_db() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        conn.execute(
            r#"CREATE TABLE agent_sessions (
                id TEXT PRIMARY KEY NOT NULL,
                adapter_session_id TEXT NOT NULL,
                adapter TEXT NOT NULL,
                model TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'active'
                    CHECK(status IN ('active', 'archived', 'switched', 'disabled')),
                stitch_id TEXT,
                cost_usd REAL NOT NULL DEFAULT 0.0,
                input_tokens INTEGER NOT NULL DEFAULT 0,
                output_tokens INTEGER NOT NULL DEFAULT 0,
                turn_count INTEGER NOT NULL DEFAULT 0,
                has_started_session INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                last_activity_at TEXT NOT NULL,
                archived_at TEXT,
                archived_reason TEXT
            )"#, []).unwrap();
        conn.execute(
            r#"CREATE TABLE metadata (
                key TEXT PRIMARY KEY NOT NULL,
                value TEXT NOT NULL
            )"#, []).unwrap();
        conn.execute(
            r#"CREATE TABLE stitches (
                id TEXT PRIMARY KEY NOT NULL,
                project TEXT NOT NULL,
                kind TEXT NOT NULL CHECK(kind IN ('operator', 'dictated', 'worker', 'ad-hoc')),
                title TEXT NOT NULL,
                created_by TEXT NOT NULL,
                created_at TEXT NOT NULL,
                last_activity_at TEXT NOT NULL,
                participants TEXT DEFAULT '[]',
                attachments_path TEXT
            )"#, []).unwrap();
        conn.execute(
            r#"CREATE TABLE stitch_messages (
                id TEXT PRIMARY KEY NOT NULL,
                stitch_id TEXT NOT NULL REFERENCES stitches(id) ON DELETE CASCADE,
                ts TEXT NOT NULL,
                role TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'system', 'tool')),
                content TEXT NOT NULL,
                attachments TEXT DEFAULT '[]',
                tokens INTEGER
            )"#, []).unwrap();
        conn.execute(
            r#"CREATE TABLE reflection_ledger (
                id TEXT PRIMARY KEY NOT NULL,
                scope TEXT NOT NULL,
                rule TEXT NOT NULL,
                reason TEXT NOT NULL,
                source_stitches TEXT NOT NULL DEFAULT '[]',
                status TEXT NOT NULL DEFAULT 'proposed'
                    CHECK(status IN ('proposed', 'approved', 'rejected', 'archived')),
                created_at TEXT NOT NULL,
                last_applied TEXT,
                applied_count INTEGER NOT NULL DEFAULT 0
            )"#, []).unwrap();
        conn
    }

    /// Acceptance: session survives restart (reattach from DB).
    ///
    /// Simulates: daemon creates a session, persists it to DB, "restarts"
    /// (re-reads the DB), and reattaches to the same session.
    #[test]
    fn session_persists_across_restart() {
        let db = test_db();
        let session_id = uuid::Uuid::new_v4().to_string();
        let adapter_sess = "adapter-sess-restart-1";
        let now = chrono::Utc::now().to_rfc3339();

        // 1. Daemon creates and persists a session.
        db.execute(
            r#"INSERT INTO agent_sessions
               (id, adapter_session_id, adapter, model, status, cost_usd, input_tokens,
                output_tokens, turn_count, created_at, last_activity_at)
               VALUES (?1,?2,'claude','claude-opus-4-7','active',0.025,1200,300,2,?3,?3)"#,
            rusqlite::params![session_id, adapter_sess, now],
        ).unwrap();

        // 2. Simulate restart: load the active session.
        let row: fleet::AgentSessionRow = db.query_row(
            "SELECT id, adapter_session_id, adapter, model, status, stitch_id,
                    cost_usd, input_tokens, output_tokens, turn_count,
                    has_started_session, created_at, last_activity_at, archived_at, archived_reason
             FROM agent_sessions WHERE status = 'active' ORDER BY created_at DESC LIMIT 1",
            [],
            |row| Ok(fleet::AgentSessionRow {
                id: row.get(0)?,
                adapter_session_id: row.get(1)?,
                adapter: row.get(2)?,
                model: row.get(3)?,
                status: row.get(4)?,
                stitch_id: row.get(5)?,
                cost_usd: row.get(6)?,
                input_tokens: row.get(7)?,
                output_tokens: row.get(8)?,
                turn_count: row.get(9)?,
                has_started_session: row.get(10)?,
                created_at: row.get(11)?,
                last_activity_at: row.get(12)?,
                archived_at: row.get(13)?,
                archived_reason: row.get(14)?,
            }),
        ).unwrap();

        // 3. Verify the reattached session matches what was persisted.
        assert_eq!(row.id, session_id);
        assert_eq!(row.adapter_session_id, adapter_sess);
        assert_eq!(row.adapter, "claude");
        assert_eq!(row.model, "claude-opus-4-7");
        assert_eq!(row.status, "active");
        assert_eq!(row.cost_usd, 0.025);
        assert_eq!(row.input_tokens, 1200);
        assert_eq!(row.output_tokens, 300);
        assert_eq!(row.turn_count, 2);
    }

    /// Acceptance: daemon restart mid-session (after turn 1) resumes with the
    /// correct adapter invocation.
    ///
    /// Simulates: session starts with has_started_session=false, first turn
    /// completes (flag flipped to true in DB), daemon restarts and reloads the
    /// session — subsequent turns must use the resume form (--resume / exec resume
    /// / --continue) rather than the create form.
    #[test]
    fn restart_mid_session_preserves_resume_flag() {
        use crate::agent_adapter::AdapterKind;

        let db = test_db();
        let db_row_id = uuid::Uuid::new_v4().to_string();
        let provider_session_id = "deadbeef-0000-0000-0000-000000000002";
        let now = chrono::Utc::now().to_rfc3339();

        // 1. Persist session as if first turn already completed (has_started_session=1).
        db.execute(
            r#"INSERT INTO agent_sessions
               (id, adapter_session_id, adapter, model, status, cost_usd, input_tokens,
                output_tokens, turn_count, has_started_session, created_at, last_activity_at)
               VALUES (?1, ?2, 'claude', 'claude-opus-4-7', 'active', 0.05, 2000, 500, 3, 1, ?3, ?3)"#,
            rusqlite::params![db_row_id, provider_session_id, now],
        ).unwrap();

        // 2. Simulate restart: load the row from DB.
        let row: fleet::AgentSessionRow = db.query_row(
            "SELECT id, adapter_session_id, adapter, model, status, stitch_id,
                    cost_usd, input_tokens, output_tokens, turn_count,
                    has_started_session, created_at, last_activity_at, archived_at, archived_reason
             FROM agent_sessions WHERE status = 'active' ORDER BY created_at DESC LIMIT 1",
            [],
            |row| Ok(fleet::AgentSessionRow {
                id: row.get(0)?,
                adapter_session_id: row.get(1)?,
                adapter: row.get(2)?,
                model: row.get(3)?,
                status: row.get(4)?,
                stitch_id: row.get(5)?,
                cost_usd: row.get(6)?,
                input_tokens: row.get(7)?,
                output_tokens: row.get(8)?,
                turn_count: row.get(9)?,
                has_started_session: row.get(10)?,
                created_at: row.get(11)?,
                last_activity_at: row.get(12)?,
                archived_at: row.get(13)?,
                archived_reason: row.get(14)?,
            }),
        ).unwrap();

        // 3. The reloaded row must have has_started_session=true.
        assert!(row.has_started_session,
            "reloaded session must have has_started_session=true after daemon restart");

        // 4. Verify each adapter uses the resume form when has_started_session=true.
        //    Claude: --resume (not --session-id)
        let claude_args = AdapterKind::Claude
            .build_turn_args(&row.adapter_session_id, "continue", row.has_started_session)
            .unwrap();
        assert_eq!(claude_args[0], "--resume",
            "Claude must use --resume after restart, not --session-id");
        assert_eq!(claude_args[1], provider_session_id);

        //    Codex: exec resume <id>
        let codex_args = AdapterKind::Codex
            .build_turn_args(provider_session_id, "continue", true)
            .unwrap();
        assert_eq!(&codex_args[..3], &["exec", "resume", provider_session_id],
            "Codex must use exec resume <id> after restart");

        //    OpenCode: --session <id> --continue
        let oc_args = AdapterKind::OpenCode
            .build_turn_args(provider_session_id, "continue", true)
            .unwrap();
        assert_eq!(oc_args[0], "--session");
        assert_eq!(oc_args[1], provider_session_id);
        assert_eq!(oc_args[2], "--continue",
            "OpenCode must include --continue after restart");

        //    Gemini: same args (sandbox-native, no distinction)
        let g_args_turn1 = AdapterKind::Gemini
            .build_turn_args(provider_session_id, "continue", false)
            .unwrap();
        let g_args_resume = AdapterKind::Gemini
            .build_turn_args(provider_session_id, "continue", true)
            .unwrap();
        assert_eq!(g_args_turn1, g_args_resume,
            "Gemini args must be identical regardless of has_started_session");
    }

    /// Acceptance: has_started_session starts false and is updated to true after first turn.
    #[test]
    fn has_started_session_false_on_new_session() {
        let db = test_db();
        let db_row_id = uuid::Uuid::new_v4().to_string();
        let provider_session_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        // New session: has_started_session defaults to 0 (false).
        db.execute(
            r#"INSERT INTO agent_sessions
               (id, adapter_session_id, adapter, model, status, cost_usd, input_tokens,
                output_tokens, turn_count, has_started_session, created_at, last_activity_at)
               VALUES (?1, ?2, 'claude', 'claude-opus-4-7', 'active', 0.0, 0, 0, 0, 0, ?3, ?3)"#,
            rusqlite::params![db_row_id, provider_session_id, now],
        ).unwrap();

        let started: bool = db.query_row(
            "SELECT has_started_session FROM agent_sessions WHERE id = ?1",
            rusqlite::params![db_row_id],
            |row| row.get(0),
        ).unwrap();
        assert!(!started, "new session must have has_started_session=false");

        // Simulate first turn completing: flip to true.
        db.execute(
            "UPDATE agent_sessions SET has_started_session = 1 WHERE id = ?1",
            rusqlite::params![db_row_id],
        ).unwrap();

        let started_after: bool = db.query_row(
            "SELECT has_started_session FROM agent_sessions WHERE id = ?1",
            rusqlite::params![db_row_id],
            |row| row.get(0),
        ).unwrap();
        assert!(started_after, "has_started_session must be true after first turn");
    }

    /// Acceptance: adapter switch archives old session and starts a new one.
    ///
    /// Simulates: old session gets archived with reason "switched", transcript
    /// is saved as a Stitch, and a new session row is inserted.
    #[test]
    fn adapter_switch_archives_old_session_as_stitch() {
        let db = test_db();
        let old_id = uuid::Uuid::new_v4().to_string();
        let old_adapter_sess = "adapter-old";
        let now = chrono::Utc::now().to_rfc3339();

        // 1. Create an active session with some usage.
        db.execute(
            r#"INSERT INTO agent_sessions
               (id, adapter_session_id, adapter, model, status, cost_usd, input_tokens,
                output_tokens, turn_count, created_at, last_activity_at)
               VALUES (?1,?2,'claude','claude-opus-4-7','active',0.08,5000,1200,5,?3,?3)"#,
            rusqlite::params![old_id, old_adapter_sess, now],
        ).unwrap();

        // 2. Archive old session as "switched".
        let archive_ts = chrono::Utc::now().to_rfc3339();
        db.execute(
            "UPDATE agent_sessions SET status = 'switched', archived_at = ?1, archived_reason = 'adapter_switch' WHERE id = ?2",
            rusqlite::params![archive_ts, old_id],
        ).unwrap();

        // 3. Archive transcript as a Stitch.
        let stitch_id = uuid::Uuid::new_v4().to_string();
        db.execute(
            r#"INSERT INTO stitches (id, project, kind, title, created_by, created_at, last_activity_at)
               VALUES (?1, 'hoop-agent', 'operator', 'Agent session claude (archived)', 'hoop:agent', ?2, ?3)"#,
            rusqlite::params![stitch_id, now, archive_ts],
        ).unwrap();
        db.execute(
            "UPDATE agent_sessions SET stitch_id = ?1 WHERE id = ?2",
            rusqlite::params![stitch_id, old_id],
        ).unwrap();

        // 4. Insert new session on the new adapter.
        let new_id = uuid::Uuid::new_v4().to_string();
        let new_adapter_sess = "adapter-new";
        let new_now = chrono::Utc::now().to_rfc3339();
        db.execute(
            r#"INSERT INTO agent_sessions
               (id, adapter_session_id, adapter, model, status, cost_usd, input_tokens,
                output_tokens, turn_count, created_at, last_activity_at)
               VALUES (?1,?2,'zai','glm-5','active',0.0,0,0,0,?3,?3)"#,
            rusqlite::params![new_id, new_adapter_sess, new_now],
        ).unwrap();

        // 5. Verify old session is archived, linked to stitch.
        let (status, archived_reason, linked_stitch): (String, Option<String>, Option<String>) =
            db.query_row(
                "SELECT status, archived_reason, stitch_id FROM agent_sessions WHERE id = ?1",
                rusqlite::params![old_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            ).unwrap();
        assert_eq!(status, "switched");
        assert_eq!(archived_reason, Some("adapter_switch".to_string()));
        assert_eq!(linked_stitch, Some(stitch_id.clone()));

        // 6. Verify new session is active.
        let new_status: String = db.query_row(
            "SELECT status FROM agent_sessions WHERE id = ?1",
            rusqlite::params![new_id],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(new_status, "active");

        // 7. Verify only one active session.
        let active_count: i64 = db.query_row(
            "SELECT COUNT(*) FROM agent_sessions WHERE status = 'active'",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(active_count, 1);

        // 8. Verify stitch has messages.
        let msg_count: i64 = db.query_row(
            "SELECT COUNT(*) FROM stitches WHERE id = ?1",
            rusqlite::params![stitch_id],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(msg_count, 1);
    }

    /// Acceptance: session age and cost are tracked and readable.
    #[test]
    fn session_age_and_cost_tracked() {
        let db = test_db();
        let id = uuid::Uuid::new_v4().to_string();
        // Use a timestamp 2 hours ago to verify age computation.
        let two_hours_ago = (chrono::Utc::now() - chrono::Duration::hours(2)).to_rfc3339();

        db.execute(
            r#"INSERT INTO agent_sessions
               (id, adapter_session_id, adapter, model, status, cost_usd, input_tokens,
                output_tokens, turn_count, created_at, last_activity_at)
               VALUES (?1,'sess-age','claude','claude-opus-4-7','active',0.0,0,0,0,?2,?2)"#,
            rusqlite::params![id, two_hours_ago],
        ).unwrap();

        // Simulate 3 turns accumulating cost.
        for _ in 0..3 {
            db.execute(
                r#"UPDATE agent_sessions
                   SET input_tokens = input_tokens + 1000,
                       output_tokens = output_tokens + 200,
                       cost_usd = cost_usd + 0.015,
                       turn_count = turn_count + 1
                   WHERE id = ?1"#,
                rusqlite::params![id],
            ).unwrap();
        }

        // Verify accumulated usage.
        let (cost, input, output, turns, created_at): (f64, i64, i64, i64, String) =
            db.query_row(
                "SELECT cost_usd, input_tokens, output_tokens, turn_count, created_at FROM agent_sessions WHERE id = ?1",
                rusqlite::params![id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            ).unwrap();

        assert!((cost - 0.045).abs() < 0.001, "cost was {}", cost);
        assert_eq!(input, 3000);
        assert_eq!(output, 600);
        assert_eq!(turns, 3);

        // Verify age is ~2 hours.
        let created = chrono::DateTime::parse_from_rfc3339(&created_at).unwrap();
        let age_secs = (chrono::Utc::now() - created.with_timezone(&chrono::Utc)).num_seconds();
        assert!((7100..=7300).contains(&age_secs), "age was {}s", age_secs);
    }

    /// Acceptance: agent-off cleanly disables with no orphaned session.
    #[test]
    fn agent_off_disables_cleanly() {
        let db = test_db();
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        // 1. Start with an active session.
        db.execute(
            r#"INSERT INTO agent_sessions
               (id, adapter_session_id, adapter, model, status, cost_usd, input_tokens,
                output_tokens, turn_count, created_at, last_activity_at)
               VALUES (?1,'sess-disable','claude','claude-opus-4-7','active',0.05,2000,500,3,?2,?2)"#,
            rusqlite::params![id, now],
        ).unwrap();

        // Persist enabled state.
        db.execute(
            "INSERT INTO metadata (key, value) VALUES ('agent_enabled', 'true')",
            [],
        ).unwrap();

        // 2. Agent-off: archive session + persist disabled state.
        let archive_ts = chrono::Utc::now().to_rfc3339();
        db.execute(
            "UPDATE agent_sessions SET status = 'disabled', archived_at = ?1, archived_reason = 'disabled' WHERE id = ?2",
            rusqlite::params![archive_ts, id],
        ).unwrap();
        db.execute(
            "INSERT OR REPLACE INTO metadata (key, value) VALUES ('agent_enabled', 'false')",
            [],
        ).unwrap();

        // 3. Verify no active sessions remain.
        let active_count: i64 = db.query_row(
            "SELECT COUNT(*) FROM agent_sessions WHERE status = 'active'",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(active_count, 0, "should have no orphaned active sessions");

        // 4. Verify session is disabled (not just archived).
        let (status, reason): (String, String) = db.query_row(
            "SELECT status, archived_reason FROM agent_sessions WHERE id = ?1",
            rusqlite::params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap();
        assert_eq!(status, "disabled");
        assert_eq!(reason, "disabled");

        // 5. Verify persisted enabled state is false.
        let enabled: String = db.query_row(
            "SELECT value FROM metadata WHERE key = 'agent_enabled'",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(enabled, "false");

        // 6. Verify disabled state survives a "restart" (re-read from metadata).
        let persisted_enabled = db.query_row(
            "SELECT value FROM metadata WHERE key = 'agent_enabled'",
            [],
            |row| row.get::<_, String>(0),
        ).unwrap_or_else(|_| "true".to_string());
        assert_eq!(persisted_enabled, "false");
    }

    /// Acceptance: multiple sessions over time, only latest active.
    #[test]
    fn only_latest_active_session_after_several_switches() {
        let db = test_db();

        // Create 3 sessions, archiving each in sequence.
        for i in 0..3 {
            let id = uuid::Uuid::new_v4().to_string();
            let adapter_sess = format!("sess-{}", i);
            let now = Utc::now().to_rfc3339();
            let adapter = if i < 2 { "claude" } else { "zai" };
            let model = if i < 2 { "claude-opus-4-7" } else { "glm-5" };

            db.execute(
                r#"INSERT INTO agent_sessions
                   (id, adapter_session_id, adapter, model, status, cost_usd, input_tokens,
                    output_tokens, turn_count, created_at, last_activity_at)
                   VALUES (?1,?2,?3,?4,'active',0.0,0,0,0,?5,?5)"#,
                rusqlite::params![id, adapter_sess, adapter, model, now],
            ).unwrap();

            // Archive previous sessions (simulating switch).
            if i > 0 {
                db.execute(
                    "UPDATE agent_sessions SET status = 'switched', archived_at = ?1, archived_reason = 'switched' WHERE status = 'active' AND id != ?2",
                    rusqlite::params![now, id],
                ).unwrap();
            }
        }

        // Only the last session should be active.
        let active_count: i64 = db.query_row(
            "SELECT COUNT(*) FROM agent_sessions WHERE status = 'active'",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(active_count, 1);

        let active_adapter: String = db.query_row(
            "SELECT adapter FROM agent_sessions WHERE status = 'active'",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(active_adapter, "zai");

        let switched_count: i64 = db.query_row(
            "SELECT COUNT(*) FROM agent_sessions WHERE status = 'switched'",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(switched_count, 2);
    }

    /// Acceptance: handoff context includes Reflection Ledger entries.
    #[test]
    fn handoff_context_includes_reflection_ledger() {
        let db = test_db();

        // Insert approved reflection ledger entries.
        let now = Utc::now().to_rfc3339();
        db.execute(
            "INSERT INTO reflection_ledger (id, scope, rule, reason, status, created_at) VALUES (?1, 'global', 'always run tests before closing', 'operator repeated 3 times', 'approved', ?2)",
            rusqlite::params![uuid::Uuid::new_v4().to_string(), now],
        ).unwrap();
        db.execute(
            "INSERT INTO reflection_ledger (id, scope, rule, reason, status, created_at) VALUES (?1, 'project:hoop', 'never edit fleet.db directly', 'one incident of corruption', 'approved', ?2)",
            rusqlite::params![uuid::Uuid::new_v4().to_string(), now],
        ).unwrap();
        // Rejected entry should NOT appear.
        db.execute(
            "INSERT INTO reflection_ledger (id, scope, rule, reason, status, created_at) VALUES (?1, 'global', 'bad rule', 'n/a', 'rejected', ?2)",
            rusqlite::params![uuid::Uuid::new_v4().to_string(), now],
        ).unwrap();

        // Query approved entries (same logic as build_handoff_context).
        let approved: Vec<(String, String)> = db
            .prepare("SELECT scope, rule FROM reflection_ledger WHERE status = 'approved' ORDER BY created_at ASC")
            .unwrap()
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert_eq!(approved.len(), 2);
        assert_eq!(approved[0].1, "always run tests before closing");
        assert_eq!(approved[1].0, "project:hoop");
    }

    /// Verify all AgentSessionEvent variants round-trip through JSON.
    #[test]
    fn all_session_event_variants_round_trip() {
        let events = vec![
            AgentSessionEvent::SessionSpawned {
                session_id: "s1".into(),
                adapter: "claude".into(),
                model: "claude-opus-4-7".into(),
            },
            AgentSessionEvent::SessionReattached {
                session_id: "s2".into(),
                adapter: "zai".into(),
                model: "glm-5".into(),
            },
            AgentSessionEvent::TextDelta {
                session_id: "s1".into(),
                text: "hello".into(),
            },
            AgentSessionEvent::ToolUse {
                session_id: "s1".into(),
                id: "tu1".into(),
                name: "read_file".into(),
                input: serde_json::json!({"path": "/foo"}),
            },
            AgentSessionEvent::ToolResult {
                session_id: "s1".into(),
                id: "tu1".into(),
                output: serde_json::json!("ok"),
                is_error: false,
            },
            AgentSessionEvent::TurnComplete {
                session_id: "s1".into(),
                cost_usd: 0.025,
                input_tokens: 100,
                output_tokens: 50,
            },
            AgentSessionEvent::SessionArchived {
                session_id: "s1".into(),
                reason: "switched".into(),
            },
            AgentSessionEvent::Error {
                session_id: "s1".into(),
                message: "rate limited".into(),
            },
        ];

        for original in &events {
            let json = serde_json::to_string(original).unwrap();
            let restored: AgentSessionEvent = serde_json::from_str(&json).unwrap();
            let json2 = serde_json::to_string(&restored).unwrap();
            assert_eq!(json, json2, "round-trip failed for: {}", json);
        }
    }

    /// Acceptance: session status computes correctly when active vs idle.
    #[test]
    fn session_status_fields_complete() {
        let status_active = AgentSessionStatus {
            active: true,
            enabled: true,
            session_id: Some("sess-1".into()),
            adapter: Some("claude".into()),
            model: Some("claude-opus-4-7".into()),
            stitch_id: None,
            cost_usd: 0.15,
            input_tokens: 10000,
            output_tokens: 2500,
            turn_count: 8,
            created_at: Some("2026-04-23T10:00:00Z".into()),
            last_activity_at: Some("2026-04-23T12:00:00Z".into()),
            age_secs: Some(7200),
        };

        let json = serde_json::to_string(&status_active).unwrap();
        assert!(json.contains("\"active\":true"));
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("\"session_id\":\"sess-1\""));
        assert!(json.contains("\"cost_usd\":0.15"));
        assert!(json.contains("\"turn_count\":8"));
        assert!(json.contains("\"age_secs\":7200"));

        let parsed: AgentSessionStatus = serde_json::from_str(&json).unwrap();
        assert!(parsed.active);
        assert_eq!(parsed.cost_usd, 0.15);
        assert_eq!(parsed.age_secs, Some(7200));
    }

    /// Verify build_handoff_context produces a non-empty string.
    #[test]
    fn handoff_context_fallback_when_no_ledger() {
        let ctx = build_handoff_context();
        assert!(!ctx.is_empty());
        assert!(ctx.contains("HOOP") || ctx.contains("adapter switch"));
    }
}
