//! Pattern view API
//!
//! `GET /api/patterns` — list all patterns with aggregate stats
//! `GET /api/patterns/:id` — single pattern detail with member stitches,
//!   parent breadcrumb, and aggregate totals

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use rusqlite::{params, Connection};
use serde::Serialize;

use crate::fleet;
use crate::{Bead, BeadStatus};

// ---------------------------------------------------------------------------
// Response types — list
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct PatternListResponse {
    pub patterns: Vec<PatternListItem>,
}

#[derive(Debug, Serialize)]
pub struct PatternListItem {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_pattern: Option<String>,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    pub member_count: usize,
    pub closed_member_count: usize,
    pub progress_percent: f64,
    pub total_tokens: i64,
}

// ---------------------------------------------------------------------------
// Response types — detail
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct PatternDetailResponse {
    pub pattern: PatternRow,
    pub parent_chain: Vec<PatternBreadcrumb>,
    pub members: Vec<PatternMemberDetail>,
    pub aggregate: PatternAggregate,
}

#[derive(Debug, Serialize, Clone)]
pub struct PatternRow {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_pattern: Option<String>,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PatternBreadcrumb {
    pub id: String,
    pub title: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct PatternMemberDetail {
    pub stitch_id: String,
    pub project: String,
    pub kind: String,
    pub title: String,
    pub created_at: String,
    pub last_activity_at: String,
    pub added_at: String,
    pub linked_beads: Vec<BeadSummary>,
    pub is_closed: bool,
    pub total_tokens: i64,
}

#[derive(Debug, Serialize)]
pub struct BeadSummary {
    pub bead_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    pub relationship: String,
}

#[derive(Debug, Serialize)]
pub struct PatternAggregate {
    pub total_members: usize,
    pub closed_members: usize,
    pub progress_percent: f64,
    pub total_tokens: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<i64>,
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<crate::DaemonState> {
    Router::new()
        .route("/api/patterns", get(list_patterns))
        .route("/api/patterns/{id}", get(get_pattern))
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn list_patterns(
    State(state): State<crate::DaemonState>,
) -> Result<Json<PatternListResponse>, (StatusCode, String)> {
    let db_path = fleet::db_path();
    let conn = Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("open fleet.db: {e}")))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, title, description, status, owner, deadline, parent_pattern, \
                    created_at, updated_at \
             FROM patterns ORDER BY created_at DESC",
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("prepare patterns: {e}")))?;

    let pattern_rows: Vec<PatternRow> = stmt
        .query_map([], |row| {
            Ok(PatternRow {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                status: row.get(3)?,
                owner: row.get(4)?,
                deadline: row.get(5)?,
                parent_pattern: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("query patterns: {e}")))?
        .filter_map(|r| r.ok())
        .collect();

    let beads_lock = state.beads.read().unwrap();

    let mut items = Vec::with_capacity(pattern_rows.len());
    for p in pattern_rows {
        let (member_count, closed_member_count, total_tokens) =
            query_pattern_stats(&conn, &p.id, &beads_lock)?;
        let progress_percent = if member_count == 0 {
            0.0
        } else {
            (closed_member_count as f64 / member_count as f64) * 100.0
        };
        items.push(PatternListItem {
            id: p.id,
            title: p.title,
            description: p.description,
            status: p.status,
            owner: p.owner,
            deadline: p.deadline,
            parent_pattern: p.parent_pattern,
            created_at: p.created_at,
            updated_at: p.updated_at,
            member_count,
            closed_member_count,
            progress_percent,
            total_tokens,
        });
    }

    drop(beads_lock);
    Ok(Json(PatternListResponse { patterns: items }))
}

async fn get_pattern(
    Path(pattern_id): Path<String>,
    State(state): State<crate::DaemonState>,
) -> Result<Json<PatternDetailResponse>, (StatusCode, String)> {
    let db_path = fleet::db_path();
    let conn = Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("open fleet.db: {e}")))?;

    let pattern = conn
        .query_row(
            "SELECT id, title, description, status, owner, deadline, parent_pattern, \
                    created_at, updated_at \
             FROM patterns WHERE id = ?1",
            params![pattern_id],
            |row| {
                Ok(PatternRow {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    status: row.get(3)?,
                    owner: row.get(4)?,
                    deadline: row.get(5)?,
                    parent_pattern: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        )
        .map_err(|e| {
            if e == rusqlite::Error::QueryReturnedNoRows {
                (StatusCode::NOT_FOUND, format!("Pattern '{}' not found", pattern_id))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
            }
        })?;

    let parent_chain = build_parent_chain(&conn, pattern.parent_pattern.as_deref())?;

    let beads_lock = state.beads.read().unwrap();
    let members = query_members(&conn, &pattern_id, &beads_lock)?;
    drop(beads_lock);

    let total_members = members.len();
    let closed_members = members.iter().filter(|m| m.is_closed).count();
    let progress_percent = if total_members == 0 {
        0.0
    } else {
        (closed_members as f64 / total_members as f64) * 100.0
    };
    let total_tokens: i64 = members.iter().map(|m| m.total_tokens).sum();
    let duration_seconds = compute_duration(&members);

    let aggregate = PatternAggregate {
        total_members,
        closed_members,
        progress_percent,
        total_tokens,
        duration_seconds,
    };

    Ok(Json(PatternDetailResponse {
        pattern,
        parent_chain,
        members,
        aggregate,
    }))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn query_pattern_stats(
    conn: &Connection,
    pattern_id: &str,
    beads: &[Bead],
) -> Result<(usize, usize, i64), (StatusCode, String)> {
    let mut stmt = conn
        .prepare(
            "SELECT pm.stitch_id, COALESCE(SUM(sm.tokens), 0) \
             FROM pattern_members pm \
             LEFT JOIN stitch_messages sm ON sm.stitch_id = pm.stitch_id \
             WHERE pm.pattern_id = ?1 \
             GROUP BY pm.stitch_id",
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("prepare stats: {e}")))?;

    let rows: Vec<(String, i64)> = stmt
        .query_map(params![pattern_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("query stats: {e}")))?
        .filter_map(|r| r.ok())
        .collect();

    let member_count = rows.len();
    let total_tokens: i64 = rows.iter().map(|(_, t)| *t).sum();

    let mut closed = 0;
    for (stitch_id, _) in &rows {
        if is_stitch_closed(conn, stitch_id, beads)? {
            closed += 1;
        }
    }

    Ok((member_count, closed, total_tokens))
}

fn is_stitch_closed(
    conn: &Connection,
    stitch_id: &str,
    beads: &[Bead],
) -> Result<bool, (StatusCode, String)> {
    let mut stmt = conn
        .prepare("SELECT bead_id FROM stitch_beads WHERE stitch_id = ?1")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("prepare is_closed: {e}")))?;

    let bead_ids: Vec<String> = stmt
        .query_map(params![stitch_id], |row| row.get::<_, String>(0))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("query is_closed: {e}")))?
        .filter_map(|r| r.ok())
        .collect();

    if bead_ids.is_empty() {
        return Ok(false);
    }

    Ok(bead_ids
        .iter()
        .any(|bid| beads.iter().any(|b| b.id == *bid && b.status == BeadStatus::Closed)))
}

fn build_parent_chain(
    conn: &Connection,
    parent_id: Option<&str>,
) -> Result<Vec<PatternBreadcrumb>, (StatusCode, String)> {
    let mut chain = Vec::new();
    let mut current = parent_id.map(|s| s.to_string());
    let mut depth = 0;

    while let Some(id) = current {
        if depth > 20 {
            break; // cycle guard (triggers should prevent this, but be defensive)
        }
        match conn.query_row(
            "SELECT id, title, status, parent_pattern FROM patterns WHERE id = ?1",
            params![id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            },
        ) {
            Ok((pid, title, status, next_parent)) => {
                chain.push(PatternBreadcrumb { id: pid, title, status });
                current = next_parent;
            }
            Err(_) => break,
        }
        depth += 1;
    }

    chain.reverse(); // root first
    Ok(chain)
}

fn query_members(
    conn: &Connection,
    pattern_id: &str,
    beads: &[Bead],
) -> Result<Vec<PatternMemberDetail>, (StatusCode, String)> {
    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.project, s.kind, s.title, s.created_at, s.last_activity_at, pm.added_at \
             FROM pattern_members pm \
             JOIN stitches s ON s.id = pm.stitch_id \
             WHERE pm.pattern_id = ?1 \
             ORDER BY s.last_activity_at DESC",
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("prepare members: {e}")))?;

    let rows: Vec<(String, String, String, String, String, String, String)> = stmt
        .query_map(params![pattern_id], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
            ))
        })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("query members: {e}")))?
        .filter_map(|r| r.ok())
        .collect();

    let mut members = Vec::with_capacity(rows.len());
    for (stitch_id, project, kind, title, created_at, last_activity_at, added_at) in rows {
        let linked_beads = query_stitch_beads(conn, &stitch_id, beads)?;
        let is_closed = linked_beads
            .iter()
            .any(|b| b.status.as_deref() == Some("closed"));

        let total_tokens: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(tokens), 0) FROM stitch_messages WHERE stitch_id = ?1",
                params![stitch_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        members.push(PatternMemberDetail {
            stitch_id,
            project,
            kind,
            title,
            created_at,
            last_activity_at,
            added_at,
            linked_beads,
            is_closed,
            total_tokens,
        });
    }

    Ok(members)
}

fn query_stitch_beads(
    conn: &Connection,
    stitch_id: &str,
    beads: &[Bead],
) -> Result<Vec<BeadSummary>, (StatusCode, String)> {
    let mut stmt = conn
        .prepare("SELECT bead_id, relationship FROM stitch_beads WHERE stitch_id = ?1")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("prepare stitch beads: {e}")))?;

    let rows: Vec<(String, String)> = stmt
        .query_map(params![stitch_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("query stitch beads: {e}")))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rows
        .into_iter()
        .map(|(bead_id, relationship)| {
            let live = beads.iter().find(|b| b.id == bead_id);
            BeadSummary {
                title: live.map(|b| b.title.clone()),
                status: live.map(|b| match b.status {
                    BeadStatus::Open => "open".to_string(),
                    BeadStatus::Closed => "closed".to_string(),
                }),
                bead_id,
                relationship,
            }
        })
        .collect())
}

fn compute_duration(members: &[PatternMemberDetail]) -> Option<i64> {
    if members.len() < 2 {
        return None;
    }
    let first = members.iter().map(|m| m.created_at.as_str()).min()?;
    let last = members.iter().map(|m| m.last_activity_at.as_str()).max()?;

    let first_dt = chrono::DateTime::parse_from_rfc3339(first)
        .or_else(|_| {
            // Try SQLite datetime format: "YYYY-MM-DD HH:MM:SS"
            chrono::NaiveDateTime::parse_from_str(first, "%Y-%m-%d %H:%M:%S")
                .map(|ndt| ndt.and_utc().fixed_offset())
        })
        .ok()?;
    let last_dt = chrono::DateTime::parse_from_rfc3339(last)
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(last, "%Y-%m-%d %H:%M:%S")
                .map(|ndt| ndt.and_utc().fixed_offset())
        })
        .ok()?;

    let secs = (last_dt.timestamp() - first_dt.timestamp()).max(0);
    if secs == 0 { None } else { Some(secs) }
}
