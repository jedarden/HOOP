//! `hoop backup` — Manual backup trigger and status.
//!
//! Plan reference: §15

use anyhow::{bail, Context, Result};
use std::net::SocketAddr;
use std::time::Duration;

#[derive(clap::Subcommand, Debug)]
pub enum BackupCommands {
    /// Manually trigger a backup run
    Trigger {
        /// Daemon address (default: 127.0.0.1:3000)
        #[arg(short, long)]
        addr: Option<SocketAddr>,
    },
    /// Show backup configuration and last run status
    Status {
        /// Daemon address (default: 127.0.0.1:3000)
        #[arg(short, long)]
        addr: Option<SocketAddr>,
    },
}

/// Handle `hoop backup` subcommands.
pub async fn handle_backup(cmd: BackupCommands) -> Result<()> {
    match cmd {
        BackupCommands::Trigger { addr } => trigger_backup(addr).await,
        BackupCommands::Status { addr } => show_status(addr).await,
    }
}

/// Trigger a backup via the daemon's REST API.
async fn trigger_backup(addr: Option<SocketAddr>) -> Result<()> {
    let addr = addr.unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 3000)));

    println!("Triggering backup on {} ...", addr);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()?;

    let resp = client
        .post(format!("http://{}/api/backup/trigger", addr))
        .send()
        .await
        .context("Failed to connect to daemon — is it running?")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        bail!("Backup trigger failed: {} — {}", status, body.trim());
    }

    let json: serde_json::Value = resp.json().await?;
    if let Some(message) = json.get("message").and_then(|m| m.as_str()) {
        println!("{}", message);
    } else {
        println!("Backup triggered successfully");
    }

    Ok(())
}

/// Show backup configuration and status.
async fn show_status(addr: Option<SocketAddr>) -> Result<()> {
    let addr = addr.unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 3000)));

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;

    // Check daemon health first
    let health_resp = client
        .get(format!("http://{}/api/health", addr))
        .send()
        .await;

    let is_running = match health_resp {
        Ok(r) => r.status().is_success(),
        Err(_) => false,
    };

    if !is_running {
        println!("HOOP daemon status: \x1b[31mDOWN\x1b[0m");
        println!();
        println!("Backup requires the daemon to be running.");
        println!("Start it with: systemctl --user start hoop");
        return Ok(());
    }

    println!("HOOP daemon status: \x1b[32mUP\x1b[0m ({})", addr);
    println!();

    // Try to get metrics for backup status
    let metrics_resp = client.get(format!("http://{}/metrics", addr)).send().await;

    match metrics_resp {
        Ok(resp) if resp.status().is_success() => {
            let metrics = resp.text().await?;
            print_backup_metrics(&metrics);
        }
        _ => {
            println!("Backup status: \x1b[33mUnknown\x1b[0m (metrics endpoint unavailable)");
        }
    }

    // Show configuration from config.yml
    print_backup_config();

    Ok(())
}

/// Print backup-related metrics from Prometheus output.
fn print_backup_metrics(metrics: &str) {
    let mut last_success = None;
    let mut last_size = None;
    let mut failures = None;

    for line in metrics.lines() {
        if line.starts_with("hoop_backup_last_success_timestamp ") {
            let ts = line.split_whitespace().nth(1).unwrap_or("0");
            if let Ok(secs) = ts.parse::<i64>() {
                if secs > 0 {
                    if let Some(dt) = chrono::DateTime::from_timestamp(secs, 0) {
                        last_success = Some(dt.format("%Y-%m-%d %H:%M:%S UTC").to_string());
                    }
                }
            }
        }
        if line.starts_with("hoop_backup_last_size_bytes ") {
            let bytes = line.split_whitespace().nth(1).unwrap_or("0");
            if let Ok(b) = bytes.parse::<u64>() {
                last_size = Some(b);
            }
        }
        if line.starts_with("hoop_backup_failures_total ") {
            let count = line.split_whitespace().nth(1).unwrap_or("0");
            if let Ok(c) = count.parse::<u64>() {
                failures = Some(c);
            }
        }
    }

    println!("Backup status:");
    match &last_success {
        Some(ts) => println!("  Last success: \x1b[32m{}\x1b[0m", ts),
        None => println!("  Last success: \x1b[33mNever\x1b[0m"),
    }

    if let Some(size) = last_size {
        println!("  Last size: {} MB", size / 1_000_000);
    }

    if let Some(f) = failures {
        if f > 0 {
            println!("  Failures: \x1b[31m{}\x1b[0m", f);
        } else {
            println!("  Failures: {}", f);
        }
    }
}

/// Print backup configuration from config.yml.
fn print_backup_config() {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let config_path = home.join(".hoop").join("config.yml");

    let contents = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => {
            println!();
            println!("Configuration: \x1b[33mNot configured\x1b[0m (config.yml not found)");
            println!();
            println!("To enable backups, add a `backup:` section to ~/.hoop/config.yml:");
            println!("  backup:");
            println!("    endpoint: https://s3.us-west-000.backblazeb2.com");
            println!("    bucket: your-bucket");
            println!("    prefix: hoop/");
            println!("    schedule: \"0 4 * * *\"");
            println!("    retention_days: 30");
            return;
        }
    };

    let yaml: serde_yaml::Value = match serde_yaml::from_str(&contents) {
        Ok(y) => y,
        Err(_) => {
            println!();
            println!("Configuration: \x1b[31mInvalid YAML\x1b[0m");
            return;
        }
    };

    let backup_section = match yaml.get("backup") {
        Some(s) => s,
        None => {
            println!();
            println!("Configuration: \x1b[33mNot configured\x1b[0m (no `backup:` section)");
            return;
        }
    };

    println!();
    println!("Configuration:");
    if let Some(endpoint) = backup_section.get("endpoint").and_then(|e| e.as_str()) {
        println!("  Endpoint: {}", endpoint);
    }
    if let Some(bucket) = backup_section.get("bucket").and_then(|b| b.as_str()) {
        println!("  Bucket: {}", bucket);
    }
    if let Some(prefix) = backup_section.get("prefix").and_then(|p| p.as_str()) {
        println!("  Prefix: {}", prefix);
    }
    if let Some(schedule) = backup_section.get("schedule").and_then(|s| s.as_str()) {
        println!("  Schedule: {}", schedule);
    }
    if let Some(retention) = backup_section
        .get("retention_days")
        .and_then(|r| r.as_i64())
    {
        println!("  Retention: {} days", retention);
    }
    if let Some(encryption) = backup_section.get("encryption").and_then(|e| e.as_bool()) {
        println!("  Encryption: {}", if encryption { "age" } else { "none" });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_backup_metrics_parses_prometheus() {
        let metrics = r#"
# HELP hoop_backup_last_success_timestamp Unix timestamp of last successful backup
# TYPE hoop_backup_last_success_timestamp gauge
hoop_backup_last_success_timestamp 1718464800

# HELP hoop_backup_last_size_bytes Size in bytes of last successful backup
# TYPE hoop_backup_last_size_bytes gauge
hoop_backup_last_size_bytes 12345678

# HELP hoop_backup_failures_total Total number of backup failures
# TYPE hoop_backup_failures_total counter
hoop_backup_failures_total 0
"#;
        // Just verify it doesn't panic
        print_backup_metrics(metrics);
    }

    #[test]
    fn print_backup_metrics_handles_zero_timestamp() {
        let metrics = "hoop_backup_last_success_timestamp 0\n";
        print_backup_metrics(metrics);
    }
}
