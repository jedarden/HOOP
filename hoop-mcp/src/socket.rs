//! Unix domain socket server for MCP
//!
//! Runs the stdio-based MCP protocol over a Unix domain socket with proper permissions.

use anyhow::Result;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tokio::net::UnixListener;
use tokio::signal;
use tracing::{error, info, warn};

/// MCP socket server configuration
#[derive(Debug, Clone)]
pub struct SocketConfig {
    /// Path to the Unix domain socket
    pub socket_path: PathBuf,
    /// Actor name for audit logging
    pub actor: String,
}

impl Default for SocketConfig {
    fn default() -> Self {
        let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.push(".hoop");
        home.push("mcp.sock");
        Self {
            socket_path: home,
            actor: "mcp-client".to_string(),
        }
    }
}

/// Run the MCP server over Unix domain socket
pub async fn run_socket_server(config: SocketConfig) -> Result<()> {
    let socket_path = &config.socket_path;

    // Ensure parent directory exists
    if let Some(parent) = socket_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Remove existing socket if present
    if socket_path.exists() {
        fs::remove_file(socket_path)?;
    }

    let listener = UnixListener::bind(socket_path)?;

    // Set socket permissions: user read/write only (0o600)
    // This ensures only the same user can connect (§13 security)
    let expected_mode = 0o600;
    fs::set_permissions(socket_path, fs::Permissions::from_mode(expected_mode))
        .map_err(|e| {
            // Clean failure: remove socket if mode set fails
            let _ = fs::remove_file(socket_path);
            anyhow::anyhow!("failed to set socket mode 0{:o}: {}. Socket removed for security.", expected_mode, e)
        })?;

    // Verify the mode was set correctly (startup verification)
    let metadata = fs::metadata(socket_path)?;
    let actual_mode = metadata.permissions().mode() & 0o777;
    if actual_mode != expected_mode {
        // Clean failure: remove socket if verification fails
        let _ = fs::remove_file(socket_path);
        return Err(anyhow::anyhow!(
            "socket mode verification failed: expected 0{:o}, got 0{:o}. Socket removed for security.",
            expected_mode, actual_mode
        ));
    }

    info!(
        "MCP socket listening at {} (mode 0{:o}, same-user only, no TCP)",
        socket_path.display(),
        actual_mode
    );

    // Handle shutdown signals
    let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())?;
    let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())?;

    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((socket, _addr)) => {
                        let actor = config.actor.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_socket_connection(socket, actor).await {
                                error!("Socket connection handler error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Socket accept error: {}", e);
                    }
                }
            }
            _ = sigterm.recv() => {
                info!("Received SIGTERM, shutting down");
                break;
            }
            _ = sigint.recv() => {
                info!("Received SIGINT, shutting down");
                break;
            }
        }
    }

    // Clean up socket on shutdown
    if socket_path.exists() {
        let _ = fs::remove_file(socket_path);
        info!("Removed socket at {}", socket_path.display());
    }

    Ok(())
}

/// Handle a single socket connection
async fn handle_socket_connection(
    mut socket: tokio::net::UnixStream,
    actor: String,
) -> Result<()> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let (reader, mut writer) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    // Initialize MCP server state for this connection
    let server_state = crate::tools::McpServerState::new(actor)?;

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break; // Connection closed
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Parse JSON-RPC request
        let request: crate::protocol::JsonRpcRequest = match serde_json::from_str(line) {
            Ok(req) => req,
            Err(e) => {
                let error_response = crate::protocol::JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(crate::protocol::JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                };
                let response_json = serde_json::to_string(&error_response)?;
                writer.write_all(response_json.as_bytes()).await?;
                writer.write_all(b"\n").await?;
                writer.flush().await?;
                continue;
            }
        };

        // Check if shutdown is requested before handling
        let is_shutdown = matches!(request.method, crate::protocol::Method::Shutdown(_));

        // Handle request
        let response = handle_request(request.method, &server_state);

        // Send response
        let response_json = serde_json::to_string(&response)?;
        writer.write_all(response_json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;

        // Shutdown if requested
        if is_shutdown {
            break;
        }
    }

    Ok(())
}

/// Handle a single MCP request
fn handle_request(
    method: crate::protocol::Method,
    server_state: &crate::tools::McpServerState,
) -> crate::protocol::JsonRpcResponse {
    match method {
        crate::protocol::Method::Initialize(params) => {
            info!(
                "MCP client connected: {} {}",
                params.client_info.name, params.client_info.version
            );
            let result = crate::protocol::InitializeResult {
                protocol_version: "2024-11-05".to_string(),
                capabilities: crate::protocol::ServerCapabilities {
                    tools: crate::protocol::ToolsCapability {
                        list_changed: false,
                    },
                    prompts: None,
                    resources: None,
                },
                server_info: crate::protocol::ServerInfo {
                    name: "hoop-mcp".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                },
            };
            crate::protocol::JsonRpcResponse::result(serde_json::json!(null), serde_json::to_value(result).unwrap())
        }
        crate::protocol::Method::ToolsList(_) => {
            let tools = crate::tools::McpServerState::get_tools();
            let result = serde_json::json!({ "tools": tools });
            crate::protocol::JsonRpcResponse::result(serde_json::json!(null), result)
        }
        crate::protocol::Method::ToolsCall(params) => {
            match server_state.call_tool(&params.name, &params.arguments) {
                Ok(result) => {
                    let result_value = serde_json::to_value(result).unwrap_or_default();
                    crate::protocol::JsonRpcResponse::result(serde_json::json!(null), result_value)
                }
                Err(e) => {
                    warn!("Tool call error: {}", e);
                    crate::protocol::JsonRpcResponse::error(
                        serde_json::json!(null),
                        -32603,
                        e,
                    )
                }
            }
        }
        crate::protocol::Method::PromptsList(_) => {
            let result = serde_json::json!({ "prompts": [] });
            crate::protocol::JsonRpcResponse::result(serde_json::json!(null), result)
        }
        crate::protocol::Method::ResourcesList(_) => {
            let result = serde_json::json!({ "resources": [] });
            crate::protocol::JsonRpcResponse::result(serde_json::json!(null), result)
        }
        crate::protocol::Method::Shutdown(_) => {
            info!("Shutdown requested");
            crate::protocol::JsonRpcResponse::result(serde_json::json!(null), serde_json::json!({}))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_config_default() {
        let config = SocketConfig::default();
        assert!(config.socket_path.ends_with("mcp.sock"));
        assert_eq!(config.actor, "mcp-client");
    }
}
