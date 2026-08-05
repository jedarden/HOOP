//! HOOP stitch command - list open Stitches

use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

const DAEMON_ADDR: &str = "http://127.0.0.1:3000";

/// Conversations query response from daemon
#[derive(Debug, Deserialize)]
struct ConversationsResponse {
    conversations: Vec<ConversationSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_cursor: Option<String>,
    has_more: bool,
}

/// Summary of a conversation for the list view
#[derive(Debug, Clone, Deserialize)]
struct ConversationSummary {
    id: String,
    session_id: String,
    provider: String,
    kind: String,
    project: String,
    cwd: String,
    title: String,
    message_count: usize,
    total_tokens: i64,
    created_at: String,
    updated_at: String,
    complete: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    worker_metadata: Option<WorkerMetadata>,
}

/// Worker metadata for fleet sessions
#[derive(Debug, Clone, Deserialize)]
struct WorkerMetadata {
    worker: String,
    bead: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    strand: Option<String>,
}

/// Run the stitch list command
pub async fn run(project_filter: Option<String>) -> Result<()> {
    let client = Client::new();

    // Build query URL with optional project filter
    let mut query_url = format!("{}/api/conversations?limit=100", DAEMON_ADDR);

    if let Some(project) = &project_filter {
        query_url.push_str(&format!("&project={}", urlencode(project)));
    }

    let response = match client.get(&query_url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            anyhow::bail!(
                "Failed to connect to daemon at {} — is it running?\n\nError: {}",
                DAEMON_ADDR,
                e
            )
        }
    };

    if !response.status().is_success() {
        anyhow::bail!(
            "Daemon returned error status: {}",
            response.status()
        );
    }

    let result: ConversationsResponse = response.json().await?;

    if result.conversations.is_empty() {
        if let Some(project) = project_filter {
            println!("No open stitches found for project '{}'", project);
        } else {
            println!("No open stitches found");
        }
        println!();
        println!("Stitches are created when you interact with the agent via the web UI.");
        return Ok(());
    }

    // Group conversations by project if no project filter
    if project_filter.is_none() {
        print_by_project(&result.conversations);
    } else {
        print_single_project(&result.conversations, &project_filter.unwrap());
    }

    if result.has_more {
        println!("...");
        println!("(Only showing first 100 stitches. Use the web UI for full list.)");
    }

    Ok(())
}

/// Print conversations grouped by project
fn print_by_project(conversations: &[ConversationSummary]) {
    let mut by_project: std::collections::HashMap<String, Vec<&ConversationSummary>> =
        std::collections::HashMap::new();

    for conv in conversations {
        by_project
            .entry(conv.project.clone())
            .or_default()
            .push(conv);
    }

    println!("Open Stitches by Project:");
    println!();

    let mut sorted_projects: Vec<_> = by_project.iter().collect();
    sorted_projects.sort_by_key(|(k, _)| *k);

    for (project, convs) in sorted_projects {
        println!("{} ({} stitches)", project, convs.len());
        println!();

        for conv in convs {
            print_conversation(conv);
            println!();
        }
    }

    let total = conversations.len();
    println!("Total: {} open stitch{}", total, if total == 1 { "" } else { "es" });
}

/// Print conversations for a single project
fn print_single_project(conversations: &[ConversationSummary], project: &str) {
    println!("Open Stitches for project '{}':", project);
    println!();
    println!("Total: {} stitch{}", conversations.len(), if conversations.len() == 1 { "" } else { "es" });
    println!();

    for conv in conversations {
        print_conversation(conv);
        println!();
    }
}

/// Print a single conversation summary
fn print_conversation(conv: &ConversationSummary) {
    let status = if conv.complete { "✓" } else { "○" };

    println!("  {} {} - {}", status, conv.id, conv.title);
    println!("     Provider: {} | Kind: {}", conv.provider, conv.kind);

    if let Some(worker) = &conv.worker_metadata {
        println!("     Worker: {} | Bead: {}", worker.worker, worker.bead);
        if let Some(strand) = &worker.strand {
            println!("     Strand: {}", strand);
        }
    } else {
        println!("     Session: {}", conv.session_id);
    }

    println!("     Messages: {} | Tokens: {} | Complete: {}", conv.message_count, conv.total_tokens, if conv.complete { "yes" } else { "no" });
    println!("     Created: {} | Updated: {}", conv.created_at, conv.updated_at);
    println!("     CWD: {}", conv.cwd);
}

/// Simple URL encoding for query parameters
fn urlencode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect()
}
