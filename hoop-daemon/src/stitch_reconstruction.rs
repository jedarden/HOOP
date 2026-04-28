//! Stitch Replay: reconstruct failure state + resume-as-new-bead OR continue-in-agent
//!
//! When a Stitch's linked bead fails, offer two resume options:
//! 1. **Resume as new bead** — HOOP creates a new bead with same `stitch:<id>` label,
//!    pre-populated with "pick up from step N" prompt + reconstructed context.
//! 2. **Continue in agent** — (phase 5) the agent inherits the reconstructed state in its chat.
//!
//! Reconstruction sources:
//! - NEEDLE events (fail events include stash_sha)
//! - CLI session JSONL
//! - Worktree git state at time of failure (via git stash-create hook from NEEDLE)
//!
//! Acceptance:
//! - Reconstruction completes in <10s for a 30-minute-work-session failure
//! - Resume-as-new-bead tested: new bead picks up correctly; old failed bead stays closed as history
//! - Continue-in-agent: agent sees reconstructed state via MCP context (phase 5)
//! - NEEDLE hook: `git stash create` on bead failure, stash SHA recorded in events

use anyhow::{anyhow, bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

use crate::events::NeedleEvent;

/// Reconstructed failure state for a bead
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureState {
    /// The bead ID that failed
    pub bead_id: String,
    /// The stitch ID this bead belongs to
    pub stitch_id: String,
    /// Workspace/project path
    pub workspace: String,
    /// The failure event
    pub fail_event: FailEvent,
    /// Conversation history leading to failure
    pub conversation_history: Vec<ConversationMessage>,
    /// CLI session data
    pub cli_session: Option<SessionData>,
    /// Git state at failure (stash SHA)
    pub git_state: Option<GitState>,
    /// Files that were being worked on
    pub touched_files: Vec<String>,
    /// Time work started
    pub work_started_at: DateTime<Utc>,
    /// Time work failed
    pub failed_at: DateTime<Utc>,
    /// Duration of work before failure
    pub duration_ms: u64,
}

/// Failure event details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailEvent {
    /// Timestamp of failure
    pub timestamp: DateTime<Utc>,
    /// Worker that was executing
    pub worker: String,
    /// Error message
    pub error: Option<String>,
    /// Duration of execution before failure
    pub duration_ms: Option<u64>,
    /// Git stash SHA at time of failure
    pub stash_sha: Option<String>,
}

/// A message from the conversation history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    /// Role: user, assistant, system, tool
    pub role: String,
    /// Message content
    pub content: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Token count (if available)
    pub tokens: Option<i64>,
}

/// CLI session data at time of failure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    /// Session ID
    pub session_id: String,
    /// CLI adapter used (e.g., "claude")
    pub adapter: String,
    /// Model used
    pub model: String,
    /// Current working directory
    pub cwd: PathBuf,
    /// Session start time
    pub started_at: DateTime<Utc>,
    /// Commands run during session
    pub commands: Vec<SessionCommand>,
}

/// A command run during the session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCommand {
    /// Command string
    pub command: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Exit code (if completed)
    pub exit_code: Option<i32>,
}

/// Git state at time of failure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitState {
    /// Stash SHA created at failure
    pub stash_sha: String,
    /// Branch name
    pub branch: String,
    /// Latest commit SHA
    pub commit_sha: String,
    /// Number of modified files
    pub modified_files_count: usize,
}

/// Replay options for a failed bead
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayOptions {
    /// The original bead ID that failed
    pub original_bead_id: String,
    /// The stitch ID
    pub stitch_id: String,
    /// Suggested title for new bead (if resuming as new)
    pub suggested_title: String,
    /// Suggested body for new bead
    pub suggested_body: String,
    /// Labels to include (stitch:<id> preserved)
    pub labels: Vec<String>,
    /// Step number to resume from (e.g., "step 3 of 5")
    pub resume_step: String,
    /// Failure state (for continue-in-agent)
    pub failure_state: FailureState,
}

/// Reconstruct failure state for a failed bead
///
/// # Arguments
/// * `bead_id` - The bead that failed
/// * `workspace` - Project path
/// * `events_jsonl_path` - Path to events.jsonl
///
/// # Returns
/// Reconstructed failure state or error
pub fn reconstruct_failure_state(
    bead_id: &str,
    workspace: &Path,
    events_jsonl_path: &Path,
) -> Result<FailureState> {
    let start = std::time::Instant::now();

    // 1. Parse events.jsonl to find the failure event and work start
    let (fail_event, work_started_at, stitch_id, worker) =
        parse_fail_event(bead_id, events_jsonl_path)?;

    let failed_at = fail_event.timestamp;
    let duration_ms = fail_event.duration_ms.unwrap_or(0);

    info!(
        "Reconstructing failure for bead {} (stitch: {}, worker: {})",
        bead_id, stitch_id, worker
    );

    // 2. Load conversation history from stitch_messages table
    let conversation_history = load_conversation_history(&stitch_id, workspace)?;

    // 3. Load CLI session data from workspace
    let cli_session = load_cli_session(&worker, workspace, &work_started_at, &failed_at)?;

    // 4. Load git state from stash_sha
    let git_state = if let Some(ref stash_sha) = fail_event.stash_sha {
        load_git_state(stash_sha, workspace)?
    } else {
        warn!("No stash_sha in fail event for bead {}", bead_id);
        None
    };

    // 5. Extract touched files from conversation
    let touched_files = extract_touched_files(&conversation_history);

    let reconstruction_time = start.elapsed();
    info!(
        "Failure state reconstruction for bead {} took {}ms",
        bead_id,
        reconstruction_time.as_millis()
    );

    Ok(FailureState {
        bead_id: bead_id.to_string(),
        stitch_id,
        workspace: workspace.display().to_string(),
        fail_event,
        conversation_history,
        cli_session,
        git_state,
        touched_files,
        work_started_at,
        failed_at,
        duration_ms,
    })
}

/// Parse events.jsonl to find the Fail event for a bead
fn parse_fail_event(
    bead_id: &str,
    events_jsonl_path: &Path,
) -> Result<(FailEvent, DateTime<Utc>, String, String)> {
    let file = std::fs::File::open(events_jsonl_path)
        .with_context(|| format!("Failed to open events file {}", events_jsonl_path.display()))?;

    use std::io::{BufRead, BufReader};
    let reader = BufReader::new(file);

    let mut fail_event: Option<FailEvent> = None;
    let mut dispatch_time: Option<DateTime<Utc>> = None;
    let mut stitch_id: Option<String> = None;
    let mut worker: Option<String> = None;

    for line in reader.lines() {
        let line = line.context("Failed to read line from events file")?;

        // Parse as NeedleEvent
        if let Ok(event) = serde_json::from_str::<NeedleEvent>(&line) {
            match event {
                NeedleEvent::Dispatch {
                    ts,
                    bead,
                    worker: w,
                    ..
                } if bead == bead_id => {
                    debug!("Found Dispatch event for bead {}", bead_id);
                    dispatch_time = Some(DateTime::parse_from_rfc3339(&ts)
                        .context("Failed to parse dispatch timestamp")?
                        .with_timezone(&Utc));
                    worker = Some(w);
                }
                NeedleEvent::Fail {
                    ts,
                    bead,
                    worker: w,
                    error,
                    duration_ms,
                    stash_sha,
                } if bead == bead_id => {
                    debug!("Found Fail event for bead {}", bead_id);
                    let w_clone = w.clone();
                    fail_event = Some(FailEvent {
                        timestamp: DateTime::parse_from_rfc3339(&ts)
                            .context("Failed to parse fail timestamp")?
                            .with_timezone(&Utc),
                        worker: w_clone.clone(),
                        error,
                        duration_ms,
                        stash_sha,
                    });
                    worker = Some(w);
                }
                _ => {}
            }
        }

        // Also look for stitch ID in raw JSON
        if stitch_id.is_none() {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) {
                if let Some(sid) = value.get("stitch_id").and_then(|v| v.as_str()) {
                    stitch_id = Some(sid.to_string());
                }
            }
        }

        // Stop once we have both dispatch and fail events
        if fail_event.is_some() && dispatch_time.is_some() && stitch_id.is_some() {
            break;
        }
    }

    let fail_event = fail_event.context("No Fail event found for bead")?;
    let worker = worker.context("No worker found for bead")?;
    let stitch_id = stitch_id.context("No stitch_id found in events")?;

    // Work started at dispatch time, or fail_time - duration if dispatch not found
    let work_started_at = dispatch_time.unwrap_or_else(|| {
        fail_event.timestamp - chrono::Duration::milliseconds(fail_event.duration_ms.unwrap_or(0) as i64)
    });

    Ok((fail_event, work_started_at, stitch_id, worker))
}

/// Load conversation history from stitch_messages table
fn load_conversation_history(stitch_id: &str, workspace: &Path) -> Result<Vec<ConversationMessage>> {
    use rusqlite::Connection;

    let fleet_db = crate::fleet::db_path();
    let conn = Connection::open(&fleet_db)
        .with_context(|| format!("Failed to open fleet.db at {}", fleet_db.display()))?;

    let mut stmt = conn.prepare(
        "SELECT ts, role, content, tokens
         FROM stitch_messages
         WHERE stitch_id = ?1
         ORDER BY ts ASC"
    ).context("Failed to prepare stitch_messages query")?;

    let messages = stmt
        .query_map(rusqlite::params![stitch_id], |row| {
            Ok(ConversationMessage {
                role: row.get(1)?,
                content: row.get(2)?,
                timestamp: {
                    let ts: String = row.get(0)?;
                    DateTime::parse_from_rfc3339(&ts)
                        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
                        .with_timezone(&Utc)
                },
                tokens: row.get(3)?,
            })
        })
        .context("Failed to query stitch_messages")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect conversation messages")?;

    debug!("Loaded {} messages from stitch_messages for stitch {}", messages.len(), stitch_id);

    Ok(messages)
}

/// Load CLI session data from workspace
fn load_cli_session(
    worker: &str,
    workspace: &Path,
    work_started_at: &DateTime<Utc>,
    failed_at: &DateTime<Utc>,
) -> Result<Option<SessionData>> {
    // CLI sessions are stored in .beads/cli-sessions/<worker>/session.jsonl
    let session_path = workspace
        .join(".beads")
        .join("cli-sessions")
        .join(worker)
        .join("session.jsonl");

    if !session_path.exists() {
        debug!("No CLI session found at {}", session_path.display());
        return Ok(None);
    }

    use std::io::{BufRead, BufReader};
    let file = std::fs::File::open(&session_path)
        .with_context(|| format!("Failed to open session file {}", session_path.display()))?;
    let reader = BufReader::new(file);

    let mut commands: Vec<SessionCommand> = Vec::new();
    let mut session_id: Option<String> = None;
    let mut adapter: Option<String> = None;
    let mut model: Option<String> = None;
    let mut cwd: Option<PathBuf> = None;
    let mut started_at: Option<DateTime<Utc>> = None;

    for line in reader.lines() {
        let line = line.context("Failed to read line from session file")?;

        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) {
            // Extract session metadata
            if session_id.is_none() {
                if let Some(sid) = value.get("session_id").and_then(|v| v.as_str()) {
                    session_id = Some(sid.to_string());
                }
            }
            if adapter.is_none() {
                if let Some(a) = value.get("adapter").and_then(|v| v.as_str()) {
                    adapter = Some(a.to_string());
                }
            }
            if model.is_none() {
                if let Some(m) = value.get("model").and_then(|v| v.as_str()) {
                    model = Some(m.to_string());
                }
            }
            if cwd.is_none() {
                if let Some(c) = value.get("cwd").and_then(|v| v.as_str()) {
                    cwd = Some(PathBuf::from(c));
                }
            }
            if started_at.is_none() {
                if let Some(ts) = value.get("started_at").and_then(|v| v.as_str()) {
                    if let Ok(dt) = DateTime::parse_from_rfc3339(ts) {
                        started_at = Some(dt.with_timezone(&Utc));
                    }
                }
            }

            // Extract commands
            if let Some(cmd) = value.get("cmd").and_then(|v| v.as_str()) {
                if let Some(ts) = value.get("timestamp").and_then(|v| v.as_str()) {
                    if let Ok(timestamp) = DateTime::parse_from_rfc3339(ts) {
                        // Only include commands within the work window
                        if timestamp >= *work_started_at && timestamp <= *failed_at {
                            commands.push(SessionCommand {
                                command: cmd.to_string(),
                                timestamp: timestamp.with_timezone(&Utc),
                                exit_code: value.get("exit_code").and_then(|v| v.as_i64()).map(|v| v as i32),
                            });
                        }
                    }
                }
            }
        }
    }

    if session_id.is_some() || !commands.is_empty() {
        debug!("Loaded CLI session with {} commands for worker {}", commands.len(), worker);
        Ok(Some(SessionData {
            session_id: session_id.unwrap_or_else(|| format!("session-{}", worker)),
            adapter: adapter.unwrap_or_else(|| "claude".to_string()),
            model: model.unwrap_or_else(|| "unknown".to_string()),
            cwd: cwd.unwrap_or_else(|| workspace.to_path_buf()),
            started_at: started_at.unwrap_or(*work_started_at),
            commands,
        }))
    } else {
        Ok(None)
    }
}

/// Load git state from stash SHA
fn load_git_state(stash_sha: &str, workspace: &Path) -> Result<Option<GitState>> {
    use std::process::Command;

    // Get current branch
    let branch = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(workspace)
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Get current commit
    let commit_sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(workspace)
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Try to get stash info
    let modified_files_count = Command::new("git")
        .args(["stash", "show", stash_sha, "--name-only"])
        .current_dir(workspace)
        .output()
        .ok()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter(|l| !l.is_empty())
                .count()
        })
        .unwrap_or(0);

    debug!(
        "Loaded git state: branch={}, commit={}, stash_sha={}, modified_files={}",
        branch, commit_sha, stash_sha, modified_files_count
    );

    Ok(Some(GitState {
        stash_sha: stash_sha.to_string(),
        branch,
        commit_sha,
        modified_files_count,
    }))
}

/// Extract touched files from conversation history
fn extract_touched_files(messages: &[ConversationMessage]) -> Vec<String> {
    use regex::Regex;

    let file_re = Regex::new(r"[`']?([a-zA-Z0-9_\-./]+\.[a-zA-Z0-9]+)[`']?").unwrap();

    let mut files = std::collections::HashSet::new();

    for msg in messages {
        // Look for file paths in tool outputs and messages
        if msg.role == "tool" || msg.role == "assistant" {
            for cap in file_re.captures_iter(&msg.content) {
                if let Some(file) = cap.get(1) {
                    let file_str = file.as_str();
                    // Filter out common non-code paths
                    if !file_str.starts_with("http")
                        && !file_str.starts_with("/tmp/")
                        && file_str.contains('.')
                    {
                        files.insert(file_str.to_string());
                    }
                }
            }
        }
    }

    files.into_iter().collect()
}

/// Generate replay options for a failed bead
pub fn generate_replay_options(failure_state: &FailureState) -> ReplayOptions {
    let original_bead_id = failure_state.bead_id.clone();
    let stitch_id = failure_state.stitch_id.clone();

    // Generate suggested title
    let suggested_title = format!(
        "Resume: {} (failed at {})",
        original_bead_id,
        failure_state.fail_event.timestamp.format("%H:%M")
    );

    // Generate suggested body with context
    let mut body = format!(
        "## Resume from failure\n\n\
         Original bead `{}` failed at {} with error:\n\
         ```
         {}\n\
         ```\n\n",
        original_bead_id,
        failure_state.fail_event.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
        failure_state.fail_event.error.as_deref().unwrap_or("unknown error")
    );

    // Add work context
    if let Some(ref cli) = failure_state.cli_session {
        body.push_str(&format!(
            "### Work session ({} minutes)\n\
             - Adapter: {}\n\
             - Model: {}\n\
             - Commands run: {}\n\n",
            failure_state.duration_ms / 60000,
            cli.adapter,
            cli.model,
            cli.commands.len()
        ));
    }

    // Add touched files
    if !failure_state.touched_files.is_empty() {
        body.push_str("### Files being worked on\n\n");
        for file in &failure_state.touched_files {
            body.push_str(&format!("- `{}`\n", file));
        }
        body.push('\n');
    }

    // Add git state
    if let Some(ref git) = failure_state.git_state {
        body.push_str(&format!(
            "### Git state\n\
             - Stash: `{}`\n\
             - Branch: `{}`\n\
             - Modified files: {}\n\n",
            git.stash_sha, git.branch, git.modified_files_count
        ));
    }

    body.push_str("## Next steps\n\nPick up from where the work left off.");

    // Calculate resume step
    let step_num = failure_state.conversation_history.len() / 2; // Rough estimate
    let resume_step = format!("approximately step {}", step_num);

    // Labels include stitch:<id>
    let labels = vec![format!("stitch:{}", stitch_id), "resume".to_string()];

    ReplayOptions {
        original_bead_id,
        stitch_id,
        suggested_title,
        suggested_body: body,
        labels,
        resume_step,
        failure_state: failure_state.clone(),
    }
}

/// Restore workspace state from git stash
pub fn restore_workspace_state(stash_sha: &str, workspace: &Path) -> Result<()> {
    info!("Restoring workspace state from stash {}", stash_sha);

    use std::process::Command;

    // Apply stash
    let output = Command::new("git")
        .args(["stash", "apply", stash_sha])
        .current_dir(workspace)
        .output()
        .context("Failed to apply git stash")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to apply stash {}: {}", stash_sha, stderr);
    }

    info!("Workspace state restored from stash {}", stash_sha);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_extract_touched_files_from_conversation() {
        let messages = vec![
            ConversationMessage {
                role: "user".to_string(),
                content: "Fix the bug in src/main.rs".to_string(),
                timestamp: Utc::now(),
                tokens: None,
            },
            ConversationMessage {
                role: "assistant".to_string(),
                content: "I'll help you fix src/main.rs and src/utils.rs".to_string(),
                timestamp: Utc::now(),
                tokens: None,
            },
            ConversationMessage {
                role: "tool".to_string(),
                content: "Read src/main.rs\nRead src/utils.rs\nWrite src/config.toml".to_string(),
                timestamp: Utc::now(),
                tokens: None,
            },
        ];

        let files = extract_touched_files(&messages);

        assert!(files.contains(&"src/main.rs".to_string()));
        assert!(files.contains(&"src/utils.rs".to_string()));
        assert!(files.contains(&"src/config.toml".to_string()));
    }

    #[test]
    fn test_generate_replay_options() {
        let failure_state = FailureState {
            bead_id: "test-bead-1".to_string(),
            stitch_id: "stitch-abc123".to_string(),
            workspace: "/tmp/test".to_string(),
            fail_event: FailEvent {
                timestamp: Utc::now(),
                worker: "alpha".to_string(),
                error: Some("syntax error".to_string()),
                duration_ms: Some(300000),
                stash_sha: Some("abc123".to_string()),
            },
            conversation_history: vec![],
            cli_session: None,
            git_state: None,
            touched_files: vec!["src/main.rs".to_string()],
            work_started_at: Utc::now() - chrono::Duration::minutes(5),
            failed_at: Utc::now(),
            duration_ms: 300000,
        };

        let options = generate_replay_options(&failure_state);

        assert_eq!(options.original_bead_id, "test-bead-1");
        assert_eq!(options.stitch_id, "stitch-abc123");
        assert!(options.suggested_title.contains("Resume"));
        assert!(options.suggested_body.contains("syntax error"));
        assert!(options.labels.contains(&"stitch:stitch-abc123".to_string()));
        assert!(options.labels.contains(&"resume".to_string()));
    }

    #[test]
    fn test_reconstruct_failure_state_missing_events_file() {
        let tmp_dir = TempDir::new().unwrap();
        let workspace = tmp_dir.path();

        let result = reconstruct_failure_state(
            "nonexistent-bead",
            workspace,
            &workspace.join("nonexistent-events.jsonl"),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_reconstruct_failure_state_with_events_file() {
        let tmp_dir = TempDir::new().unwrap();
        let workspace = tmp_dir.path();
        let events_file = workspace.join("events.jsonl");

        // Write test events
        let mut file = fs::File::create(&events_file).unwrap();
        writeln!(
            file,
            r#"{{"event":"dispatch","ts":"2026-04-26T10:00:00Z","worker":"alpha","bead":"test-bead-1","adapter":"claude","model":"claude-opus-4-6","stitch_id":"stitch-abc123"}}"#
        ).unwrap();
        writeln!(
            file,
            r#"{{"event":"fail","ts":"2026-04-26T10:05:00Z","worker":"alpha","bead":"test-bead-1","error":"test error","duration_ms":300000,"stash_sha":"abc123def456"}}"#
        ).unwrap();

        // This will fail because fleet.db doesn't exist, but we can test the parsing part
        let result = reconstruct_failure_state("test-bead-1", workspace, &events_file);

        // Should fail at fleet.db open, not at event parsing
        assert!(result.is_err());
        if let Err(e) = result {
            let msg = e.to_string();
            // Should have parsed the events successfully before failing at DB
            assert!(msg.contains("fleet.db") || msg.contains("stitch_messages") || msg.contains("no such table"));
        }
    }
}
