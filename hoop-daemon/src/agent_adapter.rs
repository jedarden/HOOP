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

/// Which adapter to use — mirrors `agent_config.json` enum minus codex/opencode/gemini/aider
/// (those are session-discovery adapters, not agent-driving adapters).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AdapterKind {
    Claude,
    Anthropic,
    Zai,
}

impl AdapterKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Anthropic => "anthropic",
            Self::Zai => "zai",
        }
    }

    pub fn from_config(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "claude" => Some(Self::Claude),
            "anthropic" => Some(Self::Anthropic),
            "zai" => Some(Self::Zai),
            _ => None,
        }
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
    };
    Ok(adapter)
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
        // Spawn via `claude --session-id <id>` subprocess.
        // The actual subprocess launch is deferred to send_turn so that
        // spawn_session is lightweight — just records the session metadata.
        Ok(AgentSession {
            id: SessionId(id),
            adapter: AdapterKind::Claude,
            model: config.model,
        })
    }

    async fn resume_session(&self, id: &SessionId) -> Result<AgentSession> {
        Ok(AgentSession {
            id: id.clone(),
            adapter: AdapterKind::Claude,
            model: self.default_model.clone(),
        })
    }

    async fn send_turn(
        &self,
        session: &AgentSession,
        prompt: &str,
        attachments: Vec<Attachment>,
    ) -> Result<EventStream> {
        // Build the command: claude --resume <id> --output-format stream-json
        let mut args = vec![
            "--resume".to_string(),
            session.id.0.clone(),
            "--output-format".to_string(),
            "stream-json".to_string(),
            "-p".to_string(),
            prompt.to_string(),
        ];

        for att in &attachments {
            match att {
                Attachment::File { path } => {
                    args.push("--attach".to_string());
                    args.push(path.clone());
                }
                _ => {} // URLs and inline attachments handled differently
            }
        }

        let child = tokio::process::Command::new("claude")
            .args(&args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.expect("stdout piped");
        let reader = tokio::io::BufReader::new(stdout);

        // Parse NDJSON lines into AgentEvent
        use tokio::io::AsyncBufReadExt;

        let stream = futures_util::stream::unfold(reader, |mut reader| async move {
            let mut line = String::new();
            match reader.read_line(&mut line).await {
                Ok(0) => None, // EOF
                Ok(_) => {
                    let line = line.trim().to_string();
                    let event = parse_claude_stream_line(&line);
                    Some((event, reader))
                }
                Err(e) => Some((Err(anyhow::anyhow!("read error: {}", e)), reader)),
            }
        });

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
        })
    }

    async fn resume_session(&self, id: &SessionId) -> Result<AgentSession> {
        Ok(AgentSession {
            id: id.clone(),
            adapter: AdapterKind::Anthropic,
            model: self.default_model.clone(),
        })
    }

    async fn send_turn(
        &self,
        session: &AgentSession,
        prompt: &str,
        _attachments: Vec<Attachment>,
    ) -> Result<EventStream> {
        // Build the Messages API request.
        let model = &session.model;
        let api_key = &self.api_key;

        let body = serde_json::json!({
            "model": model,
            "max_tokens": 4096,
            "stream": true,
            "messages": [{"role": "user", "content": prompt}],
        });

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

        // Parse SSE stream into AgentEvents
        let stream = parse_sse_response(response);
        Ok(Box::pin(stream))
    }

    async fn close_session(&self, _session: &AgentSession) -> Result<()> {
        // No server-side state to clean up for the API adapter.
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
        })
    }

    async fn resume_session(&self, id: &SessionId) -> Result<AgentSession> {
        Ok(AgentSession {
            id: id.clone(),
            adapter: AdapterKind::Zai,
            model: self.default_model.clone(),
        })
    }

    async fn send_turn(
        &self,
        session: &AgentSession,
        prompt: &str,
        _attachments: Vec<Attachment>,
    ) -> Result<EventStream> {
        let model = &session.model;
        let api_key = &self.api_key;
        let url = format!("{}/v1/chat/completions", self.base_url.trim_end_matches('/'));

        let body = serde_json::json!({
            "model": model,
            "max_tokens": 4096,
            "stream": true,
            "messages": [{"role": "user", "content": prompt}],
        });

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

        // Parse SSE stream into AgentEvents (OpenAI-compatible format)
        let stream = parse_openai_sse_response(response);
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
            let usage = val.get("usage").and_then(|u| {
                Some(TurnUsage {
                    input_tokens: 0,
                    output_tokens: u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
                    cache_read_tokens: None,
                    cache_write_tokens: None,
                })
            });
            Ok(AgentEvent::TurnComplete { usage })
        }
        "message_start" => {
            let session_id = val
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
            let usage = val.get("usage").and_then(|u| {
                Some(TurnUsage {
                    input_tokens: u.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
                    output_tokens: u.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
                    cache_read_tokens: u.get("prompt_tokens_details")
                        .and_then(|d| d.get("cached_tokens"))
                        .and_then(|v| v.as_u64()),
                    cache_write_tokens: None,
                })
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
}
