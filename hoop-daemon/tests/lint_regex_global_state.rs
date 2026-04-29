//! Test lint for global Regex with captures_iter
//!
//! This test checks for the dangerous pattern where:
//! 1. A `Regex` is stored in global state (OnceLock, lazy_static)
//! 2. AND that regex is used with `captures_iter()` method
//!
//! This pattern can cause race conditions under load because `captures_iter()`
//! uses internal caching that can produce nondeterministic parses when the
//! same Regex is used concurrently across async boundaries.
//!
//! §F3: shared stateful regexes produce nondeterministic parses under load
//!
//! To allow an exception (for stateless uses), add:
//! `#[allow(clippy::regex_global_state)]` or `#[expect(clippy::regex_global_state)]`

use std::path::Path;
use std::fs;
use std::collections::HashMap;

/// Pattern to find: `captures_iter` calls on expressions that trace back to a global static
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
        panic!("Found {} violations of regex_global_state lint", violations.len());
    }
}

/// Check a single source file for the pattern
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
        if line.contains("allow(clippy::regex_global_state)")
            || line.contains("expect(clippy::regex_global_state)")
        {
            continue;
        }

        // Find: static NAME: OnceLock<Regex>
        if line.contains("static ") && line.contains("OnceLock") && line.contains("Regex") {
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
        if line.contains(".captures_iter(") {
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

        if line.contains(".captures_iter(") {
            // Look for function calls like: needle_tag_re().captures_iter(...)
            if let Some(func_name) = extract_function_call(line) {
                // Check if this is a known global regex accessor function
                if is_global_regex_function(&content, &func_name) {
                    violations.push((
                        path.to_path_buf(),
                        line_num,
                        format!("captures_iter() on global Regex from function '{}()'", func_name),
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
    let dot_pos = line.rfind(".captures_iter(")?;
    let before = &line[..dot_pos];
    let receiver = before.split_whitespace().last()?;

    Some(receiver.to_string())
}

/// Extract the function name from a call: `foo().captures_iter(...)` -> `foo`
fn extract_function_call(line: &str) -> Option<String> {
    let dot_pos = line.rfind(".captures_iter(")?;
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
        line.contains(&pattern) || (line.contains("fn ") && line.contains(func_name) && line.contains("Regex"))
    })
}
