//! `hoop config` subcommands — config inspection and diff
//!
//! Plan reference: §17.4

use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use std::path::PathBuf;
use std::fs;

/// Configuration response from /api/config
#[derive(Debug, Deserialize)]
struct ConfigResponse {
    schema_version: String,
    config: RunningConfig,
}

/// Running config from the daemon
#[derive(Debug, Deserialize)]
struct RunningConfig {
    server_bind_addr: String,
    agent_adapter: String,
    agent_model: String,
    ui_theme: String,
    metrics_enabled: bool,
    metrics_port: u16,
    voice_hotkey: String,
    audit_retention_days: u32,
    audit_hash_chain: bool,
    reflection_enabled: bool,
    reflection_detection_threshold: f64,
    reflection_auto_archive_after_days: u32,
}

/// Display the diff between running config and config.yml, highlighting restart-required changes
pub async fn run_diff() -> Result<()> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        println!("No config.yml found at {}", config_path.display());
        println!("Create one to see configuration diff.");
        println!("\nRunning config (daemon defaults):");
        print_running_config_only().await?;
        return Ok(());
    }

    // Fetch running config from daemon
    let running = fetch_running_config().await?;

    // Read and parse config.yml
    let raw = fs::read_to_string(&config_path)
        .context("Failed to read config.yml")?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&raw)
        .context("Failed to parse config.yml")?;

    println!("Configuration diff: running vs config.yml");
    println!("===========================================\n");

    // Compare each key and show differences
    let mut has_changes = false;
    let mut has_restart_required = false;

    // server.bind_addr (restart-required)
    if let Some(yaml_val) = get_nested_yaml_value(&yaml, &["server", "bind_addr"]) {
        if yaml_val != running.config.server_bind_addr {
            has_changes = true;
            has_restart_required = true;
            println!("  [RESTART REQUIRED] server.bind_addr: {} → {}", running.config.server_bind_addr, yaml_val);
        }
    }

    // agent.adapter
    if let Some(yaml_val) = get_nested_yaml_value(&yaml, &["agent", "adapter"]) {
        if yaml_val != running.config.agent_adapter {
            has_changes = true;
            println!("  [HOT-RELOADABLE] agent.adapter: {} → {}", running.config.agent_adapter, yaml_val);
        }
    }

    // agent.model
    if let Some(yaml_val) = get_nested_yaml_value(&yaml, &["agent", "model"]) {
        if yaml_val != running.config.agent_model {
            has_changes = true;
            println!("  [HOT-RELOADABLE] agent.model: {} → {}", running.config.agent_model, yaml_val);
        }
    }

    // ui.theme
    if let Some(yaml_val) = get_nested_yaml_value(&yaml, &["ui", "theme"]) {
        if yaml_val != running.config.ui_theme {
            has_changes = true;
            println!("  [HOT-RELOADABLE] ui.theme: {} → {}", running.config.ui_theme, yaml_val);
        }
    }

    // metrics.enabled
    if let Some(yaml_val) = get_nested_yaml_bool(&yaml, &["metrics", "enabled"]) {
        if yaml_val != running.config.metrics_enabled {
            has_changes = true;
            println!("  [HOT-RELOADABLE] metrics.enabled: {} → {}", running.config.metrics_enabled, yaml_val);
        }
    }

    // metrics.port (restart-required)
    if let Some(yaml_val) = get_nested_yaml_value(&yaml, &["metrics", "port"]) {
        let yaml_port = yaml_val.parse::<u16>().unwrap_or(9091);
        if yaml_port != running.config.metrics_port {
            has_changes = true;
            has_restart_required = true;
            println!("  [RESTART REQUIRED] metrics.port: {} → {}", running.config.metrics_port, yaml_port);
        }
    }

    // voice.hotkey
    if let Some(yaml_val) = get_nested_yaml_value(&yaml, &["voice", "hotkey"]) {
        if yaml_val != running.config.voice_hotkey {
            has_changes = true;
            println!("  [HOT-RELOADABLE] voice.hotkey: {} → {}", running.config.voice_hotkey, yaml_val);
        }
    }

    // audit.retention_days
    if let Some(yaml_val) = get_nested_yaml_value(&yaml, &["audit", "retention_days"]) {
        let yaml_days = yaml_val.parse::<u32>().unwrap_or(90);
        if yaml_days != running.config.audit_retention_days {
            has_changes = true;
            println!("  [HOT-RELOADABLE] audit.retention_days: {} → {}", running.config.audit_retention_days, yaml_days);
        }
    }

    // audit.hash_chain
    if let Some(yaml_val) = get_nested_yaml_bool(&yaml, &["audit", "hash_chain"]) {
        if yaml_val != running.config.audit_hash_chain {
            has_changes = true;
            println!("  [HOT-RELOADABLE] audit.hash_chain: {} → {}", running.config.audit_hash_chain, yaml_val);
        }
    }

    // reflection.enabled
    if let Some(yaml_val) = get_nested_yaml_bool(&yaml, &["reflection", "enabled"]) {
        if yaml_val != running.config.reflection_enabled {
            has_changes = true;
            println!("  [HOT-RELOADABLE] reflection.enabled: {} → {}", running.config.reflection_enabled, yaml_val);
        }
    }

    // reflection.detection_threshold
    if let Some(yaml_val) = get_nested_yaml_value(&yaml, &["reflection", "detection_threshold"]) {
        let yaml_val_f = yaml_val.parse::<f64>().unwrap_or(0.8);
        if (yaml_val_f - running.config.reflection_detection_threshold).abs() > 0.001 {
            has_changes = true;
            println!("  [HOT-RELOADABLE] reflection.detection_threshold: {} → {}", running.config.reflection_detection_threshold, yaml_val_f);
        }
    }

    // reflection.auto_archive_after_days
    if let Some(yaml_val) = get_nested_yaml_value(&yaml, &["reflection", "auto_archive_after_days"]) {
        let yaml_days = yaml_val.parse::<u32>().unwrap_or(30);
        if yaml_days != running.config.reflection_auto_archive_after_days {
            has_changes = true;
            println!("  [HOT-RELOADABLE] reflection.auto_archive_after_days: {} → {}", running.config.reflection_auto_archive_after_days, yaml_days);
        }
    }

    if !has_changes {
        println!("✓ No differences detected — config.yml matches running config");
    } else if has_restart_required {
        println!();
        println!("⚠️  Changes marked [RESTART REQUIRED] need daemon restart to take effect.");
        println!("   Run: systemctl --user restart hoop");
        println!();
        println!("   All other changes are hot-reloadable and will apply on next config.yml save.");
    } else {
        println!();
        println!("✓ All changes are hot-reloadable — they will apply on next config.yml save.");
    }

    Ok(())
}

/// Fetch the running config from the daemon
async fn fetch_running_config() -> Result<ConfigResponse> {
    let client = Client::new();
    let resp = client
        .get("http://127.0.0.1:3000/api/config")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            r.json::<ConfigResponse>()
                .await
                .context("Failed to parse config response")
        }
        Ok(r) => {
            Err(anyhow::anyhow!("Daemon returned status {}: {}", r.status(), r.text().await.unwrap_or_default()))
        }
        Err(e) => {
            Err(anyhow::anyhow!("Failed to connect to daemon at http://127.0.0.1:3000 — is it running?\n\nError: {}", e))
        }
    }
}

/// Print running config when config.yml doesn't exist
async fn print_running_config_only() -> Result<()> {
    let running = fetch_running_config().await?;
    println!("  server.bind_addr: {}", running.config.server_bind_addr);
    println!("  agent.adapter: {}", running.config.agent_adapter);
    println!("  agent.model: {}", running.config.agent_model);
    println!("  ui.theme: {}", running.config.ui_theme);
    println!("  metrics.enabled: {}", running.config.metrics_enabled);
    println!("  metrics.port: {}", running.config.metrics_port);
    println!("  voice.hotkey: {}", running.config.voice_hotkey);
    println!("  audit.retention_days: {}", running.config.audit_retention_days);
    println!("  audit.hash_chain: {}", running.config.audit_hash_chain);
    println!("  reflection.enabled: {}", running.config.reflection_enabled);
    println!("  reflection.detection_threshold: {}", running.config.reflection_detection_threshold);
    println!("  reflection.auto_archive_after_days: {}", running.config.reflection_auto_archive_after_days);
    Ok(())
}

fn get_config_path() -> Result<PathBuf> {
    let mut home = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."));
    home.push(".hoop");
    home.push("config.yml");
    Ok(home)
}

fn get_nested_yaml_value(yaml: &serde_yaml::Value, parts: &[&str]) -> Option<String> {
    let mut current = yaml;

    for (i, &part) in parts.iter().enumerate() {
        match current.get(part) {
            Some(v) => {
                if i == parts.len() - 1 {
                    // Last part - return the value as string
                    return Some(v.to_string());
                }
                current = v;
            }
            None => return None,
        }
    }

    None
}

fn get_nested_yaml_bool(yaml: &serde_yaml::Value, parts: &[&str]) -> Option<bool> {
    let mut current = yaml;

    for (i, &part) in parts.iter().enumerate() {
        match current.get(part) {
            Some(v) => {
                if i == parts.len() - 1 {
                    // Last part - return the value as bool
                    return v.as_bool();
                }
                current = v;
            }
            None => return None,
        }
    }

    None
}
