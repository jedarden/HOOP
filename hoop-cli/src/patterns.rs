//! Pattern management CLI commands
//!
//! `hoop pattern new <title>` — create a new pattern
//! `hoop pattern list` — list all patterns
//! `hoop pattern show <id>` — show pattern details
//! `hoop pattern update <id>` — update a pattern
//! `hoop pattern close <id>` — close a pattern (status → done)
//! `hoop pattern delete <id>` — delete a pattern
//! `hoop pattern add-member <id> <stitch_id>` — add a stitch to a pattern
//! `hoop pattern remove-member <id> <stitch_id>` — remove a stitch from a pattern
//! `hoop pattern add-query <id> <query>` — add a saved query to a pattern
//! `hoop pattern remove-query <id> <query>` — remove a saved query from a pattern

use anyhow::Result;
use clap::Subcommand;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const DEFAULT_DAEMON_ADDR: &str = "http://127.0.0.1:3000";

#[derive(Subcommand, Debug)]
pub enum PatternCommands {
    /// Create a new pattern
    New {
        /// Pattern title
        title: String,
        /// Pattern description
        #[arg(long)]
        description: Option<String>,
        /// Initial status (default: planned)
        #[arg(long)]
        status: Option<String>,
        /// Pattern owner
        #[arg(long)]
        owner: Option<String>,
        /// Deadline (ISO 8601 datetime)
        #[arg(long)]
        deadline: Option<String>,
        /// Parent pattern ID
        #[arg(long)]
        parent: Option<String>,
        /// Daemon address (default: http://127.0.0.1:3000)
        #[arg(long)]
        addr: Option<String>,
    },
    /// List all patterns
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Daemon address (default: http://127.0.0.1:3000)
        #[arg(long)]
        addr: Option<String>,
    },
    /// Show pattern details
    Show {
        /// Pattern ID
        id: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Daemon address (default: http://127.0.0.1:3000)
        #[arg(long)]
        addr: Option<String>,
    },
    /// Update a pattern
    Update {
        /// Pattern ID
        id: String,
        /// New title
        #[arg(long)]
        title: Option<String>,
        /// New description
        #[arg(long)]
        description: Option<String>,
        /// New status
        #[arg(long)]
        status: Option<String>,
        /// New owner
        #[arg(long)]
        owner: Option<String>,
        /// New deadline (ISO 8601 datetime)
        #[arg(long)]
        deadline: Option<String>,
        /// New parent pattern ID
        #[arg(long)]
        parent: Option<String>,
        /// Daemon address (default: http://127.0.0.1:3000)
        #[arg(long)]
        addr: Option<String>,
    },
    /// Close a pattern (set status to done)
    Close {
        /// Pattern ID
        id: String,
        /// Daemon address (default: http://127.0.0.1:3000)
        #[arg(long)]
        addr: Option<String>,
    },
    /// Delete a pattern
    Delete {
        /// Pattern ID
        id: String,
        /// Confirm deletion without prompt
        #[arg(long)]
        confirm: bool,
        /// Daemon address (default: http://127.0.0.1:3000)
        #[arg(long)]
        addr: Option<String>,
    },
    /// Add a stitch to a pattern
    AddMember {
        /// Pattern ID
        id: String,
        /// Stitch ID
        stitch_id: String,
        /// Daemon address (default: http://127.0.0.1:3000)
        #[arg(long)]
        addr: Option<String>,
    },
    /// Remove a stitch from a pattern
    RemoveMember {
        /// Pattern ID
        id: String,
        /// Stitch ID
        stitch_id: String,
        /// Daemon address (default: http://127.0.0.1:3000)
        #[arg(long)]
        addr: Option<String>,
    },
    /// Add a saved query to a pattern
    AddQuery {
        /// Pattern ID
        id: String,
        /// Query string (use quotes if contains spaces)
        query: String,
        /// Daemon address (default: http://127.0.0.1:3000)
        #[arg(long)]
        addr: Option<String>,
    },
    /// Remove a saved query from a pattern
    RemoveQuery {
        /// Pattern ID
        id: String,
        /// Query string (must match exactly)
        query: String,
        /// Daemon address (default: http://127.0.0.1:3000)
        #[arg(long)]
        addr: Option<String>,
    },
}

pub async fn handle_patterns(cmd: PatternCommands) -> Result<()> {
    match cmd {
        PatternCommands::New {
            title,
            description,
            status,
            owner,
            deadline,
            parent,
            addr,
        } => {
            let addr = addr.unwrap_or_else(|| DEFAULT_DAEMON_ADDR.to_string());
            let client = Client::new();

            let mut payload = serde_json::json!({ "title": title });
            if let Some(d) = description {
                payload["description"] = serde_json::Value::String(d);
            }
            if let Some(s) = status {
                payload["status"] = serde_json::Value::String(s);
            }
            if let Some(o) = owner {
                payload["owner"] = serde_json::Value::String(o);
            }
            if let Some(d) = deadline {
                payload["deadline"] = serde_json::Value::String(d);
            }
            if let Some(p) = parent {
                payload["parent_pattern"] = serde_json::Value::String(p);
            }

            let resp = client
                .post(format!("{}/api/patterns", addr))
                .json(&payload)
                .send()
                .await?;

            if resp.status().is_success() {
                let data: serde_json::Value = resp.json().await?;
                let pattern = &data["pattern"];
                println!("Pattern created:");
                println!("  ID: {}", pattern["id"]);
                println!("  Title: {}", pattern["title"]);
                println!("  Status: {}", pattern["status"]);
            } else {
                let err: serde_json::Value = resp.json().await?;
                eprintln!("Failed to create pattern: {}", err);
                std::process::exit(1);
            }
        }
        PatternCommands::List { json, addr } => {
            let addr = addr.unwrap_or_else(|| DEFAULT_DAEMON_ADDR.to_string());
            let client = Client::new();

            let resp = client.get(format!("{}/api/patterns", addr)).send().await?;

            if resp.status().is_success() {
                let data: serde_json::Value = resp.json().await?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&data)?);
                } else {
                    let empty = vec![];
                    let patterns = data["patterns"].as_array().unwrap_or(&empty);
                    if patterns.is_empty() {
                        println!("No patterns found");
                    } else {
                        println!("Patterns:");
                        for p in patterns {
                            println!(
                                "  {} - {} ({}) - {} members",
                                p["id"], p["title"], p["status"], p["member_count"]
                            );
                        }
                    }
                }
            } else {
                eprintln!("Failed to list patterns");
                std::process::exit(1);
            }
        }
        PatternCommands::Show { id, json, addr } => {
            let addr = addr.unwrap_or_else(|| DEFAULT_DAEMON_ADDR.to_string());
            let client = Client::new();

            let resp = client
                .get(format!("{}/api/patterns/{}", addr, id))
                .send()
                .await?;

            if resp.status().is_success() {
                let data: serde_json::Value = resp.json().await?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&data)?);
                } else {
                    let pattern = &data["pattern"];
                    println!("Pattern: {}", pattern["title"]);
                    println!("ID: {}", pattern["id"]);
                    println!("Status: {}", pattern["status"]);
                    if let Some(desc) = pattern["description"].as_str() {
                        println!("Description: {}", desc);
                    }
                    if let Some(owner) = pattern["owner"].as_str() {
                        println!("Owner: {}", owner);
                    }
                    if let Some(deadline) = pattern["deadline"].as_str() {
                        println!("Deadline: {}", deadline);
                    }
                    if let Some(parent) = pattern["parent_pattern"].as_str() {
                        println!("Parent: {}", parent);
                    }
                    println!("Created: {}", pattern["created_at"]);

                    let aggregate = &data["aggregate"];
                    println!("\nAggregate:");
                    println!("  Members: {}", aggregate["total_members"]);
                    println!("  Closed: {}", aggregate["closed_members"]);
                    println!("  Progress: {:.1}%", aggregate["progress_percent"]);

                    let empty = vec![];
                    let members = data["members"].as_array().unwrap_or(&empty);
                    if !members.is_empty() {
                        println!("\nMembers:");
                        for m in members {
                            println!(
                                "  {} - {} ({}) - {}",
                                m["stitch_id"], m["title"], m["project"], m["kind"]
                            );
                        }
                    }
                }
            } else {
                eprintln!("Failed to show pattern");
                std::process::exit(1);
            }
        }
        PatternCommands::Update {
            id,
            title,
            description,
            status,
            owner,
            deadline,
            parent,
            addr,
        } => {
            let addr = addr.unwrap_or_else(|| DEFAULT_DAEMON_ADDR.to_string());
            let client = Client::new();

            let mut payload = serde_json::json!({});
            if let Some(t) = title {
                payload["title"] = serde_json::Value::String(t);
            }
            if let Some(d) = description {
                payload["description"] = serde_json::Value::String(d);
            }
            if let Some(s) = status {
                payload["status"] = serde_json::Value::String(s);
            }
            if let Some(o) = owner {
                payload["owner"] = serde_json::Value::String(o);
            }
            if let Some(d) = deadline {
                payload["deadline"] = serde_json::Value::String(d);
            }
            if let Some(p) = parent {
                payload["parent_pattern"] = serde_json::Value::String(p);
            }

            let resp = client
                .put(format!("{}/api/patterns/{}", addr, id))
                .json(&payload)
                .send()
                .await?;

            if resp.status().is_success() {
                let data: serde_json::Value = resp.json().await?;
                let pattern = &data["pattern"];
                println!("Pattern updated:");
                println!("  ID: {}", pattern["id"]);
                println!("  Title: {}", pattern["title"]);
                println!("  Status: {}", pattern["status"]);
            } else {
                let err: serde_json::Value = resp.json().await?;
                eprintln!("Failed to update pattern: {}", err);
                std::process::exit(1);
            }
        }
        PatternCommands::Close { id, addr } => {
            let addr = addr.unwrap_or_else(|| DEFAULT_DAEMON_ADDR.to_string());
            let client = Client::new();

            let payload = serde_json::json!({ "status": "done" });

            let resp = client
                .put(format!("{}/api/patterns/{}", addr, id))
                .json(&payload)
                .send()
                .await?;

            if resp.status().is_success() {
                let data: serde_json::Value = resp.json().await?;
                let pattern = &data["pattern"];
                println!("Pattern closed:");
                println!("  ID: {}", pattern["id"]);
                println!("  Title: {}", pattern["title"]);
                println!("  Status: {}", pattern["status"]);
            } else {
                let err: serde_json::Value = resp.json().await?;
                eprintln!("Failed to close pattern: {}", err);
                std::process::exit(1);
            }
        }
        PatternCommands::Delete { id, confirm, addr } => {
            if !confirm {
                println!("Are you sure you want to delete pattern '{}'?", id);
                println!("This will cascade to all members and queries.");
                print!("Confirm (yes/no): ");
                use std::io::Write;
                std::io::stdout().flush()?;
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if input.trim() != "yes" {
                    println!("Deletion cancelled");
                    return Ok(());
                }
            }

            let addr = addr.unwrap_or_else(|| DEFAULT_DAEMON_ADDR.to_string());
            let client = Client::new();

            let resp = client
                .delete(format!("{}/api/patterns/{}", addr, id))
                .send()
                .await?;

            if resp.status().is_success() {
                let data: serde_json::Value = resp.json().await?;
                println!("{}", data["message"]);
            } else {
                eprintln!("Failed to delete pattern");
                std::process::exit(1);
            }
        }
        PatternCommands::AddMember { id, stitch_id, addr } => {
            let addr = addr.unwrap_or_else(|| DEFAULT_DAEMON_ADDR.to_string());
            let client = Client::new();

            let payload = serde_json::json!({ "stitch_id": stitch_id });

            let resp = client
                .post(format!("{}/api/patterns/{}/members", addr, id))
                .json(&payload)
                .send()
                .await?;

            if resp.status().is_success() {
                let data: serde_json::Value = resp.json().await?;
                println!("{}", data["message"]);
            } else {
                let err: serde_json::Value = resp.json().await?;
                eprintln!("Failed to add member: {}", err);
                std::process::exit(1);
            }
        }
        PatternCommands::RemoveMember { id, stitch_id, addr } => {
            let addr = addr.unwrap_or_else(|| DEFAULT_DAEMON_ADDR.to_string());
            let client = Client::new();

            let resp = client
                .delete(format!(
                    "{}/api/patterns/{}/members/{}",
                    addr, id, stitch_id
                ))
                .send()
                .await?;

            if resp.status().is_success() {
                let data: serde_json::Value = resp.json().await?;
                println!("{}", data["message"]);
            } else {
                eprintln!("Failed to remove member");
                std::process::exit(1);
            }
        }
        PatternCommands::AddQuery { id, query, addr } => {
            let addr = addr.unwrap_or_else(|| DEFAULT_DAEMON_ADDR.to_string());
            let client = Client::new();

            let payload = serde_json::json!({ "query": query });

            let resp = client
                .post(format!("{}/api/patterns/{}/queries", addr, id))
                .json(&payload)
                .send()
                .await?;

            if resp.status().is_success() {
                let data: serde_json::Value = resp.json().await?;
                println!("{}", data["message"]);
            } else {
                let err: serde_json::Value = resp.json().await?;
                eprintln!("Failed to add query: {}", err);
                std::process::exit(1);
            }
        }
        PatternCommands::RemoveQuery { id, query, addr } => {
            let addr = addr.unwrap_or_else(|| DEFAULT_DAEMON_ADDR.to_string());
            let client = Client::new();

            // URL encode the query
            let encoded_query = urlencoding::encode(&query);

            let resp = client
                .delete(format!(
                    "{}/api/patterns/{}/queries/{}",
                    addr, id, encoded_query
                ))
                .send()
                .await?;

            if resp.status().is_success() {
                let data: serde_json::Value = resp.json().await?;
                println!("{}", data["message"]);
            } else {
                eprintln!("Failed to remove query");
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
