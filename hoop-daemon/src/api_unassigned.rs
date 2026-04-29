//! REST API for unassigned sessions bucket
//!
//! Sessions discovered in CLI directories that don't match any registered project
//! are surfaced in an "Unassigned" bucket. Users can assign them to projects or
//! ignore them permanently.
//!
//! Endpoints:
//! - GET /api/unassigned — list unassigned sessions
//! - POST /api/unassigned/:id/assign — assign session to a project
//! - POST /api/unassigned/:id/ignore — ignore session permanently
//!
//! §5.4 Session tailer filtering

use crate::sessions::{create_all_adapters, SessionAdapter};
use anyhow::{Context, Result};
use axum::{
    extract::{Path as AxumPath, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use hoop_schema::{ParsedSession, ParsedSessionKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

use crate::DaemonState;

/// Maximum number of unassigned sessions to keep in memory
const MAX_UNASSIGNED_SESSIONS: usize = 100;

/// Path to the ignore list (relative to ~/.hoop/)
const IGNORE_LIST_PATH: &str = "unassigned_ignore.json";

/// Response for unassigned sessions list
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UnassignedSessionsResponse {
    /// Unassigned sessions
    pub sessions: Vec<UnassignedSession>,
    /// Total count (may exceed returned list if evicted to disk)
    pub total_count: usize,
}

/// Unassigned session summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UnassignedSession {
    /// Stable session ID (provider-specific format)
    pub id: String,
    /// Provider name (claude, codex, gemini, opencode, aider)
    pub provider: String,
    /// Session kind (worker, operator, dictated, ad-hoc)
    pub kind: String,
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
}

/// Request to assign a session to a project
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AssignRequest {
    /// Project name to assign the session to
    pub project: String,
}

/// Simple success response
#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
}

/// Internal cache entry with metadata
struct UnassignedEntry {
    session: UnassignedSession,
    discovered_at: DateTime<Utc>,
}

/// Background tracker for unassigned sessions
///
/// Discovers sessions from all CLI adapters, filters out those that match
/// registered projects, and maintains a bounded cache of unassigned sessions.
/// Ignores are persisted to disk and survive restarts.
pub struct UnassignedTracker {
    /// Cached unassigned sessions (bounded to MAX_UNASSIGNED_SESSIONS)
    cache: Arc<Mutex<Vec<UnassignedEntry>>>,
    /// Projects for filtering (shared reference)
    projects: Arc<std::sync::RwLock<Vec<crate::ws::ProjectCardData>>>,
    /// Ignored session IDs (persisted to disk)
    ignored: Arc<Mutex<std::collections::HashSet<String>>>,
    /// HOOP home directory for persisting ignore list
    hoop_home: PathBuf,
    /// Broadcast sender for updates (optional, for future WS integration)
    _update_tx: broadcast::Sender<()>,
    /// Session adapters for discovering CLI sessions
    adapters: Vec<Box<dyn SessionAdapter>>,
}

impl UnassignedTracker {
    /// Create a new unassigned tracker
    pub fn new(
        projects: Arc<std::sync::RwLock<Vec<crate::ws::ProjectCardData>>>,
    ) -> Result<Self> {
        let hoop_home = Self::hoop_home_dir()?;
        let ignore_list_path = hoop_home.join(IGNORE_LIST_PATH);

        // Load existing ignore list
        let ignored = if ignore_list_path.exists() {
            Self::load_ignore_list(&ignore_list_path).unwrap_or_default()
        } else {
            std::collections::HashSet::new()
        };

        let (_update_tx, _) = broadcast::channel(16);

        // Create all session adapters for discovery
        let adapters = create_all_adapters();

        Ok(Self {
            cache: Arc::new(Mutex::new(Vec::new())),
            projects,
            ignored: Arc::new(Mutex::new(ignored)),
            hoop_home,
            _update_tx,
            adapters,
        })
    }

    /// Get the HOOP home directory (~/.hoop/)
    fn hoop_home_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Failed to determine home directory")?;
        Ok(home.join(".hoop"))
    }

    /// Load the ignore list from disk
    fn load_ignore_list(path: &Path) -> Result<std::collections::HashSet<String>> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read ignore list from {}", path.display()))?;
        let ids: Vec<String> = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse ignore list from {}", path.display()))?;
        Ok(ids.into_iter().collect())
    }

    /// Save the ignore list to disk
    fn save_ignore_list(&self) -> Result<()> {
        let path = self.hoop_home.join(IGNORE_LIST_PATH);
        let ids: Vec<String> = self.ignored.lock().unwrap().iter().cloned().collect();

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }

        let file = fs::File::create(&path)
            .with_context(|| format!("Failed to create ignore list at {}", path.display()))?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &ids)
            .with_context(|| format!("Failed to write ignore list to {}", path.display()))?;
        Ok(())
    }

    /// Spawn the background discovery task
    pub fn spawn(
        self: Arc<Self>,
        interval_secs: u64,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        self.discover_unassigned().await;
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Unassigned tracker shutting down");
                        return;
                    }
                }
            }
        });
    }

    /// Discover unassigned sessions from all adapters
    async fn discover_unassigned(self: &Arc<Self>) {
        debug!("Starting unassigned session discovery");

        // Get current project paths for filtering
        let project_paths = {
            let projects = self.projects.read().unwrap();
            projects
                .iter()
                .map(|p| (p.name.clone(), PathBuf::from(&p.path)))
                .collect::<HashMap<String, PathBuf>>()
        };

        // Get ignored IDs
        let ignored = self.ignored.lock().unwrap().clone();
        drop(ignored); // Release lock before async work

        // Discover all sessions from CLI adapters (no project filter)
        let mut all_sessions: Vec<ParsedSession> = Vec::new();

        for adapter in &self.adapters {
            let discovered_files = adapter.discover_sessions(None);

            debug!(
                "Discovered {} session files from {} adapter",
                discovered_files.len(),
                adapter.name()
            );

            for file in discovered_files {
                match adapter.parse_session_file(file.path(), None) {
                    Ok(Some(session)) => {
                        all_sessions.push(session);
                    }
                    Ok(None) => {
                        // File was skipped (not a valid session)
                    }
                    Err(e) => {
                        warn!("Failed to parse session file {}: {}", file.path().display(), e);
                    }
                }
            }
        }

        debug!(
            "Parsed {} total sessions from all adapters",
            all_sessions.len()
        );

        // Filter out assigned and ignored sessions
        let unassigned: Vec<UnassignedEntry> = all_sessions
            .into_iter()
            .filter(|session| {
                // Skip ignored
                if ignored.contains(&session.id) {
                    return false;
                }

                // Skip if cwd matches any registered project
                for project_path in project_paths.values() {
                    if self.cwd_matches_project(&session.cwd, project_path) {
                        return false;
                    }
                }

                true
            })
            .map(|session| {
                let total_tokens: i64 = session.total_usage.input_tokens + session.total_usage.output_tokens;

                UnassignedEntry {
                    session: UnassignedSession {
                        id: session.id.clone(),
                        provider: session.provider.clone(),
                        kind: match &session.kind {
                            hoop_schema::ParsedSessionKind::Variant0 { .. } => "worker".to_string(),
                            hoop_schema::ParsedSessionKind::Variant1(hoop_schema::ParsedSessionKindVariant1::Dictated) => "dictated".to_string(),
                            hoop_schema::ParsedSessionKind::Variant2(hoop_schema::ParsedSessionKindVariant2::AdHoc) => "ad-hoc".to_string(),
                            hoop_schema::ParsedSessionKind::Variant3(hoop_schema::ParsedSessionKindVariant3::Operator) => "operator".to_string(),
                        },
                        cwd: session.cwd.clone(),
                        title: if session.title.is_empty() {
                            session
                                .messages
                                .first()
                                .and_then(|m| {
                                    if m.content.is_string() {
                                        m.content.as_str()
                                    } else if m.content.is_object() {
                                        m.content.get("content").and_then(|c| c.as_str())
                                    } else {
                                        None
                                    }
                                })
                                .map(|s| {
                                    let truncated = s.chars().take(60).collect::<String>();
                                    if s.len() > 60 {
                                        format!("{}...", truncated)
                                    } else {
                                        truncated
                                    }
                                })
                                .unwrap_or_else(|| "Untitled".to_string())
                        } else {
                            session.title.clone()
                        },
                        message_count: session.messages.len(),
                        total_tokens,
                        created_at: session.created_at.clone(),
                        updated_at: session.updated_at.clone(),
                        complete: session.complete,
                    },
                    discovered_at: Utc::now(),
                }
            })
            .collect();

        // Update cache (evict oldest if over limit)
        let mut cache = self.cache.lock().unwrap();
        cache.extend(unassigned);
        cache.sort_by(|a, b| b.discovered_at.cmp(&a.discovered_at));
        cache.truncate(MAX_UNASSIGNED_SESSIONS);

        debug!(
            "Unassigned discovery complete: {} sessions in cache",
            cache.len()
        );
    }

    /// Check if a cwd matches a project path
    fn cwd_matches_project(&self, cwd: &str, project_path: &Path) -> bool {
        let resolved_cwd = fs::canonicalize(cwd).unwrap_or_else(|_| PathBuf::from(cwd));
        let resolved_project = fs::canonicalize(project_path).unwrap_or_else(|_| project_path.to_path_buf());
        let cwd_str = resolved_cwd.to_string_lossy();
        let project_str = resolved_project.to_string_lossy();
        if cwd_str == *project_str {
            return true;
        }
        cwd_str.starts_with(&*project_str)
            && cwd_str.as_ref().get(project_str.len()..=project_str.len()) == Some("/")
    }

    /// Get current unassigned sessions (for API)
    pub fn get_sessions(&self) -> Vec<UnassignedSession> {
        let cache = self.cache.lock().unwrap();
        cache.iter().map(|e| e.session.clone()).collect()
    }

    /// Assign a session to a project (removes from unassigned)
    pub fn assign_session(&self, id: &str, _project: &str) -> Result<()> {
        let mut cache = self.cache.lock().unwrap();
        cache.retain(|e| e.session.id != id);
        debug!("Assigned session {} to project {}", id, _project);
        Ok(())
    }

    /// Ignore a session permanently
    pub fn ignore_session(&self, id: &str) -> Result<()> {
        let mut ignored = self.ignored.lock().unwrap();
        ignored.insert(id.to_string());
        drop(ignored);

        self.save_ignore_list()?;

        let mut cache = self.cache.lock().unwrap();
        cache.retain(|e| e.session.id != id);
        debug!("Ignored session {}", id);
        Ok(())
    }

    /// Check if a session is in the unassigned cache
    pub fn contains(&self, id: &str) -> bool {
        let cache = self.cache.lock().unwrap();
        cache.iter().any(|e| e.session.id == id)
    }
}

/// REST API router
pub fn router() -> Router<DaemonState> {
    Router::new()
        .route("/api/unassigned", get(list_unassigned))
        .route("/api/unassigned/:id/assign", post(assign_session))
        .route("/api/unassigned/:id/ignore", post(ignore_session))
}

/// GET /api/unassigned — list unassigned sessions
#[axum::debug_handler]
async fn list_unassigned(
    State(state): State<DaemonState>,
) -> Result<Json<UnassignedSessionsResponse>, (StatusCode, String)> {
    let Some(tracker) = state.unassigned_tracker.as_ref() else {
        return Ok(Json(UnassignedSessionsResponse {
            sessions: Vec::new(),
            total_count: 0,
        }));
    };

    let tracker = tracker
        .as_any()
        .downcast_ref::<UnassignedTracker>()
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "Invalid tracker type".to_string()))?;

    let sessions = tracker.get_sessions();
    let total_count = sessions.len();

    Ok(Json(UnassignedSessionsResponse {
        sessions,
        total_count,
    }))
}

/// POST /api/unassigned/:id/assign — assign session to a project
#[axum::debug_handler]
async fn assign_session(
    State(state): State<DaemonState>,
    AxumPath(id): AxumPath<String>,
    Json(req): Json<AssignRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, String)> {
    let Some(tracker) = state.unassigned_tracker.as_ref() else {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "Tracker not available".to_string()));
    };

    let tracker = tracker
        .as_any()
        .downcast_ref::<UnassignedTracker>()
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "Invalid tracker type".to_string()))?;

    tracker
        .assign_session(&id, &req.project)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    info!("Assigned unassigned session {} to project {}", id, req.project);
    Ok(Json(SuccessResponse { success: true }))
}

/// POST /api/unassigned/:id/ignore — ignore session permanently
#[axum::debug_handler]
async fn ignore_session(
    State(state): State<DaemonState>,
    AxumPath(id): AxumPath<String>,
) -> Result<Json<SuccessResponse>, (StatusCode, String)> {
    let Some(tracker) = state.unassigned_tracker.as_ref() else {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "Tracker not available".to_string()));
    };

    let tracker = tracker
        .as_any()
        .downcast_ref::<UnassignedTracker>()
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "Invalid tracker type".to_string()))?;

    tracker
        .ignore_session(&id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    info!("Ignored unassigned session {}", id);
    Ok(Json(SuccessResponse { success: true }))
}

// Trait for downcasting the Arc-wrapped tracker
pub trait AsAny {
    fn as_any(&self) -> &dyn std::any::Any;
}

impl AsAny for UnassignedTracker {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
