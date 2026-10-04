//! HOOP agent command - attach to or start the human-interface agent conversation

use anyhow::Result;
use hoop::daemon::resolve_daemon_url;
use reqwest::Client;
use serde::Deserialize;

/// Agent session status from daemon
#[derive(Debug, Deserialize)]
struct AgentStatus {
    active: bool,
    enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    adapter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stitch_id: Option<String>,
    cost_usd: f64,
    input_tokens: i64,
    output_tokens: i64,
    turn_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_activity_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    age_secs: Option<i64>,
}

/// Spawn response from daemon
#[derive(Debug, Deserialize)]
struct SpawnResponse {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_db_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

/// Run the agent command
pub async fn run() -> Result<()> {
    let client = Client::new();
    let daemon_url = resolve_daemon_url(None);

    // First, check if there's an active session
    let status_url = format!("{}/api/agent/status", daemon_url);
    let status_response = match client.get(&status_url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            anyhow::bail!(
                "Failed to connect to daemon at {} — is it running?\n\nError: {}",
                daemon_url,
                e
            )
        }
    };

    if !status_response.status().is_success() {
        anyhow::bail!("Daemon returned error status: {}", status_response.status());
    }

    let status: AgentStatus = status_response.json().await?;

    if status.active {
        // Session is already active
        println!("Agent session is already active");
        println!();
        println!(
            "Session ID: {}",
            status.session_id.as_ref().unwrap_or(&"unknown".to_string())
        );
        println!(
            "Adapter: {}",
            status.adapter.as_ref().unwrap_or(&"unknown".to_string())
        );
        println!(
            "Model: {}",
            status.model.as_ref().unwrap_or(&"unknown".to_string())
        );

        if let Some(stitch_id) = &status.stitch_id {
            println!("Stitch ID: {}", stitch_id);
        }

        println!();
        println!("Session stats:");
        println!("  Turns: {}", status.turn_count);
        println!("  Input tokens: {}", status.input_tokens);
        println!("  Output tokens: {}", status.output_tokens);
        println!("  Cost: ${:.6}", status.cost_usd);

        if let Some(created_at) = &status.created_at {
            println!("  Created at: {}", created_at);
        }

        if let Some(last_activity) = &status.last_activity_at {
            println!("  Last activity: {}", last_activity);
        }

        if let Some(age_secs) = status.age_secs {
            println!("  Age: {} seconds", age_secs);
        }

        println!();
        println!(
            "To interact with this session, use the HOOP web UI at {}",
            daemon_url
        );

        return Ok(());
    }

    // No active session, spawn one
    println!("No active agent session found. Starting a new session...");

    let spawn_url = format!("{}/api/agent/spawn", daemon_url);
    let spawn_response = match client.post(&spawn_url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            anyhow::bail!("Failed to spawn agent session: {}", e);
        }
    };

    if !spawn_response.status().is_success() {
        anyhow::bail!(
            "Failed to spawn agent session. Daemon returned status: {}",
            spawn_response.status()
        );
    }

    let spawn_result: SpawnResponse = spawn_response.json().await?;

    if spawn_result.status == "ok" {
        println!("✓ Agent session started successfully");

        if let Some(session_id) = spawn_result.session_db_id {
            println!("Session DB ID: {}", session_id);
        }

        println!();
        println!(
            "To interact with this session, use the HOOP web UI at {}",
            daemon_url
        );
    } else {
        anyhow::bail!(
            "Failed to start agent session: {}",
            spawn_result
                .message
                .unwrap_or_else(|| "unknown error".to_string())
        );
    }

    Ok(())
}
