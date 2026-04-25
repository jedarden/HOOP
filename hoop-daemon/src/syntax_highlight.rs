//! Server-side syntax highlighting via syntect.
//!
//! Highlights file content using bundled TextMate grammars and themes.
//! The SyntaxSet and ThemeSet are loaded once at first use and reused for
//! every request (loading is ~50ms; reuse is ~0ms).

use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use serde::Serialize;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::html::{styled_line_to_highlighted_html, IncludeBackground};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

// Extended syntax set from two-face (bat's language pack): includes TypeScript,
// TSX, TOML, Dockerfile, and 150+ languages beyond syntect's default bundle.
static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(two_face::syntax::extra_newlines);
static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

/// Maximum lines highlighted per request. Lines beyond this are counted but
/// not highlighted; `truncated` is set to true in that case.
const MAX_LINES: usize = 50_000;

/// Supported theme aliases exposed to the client.
///
/// Bundles included in syntect 5's default themes:
///   base16-ocean.dark / base16-ocean.light
///   base16-eighties.dark
///   base16-mocha.dark / base16-mocha.light
///   InspiredGitHub
///   Solarized (dark) / Solarized (light)
fn resolve_theme(alias: &str) -> &'static str {
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

fn color_to_css(c: syntect::highlighting::Color) -> String {
    if c.a == 255 {
        format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
    } else {
        format!("rgba({},{},{},{})", c.r, c.g, c.b, c.a as f32 / 255.0)
    }
}

/// The JSON payload returned by the content endpoint.
#[derive(Debug, Serialize)]
pub struct HighlightResult {
    /// Detected language name (e.g. "Rust", "TypeScript").
    pub language: String,
    /// Total line count in the file (may exceed `lines.len()` when truncated).
    pub line_count: usize,
    /// True when the file exceeded MAX_LINES and was truncated.
    pub truncated: bool,
    /// CSS colour for the theme background (hex or rgba).
    pub theme_bg: String,
    /// CSS colour for the theme foreground (hex or rgba).
    pub theme_fg: String,
    /// Per-line HTML fragments (inline-styled spans from syntect).
    /// Each entry corresponds to one source line; trailing newline is stripped.
    pub lines: Vec<String>,
}

/// Highlight `content` using the syntax inferred from `filename` and the
/// theme resolved from `theme_alias`.
pub fn highlight_content(
    content: &str,
    filename: &str,
    theme_alias: &str,
) -> Result<HighlightResult> {
    let ss = &*SYNTAX_SET;
    let ts = &*THEME_SET;

    let theme_name = resolve_theme(theme_alias);
    let theme = ts
        .themes
        .get(theme_name)
        .or_else(|| ts.themes.get("base16-ocean.dark"))
        .context("default theme missing from syntect bundle")?;

    // Infer syntax from filename extension; fall back to plain text.
    // Some extensions (e.g. .jsx) aren't registered in the two-face pack;
    // remap them to a known equivalent before falling back.
    let syntax = ss
        .find_syntax_for_file(filename)
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
        .unwrap_or_else(|| ss.find_syntax_plain_text());

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

    let mut h = HighlightLines::new(syntax, theme);
    let mut lines: Vec<String> = Vec::new();
    let mut total_lines: usize = 0;
    let mut truncated = false;

    for line in LinesWithEndings::from(content) {
        total_lines += 1;

        if lines.len() >= MAX_LINES {
            truncated = true;
            // Still count remaining lines; skip highlighting.
            continue;
        }

        let ranges = h
            .highlight_line(line, ss)
            .context("syntect highlight_line")?;

        let html = styled_line_to_highlighted_html(&ranges[..], IncludeBackground::No)
            .context("syntect styled_line_to_highlighted_html")?;

        // Strip trailing newline that syntect sometimes includes.
        let html = html.trim_end_matches('\n').to_owned();
        lines.push(html);
    }

    Ok(HighlightResult {
        language,
        line_count: total_lines,
        truncated,
        theme_bg,
        theme_fg,
        lines,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detect_lang(filename: &str) -> String {
        let ss = &*SYNTAX_SET;
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
        let syntax = ss
            .find_syntax_for_file(filename)
            .unwrap_or(None)
            .or_else(|| alt.and_then(|n| ss.find_syntax_for_file(n).unwrap_or(None)))
            .unwrap_or_else(|| ss.find_syntax_plain_text());
        syntax.name.clone()
    }

    #[test]
    fn required_languages_detected() {
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
            let lang = detect_lang(file);
            assert_eq!(
                &lang, expected_lang,
                "{file} detected as {lang:?}, expected {expected_lang:?}"
            );
        }
    }

    #[test]
    fn highlight_small_rust_file() {
        let code = "fn main() {\n    println!(\"hello\");\n}\n";
        let result = highlight_content(code, "main.rs", "dark").unwrap();
        assert_eq!(result.language, "Rust");
        assert_eq!(result.line_count, 3);
        assert!(!result.truncated);
        assert_eq!(result.lines.len(), 3);
        // Lines should contain HTML spans from syntect
        assert!(result.lines[0].contains("<span"), "expected HTML spans in output");
    }

    #[test]
    fn highlight_respects_theme() {
        let code = "let x = 1;\n";
        let dark = highlight_content(code, "app.ts", "dark").unwrap();
        let light = highlight_content(code, "app.ts", "light").unwrap();
        // Different themes produce different background colours
        assert_ne!(dark.theme_bg, light.theme_bg);
    }

    #[test]
    fn truncates_at_max_lines() {
        // Build a file with MAX_LINES + 10 lines
        let line = "x = 1\n";
        let content: String = line.repeat(MAX_LINES + 10);
        let result = highlight_content(&content, "script.py", "dark").unwrap();
        assert!(result.truncated);
        assert_eq!(result.lines.len(), MAX_LINES);
        assert_eq!(result.line_count, MAX_LINES + 10);
    }
}
