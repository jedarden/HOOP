//! Streaming server-side syntax highlighting via syntect.
//!
//! Provides efficient line-by-line streaming for large files (>50KB).
//! Results are yielded as JSON chunks suitable for Server-Sent Events.

use futures_util::stream::{self, Stream, StreamExt};
use once_cell::sync::Lazy;
use serde::Serialize;
use std::sync::Arc;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::html::styled_line_to_highlighted_html;
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;

// Reuse the same syntax and theme sets from the non-streaming module
static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(two_face::syntax::extra_newlines);
static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

/// Maximum lines highlighted per request. Lines beyond this are counted but
/// not highlighted; the final chunk will have `truncated: true`.
const MAX_LINES: usize = 50_000;

/// Chunk size for streaming responses (lines per chunk).
const CHUNK_SIZE: usize = 100;

/// Threshold for using streaming vs non-streaming (50 KB).
pub const STREAMING_THRESHOLD: u64 = 50 * 1024;

/// Supported theme aliases exposed to the client.
pub fn resolve_theme(alias: &str) -> &'static str {
    match alias {
        "dark" | "ocean-dark" => "base16-ocean.dark",
        "light" | "github" => "InspiredGitHub",
        "solarized-dark" => "Solarized (dark)",
        "solarized-light" => "Solarized (light)",
        "eighties" => "base16-eighties.dark",
        "mocha-dark" => "base16-mocha.dark",
        "ocean-light" => "base16-ocean.light",
        _ => "base16-ocean.dark",
    }
}

pub fn color_to_css(c: syntect::highlighting::Color) -> String {
    if c.a == 255 {
        format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
    } else {
        format!("rgba({},{},{},{})", c.r, c.g, c.b, c.a as f32 / 255.0)
    }
}

/// Initial metadata sent at the start of the stream.
#[derive(Debug, Clone, Serialize)]
pub struct StreamHeader {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub language: String,
    pub line_count: usize,
    pub theme_bg: String,
    pub theme_fg: String,
}

/// A chunk of highlighted lines.
#[derive(Debug, Clone, Serialize)]
pub struct StreamChunk {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub lines: Vec<String>,
}

/// Final trailer sent after all chunks (indicates completion).
#[derive(Debug, Clone, Serialize)]
pub struct StreamTrailer {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub truncated: bool,
}

/// Stream item yielded by the highlighting stream.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum StreamItem {
    Header(StreamHeader),
    Chunk(StreamChunk),
    Trailer(StreamTrailer),
    Error { #[serde(rename = "type")] msg_type: String, error: String },
}

/// Resolve syntax reference for a filename, using common extension remaps.
pub fn resolve_syntax(filename: &str) -> Arc<SyntaxReference> {
    let ss = &*SYNTAX_SET;
    ss.find_syntax_for_file(filename)
        .unwrap_or(None)
        .or_else(|| {
            let ext = std::path::Path::new(filename)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            let alt = match ext {
                "jsx" => Some("index.js"),
                "mjs" | "cjs" => Some("index.js"),
                "mts" | "cts" => Some("app.ts"),
                _ => None,
            };
            alt.and_then(|name| ss.find_syntax_for_file(name).unwrap_or(None))
        })
        .map(|syntax| Arc::clone(syntax))
        .unwrap_or_else(|| Arc::new(ss.find_syntax_plain_text().clone()))
}

/// Create a stream of highlighted line chunks for a file.
///
/// The stream yields JSON-serializable items suitable for SSE:
/// - Header (once): language, line count, theme colors
/// - Chunk (repeated): batch of highlighted lines
/// - Trailer (once): completion status, truncated flag
///
/// # Arguments
/// * `content` - Full file content as string
/// * `filename` - Filename for syntax detection
/// * `theme_alias` - Theme alias (dark, light, etc.)
///
/// # Returns
/// A stream of `StreamItem` values.
pub fn highlight_stream(
    content: String,
    filename: &str,
    theme_alias: &str,
) -> impl Stream<Item = StreamItem> + Send + 'static {
    let ss = Arc::new((*SYNTAX_SET).clone());

    let theme_name = resolve_theme(theme_alias);
    let theme = Arc::new(
        THEME_SET
            .themes
            .get(theme_name)
            .or_else(|| THEME_SET.themes.get("base16-ocean.dark"))
            .expect("default theme missing from syntect bundle")
            .clone(),
    );

    let syntax = resolve_syntax(filename);
    let language = syntax.name.clone();

    let theme_bg = theme
        .settings
        .background
        .map(color_to_css)
        .unwrap_or_else(|| "#1e1e2e".to_owned());

    let theme_fg = theme
        .settings
        .foreground
        .map(color_to_css)
        .unwrap_or_else(|| "#c0c5ce".to_owned());

    let total_lines = content.lines().count();

    // Clone Arcs so they can be moved into the stream
    let theme_for_highlighter = theme.clone();
    let syntax_for_highlighter = syntax.clone();

    stream::once(async move {
        StreamItem::Header(StreamHeader {
            msg_type: "header".to_string(),
            language,
            line_count: total_lines,
            theme_bg,
            theme_fg,
        })
    })
    .chain(stream::unfold(
        (0usize, content, ss, syntax_for_highlighter, theme_for_highlighter),
        move |(line_idx, remaining_content, ss, syntax, theme)| async move {
            let mut line_idx = line_idx;

            // Recreate highlighter on each iteration (cheap operation)
            let mut highlighter = HighlightLines::new(&*syntax, &*theme);

            if line_idx >= MAX_LINES {
                return Some((
                    StreamItem::Trailer(StreamTrailer {
                        msg_type: "trailer".to_string(),
                        truncated: true,
                    }),
                    (line_idx, remaining_content, ss, syntax, theme),
                ));
            }

            let mut chunk_lines = Vec::with_capacity(CHUNK_SIZE);
            let mut bytes_consumed = 0;

            for line in LinesWithEndings::from(&remaining_content) {
                if chunk_lines.len() >= CHUNK_SIZE || line_idx >= MAX_LINES {
                    break;
                }
                line_idx += 1;

                match highlighter.highlight_line(line, &ss) {
                    Ok(ranges) => {
                        match styled_line_to_highlighted_html(&ranges[..], syntect::html::IncludeBackground::No) {
                            Ok(html) => {
                                chunk_lines.push(html.trim_end_matches('\n').to_owned());
                                bytes_consumed += line.len();
                            }
                            Err(e) => {
                                return Some((
                                    StreamItem::Error {
                                        msg_type: "error".to_string(),
                                        error: format!("highlight error: {e}"),
                                    },
                                    (line_idx, remaining_content, ss, syntax, theme),
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        return Some((
                            StreamItem::Error {
                                msg_type: "error".to_string(),
                                error: format!("highlight error: {e}"),
                            },
                            (line_idx, remaining_content, ss, syntax, theme),
                        ));
                    }
                }
            }

            if chunk_lines.is_empty() {
                return Some((
                    StreamItem::Trailer(StreamTrailer {
                        msg_type: "trailer".to_string(),
                        truncated: false,
                    }),
                    (line_idx, remaining_content, ss, syntax, theme),
                ));
            }

            // Advance past consumed bytes
            let new_remaining = remaining_content[bytes_consumed..].to_string();

            Some((
                StreamItem::Chunk(StreamChunk {
                    msg_type: "chunk".to_string(),
                    lines: chunk_lines,
                }),
                (line_idx, new_remaining, ss, syntax, theme),
            ))
        },
    ))
}

/// Convert a StreamItem to an SSE event (data: {...}\n\n).
pub fn item_to_sse(item: &StreamItem) -> String {
    let json = serde_json::to_string(item).unwrap_or_else(|_| "{}".to_string());
    format!("data: {}\n\n", json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;

    #[tokio::test]
    async fn test_stream_small_file() {
        let content = "fn main() {\n    println!(\"hello\");\n}\n";
        let mut stream = highlight_stream(content.to_string(), "main.rs", "dark");

        let first = stream.next().await.unwrap();
        match first {
            StreamItem::Header(h) => {
                assert_eq!(h.language, "Rust");
                assert_eq!(h.line_count, 3);
            }
            _ => panic!("expected header"),
        }

        let second = stream.next().await.unwrap();
        match second {
            StreamItem::Chunk(c) => {
                assert_eq!(c.lines.len(), 3);
            }
            _ => panic!("expected chunk"),
        }

        let third = stream.next().await.unwrap();
        match third {
            StreamItem::Trailer(t) => {
                assert!(!t.truncated);
            }
            _ => panic!("expected trailer"),
        }
    }

    #[tokio::test]
    async fn test_stream_large_file_chunks() {
        let line = "x = 1\n";
        let content: String = line.repeat(CHUNK_SIZE + 50);
        let mut stream = highlight_stream(content.clone(), "test.py", "dark");

        match stream.next().await.unwrap() {
            StreamItem::Header(h) => {
                assert_eq!(h.line_count, CHUNK_SIZE + 50);
            }
            _ => panic!("expected header"),
        }

        match stream.next().await.unwrap() {
            StreamItem::Chunk(c) => {
                assert_eq!(c.lines.len(), CHUNK_SIZE);
            }
            _ => panic!("expected chunk"),
        }

        match stream.next().await.unwrap() {
            StreamItem::Chunk(c) => {
                assert_eq!(c.lines.len(), 50);
            }
            _ => panic!("expected chunk"),
        }

        match stream.next().await.unwrap() {
            StreamItem::Trailer(t) => {
                assert!(!t.truncated);
            }
            _ => panic!("expected trailer"),
        }
    }

    #[test]
    fn test_required_languages_detected() {
        let cases = [
            ("main.rs", "Rust"),
            ("app.ts", "TypeScript"),
            ("app.tsx", "TypeScriptReact"),
            ("index.js", "JavaScript"),
            ("index.jsx", "JavaScript"),
            ("main.py", "Python"),
            ("main.go", "Go"),
            ("core.clj", "Clojure"),
            ("config.yaml", "YAML"),
            ("Cargo.toml", "TOML"),
            ("README.md", "Markdown"),
            ("build.sh", "Bourne Again Shell (bash)"),
            ("query.sql", "SQL"),
            ("Dockerfile", "Dockerfile"),
        ];
        for (file, expected_lang) in &cases {
            let syntax = resolve_syntax(file);
            assert_eq!(
                syntax.name, *expected_lang,
                "{file} detected as {}, expected {}",
                syntax.name, expected_lang
            );
        }
    }

    #[test]
    fn test_sse_format() {
        let item = StreamItem::Header(StreamHeader {
            msg_type: "header".to_string(),
            language: "Rust".to_string(),
            line_count: 10,
            theme_bg: "#1e1e2e".to_string(),
            theme_fg: "#c0c5ce".to_string(),
        });
        let sse = item_to_sse(&item);
        assert!(sse.starts_with("data: {"));
        assert!(sse.ends_with("\n\n"));
        assert!(sse.contains("\"Rust\""));
    }
}
