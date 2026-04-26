//! Event-triggered script runner with pattern matching (§22.3)
//!
//! Scripts can subscribe to events via their manifest.yml `on:` field.
//! When a matching event fires, the script is executed with the event JSON on stdin.
//!
//! Event patterns support:
//! - Glob matching on event type (e.g., "stitch.*", "bead.*")
//! - Glob matching on project name
//! - Exact matching on kind, adapter, and result filters

use glob::Pattern;
use std::path::Path;
use tracing::{debug, info, warn};

use crate::api_scripts::{discover_scripts, EventSubscription, ScriptEntry};
use crate::events::NeedleEvent;

/// Context for event matching - extracted from NeedleEvent for subscription filtering
#[derive(Debug, Clone)]
pub struct EventContext {
    /// Event type (e.g., "claim", "dispatch", "complete", "fail", etc.)
    pub event_type: String,
    /// Project name (if available from bead lookup)
    pub project: Option<String>,
    /// Bead kind (if available)
    pub kind: Option<String>,
    /// Adapter name (if available)
    pub adapter: Option<String>,
    /// Result ("success" or "failure")
    pub result: Option<String>,
    /// Raw event JSON for passing to script stdin
    pub event_json: String,
    /// Bead ID
    pub bead_id: String,
}

impl EventContext {
    /// Extract event context from a NeedleEvent
    pub fn from_event(event: &NeedleEvent, event_json: &str) -> Self {
        let (event_type, bead_id, adapter) = match event {
            NeedleEvent::Claim { bead, .. } => ("claim", bead.clone(), None),
            NeedleEvent::Dispatch { bead, adapter, .. } => ("dispatch", bead.clone(), adapter.clone()),
            NeedleEvent::Complete { bead, .. } => ("complete", bead.clone(), None),
            NeedleEvent::Fail { bead, .. } => ("fail", bead.clone(), None),
            NeedleEvent::Timeout { bead, .. } => ("timeout", bead.clone(), None),
            NeedleEvent::Crash { bead, .. } => ("crash", bead.clone(), None),
            NeedleEvent::Close { bead, .. } => ("close", bead.clone(), None),
            NeedleEvent::Release { bead, .. } => ("release", bead.clone(), None),
            NeedleEvent::Update { bead, .. } => ("update", bead.clone(), None),
            NeedleEvent::Unknown => ("unknown", String::new(), None),
        };

        // Determine result based on event type
        let result = match event {
            NeedleEvent::Complete { .. } => Some("success".to_string()),
            NeedleEvent::Fail { .. } | NeedleEvent::Timeout { .. } | NeedleEvent::Crash { .. } => {
                Some("failure".to_string())
            }
            _ => None,
        };

        Self {
            event_type: event_type.to_string(),
            project: None, // Will be filled in by caller
            kind: None,    // Will be filled in by caller
            adapter,
            result,
            event_json: event_json.to_string(),
            bead_id,
        }
    }
}

/// Check if a glob pattern matches a value
fn glob_match(pattern: &str, value: &str) -> bool {
    match Pattern::new(pattern) {
        Ok(p) => p.matches(value),
        Err(_) => {
            // If pattern is invalid, treat as exact match
            pattern == value
        }
    }
}

/// Check if an event subscription matches an event context
pub fn matches_subscription(sub: &EventSubscription, ctx: &EventContext) -> bool {
    // Match event type (glob)
    if !glob_match(&sub.event, &ctx.event_type) {
        return false;
    }

    // Match project filter (glob)
    if let Some(ref project_filter) = sub.project {
        match &ctx.project {
            Some(project_name) => {
                if !glob_match(project_filter, project_name) {
                    return false;
                }
            }
            None => return false, // Subscription requires project but we don't have one
        }
    }

    // Match kind filter (exact match)
    if let Some(ref kind_filter) = sub.kind {
        match &ctx.kind {
            Some(kind) => {
                if kind != kind_filter {
                    return false;
                }
            }
            None => return false, // Subscription requires kind but we don't have one
        }
    }

    // Match adapter filter (exact match)
    if let Some(ref adapter_filter) = sub.adapter {
        match &ctx.adapter {
            Some(adapter) => {
                if adapter != adapter_filter {
                    return false;
                }
            }
            None => return false, // Subscription requires adapter but we don't have one
        }
    }

    // Match result filter (exact match: "success" or "failure")
    if let Some(ref result_filter) = sub.result {
        match &ctx.result {
            Some(result) => {
                if result != result_filter {
                    return false;
                }
            }
            None => return false, // Subscription requires result but we don't have one
        }
    }

    true
}

/// Result of a triggered script execution
#[derive(Debug, Clone)]
pub struct ScriptTriggerResult {
    /// Script name
    pub script_name: String,
    /// Whether execution was attempted
    pub attempted: bool,
    /// Whether execution succeeded
    pub succeeded: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Find and trigger all scripts that match an event
///
/// All matching scripts execute in parallel. Failures are logged but don't block
/// other scripts from running.
pub async fn trigger_matching_scripts(
    scripts_dir: &Path,
    ctx: &EventContext,
) -> Vec<ScriptTriggerResult> {
    let scripts = discover_scripts(scripts_dir);

    // Find all matching scripts first
    let matching_scripts: Vec<_> = scripts
        .into_iter()
        .filter(|script_entry| {
            // Skip if not executable
            if !script_entry.executable {
                debug!(
                    "Skipping script '{}': not executable",
                    script_entry.name
                );
                return false;
            }

            // Get manifest
            let manifest = match &script_entry.manifest {
                Some(m) => m,
                None => {
                    // No manifest means no event subscriptions
                    return false;
                }
            };

            // Check if any subscription matches
            manifest.on.iter().any(|sub| matches_subscription(sub, ctx))
        })
        .collect();

    if matching_scripts.is_empty() {
        return Vec::new();
    }

    // Spawn all scripts in parallel
    let mut handles = Vec::new();
    for script_entry in matching_scripts {
        let ctx = ctx.clone();
        let handle = tokio::spawn(async move {
            info!(
                "Event '{}' matched script '{}', executing with event JSON on stdin",
                ctx.event_type, script_entry.name
            );
            trigger_script(&script_entry, &ctx).await
        });
        handles.push(handle);
    }

    // Wait for all scripts to complete and collect results
    let mut results = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(result) => results.push(result),
            Err(e) => {
                // Task panicked or was cancelled
                warn!("Script trigger task failed: {}", e);
            }
        }
    }

    results
}

/// Execute a single script with event JSON on stdin
async fn trigger_script(
    script_entry: &ScriptEntry,
    ctx: &EventContext,
) -> ScriptTriggerResult {
    use tokio::process::Command;
    use tokio::time::{timeout, Duration};

    let script_name = script_entry.name.clone();
    let script_path = &script_entry.path;

    // Get timeout from manifest or use default
    let timeout_secs = script_entry
        .manifest
        .as_ref()
        .map(|m| m.timeout_secs)
        .unwrap_or(300);

    // Spawn the script with event JSON on stdin
    let mut child = match Command::new(script_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            let error = format!("Failed to spawn script '{}': {}", script_name, e);
            warn!("{}", error);
            return ScriptTriggerResult {
                script_name,
                attempted: true,
                succeeded: false,
                error: Some(error),
            };
        }
    };

    // Write event JSON to stdin
    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        if let Err(e) = stdin.write_all(ctx.event_json.as_bytes()).await {
            warn!("Failed to write to stdin of '{}': {}", script_name, e);
        }
        let _ = stdin.shutdown().await;
    }

    // Wait for completion with timeout
    let duration = Duration::from_secs(timeout_secs);
    match timeout(duration, child.wait()).await {
        Ok(Ok(status)) => {
            let succeeded = status.success();
            if !succeeded {
                let error = format!(
                    "Script '{}' exited with status: {}",
                    script_name, status
                );
                warn!("{}", error);
                ScriptTriggerResult {
                    script_name,
                    attempted: true,
                    succeeded: false,
                    error: Some(error),
                }
            } else {
                info!("Script '{}' completed successfully", script_name);
                ScriptTriggerResult {
                    script_name,
                    attempted: true,
                    succeeded: true,
                    error: None,
                }
            }
        }
        Ok(Err(e)) => {
            let error = format!("Failed to wait for script '{}': {}", script_name, e);
            warn!("{}", error);
            ScriptTriggerResult {
                script_name,
                attempted: true,
                succeeded: false,
                error: Some(error),
            }
        }
        Err(_) => {
            // Timeout - kill the process
            let _ = child.kill().await;
            let error = format!(
                "Script '{}' timed out after {} seconds",
                script_name, timeout_secs
            );
            warn!("{}", error);
            ScriptTriggerResult {
                script_name,
                attempted: true,
                succeeded: false,
                error: Some(error),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match() {
        assert!(glob_match("claim", "claim"));
        assert!(glob_match("stitch.*", "stitch.archived"));
        assert!(glob_match("bead.*", "bead.closed"));
        assert!(glob_match("*", "anything"));
        assert!(!glob_match("claim", "dispatch"));
    }

    #[test]
    fn test_matches_subscription_event_type() {
        let sub = EventSubscription {
            event: "claim".to_string(),
            project: None,
            kind: None,
            adapter: None,
            result: None,
        };

        let ctx = EventContext {
            event_type: "claim".to_string(),
            project: None,
            kind: None,
            adapter: None,
            result: None,
            event_json: String::new(),
            bead_id: "bd-123".to_string(),
        };

        assert!(matches_subscription(&sub, &ctx));
    }

    #[test]
    fn test_matches_subscription_glob_event_type() {
        let sub = EventSubscription {
            event: "stitch.*".to_string(),
            project: None,
            kind: None,
            adapter: None,
            result: None,
        };

        let ctx = EventContext {
            event_type: "stitch.archived".to_string(),
            project: None,
            kind: None,
            adapter: None,
            result: None,
            event_json: String::new(),
            bead_id: "bd-123".to_string(),
        };

        assert!(matches_subscription(&sub, &ctx));
    }

    #[test]
    fn test_matches_subscription_with_project() {
        let sub = EventSubscription {
            event: "*".to_string(),
            project: Some("my-project".to_string()),
            kind: None,
            adapter: None,
            result: None,
        };

        let ctx = EventContext {
            event_type: "claim".to_string(),
            project: Some("my-project".to_string()),
            kind: None,
            adapter: None,
            result: None,
            event_json: String::new(),
            bead_id: "bd-123".to_string(),
        };

        assert!(matches_subscription(&sub, &ctx));
    }

    #[test]
    fn test_matches_subscription_project_mismatch() {
        let sub = EventSubscription {
            event: "*".to_string(),
            project: Some("my-project".to_string()),
            kind: None,
            adapter: None,
            result: None,
        };

        let ctx = EventContext {
            event_type: "claim".to_string(),
            project: Some("other-project".to_string()),
            kind: None,
            adapter: None,
            result: None,
            event_json: String::new(),
            bead_id: "bd-123".to_string(),
        };

        assert!(!matches_subscription(&sub, &ctx));
    }

    #[test]
    fn test_matches_subscription_with_adapter() {
        let sub = EventSubscription {
            event: "*".to_string(),
            project: None,
            kind: None,
            adapter: Some("claude".to_string()),
            result: None,
        };

        let ctx = EventContext {
            event_type: "dispatch".to_string(),
            project: None,
            kind: None,
            adapter: Some("claude".to_string()),
            result: None,
            event_json: String::new(),
            bead_id: "bd-123".to_string(),
        };

        assert!(matches_subscription(&sub, &ctx));
    }

    #[test]
    fn test_matches_subscription_with_result() {
        let sub = EventSubscription {
            event: "*".to_string(),
            project: None,
            kind: None,
            adapter: None,
            result: Some("success".to_string()),
        };

        let ctx = EventContext {
            event_type: "complete".to_string(),
            project: None,
            kind: None,
            adapter: None,
            result: Some("success".to_string()),
            event_json: String::new(),
            bead_id: "bd-123".to_string(),
        };

        assert!(matches_subscription(&sub, &ctx));
    }

    #[test]
    fn test_event_context_from_claim_event() {
        let event_json = r#"{"event":"claim","ts":"2026-04-21T18:42:10Z","worker":"alpha","bead":"bd-abc123"}"#;
        let event: NeedleEvent = serde_json::from_str(event_json).unwrap();

        let ctx = EventContext::from_event(&event, event_json);

        assert_eq!(ctx.event_type, "claim");
        assert_eq!(ctx.bead_id, "bd-abc123");
        assert_eq!(ctx.adapter, None);
        assert_eq!(ctx.result, None);
    }

    #[test]
    fn test_event_context_from_dispatch_event() {
        let event_json = r#"{"event":"dispatch","ts":"2026-04-21T18:42:10Z","worker":"alpha","bead":"bd-abc123","adapter":"claude","model":"sonnet"}"#;
        let event: NeedleEvent = serde_json::from_str(event_json).unwrap();

        let ctx = EventContext::from_event(&event, event_json);

        assert_eq!(ctx.event_type, "dispatch");
        assert_eq!(ctx.bead_id, "bd-abc123");
        assert_eq!(ctx.adapter, Some("claude".to_string()));
        assert_eq!(ctx.result, None);
    }

    #[test]
    fn test_event_context_from_complete_event() {
        let event_json = r#"{"event":"complete","ts":"2026-04-21T18:42:10Z","worker":"alpha","bead":"bd-abc123","outcome":"success","duration_ms":5000,"exit_code":0}"#;
        let event: NeedleEvent = serde_json::from_str(event_json).unwrap();

        let ctx = EventContext::from_event(&event, event_json);

        assert_eq!(ctx.event_type, "complete");
        assert_eq!(ctx.bead_id, "bd-abc123");
        assert_eq!(ctx.adapter, None);
        assert_eq!(ctx.result, Some("success".to_string()));
    }

    #[test]
    fn test_event_context_from_fail_event() {
        let event_json = r#"{"event":"fail","ts":"2026-04-21T18:42:10Z","worker":"alpha","bead":"bd-abc123","error":"task failed","duration_ms":3000}"#;
        let event: NeedleEvent = serde_json::from_str(event_json).unwrap();

        let ctx = EventContext::from_event(&event, event_json);

        assert_eq!(ctx.event_type, "fail");
        assert_eq!(ctx.bead_id, "bd-abc123");
        assert_eq!(ctx.adapter, None);
        assert_eq!(ctx.result, Some("failure".to_string()));
    }
}
