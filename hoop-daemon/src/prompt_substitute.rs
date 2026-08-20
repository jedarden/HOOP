//! Prompt parameter substitution engine (§22.5 prompts)
//!
//! Handlebars-subset substitution with fixed safe variable set:
//!   - {{project}} — project name
//!   - {{file}} — file path
//!   - {{stitch}} — stitch ID
//!   - {{now}} — current timestamp (ISO 8601)
//!   - Plus operator-passed args (dynamic)
//!
//! Unknown variables are rejected (not left raw) to catch typos early.
//! Handles escaped braces: \{{ → literal '{{', \}} → literal '}}'

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::HashMap;

/// Fixed built-in variables that are always available
pub const BUILTIN_VARS: &[&str] = &["project", "file", "stitch", "now"];

/// Substitution context with built-in and custom variables
#[derive(Debug, Clone, Default)]
pub struct SubstitutionContext {
    /// Built-in variables
    builtins: HashMap<String, String>,
    /// Custom operator-passed variables
    custom: HashMap<String, String>,
}

impl SubstitutionContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a built-in variable (project, file, stitch, now)
    pub fn set_builtin(mut self, key: &str, value: String) -> Self {
        self.builtins.insert(key.to_string(), value);
        self
    }

    /// Set custom operator-passed variable
    pub fn set_custom(mut self, key: &str, value: String) -> Self {
        self.custom.insert(key.to_string(), value);
        self
    }

    /// Set project name
    pub fn project(mut self, project: String) -> Self {
        self.builtins.insert("project".to_string(), project);
        self
    }

    /// Set file path
    pub fn file(mut self, file: String) -> Self {
        self.builtins.insert("file".to_string(), file);
        self
    }

    /// Set stitch ID
    pub fn stitch(mut self, stitch: String) -> Self {
        self.builtins.insert("stitch".to_string(), stitch);
        self
    }

    /// Set current timestamp (defaults to now if not called)
    pub fn now(mut self, now: DateTime<Utc>) -> Self {
        self.builtins.insert("now".to_string(), now.to_rfc3339());
        self
    }

    /// Get a variable value, resolving built-ins first, then custom
    pub fn get(&self, key: &str) -> Option<String> {
        // Special case: now is computed dynamically
        if key == "now" {
            if let Some(v) = self.builtins.get("now") {
                return Some(v.clone());
            }
            return Some(Utc::now().to_rfc3339());
        }
        self.builtins
            .get(key)
            .or_else(|| self.custom.get(key))
            .cloned()
    }

    /// Check if a key is a known built-in
    pub fn is_builtin(&self, key: &str) -> bool {
        BUILTIN_VARS.contains(&key)
    }

    /// Get all known variable names (built-ins + custom)
    pub fn known_keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = BUILTIN_VARS.iter().map(|s| s.to_string()).collect();
        keys.extend(self.custom.keys().cloned());
        keys.sort();
        keys.dedup();
        keys
    }
}

/// Substitution error with context
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubstitutionError {
    /// Unknown variable in template
    UnknownVariable { name: String, position: usize },
    /// Unbalanced or malformed braces
    MalformedTemplate { message: String, position: usize },
    /// Escaping error
    EscapeError { position: usize },
}

impl std::fmt::Display for SubstitutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownVariable { name, position } => write!(
                f,
                "Unknown variable '{}' at position {}. Did you mean one of: project, file, stitch, now?",
                name, position
            ),
            Self::MalformedTemplate { message, position } => {
                write!(f, "Malformed template at position {}: {}", position, message)
            }
            Self::EscapeError { position } => {
                write!(f, "Escape sequence error at position {}", position)
            }
        }
    }
}

impl std::error::Error for SubstitutionError {}

/// Substitute variables in a template string
///
/// # Examples
///
/// ```
/// use hoop_daemon::prompt_substitute::{substitute, SubstitutionContext};
///
/// let ctx = SubstitutionContext::new()
///     .project("myproject".to_string());
///
/// let result = substitute("Working on {{project}}", &ctx).unwrap();
/// assert_eq!(result, "Working on myproject");
/// ```
pub fn substitute(template: &str, ctx: &SubstitutionContext) -> Result<String, SubstitutionError> {
    let mut result = String::with_capacity(template.len());
    let mut chars = template.char_indices().peekable();
    let mut var_start: Option<usize> = None;

    while let Some((i, c)) = chars.next() {
        match c {
            '\\' => {
                // Handle escape sequences
                match chars.next() {
                    Some((_, '{')) | Some((_, '}')) => {
                        // Escaped brace, add literal
                        result.push(chars.peek().map(|&(_, nc)| nc).unwrap());
                        chars.next();
                    }
                    Some((_, next_c)) => {
                        // Not a brace escape, keep both
                        result.push('\\');
                        result.push(next_c);
                    }
                    None => {
                        result.push('\\');
                    }
                }
            }
            '{' => {
                match chars.next() {
                    Some((_, '{')) => {
                        // Start of variable
                        var_start = Some(i);
                    }
                    Some((_, next_c)) => {
                        // Single brace, not a variable
                        result.push('{');
                        result.push(next_c);
                    }
                    None => {
                        result.push('{');
                    }
                }
            }
            '}' => {
                match chars.next() {
                    Some((_, '}')) => {
                        // End of variable
                        let start = var_start.take().ok_or_else(|| {
                            SubstitutionError::MalformedTemplate {
                                message: "Closing }} without opening {{".to_string(),
                                position: i,
                            }
                        })?;

                        // Extract variable name
                        let var_name = &result[start..];
                        if var_name.is_empty() {
                            return Err(SubstitutionError::MalformedTemplate {
                                message: "Empty variable name".to_string(),
                                position: i,
                            });
                        }

                        // Validate variable name (alphanumeric, underscore, hyphen)
                        if !var_name
                            .chars()
                            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
                        {
                            return Err(SubstitutionError::MalformedTemplate {
                                message: format!("Invalid variable name '{}'", var_name),
                                position: start,
                            });
                        }

                        // Look up value
                        match ctx.get(var_name) {
                            Some(value) => {
                                // Replace the variable name with the value
                                result.truncate(start);
                                result.push_str(&value);
                            }
                            None => {
                                return Err(SubstitutionError::UnknownVariable {
                                    name: var_name.to_string(),
                                    position: start,
                                });
                            }
                        }
                    }
                    Some((_, next_c)) => {
                        // Single brace, not a variable
                        result.push('}');
                        result.push(next_c);
                    }
                    None => {
                        result.push('}');
                    }
                }
            }
            _ => {
                if var_start.is_none() {
                    result.push(c);
                } else {
                    // Accumulate variable name characters
                    result.push(c);
                }
            }
        }
    }

    // Check for unclosed variable
    if var_start.is_some() {
        return Err(SubstitutionError::MalformedTemplate {
            message: "Unclosed {{ variable".to_string(),
            position: template.len().saturating_sub(1),
        });
    }

    Ok(result)
}

/// Substitute with JSON-value arguments (for API use)
///
/// Accepts a JSON object of custom variables and returns the substituted string.
pub fn substitute_with_args(
    template: &str,
    project: Option<&str>,
    file: Option<&str>,
    stitch: Option<&str>,
    args: &Value,
) -> Result<String, SubstitutionError> {
    let mut ctx = SubstitutionContext::new();

    if let Some(p) = project {
        ctx = ctx.project(p.to_string());
    }
    if let Some(f) = file {
        ctx = ctx.file(f.to_string());
    }
    if let Some(s) = stitch {
        ctx = ctx.stitch(s.to_string());
    }

    // Add custom args
    if let Some(obj) = args.as_object() {
        for (key, value) in obj {
            let value_str = match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Null => String::new(),
                _ => serde_json::to_string(value).unwrap_or_default(),
            };
            ctx = ctx.set_custom(key, value_str);
        }
    }

    substitute(template, &ctx)
}

/// Validate a template without substituting
///
/// Returns Ok if all variables are known, Err with the first unknown variable.
pub fn validate_template(template: &str, known_vars: &[String]) -> Result<(), SubstitutionError> {
    let mut chars = template.char_indices().peekable();
    let mut var_start: Option<usize> = None;
    let mut current_var = String::new();

    while let Some((i, c)) = chars.next() {
        match c {
            '\\' => {
                // Skip escaped characters
                chars.next();
            }
            '{' => {
                if chars.peek().map(|&(_, nc)| nc) == Some('{') {
                    chars.next(); // consume second '{'
                    var_start = Some(i);
                    current_var.clear();
                }
            }
            '}' => {
                if chars.peek().map(|&(_, nc)| nc) == Some('}') {
                    chars.next(); // consume second '}'
                    let start = var_start.take();
                    if start.is_some()
                        && !current_var.is_empty()
                        && !known_vars.contains(&current_var)
                        && !BUILTIN_VARS.contains(&current_var.as_str())
                    {
                        return Err(SubstitutionError::UnknownVariable {
                            name: current_var.clone(),
                            position: start.unwrap_or(i),
                        });
                    }
                    current_var.clear();
                }
            }
            _ => {
                if var_start.is_some() {
                    current_var.push(c);
                }
            }
        }
    }

    Ok(())
}

/// Extract all variable names from a template
pub fn extract_variables(template: &str) -> Vec<String> {
    let mut vars = Vec::new();
    let mut chars = template.char_indices().peekable();
    let mut in_var = false;
    let mut current_var = String::new();

    while let Some((_, c)) = chars.next() {
        match c {
            '\\' => {
                // Skip escaped characters
                chars.next();
            }
            '{' => {
                if chars.peek().map(|&(_, nc)| nc) == Some('{') {
                    chars.next(); // consume second '{'
                    in_var = true;
                    current_var.clear();
                }
            }
            '}' => {
                if chars.peek().map(|&(_, nc)| nc) == Some('}') {
                    chars.next(); // consume second '}'
                    if in_var && !current_var.is_empty() {
                        vars.push(current_var.clone());
                    }
                    in_var = false;
                    current_var.clear();
                }
            }
            _ => {
                if in_var {
                    current_var.push(c);
                }
            }
        }
    }

    vars.sort();
    vars.dedup();
    vars
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute_project() {
        let ctx = SubstitutionContext::new().project("myproject".to_string());
        let result = substitute("Working on {{project}}", &ctx).unwrap();
        assert_eq!(result, "Working on myproject");
    }

    #[test]
    fn test_substitute_file() {
        let ctx = SubstitutionContext::new().file("src/main.rs".to_string());
        let result = substitute("Editing {{file}}", &ctx).unwrap();
        assert_eq!(result, "Editing src/main.rs");
    }

    #[test]
    fn test_substitute_stitch() {
        let ctx = SubstitutionContext::new().stitch("stitch-123".to_string());
        let result = substitute("In {{stitch}}", &ctx).unwrap();
        assert_eq!(result, "In stitch-123");
    }

    #[test]
    fn test_substitute_now() {
        // The now variable is computed dynamically
        let ctx = SubstitutionContext::new();
        let result = substitute("Current time: {{now}}", &ctx).unwrap();
        // Should contain an ISO 8601 timestamp
        assert!(result.contains("Current time:"));
        assert!(result.chars().filter(|&c| c == ':').count() >= 2); // At least HH:MM
    }

    #[test]
    fn test_substitute_custom() {
        let ctx = SubstitutionContext::new().set_custom("url", "https://example.com".to_string());
        let result = substitute("Fetch {{url}}", &ctx).unwrap();
        assert_eq!(result, "Fetch https://example.com");
    }

    #[test]
    fn test_substitute_multiple() {
        let ctx = SubstitutionContext::new()
            .project("myproject".to_string())
            .file("src/main.rs".to_string())
            .set_custom("count", "42".to_string());
        let result =
            substitute("Project {{project}}, file {{file}}, count {{count}}", &ctx).unwrap();
        assert_eq!(result, "Project myproject, file src/main.rs, count 42");
    }

    #[test]
    fn test_substitute_unknown_variable() {
        let ctx = SubstitutionContext::new();
        let result = substitute("Unknown {{typo}}", &ctx);
        assert!(result.is_err());
        match result.unwrap_err() {
            SubstitutionError::UnknownVariable { name, .. } => {
                assert_eq!(name, "typo");
            }
            _ => panic!("Expected UnknownVariable error"),
        }
    }

    #[test]
    fn test_escaped_opening_braces() {
        let ctx = SubstitutionContext::new();
        let result = substitute(r"Literal \{{ braces", &ctx).unwrap();
        assert_eq!(result, r"Literal {{ braces");
    }

    #[test]
    fn test_escaped_closing_braces() {
        let ctx = SubstitutionContext::new();
        let result = substitute(r"Literal \}} braces", &ctx).unwrap();
        assert_eq!(result, r"Literal }} braces");
    }

    #[test]
    fn test_escaped_variable() {
        let ctx = SubstitutionContext::new().project("myproject".to_string());
        let result = substitute(r"Not a variable: \{{project}}", &ctx).unwrap();
        assert_eq!(result, r"Not a variable: {{project}}");
    }

    #[test]
    fn test_empty_variable_name() {
        let ctx = SubstitutionContext::new();
        let result = substitute("Empty {{}}", &ctx);
        assert!(result.is_err());
        match result.unwrap_err() {
            SubstitutionError::MalformedTemplate { .. } => {}
            _ => panic!("Expected MalformedTemplate error"),
        }
    }

    #[test]
    fn test_unclosed_variable() {
        let ctx = SubstitutionContext::new();
        let result = substitute("Unclosed {{var", &ctx);
        assert!(result.is_err());
        match result.unwrap_err() {
            SubstitutionError::MalformedTemplate { .. } => {}
            _ => panic!("Expected MalformedTemplate error"),
        }
    }

    #[test]
    fn test_unexpected_closing() {
        let ctx = SubstitutionContext::new();
        let result = substitute("Unexpected }} closing", &ctx);
        assert!(result.is_err());
        match result.unwrap_err() {
            SubstitutionError::MalformedTemplate { .. } => {}
            _ => panic!("Expected MalformedTemplate error"),
        }
    }

    #[test]
    fn test_invalid_variable_name() {
        let ctx = SubstitutionContext::new();
        let result = substitute("Invalid {{var with spaces}}", &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_substitute_with_args() {
        let args = json!({
            "url": "https://example.com",
            "count": 42
        });
        let result = substitute_with_args(
            "Fetch {{url}} ({{count}} times)",
            Some("myproject"),
            None,
            None,
            &args,
        )
        .unwrap();
        assert_eq!(result, "Fetch https://example.com (42 times)");
    }

    #[test]
    fn test_substitute_with_args_project() {
        let result = substitute_with_args(
            "In {{project}}",
            Some("testproject"),
            None,
            None,
            &json!({}),
        )
        .unwrap();
        assert_eq!(result, "In testproject");
    }

    #[test]
    fn test_extract_variables() {
        let vars = extract_variables("{{project}} and {{file}} and {{custom}}");
        assert_eq!(vars, vec!["custom", "file", "project"]);
    }

    #[test]
    fn test_extract_variables_duplicate() {
        let vars = extract_variables("{{project}} and {{project}} again");
        assert_eq!(vars, vec!["project"]);
    }

    #[test]
    fn test_extract_variables_none() {
        let vars = extract_variables("No variables here");
        assert!(vars.is_empty());
    }

    #[test]
    fn test_extract_variables_escaped() {
        let vars = extract_variables(r"Not a variable: \{{project}}");
        assert!(vars.is_empty());
    }

    #[test]
    fn test_validate_template_valid() {
        let known = vec![
            "project".to_string(),
            "file".to_string(),
            "custom".to_string(),
        ];
        let result = validate_template("{{project}} and {{file}} and {{custom}}", &known);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_template_unknown() {
        let known = vec!["project".to_string()];
        let result = validate_template("{{project}} and {{unknown}}", &known);
        assert!(result.is_err());
        match result.unwrap_err() {
            SubstitutionError::UnknownVariable { name, .. } => {
                assert_eq!(name, "unknown");
            }
            _ => panic!("Expected UnknownVariable error"),
        }
    }

    #[test]
    fn test_validate_template_builtin() {
        let result = validate_template("{{project}} and {{file}} and {{stitch}} and {{now}}", &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_variable_name_with_underscore() {
        let ctx = SubstitutionContext::new().set_custom("my_var", "value".to_string());
        let result = substitute("{{my_var}}", &ctx).unwrap();
        assert_eq!(result, "value");
    }

    #[test]
    fn test_variable_name_with_hyphen() {
        let ctx = SubstitutionContext::new().set_custom("my-var", "value".to_string());
        let result = substitute("{{my-var}}", &ctx).unwrap();
        assert_eq!(result, "value");
    }

    #[test]
    fn test_variable_name_with_number() {
        let ctx = SubstitutionContext::new().set_custom("var123", "value".to_string());
        let result = substitute("{{var123}}", &ctx).unwrap();
        assert_eq!(result, "value");
    }

    #[test]
    fn test_context_known_keys() {
        let ctx = SubstitutionContext::new()
            .project("p".to_string())
            .set_custom("custom1", "v1".to_string())
            .set_custom("custom2", "v2".to_string());
        let keys = ctx.known_keys();
        // Should contain built-ins + custom vars
        assert!(keys.contains(&"project".to_string()));
        assert!(keys.contains(&"file".to_string()));
        assert!(keys.contains(&"stitch".to_string()));
        assert!(keys.contains(&"now".to_string()));
        assert!(keys.contains(&"custom1".to_string()));
        assert!(keys.contains(&"custom2".to_string()));
    }

    #[test]
    fn test_backslash_at_end() {
        let ctx = SubstitutionContext::new();
        let result = substitute("Ends with backslash\\", &ctx).unwrap();
        assert_eq!(result, "Ends with backslash\\");
    }

    #[test]
    fn test_single_braces_literal() {
        let ctx = SubstitutionContext::new();
        let result = substitute("Single { braces } are literal", &ctx).unwrap();
        assert_eq!(result, "Single { braces } are literal");
    }

    #[test]
    fn test_empty_template() {
        let ctx = SubstitutionContext::new();
        let result = substitute("", &ctx).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_no_variables() {
        let ctx = SubstitutionContext::new();
        let result = substitute("Just plain text", &ctx).unwrap();
        assert_eq!(result, "Just plain text");
    }
}
