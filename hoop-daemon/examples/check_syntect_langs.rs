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
}
