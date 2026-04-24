//! Aggregated-read endpoint for a single Stitch.
//!
//! `GET /api/stitches/:id` returns the canonical aggregated view:
//!   - stitch row from fleet.db
//!   - all messages
//!   - linked beads with live status (queried from in-memory bead store)
//!   - touched files (from messages with file paths or audit trail)
//!   - cost / duration roll-up (from stitch_messages.tokens)
//!   - link graph (stitch_links)

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::fleet;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct AggregatedStitchResponse {
    pub stitch: StitchRow,
    pub messages: Vec<StitchMessage>,
    pub linked_beads: Vec<LinkedBead>,
    pub touched_files: Vec<TouchedFile>,
    pub cost_duration: CostDuration,
    pub link_graph: LinkGraph,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct StitchRow {
    pub id: String,
    pub project: String,
    pub kind: String,
    pub title: String,
    pub created_by: String,
    pub created_at: String,
    pub last_activity_at: String,
    pub participants: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct StitchMessage {
    pub id: String,
    pub ts: String,
    pub role: String,
    pub content: String,
    pub tokens: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkedBead {
    pub bead_id: String,
    pub workspace: String,
    pub relationship: String,
    /// Live status resolved from the in-memory bead store.
    /// None if the bead is not found in any loaded project.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub live_status: Option<LiveBeadStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LiveBeadStatus {
    pub title: String,
    pub status: String,
    pub priority: i64,
    pub issue_type: String,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: String,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct TouchedFile {
    pub path: String,
    /// How many messages reference this file
    pub mention_count: usize,
}

#[derive(Debug, Serialize)]
pub struct CostDuration {
    /// Total token count across all messages
    pub total_tokens: i64,
    /// Number of messages
    pub message_count: usize,
    /// Wall-clock duration from first to last message (ISO 8601 duration or "N/A" if <2 msgs)
    pub wall_clock: String,
    /// Earliest message timestamp
    pub first_message_ts: Option<String>,
    /// Latest message timestamp
    pub last_message_ts: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LinkGraph {
    /// Stitches this one links to (outgoing)
    pub outgoing: Vec<StitchLink>,
    /// Stitches that link to this one (incoming)
    pub incoming: Vec<StitchLink>,
}

#[derive(Debug, Serialize)]
pub struct StitchLink {
    pub stitch_id: String,
    pub kind: String,
    /// Title of the linked stitch (looked up, None if deleted)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<crate::DaemonState> {
    Router::new().route("/api/stitches/{id}", get(read_stitch))
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

async fn read_stitch(
    Path(stitch_id): Path<String>,
    State(state): State<crate::DaemonState>,
) -> Result<Json<AggregatedStitchResponse>, (StatusCode, String)> {
    crate::id_validators::validate_stitch_id(&stitch_id).map_err(crate::id_validators::rejection)?;

    let start = Instant::now();

    let db_path = fleet::db_path();
    let conn = rusqlite::Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to open fleet.db: {}", e)))?;

    // 1. Stitch row
    let stitch = query_stitch_row(&conn, &stitch_id)?;

    // 2. Messages
    let messages = query_messages(&conn, &stitch_id)?;

    // 3. Linked beads (raw from DB)
    let raw_beads = query_linked_beads_raw(&conn, &stitch_id)?;

    // 4. Resolve live bead status from in-memory store
    let beads_lock = state.beads.read().unwrap();
    let linked_beads: Vec<LinkedBead> = raw_beads
        .into_iter()
        .map(|mut b| {
            b.live_status = beads_lock.iter().find(|ib| ib.id == b.bead_id).map(|ib| {
                LiveBeadStatus {
                    title: ib.title.clone(),
                    status: format!("{:?}", ib.status).to_lowercase(),
                    priority: ib.priority,
                    issue_type: format!("{:?}", ib.issue_type).to_lowercase(),
                    created_at: ib.created_at.to_rfc3339(),
                    updated_at: ib.updated_at.to_rfc3339(),
                    created_by: ib.created_by.clone(),
                    dependencies: ib.dependencies.clone(),
                }
            });
            b
        })
        .collect();
    drop(beads_lock);

    // 5. Touched files — extract file paths from message content
    let touched_files = extract_touched_files(&messages);

    // 6. Cost / duration roll-up
    let cost_duration = compute_cost_duration(&messages);

    // 7. Link graph
    let link_graph = query_link_graph(&conn, &stitch_id)?;

    let elapsed_ms = start.elapsed().as_secs_f64() * 1_000.0;

    Ok(Json(AggregatedStitchResponse {
        stitch,
        messages,
        linked_beads,
        touched_files,
        cost_duration,
        link_graph,
        elapsed_ms: Some(elapsed_ms),
    }))
}

// ---------------------------------------------------------------------------
// Query helpers
// ---------------------------------------------------------------------------

fn query_stitch_row(
    conn: &rusqlite::Connection,
    stitch_id: &str,
) -> Result<StitchRow, (StatusCode, String)> {
    conn.query_row(
        "SELECT id, project, kind, title, created_by, created_at, last_activity_at, participants \
         FROM stitches WHERE id = ?1",
        [stitch_id],
        |row| {
            let participants: String = row.get(7).unwrap_or_else(|_| "[]".to_string());
            let participants_val: serde_json::Value =
                serde_json::from_str(&participants).unwrap_or(serde_json::Value::Array(vec![]));
            Ok(StitchRow {
                id: row.get(0)?,
                project: row.get(1)?,
                kind: row.get(2)?,
                title: row.get(3)?,
                created_by: row.get(4)?,
                created_at: row.get(5)?,
                last_activity_at: row.get(6)?,
                participants: participants_val,
            })
        },
    )
    .map_err(|e| {
        if e == rusqlite::Error::QueryReturnedNoRows {
            (StatusCode::NOT_FOUND, format!("Stitch '{}' not found", stitch_id))
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e))
        }
    })
}

fn query_messages(
    conn: &rusqlite::Connection,
    stitch_id: &str,
) -> Result<Vec<StitchMessage>, (StatusCode, String)> {
    let mut stmt = conn
        .prepare(
            "SELECT id, ts, role, content, tokens \
             FROM stitch_messages WHERE stitch_id = ?1 ORDER BY ts",
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Prepare messages: {}", e)))?;

    let rows = stmt
        .query_map([stitch_id], |row| {
            Ok(StitchMessage {
                id: row.get(0)?,
                ts: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                tokens: row.get(4)?,
            })
        })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query messages: {}", e)))?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Message row: {}", e)))?);
    }
    Ok(out)
}

fn query_linked_beads_raw(
    conn: &rusqlite::Connection,
    stitch_id: &str,
) -> Result<Vec<LinkedBead>, (StatusCode, String)> {
    let mut stmt = conn
        .prepare(
            "SELECT bead_id, workspace, relationship \
             FROM stitch_beads WHERE stitch_id = ?1",
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Prepare beads: {}", e)))?;

    let rows = stmt
        .query_map([stitch_id], |row| {
            Ok(LinkedBead {
                bead_id: row.get(0)?,
                workspace: row.get(1)?,
                relationship: row.get(2)?,
                live_status: None,
            })
        })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query beads: {}", e)))?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Bead row: {}", e)))?);
    }
    Ok(out)
}

fn query_link_graph(
    conn: &rusqlite::Connection,
    stitch_id: &str,
) -> Result<LinkGraph, (StatusCode, String)> {
    // Outgoing
    let mut stmt = conn
        .prepare(
            "SELECT sl.to_stitch, sl.kind, s.title \
             FROM stitch_links sl LEFT JOIN stitches s ON sl.to_stitch = s.id \
             WHERE sl.from_stitch = ?1",
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Prepare outgoing: {}", e)))?;

    let rows = stmt
        .query_map([stitch_id], |row| {
            let title: Option<String> = row.get(2)?;
            Ok(StitchLink {
                stitch_id: row.get(0)?,
                kind: row.get(1)?,
                title,
            })
        })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query outgoing: {}", e)))?;

    let mut outgoing = Vec::new();
    for row in rows {
        outgoing.push(row.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Outgoing row: {}", e)))?);
    }

    // Incoming
    let mut stmt = conn
        .prepare(
            "SELECT sl.from_stitch, sl.kind, s.title \
             FROM stitch_links sl LEFT JOIN stitches s ON sl.from_stitch = s.id \
             WHERE sl.to_stitch = ?1",
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Prepare incoming: {}", e)))?;

    let rows = stmt
        .query_map([stitch_id], |row| {
            let title: Option<String> = row.get(2)?;
            Ok(StitchLink {
                stitch_id: row.get(0)?,
                kind: row.get(1)?,
                title,
            })
        })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query incoming: {}", e)))?;

    let mut incoming = Vec::new();
    for row in rows {
        incoming.push(row.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Incoming row: {}", e)))?);
    }

    Ok(LinkGraph { outgoing, incoming })
}

// ---------------------------------------------------------------------------
// Derived data helpers
// ---------------------------------------------------------------------------

/// Extract file paths mentioned in messages. Looks for common patterns:
///   - `src/foo.rs`, `lib/bar.ts`, paths with extensions
///   - backtick-wrapped paths: `` `path/to/file.rs` ``
fn extract_touched_files(messages: &[StitchMessage]) -> Vec<TouchedFile> {
    use std::collections::HashMap;

    let mut counts: HashMap<String, usize> = HashMap::new();

    // Regex: backtick-wrapped path or bare path-like string (at least one / and a .extension)
    let re = regex::Regex::new(r"`([a-zA-Z0-9_./\-]+\.[a-zA-Z0-9]+)`|(?:(?:src|lib|test|pkg|cmd|crates|hoop-[a-z]+)/[a-zA-Z0-9_./\-]+\.[a-zA-Z0-9]+)")
        .unwrap();

    for msg in messages {
        for cap in re.captures_iter(&msg.content) {
            // Prefer the backtick group, fall back to full match
            let path = cap
                .get(1)
                .map(|m| m.as_str())
                .unwrap_or_else(|| cap.get(0).unwrap().as_str());
            // Filter out obvious non-file patterns
            if path.len() > 3 && !path.contains(' ') {
                *counts.entry(path.to_string()).or_insert(0) += 1;
            }
        }
    }

    let mut files: Vec<TouchedFile> = counts
        .into_iter()
        .map(|(path, mention_count)| TouchedFile { path, mention_count })
        .collect();
    files.sort_by(|a, b| b.mention_count.cmp(&a.mention_count));
    files.truncate(50);
    files
}

/// Compute cost / duration roll-up from messages.
fn compute_cost_duration(messages: &[StitchMessage]) -> CostDuration {
    let total_tokens: i64 = messages.iter().filter_map(|m| m.tokens).sum();
    let message_count = messages.len();

    let (first_ts, last_ts, wall_clock) = if messages.len() >= 2 {
        let first = &messages[0].ts;
        let last = &messages[messages.len() - 1].ts;

        let wall_clock = match (
            chrono::DateTime::parse_from_rfc3339(first),
            chrono::DateTime::parse_from_rfc3339(last),
        ) {
            (Ok(f), Ok(l)) => {
                let dur = l.signed_duration_since(f);
                format!("{}h {}m {}s", dur.num_hours(), dur.num_minutes() % 60, dur.num_seconds() % 60)
            }
            _ => "parse error".to_string(),
        };

        (
            Some(first.clone()),
            Some(last.clone()),
            wall_clock,
        )
    } else if messages.len() == 1 {
        (
            Some(messages[0].ts.clone()),
            Some(messages[0].ts.clone()),
            "N/A".to_string(),
        )
    } else {
        (None, None, "N/A".to_string())
    };

    CostDuration {
        total_tokens,
        message_count,
        wall_clock,
        first_message_ts: first_ts,
        last_message_ts: last_ts,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_touched_files() {
        let messages = vec![
            StitchMessage {
                id: "1".into(),
                ts: "2025-01-01T00:00:00Z".into(),
                role: "assistant".into(),
                content: "I edited `src/main.rs` and `hoop-daemon/src/lib.rs`".into(),
                tokens: Some(50),
            },
            StitchMessage {
                id: "2".into(),
                ts: "2025-01-01T00:01:00Z".into(),
                role: "user".into(),
                content: "Also check src/main.rs again please".into(),
                tokens: Some(20),
            },
        ];

        let files = extract_touched_files(&messages);
        assert!(!files.is_empty());
        // src/main.rs should have mention_count == 2
        let main_rs = files.iter().find(|f| f.path.contains("main.rs"));
        assert!(main_rs.is_some());
        assert_eq!(main_rs.unwrap().mention_count, 2);
    }

    #[test]
    fn test_compute_cost_duration_no_messages() {
        let cd = compute_cost_duration(&[]);
        assert_eq!(cd.message_count, 0);
        assert_eq!(cd.total_tokens, 0);
        assert_eq!(cd.wall_clock, "N/A");
    }

    #[test]
    fn test_compute_cost_duration_with_messages() {
        let messages = vec![
            StitchMessage {
                id: "1".into(),
                ts: "2025-01-01T00:00:00Z".into(),
                role: "user".into(),
                content: "hello".into(),
                tokens: Some(10),
            },
            StitchMessage {
                id: "2".into(),
                ts: "2025-01-01T01:30:00Z".into(),
                role: "assistant".into(),
                content: "world".into(),
                tokens: Some(20),
            },
        ];
        let cd = compute_cost_duration(&messages);
        assert_eq!(cd.message_count, 2);
        assert_eq!(cd.total_tokens, 30);
        assert_eq!(cd.wall_clock, "1h 30m 0s");
        assert_eq!(cd.first_message_ts.as_deref(), Some("2025-01-01T00:00:00Z"));
        assert_eq!(cd.last_message_ts.as_deref(), Some("2025-01-01T01:30:00Z"));
    }
}
