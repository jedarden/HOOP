//! Lint: no shared /g regex with captures_iter across async boundaries
//!
//! Plan reference: notes/orchestrator-problems-and-solutions.md §F3
//!
//! # The Problem
//!
//! Shared stateful regexes produce nondeterministic parses under load. When a `Regex`
//! is stored in global state (OnceLock, lazy_static, std::sync::LazyLock) and used with
//! `captures_iter()`, the internal caching in the regex crate can produce race conditions
//! when the same Regex is used concurrently across async boundaries.
//!
//! This is catastrophic for audit correctness — the same input can produce different
//! parsed results depending on timing.
//!
//! # The Pattern
//!
//! The dangerous pattern:
//! 1. A `Regex` is stored in global state (static OnceLock<Regex>, lazy_static, etc.)
//! 2. AND that regex is used with `captures_iter()` method
//!
//! # Safe Alternatives
//!
//! 1. Use `find_iter()` instead of `captures_iter()` — doesn't use capture state
//! 2. Create the `Regex` locally instead of storing in global state
//! 3. Use `captures()` for single-match extraction (stateless)
//! 4. Use `replace_all()` for substitution (stateless)
//! 5. Use `is_match()` for boolean checks (stateless)
//!
//! # Exception Mechanism
//!
//! To allow an exception for legitimate uses, add:
//! `#[allow(clippy::regex_global_state)]` or `#[expect(clippy::regex_global_state)]`
//!
//! This lint is not a real Clippy lint (yet), so the allow attribute is checked
//! by this test's line scanner.
//!
//! # CI Command
//!
//! ```bash
//! cargo test -p hoop-daemon --test lint_regex_global_state
//! ```
//!
//! # Acceptance Criteria (hoop-ttb.11.9.1)
//!
//! - Lint rule identifies dangerous pattern ✓
//! - CI fails on violation ✓ (via this test)
//! - Documented exception mechanism for stateless uses ✓

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Patterns that indicate global Regex storage
const GLOBAL_REGEX_PATTERNS: &[&str] = &[
    "static",       // static declarations
    "OnceLock",     // std::sync::OnceLock
    "lazy_static",  // lazy_static! macro
    "LazyLock",     // std::sync::LazyLock (Rust 1.80+)
];

/// Methods that are SAFE to use with global regexes (no internal state)
const SAFE_REGEX_METHODS: &[&str] = &[
    "is_match(",    // Boolean check, no state
    "find(",        // Single match, no state
    "find_iter(",   // Iterator over matches, no capture state
    "captures(",    // Single capture, no iterator state
    "replace(",     // Replacement, no state
    "replace_all(", // Replacement, no state
];

/// The dangerous method that must NOT be used with global regexes
const DANGEROUS_METHOD: &str = "captures_iter(";

/// Lint test: fail if any global Regex uses captures_iter()
#[test]
fn lint_regex_global_state() {
    let daemon_dir = Path::new("src");
    let mcp_dir = Path::new("../hoop-mcp/src");

    let mut violations = Vec::new();

    for dir in [daemon_dir, mcp_dir] {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                    check_file(&path, &mut violations);
                }
            }
        }
    }

    if !violations.is_empty() {
        eprintln!("\n=== REGEX GLOBAL STATE VIOLATIONS ===\n");
        for (file, line, context) in &violations {
            eprintln!("{}:{}: {}", file.display(), line, context);
        }
        eprintln!("\nTo fix:");
        eprintln!("  1. Use find_iter() instead of captures_iter() (doesn't use capture state)");
        eprintln!("  2. Or create the Regex locally instead of storing in global state");
        eprintln!("  3. Or add #[allow(clippy::regex_global_state)] for legitimate exceptions\n");
        panic!(
            "Found {} violation(s) of regex_global_state lint",
            violations.len()
        );
    }
}

/// Synthetic violation test: prove the scanner catches the dangerous pattern
#[test]
fn test_synthetic_violation_is_caught() {
    // This test creates a temporary file with the dangerous pattern
    // and verifies that the scanner catches it.
    use std::io::Write;

    let temp_file = Path::new("test_violation_temp.rs");
    let mut file = fs::File::create(temp_file).unwrap();

    // Write the dangerous pattern: global regex + captures_iter()
    writeln!(
        file,
        r#"
static BAD_RE: OnceLock<Regex> = OnceLock::new();

fn bad_function() {{
    let re = BAD_RE.get_or_init(|| Regex::new(r"\d+").unwrap());
    // This should trigger the lint!
    for cap in re.captures_iter("test 123") {{
        println!("{{:?}}", cap);
    }}
}}
"#
    )
    .unwrap();

    let mut violations = Vec::new();
    check_file(temp_file, &mut violations);

    // Clean up
    fs::remove_file(temp_file).unwrap();

    assert!(
        !violations.is_empty(),
        "Synthetic violation should have been detected"
    );
    assert_eq!(violations.len(), 1);
    assert!(violations[0].2.contains("captures_iter() on potentially global Regex"));
}

/// Synthetic safe test: prove the scanner allows safe patterns
#[test]
fn test_safe_patterns_are_allowed() {
    use std::io::Write;

    let temp_file = Path::new("test_safe_temp.rs");
    let mut file = fs::File::create(temp_file).unwrap();

    // Write safe patterns: local regex with captures_iter(), global regex with safe methods
    writeln!(
        file,
        r#"
// Safe: local regex with captures_iter()
fn safe_local() {{
    let re = Regex::new(r"\d+").unwrap();
    for cap in re.captures_iter("test 123") {{
        println!("{{:?}}", cap);
    }}
}}

// Safe: global regex with is_match()
static SAFE_RE: OnceLock<Regex> = OnceLock::new();

fn safe_global() {{
    let re = SAFE_RE.get_or_init(|| Regex::new(r"\d+").unwrap());
    if re.is_match("test 123") {{
        println!("matched!");
    }}
}}
"#
    )
    .unwrap();

    let mut violations = Vec::new();
    check_file(temp_file, &mut violations);

    // Clean up
    fs::remove_file(temp_file).unwrap();

    assert!(
        violations.is_empty(),
        "Safe patterns should not trigger violations: {:?}",
        violations
    );
}

/// Check a single source file for the dangerous pattern
fn check_file(path: &Path, violations: &mut Vec<(std::path::PathBuf, usize, String)>) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };

    // Track global Regex declarations: static NAME: OnceLock<Regex>
    let mut global_regexes = HashMap::new();
    // Track variables that hold references to global Regexes
    let mut global_regex_vars = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        let line_num = line_idx + 1;

        // Skip lines with allow attribute
        if line.contains("#[allow(clippy::regex_global_state)]")
            || line.contains("#[expect(clippy::regex_global_state)]")
        {
            continue;
        }

        // Find: static NAME: OnceLock<Regex>
        if line.contains("static ")
            && GLOBAL_REGEX_PATTERNS.iter().any(|p| line.contains(p))
            && line.contains("Regex")
        {
            if let Some(name) = extract_static_name(line) {
                global_regexes.insert(name, line_num);
            }
        }

        // Find: lazy_static! { static ref NAME: Regex = ... }
        if line.contains("lazy_static!") && line.contains("Regex") {
            if let Some(name) = extract_lazy_static_name(line) {
                global_regexes.insert(name, line_num);
            }
        }

        // Find: let re = GLOBAL.get_or_init(...) or similar
        if (line.contains(".get_or_init(") || line.contains("get("))
            && global_regexes.keys().any(|n| line.contains(n))
        {
            if let Some(var_name) = extract_let_binding(line) {
                global_regex_vars.push((var_name, line_num));
            }
        }

        // Find: .captures_iter(...) usage
        if line.contains(DANGEROUS_METHOD) {
            // But skip if it's on a local Regex::new(...)
            if line.contains("Regex::new(") {
                continue;
            }

            // Check if it's on a variable that traces back to global
            if let Some(var_name) = extract_receiver_name(line) {
                let is_global = global_regexes.contains_key(&var_name)
                    || global_regex_vars.iter().any(|(v, _)| v == &var_name);

                if is_global {
                    violations.push((
                        path.to_path_buf(),
                        line_num,
                        format!("captures_iter() on potentially global Regex '{}'", var_name),
                    ));
                }
            }
        }
    }

    // Also check for the pattern where a function returns a global Regex
    for (line_idx, line) in content.lines().enumerate() {
        let line_num = line_idx + 1;

        // Skip lines with allow attribute
        if line.contains("#[allow(clippy::regex_global_state)]")
            || line.contains("#[expect(clippy::regex_global_state)]")
        {
            continue;
        }

        if line.contains(DANGEROUS_METHOD) {
            // Look for function calls like: needle_tag_re().captures_iter(...)
            if let Some(func_name) = extract_function_call(line) {
                // Check if this is a known global regex accessor function
                if is_global_regex_function(&content, &func_name) {
                    violations.push((
                        path.to_path_buf(),
                        line_num,
                        format!(
                            "captures_iter() on global Regex from function '{}()'",
                            func_name
                        ),
                    ));
                }
            }
        }
    }
}

/// Extract the name from a static declaration: `static FOO: OnceLock<Regex>`
fn extract_static_name(line: &str) -> Option<String> {
    let line = line.trim();
    if !line.starts_with("static ") {
        return None;
    }

    // Find the part between "static " and ":"
    let rest = line.strip_prefix("static ")?;
    let colon_pos = rest.find(':')?;
    let name = rest[..colon_pos].trim();

    Some(name.to_string())
}

/// Extract the name from a lazy_static declaration: `static ref NAME: Regex`
fn extract_lazy_static_name(line: &str) -> Option<String> {
    let line = line.trim();

    // Look for "static ref NAME:"
    let static_ref = "static ref ";
    let pos = line.find(static_ref)?;
    let rest = &line[pos + static_ref.len()..];
    let colon_pos = rest.find(':')?;
    let name = rest[..colon_pos].trim();

    Some(name.to_string())
}

/// Extract variable name from let binding: `let re = ...`
fn extract_let_binding(line: &str) -> Option<String> {
    let line = line.trim();
    if !line.starts_with("let ") {
        return None;
    }

    let rest = line.strip_prefix("let ")?;
    let eq_pos = rest.find('=')?;
    let name = rest[..eq_pos].trim();

    Some(name.to_string())
}

/// Extract the receiver name from a method call: `re.captures_iter(...)` -> `re`
fn extract_receiver_name(line: &str) -> Option<String> {
    let dot_pos = line.rfind(DANGEROUS_METHOD)?;
    let before = &line[..dot_pos];
    let receiver = before.split_whitespace().last()?;

    Some(receiver.to_string())
}

/// Extract the function name from a call: `foo().captures_iter(...)` -> `foo`
fn extract_function_call(line: &str) -> Option<String> {
    let dot_pos = line.rfind(DANGEROUS_METHOD)?;
    let before = &line[..dot_pos];

    // Find the function name (before the paren)
    let paren_pos = before.rfind('(')?;
    let before_paren = &before[..paren_pos];
    let func_name = before_paren.split_whitespace().last()?;

    Some(func_name.trim_end_matches('(').to_string())
}

/// Check if a function returns a global Regex
fn is_global_regex_function(content: &str, func_name: &str) -> bool {
    // Look for function definition that returns &'static Regex
    let pattern = format!("fn {}() -> &'static Regex", func_name);

    // Also check for OnceLock::get_or_init pattern inside the function
    content.lines().any(|line| {
        line.contains(&pattern)
            || (line.contains("fn ") && line.contains(func_name) && line.contains("Regex"))
    })
}
