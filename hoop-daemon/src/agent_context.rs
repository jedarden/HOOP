//! Agent context: lazy-fetch index builder and budget watchdog
//!
//! Principle §3.12: Eager context overflows windows on long sessions + costs money
//! for data that might never be relevant. This module builds a thin index (~4KB token
//! budget) containing only project names, recent activity summary, open Stitch titles,
//! and active alerts. Full bead bodies, file contents, and conversation transcripts
//! are fetched on demand via MCP tools.
//!
//! ## Architecture
//!
//! - `ContextIndex`: Thin index injected into system prompt (<4KB tokens)
//! - `ContextBudget`: Watchdog that emits warning at 75% window usage
//! - `build_context_index()`: Queries fleet.db and projects.yaml for index data
//!
//! ## Acceptance criteria
//!
//! - System prompt <4KB token budget
//! - All detail retrievable via tools (MCP server tools)
//! - Context-budget watchdog emits warning at 75% window usage
//! - Test: agent successfully answers "what happened today?" using only tool calls

use anyhow::Result;
use chrono::Utc;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tracing::warn;

/// Number of recent closed stitches to include in activity summary
const RECENT_STITCH_COUNT: usize = 10;

/// Maximum tokens for the system prompt index (4KB budget)
const MAX_SYSTEM_PROMPT_TOKENS: usize = 4000;

/// Threshold percentage for context budget warning (75%)
const CONTEXT_BUDGET_WARNING_THRESHOLD: f64 = 0.75;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Thin index injected into agent's system prompt.
///
/// Contains only project names, recent activity summary (last N closed Stitches
/// with titles), open Stitch titles, and active alerts. Full details are fetched
/// via MCP tools on demand.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextIndex {
    /// Current timestamp for the index
    pub generated_at: String,
    /// Project names and their paths
    pub projects: Vec<ProjectEntry>,
    /// Recent activity summary (last N closed Stitches)
    pub recent_activity: RecentActivitySummary,
    /// Open Stitch titles only (no full bodies)
    pub open_stitches: Vec<OpenStitchEntry>,
    /// Active alerts (system warnings, errors)
    pub alerts: Vec<AlertEntry>,
}

/// Project entry in the index (name only, not full path)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectEntry {
    pub name: String,
    pub label: Option<String>,
}

/// Recent activity summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentActivitySummary {
    /// Last N closed Stitches with titles only
    pub closed_stitches: Vec<ClosedStitchEntry>,
    /// Total stitches closed today
    pub closed_today: usize,
    /// Total stitches created today
    pub created_today: usize,
}

/// Closed Stitch entry (title only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosedStitchEntry {
    pub id: String,
    pub project: String,
    pub title: String,
    pub closed_at: String,
}

/// Open Stitch entry (title only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenStitchEntry {
    pub id: String,
    pub project: String,
    pub title: String,
    pub kind: String,
    pub last_activity_at: String,
}

/// Active alert entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEntry {
    pub level: AlertLevel,
    pub message: String,
    pub project: Option<String>,
}

/// Alert severity level
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertLevel {
    Warning,
    Error,
    Info,
}

/// Context budget watchdog
///
/// Tracks token usage and emits warnings when approaching the context window limit.
#[derive(Debug, Clone)]
pub struct ContextBudget {
    /// Maximum context window size (tokens)
    pub max_tokens: usize,
    /// Current token usage
    pub current_tokens: usize,
    /// Whether warning has been emitted
    pub warning_emitted: bool,
}

impl Default for ContextBudget {
    fn default() -> Self {
        Self {
            max_tokens: 200_000, // Claude Opus 4.7 default
            current_tokens: 0,
            warning_emitted: false,
        }
    }
}

// ---------------------------------------------------------------------------
// ContextIndex builder
// ---------------------------------------------------------------------------

impl ContextIndex {
    /// Build the thin context index from fleet.db and projects.yaml
    pub fn build(projects_config: &serde_yaml::Value) -> Result<Self> {
        let conn = Connection::open(crate::fleet::db_path())?;

        // Load projects from config
        let projects = Self::load_projects(projects_config);

        // Query recent closed stitches
        let recent_activity = Self::query_recent_activity(&conn)?;

        // Query open stitches (titles only)
        let open_stitches = Self::query_open_stitches(&conn)?;

        // Query active alerts
        let alerts = Self::query_alerts(&conn)?;

        Ok(Self {
            generated_at: Utc::now().to_rfc3339(),
            projects,
            recent_activity,
            open_stitches,
            alerts,
        })
    }

    /// Build a minimal context index for testing (no database required)
    #[cfg(test)]
    fn build_for_test(projects_config: &serde_yaml::Value) -> Self {
        let projects = Self::load_projects(projects_config);

        Self {
            generated_at: Utc::now().to_rfc3339(),
            projects,
            recent_activity: RecentActivitySummary {
                closed_stitches: vec![],
                closed_today: 0,
                created_today: 0,
            },
            open_stitches: vec![],
            alerts: vec![],
        }
    }

    /// Convert to system prompt string (thin index only)
    ///
    /// This format is optimized for <4KB token budget. Full details must be
    /// fetched via MCP tools: summarize_day, summarize_project, find_stitches,
    /// read_stitch, find_beads, read_bead.
    pub fn to_system_prompt(&self) -> String {
        let mut prompt = String::from("# HOOP Context Index\n\n");
        prompt.push_str("Generated at: ");
        prompt.push_str(&self.generated_at);
        prompt.push_str("\n\n");

        // Projects
        prompt.push_str("## Projects\n");
        for project in &self.projects {
            if let Some(ref label) = project.label {
                prompt.push_str(&format!("- {} ({})\n", project.name, label));
            } else {
                prompt.push_str(&format!("- {}\n", project.name));
            }
        }
        prompt.push('\n');

        // Recent activity
        prompt.push_str("## Recent Activity (Last 10 closed Stitches)\n");
        for stitch in &self.recent_activity.closed_stitches {
            prompt.push_str(&format!("- [{}] {} - {}\n",
                stitch.closed_at.split('T').next().unwrap_or(&stitch.closed_at),
                stitch.project,
                stitch.title));
        }
        prompt.push_str(&format!("Closed today: {}\n", self.recent_activity.closed_today));
        prompt.push_str(&format!("Created today: {}\n", self.recent_activity.created_today));
        prompt.push('\n');

        // Open Stitches (titles only)
        prompt.push_str("## Open Stitches\n");
        if self.open_stitches.is_empty() {
            prompt.push_str("(No open stitches)\n");
        } else {
            for stitch in &self.open_stitches {
                prompt.push_str(&format!("- [{}] {} - {}\n",
                    stitch.kind,
                    stitch.project,
                    stitch.title));
            }
        }
        prompt.push('\n');

        // Alerts
        if !self.alerts.is_empty() {
            prompt.push_str("## Alerts\n");
            for alert in &self.alerts {
                let level_str = match alert.level {
                    AlertLevel::Warning => "WARNING",
                    AlertLevel::Error => "ERROR",
                    AlertLevel::Info => "INFO",
                };
                if let Some(ref project) = alert.project {
                    prompt.push_str(&format!("- [{}] {}: {}\n", level_str, project, alert.message));
                } else {
                    prompt.push_str(&format!("- [{}]: {}\n", level_str, alert.message));
                }
            }
            prompt.push('\n');
        }

        // Tools instruction
        prompt.push_str("## Accessing Full Details\n\n");
        prompt.push_str("This index contains summaries only. Use MCP tools to fetch full details:\n");
        prompt.push_str("- summarize_day() — daily summary across all projects\n");
        prompt.push_str("- summarize_project(project=\"<name>\") — project-specific summary\n");
        prompt.push_str("- find_stitches(project=\"<name>\") — list stitches with filters\n");
        prompt.push_str("- read_stitch(id=\"<stitch-id>\") — full stitch details with messages\n");
        prompt.push_str("- find_beads(project=\"<name>\") — list beads with filters\n");
        prompt.push_str("- read_bead(project=\"<name>\", id=\"<bead-id>\") — full bead details\n");
        prompt.push_str("- read_file(project=\"<name>\", path=\"<file>\") — file contents\n");
        prompt.push_str("- grep(project=\"<name>\", pattern=\"<regex>\") — search files\n");
        prompt.push_str("- search_conversations(query=\"<text>\") — search transcripts\n");

        prompt
    }

    /// Estimate token count of the system prompt (rough approximation)
    pub fn estimate_token_count(&self) -> usize {
        let prompt = self.to_system_prompt();
        // Rough approximation: ~4 chars per token for English text
        prompt.len() / 4
    }

    /// Load projects from YAML config
    fn load_projects(config: &serde_yaml::Value) -> Vec<ProjectEntry> {
        let mut projects = Vec::new();

        if let Some(project_list) = config.get("projects").and_then(|p| p.as_sequence()) {
            for project in project_list {
                if let Some(name) = project.get("name").and_then(|n| n.as_str()) {
                    let label = project.get("label").and_then(|l| l.as_str()).map(|s| s.to_string());
                    projects.push(ProjectEntry {
                        name: name.to_string(),
                        label,
                    });
                }
            }
        }

        projects
    }

    /// Query recent activity from fleet.db
    fn query_recent_activity(conn: &Connection) -> Result<RecentActivitySummary> {
        // Query last N closed stitches (by last_activity_at descending)
        let mut stmt = conn.prepare(
            "SELECT id, project, title, last_activity_at
             FROM stitches
             WHERE last_activity_at < datetime('now', '-1 hour')
             ORDER BY last_activity_at DESC
             LIMIT ?1"
        )?;

        let closed_stitches: Result<Vec<ClosedStitchEntry>, _> = stmt
            .query_map([RECENT_STITCH_COUNT as i64], |row| {
                Ok(ClosedStitchEntry {
                    id: row.get(0)?,
                    project: row.get(1)?,
                    title: row.get(2)?,
                    closed_at: row.get(3)?,
                })
            })?
            .collect();

        let closed_stitches = closed_stitches.unwrap_or_default();

        // Count stitches closed today (last_activity_at before today start)
        let closed_today: usize = conn.query_row(
            "SELECT COUNT(*) FROM stitches WHERE last_activity_at < datetime('now', 'start of day')",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        // Count stitches created today
        let created_today: usize = conn.query_row(
            "SELECT COUNT(*) FROM stitches WHERE created_at >= datetime('now', 'start of day')",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        Ok(RecentActivitySummary {
            closed_stitches,
            closed_today,
            created_today,
        })
    }

    /// Query open stitches from fleet.db
    fn query_open_stitches(conn: &Connection) -> Result<Vec<OpenStitchEntry>> {
        let mut stmt = conn.prepare(
            "SELECT id, project, kind, title, last_activity_at
             FROM stitches
             WHERE last_activity_at >= datetime('now', '-1 hour')
             ORDER BY last_activity_at DESC"
        )?;

        let stitches: Result<Vec<OpenStitchEntry>, _> = stmt
            .query_map([], |row| {
                Ok(OpenStitchEntry {
                    id: row.get(0)?,
                    project: row.get(1)?,
                    title: row.get(3)?,
                    kind: row.get(2)?,
                    last_activity_at: row.get(4)?,
                })
            })?
            .collect();

        Ok(stitches.unwrap_or_default())
    }

    /// Query active alerts from fleet.db
    fn query_alerts(conn: &Connection) -> Result<Vec<AlertEntry>> {
        let mut alerts = Vec::new();

        // Check for recent audit failures
        let recent_failures: Result<Vec<(String, String)>, _> = conn
            .prepare(
                "SELECT project, error FROM actions
                 WHERE result = '\"Failure\"'
                 AND ts > datetime('now', '-24 hours')
                 LIMIT 5"
            )?
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .collect();

        if let Ok(failures) = recent_failures {
            for (project, error) in failures {
                if !project.is_empty() {
                    alerts.push(AlertEntry {
                        level: AlertLevel::Error,
                        message: error,
                        project: Some(project),
                    });
                }
            }
        }

        Ok(alerts)
    }
}

// ---------------------------------------------------------------------------
// ContextBudget watchdog
// ---------------------------------------------------------------------------

impl ContextBudget {
    /// Create a new budget watchdog with custom max tokens
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            current_tokens: 0,
            warning_emitted: false,
        }
    }

    /// Update current token usage and check if warning should be emitted
    ///
    /// Returns true if warning was emitted this call
    pub fn check_usage(&mut self, current_tokens: usize) -> bool {
        self.current_tokens = current_tokens;
        let usage_ratio = current_tokens as f64 / self.max_tokens as f64;

        if usage_ratio >= CONTEXT_BUDGET_WARNING_THRESHOLD && !self.warning_emitted {
            self.warning_emitted = true;
            warn!(
                "Context budget at {:.0}% usage ({} / {} tokens). Approaching window limit.",
                usage_ratio * 100.0,
                current_tokens,
                self.max_tokens
            );
            true
        } else {
            false
        }
    }

    /// Reset the warning state (call on new session)
    pub fn reset(&mut self) {
        self.current_tokens = 0;
        self.warning_emitted = false;
    }

    /// Get current usage ratio (0.0 to 1.0)
    pub fn usage_ratio(&self) -> f64 {
        self.current_tokens as f64 / self.max_tokens as f64
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Build the thin context index from fleet.db and projects config
pub fn build_context_index(projects_config: &serde_yaml::Value) -> Result<ContextIndex> {
    let index = ContextIndex::build(projects_config)?;

    // Verify token budget is under limit
    let estimated_tokens = index.estimate_token_count();
    if estimated_tokens > MAX_SYSTEM_PROMPT_TOKENS {
        warn!(
            "Context index token count ({}) exceeds budget ({}). Consider reducing recent stitch count.",
            estimated_tokens, MAX_SYSTEM_PROMPT_TOKENS
        );
    }

    Ok(index)
}

/// Load projects.yaml config from ~/.hoop/projects.yaml
pub fn load_projects_config() -> Result<serde_yaml::Value> {
    let mut path = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?;
    path.push(".hoop");
    path.push("projects.yaml");

    let contents = std::fs::read_to_string(&path)?;
    Ok(serde_yaml::from_str(&contents)?)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_index_empty_projects() {
        let yaml = serde_yaml::from_str("projects: []").unwrap();
        let index = ContextIndex::build_for_test(&yaml);

        assert!(index.projects.is_empty());
        assert!(index.alerts.is_empty());
    }

    #[test]
    fn test_context_index_with_projects() {
        let yaml = serde_yaml::from_str(
            r#"
projects:
  - name: test-project
    label: "Test Project"
  - name: another-project
"#
        ).unwrap();

        let index = ContextIndex::build_for_test(&yaml);

        assert_eq!(index.projects.len(), 2);
        assert_eq!(index.projects[0].name, "test-project");
        assert_eq!(index.projects[0].label, Some("Test Project".to_string()));
    }

    #[test]
    fn test_system_prompt_generation() {
        let yaml = serde_yaml::from_str(
            r#"
projects:
  - name: test-project
    label: "Test"
"#
        ).unwrap();

        let index = ContextIndex::build_for_test(&yaml);
        let prompt = index.to_system_prompt();

        assert!(prompt.contains("# HOOP Context Index"));
        assert!(prompt.contains("test-project"));
        assert!(prompt.contains("summarize_day"));
        assert!(prompt.contains("read_stitch"));
    }

    #[test]
    fn test_context_budget_warning_threshold() {
        let mut budget = ContextBudget::new(1000);

        // Below threshold - no warning
        assert!(!budget.check_usage(500));
        assert!(!budget.warning_emitted);

        // At threshold - warning emitted
        assert!(budget.check_usage(760));
        assert!(budget.warning_emitted);

        // Above threshold - already warned, no duplicate warning
        assert!(!budget.check_usage(900));
    }

    #[test]
    fn test_context_budget_reset() {
        let mut budget = ContextBudget::new(1000);

        budget.check_usage(800);
        assert!(budget.warning_emitted);

        budget.reset();
        assert!(!budget.warning_emitted);
        assert_eq!(budget.current_tokens, 0);
    }

    #[test]
    fn test_context_budget_usage_ratio() {
        let mut budget = ContextBudget::new(1000);
        budget.current_tokens = 250;

        assert!((budget.usage_ratio() - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_alert_level_serialization() {
        let alert = AlertEntry {
            level: AlertLevel::Warning,
            message: "Test warning".to_string(),
            project: Some("test-project".to_string()),
        };

        let json = serde_json::to_string(&alert).unwrap();
        assert!(json.contains("\"warning\""));
        assert!(json.contains("Test warning"));
    }

    #[test]
    fn test_estimate_token_count() {
        let yaml = serde_yaml::from_str("projects: []").unwrap();
        let index = ContextIndex::build_for_test(&yaml);

        let count = index.estimate_token_count();
        // Even empty index has some overhead
        assert!(count > 0);
        assert!(count < 1000); // Should be well under 4KB
    }
}
