//! `hoop config` subcommands — config inspection and diff
//!
//! Plan reference: §17.4

use anyhow::{Context, Result};
use clap::Subcommand;
use reqwest::Client;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

/// Config subcommands
#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Show configuration diff (running vs config.yml)
    Diff,
    /// Validate config.yml syntax and schema
    Validate,
}

/// Handle the `hoop config` subcommands
pub async fn handle_config(cmd: ConfigCommands) -> Result<()> {
    match cmd {
        ConfigCommands::Diff => run_diff().await?,
        ConfigCommands::Validate => run_validate()?,
    }
    Ok(())
}

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
    let raw = fs::read_to_string(&config_path).context("Failed to read config.yml")?;
    let yaml: serde_yaml::Value =
        serde_yaml::from_str(&raw).context("Failed to parse config.yml")?;

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
            println!(
                "  [RESTART REQUIRED] server.bind_addr: {} → {}",
                running.config.server_bind_addr, yaml_val
            );
        }
    }

    // agent.adapter
    if let Some(yaml_val) = get_nested_yaml_value(&yaml, &["agent", "adapter"]) {
        if yaml_val != running.config.agent_adapter {
            has_changes = true;
            println!(
                "  [HOT-RELOADABLE] agent.adapter: {} → {}",
                running.config.agent_adapter, yaml_val
            );
        }
    }

    // agent.model
    if let Some(yaml_val) = get_nested_yaml_value(&yaml, &["agent", "model"]) {
        if yaml_val != running.config.agent_model {
            has_changes = true;
            println!(
                "  [HOT-RELOADABLE] agent.model: {} → {}",
                running.config.agent_model, yaml_val
            );
        }
    }

    // ui.theme
    if let Some(yaml_val) = get_nested_yaml_value(&yaml, &["ui", "theme"]) {
        if yaml_val != running.config.ui_theme {
            has_changes = true;
            println!(
                "  [HOT-RELOADABLE] ui.theme: {} → {}",
                running.config.ui_theme, yaml_val
            );
        }
    }

    // metrics.enabled
    if let Some(yaml_val) = get_nested_yaml_bool(&yaml, &["metrics", "enabled"]) {
        if yaml_val != running.config.metrics_enabled {
            has_changes = true;
            println!(
                "  [HOT-RELOADABLE] metrics.enabled: {} → {}",
                running.config.metrics_enabled, yaml_val
            );
        }
    }

    // metrics.port (restart-required)
    if let Some(yaml_val) = get_nested_yaml_value(&yaml, &["metrics", "port"]) {
        let yaml_port = yaml_val.parse::<u16>().unwrap_or(9091);
        if yaml_port != running.config.metrics_port {
            has_changes = true;
            has_restart_required = true;
            println!(
                "  [RESTART REQUIRED] metrics.port: {} → {}",
                running.config.metrics_port, yaml_port
            );
        }
    }

    // voice.hotkey
    if let Some(yaml_val) = get_nested_yaml_value(&yaml, &["voice", "hotkey"]) {
        if yaml_val != running.config.voice_hotkey {
            has_changes = true;
            println!(
                "  [HOT-RELOADABLE] voice.hotkey: {} → {}",
                running.config.voice_hotkey, yaml_val
            );
        }
    }

    // audit.retention_days
    if let Some(yaml_val) = get_nested_yaml_value(&yaml, &["audit", "retention_days"]) {
        let yaml_days = yaml_val.parse::<u32>().unwrap_or(90);
        if yaml_days != running.config.audit_retention_days {
            has_changes = true;
            println!(
                "  [HOT-RELOADABLE] audit.retention_days: {} → {}",
                running.config.audit_retention_days, yaml_days
            );
        }
    }

    // audit.hash_chain
    if let Some(yaml_val) = get_nested_yaml_bool(&yaml, &["audit", "hash_chain"]) {
        if yaml_val != running.config.audit_hash_chain {
            has_changes = true;
            println!(
                "  [HOT-RELOADABLE] audit.hash_chain: {} → {}",
                running.config.audit_hash_chain, yaml_val
            );
        }
    }

    // reflection.enabled
    if let Some(yaml_val) = get_nested_yaml_bool(&yaml, &["reflection", "enabled"]) {
        if yaml_val != running.config.reflection_enabled {
            has_changes = true;
            println!(
                "  [HOT-RELOADABLE] reflection.enabled: {} → {}",
                running.config.reflection_enabled, yaml_val
            );
        }
    }

    // reflection.detection_threshold
    if let Some(yaml_val) = get_nested_yaml_value(&yaml, &["reflection", "detection_threshold"]) {
        let yaml_val_f = yaml_val.parse::<f64>().unwrap_or(0.8);
        if (yaml_val_f - running.config.reflection_detection_threshold).abs() > 0.001 {
            has_changes = true;
            println!(
                "  [HOT-RELOADABLE] reflection.detection_threshold: {} → {}",
                running.config.reflection_detection_threshold, yaml_val_f
            );
        }
    }

    // reflection.auto_archive_after_days
    if let Some(yaml_val) = get_nested_yaml_value(&yaml, &["reflection", "auto_archive_after_days"])
    {
        let yaml_days = yaml_val.parse::<u32>().unwrap_or(30);
        if yaml_days != running.config.reflection_auto_archive_after_days {
            has_changes = true;
            println!(
                "  [HOT-RELOADABLE] reflection.auto_archive_after_days: {} → {}",
                running.config.reflection_auto_archive_after_days, yaml_days
            );
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
        Ok(r) if r.status().is_success() => r
            .json::<ConfigResponse>()
            .await
            .context("Failed to parse config response"),
        Ok(r) => Err(anyhow::anyhow!(
            "Daemon returned status {}: {}",
            r.status(),
            r.text().await.unwrap_or_default()
        )),
        Err(e) => Err(anyhow::anyhow!(
            "Failed to connect to daemon at http://127.0.0.1:3000 — is it running?\n\nError: {}",
            e
        )),
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
    println!(
        "  audit.retention_days: {}",
        running.config.audit_retention_days
    );
    println!("  audit.hash_chain: {}", running.config.audit_hash_chain);
    println!(
        "  reflection.enabled: {}",
        running.config.reflection_enabled
    );
    println!(
        "  reflection.detection_threshold: {}",
        running.config.reflection_detection_threshold
    );
    println!(
        "  reflection.auto_archive_after_days: {}",
        running.config.reflection_auto_archive_after_days
    );
    Ok(())
}

fn get_config_path() -> Result<PathBuf> {
    let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
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
                    return match v {
                        serde_yaml::Value::String(s) => Some(s.clone()),
                        serde_yaml::Value::Number(n) => Some(n.to_string()),
                        serde_yaml::Value::Bool(b) => Some(b.to_string()),
                        serde_yaml::Value::Null => Some("null".to_string()),
                        _ => Some(
                            serde_yaml::to_string(v)
                                .unwrap_or_else(|_| "[complex value]".to_string()),
                        ),
                    };
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

/// Validate config.yml syntax and basic schema
///
/// Checks:
/// - File exists and is readable
/// - Valid YAML syntax
/// - Required top-level keys present
/// - Basic type checking for known fields
///
/// Note: This is a basic validation. Full semantic validation happens when the daemon loads the config.
pub fn run_validate() -> Result<()> {
    let config_path = get_config_path()?;

    // Check if file exists
    if !config_path.exists() {
        println!("No config.yml found at {}", config_path.display());
        println!("\nTo create a config file:");
        println!("  1. Copy the example: cp hoop-daemon/config.yml.example ~/.hoop/config.yml");
        println!("  2. Edit it to your needs");
        println!("\nUsing daemon defaults (no config file is valid).");
        return Ok(());
    }

    println!("Validating config.yml at {}", config_path.display());
    println!("{}", "=".repeat(60));

    // Read file
    let raw = fs::read_to_string(&config_path).context("Failed to read config.yml")?;

    // Parse YAML
    let yaml: serde_yaml::Value =
        serde_yaml::from_str(&raw).with_context(|| "Invalid YAML syntax".to_string())?;

    let mut has_errors = false;
    let mut has_warnings = false;

    // Check required top-level keys
    if let Some(mapping) = yaml.as_mapping() {
        // Check schema_version
        match mapping.get(serde_yaml::Value::String("schema_version".to_string())) {
            None => {
                println!("❌ Error: Missing required key 'schema_version'");
                has_errors = true;
            }
            Some(serde_yaml::Value::String(v)) if !v.is_empty() => {
                println!("✓ schema_version: {}", v);
            }
            Some(_) => {
                println!("⚠️  Warning: schema_version should be a string (e.g., \"1.0.0\")");
                has_warnings = true;
            }
        }
    }

    // Check known sections (warning if missing, not error)
    let sections = [
        "server",
        "agent",
        "projects_file",
        "ui",
        "metrics",
        "voice",
        "agent_extensions",
        "audit",
        "reflection",
        "backup",
        "pricing_file",
        "secrets_patterns",
        "stuck_detector",
        "roles",
    ];

    if let Some(mapping) = yaml.as_mapping() {
        for section in &sections {
            if mapping.contains_key(serde_yaml::Value::String(section.to_string())) {
                println!("✓ Section '{}' present", section);
            } else {
                println!("  Section '{}' not present (will use defaults)", section);
            }
        }

        // Check for unknown top-level keys
        for key in mapping.keys() {
            if let Some(key_str) = key.as_str() {
                if !sections.contains(&key_str) && key_str != "schema_version" {
                    println!("⚠️  Warning: Unknown top-level key '{}'", key_str);
                    has_warnings = true;
                }
            }
        }
    }

    // Check agent.adapter value if present
    if let Some(adapter) = get_nested_yaml_value(&yaml, &["agent", "adapter"]) {
        let valid_adapters = ["claude", "codex", "opencode", "gemini", "aider"];
        if !valid_adapters.contains(&adapter.as_str()) {
            println!(
                "⚠️  Warning: agent.adapter '{}' is not a known adapter",
                adapter
            );
            println!("   Valid options: {}", valid_adapters.join(", "));
            has_warnings = true;
        } else {
            println!("✓ agent.adapter: {} (valid)", adapter);
        }
    }

    // Check ui.theme value if present
    if let Some(theme) = get_nested_yaml_value(&yaml, &["ui", "theme"]) {
        let valid_themes = ["auto", "light", "dark", "solarized-light", "solarized-dark"];
        if !valid_themes.contains(&theme.as_str()) {
            println!("⚠️  Warning: ui.theme '{}' is not a known theme", theme);
            println!("   Valid options: {}", valid_themes.join(", "));
            has_warnings = true;
        } else {
            println!("✓ ui.theme: {} (valid)", theme);
        }
    }

    // Check metrics.port if metrics enabled
    if let Some(enabled) = get_nested_yaml_bool(&yaml, &["metrics", "enabled"]) {
        if enabled {
            if let Some(port) = get_nested_yaml_value(&yaml, &["metrics", "port"]) {
                match port.parse::<u16>() {
                    Ok(p) if p > 0 => {
                        println!("✓ metrics.port: {} (valid, [RESTART REQUIRED])", p);
                    }
                    _ => {
                        println!(
                            "❌ Error: metrics.port '{}' is not a valid port (1-65535)",
                            port
                        );
                        has_errors = true;
                    }
                }
            }
        }
    }

    println!("{}", "=".repeat(60));

    if has_errors {
        println!("\n❌ Validation FAILED: Errors found that must be fixed.");
        println!("\nNext steps:");
        println!("  1. Fix the errors listed above");
        println!("  2. Run `hoop config validate` again");
        return Err(anyhow::anyhow!("Config validation failed"));
    } else if has_warnings {
        println!("\n⚠️  Validation PASSED with warnings.");
        println!("\nThe config file is syntactically valid but has warnings.");
        println!("The daemon will load this config, but review warnings above.");
    } else {
        println!("\n✓ Validation PASSED: config.yml is valid!");
        println!("\nThe daemon will load this config successfully.");
    }

    println!("\nNote: Full semantic validation happens when the daemon loads the config.");
    println!("      Run the daemon to check for any additional errors.");

    Ok(())
}
