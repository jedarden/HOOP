//! bead CLI adapter for HOOP
//!
//! HOOP shells out to the configured bead CLI (default: `bead`) for all bead operations.
//! This module provides a configurable adapter that:
//! - Uses a configurable command name (defaults to "bead", the bead-rs CLI)
//! - Handles JSON vs JSONL parsing differences
//! - Maps status vocabulary between CLIs (e.g., "in_progress" → "claimed")
//! - Provides typed wrappers for list/show/create operations
//!
//! ## CLI Differences
//!
//! **bead-rs (`bead` CLI):**
//! - `bead list --json` outputs JSONL (one JSON object per line)
//! - Status values: `open`, `in_progress`, `closed`
//! - Close requires explicit `bead close` command
//!
//! **beads_rust (`br` CLI):**
//! - `br list --json` outputs JSON array
//! - Status values: `open`, `claimed`, `closed`
//! - Close changes status directly
//!
//! This adapter normalizes these differences for HOOP.

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use std::process::Command;

/// The configured bead CLI command name.
///
/// Defaults to "bead" (the bead-rs CLI). Can be overridden via environment variable
/// HOOP_BEAD_CLI or config setting.
const DEFAULT_BEAD_CLI: &str = "bead";

/// Get the configured bead CLI command name.
///
/// Checks HOOP_BEAD_CLI environment variable first, then defaults to "bead".
pub fn bead_cli_command() -> String {
    std::env::var("HOOP_BEAD_CLI").unwrap_or_else(|_| DEFAULT_BEAD_CLI.to_string())
}

/// Normalize bead status from bead CLI to HOOP's internal representation.
///
/// bead-rs uses "in_progress", HOOP historically used "claimed" from beads_rust.
/// This function maps "in_progress" → "claimed" and passes through other values.
pub fn normalize_status(status: &str) -> String {
    match status {
        "in_progress" => "claimed".to_string(),
        other => other.to_string(),
    }
}

/// Denormalize bead status from HOOP's internal representation to bead CLI format.
///
/// Maps HOOP's "claimed" back to bead-rs's "in_progress".
pub fn denormalize_status(status: &str) -> String {
    match status {
        "claimed" => "in_progress".to_string(),
        other => other.to_string(),
    }
}

/// A bead as returned by the bead CLI.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Bead {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub status: String,
    #[serde(default)]
    pub priority: i64,
    #[serde(default)]
    pub issue_type: String,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub assignee: Option<String>,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

impl Bead {
    /// Get the normalized status for HOOP's internal representation.
    pub fn normalized_status(&self) -> String {
        normalize_status(&self.status)
    }
}

/// Parse JSONL output from `bead list --json`.
///
/// bead-rs outputs one JSON object per line (JSONL format).
/// Returns a Vec of parsed beads.
pub fn parse_bead_list_jsonl(output: &str) -> Result<Vec<Bead>> {
    let mut beads = Vec::new();

    for (line_num, line) in output.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match serde_json::from_str::<Bead>(line) {
            Ok(bead) => beads.push(bead),
            Err(e) => {
                // Try to parse as Value for better error message
                if let Ok(value) = serde_json::from_str::<Value>(line) {
                    tracing::warn!(
                        "Line {} of bead list output has unexpected format: {}",
                        line_num + 1,
                        value
                    );
                } else {
                    bail!(
                        "Failed to parse line {} of bead list output as JSON: {}",
                        line_num + 1,
                        e
                    );
                }
            }
        }
    }

    Ok(beads)
}

/// Parse JSON array output from legacy `br list --json`.
///
/// beads_rust outputs a JSON array of beads.
/// This function is kept for backward compatibility during migration.
pub fn parse_bead_list_json_array(output: &str) -> Result<Vec<Bead>> {
    let beads: Vec<Bead> = serde_json::from_str(output)
        .context("Failed to parse bead list output as JSON array")?;
    Ok(beads)
}

/// Auto-detect format and parse bead list output.
///
/// Tries JSONL first (bead-rs format), falls back to JSON array (beads_rust format).
pub fn parse_bead_list_auto(output: &str) -> Result<Vec<Bead>> {
    // Try JSONL first
    match parse_bead_list_jsonl(output) {
        Ok(beads) if !beads.is_empty() => return Ok(beads),
        _ => {},
    }

    // Fall back to JSON array
    parse_bead_list_json_array(output)
}

/// Invoke the bead CLI with given arguments.
///
/// # Arguments
///
/// * `args` - Arguments to pass to the bead CLI (e.g., `["list", "--json"]`)
/// * `current_dir` - Optional current directory for the command
///
/// # Returns
///
/// * `Ok(output)` - The subprocess stdout/stderr if successful
/// * `Err(e)` - Any error from spawning or executing the subprocess
pub fn invoke_bead_cli(args: &[&str], current_dir: Option<&Path>) -> Result<std::process::Output> {
    let cmd_name = bead_cli_command();
    let mut cmd = Command::new(&cmd_name);

    for arg in args {
        cmd.arg(arg);
    }

    if let Some(dir) = current_dir {
        cmd.current_dir(dir);
    }

    cmd.output()
        .with_context(|| format!("Failed to execute {} command", cmd_name))
}

/// Invoke `bead list --json` and parse the output.
///
/// # Arguments
///
/// * `workspace_path` - Path to the workspace directory (containing `.beads/`)
///
/// # Returns
///
/// * `Ok(beads)` - Parsed list of beads
/// * `Err(e)` - Any error from command execution or parsing
pub fn list_beads(workspace_path: &Path) -> Result<Vec<Bead>> {
    let parent = workspace_path
        .parent()
        .ok_or_else(|| anyhow!("Workspace path has no parent directory"))?;

    let output = invoke_bead_cli(&["list", "--json"], Some(parent))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "bead list failed: {}",
            stderr.trim()
        );
    }

    let stdout = std::str::from_utf8(&output.stdout)
        .context("bead list output is not valid UTF-8")?;

    parse_bead_list_auto(stdout)
}

/// Invoke `bead show <id>` and parse the output.
///
/// # Arguments
///
/// * `workspace_path` - Path to the workspace directory
/// * `bead_id` - ID of the bead to show
///
/// # Returns
///
/// * `Ok(bead)` - The parsed bead
/// * `Err(e)` - Any error from command execution or parsing
pub fn show_bead(workspace_path: &Path, bead_id: &str) -> Result<Bead> {
    let parent = workspace_path
        .parent()
        .ok_or_else(|| anyhow!("Workspace path has no parent directory"))?;

    let output = invoke_bead_cli(&["show", bead_id], Some(parent))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "bead show {} failed: {}",
            bead_id,
            stderr.trim()
        );
    }

    let stdout = std::str::from_utf8(&output.stdout)
        .context("bead show output is not valid UTF-8")?;

    // bead show outputs JSON (not JSONL)
    let bead: Bead = serde_json::from_str(stdout)
        .context("Failed to parse bead show output as JSON")?;

    Ok(bead)
}

/// Invoke `bead create` with given arguments.
///
/// # Arguments
///
/// * `workspace_path` - Path to the workspace directory
/// * `args` - Arguments to pass to `bead create`
///
/// # Returns
///
/// * `Ok(output)` - The subprocess output
/// * `Err(e)` - Any error from command execution
pub fn create_bead(workspace_path: &Path, args: &[&str]) -> Result<std::process::Output> {
    let parent = workspace_path
        .parent()
        .ok_or_else(|| anyhow!("Workspace path has no parent directory"))?;

    let mut create_args = vec!["create"];
    create_args.extend(args);

    let output = invoke_bead_cli(&create_args, Some(parent))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "bead create failed: {}",
            stderr.trim()
        );
    }

    Ok(output)
}

/// Check if the bead CLI is available and working.
///
/// Runs `bead --version` to verify the CLI is installed and executable.
///
/// # Returns
///
/// * `Ok(version)` - The version string if available
/// * `Err(e)` - Any error from command execution
pub fn check_bead_cli_available() -> Result<String> {
    let output = invoke_bead_cli(&["--version"], None)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "bead --version failed: {}",
            stderr.trim()
        );
    }

    let stdout = std::str::from_utf8(&output.stdout)
        .context("bead --version output is not valid UTF-8")?;

    Ok(stdout.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_bead_cli_command() {
        assert_eq!(bead_cli_command(), "bead");
    }

    #[test]
    fn test_env_override_bead_cli_command() {
        std::env::set_var("HOOP_BEAD_CLI", "br");
        assert_eq!(bead_cli_command(), "br");
        std::env::remove_var("HOOP_BEAD_CLI");
    }

    #[test]
    fn test_normalize_status() {
        assert_eq!(normalize_status("in_progress"), "claimed");
        assert_eq!(normalize_status("open"), "open");
        assert_eq!(normalize_status("closed"), "closed");
    }

    #[test]
    fn test_denormalize_status() {
        assert_eq!(denormalize_status("claimed"), "in_progress");
        assert_eq!(denormalize_status("open"), "open");
        assert_eq!(denormalize_status("closed"), "closed");
    }

    #[test]
    fn test_parse_bead_list_jsonl() {
        let jsonl = r#"{"id":"hoop-abc123","title":"Test bead","status":"in_progress"}
{"id":"hoop-def456","title":"Another bead","status":"open"}"#;

        let beads = parse_bead_list_jsonl(jsonl).unwrap();
        assert_eq!(beads.len(), 2);
        assert_eq!(beads[0].id, "hoop-abc123");
        assert_eq!(beads[0].normalized_status(), "claimed");
        assert_eq!(beads[1].id, "hoop-def456");
        assert_eq!(beads[1].normalized_status(), "open");
    }

    #[test]
    fn test_parse_bead_list_json_array() {
        let json_array = r#"[{"id":"hoop-abc123","title":"Test bead","status":"claimed"}]"#;

        let beads = parse_bead_list_json_array(json_array).unwrap();
        assert_eq!(beads.len(), 1);
        assert_eq!(beads[0].id, "hoop-abc123");
        assert_eq!(beads[0].status, "claimed");
    }

    #[test]
    fn test_parse_bead_list_auto_jsonl() {
        let jsonl = r#"{"id":"hoop-abc123","title":"Test","status":"in_progress"}"#;

        let beads = parse_bead_list_auto(jsonl).unwrap();
        assert_eq!(beads.len(), 1);
        assert_eq!(beads[0].normalized_status(), "claimed");
    }

    #[test]
    fn test_parse_bead_list_auto_json_array() {
        let json_array = r#"[{"id":"hoop-abc123","title":"Test","status":"claimed"}]"#;

        let beads = parse_bead_list_auto(json_array).unwrap();
        assert_eq!(beads.len(), 1);
        assert_eq!(beads[0].status, "claimed");
    }

    #[test]
    fn test_parse_bead_list_jsonl_empty_lines() {
        let jsonl = r#"{"id":"hoop-abc123","title":"Test","status":"in_progress"}

{"id":"hoop-def456","title":"Another","status":"open"}
"#;

        let beads = parse_bead_list_jsonl(jsonl).unwrap();
        assert_eq!(beads.len(), 2);
    }

    #[test]
    fn test_parse_bead_list_jsonl_invalid_json() {
        let jsonl = r#"{"id":"hoop-abc123","title":"Test","status":"in_progress"}
not a json line
{"id":"hoop-def456","title":"Another","status":"open"}"#;

        let result = parse_bead_list_jsonl(jsonl);
        assert!(result.is_err());
    }
}
