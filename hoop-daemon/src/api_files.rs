//! File browser API endpoints
//!
//! Provides directory listing, file content with syntax highlighting, and file search.
//! These endpoints power the FilesTab UI component.

use crate::files::{self, parse_ext_patterns, FileEntry, FileSearchResult, GrepMatch};
use crate::id_validators;
use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

// Syntax highlighting support
use syntect::highlighting::{Theme, ThemeSet};

/// Maximum file size for server-side syntax highlighting (50 KB)
const MAX_HIGHLIGHT_SIZE: usize = 50 * 1024;

/// Maximum lines to return from syntax highlighter (prevents OOM on huge files)
const MAX_HIGHLIGHT_LINES: usize = 50_000;

// ---------------------------------------------------------------------------
// Request/Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct ListQuery {
    /// Directory path relative to project root (empty for root)
    path: Option<String>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct ContentQuery {
    /// File path relative to project root
    path: String,
    /// Syntax highlighting theme (light, dark, solarized-dark, etc.)
    #[serde(default = "default_theme")]
    theme: String,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct SearchQuery {
    /// Extension filter (e.g. "rs" or "ts,tsx" or "*.{rs,tsx}")
    ext: Option<String>,
    /// Git ref for modified-since filter (e.g. "HEAD~1", "main")
    modified_since: Option<String>,
    /// Regex pattern for content grep
    grep: Option<String>,
}

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct HighlightResult {
    language: String,
    line_count: usize,
    truncated: bool,
    theme_bg: String,
    theme_fg: String,
    lines: Vec<String>,
}

fn default_theme() -> String {
    "light".to_string()
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<crate::DaemonState> {
    Router::new()
        .route("/api/projects/:project/files", get(list_directory))
        .route("/api/projects/:project/files/content", get(get_file_content))
        .route("/api/projects/:project/files/search", get(search_files))
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /api/projects/:project/files — list directory contents
#[utoipa::path(
    get,
    path = "/api/projects/{project}/files",
    tag = "files",
    params(
        ("project" = String, Path, description = "Project name"),
        ("path" = Option<String>, Query, description = "Directory path relative to project root (empty for root)")
    ),
    responses(
        (status = 200, description = "Directory contents", body = Vec<FileEntry>),
        (status = 400, description = "Invalid project name"),
        (status = 403, description = "Unsafe path (contains ..)"),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Failed to list directory")
    )
)]
async fn list_directory(
    Path(project): Path<String>,
    Query(params): Query<ListQuery>,
    State(state): State<crate::DaemonState>,
) -> Result<Json<Vec<FileEntry>>, (StatusCode, String)> {
    id_validators::validate_project_name(&project)
        .map_err(id_validators::rejection)?;

    let project_root = {
        let projects = state.projects.read().unwrap();
        projects
            .iter()
            .find(|p| p.name == project)
            .map(|p| PathBuf::from(&p.path))
            .ok_or((StatusCode::NOT_FOUND, "project not found".into()))?
    };

    let rel_dir = params.path.as_deref().unwrap_or("").to_string();
    if !files::is_safe_rel_path(&rel_dir) {
        return Err((StatusCode::FORBIDDEN, "unsafe path".into()));
    }

    tokio::task::spawn_blocking(move || files::list_dir(&project_root, &rel_dir))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        .map(Json)
}

/// GET /api/projects/:project/files/content — get file content with syntax highlighting
#[utoipa::path(
    get,
    path = "/api/projects/{project}/files/content",
    tag = "files",
    params(
        ("project" = String, Path, description = "Project name"),
        ("path" = String, Query, description = "File path relative to project root"),
        ("theme" = String, Query, description = "Syntax highlighting theme (light or dark)")
    ),
    responses(
        (status = 200, description = "File content with syntax highlighting", body = HighlightResult),
        (status = 400, description = "Invalid project name or unsafe path"),
        (status = 404, description = "Project or file not found")
    )
)]
async fn get_file_content(
    Path(project): Path<String>,
    Query(params): Query<ContentQuery>,
    State(state): State<crate::DaemonState>,
) -> Result<Json<HighlightResult>, (StatusCode, String)> {
    id_validators::validate_project_name(&project)
        .map_err(id_validators::rejection)?;

    let project_root = {
        let projects = state.projects.read().unwrap();
        projects
            .iter()
            .find(|p| p.name == project)
            .map(|p| PathBuf::from(&p.path))
            .ok_or((StatusCode::NOT_FOUND, "project not found".into()))?
    };

    if !files::is_safe_rel_path(&params.path) {
        return Err((StatusCode::FORBIDDEN, "unsafe path".into()));
    }

    let result = tokio::task::spawn_blocking(move || {
        highlight_file(&project_root, &params.path, &params.theme)
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", e)))?
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", e)))?;

    Ok(Json(result))
}

/// GET /api/projects/:project/files/search — search files
#[utoipa::path(
    get,
    path = "/api/projects/{project}/files/search",
    tag = "files",
    params(
        ("project" = String, Path, description = "Project name"),
        ("ext" = Option<String>, Query, description = "Extension filter (e.g. 'rs' or 'ts,tsx')"),
        ("modified_since" = Option<String>, Query, description = "Git ref for modified-since filter"),
        ("grep" = Option<String>, Query, description = "Regex pattern for content grep")
    ),
    responses(
        (status = 200, description = "Search results", body = Vec<FileSearchResult>),
        (status = 400, description = "Invalid project name"),
        (status = 404, description = "Project not found")
    )
)]
async fn search_files(
    Path(project): Path<String>,
    Query(params): Query<SearchQuery>,
    State(state): State<crate::DaemonState>,
) -> Result<Json<Vec<FileSearchResult>>, (StatusCode, String)> {
    id_validators::validate_project_name(&project)
        .map_err(id_validators::rejection)?;

    let project_root = {
        let projects = state.projects.read().unwrap();
        projects
            .iter()
            .find(|p| p.name == project)
            .map(|p| PathBuf::from(&p.path))
            .ok_or((StatusCode::NOT_FOUND, "project not found".into()))?
    };

    let ext_filter = params
        .ext
        .as_ref()
        .map(|e| parse_ext_patterns(e))
        .unwrap_or_default();

    let result = tokio::task::spawn_blocking(move || {
        files::search_files(
            &project_root,
            &ext_filter,
            params.modified_since.as_deref(),
            params.grep.as_deref(),
        )
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// Syntax highlighting (uses syntect)
// ---------------------------------------------------------------------------

/// Highlight a file's contents using syntect.
///
/// Returns HTML with inline styles for each line. For files larger than
/// MAX_HIGHLIGHT_SIZE, returns the first MAX_HIGHLIGHT_LINES lines.
fn highlight_file(
    project_root: &std::path::Path,
    rel_path: &str,
    theme_name: &str,
) -> Result<HighlightResult, (StatusCode, String)> {
    use syntect::highlighting::{Theme, ThemeSet};
    use syntect::html::{css_for_theme_with_class_style, ClassStyle};
    use syntect::parsing::{SyntaxReference, SyntaxSet};
    use syntect::util::LinesWithEndings;

    let abs_path = project_root.join(rel_path);

    // Read file content
    let content = std::fs::read_to_string(&abs_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            (StatusCode::NOT_FOUND, "file not found".into())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("read error: {}", e))
        }
    })?;

    let size = content.len();
    let line_count = content.lines().count();

    // Check if file is too large for highlighting
    let truncated = size > MAX_HIGHLIGHT_SIZE || line_count > MAX_HIGHLIGHT_LINES;

    // Detect syntax
    let ps = SyntaxSet::load_defaults_newlines();
    let syntax: &SyntaxReference = ps
        .find_syntax_for_file(&abs_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("syntax detection: {}", e)))?
        .unwrap_or_else(|| ps.find_syntax_plain_text());

    let language = syntax.name.clone();

    // Load theme
    let ts = ThemeSet::load_defaults();
    let theme: &Theme = resolve_theme(&ts, theme_name);

    // Generate CSS colors for the theme
    let theme_bg = format_color(theme.settings.background.unwrap_or(syntect::highlighting::Color::WHITE));
    let theme_fg = format_color(theme.settings.foreground.unwrap_or(syntect::highlighting::Color::BLACK));

    // Highlight the content
    let mut html_lines = Vec::new();

    // If truncated, limit to MAX_HIGHLIGHT_LINES
    let lines_to_highlight = if truncated {
        content
            .lines()
            .take(MAX_HIGHLIGHT_LINES)
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        content.clone()
    };

    // Highlight using syntect directly
    use syntect::easy::HighlightLines;
    use syntect::html::{styled_line_to_highlighted_html, IncludeBackground};

    let mut h = HighlightLines::new(syntax, theme);

    for line in LinesWithEndings::from(&lines_to_highlight) {
        let ranges = h
            .highlight_line(line, &ps)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("highlight error: {}", e)))?;

        let html = styled_line_to_highlighted_html(&ranges, IncludeBackground::Yes);
        html_lines.push(html);
    }

    Ok(HighlightResult {
        language,
        line_count,
        truncated,
        theme_bg,
        theme_fg,
        lines: html_lines,
    })
}

/// Resolve a theme name to a Theme, falling back to light if unknown
fn resolve_theme<'a>(ts: &'a ThemeSet, name: &str) -> &'a Theme {
    // Map common theme names to syntect theme names
    let syntect_name = match name {
        "light" => "GitHub",
        "dark" => "GitHub Dark",
        "solarized-dark" => "Solarized (dark)",
        "solarized-light" => "Solarized (light)",
        "eighties" => "Eighties",
        "mocha-dark" => "Mocha",
        "ocean-light" => "Ocean",
        _ => "GitHub", // default fallback
    };

    ts.themes
        .get(syntect_name)
        .unwrap_or_else(|| ts.themes.get("GitHub").unwrap())
}

/// Format a syntect Color as hex string
fn format_color(c: syntect::highlighting::Color) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
}

/// Escape HTML entities - basic sanitization for server-rendered content
fn escape_html_entities(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_color() {
        assert_eq!(format_color(syntect::highlighting::Color { r: 255, g: 0, b: 0 }), "#ff0000");
        assert_eq!(format_color(syntect::highlighting::Color { r: 0, g: 255, b: 0 }), "#00ff00");
        assert_eq!(format_color(syntect::highlighting::Color { r: 0, g: 0, b: 255 }), "#0000ff");
    }

    #[test]
    fn test_escape_html_entities() {
        assert_eq!(escape_html_entities("<div>"), "&lt;div&gt;");
        assert_eq!(escape_html_entities("a & b"), "a &amp; b");
        assert_eq!(escape_html_entities("\"quote\""), "&quot;quote&quot;");
    }

    #[test]
    fn test_default_theme() {
        assert_eq!(default_theme(), "light");
    }
}
