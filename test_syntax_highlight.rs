// Standalone test to verify syntax highlighting works
// Run with: rustc --edition 2021 test_syntax_highlight.rs --extern syntect --extern two_face -L target/debug/deps 2>&1 || cargo run --example check_syntect_langs

fn main() {
    let ss = two_face::syntax::extra_newlines();

    println!("Language detection by filename:");
    let files = [
        "main.rs",
        "app.ts",
        "app.tsx",
        "index.js",
        "index.jsx",
        "main.py",
        "main.go",
        "core.clj",
        "config.yaml",
        "config.yml",
        "Cargo.toml",
        "README.md",
        "build.sh",
        "query.sql",
        "Dockerfile",
    ];
    let mut all_found = true;
    for f in &files {
        let syntax = ss
            .find_syntax_for_file(f)
            .unwrap_or(None)
            .unwrap_or_else(|| ss.find_syntax_plain_text());
        let is_plain = syntax.name == "Plain Text";
        println!(
            "  {} -> {} {}",
            f,
            syntax.name,
            if is_plain { "⚠️ PLAIN TEXT" } else { "✓" }
        );
        if is_plain {
            all_found = false;
        }
    }
    println!("\nAll detected: {}", all_found);

    // Test highlighting
    use syntect::easy::HighlightLines;
    use syntect::highlighting::ThemeSet;
    use syntect::parsing::SyntaxSet;

    let ss = two_face::syntax::extra_newlines();
    let ts = ThemeSet::load_defaults();

    let rust_syntax = ss.find_syntax_for_file("main.rs").unwrap().unwrap();
    let theme = ts.themes.get("base16-ocean.dark").unwrap();
    let mut h = HighlightLines::new(rust_syntax, theme);

    let code = "fn main() {\n    println!(\"hello\");\n}\n";
    let lines: Vec<&str> = code.lines().collect();

    println!("\nHighlighting test:");
    for (i, line) in lines.iter().enumerate() {
        let line_with_newline = format!("{}\n", line);
        let ranges = h.highlight_line(&line_with_newline, &ss).unwrap();
        println!("  Line {}: {} spans", i + 1, ranges.len());
    }

    println!("\n✓ Syntax highlighting works!");
}
