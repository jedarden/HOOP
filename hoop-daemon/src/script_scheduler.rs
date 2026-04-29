//! Script scheduler — cron-based automatic script execution (§22.3)
//!
//! Scripts can declare a cron schedule in their manifest.yml. The scheduler
//! fires scripts per their schedule, respecting overlap policies (skip/queue/parallel).
//!
//! Schedule changes are hot-reloaded on each tick. Last-fire and next-fire times
//! are surfaced via the API for UI display.

use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::api_scripts::{discover_scripts, execute_script, OverlapPolicy};
use crate::shutdown::ShutdownPhase;

/// Tracking state for a scheduled script
#[derive(Debug, Clone)]
struct ScriptScheduleState {
    /// Last time this script was executed
    last_fire: Option<DateTime<Utc>>,
    /// Next scheduled execution time
    next_fire: Option<DateTime<Utc>>,
    /// Whether the script is currently running
    running: bool,
    /// Parsed cron schedule
    schedule: Option<CronSchedule>,
}

/// Script scheduler state
#[derive(Debug)]
pub struct ScriptScheduler {
    /// Directory containing scripts
    scripts_dir: PathBuf,
    /// Per-script schedule state
    state: Arc<RwLock<HashMap<String, ScriptScheduleState>>>,
}

impl ScriptScheduler {
    /// Create a new script scheduler
    pub fn new(scripts_dir: PathBuf) -> Self {
        Self {
            scripts_dir,
            state: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start the scheduler loop
    ///
    /// Checks every 60 seconds for scripts that should run based on their cron schedule.
    /// Hot-reloads schedule changes on each tick.
    pub fn start_scheduler(
        self: Arc<Self>,
        mut shutdown: tokio::sync::broadcast::Receiver<ShutdownPhase>,
    ) {
        tokio::spawn(async move {
            let mut tick_interval = tokio::time::interval(std::time::Duration::from_secs(60));
            tick_interval.tick().await; // Skip first immediate tick

            loop {
                tokio::select! {
                    _ = shutdown.recv() => {
                        info!("Script scheduler shutting down");
                        break;
                    }
                    _ = tick_interval.tick() => {
                        if let Err(e) = Self::tick(&self).await {
                            warn!("Script scheduler tick failed: {}", e);
                        }
                    }
                }
            }
        });
    }

    /// Single scheduler tick — check all scripts and run those due
    async fn tick(this: &Arc<Self>) -> Result<()> {
        let now = Utc::now();

        // Discover scripts and update schedule state
        let scripts = discover_scripts(&this.scripts_dir);

        // Build map of scheduled scripts
        let mut scheduled = HashMap::new();
        for script in &scripts {
            if let Some(ref manifest) = script.manifest {
                if let Some(ref schedule_str) = manifest.schedule {
                    match CronSchedule::parse(schedule_str) {
                        Ok(schedule) => {
                            scheduled.insert(
                                script.name.clone(),
                                (schedule, manifest.overlap_policy.clone()),
                            );
                        }
                        Err(e) => {
                            warn!("Invalid cron schedule for script '{}': {}", script.name, e);
                        }
                    }
                }
            }
        }

        // Update state with current schedule info
        let mut state = this.state.write().await;
        for (name, (schedule, overlap_policy)) in &scheduled {
            let entry = state
                .entry(name.clone())
                .or_insert_with(|| ScriptScheduleState {
                    last_fire: None,
                    next_fire: None,
                    running: false,
                    schedule: Some(schedule.clone()),
                });

            // Update schedule if changed
            entry.schedule = Some(schedule.clone());

            // Calculate next fire time if not set
            if entry.next_fire.is_none() || entry.next_fire.unwrap() < now {
                entry.next_fire = Self::find_next_fire(schedule, &now);
            }
        }

        // Remove state for scripts that no longer exist or have no schedule
        state.retain(|name, _| scheduled.contains_key(name));

        // Drop write lock before spawning tasks
        drop(state);

        // Check which scripts should run now
        let scripts_to_run: Vec<_> = {
            let state = this.state.read().await;
            scheduled
                .into_iter()
                .filter_map(|(name, (schedule, overlap_policy))| {
                    let entry = state.get(&name)?;
                    let next_fire = entry.next_fire?;

                    // Check if due (within the last minute to avoid missing runs)
                    let due = next_fire <= now && next_fire > now - chrono::Duration::minutes(1);

                    if !due {
                        return None;
                    }

                    // Check overlap policy
                    if entry.running && overlap_policy == OverlapPolicy::Skip {
                        debug!("Script '{}' skipped due to overlap policy", name);
                        return None;
                    }

                    Some((name, overlap_policy))
                })
                .collect()
        };

        // Execute due scripts
        for (name, overlap_policy) in scripts_to_run {
            let scheduler = Arc::clone(this);
            tokio::spawn(async move {
                if let Err(e) = scheduler.run_script(&name, overlap_policy).await {
                    warn!("Failed to run scheduled script '{}': {}", name, e);
                }
            });
        }

        Ok(())
    }

    /// Run a scheduled script
    async fn run_script(&self, name: &str, overlap_policy: OverlapPolicy) -> Result<()> {
        // Check overlap policy again (might have changed since we checked)
        if overlap_policy == OverlapPolicy::Skip {
            let state = self.state.read().await;
            if let Some(entry) = state.get(name) {
                if entry.running {
                    debug!("Script '{}' skipped due to overlap policy (queue)", name);
                    return Ok(());
                }
            }
        }

        // Mark as running
        {
            let mut state = self.state.write().await;
            if let Some(entry) = state.get_mut(name) {
                entry.running = true;
            }
        }

        // Discover script to get path
        let scripts = discover_scripts(&self.scripts_dir);
        let script = scripts
            .iter()
            .find(|s| s.name == name)
            .context("Script not found")?;

        let timeout_secs = script
            .manifest
            .as_ref()
            .map(|m| m.timeout_secs)
            .unwrap_or(300);

        info!("Executing scheduled script '{}'", name);

        // Run in blocking task
        let script_path = script.path.clone();
        let result =
            tokio::task::spawn_blocking(move || execute_script(&script_path, &[], timeout_secs))
                .await
                .map_err(|e| anyhow::anyhow!("Failed to join script execution task: {}", e))?
                .map_err(|e| anyhow::anyhow!("{}", e))?;

        // Update state
        let now = Utc::now();
        {
            let mut state = self.state.write().await;
            if let Some(entry) = state.get_mut(name) {
                entry.running = false;
                entry.last_fire = Some(now);
                // Recalculate next fire time
                if let Some(ref schedule) = entry.schedule {
                    entry.next_fire = Self::find_next_fire(schedule, &now);
                }
            }
        }

        info!(
            "Scheduled script '{}' completed with status: {}",
            name, result.status
        );

        Ok(())
    }

    /// Find the next fire time for a cron schedule
    fn find_next_fire(schedule: &CronSchedule, after: &DateTime<Utc>) -> Option<DateTime<Utc>> {
        // Check up to 4 years ahead (covers leap years)
        for day in 0..(366 * 4) {
            let candidate = *after + chrono::Duration::days(day);
            // Round to next minute
            let naive = candidate.naive_utc();
            let naive = naive
                .with_second(0)
                .and_then(|d| d.with_nanosecond(0))
                .unwrap_or(naive);
            let candidate = DateTime::from_naive_utc_and_offset(naive, Utc);

            if schedule.matches(&candidate) {
                // Find the first valid minute on this day
                for minute in 0..(24 * 60) {
                    let time_candidate = candidate + chrono::Duration::minutes(minute);
                    if schedule.matches(&time_candidate) && time_candidate > *after {
                        return Some(time_candidate);
                    }
                }
            }
        }
        None
    }

    /// Get schedule state for all scripts (for API)
    pub async fn get_schedule_state(&self) -> HashMap<String, ScheduleState> {
        let state = self.state.read().await;
        state
            .iter()
            .map(|(name, entry)| {
                (
                    name.clone(),
                    ScheduleState {
                        last_fire: entry.last_fire.map(|d| d.to_rfc3339()),
                        next_fire: entry.next_fire.map(|d| d.to_rfc3339()),
                        running: entry.running,
                    },
                )
            })
            .collect()
    }

    /// Update schedule state for a script (called after execution)
    pub async fn update_script_state(&self, name: &str, last_fire: DateTime<Utc>) {
        let mut state = self.state.write().await;
        if let Some(entry) = state.get_mut(name) {
            entry.last_fire = Some(last_fire);
            if let Some(ref schedule) = entry.schedule {
                entry.next_fire = Self::find_next_fire(schedule, &last_fire);
            }
        }
    }
}

/// Schedule state for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleState {
    /// Last scheduled execution time (RFC3339)
    pub last_fire: Option<String>,
    /// Next scheduled execution time (RFC3339)
    pub next_fire: Option<String>,
    /// Whether the script is currently running
    pub running: bool,
}

/// Minimal 5-field cron matcher (copied from backup_pipeline)
#[derive(Debug, Clone)]
struct CronSchedule {
    minutes: Vec<u32>,
    hours: Vec<u32>,
    doms: Vec<u32>,
    months: Vec<u32>,
    dows: Vec<u32>,
}

impl CronSchedule {
    fn parse(expr: &str) -> Result<Self> {
        let fields: Vec<&str> = expr.split_whitespace().collect();
        if fields.len() != 5 {
            anyhow::bail!("cron must have 5 fields, got {}", fields.len());
        }

        Ok(CronSchedule {
            minutes: parse_cron_field(fields[0], 0, 59)?,
            hours: parse_cron_field(fields[1], 0, 23)?,
            doms: parse_cron_field(fields[2], 1, 31)?,
            months: parse_cron_field(fields[3], 1, 12)?,
            dows: parse_cron_field(fields[4], 0, 6)?,
        })
    }

    fn matches(&self, t: &DateTime<Utc>) -> bool {
        self.minutes.contains(&(t.time().minute() as u32))
            && self.hours.contains(&(t.time().hour() as u32))
            && self.doms.contains(&(t.date_naive().day() as u32))
            && self.months.contains(&(t.date_naive().month() as u32))
            && self.dows.contains(&t.weekday().num_days_from_sunday())
    }
}

fn parse_cron_field(field: &str, lo: u32, hi: u32) -> Result<Vec<u32>> {
    if field == "*" {
        return Ok((lo..=hi).collect());
    }
    let mut vals = Vec::new();
    for part in field.split(',') {
        if let Some((a, b)) = part.split_once('-') {
            let s: u32 = a.parse().unwrap_or(lo);
            let e: u32 = b.parse().unwrap_or(hi);
            vals.extend(s..=e);
        } else if let Some(step_str) = part.strip_prefix("*/") {
            let step: u32 = step_str.parse().unwrap_or(1).max(1);
            let mut v = lo;
            while v <= hi {
                vals.push(v);
                v += step;
            }
        } else if let Ok(v) = part.parse::<u32>() {
            vals.push(v);
        }
    }
    vals.sort();
    vals.dedup();
    Ok(vals)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cron_field_star() {
        assert_eq!(
            parse_cron_field("*", 0, 59).unwrap(),
            (0..=59).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_parse_cron_field_list() {
        assert_eq!(parse_cron_field("1,2,3", 0, 59).unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn test_parse_cron_field_range() {
        assert_eq!(parse_cron_field("1-5", 0, 59).unwrap(), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_parse_cron_field_step() {
        assert_eq!(
            parse_cron_field("*/15", 0, 59).unwrap(),
            vec![0, 15, 30, 45]
        );
    }

    #[test]
    fn test_cron_schedule_parse() {
        let s = CronSchedule::parse("0 4 * * *").unwrap();
        assert_eq!(s.minutes, vec![0]);
        assert_eq!(s.hours, vec![4]);
        assert_eq!(s.doms.len(), 31);
    }

    #[test]
    fn test_cron_schedule_matches() {
        let s = CronSchedule::parse("30 14 * * *").unwrap();
        let t = DateTime::parse_from_rfc3339("2024-06-15T14:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert!(s.matches(&t));

        let t2 = DateTime::parse_from_rfc3339("2024-06-15T14:31:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert!(!s.matches(&t2));
    }
}
