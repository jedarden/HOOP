//! REST API endpoint for per-worker timeline data (Gantt-style activity view).
//!
//! Derives timeline segments from bead events (Claim/Close/Release) stored in
//! the WorkerRegistry. Returns per-worker sequences of activity segments with
//! start/end times, bead IDs, and outcomes.

use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{DaemonState, ws::{BeadEventData, WorkerData}};

/// Query parameters for the timeline endpoint
#[derive(Debug, Deserialize)]
pub struct TimelineQuery {
    /// Number of hours to look back (default: 24)
    #[serde(default = "default_hours")]
    pub hours: u32,
}

fn default_hours() -> u32 {
    24
}

/// A single segment of worker activity on the timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineSegment {
    /// ISO 8601 timestamp when this segment started
    pub start: String,
    /// ISO 8601 timestamp when this segment ended (None if still active)
    pub end: Option<String>,
    /// Bead ID being worked on
    pub bead_id: String,
    /// How the segment ended: "closed", "released", or "active"
    pub outcome: String,
}

/// Per-worker timeline data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerTimeline {
    /// Worker name
    pub worker: String,
    /// Activity segments (executing periods)
    pub segments: Vec<TimelineSegment>,
    /// Heartbeat timestamps within the window
    pub heartbeats: Vec<String>,
    /// Current liveness state
    pub liveness: String,
}

/// Timeline response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineResponse {
    /// Start of the time window (ISO 8601)
    pub window_start: String,
    /// End of the time window (ISO 8601)
    pub window_end: String,
    /// Per-worker timelines
    pub workers: Vec<WorkerTimeline>,
}

/// GET /api/workers/timeline?hours=24
pub async fn get_worker_timeline(
    State(state): State<DaemonState>,
    Query(query): Query<TimelineQuery>,
) -> Json<TimelineResponse> {
    let hours = query.hours.min(168).max(1); // clamp to 1h–7d
    let now = chrono::Utc::now();
    let window_start = now - chrono::Duration::hours(hours as i64);

    let all_events = state.worker_registry.all_bead_events().await;
    let workers = state.worker_registry.snapshot().await;

    // Build a reverse index: worker -> sorted events
    let mut worker_events: HashMap<String, Vec<&BeadEventData>> = HashMap::new();
    for (_bead_id, events) in &all_events {
        for event in events {
            // Parse timestamp and filter to window
            if let Ok(ts) = event.timestamp.parse::<chrono::DateTime<chrono::Utc>>() {
                if ts >= window_start {
                    worker_events
                        .entry(event.worker.clone())
                        .or_default()
                        .push(event);
                }
            }
        }
    }

    // Sort each worker's events by timestamp
    for events in worker_events.values_mut() {
        events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    }

    // Derive segments: a Claim starts a segment, Close/Release ends it
    let mut timelines: Vec<WorkerTimeline> = Vec::new();

    // Build a map of current worker states for active segments
    let worker_states: HashMap<String, &WorkerData> =
        workers.iter().map(|w| (w.worker.clone(), w)).collect();

    for (worker, events) in &worker_events {
        let mut segments: Vec<TimelineSegment> = Vec::new();
        let mut current_segment: Option<TimelineSegment> = None;
        let mut heartbeats: Vec<String> = Vec::new();

        for event in events {
            match event.event_type.as_str() {
                "claim" => {
                    // Close any open segment (shouldn't normally happen, but be safe)
                    if let Some(seg) = current_segment.take() {
                        segments.push(seg);
                    }
                    current_segment = Some(TimelineSegment {
                        start: event.timestamp.clone(),
                        end: None,
                        bead_id: event.bead_id.clone(),
                        outcome: "active".to_string(),
                    });
                    heartbeats.push(event.timestamp.clone());
                }
                "close" => {
                    if let Some(mut seg) = current_segment.take() {
                        seg.end = Some(event.timestamp.clone());
                        seg.outcome = "closed".to_string();
                        segments.push(seg);
                    }
                    heartbeats.push(event.timestamp.clone());
                }
                "release" => {
                    if let Some(mut seg) = current_segment.take() {
                        seg.end = Some(event.timestamp.clone());
                        seg.outcome = "released".to_string();
                        segments.push(seg);
                    }
                    heartbeats.push(event.timestamp.clone());
                }
                "update" => {
                    heartbeats.push(event.timestamp.clone());
                }
                _ => {}
            }
        }

        // If there's still an open segment, check current worker state
        if let Some(seg) = &mut current_segment {
            // Check if the worker is still executing this bead
            if let Some(wdata) = worker_states.get(worker) {
                if let crate::ws::WorkerDisplayState::Executing { bead, .. } = &wdata.state {
                    if bead == &seg.bead_id {
                        seg.outcome = "active".to_string();
                        segments.push(current_segment.take().unwrap());
                    } else {
                        // Worker moved on — close the segment
                        seg.end = Some(wdata.last_heartbeat.to_rfc3339());
                        seg.outcome = "released".to_string();
                        segments.push(current_segment.take().unwrap());
                    }
                } else {
                    // Worker is idle/knot — segment ended
                    seg.end = Some(wdata.last_heartbeat.to_rfc3339());
                    seg.outcome = if matches!(wdata.state, crate::ws::WorkerDisplayState::Knot { .. }) {
                        "knot".to_string()
                    } else {
                        "released".to_string()
                    };
                    segments.push(current_segment.take().unwrap());
                }
            } else {
                // Worker no longer known — close it at the window boundary
                seg.end = Some(now.to_rfc3339());
                seg.outcome = "unknown".to_string();
                segments.push(current_segment.take().unwrap());
            }
        }

        let liveness = worker_states
            .get(worker)
            .map(|w| format!("{:?}", w.liveness).to_lowercase())
            .unwrap_or_else(|| "unknown".to_string());

        timelines.push(WorkerTimeline {
            worker: worker.clone(),
            segments,
            heartbeats,
            liveness,
        });
    }

    // Add workers that have no events in the window but are currently active
    for wdata in &workers {
        if !worker_events.contains_key(&wdata.worker) {
            timelines.push(WorkerTimeline {
                worker: wdata.worker.clone(),
                segments: Vec::new(),
                heartbeats: Vec::new(),
                liveness: format!("{:?}", wdata.liveness).to_lowercase(),
            });
        }
    }

    // Sort by worker name for stable ordering
    timelines.sort_by(|a, b| a.worker.cmp(&b.worker));

    Json(TimelineResponse {
        window_start: window_start.to_rfc3339(),
        window_end: now.to_rfc3339(),
        workers: timelines,
    })
}
