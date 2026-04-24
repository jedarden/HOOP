//! Morning Brief: autonomous daily briefing generator (marquee #10, Phase 5)
//!
//! Queries overnight activity from fleet.db, builds a structured prompt,
//! sends it to the agent session as a turn, collects the streaming response,
//! parses draft Stitch blocks from the response, inserts them into the
//! draft_queue (preview flow — no auto-submits), and stores the brief.
//!
//! Trigger: HTTP POST /api/agent/morning-brief/trigger (manual) or the
//! background scheduler (configurable hour, default 07:00 local time).
//!
//! Plan reference: §6 Phase 5 marquee #10

use anyhow::Result;
use chrono::{DateTime, Duration, Timelike, Utc};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tracing::{info, warn};
use uuid::Uuid;

use crate::agent_session::AgentSessionManager;
use crate::fleet;
use crate::ws::MorningBriefData;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Config for the morning brief scheduler (loaded from ~/.hoop/config.yml).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorningBriefConfig {
    /// How many hours back to look for overnight activity (default: 24)
    #[serde(default = "default_window_hours")]
    pub window_hours: u64,
    /// Hour of day (0–23) to auto-run the brief (default: 7)
    #[serde(default = "default_schedule_hour")]
    pub schedule_hour: u32,
    /// Whether the scheduled auto-run is enabled (default: true)
    #[serde(default = "default_auto_run")]
    pub auto_run_enabled: bool,
}

fn default_window_hours() -> u64 {
    24
}
fn default_schedule_hour() -> u32 {
    7
}
fn default_auto_run() -> bool {
    true
}

impl Default for MorningBriefConfig {
    fn default() -> Self {
        Self {
            window_hours: 24,
            schedule_hour: 7,
            auto_run_enabled: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Overnight activity
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct OvernightActivity {
    window_from: DateTime<Utc>,
    window_to: DateTime<Utc>,
    closed_stitches: Vec<StitchSummary>,
    open_stitches: Vec<StitchSummary>,
    new_stitches: Vec<StitchSummary>,
    failed_actions: Vec<ActionSummary>,
    stuck_stitches: Vec<StitchSummary>,
    pending_drafts: Vec<DraftSummary>,
    project_names: Vec<String>,
}

#[derive(Debug)]
struct ActionSummary {
    kind: String,
    target: String,
    project: Option<String>,
    error: Option<String>,
    ts: String,
}

#[derive(Debug)]
struct DraftSummary {
    title: String,
    project: String,
    status: String,
    created_at: String,
}

#[derive(Debug)]
struct StitchSummary {
    title: String,
    project: String,
    kind: String,
    last_activity_at: String,
}

fn query_overnight_activity(window_hours: u64) -> Result<OvernightActivity> {
    let path = fleet::db_path();
    let conn = rusqlite::Connection::open(&path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;

    let window_to = Utc::now();
    let window_from = window_to - Duration::hours(window_hours as i64);
    let from_str = window_from.to_rfc3339();
    let to_str = window_to.to_rfc3339();

    // Stitches closed in the window (last_activity_at within window AND older than 1h)
    let mut stmt = conn.prepare(
        "SELECT title, project, kind, last_activity_at FROM stitches
         WHERE last_activity_at >= ?1 AND last_activity_at <= ?2
           AND last_activity_at < datetime('now', '-1 hour')
         ORDER BY last_activity_at DESC LIMIT 50",
    )?;
    let closed_stitches: Vec<StitchSummary> = stmt
        .query_map(rusqlite::params![from_str, to_str], |row| {
            Ok(StitchSummary {
                title: row.get(0)?,
                project: row.get(1)?,
                kind: row.get(2)?,
                last_activity_at: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Open stitches (active in the last hour)
    let mut stmt = conn.prepare(
        "SELECT title, project, kind, last_activity_at FROM stitches
         WHERE last_activity_at >= datetime('now', '-1 hour')
         ORDER BY last_activity_at DESC LIMIT 30",
    )?;
    let open_stitches: Vec<StitchSummary> = stmt
        .query_map(rusqlite::params![], |row| {
            Ok(StitchSummary {
                title: row.get(0)?,
                project: row.get(1)?,
                kind: row.get(2)?,
                last_activity_at: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Newly created stitches in the window
    let mut stmt = conn.prepare(
        "SELECT title, project, kind, created_at FROM stitches
         WHERE created_at >= ?1 AND created_at <= ?2
         ORDER BY created_at DESC LIMIT 30",
    )?;
    let new_stitches: Vec<StitchSummary> = stmt
        .query_map(rusqlite::params![from_str, to_str], |row| {
            Ok(StitchSummary {
                title: row.get(0)?,
                project: row.get(1)?,
                kind: row.get(2)?,
                last_activity_at: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Failed audit actions in the window
    let mut stmt = conn.prepare(
        "SELECT kind, target, project, error, ts FROM actions
         WHERE ts >= ?1 AND ts <= ?2 AND result = '\"Failure\"'
         ORDER BY ts DESC LIMIT 30",
    )?;
    let failed_actions: Vec<ActionSummary> = stmt
        .query_map(rusqlite::params![from_str, to_str], |row| {
            Ok(ActionSummary {
                kind: row.get(0)?,
                target: row.get(1)?,
                project: row.get(2)?,
                error: row.get(3)?,
                ts: row.get(4)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Stuck stitches (open but no activity for > 12h)
    let mut stmt = conn.prepare(
        "SELECT title, project, kind, last_activity_at FROM stitches
         WHERE last_activity_at < datetime('now', '-12 hours')
         ORDER BY last_activity_at ASC LIMIT 20",
    )?;
    let stuck_stitches: Vec<StitchSummary> = stmt
        .query_map(rusqlite::params![], |row| {
            Ok(StitchSummary {
                title: row.get(0)?,
                project: row.get(1)?,
                kind: row.get(2)?,
                last_activity_at: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Pending drafts (blocked-on-human — awaiting operator review)
    let mut stmt = conn.prepare(
        "SELECT title, project, status, created_at FROM draft_queue
         WHERE status IN ('pending', 'edited')
         ORDER BY created_at ASC LIMIT 20",
    )?;
    let pending_drafts: Vec<DraftSummary> = stmt
        .query_map(rusqlite::params![], |row| {
            Ok(DraftSummary {
                title: row.get(0)?,
                project: row.get(1)?,
                status: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Distinct projects
    let mut stmt = conn.prepare(
        "SELECT DISTINCT project FROM stitches
         WHERE last_activity_at >= ?1
         ORDER BY project",
    )?;
    let project_names: Vec<String> = stmt
        .query_map(rusqlite::params![from_str], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(OvernightActivity {
        window_from,
        window_to,
        closed_stitches,
        open_stitches,
        new_stitches,
        failed_actions,
        stuck_stitches,
        pending_drafts,
        project_names,
    })
}

// ---------------------------------------------------------------------------
// Prompt builder
// ---------------------------------------------------------------------------

fn build_brief_prompt(activity: &OvernightActivity) -> String {
    let from = activity.window_from.format("%Y-%m-%d %H:%M UTC").to_string();
    let to = activity.window_to.format("%Y-%m-%d %H:%M UTC").to_string();
    let date = activity.window_to.format("%Y-%m-%d").to_string();

    let mut prompt = format!(
        "## Morning Brief Request\n\n\
         Activity window: {from} → {to}\n\n"
    );

    // Projects
    if activity.project_names.is_empty() {
        prompt.push_str("### Projects monitored\n(none active in window)\n\n");
    } else {
        prompt.push_str("### Projects monitored\n");
        for p in &activity.project_names {
            prompt.push_str(&format!("- {p}\n"));
        }
        prompt.push('\n');
    }

    // Closed stitches
    prompt.push_str(&format!(
        "### Stitches settled in this window ({})\n",
        activity.closed_stitches.len()
    ));
    if activity.closed_stitches.is_empty() {
        prompt.push_str("(none)\n\n");
    } else {
        for s in &activity.closed_stitches {
            let ts = s.last_activity_at.split('T').next().unwrap_or(&s.last_activity_at);
            prompt.push_str(&format!("- [{}] [{}] {}\n", ts, s.project, s.title));
        }
        prompt.push('\n');
    }

    // Open stitches
    prompt.push_str(&format!(
        "### Stitches still in flight ({})\n",
        activity.open_stitches.len()
    ));
    if activity.open_stitches.is_empty() {
        prompt.push_str("(none)\n\n");
    } else {
        for s in &activity.open_stitches {
            let ts = s.last_activity_at.split('T').next().unwrap_or(&s.last_activity_at);
            prompt.push_str(&format!("- [{}] [{}] {} (kind: {})\n", ts, s.project, s.title, s.kind));
        }
        prompt.push('\n');
    }

    // New stitches
    prompt.push_str(&format!(
        "### New stitches created in window ({})\n",
        activity.new_stitches.len()
    ));
    if activity.new_stitches.is_empty() {
        prompt.push_str("(none)\n\n");
    } else {
        for s in &activity.new_stitches {
            let ts = s.last_activity_at.split('T').next().unwrap_or(&s.last_activity_at);
            prompt.push_str(&format!("- [{}] [{}] {}\n", ts, s.project, s.title));
        }
        prompt.push('\n');
    }

    // Failed actions
    prompt.push_str(&format!(
        "### Failures in this window ({})\n",
        activity.failed_actions.len()
    ));
    if activity.failed_actions.is_empty() {
        prompt.push_str("(none)\n\n");
    } else {
        for a in &activity.failed_actions {
            let ts = a.ts.split('T').next().unwrap_or(&a.ts);
            let proj = a.project.as_deref().unwrap_or("—");
            let err = a.error.as_deref().unwrap_or("no detail");
            prompt.push_str(&format!(
                "- [{}] [{}] {} on {} ({})\n",
                ts, proj, a.kind, a.target, err
            ));
        }
        prompt.push('\n');
    }

    // Stuck stitches
    prompt.push_str(&format!(
        "### Stuck stitches (no activity > 12h) ({})\n",
        activity.stuck_stitches.len()
    ));
    if activity.stuck_stitches.is_empty() {
        prompt.push_str("(none)\n\n");
    } else {
        for s in &activity.stuck_stitches {
            let ts = s.last_activity_at.split('T').next().unwrap_or(&s.last_activity_at);
            prompt.push_str(&format!("- [{}] [{}] {} (kind: {})\n", ts, s.project, s.title, s.kind));
        }
        prompt.push('\n');
    }

    // Pending drafts (blocked-on-human)
    prompt.push_str(&format!(
        "### Drafts awaiting review (blocked on human) ({})\n",
        activity.pending_drafts.len()
    ));
    if activity.pending_drafts.is_empty() {
        prompt.push_str("(none)\n\n");
    } else {
        for d in &activity.pending_drafts {
            let ts = d.created_at.split('T').next().unwrap_or(&d.created_at);
            prompt.push_str(&format!("- [{}] [{}] {} ({})\n", ts, d.project, d.title, d.status));
        }
        prompt.push('\n');
    }

    prompt.push_str(&format!(
        "---\n\n\
         Please produce the morning brief for {date}.\n\n\
         **Format your response as follows (use exactly these section headings):**\n\n\
         ```\n\
         # Morning Brief — {date}\n\n\
         ## Headline\n\
         **[The single most important thing today, with evidence]**\n\n\
         ## Summary\n\
         [2–3 sentences covering the overnight period at a glance]\n\n\
         ## Settled overnight\n\
         [What completed or moved to done]\n\n\
         ## Still in flight\n\
         [Active items and their status]\n\n\
         ## Anomalies & concerns\n\
         [Cost spikes, stuck items, failures, unusual patterns — or \"n/a\"]\n\n\
         ## Blocked on human\n\
         [Anything that needs operator input to proceed — or \"n/a\"]\n\
         ```\n\n\
         After the brief, include any follow-up Stitch drafts using this exact format \
         (one block per draft, no auto-submits — they go into the preview queue for operator review):\n\n\
         ```draft_stitch\n\
         project: <project-name>\n\
         title: <title>\n\
         description: <brief description of what to do>\n\
         kind: task\n\
         ```\n\n\
         Include draft Stitch blocks only if you identify concrete follow-ups worth the \
         operator's attention. Keep the brief concise — readable in 2 minutes on mobile.\n"
    ));

    prompt
}

// ---------------------------------------------------------------------------
// Draft Stitch parser
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
struct DraftStitchBlock {
    project: String,
    title: String,
    description: Option<String>,
    kind: String,
}

fn parse_draft_stitches(response: &str) -> Vec<DraftStitchBlock> {
    let mut drafts = Vec::new();
    let mut remaining = response;

    while let Some(start) = remaining.find("```draft_stitch") {
        let after_fence = &remaining[start + "```draft_stitch".len()..];
        // Skip optional newline after fence
        let block_start = after_fence.strip_prefix('\n').unwrap_or(after_fence);
        if let Some(end) = block_start.find("```") {
            let block = &block_start[..end];
            let draft = parse_draft_block(block);
            if !draft.project.is_empty() && !draft.title.is_empty() {
                drafts.push(draft);
            }
            remaining = &block_start[end + 3..];
        } else {
            break;
        }
    }
    drafts
}

fn parse_draft_block(block: &str) -> DraftStitchBlock {
    let mut draft = DraftStitchBlock {
        kind: "task".to_string(),
        ..Default::default()
    };
    for line in block.lines() {
        if let Some(v) = line.strip_prefix("project:") {
            draft.project = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("title:") {
            draft.title = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("description:") {
            draft.description = Some(v.trim().to_string());
        } else if let Some(v) = line.strip_prefix("kind:") {
            let k = v.trim().to_string();
            draft.kind = if matches!(k.as_str(), "task" | "bug" | "fix" | "epic") {
                k
            } else {
                "task".to_string()
            };
        }
    }
    draft
}

fn extract_headline(markdown: &str) -> String {
    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("**") && trimmed.ends_with("**") && trimmed.len() > 4 {
            return trimmed
                .trim_start_matches("**")
                .trim_end_matches("**")
                .trim()
                .to_string();
        }
    }
    // Fallback: first non-empty line after "## Headline"
    let mut after_headline = false;
    for line in markdown.lines() {
        if line.trim() == "## Headline" {
            after_headline = true;
            continue;
        }
        if after_headline {
            let t = line.trim();
            if !t.is_empty() && !t.starts_with('#') {
                return t.trim_matches('*').trim().to_string();
            }
        }
    }
    "Morning brief generated".to_string()
}

// ---------------------------------------------------------------------------
// Runner
// ---------------------------------------------------------------------------

/// Controls whether a brief is currently running (prevents concurrent runs).
#[derive(Debug, Default)]
pub struct RunState {
    running: bool,
    brief_id: Option<String>,
}

/// Orchestrates morning brief generation.
#[derive(Debug)]
pub struct MorningBriefRunner {
    config: MorningBriefConfig,
    agent_manager: Arc<AgentSessionManager>,
    brief_tx: broadcast::Sender<MorningBriefData>,
    run_state: Arc<Mutex<RunState>>,
}

impl MorningBriefRunner {
    pub fn new(
        config: MorningBriefConfig,
        agent_manager: Arc<AgentSessionManager>,
        brief_tx: broadcast::Sender<MorningBriefData>,
    ) -> Self {
        Self {
            config,
            agent_manager,
            brief_tx,
            run_state: Arc::new(Mutex::new(RunState::default())),
        }
    }

    /// Returns true if a brief is currently running.
    pub async fn is_running(&self) -> bool {
        self.run_state.lock().await.running
    }

    /// Trigger the morning brief. Returns the brief ID, or None if already running.
    pub async fn trigger(&self) -> Result<Option<String>> {
        {
            let mut state = self.run_state.lock().await;
            if state.running {
                return Ok(None);
            }
            state.running = true;
        }

        let brief_id = Uuid::new_v4().to_string();

        // Insert a "running" placeholder in the DB so the UI can show progress.
        let activity = match query_overnight_activity(self.config.window_hours) {
            Ok(a) => a,
            Err(e) => {
                let mut state = self.run_state.lock().await;
                state.running = false;
                return Err(e);
            }
        };

        let placeholder = fleet::MorningBriefRow {
            id: brief_id.clone(),
            generated_at: Utc::now().to_rfc3339(),
            window_from: activity.window_from.to_rfc3339(),
            window_to: activity.window_to.to_rfc3339(),
            headline: "Generating…".to_string(),
            markdown_content: String::new(),
            draft_ids: Vec::new(),
            session_id: None,
            status: "running".to_string(),
            error: None,
        };
        if let Err(e) = fleet::insert_morning_brief(&placeholder) {
            warn!("Failed to insert morning brief placeholder: {}", e);
        }

        // Broadcast "running" state to WS clients.
        let _ = self.brief_tx.send(MorningBriefData {
            id: brief_id.clone(),
            headline: "Generating morning brief…".to_string(),
            generated_at: placeholder.generated_at.clone(),
            draft_count: 0,
            status: "running".to_string(),
        });

        let prompt = build_brief_prompt(&activity);
        let manager = self.agent_manager.clone();
        let tx = self.brief_tx.clone();
        let run_state = self.run_state.clone();
        let id = brief_id.clone();
        let window_from = activity.window_from.to_rfc3339();
        let window_to = activity.window_to.to_rfc3339();

        tokio::spawn(async move {
            let result = run_brief_turn(manager, prompt, id.clone(), window_from, window_to).await;
            match result {
                Ok((headline, _markdown, draft_ids)) => {
                    let _ = tx.send(MorningBriefData {
                        id: id.clone(),
                        headline: headline.clone(),
                        generated_at: Utc::now().to_rfc3339(),
                        draft_count: draft_ids.len(),
                        status: "complete".to_string(),
                    });
                    info!("Morning brief {} complete: {} drafts", id, draft_ids.len());
                }
                Err(e) => {
                    warn!("Morning brief {} failed: {}", id, e);
                    if let Err(de) = fleet::update_morning_brief_status(&id, "failed", Some(&e.to_string())) {
                        warn!("Failed to mark brief as failed: {}", de);
                    }
                    let _ = tx.send(MorningBriefData {
                        id: id.clone(),
                        headline: format!("Brief failed: {}", e),
                        generated_at: Utc::now().to_rfc3339(),
                        draft_count: 0,
                        status: "failed".to_string(),
                    });
                }
            }
            let mut state = run_state.lock().await;
            state.running = false;
            state.brief_id = None;
        });

        Ok(Some(brief_id))
    }

    /// Start the background scheduler (checks every minute, runs at configured hour).
    pub fn start_scheduler(self: Arc<Self>, mut shutdown: broadcast::Receiver<crate::shutdown::ShutdownPhase>) {
        tokio::spawn(async move {
            let mut last_brief_date: Option<chrono::NaiveDate> = None;
            loop {
                tokio::select! {
                    _ = shutdown.recv() => break,
                    _ = tokio::time::sleep(std::time::Duration::from_secs(60)) => {
                        if !self.config.auto_run_enabled {
                            continue;
                        }
                        let now = Utc::now();
                        let current_hour = now.time().hour();
                        let today = now.date_naive();
                        let should_run = current_hour == self.config.schedule_hour
                            && last_brief_date.as_ref() != Some(&today);
                        if should_run {
                            last_brief_date = Some(today);
                            info!("Scheduler: triggering morning brief for {}", today);
                            if let Err(e) = self.trigger().await {
                                warn!("Scheduled morning brief failed to start: {}", e);
                            }
                        }
                    }
                }
            }
        });
    }
}

/// Sends the brief prompt to the active agent session and collects the response.
/// Returns (headline, markdown_content, draft_ids).
async fn run_brief_turn(
    manager: Arc<AgentSessionManager>,
    prompt: String,
    brief_id: String,
    window_from: String,
    window_to: String,
) -> Result<(String, String, Vec<String>)> {
    // Ensure there's an active session, spawning one if needed.
    if !manager.status().await.active {
        manager.spawn().await?;
    }

    // Capture the session_id for this brief.
    let session_id = manager.status().await.session_id;
    if let Some(ref sid) = session_id {
        let _ = fleet::update_morning_brief_session(&brief_id, sid);
    }

    let mut stream = manager.send_turn(prompt, vec![]).await?;

    // 3-minute wall-time budget for brief generation
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(180);

    let mut full_text = String::new();
    loop {
        let next = tokio::select! {
            event_result = stream.next() => {
                match event_result {
                    Some(Ok(event)) => Some(event),
                    Some(Err(e)) => {
                        warn!("Agent stream error during morning brief: {}", e);
                        None
                    }
                    None => None,
                }
            }
            _ = tokio::time::sleep_until(deadline) => {
                warn!("Morning brief timed out after 180s — using partial response");
                break;
            }
        };
        match next {
            Some(event) => {
                manager.handle_event(&event).await;
                if let crate::agent_adapter::AgentEvent::TextDelta { text } = &event {
                    full_text.push_str(text);
                }
            }
            None => break,
        }
    }

    if full_text.is_empty() {
        return Err(anyhow::anyhow!("Agent produced no response for morning brief"));
    }

    let headline = extract_headline(&full_text);

    // Parse draft Stitch blocks from the response.
    let draft_blocks = parse_draft_stitches(&full_text);
    let mut draft_ids = Vec::new();
    let now = Utc::now().to_rfc3339();

    for block in draft_blocks {
        let draft_id = Uuid::new_v4().to_string();
        let row = fleet::DraftRow {
            id: draft_id.clone(),
            project: block.project,
            title: block.title,
            kind: block.kind,
            description: block.description,
            has_acceptance_criteria: false,
            priority: None,
            labels: Vec::new(),
            created_by: format!("hoop:agent:morning-brief:{}", brief_id),
            created_at: now.clone(),
            source: "morning_brief".to_string(),
            agent_session_id: None,
            status: "pending".to_string(),
            version: 1,
            original_json: None,
            resolved_by: None,
            resolved_at: None,
            rejection_reason: None,
            stitch_id: None,
            preview_json: None,
        };
        match fleet::insert_draft(&row) {
            Ok(()) => draft_ids.push(draft_id),
            Err(e) => warn!("Failed to insert draft stitch from morning brief: {}", e),
        }
    }

    // Persist the completed brief.
    fleet::update_morning_brief_content(&brief_id, &headline, &full_text, &draft_ids)?;

    // Update with session ID from the manager status.
    let _ = fleet::update_morning_brief_status(&brief_id, "complete", None);

    let _ = window_from;
    let _ = window_to;
    Ok((headline, full_text, draft_ids))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_activity() -> OvernightActivity {
        OvernightActivity {
            window_from: Utc::now() - Duration::hours(24),
            window_to: Utc::now(),
            closed_stitches: vec![StitchSummary {
                title: "Fix auth token expiry".to_string(),
                project: "HOOP".to_string(),
                kind: "fix".to_string(),
                last_activity_at: "2026-04-23T05:00:00Z".to_string(),
            }],
            open_stitches: vec![StitchSummary {
                title: "Refactor session manager".to_string(),
                project: "HOOP".to_string(),
                kind: "task".to_string(),
                last_activity_at: "2026-04-23T06:30:00Z".to_string(),
            }],
            new_stitches: vec![StitchSummary {
                title: "Add logging to API".to_string(),
                project: "NEEDLE".to_string(),
                kind: "task".to_string(),
                last_activity_at: "2026-04-23T03:00:00Z".to_string(),
            }],
            failed_actions: vec![ActionSummary {
                kind: "stitch_submit".to_string(),
                target: "stitch-42".to_string(),
                project: Some("HOOP".to_string()),
                error: Some("br create failed: database locked".to_string()),
                ts: "2026-04-23T02:15:00Z".to_string(),
            }],
            stuck_stitches: vec![StitchSummary {
                title: "Investigate memory leak".to_string(),
                project: "SIGIL".to_string(),
                kind: "investigation".to_string(),
                last_activity_at: "2026-04-22T10:00:00Z".to_string(),
            }],
            pending_drafts: vec![DraftSummary {
                title: "Upgrade deps".to_string(),
                project: "HOOP".to_string(),
                status: "pending".to_string(),
                created_at: "2026-04-22T20:00:00Z".to_string(),
            }],
            project_names: vec!["HOOP".to_string(), "NEEDLE".to_string(), "SIGIL".to_string()],
        }
    }

    #[test]
    fn test_build_brief_prompt_contains_all_sections() {
        let activity = sample_activity();
        let prompt = build_brief_prompt(&activity);

        assert!(prompt.contains("## Morning Brief Request"));
        assert!(prompt.contains("### Projects monitored"));
        assert!(prompt.contains("- HOOP"));
        assert!(prompt.contains("- NEEDLE"));
        assert!(prompt.contains("### Stitches settled"));
        assert!(prompt.contains("Fix auth token expiry"));
        assert!(prompt.contains("### Stitches still in flight"));
        assert!(prompt.contains("Refactor session manager"));
        assert!(prompt.contains("### New stitches created"));
        assert!(prompt.contains("Add logging to API"));
        assert!(prompt.contains("### Failures"));
        assert!(prompt.contains("stitch_submit"));
        assert!(prompt.contains("database locked"));
        assert!(prompt.contains("### Stuck stitches"));
        assert!(prompt.contains("Investigate memory leak"));
        assert!(prompt.contains("### Drafts awaiting review"));
        assert!(prompt.contains("Upgrade deps"));
        assert!(prompt.contains("draft_stitch"));
        assert!(prompt.contains("## Headline"));
    }

    #[test]
    fn test_build_brief_prompt_empty_activity() {
        let activity = OvernightActivity {
            window_from: Utc::now() - Duration::hours(24),
            window_to: Utc::now(),
            closed_stitches: vec![],
            open_stitches: vec![],
            new_stitches: vec![],
            failed_actions: vec![],
            stuck_stitches: vec![],
            pending_drafts: vec![],
            project_names: vec![],
        };
        let prompt = build_brief_prompt(&activity);

        assert!(prompt.contains("(none active in window)"));
        assert!(prompt.contains("### Stitches settled"));
        assert!(prompt.contains("(none)"));
    }

    #[test]
    fn test_parse_draft_stitches_single() {
        let response = r#"
# Morning Brief — 2026-04-23

## Headline
**Auth fix deployed overnight**

## Summary
All quiet on the western front.

```draft_stitch
project: HOOP
title: Follow up on token rotation
description: Verify the new token rotation is working across all services
kind: task
```
"#;
        let drafts = parse_draft_stitches(response);
        assert_eq!(drafts.len(), 1);
        assert_eq!(drafts[0].project, "HOOP");
        assert_eq!(drafts[0].title, "Follow up on token rotation");
        assert_eq!(
            drafts[0].description.as_deref(),
            Some("Verify the new token rotation is working across all services")
        );
        assert_eq!(drafts[0].kind, "task");
    }

    #[test]
    fn test_parse_draft_stitches_multiple() {
        let response = r#"
Some text before.

```draft_stitch
project: HOOP
title: First draft
kind: bug
```

Some text between.

```draft_stitch
project: NEEDLE
title: Second draft
description: With a description
kind: fix
```
"#;
        let drafts = parse_draft_stitches(response);
        assert_eq!(drafts.len(), 2);
        assert_eq!(drafts[0].project, "HOOP");
        assert_eq!(drafts[0].kind, "bug");
        assert_eq!(drafts[1].project, "NEEDLE");
        assert_eq!(drafts[1].kind, "fix");
    }

    #[test]
    fn test_parse_draft_stitches_invalid_kind_defaults_to_task() {
        let response = r#"
```draft_stitch
project: HOOP
title: Some title
kind: invalid_kind
```
"#;
        let drafts = parse_draft_stitches(response);
        assert_eq!(drafts.len(), 1);
        assert_eq!(drafts[0].kind, "task");
    }

    #[test]
    fn test_parse_draft_stitches_missing_project_skipped() {
        let response = r#"
```draft_stitch
title: No project
kind: task
```
"#;
        let drafts = parse_draft_stitches(response);
        assert!(drafts.is_empty());
    }

    #[test]
    fn test_parse_draft_stitches_missing_title_skipped() {
        let response = r#"
```draft_stitch
project: HOOP
kind: task
```
"#;
        let drafts = parse_draft_stitches(response);
        assert!(drafts.is_empty());
    }

    #[test]
    fn test_parse_draft_stitches_no_blocks() {
        let response = "Just some regular markdown without any draft blocks.";
        let drafts = parse_draft_stitches(response);
        assert!(drafts.is_empty());
    }

    #[test]
    fn test_parse_draft_stitches_unclosed_block() {
        let response = r#"
```draft_stitch
project: HOOP
title: Unclosed block
"#;
        let drafts = parse_draft_stitches(response);
        assert!(drafts.is_empty());
    }

    #[test]
    fn test_extract_headline_bold_format() {
        let md = "# Morning Brief — 2026-04-23\n\n## Headline\n\n**Auth fix deployed with zero downtime**\n";
        assert_eq!(extract_headline(md), "Auth fix deployed with zero downtime");
    }

    #[test]
    fn test_extract_headline_after_heading() {
        let md = "## Headline\nAuth fix deployed\n\n## Summary\nSome text\n";
        assert_eq!(extract_headline(md), "Auth fix deployed");
    }

    #[test]
    fn test_extract_headline_fallback() {
        let md = "## Summary\nJust some text without a headline section.\n";
        assert_eq!(extract_headline(md), "Morning brief generated");
    }

    #[test]
    fn test_extract_headline_strips_bold_markers() {
        let md = "## Headline\n**This is bold** and this is not\n";
        assert_eq!(extract_headline(md), "This is bold");
    }

    #[test]
    fn test_morning_brief_config_defaults() {
        let config = MorningBriefConfig::default();
        assert_eq!(config.window_hours, 24);
        assert_eq!(config.schedule_hour, 7);
        assert!(config.auto_run_enabled);
    }

    #[test]
    fn test_morning_brief_config_deserialize() {
        let yaml = "window_hours: 12\nschedule_hour: 8\nauto_run_enabled: false";
        let config: MorningBriefConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.window_hours, 12);
        assert_eq!(config.schedule_hour, 8);
        assert!(!config.auto_run_enabled);
    }

    #[test]
    fn test_parse_draft_stitches_epic_kind() {
        let response = r#"
```draft_stitch
project: HOOP
title: Big feature
kind: epic
```
"#;
        let drafts = parse_draft_stitches(response);
        assert_eq!(drafts.len(), 1);
        assert_eq!(drafts[0].kind, "epic");
    }
}
