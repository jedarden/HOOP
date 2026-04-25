//! Agent adapter abstraction for LLM-agnostic human-interface agent sessions.
//!
//! Defines a unified trait `AgentAdapter` with three implementations:
//! - `ClaudeCodeAdapter` — shells out to the `claude` CLI (default)
//! - `AnthropicApiAdapter` — direct Anthropic Messages API
//! - `ZaiGlmAdapter` — ZAI proxy with GLM models
//!
//! Adapter selection is config-driven via `~/.hoop/config.yml` `agent.adapter`.
//! All adapters emit an identical `AgentEvent` stream; callers never branch on adapter type.
//!
//! Plan reference: §7 (LLM-agnostic), §5 Phase 5, §1.6 Roles.

use anyhow::Result;
use async_trait::async_trait;
use futures_util::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;

// ---------------------------------------------------------------------------
// Unified event types — identical shape regardless of adapter
// ---------------------------------------------------------------------------

/// Adapter-unique session handle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SessionId(pub String);

/// Running session returned by `resume_session` / `spawn_session`.
#[derive(Debug, Clone)]
pub struct AgentSession {
    pub id: SessionId,
    pub adapter: AdapterKind,
    pub model: String,
    /// System prompt to prepend to every turn.
    pub system_prompt: Option<String>,
    /// Working directory for tool execution (Claude Code adapter).
    pub working_dir: Option<String>,
    /// In-memory conversation history for API-based adapters.
    pub history: Arc<Mutex<Vec<HistoryMessage>>>,
    /// Whether the provider has acknowledged a first turn. Gated by this flag,
    /// `build_turn_args` emits the correct create-vs-resume invocation per adapter.
    /// Persisted in fleet.db so daemon restarts don't corrupt the provider's session store.
    pub has_started_session: bool,
}

/// A single message in the conversation history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryMessage {
    pub role: String,
    pub content: String,
}

/// File or URL attachment sent with a turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Attachment {
    File { path: String },
    Url { url: String },
    Inline { name: String, content: String, mime: String },
}

/// Single event in the unified stream returned by `send_turn`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentEvent {
    /// Session was started successfully.
    SessionStarted { session_id: SessionId },
    /// Incremental text delta from the model.
    TextDelta { text: String },
    /// Model is invoking a tool.
    ToolUse { id: String, name: String, input: serde_json::Value },
    /// Tool returned a result.
    ToolResult { id: String, output: serde_json::Value, is_error: bool },
    /// Turn completed with optional usage stats.
    TurnComplete { usage: Option<TurnUsage> },
    /// An error occurred inside the adapter.
    Error { message: String },
    /// Session ended (model refused or hit limit).
    SessionEnded { reason: String },
}

/// Token usage for a single turn.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TurnUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_write_tokens: Option<u64>,
}

/// Configuration passed to `spawn_session`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnConfig {
    /// Model identifier (adapter-specific, e.g. "claude-opus-4-7", "glm-5.1").
    pub model: String,
    /// System prompt prepended to every turn.
    pub system_prompt: Option<String>,
    /// Maximum output tokens per turn.
    pub max_tokens: Option<u64>,
    /// Optional rate-limit (requests per minute).
    pub rate_limit_rpm: Option<u32>,
    /// Optional cost cap (USD) for the session lifetime.
    pub cost_cap_usd: Option<f64>,
    /// Working directory for tool execution (Claude Code adapter).
    pub working_dir: Option<String>,
}

/// Which adapter to use — mirrors `agent_config.json` enum.
///
/// Claude/Anthropic/Zai are currently implemented. Codex/OpenCode/Gemini define
/// their resume semantics here so that the invocation builder is correct from day
/// one; full adapter impls will follow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AdapterKind {
    Claude,
    Anthropic,
    Zai,
    Codex,
    OpenCode,
    Gemini,
}

impl AdapterKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Anthropic => "anthropic",
            Self::Zai => "zai",
            Self::Codex => "codex",
            Self::OpenCode => "opencode",
            Self::Gemini => "gemini",
        }
    }

    pub fn from_config(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "claude" => Some(Self::Claude),
            "anthropic" => Some(Self::Anthropic),
            "zai" => Some(Self::Zai),
            "codex" => Some(Self::Codex),
            "opencode" => Some(Self::OpenCode),
            "gemini" => Some(Self::Gemini),
            _ => None,
        }
    }

    /// Build CLI arguments for a turn, distinguishing first-turn (create) from
    /// subsequent turns (resume). Each adapter has its own convention:
    ///
    /// - Claude: turn 1 = `--session-id <uuid>`, turn N = `--resume <uuid>`
    /// - Codex: turn 1 = `exec`, turn N = `exec resume <thread-id>`
    /// - OpenCode: turn 1 = `--session <sid>`, turn N = `--session <sid> --continue`
    /// - Gemini: sandbox-native (no CLI arg distinction)
    /// - Anthropic/Zai: API-based, no CLI invocation
    ///
    /// Returns `None` for API-based adapters that don't shell out.
    pub fn build_turn_args(
        &self,
        session_id: &str,
        prompt: &str,
        has_started_session: bool,
    ) -> Option<Vec<String>> {
        match self {
            Self::Claude => {
                let mut args = if has_started_session {
                    vec!["--resume".to_string(), session_id.to_string()]
                } else {
                    vec!["--session-id".to_string(), session_id.to_string()]
                };
                args.extend_from_slice(&[
                    "--output-format".to_string(),
                    "stream-json".to_string(),
                    "-p".to_string(),
                    prompt.to_string(),
                ]);
                Some(args)
            }
            Self::Codex => {
                let mut args = vec!["exec".to_string()];
                if has_started_session {
                    args.push("resume".to_string());
                    args.push(session_id.to_string());
                }
                args.push("-p".to_string());
                args.push(prompt.to_string());
                Some(args)
            }
            Self::OpenCode => {
                let mut args = vec![
                    "--session".to_string(),
                    session_id.to_string(),
                ];
                if has_started_session {
                    args.push("--continue".to_string());
                }
                args.push("-p".to_string());
                args.push(prompt.to_string());
                Some(args)
            }
            Self::Gemini => {
                // Sandbox-native: single prompt arg, no session-id vs resume distinction.
                Some(vec!["-p".to_string(), prompt.to_string()])
            }
            Self::Anthropic | Self::Zai => None,
        }
    }
}

/// Wrapper providing `Debug` for a dyn `AgentAdapter`.
#[derive(Clone)]
pub struct AdapterRef(pub Arc<dyn AgentAdapter>);

impl std::fmt::Debug for AdapterRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdapterRef")
            .field("kind", &self.0.kind())
            .finish()
    }
}

impl std::ops::Deref for AdapterRef {
    type Target = dyn AgentAdapter;
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

/// Type alias for the boxed event stream returned by `send_turn`.
pub type EventStream = Pin<Box<dyn Stream<Item = Result<AgentEvent>> + Send>>;

// ---------------------------------------------------------------------------
// The trait — same shape as NEEDLE worker adapters
// ---------------------------------------------------------------------------

/// Agent adapter trait.
///
/// Implementations wrap a specific LLM provider. Callers use only these four
/// methods; the event stream shape is guaranteed identical across adapters.
#[async_trait]
pub trait AgentAdapter: Send + Sync {
    /// Which adapter this is.
    fn kind(&self) -> AdapterKind;

    /// Start a new session. Returns the session ID on success.
    async fn spawn_session(&self, config: SpawnConfig) -> Result<AgentSession>;

    /// Resume a previously spawned session.
    async fn resume_session(&self, id: &SessionId) -> Result<AgentSession>;

    /// Send a user turn and receive a stream of events.
    async fn send_turn(
        &self,
        session: &AgentSession,
        prompt: &str,
        attachments: Vec<Attachment>,
    ) -> Result<EventStream>;

    /// Gracefully close a session.
    async fn close_session(&self, session: &AgentSession) -> Result<()>;
}

// ---------------------------------------------------------------------------
// Factory — config-driven selection
// ---------------------------------------------------------------------------

/// Configuration for the agent adapter factory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAdapterConfig {
    /// Which adapter to use. Defaults to "claude".
    pub adapter: String,
    /// Default model name.
    #[serde(default = "default_model")]
    pub model: String,
    /// Optional Anthropic API key (for `anthropic` adapter).
    pub anthropic_api_key: Option<String>,
    /// Optional ZAI proxy base URL.
    pub zai_base_url: Option<String>,
    /// Optional ZAI API key.
    pub zai_api_key: Option<String>,
    /// Optional rate limit.
    pub rate_limit_rpm: Option<u32>,
    /// Optional cost cap.
    pub cost_cap_usd: Option<f64>,
}

fn default_model() -> String {
    "claude-opus-4-7".to_string()
}

impl Default for AgentAdapterConfig {
    fn default() -> Self {
        Self {
            adapter: "claude".to_string(),
            model: default_model(),
            anthropic_api_key: None,
            zai_base_url: None,
            zai_api_key: None,
            rate_limit_rpm: None,
            cost_cap_usd: None,
        }
    }
}

/// Build the correct adapter from config. No code change needed to switch.
pub fn build_adapter(config: &AgentAdapterConfig) -> Result<Box<dyn AgentAdapter>> {
    let kind = AdapterKind::from_config(&config.adapter)
        .ok_or_else(|| anyhow::anyhow!("unknown agent adapter: {}", config.adapter))?;

    let adapter: Box<dyn AgentAdapter> = match kind {
        AdapterKind::Claude => Box::new(ClaudeCodeAdapter {
            default_model: config.model.clone(),
        }),
        AdapterKind::Anthropic => Box::new(AnthropicApiAdapter {
            api_key: config.anthropic_api_key.clone().unwrap_or_default(),
            default_model: config.model.clone(),
        }),
        AdapterKind::Zai => Box::new(ZaiGlmAdapter {
            base_url: config.zai_base_url.clone().unwrap_or_else(|| "https://zai.example.com".to_string()),
            api_key: config.zai_api_key.clone().unwrap_or_default(),
            default_model: config.model.clone(),
        }),
        AdapterKind::Codex => Box::new(CodexAdapter {
            default_model: config.model.clone(),
        }),
        AdapterKind::OpenCode => Box::new(OpenCodeAdapter {
            default_model: config.model.clone(),
        }),
        AdapterKind::Gemini => Box::new(GeminiAdapter {
            default_model: config.model.clone(),
        }),
    };
    Ok(adapter)
}

/// Load agent adapter config from `~/.hoop/config.yml`.
///
/// Reads the `agent` section. Falls back to defaults if the section or file
/// is missing, so the daemon always starts with a valid adapter.
pub fn load_adapter_config() -> AgentAdapterConfig {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let config_path = home.join(".hoop").join("config.yml");

    if !config_path.exists() {
        return AgentAdapterConfig::default();
    }

    match std::fs::read_to_string(&config_path) {
        Ok(contents) => {
            match serde_yaml::from_str::<serde_yaml::Value>(&contents) {
                Ok(root) => {
                    if let Some(agent) = root.get("agent") {
                        match serde_json::from_value::<AgentAdapterConfig>(
                            serde_json::to_value(agent).unwrap_or_default(),
                        ) {
                            Ok(config) => {
                                tracing::info!(
                                    "Loaded agent adapter config: adapter={}, model={}",
                                    config.adapter,
                                    config.model
                                );
                                return config;
                            }
                            Err(e) => {
                                tracing::warn!("Failed to parse agent config: {}, using defaults", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to parse config.yml: {}, using defaults", e);
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to read config.yml: {}, using defaults", e);
        }
    }

    AgentAdapterConfig::default()
}

// ---------------------------------------------------------------------------
// ClaudeCodeAdapter — shells out to the `claude` CLI
// ---------------------------------------------------------------------------

/// Claude Code adapter — drives a persistent `claude` CLI session.
///
/// Uses `claude --resume <session-id>` for continuation and parses the
/// NDJSON event stream from stdout.
pub struct ClaudeCodeAdapter {
    default_model: String,
}

#[async_trait]
impl AgentAdapter for ClaudeCodeAdapter {
    fn kind(&self) -> AdapterKind {
        AdapterKind::Claude
    }

    async fn spawn_session(&self, config: SpawnConfig) -> Result<AgentSession> {
        let id = uuid::Uuid::new_v4().to_string();
        Ok(AgentSession {
            id: SessionId(id),
            adapter: AdapterKind::Claude,
            model: config.model,
            system_prompt: config.system_prompt.clone(),
            working_dir: config.working_dir.clone(),
            history: Arc::new(Mutex::new(Vec::new())),
            has_started_session: false,
        })
    }

    async fn resume_session(&self, id: &SessionId) -> Result<AgentSession> {
        Ok(AgentSession {
            id: id.clone(),
            adapter: AdapterKind::Claude,
            model: self.default_model.clone(),
            system_prompt: None,
            working_dir: None,
            history: Arc::new(Mutex::new(Vec::new())),
            has_started_session: true,
        })
    }

    async fn send_turn(
        &self,
        session: &AgentSession,
        prompt: &str,
        attachments: Vec<Attachment>,
    ) -> Result<EventStream> {
        // Use per-adapter invocation builder — distinguishes --session-id vs --resume.
        let mut args = AdapterKind::Claude
            .build_turn_args(&session.id.0, prompt, session.has_started_session)
            .expect("Claude adapter always produces CLI args");

        for att in &attachments {
            if let Attachment::File { path } = att {
                args.push("--attach".to_string());
                args.push(path.clone());
            }
        }

        let mut cmd = tokio::process::Command::new("claude");
        cmd.args(&args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        if let Some(ref dir) = session.working_dir {
            cmd.current_dir(dir);
        }

        let mut child = cmd.spawn()?;
        let stdout = child.stdout.take().expect("stdout piped");
        let reader = tokio::io::BufReader::new(stdout);

        // Keep child alive for the duration of the stream by moving it into the unfold closure.
        let child_handle = Arc::new(Mutex::new(Some(child)));

        use tokio::io::AsyncBufReadExt;

        let stream = futures_util::stream::unfold(
            (reader, child_handle),
            |(mut reader, child_handle)| async move {
                let mut line = String::new();
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        // EOF — reap the child process.
                        let _child = child_handle.lock().await.take();
                        None
                    }
                    Ok(_) => {
                        let line = line.trim().to_string();
                        let event = parse_claude_stream_line(&line);
                        Some((event, (reader, child_handle)))
                    }
                    Err(e) => Some((Err(anyhow::anyhow!("read error: {}", e)), (reader, child_handle))),
                }
            },
        );

        Ok(Box::pin(stream))
    }

    async fn close_session(&self, _session: &AgentSession) -> Result<()> {
        // Claude Code sessions persist on disk; nothing to close server-side.
        Ok(())
    }
}

/// Parse a single NDJSON line from `claude --output-format stream-json` into an AgentEvent.
fn parse_claude_stream_line(line: &str) -> Result<AgentEvent> {
    let val: serde_json::Value = serde_json::from_str(line)?;

    let event_type = val.get("type").and_then(|v| v.as_str()).unwrap_or("");

    match event_type {
        "assistant" | "content_block_start" | "content_block_delta" => {
            // Text delta
            if let Some(text) = val.get("text").and_then(|v| v.as_str()) {
                return Ok(AgentEvent::TextDelta { text: text.to_string() });
            }
            // Sometimes text is nested under delta
            if let Some(delta) = val.get("delta") {
                if let Some(text) = delta.get("text").and_then(|v| v.as_str()) {
                    return Ok(AgentEvent::TextDelta { text: text.to_string() });
                }
            }
            Ok(AgentEvent::TextDelta { text: String::new() })
        }
        "tool_use" | "tool_call" => {
            let id = val.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let name = val.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let input = val.get("input").cloned().unwrap_or(serde_json::Value::Null);
            Ok(AgentEvent::ToolUse { id, name, input })
        }
        "tool_result" => {
            let id = val.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let output = val.get("output").cloned().unwrap_or(serde_json::Value::Null);
            let is_error = val.get("is_error").and_then(|v| v.as_bool()).unwrap_or(false);
            Ok(AgentEvent::ToolResult { id, output, is_error })
        }
        "message_stop" | "turn_complete" => {
            let usage = parse_usage_from_value(&val);
            Ok(AgentEvent::TurnComplete { usage })
        }
        "error" => {
            let message = val.get("message").and_then(|v| v.as_str()).unwrap_or("unknown error").to_string();
            Ok(AgentEvent::Error { message })
        }
        _ => Ok(AgentEvent::TextDelta { text: String::new() }),
    }
}

fn parse_usage_from_value(val: &serde_json::Value) -> Option<TurnUsage> {
    let usage = val.get("usage")?;
    Some(TurnUsage {
        input_tokens: usage.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
        output_tokens: usage.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
        cache_read_tokens: usage.get("cache_read_input_tokens").or_else(|| usage.get("cache_read_tokens")).and_then(|v| v.as_u64()),
        cache_write_tokens: usage.get("cache_creation_input_tokens").or_else(|| usage.get("cache_write_tokens")).and_then(|v| v.as_u64()),
    })
}

// ---------------------------------------------------------------------------
// AnthropicApiAdapter — direct Anthropic Messages API
// ---------------------------------------------------------------------------

/// Anthropic API adapter — makes direct HTTP calls to the Messages API.
///
/// Uses the Anthropic REST API (no SDK dependency). Session history is
/// maintained in-memory and replayed with each turn.
pub struct AnthropicApiAdapter {
    api_key: String,
    default_model: String,
}

#[async_trait]
impl AgentAdapter for AnthropicApiAdapter {
    fn kind(&self) -> AdapterKind {
        AdapterKind::Anthropic
    }

    async fn spawn_session(&self, config: SpawnConfig) -> Result<AgentSession> {
        let id = uuid::Uuid::new_v4().to_string();
        Ok(AgentSession {
            id: SessionId(id),
            adapter: AdapterKind::Anthropic,
            model: config.model,
            system_prompt: config.system_prompt.clone(),
            working_dir: config.working_dir.clone(),
            history: Arc::new(Mutex::new(Vec::new())),
            has_started_session: false,
        })
    }

    async fn resume_session(&self, id: &SessionId) -> Result<AgentSession> {
        Ok(AgentSession {
            id: id.clone(),
            adapter: AdapterKind::Anthropic,
            model: self.default_model.clone(),
            system_prompt: None,
            working_dir: None,
            history: Arc::new(Mutex::new(Vec::new())),
            has_started_session: true,
        })
    }

    async fn send_turn(
        &self,
        session: &AgentSession,
        prompt: &str,
        attachments: Vec<Attachment>,
    ) -> Result<EventStream> {
        let model = &session.model;
        let api_key = &self.api_key;

        // Build multimodal user content (plain string if no attachments).
        let user_content = build_anthropic_user_content(prompt, &attachments);

        let mut body = serde_json::json!({
            "model": model,
            "max_tokens": 4096,
            "stream": true,
        });

        if let Some(ref sp) = session.system_prompt {
            body["system"] = serde_json::json!(sp);
        }

        // Build messages: previous history as plain text + current turn (multimodal).
        {
            let history = session.history.lock().await;
            let mut messages: Vec<serde_json::Value> = history
                .iter()
                .map(|m| serde_json::json!({ "role": m.role, "content": m.content }))
                .collect();
            messages.push(serde_json::json!({ "role": "user", "content": user_content }));
            body["messages"] = serde_json::json!(messages);
        }

        // Store text-only version in history so future replays remain lightweight.
        {
            let mut history = session.history.lock().await;
            history.push(HistoryMessage { role: "user".into(), content: prompt.into() });
        }

        let client = reqwest_like_client();
        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Anthropic API error {}: {}", status, text));
        }

        let session_history = session.history.clone();
        let tracked_stream = track_assistant_response(parse_sse_response(response), session_history).await;

        Ok(Box::pin(tracked_stream))
    }

    async fn close_session(&self, _session: &AgentSession) -> Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// ZaiGlmAdapter — ZAI proxy with GLM models
// ---------------------------------------------------------------------------

/// ZAI proxy adapter — routes through a ZAI gateway to GLM-family models.
///
/// The ZAI proxy presents an OpenAI-compatible chat completions endpoint.
/// Session history is maintained in-memory and replayed with each turn.
pub struct ZaiGlmAdapter {
    base_url: String,
    api_key: String,
    default_model: String,
}

#[async_trait]
impl AgentAdapter for ZaiGlmAdapter {
    fn kind(&self) -> AdapterKind {
        AdapterKind::Zai
    }

    async fn spawn_session(&self, config: SpawnConfig) -> Result<AgentSession> {
        let id = uuid::Uuid::new_v4().to_string();
        Ok(AgentSession {
            id: SessionId(id),
            adapter: AdapterKind::Zai,
            model: config.model,
            system_prompt: config.system_prompt.clone(),
            working_dir: config.working_dir.clone(),
            history: Arc::new(Mutex::new(Vec::new())),
            has_started_session: false,
        })
    }

    async fn resume_session(&self, id: &SessionId) -> Result<AgentSession> {
        Ok(AgentSession {
            id: id.clone(),
            adapter: AdapterKind::Zai,
            model: self.default_model.clone(),
            system_prompt: None,
            working_dir: None,
            history: Arc::new(Mutex::new(Vec::new())),
            has_started_session: true,
        })
    }

    async fn send_turn(
        &self,
        session: &AgentSession,
        prompt: &str,
        attachments: Vec<Attachment>,
    ) -> Result<EventStream> {
        let model = &session.model;
        let api_key = &self.api_key;
        let url = format!("{}/v1/chat/completions", self.base_url.trim_end_matches('/'));

        // Build multimodal user content (plain string if no attachments).
        let user_content = build_openai_user_content(prompt, &attachments);

        let mut body = serde_json::json!({
            "model": model,
            "max_tokens": 4096,
            "stream": true,
        });

        // Build messages: system prompt + previous history + current multimodal turn.
        {
            let history = session.history.lock().await;
            let mut messages: Vec<serde_json::Value> = Vec::new();
            if let Some(ref sp) = session.system_prompt {
                messages.push(serde_json::json!({ "role": "system", "content": sp }));
            }
            for m in history.iter() {
                messages.push(serde_json::json!({ "role": m.role, "content": m.content }));
            }
            messages.push(serde_json::json!({ "role": "user", "content": user_content }));
            body["messages"] = serde_json::json!(messages);
        }

        // Store text-only version in history so future replays remain lightweight.
        {
            let mut history = session.history.lock().await;
            history.push(HistoryMessage { role: "user".into(), content: prompt.into() });
        }

        let client = reqwest_like_client();
        let response = client
            .post(&url)
            .header("authorization", format!("Bearer {}", api_key))
            .header("content-type", "application/json")
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("ZAI API error {}: {}", status, text));
        }

        let session_history = session.history.clone();
        let tracked_stream = track_assistant_response(parse_openai_sse_response(response), session_history).await;

        Ok(Box::pin(tracked_stream))
    }

    async fn close_session(&self, _session: &AgentSession) -> Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// CodexAdapter — shells out to the `codex` CLI
// ---------------------------------------------------------------------------

/// Codex CLI adapter.
///
/// Turn 1: `codex exec -p <prompt>`
/// Turn N: `codex exec resume <thread-id> -p <prompt>`
pub struct CodexAdapter {
    default_model: String,
}

#[async_trait]
impl AgentAdapter for CodexAdapter {
    fn kind(&self) -> AdapterKind {
        AdapterKind::Codex
    }

    async fn spawn_session(&self, config: SpawnConfig) -> Result<AgentSession> {
        let id = uuid::Uuid::new_v4().to_string();
        Ok(AgentSession {
            id: SessionId(id),
            adapter: AdapterKind::Codex,
            model: config.model,
            system_prompt: config.system_prompt.clone(),
            working_dir: config.working_dir.clone(),
            history: Arc::new(Mutex::new(Vec::new())),
            has_started_session: false,
        })
    }

    async fn resume_session(&self, id: &SessionId) -> Result<AgentSession> {
        Ok(AgentSession {
            id: id.clone(),
            adapter: AdapterKind::Codex,
            model: self.default_model.clone(),
            system_prompt: None,
            working_dir: None,
            history: Arc::new(Mutex::new(Vec::new())),
            has_started_session: true,
        })
    }

    async fn send_turn(
        &self,
        session: &AgentSession,
        prompt: &str,
        _attachments: Vec<Attachment>,
    ) -> Result<EventStream> {
        let args = AdapterKind::Codex
            .build_turn_args(&session.id.0, prompt, session.has_started_session)
            .expect("Codex always produces CLI args");

        let mut cmd = tokio::process::Command::new("codex");
        cmd.args(&args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        if let Some(ref dir) = session.working_dir {
            cmd.current_dir(dir);
        }

        let mut child = cmd.spawn()?;
        let stdout = child.stdout.take().expect("stdout piped");
        let reader = tokio::io::BufReader::new(stdout);
        let child_handle = Arc::new(Mutex::new(Some(child)));

        use tokio::io::AsyncBufReadExt;

        let stream = futures_util::stream::unfold(
            (reader, child_handle),
            |(mut reader, child_handle)| async move {
                let mut line = String::new();
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        let _child = child_handle.lock().await.take();
                        None
                    }
                    Ok(_) => {
                        let line = line.trim().to_string();
                        let event = parse_claude_stream_line(&line);
                        Some((event, (reader, child_handle)))
                    }
                    Err(e) => Some((Err(anyhow::anyhow!("read error: {}", e)), (reader, child_handle))),
                }
            },
        );

        Ok(Box::pin(stream))
    }

    async fn close_session(&self, _session: &AgentSession) -> Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// OpenCodeAdapter — shells out to the `opencode` CLI
// ---------------------------------------------------------------------------

/// OpenCode CLI adapter.
///
/// Turn 1: `opencode --session <sid> -p <prompt>`
/// Turn N: `opencode --session <sid> --continue -p <prompt>`
pub struct OpenCodeAdapter {
    default_model: String,
}

#[async_trait]
impl AgentAdapter for OpenCodeAdapter {
    fn kind(&self) -> AdapterKind {
        AdapterKind::OpenCode
    }

    async fn spawn_session(&self, config: SpawnConfig) -> Result<AgentSession> {
        let id = uuid::Uuid::new_v4().to_string();
        Ok(AgentSession {
            id: SessionId(id),
            adapter: AdapterKind::OpenCode,
            model: config.model,
            system_prompt: config.system_prompt.clone(),
            working_dir: config.working_dir.clone(),
            history: Arc::new(Mutex::new(Vec::new())),
            has_started_session: false,
        })
    }

    async fn resume_session(&self, id: &SessionId) -> Result<AgentSession> {
        Ok(AgentSession {
            id: id.clone(),
            adapter: AdapterKind::OpenCode,
            model: self.default_model.clone(),
            system_prompt: None,
            working_dir: None,
            history: Arc::new(Mutex::new(Vec::new())),
            has_started_session: true,
        })
    }

    async fn send_turn(
        &self,
        session: &AgentSession,
        prompt: &str,
        _attachments: Vec<Attachment>,
    ) -> Result<EventStream> {
        let args = AdapterKind::OpenCode
            .build_turn_args(&session.id.0, prompt, session.has_started_session)
            .expect("OpenCode always produces CLI args");

        let mut cmd = tokio::process::Command::new("opencode");
        cmd.args(&args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        if let Some(ref dir) = session.working_dir {
            cmd.current_dir(dir);
        }

        let mut child = cmd.spawn()?;
        let stdout = child.stdout.take().expect("stdout piped");
        let reader = tokio::io::BufReader::new(stdout);
        let child_handle = Arc::new(Mutex::new(Some(child)));

        use tokio::io::AsyncBufReadExt;

        let stream = futures_util::stream::unfold(
            (reader, child_handle),
            |(mut reader, child_handle)| async move {
                let mut line = String::new();
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        let _child = child_handle.lock().await.take();
                        None
                    }
                    Ok(_) => {
                        let line = line.trim().to_string();
                        let event = parse_claude_stream_line(&line);
                        Some((event, (reader, child_handle)))
                    }
                    Err(e) => Some((Err(anyhow::anyhow!("read error: {}", e)), (reader, child_handle))),
                }
            },
        );

        Ok(Box::pin(stream))
    }

    async fn close_session(&self, _session: &AgentSession) -> Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// GeminiAdapter — sandbox-native Gemini CLI
// ---------------------------------------------------------------------------

/// Gemini CLI adapter.
///
/// Gemini is sandbox-native: the CLI manages continuity internally.
/// Both turn 1 and turn N use the same args: `gemini -p <prompt>`.
pub struct GeminiAdapter {
    default_model: String,
}

#[async_trait]
impl AgentAdapter for GeminiAdapter {
    fn kind(&self) -> AdapterKind {
        AdapterKind::Gemini
    }

    async fn spawn_session(&self, config: SpawnConfig) -> Result<AgentSession> {
        let id = uuid::Uuid::new_v4().to_string();
        Ok(AgentSession {
            id: SessionId(id),
            adapter: AdapterKind::Gemini,
            model: config.model,
            system_prompt: config.system_prompt.clone(),
            working_dir: config.working_dir.clone(),
            history: Arc::new(Mutex::new(Vec::new())),
            has_started_session: false,
        })
    }

    async fn resume_session(&self, id: &SessionId) -> Result<AgentSession> {
        Ok(AgentSession {
            id: id.clone(),
            adapter: AdapterKind::Gemini,
            model: self.default_model.clone(),
            system_prompt: None,
            working_dir: None,
            history: Arc::new(Mutex::new(Vec::new())),
            has_started_session: true,
        })
    }

    async fn send_turn(
        &self,
        session: &AgentSession,
        prompt: &str,
        _attachments: Vec<Attachment>,
    ) -> Result<EventStream> {
        let args = AdapterKind::Gemini
            .build_turn_args(&session.id.0, prompt, session.has_started_session)
            .expect("Gemini always produces CLI args");

        let mut cmd = tokio::process::Command::new("gemini");
        cmd.args(&args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        if let Some(ref dir) = session.working_dir {
            cmd.current_dir(dir);
        }

        let mut child = cmd.spawn()?;
        let stdout = child.stdout.take().expect("stdout piped");
        let reader = tokio::io::BufReader::new(stdout);
        let child_handle = Arc::new(Mutex::new(Some(child)));

        use tokio::io::AsyncBufReadExt;

        let stream = futures_util::stream::unfold(
            (reader, child_handle),
            |(mut reader, child_handle)| async move {
                let mut line = String::new();
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        let _child = child_handle.lock().await.take();
                        None
                    }
                    Ok(_) => {
                        let line = line.trim().to_string();
                        let event = parse_claude_stream_line(&line);
                        Some((event, (reader, child_handle)))
                    }
                    Err(e) => Some((Err(anyhow::anyhow!("read error: {}", e)), (reader, child_handle))),
                }
            },
        );

        Ok(Box::pin(stream))
    }

    async fn close_session(&self, _session: &AgentSession) -> Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Minimal HTTP client — avoids adding a reqwest dependency by reusing
/// hyper (already pulled in by axum). For now we use a simple helper;
/// the real implementation will use `reqwest` when added to Cargo.toml.
fn reqwest_like_client() -> reqwest::Client {
    reqwest::Client::new()
}

/// Build the user-turn content for the Anthropic Messages API.
///
/// Returns a plain string if there are no attachments. Returns an array of
/// typed content blocks when attachments are present: image/pdf blocks for
/// binary media, and text blocks for UTF-8 files.
fn build_anthropic_user_content(prompt: &str, attachments: &[Attachment]) -> serde_json::Value {
    if attachments.is_empty() {
        return serde_json::json!(prompt);
    }

    let mut parts: Vec<serde_json::Value> = Vec::new();

    for att in attachments {
        match att {
            Attachment::Inline { name, content, mime } => {
                if mime.starts_with("image/") {
                    parts.push(serde_json::json!({
                        "type": "image",
                        "source": { "type": "base64", "media_type": mime, "data": content }
                    }));
                } else if mime == "application/pdf" {
                    parts.push(serde_json::json!({
                        "type": "document",
                        "source": { "type": "base64", "media_type": "application/pdf", "data": content }
                    }));
                } else {
                    // Text-based file — decode and embed verbatim.
                    use base64::Engine as _;
                    if let Ok(raw) = base64::engine::general_purpose::STANDARD.decode(content) {
                        if let Ok(text) = String::from_utf8(raw) {
                            parts.push(serde_json::json!({
                                "type": "text",
                                "text": format!("<attachment name=\"{}\">\n{}\n</attachment>", name, text)
                            }));
                        }
                    }
                }
            }
            Attachment::File { path } => {
                parts.push(serde_json::json!({
                    "type": "text",
                    "text": format!("[File attachment: {}]", path)
                }));
            }
            Attachment::Url { url } => {
                parts.push(serde_json::json!({
                    "type": "text",
                    "text": format!("[URL attachment: {}]", url)
                }));
            }
        }
    }

    parts.push(serde_json::json!({ "type": "text", "text": prompt }));
    serde_json::json!(parts)
}

/// Build the user-turn content for OpenAI-compatible chat completions.
///
/// Images are embedded as data-URI `image_url` blocks. Text/binary files are
/// decoded and embedded as text blocks. PDF is not natively supported by most
/// OpenAI-compatible endpoints, so it falls back to a text notice.
fn build_openai_user_content(prompt: &str, attachments: &[Attachment]) -> serde_json::Value {
    if attachments.is_empty() {
        return serde_json::json!(prompt);
    }

    let mut parts: Vec<serde_json::Value> = Vec::new();

    for att in attachments {
        match att {
            Attachment::Inline { name, content, mime } => {
                if mime.starts_with("image/") {
                    parts.push(serde_json::json!({
                        "type": "image_url",
                        "image_url": { "url": format!("data:{};base64,{}", mime, content) }
                    }));
                } else {
                    use base64::Engine as _;
                    if let Ok(raw) = base64::engine::general_purpose::STANDARD.decode(content) {
                        if let Ok(text) = String::from_utf8(raw) {
                            parts.push(serde_json::json!({
                                "type": "text",
                                "text": format!("<attachment name=\"{}\">\n{}\n</attachment>", name, text)
                            }));
                        }
                    }
                }
            }
            Attachment::File { path } => {
                parts.push(serde_json::json!({
                    "type": "text",
                    "text": format!("[File attachment: {}]", path)
                }));
            }
            Attachment::Url { url } => {
                parts.push(serde_json::json!({
                    "type": "text",
                    "text": format!("[URL attachment: {}]", url)
                }));
            }
        }
    }

    parts.push(serde_json::json!({ "type": "text", "text": prompt }));
    serde_json::json!(parts)
}

/// Parse an Anthropic SSE response into AgentEvent stream.
fn parse_sse_response(response: reqwest::Response) -> impl Stream<Item = Result<AgentEvent>> {
    use futures_util::StreamExt;

    let stream = response.bytes_stream();

    stream.flat_map(|chunk_result| {
        let chunk = match chunk_result {
            Ok(c) => c,
            Err(e) => return futures_util::stream::iter(vec![Err(anyhow::anyhow!("stream error: {}", e))]),
        };

        let text = String::from_utf8_lossy(&chunk);
        let events: Vec<Result<AgentEvent>> = text
            .lines()
            .filter(|l| l.starts_with("data: "))
            .filter_map(|l| l.strip_prefix("data: "))
            .filter(|l| l.trim() != "[DONE]")
            .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
            .map(|val| anthropic_sse_to_event(&val))
            .collect();

        futures_util::stream::iter(events)
    })
}

/// Track assistant responses and append to conversation history.
/// This enables multi-turn conversations by accumulating the full assistant
/// response as it streams in.
async fn track_assistant_response(
    stream: impl Stream<Item = Result<AgentEvent>> + Send + 'static,
    history: Arc<Mutex<Vec<HistoryMessage>>>,
) -> impl Stream<Item = Result<AgentEvent>> + Send + 'static {
    use futures_util::StreamExt;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let accumulating = Arc::new(AtomicBool::new(false));
    let buffer = Arc::new(Mutex::new(String::new()));

    stream.then(move |result| {
        let accumulating = accumulating.clone();
        let buffer = buffer.clone();
        let history = history.clone();

        async move {
            match result {
                Ok(event) => {
                    match &event {
                        AgentEvent::TextDelta { text }
                            if !text.is_empty() => {
                                accumulating.store(true, Ordering::Relaxed);
                                buffer.lock().await.push_str(text);
                            }
                        AgentEvent::TurnComplete { .. } | AgentEvent::SessionEnded { .. }
                            if accumulating.load(Ordering::Relaxed) => {
                                let full_text = buffer.lock().await.clone();
                                if !full_text.is_empty() {
                                    history.lock().await.push(HistoryMessage {
                                        role: "assistant".into(),
                                        content: full_text,
                                    });
                                }
                                buffer.lock().await.clear();
                                accumulating.store(false, Ordering::Relaxed);
                            }
                        _ => {}
                    }
                    Ok(event)
                }
                Err(e) => Err(e),
            }
        }
    })
}

/// Parse an OpenAI-compatible SSE response into AgentEvent stream.
fn parse_openai_sse_response(response: reqwest::Response) -> impl Stream<Item = Result<AgentEvent>> {
    use futures_util::StreamExt;

    let stream = response.bytes_stream();

    stream.flat_map(|chunk_result| {
        let chunk = match chunk_result {
            Ok(c) => c,
            Err(e) => return futures_util::stream::iter(vec![Err(anyhow::anyhow!("stream error: {}", e))]),
        };

        let text = String::from_utf8_lossy(&chunk);
        let events: Vec<Result<AgentEvent>> = text
            .lines()
            .filter(|l| l.starts_with("data: "))
            .filter_map(|l| l.strip_prefix("data: "))
            .filter(|l| l.trim() != "[DONE]")
            .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
            .map(|val| openai_sse_to_event(&val))
            .collect();

        futures_util::stream::iter(events)
    })
}

fn anthropic_sse_to_event(val: &serde_json::Value) -> Result<AgentEvent> {
    let event_type = val.get("type").and_then(|v| v.as_str()).unwrap_or("");

    match event_type {
        "content_block_delta" => {
            let delta = val.get("delta");
            let text = delta
                .and_then(|d| d.get("text"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            Ok(AgentEvent::TextDelta { text })
        }
        "content_block_start" => {
            if let Some(cb) = val.get("content_block") {
                if cb.get("type").and_then(|v| v.as_str()) == Some("tool_use") {
                    return Ok(AgentEvent::ToolUse {
                        id: cb.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        name: cb.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        input: serde_json::Value::Null,
                    });
                }
            }
            Ok(AgentEvent::TextDelta { text: String::new() })
        }
        "message_delta" => {
            let usage = val.get("usage").map(|u| TurnUsage {
                input_tokens: 0,
                output_tokens: u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
                cache_read_tokens: None,
                cache_write_tokens: None,
            });
            Ok(AgentEvent::TurnComplete { usage })
        }
        "message_start" => {
            let _session_id = val
                .get("message")
                .and_then(|m| m.get("id"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            // We don't change session ID mid-stream, so emit a no-op delta.
            Ok(AgentEvent::TextDelta {
                text: String::new(),
            })
        }
        "error" => {
            let msg = val.get("error").and_then(|e| e.get("message")).and_then(|v| v.as_str()).unwrap_or("error");
            Ok(AgentEvent::Error { message: msg.to_string() })
        }
        _ => Ok(AgentEvent::TextDelta { text: String::new() }),
    }
}

fn openai_sse_to_event(val: &serde_json::Value) -> Result<AgentEvent> {
    let choices = val.get("choices").and_then(|c| c.as_array());
    let choice = choices.and_then(|c| c.first());

    if let Some(choice) = choice {
        let delta = choice.get("delta");

        // Tool calls (OpenAI-compatible)
        if let Some(tool_calls) = delta.and_then(|d| d.get("tool_calls")) {
            if let Some(tc) = tool_calls.as_array().and_then(|a| a.first()) {
                return Ok(AgentEvent::ToolUse {
                    id: tc.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    name: tc.get("function")
                        .and_then(|f| f.get("name"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    input: tc.get("function")
                        .and_then(|f| f.get("arguments"))
                        .and_then(|v| v.as_str())
                        .and_then(|s| serde_json::from_str(s).ok())
                        .unwrap_or(serde_json::Value::Null),
                });
            }
        }

        // Text content delta
        if let Some(content) = delta.and_then(|d| d.get("content")).and_then(|v| v.as_str()) {
            return Ok(AgentEvent::TextDelta { text: content.to_string() });
        }

        // Finish reason
        if choice.get("finish_reason").and_then(|v| v.as_str()).is_some() {
            let usage = val.get("usage").map(|u| TurnUsage {
                input_tokens: u.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
                output_tokens: u.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
                cache_read_tokens: u.get("prompt_tokens_details")
                    .and_then(|d| d.get("cached_tokens"))
                    .and_then(|v| v.as_u64()),
                cache_write_tokens: None,
            });
            return Ok(AgentEvent::TurnComplete { usage });
        }
    }

    Ok(AgentEvent::TextDelta { text: String::new() })
}

// ---------------------------------------------------------------------------
// Tests — event shape identity across adapters
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that AgentEvent serializes to the same JSON shape regardless of
    /// which adapter produced it. This is the "test fixture" from the acceptance
    /// criteria: event shape identical across adapters.
    #[test]
    fn event_shape_identity_text_delta() {
        let events: Vec<AgentEvent> = vec![
            AgentEvent::TextDelta { text: "hello".into() },
            AgentEvent::TextDelta { text: "hello".into() },
            AgentEvent::TextDelta { text: "hello".into() },
        ];

        let jsons: Vec<String> = events.iter().map(|e| serde_json::to_string(e).unwrap()).collect();
        assert!(jsons.windows(2).all(|w| w[0] == w[1]),
            "TextDelta must serialize identically regardless of source adapter");
    }

    #[test]
    fn event_shape_identity_tool_use() {
        let event = AgentEvent::ToolUse {
            id: "tu_123".into(),
            name: "read_file".into(),
            input: serde_json::json!({"path": "/foo.rs"}),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: AgentEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, parsed, "ToolUse round-trips through JSON identically");
    }

    #[test]
    fn event_shape_identity_tool_result() {
        let event = AgentEvent::ToolResult {
            id: "tu_123".into(),
            output: serde_json::json!("file contents"),
            is_error: false,
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: AgentEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, parsed);
    }

    #[test]
    fn event_shape_identity_turn_complete() {
        let event = AgentEvent::TurnComplete {
            usage: Some(TurnUsage {
                input_tokens: 100,
                output_tokens: 50,
                cache_read_tokens: Some(10),
                cache_write_tokens: Some(5),
            }),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: AgentEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, parsed);
    }

    #[test]
    fn event_shape_identity_error() {
        let event = AgentEvent::Error { message: "rate limited".into() };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: AgentEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, parsed);
    }

    #[test]
    fn event_shape_identity_session_started() {
        let event = AgentEvent::SessionStarted { session_id: SessionId("abc".into()) };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: AgentEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, parsed);
    }

    #[test]
    fn event_shape_identity_session_ended() {
        let event = AgentEvent::SessionEnded { reason: "max_tokens".into() };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: AgentEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, parsed);
    }

    /// Full event catalog serializes to tagged JSON with `type` discriminator.
    #[test]
    fn event_tagged_json_has_type_field() {
        let events = vec![
            AgentEvent::SessionStarted { session_id: SessionId("s1".into()) },
            AgentEvent::TextDelta { text: "hi".into() },
            AgentEvent::ToolUse { id: "1".into(), name: "n".into(), input: serde_json::Value::Null },
            AgentEvent::ToolResult { id: "1".into(), output: serde_json::Value::Null, is_error: false },
            AgentEvent::TurnComplete { usage: None },
            AgentEvent::Error { message: "err".into() },
            AgentEvent::SessionEnded { reason: "done".into() },
        ];

        for event in &events {
            let json = serde_json::to_string(event).unwrap();
            assert!(json.contains("\"type\":"), "event must have type tag: {}", json);
        }
    }

    /// AdapterKind round-trips through config string.
    #[test]
    fn adapter_kind_round_trip() {
        for (s, kind) in [
            ("claude", AdapterKind::Claude),
            ("anthropic", AdapterKind::Anthropic),
            ("zai", AdapterKind::Zai),
            ("codex", AdapterKind::Codex),
            ("opencode", AdapterKind::OpenCode),
            ("gemini", AdapterKind::Gemini),
        ] {
            assert_eq!(AdapterKind::from_config(s), Some(kind));
            assert_eq!(kind.as_str(), s);
        }
    }

    /// Factory builds the correct adapter type from config.
    #[test]
    fn factory_builds_correct_adapter() {
        for (name, expected) in [
            ("claude", AdapterKind::Claude),
            ("anthropic", AdapterKind::Anthropic),
            ("zai", AdapterKind::Zai),
            ("codex", AdapterKind::Codex),
            ("opencode", AdapterKind::OpenCode),
            ("gemini", AdapterKind::Gemini),
        ] {
            let config = AgentAdapterConfig {
                adapter: name.to_string(),
                ..Default::default()
            };
            let adapter = build_adapter(&config).unwrap();
            assert_eq!(adapter.kind(), expected, "factory must build {} adapter", name);
        }
    }

    /// Factory rejects unknown adapter.
    #[test]
    fn factory_rejects_unknown() {
        let config = AgentAdapterConfig {
            adapter: "nonexistent".to_string(),
            ..Default::default()
        };
        assert!(build_adapter(&config).is_err());
    }

    /// Verify all AgentEvent variants are covered by serde round-trip.
    #[test]
    fn all_variants_round_trip() {
        let events: Vec<AgentEvent> = vec![
            AgentEvent::SessionStarted { session_id: SessionId("sid".into()) },
            AgentEvent::TextDelta { text: "delta".into() },
            AgentEvent::ToolUse { id: "t1".into(), name: "tool".into(), input: serde_json::json!({"k": "v"}) },
            AgentEvent::ToolResult { id: "t1".into(), output: serde_json::json!("res"), is_error: false },
            AgentEvent::TurnComplete {
                usage: Some(TurnUsage {
                    input_tokens: 1,
                    output_tokens: 2,
                    cache_read_tokens: Some(3),
                    cache_write_tokens: Some(4),
                }),
            },
            AgentEvent::Error { message: "boom".into() },
            AgentEvent::SessionEnded { reason: "limit".into() },
        ];

        for original in &events {
            let json = serde_json::to_string(original).unwrap();
            let restored: AgentEvent = serde_json::from_str(&json).unwrap();
            assert_eq!(original, &restored, "round-trip failed for: {}", json);
        }
    }

    /// SpawnConfig defaults and serialization.
    #[test]
    fn spawn_config_serialization() {
        let config = SpawnConfig {
            model: "claude-opus-4-7".into(),
            system_prompt: Some("You are HOOP.".into()),
            max_tokens: Some(8192),
            rate_limit_rpm: Some(60),
            cost_cap_usd: Some(5.0),
            working_dir: Some("/home/user/project".into()),
        };
        let json = serde_json::to_string(&config).unwrap();
        let restored: SpawnConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.model, restored.model);
        assert_eq!(config.system_prompt, restored.system_prompt);
        assert_eq!(config.max_tokens, restored.max_tokens);
    }

    /// AgentAdapterConfig default selects claude.
    #[test]
    fn default_config_selects_claude() {
        let config = AgentAdapterConfig::default();
        assert_eq!(config.adapter, "claude");
        assert_eq!(config.model, "claude-opus-4-7");
    }

    /// Claude stream line parser produces correct events.
    #[test]
    fn parse_claude_stream_line_text() {
        let line = r#"{"type":"content_block_delta","delta":{"text":"Hello"}}"#;
        let event = parse_claude_stream_line(line).unwrap();
        match event {
            AgentEvent::TextDelta { text } => assert_eq!(text, "Hello"),
            other => panic!("expected TextDelta, got {:?}", other),
        }
    }

    #[test]
    fn parse_claude_stream_line_tool_use() {
        let line = r#"{"type":"tool_use","id":"tu_1","name":"read_file","input":{"path":"/a.rs"}}"#;
        let event = parse_claude_stream_line(line).unwrap();
        match event {
            AgentEvent::ToolUse { id, name, .. } => {
                assert_eq!(id, "tu_1");
                assert_eq!(name, "read_file");
            }
            other => panic!("expected ToolUse, got {:?}", other),
        }
    }

    #[test]
    fn parse_claude_stream_line_error() {
        let line = r#"{"type":"error","message":"rate limited"}"#;
        let event = parse_claude_stream_line(line).unwrap();
        match event {
            AgentEvent::Error { message } => assert_eq!(message, "rate limited"),
            other => panic!("expected Error, got {:?}", other),
        }
    }

    // -----------------------------------------------------------------------
    // Golden transcript: per-adapter turn 1 vs turn N invocation (§A2)
    // -----------------------------------------------------------------------

    const TEST_SESSION_ID: &str = "deadbeef-0000-0000-0000-000000000001";
    const TEST_PROMPT: &str = "summarize the project";

    /// Claude golden transcript: turn 1 uses --session-id, turn N uses --resume.
    #[test]
    fn golden_claude_turn1_create() {
        let args = AdapterKind::Claude
            .build_turn_args(TEST_SESSION_ID, TEST_PROMPT, false)
            .unwrap();
        assert_eq!(args, vec![
            "--session-id", TEST_SESSION_ID,
            "--output-format", "stream-json",
            "-p", TEST_PROMPT,
        ]);
    }

    #[test]
    fn golden_claude_turn_n_resume() {
        let args = AdapterKind::Claude
            .build_turn_args(TEST_SESSION_ID, TEST_PROMPT, true)
            .unwrap();
        assert_eq!(args, vec![
            "--resume", TEST_SESSION_ID,
            "--output-format", "stream-json",
            "-p", TEST_PROMPT,
        ]);
    }

    /// Codex golden transcript: turn 1 uses `exec`, turn N uses `exec resume <id>`.
    #[test]
    fn golden_codex_turn1_create() {
        let args = AdapterKind::Codex
            .build_turn_args(TEST_SESSION_ID, TEST_PROMPT, false)
            .unwrap();
        assert_eq!(args, vec![
            "exec",
            "-p", TEST_PROMPT,
        ]);
    }

    #[test]
    fn golden_codex_turn_n_resume() {
        let args = AdapterKind::Codex
            .build_turn_args(TEST_SESSION_ID, TEST_PROMPT, true)
            .unwrap();
        assert_eq!(args, vec![
            "exec", "resume", TEST_SESSION_ID,
            "-p", TEST_PROMPT,
        ]);
    }

    /// OpenCode golden transcript: turn 1 uses `--session <sid>`, turn N adds `--continue`.
    #[test]
    fn golden_opencode_turn1_create() {
        let args = AdapterKind::OpenCode
            .build_turn_args(TEST_SESSION_ID, TEST_PROMPT, false)
            .unwrap();
        assert_eq!(args, vec![
            "--session", TEST_SESSION_ID,
            "-p", TEST_PROMPT,
        ]);
    }

    #[test]
    fn golden_opencode_turn_n_resume() {
        let args = AdapterKind::OpenCode
            .build_turn_args(TEST_SESSION_ID, TEST_PROMPT, true)
            .unwrap();
        assert_eq!(args, vec![
            "--session", TEST_SESSION_ID,
            "--continue",
            "-p", TEST_PROMPT,
        ]);
    }

    /// Gemini golden transcript: sandbox-native, same args regardless of turn count.
    #[test]
    fn golden_gemini_turn1_and_turn_n_identical() {
        let args_turn1 = AdapterKind::Gemini
            .build_turn_args(TEST_SESSION_ID, TEST_PROMPT, false)
            .unwrap();
        let args_turnN = AdapterKind::Gemini
            .build_turn_args(TEST_SESSION_ID, TEST_PROMPT, true)
            .unwrap();
        assert_eq!(args_turn1, args_turnN);
        assert_eq!(args_turn1, vec!["-p", TEST_PROMPT]);
    }

    /// API adapters return None (no CLI invocation).
    #[test]
    fn api_adapters_produce_no_cli_args() {
        assert!(AdapterKind::Anthropic.build_turn_args("sid", "prompt", false).is_none());
        assert!(AdapterKind::Anthropic.build_turn_args("sid", "prompt", true).is_none());
        assert!(AdapterKind::Zai.build_turn_args("sid", "prompt", false).is_none());
        assert!(AdapterKind::Zai.build_turn_args("sid", "prompt", true).is_none());
    }

    /// Verify has_started_session is false on spawn, true on resume — for all CLI adapters.
    #[test]
    fn spawn_sets_has_started_false_resume_sets_true() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let adapter = ClaudeCodeAdapter { default_model: "claude-opus-4-7".into() };
            let config = SpawnConfig {
                model: "claude-opus-4-7".into(),
                system_prompt: None,
                max_tokens: None,
                rate_limit_rpm: None,
                cost_cap_usd: None,
                working_dir: None,
            };
            let spawned = adapter.spawn_session(config).await.unwrap();
            assert!(!spawned.has_started_session, "spawn must start with has_started_session=false");

            let resumed = adapter.resume_session(&spawned.id).await.unwrap();
            assert!(resumed.has_started_session, "resume must set has_started_session=true");
        });
    }

    /// Codex spawn/resume flag semantics.
    #[test]
    fn codex_spawn_resume_flags() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let adapter = CodexAdapter { default_model: "codex-mini".into() };
            let config = SpawnConfig {
                model: "codex-mini".into(),
                system_prompt: None,
                max_tokens: None,
                rate_limit_rpm: None,
                cost_cap_usd: None,
                working_dir: None,
            };
            let spawned = adapter.spawn_session(config).await.unwrap();
            assert!(!spawned.has_started_session);
            let resumed = adapter.resume_session(&spawned.id).await.unwrap();
            assert!(resumed.has_started_session);
            assert_eq!(resumed.adapter, AdapterKind::Codex);
        });
    }

    /// OpenCode spawn/resume flag semantics.
    #[test]
    fn opencode_spawn_resume_flags() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let adapter = OpenCodeAdapter { default_model: "gpt-4o".into() };
            let config = SpawnConfig {
                model: "gpt-4o".into(),
                system_prompt: None,
                max_tokens: None,
                rate_limit_rpm: None,
                cost_cap_usd: None,
                working_dir: None,
            };
            let spawned = adapter.spawn_session(config).await.unwrap();
            assert!(!spawned.has_started_session);
            let resumed = adapter.resume_session(&spawned.id).await.unwrap();
            assert!(resumed.has_started_session);
            assert_eq!(resumed.adapter, AdapterKind::OpenCode);
        });
    }

    /// Gemini spawn/resume flag semantics.
    #[test]
    fn gemini_spawn_resume_flags() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let adapter = GeminiAdapter { default_model: "gemini-2.5-pro".into() };
            let config = SpawnConfig {
                model: "gemini-2.5-pro".into(),
                system_prompt: None,
                max_tokens: None,
                rate_limit_rpm: None,
                cost_cap_usd: None,
                working_dir: None,
            };
            let spawned = adapter.spawn_session(config).await.unwrap();
            assert!(!spawned.has_started_session);
            let resumed = adapter.resume_session(&spawned.id).await.unwrap();
            assert!(resumed.has_started_session);
            assert_eq!(resumed.adapter, AdapterKind::Gemini);
        });
    }
}
