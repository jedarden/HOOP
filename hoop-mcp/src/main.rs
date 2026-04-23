//! HOOP MCP Server
//!
//! Model Context Protocol server exposing HOOP's read APIs + create_stitch tool.
//! Runs over Unix domain socket with same user:group permissions.
//!
//! Per §6 Phase 5 canonical tool belt and §9.3 open question resolution.

mod audit;
mod br_verbs;
mod protocol;
mod socket;
mod tools;

use anyhow::Result;
use clap::Parser;
use socket::SocketConfig;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[derive(Parser, Debug)]
#[command(name = "hoop-mcp")]
#[command(about = "HOOP MCP Server - Exposes HOOP tools via Model Context Protocol", long_about = None)]
struct Cli {
    /// Path to the Unix domain socket (default: ~/.hoop/mcp.sock)
    #[arg(short, long)]
    socket: Option<String>,

    /// Actor name for audit logging (default: mcp-client)
    #[arg(short, long)]
    actor: Option<String>,

    /// Run in stdio mode instead of socket mode (for testing)
    #[arg(long)]
    stdio: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    // Validate write invariant at startup
    br_verbs::validate_write_invariant();

    // Print tool list for documentation
    println!("# HOOP MCP Server Tools");
    println!("");
    println!("Read tools:");
    println!("  - find_stitches: List stitches with filtering");
    println!("  - read_stitch: Get detailed stitch information");
    println!("  - find_beads: List beads with filtering");
    println!("  - read_bead: Get detailed bead information");
    println!("  - read_file: Read a file from a project");
    println!("  - grep: Search for pattern in project files");
    println!("  - search_conversations: Search conversation transcripts");
    println!("  - summarize_project: Get project summary");
    println!("  - summarize_day: Get daily summary across all projects");
    println!("");
    println!("Write tools:");
    println!("  - create_stitch: Create a new stitch (ONE write operation)");
    println!("");
    println!("Utility tools:");
    println!("  - escalate_to_operator: Send message to operator (UI banner)");
    println!("");
    println!("Security:");
    println!("  - Unix socket with 0600 permissions (same user only)");
    println!("  - Audit log records every tool call with args hash");
    println!("  - No forbidden verbs (close, update, release, claim, depend)");
    println!("");

    if cli.stdio {
        // Run in stdio mode (useful for testing with mcp-client)
        run_stdio_mode(cli.actor).await?;
    } else {
        // Run in socket mode (production)
        let mut config = SocketConfig::default();
        if let Some(socket_path) = cli.socket {
            config.socket_path = socket_path.into();
        }
        if let Some(actor) = cli.actor {
            config.actor = actor;
        }

        socket::run_socket_server(config).await?;
    }

    Ok(())
}

/// Run the MCP server in stdio mode (for testing)
async fn run_stdio_mode(actor_override: Option<String>) -> Result<()> {
    use protocol::Method;
    use std::io::{self, BufRead, BufReader, Write};

    let actor = actor_override.unwrap_or_else(|| "mcp-client-stdio".to_string());
    let server_state = tools::McpServerState::new(actor)?;

    let stdin = io::stdin();
    let stdout = io::stdout();
    let reader = BufReader::new(stdin.lock());
    let mut writer = stdout.lock();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let request: protocol::JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                let error_response = protocol::JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(protocol::JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                };
                writeln!(writer, "{}", serde_json::to_string(&error_response)?)?;
                writer.flush()?;
                continue;
            }
        };

        // Check if shutdown is requested
        let is_shutdown = matches!(request.method, Method::Shutdown(_));

        let response = match request.method {
            Method::Initialize(ref params) => {
                eprintln!("Client connected: {} {}", params.client_info.name, params.client_info.version);
                let result = protocol::InitializeResult {
                    protocol_version: "2024-11-05".to_string(),
                    capabilities: protocol::ServerCapabilities {
                        tools: protocol::ToolsCapability {
                            list_changed: false,
                        },
                        prompts: None,
                        resources: None,
                    },
                    server_info: protocol::ServerInfo {
                        name: "hoop-mcp".to_string(),
                        version: env!("CARGO_PKG_VERSION").to_string(),
                    },
                };
                protocol::JsonRpcResponse::result(serde_json::json!(null), serde_json::to_value(result)?)
            }
            Method::ToolsList(_) => {
                let tools = tools::McpServerState::get_tools();
                let result = serde_json::json!({ "tools": tools });
                protocol::JsonRpcResponse::result(serde_json::json!(null), result)
            }
            Method::ToolsCall(ref params) => {
                match server_state.call_tool(&params.name, &params.arguments) {
                    Ok(result) => {
                        let result_value = serde_json::to_value(result)?;
                        protocol::JsonRpcResponse::result(request.id.clone(), result_value)
                    }
                    Err(e) => {
                        protocol::JsonRpcResponse::error(request.id.clone(), -32603, e)
                    }
                }
            }
            Method::PromptsList(_) => {
                let result = serde_json::json!({ "prompts": [] });
                protocol::JsonRpcResponse::result(request.id.clone(), result)
            }
            Method::ResourcesList(_) => {
                let result = serde_json::json!({ "resources": [] });
                protocol::JsonRpcResponse::result(request.id.clone(), result)
            }
            Method::Shutdown(_) => {
                eprintln!("Shutdown requested");
                protocol::JsonRpcResponse::result(request.id, serde_json::json!({}))
            }
        };

        writeln!(writer, "{}", serde_json::to_string(&response)?)?;
        writer.flush()?;

        // Exit on shutdown
        if is_shutdown {
            break;
        }
    }

    Ok(())
}
