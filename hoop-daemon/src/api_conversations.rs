//! REST API endpoint for cross-project conversations listing
//!
//! Endpoints:
//! - GET /api/conversations — query conversations across all projects with filters

use crate::ws::WorkerMetadataData;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::DaemonState;

/// Query parameters for conversations list
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ConversationsQuery {
    /// Cursor for pagination (base64-encoded timestamp + id)
    pub cursor: Option<String>,
    /// Maximum number of results to return (default: 50, max: 200)
    pub limit: Option<usize>,
    /// Filter by project name
    pub project: Option<String>,
    /// Filter by provider (claude, codex, gemini, opencode, aider)
    pub provider: Option<String>,
    /// Filter by kind (worker, operator, dictated, ad-hoc)
    pub kind: Option<String>,
    /// Filter by fleet vs ad-hoc (fleet=true for worker, fleet=false for ad-hoc)
    pub fleet: Option<bool>,
    /// Search in title and cwd
    pub search: Option<String>,
    /// Date range start (ISO 8601)
    pub after: Option<String>,
    /// Date range end (ISO 8601)
    pub before: Option<String>,
    /// Sort field (created_at, updated_at, title)
    pub sort: Option<String>,
    /// Sort order (asc, desc)
    pub order: Option<String>,
}

/// Response for conversations query with cursor-based pagination
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ConversationsResponse {
    /// Conversations matching the query
    pub conversations: Vec<ConversationSummary>,
    /// Next cursor for pagination (empty if no more results)
    pub next_cursor: Option<String>,
    /// Whether more results exist
    pub has_more: bool,
}

/// Summary of a conversation for the list view
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ConversationSummary {
    /// Stable conversation ID
    pub id: String,
    /// Provider-native session ID
    pub session_id: String,
    /// Provider name (claude, codex, gemini, opencode, aider)
    pub provider: String,
    /// Session kind (worker, operator, dictated, ad-hoc)
    pub kind: String,
    /// Project name (derived from cwd or worker metadata)
    pub project: String,
    /// Working directory
    pub cwd: String,
    /// Session title
    pub title: String,
    /// Number of messages
    pub message_count: usize,
    /// Total tokens used
    pub total_tokens: i64,
    /// Creation timestamp
    pub created_at: String,
    /// Last update timestamp
    pub updated_at: String,
    /// Whether session is complete
    pub complete: bool,
    /// Worker metadata (for worker sessions)
    pub worker_metadata: Option<WorkerMetadata>,
}

/// Worker metadata for fleet sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct WorkerMetadata {
    /// Worker name
    pub worker: String,
    /// Bead ID
    pub bead: String,
    /// Strand (optional)
    pub strand: Option<String>,
}

impl From<WorkerMetadataData> for WorkerMetadata {
    fn from(w: WorkerMetadataData) -> Self {
        Self {
            worker: w.worker,
            bead: w.bead,
            strand: w.strand,
        }
    }
}

/// Internal representation for filtering/sorting
struct ConversationWithProject {
    summary: ConversationSummary,
    created_dt: DateTime<Utc>,
    updated_dt: DateTime<Utc>,
}

impl ConversationWithProject {
    fn get_sort_key(&self, sort_field: &str) -> (String, String) {
        match sort_field {
            "title" => (self.summary.title.clone(), self.summary.id.clone()),
            "created_at" => (self.created_dt.to_rfc3339(), self.summary.id.clone()),
            _ => (self.updated_dt.to_rfc3339(), self.summary.id.clone()), // default: updated_at
        }
    }
}

/// Derive project name from cwd by matching against registered project paths
fn derive_project_name(cwd: &str, project_paths: &HashMap<String, String>) -> String {
    // Try exact match first
    for (project_name, project_path) in project_paths {
        if cwd.starts_with(project_path) {
            // Check if cwd is exactly the project path or a subdirectory
            let rest = &cwd[project_path.len()..];
            if rest.is_empty() || rest.starts_with('/') {
                return project_name.clone();
            }
        }
    }

    // Fallback: try to extract project name from path
    if let Some(last_seg) = cwd.split('/').filter(|s| !s.is_empty()).last() {
        last_seg.to_string()
    } else {
        "unknown".to_string()
    }
}

pub fn router() -> Router<DaemonState> {
    Router::new().route("/api/conversations", get(list_conversations))
}

/// GET /api/conversations — query conversations across all projects
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/api/conversations",
    tag = "conversations",
    params(
        ("cursor" = Option<String>, Query, description = "Cursor for pagination (base64-encoded timestamp + id)"),
        ("limit" = Option<usize>, Query, description = "Maximum number of results to return (default: 50, max: 200)"),
        ("project" = Option<String>, Query, description = "Filter by project name"),
        ("provider" = Option<String>, Query, description = "Filter by provider (claude, codex, gemini, opencode, aider)"),
        ("kind" = Option<String>, Query, description = "Filter by kind (worker, operator, dictated, ad-hoc)"),
        ("fleet" = Option<bool>, Query, description = "Filter by fleet vs ad-hoc (fleet=true for worker, fleet=false for ad-hoc)"),
        ("search" = Option<String>, Query, description = "Search in title and cwd"),
        ("after" = Option<String>, Query, description = "Date range start (ISO 8601)"),
        ("before" = Option<String>, Query, description = "Date range end (ISO 8601)"),
        ("sort" = Option<String>, Query, description = "Sort field (created_at, updated_at, title)"),
        ("order" = Option<String>, Query, description = "Sort order (asc, desc)"),
    ),
    responses(
        (status = 200, description = "Conversations query successful", body = ConversationsResponse),
        (status = 400, description = "Invalid request parameters"),
    )
))]
async fn list_conversations(
    State(state): State<DaemonState>,
    Query(params): Query<ConversationsQuery>,
) -> Result<Json<ConversationsResponse>, (StatusCode, String)> {
    let limit = params.limit.unwrap_or(50).min(200);
    let sort_field = params.sort.as_deref().unwrap_or("updated_at");
    let descending = params.order.as_deref() != Some("asc");

    // Validate sort field
    if !matches!(sort_field, "created_at" | "updated_at" | "title") {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Invalid sort field: {}", sort_field),
        ));
    }

    // Parse date filters
    let after_dt = params
        .after
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));
    let before_dt = params
        .before
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    // Parse cursor if provided
    let cursor_ts = if let Some(ref cursor) = params.cursor {
        decode_cursor(cursor)
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid cursor: {}", e)))?
    } else {
        None
    };

    // Build project path map for deriving project names from cwd
    let project_paths = {
        let projects = state.projects.read().unwrap();
        let mut map: HashMap<String, String> = HashMap::new();
        for proj in &*projects {
            map.insert(proj.name.clone(), proj.path.clone());
        }
        map
    }; // Lock guard dropped here

    // Get all conversations from worker registry
    let conversations = state.worker_registry.conversations_snapshot().await;
    let mut all_conversations: Vec<ConversationWithProject> = Vec::new();

    for conv in conversations {
        // Apply filters
        let project_name = derive_project_name(&conv.cwd, &project_paths);

        if let Some(ref proj_filter) = params.project {
            if proj_filter != &project_name {
                continue;
            }
        }

        if let Some(ref provider_filter) = params.provider {
            if &conv.provider != provider_filter {
                continue;
            }
        }

        if let Some(ref kind_filter) = params.kind {
            if &conv.kind != kind_filter {
                continue;
            }
        }

        // Fleet filter (worker vs ad-hoc)
        if let Some(fleet_flag) = params.fleet {
            let is_fleet = conv.kind == "worker";
            if fleet_flag != is_fleet {
                continue;
            }
        }

        // Search filter
        if let Some(ref search_term) = params.search {
            let search_lower = search_term.to_lowercase();
            let title_match = conv.title.to_lowercase().contains(&search_lower);
            let cwd_match = conv.cwd.to_lowercase().contains(&search_lower);
            if !title_match && !cwd_match {
                continue;
            }
        }

        // Parse timestamps
        let created_dt = DateTime::parse_from_rfc3339(&conv.created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        let updated_dt = DateTime::parse_from_rfc3339(&conv.updated_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        // Date filters
        if let Some(after) = after_dt {
            if updated_dt < after {
                continue;
            }
        }
        if let Some(before) = before_dt {
            if updated_dt > before {
                continue;
            }
        }

        // Cursor pagination
        if let Some(cursor_ts_val) = cursor_ts {
            if descending {
                if updated_dt > cursor_ts_val {
                    continue;
                }
            } else {
                if updated_dt < cursor_ts_val {
                    continue;
                }
            }
        }

        // Build summary
        let summary = ConversationSummary {
            id: conv.id.clone(),
            session_id: conv.session_id.clone(),
            provider: conv.provider.clone(),
            kind: conv.kind.clone(),
            project: project_name,
            cwd: conv.cwd.clone(),
            title: conv.title.clone(),
            message_count: conv.messages.len(),
            total_tokens: conv.total_tokens as i64,
            created_at: conv.created_at.clone(),
            updated_at: conv.updated_at.clone(),
            complete: conv.complete,
            worker_metadata: conv.worker_metadata.map(|w| WorkerMetadata::from(w)),
        };

        all_conversations.push(ConversationWithProject {
            summary,
            created_dt,
            updated_dt,
        });
    }

    // Sort results
    if descending {
        all_conversations
            .sort_by(|a, b| b.get_sort_key(sort_field).cmp(&a.get_sort_key(sort_field)));
    } else {
        all_conversations
            .sort_by(|a, b| a.get_sort_key(sort_field).cmp(&b.get_sort_key(sort_field)));
    }

    // Apply limit
    let has_more = all_conversations.len() > limit;
    let conversations: Vec<ConversationSummary> = all_conversations
        .into_iter()
        .take(limit)
        .map(|c| c.summary)
        .collect();

    // Generate next cursor
    let next_cursor = if has_more {
        conversations
            .last()
            .map(|c| encode_cursor(&c.id, &c.updated_at))
    } else {
        None
    };

    Ok(Json(ConversationsResponse {
        conversations,
        next_cursor,
        has_more,
    }))
}

/// Encode a cursor from conversation ID and timestamp
fn encode_cursor(id: &str, updated_at: &str) -> String {
    use base64::prelude::*;
    let cursor_data = format!("{}|{}", updated_at, id);
    BASE64_URL_SAFE_NO_PAD.encode(cursor_data)
}

/// Decode a cursor to get the timestamp
fn decode_cursor(cursor: &str) -> Result<Option<DateTime<Utc>>, String> {
    use base64::prelude::*;
    let decoded = BASE64_URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|e| format!("Invalid base64: {}", e))?;
    let cursor_str = String::from_utf8(decoded).map_err(|e| format!("Invalid UTF-8: {}", e))?;
    let mut parts = cursor_str.split('|');
    let ts_str = parts.next().ok_or("Missing timestamp in cursor")?;
    DateTime::parse_from_rfc3339(ts_str)
        .map(|dt| Some(dt.with_timezone(&Utc)))
        .map_err(|e| format!("Invalid timestamp in cursor: {}", e))
}
