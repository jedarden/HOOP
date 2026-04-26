//! Script execution API — operator-invoked scripts (§22.3)
//!
//! Scripts at `~/.hoop/scripts/<name>` are executable files that operators
//! can trigger via UI button or `hoop script run <name> [args]`.
//! Runs as HOOP user, captures stdout/stderr + exit code.

use axum::{
    extract::{ConnectInfo, Path as AxumPath, Query, State},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{BufRead, BufReader},
    net::SocketAddr,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};

use crate::DaemonState;
use crate::fleet::{self, ActionKind, ActionResult};

/// Script manifest metadata (from optional manifest.yml next to script)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptManifest {
    /// Script name (must match executable filename)
    pub name: String,
    /// Human-readable description
    #[serde(default)]
    pub description: String,
    /// Visibility scope: global or project-specific
    #[serde(default = "default_scope")]
    pub scope: ScriptScope,
    /// Projects where this script is available (when scope=project)
    #[serde(default)]
    pub projects: Vec<String>,
    /// Execution timeout in seconds (default: 300)
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    /// Optional argument schema for UI prompts
    #[serde(default)]
    pub arguments: Vec<ScriptArgument>,
    /// Cron schedule for automatic execution (5-field format: min hour dom month dow)
    #[serde(default)]
    pub schedule: Option<String>,
    /// How to handle overlapping executions
    #[serde(default = "default_overlap_policy")]
    pub overlap_policy: OverlapPolicy,
    /// Event subscriptions - when matching events fire, this script runs with event JSON on stdin
    #[serde(default)]
    pub on: Vec<EventSubscription>,
}

/// Overlap policy for scheduled script executions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OverlapPolicy {
    /// Skip if previous run is still active (default)
    Skip,
    /// Wait for previous run to finish before starting new run
    Queue,
    /// Allow concurrent runs
    Parallel,
}

fn default_overlap_policy() -> OverlapPolicy {
    OverlapPolicy::Skip
}

fn default_scope() -> ScriptScope {
    ScriptScope::Global
}

fn default_timeout_secs() -> u64 {
    300
}

/// Script visibility scope
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ScriptScope {
    /// Script appears globally
    Global,
    /// Script only appears on matching projects
    Project,
}

/// Script argument definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptArgument {
    /// Argument name
    pub name: String,
    /// What this argument does
    #[serde(default)]
    pub description: String,
    /// Whether argument is required
    #[serde(default)]
    pub required: bool,
    /// Default value (if any)
    #[serde(default)]
    pub default: Option<String>,
}

/// Event subscription for triggering scripts automatically
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSubscription {
    /// Event pattern to match (glob pattern, e.g., "stitch.*", "bead.closed")
    pub event: String,
    /// Project name filter (exact match or glob)
    #[serde(default)]
    pub project: Option<String>,
    /// Kind filter (exact match)
    #[serde(default)]
    pub kind: Option<String>,
    /// Adapter filter (exact match)
    #[serde(default)]
    pub adapter: Option<String>,
    /// Result filter (success or failure)
    #[serde(default)]
    pub result: Option<String>,
}

/// Discovered script entry
#[derive(Debug, Clone, Serialize)]
pub struct ScriptEntry {
    /// Script name
    pub name: String,
    /// Path to executable
    pub path: PathBuf,
    /// Manifest metadata (if present)
    pub manifest: Option<ScriptManifest>,
    /// Whether executable exists and has +x bit
    pub executable: bool,
    /// Last scheduled execution time (if scheduled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_fire: Option<String>,
    /// Next scheduled execution time (if scheduled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_fire: Option<String>,
    /// Whether the script is currently running
    #[serde(default)]
    pub running: bool,
}

/// Script execution request
#[derive(Debug, Deserialize)]
pub struct ScriptRunRequest {
    /// Arguments to pass to the script
    #[serde(default)]
    pub args: Vec<String>,
    /// Optional project context (for project-scoped scripts)
    pub project: Option<String>,
}

/// Script execution response
#[derive(Debug, Serialize, Deserialize)]
pub struct ScriptRunResponse {
    /// Script name
    pub script: String,
    /// Exit code (0 = success)
    pub exit_code: Option<i32>,
    /// Whether execution timed out
    pub timed_out: bool,
    /// Stdout output
    pub stdout: String,
    /// Stderr output
    pub stderr: String,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Human-readable status message
    pub status: String,
}

/// Query parameters for listing scripts
#[derive(Debug, Deserialize)]
pub struct ScriptListQuery {
    /// Filter by project (returns global + matching project scripts)
    pub project: Option<String>,
}

/// Resolve the actor identity for audit purposes.
///
/// Per §13: identity from Tailscale whois where available,
/// falling back to the OS user running the HOOP process.
fn resolve_actor(remote_addr: Option<SocketAddr>) -> String {
    if let Some(addr) = remote_addr {
        let ip = addr.ip();
        let output = std::process::Command::new("tailscale")
            .arg("whois")
            .arg(ip.to_string())
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                let identity = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !identity.is_empty() {
                    return format!("tailscale:{}", identity);
                }
            }
        }
    }

    // Fallback to OS username
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    format!("os:{}", user)
}

/// Compute SHA-256 hash of script arguments for audit integrity
fn hash_args(args: &[String]) -> String {
    let mut hasher = Sha256::new();
    for arg in args {
        hasher.update(arg.as_bytes());
        hasher.update(b"\x00");
    }
    format!("{:x}", hasher.finalize())
}

/// Discover all scripts in the configured scripts directory
pub fn discover_scripts(scripts_dir: &Path) -> Vec<ScriptEntry> {
    let mut scripts = Vec::new();

    let Ok(entries) = fs::read_dir(scripts_dir) else {
        debug!("Scripts directory not readable: {}", scripts_dir.display());
        return scripts;
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();

        // Skip directories and hidden files
        if path.is_dir()
            || path
                .file_name()
                .is_none_or(|n| n.to_string_lossy().starts_with('.'))
        {
            continue;
        }

        let name = path.file_name().unwrap().to_string_lossy().to_string();

        // Check if executable (has +x bit)
        let metadata = fs::metadata(&path);
        let executable = metadata
            .as_ref()
            .ok()
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false);

        // Load optional manifest.yml
        let manifest_path = path.with_extension("yml");
        let manifest = if manifest_path.exists() {
            fs::read_to_string(&manifest_path)
                .ok()
                .and_then(|content| serde_yaml::from_str::<ScriptManifest>(&content).ok())
        } else {
            None
        };

        // Validate manifest name matches script name
        let manifest = manifest.and_then(|m| {
            if m.name == name {
                Some(m)
            } else {
                warn!(
                    "Script manifest name '{}' does not match script filename '{}', ignoring manifest",
                    m.name, name
                );
                None
            }
        });

        scripts.push(ScriptEntry {
            name,
            path,
            manifest,
            executable,
            last_fire: None,
            next_fire: None,
            running: false,
        });
    }

    scripts.sort_by(|a, b| a.name.cmp(&b.name));
    scripts
}

/// Execute a script and capture its output
pub fn execute_script(
    script_path: &Path,
    args: &[String],
    timeout_secs: u64,
) -> Result<ScriptRunResponse, String> {
    let start = Instant::now();

    info!(
        "Executing script: {} with args: {:?}",
        script_path.display(),
        args
    );

    let mut child = Command::new(script_path)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn script: {}", e))?;

    let timeout = Duration::from_secs(timeout_secs);

    // Take stdout and stderr once
    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

    use std::sync::mpsc;
    use std::thread;

    let (stdout_tx, stdout_rx) = mpsc::channel();
    let (stderr_tx, stderr_rx) = mpsc::channel();

    // Spawn thread to collect stdout
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(l) = line {
                let _ = stdout_tx.send(l);
            }
        }
    });

    // Spawn thread to collect stderr
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(l) = line {
                let _ = stderr_tx.send(l);
            }
        }
    });

    // Wait for completion with timeout
    let start_time = Instant::now();
    let mut timed_out = false;

    loop {
        let elapsed = start_time.elapsed();
        if elapsed >= timeout {
            // Kill the child process
            let _ = child.kill();
            timed_out = true;
            break;
        }

        // Check if process has exited
        match child.try_wait() {
            Ok(Some(status)) => {
                // Process has exited
                let exit_code = status.code();
                let duration_ms = elapsed.as_millis() as u64;

                // Collect remaining output with timeout
                let collect_start = Instant::now();
                let mut stdout_lines = Vec::new();
                let mut stderr_lines = Vec::new();

                while stdout_lines.len() + stderr_lines.len() < 1000
                    && collect_start.elapsed() < Duration::from_secs(1)
                {
                    let received = match stdout_rx.recv_timeout(Duration::from_millis(50)) {
                        Ok(l) => {
                            stdout_lines.push(l);
                            true
                        }
                        Err(_) => false,
                    };
                    let _ = stderr_rx
                        .recv_timeout(Duration::from_millis(50))
                        .ok()
                        .map(|l| stderr_lines.push(l));
                    if !received {
                        break;
                    }
                }

                // Drain any remaining
                while let Ok(line) = stdout_rx.try_recv() {
                    stdout_lines.push(line);
                }
                while let Ok(line) = stderr_rx.try_recv() {
                    stderr_lines.push(line);
                }

                let stdout = stdout_lines.join("\n");
                let stderr = stderr_lines.join("\n");

                let status = if exit_code == Some(0) {
                    "Script completed successfully".to_string()
                } else {
                    format!("Script exited with code: {:?}", exit_code)
                };

                return Ok(ScriptRunResponse {
                    script: script_path
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string(),
                    exit_code,
                    timed_out: false,
                    stdout,
                    stderr,
                    duration_ms,
                    status,
                });
            }
            Ok(None) => {
                // Still running, wait a bit
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                return Err(format!("Failed to wait for script: {}", e));
            }
        }
    }

    // If we get here, the script timed out
    let duration_ms = timeout.as_millis() as u64;
    let stdout_lines: Vec<_> = stdout_rx.try_iter().collect();
    let stderr_lines: Vec<_> = stderr_rx.try_iter().collect();

    Ok(ScriptRunResponse {
        script: script_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string(),
        exit_code: None,
        timed_out: true,
        stdout: stdout_lines.join("\n"),
        stderr: stderr_lines.join("\n"),
        duration_ms,
        status: format!("Script timed out after {} seconds", timeout_secs),
    })
}

/// GET /api/scripts — list all discovered scripts
async fn list_scripts(
    State(state): State<DaemonState>,
    Query(params): Query<ScriptListQuery>,
) -> Json<Vec<ScriptEntry>> {
    let scripts_dir_str = state
        .resolved_config
        .agent_extensions_scripts
        .value
        .clone()
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".hoop")
                .join("scripts")
                .to_string_lossy()
                .to_string()
        });

    let scripts_dir = PathBuf::from(&scripts_dir_str);
    let mut scripts = discover_scripts(&scripts_dir);

    // Get schedule state from scheduler
    let schedule_states = if let Some(ref scheduler) = state.script_scheduler {
        scheduler.get_schedule_state().await
    } else {
        std::collections::HashMap::new()
    };

    // Enrich scripts with schedule state
    for script in &mut scripts {
        if let Some(schedule_state) = schedule_states.get(&script.name) {
            script.last_fire = schedule_state.last_fire.clone();
            script.next_fire = schedule_state.next_fire.clone();
            script.running = schedule_state.running;
        }
    }

    // Filter by project if specified
    let filtered = if let Some(project_name) = params.project {
        scripts
            .into_iter()
            .filter(|s| {
                let manifest = s.manifest.as_ref();
                match manifest.map(|m| &m.scope) {
                    None | Some(ScriptScope::Global) => true,
                    Some(ScriptScope::Project) => manifest
                        .and_then(|m| Some(m.projects.contains(&project_name)))
                        .unwrap_or(false),
                }
            })
            .collect()
    } else {
        scripts
    };

    Json(filtered)
}

/// GET /api/scripts/:name — get a specific script's manifest
async fn get_script(
    State(state): State<DaemonState>,
    AxumPath(name): AxumPath<String>,
) -> Result<Json<ScriptEntry>, (axum::http::StatusCode, String)> {
    let scripts_dir_str = state
        .resolved_config
        .agent_extensions_scripts
        .value
        .clone()
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".hoop")
                .join("scripts")
                .to_string_lossy()
                .to_string()
        });

    let scripts_dir = PathBuf::from(&scripts_dir_str);
    let script_path = scripts_dir.join(&name);

    if !script_path.exists() {
        return Err((
            axum::http::StatusCode::NOT_FOUND,
            format!("Script not found: {}", name),
        ));
    }

    let scripts = discover_scripts(&scripts_dir);
    let script = scripts
        .into_iter()
        .find(|s| s.name == name)
        .ok_or_else(|| {
            (
                axum::http::StatusCode::NOT_FOUND,
                format!("Script not found: {}", name),
            )
        })?;

    Ok(Json(script))
}

/// POST /api/scripts/:name/run — execute a script
async fn run_script(
    State(state): State<DaemonState>,
    AxumPath(name): AxumPath<String>,
    Json(req): Json<ScriptRunRequest>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
) -> Result<Json<ScriptRunResponse>, (axum::http::StatusCode, String)> {
    let scripts_dir_str = state
        .resolved_config
        .agent_extensions_scripts
        .value
        .clone()
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".hoop")
                .join("scripts")
                .to_string_lossy()
                .to_string()
        });

    let scripts_dir = PathBuf::from(&scripts_dir_str);
    let script_path = scripts_dir.join(&name);

    if !script_path.exists() {
        return Err((
            axum::http::StatusCode::NOT_FOUND,
            format!("Script not found: {}", name),
        ));
    }

    // Check if executable
    let metadata = fs::metadata(&script_path).map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read script metadata: {}", e),
        )
    })?;

    if metadata.permissions().mode() & 0o111 == 0 {
        return Err((
            axum::http::StatusCode::FORBIDDEN,
            format!("Script is not executable: {}", name),
        ));
    }

    // Get timeout from manifest or use default
    let timeout_secs = {
        let manifest_path = script_path.with_extension("yml");
        if manifest_path.exists() {
            fs::read_to_string(&manifest_path)
                .ok()
                .and_then(|content| serde_yaml::from_str::<ScriptManifest>(&content).ok())
                .map(|m| m.timeout_secs)
                .unwrap_or(300)
        } else {
            300
        }
    };

    // Execute the script (blocking call in spawn_blocking)
    let script_path_clone = script_path.clone();
    let args = req.args.clone();
    let result = tokio::task::spawn_blocking(move || {
        execute_script(&script_path_clone, &args, timeout_secs)
    })
    .await
    .map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to join script execution task: {}", e),
        )
    })?
    .map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Script execution failed: {}", e),
        )
    })?;

    // Write audit row for script execution
    let actor = resolve_actor(connect_info.map(|ci| ci.0));
    let args_json = serde_json::to_string(&req.args).ok();
    let args_hash = hash_args(&req.args);
    let audit_result = if result.exit_code == Some(0) {
        ActionResult::Success
    } else {
        ActionResult::Failure
    };
    let audit_error = if result.exit_code != Some(0) {
        Some(result.status.clone())
    } else {
        None
    };

    if let Err(e) = fleet::write_audit_row(
        &actor,
        ActionKind::ScriptExecuted,
        &name,
        req.project.as_deref(),
        args_json,
        audit_result,
        audit_error,
        Some("api"),
        None,
        Some(&args_hash),
    ) {
        warn!("Failed to write audit row for script execution: {}", e);
    }

    info!("Script '{}' completed with status: {}", name, result.status);

    Ok(Json(result))
}

pub fn router() -> Router<DaemonState> {
    Router::new()
        .route("/api/scripts", get(list_scripts))
        .route("/api/scripts/:name", get(get_script))
        .route("/api/scripts/:name/run", post(run_script))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_scope_default() {
        let manifest = ScriptManifest {
            name: "test".to_string(),
            description: String::new(),
            scope: default_scope(),
            projects: Vec::new(),
            timeout_secs: default_timeout_secs(),
            arguments: Vec::new(),
            schedule: None,
            overlap_policy: default_overlap_policy(),
            on: Vec::new(),
        };
        assert_eq!(manifest.scope, ScriptScope::Global);
        assert_eq!(manifest.timeout_secs, 300);
    }
}
