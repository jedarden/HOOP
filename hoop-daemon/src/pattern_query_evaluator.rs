//! Pattern saved-query evaluator
//!
//! On every Stitch creation/update, evaluate each Pattern's `pattern_queries`
//! entry against the new Stitch. On match, insert into `pattern_members`
//! idempotently. Emit `pattern.saved_query_synced` event for UI.
//!
//! ## Query DSL
//!
//! The query DSL supports:
//! - `title:regex` - Match title against regex
//! - `label:name` - Match beads with label (checks `stitch:*` labels on beads)
//! - `project:name` - Match project name exactly
//! - `kind:name` - Match kind exactly (operator, dictated, worker, ad-hoc)
//! - `AND` - All conditions must match (implicit when multiple conditions)
//! - `OR` - Any condition must match
//! - Parentheses `()` for grouping
//!
//! Examples:
//! - `title:fix.*urgent AND project:HOOP` - Title matches "fix.*urgent" AND project is "HOOP"
//! - `label:bug OR label:hotfix` - Has label "bug" OR "hotfix"
//! - `kind:operator AND (label:urgent OR label:p0)` - Operator kind AND (urgent OR p0 label)

use anyhow::{anyhow, Result};
use chrono::Utc;
use regex::Regex;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::Instant;
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

use crate::fleet;
use crate::ws::PatternSavedQuerySyncedData;

/// Slow query threshold in milliseconds
const SLOW_QUERY_THRESHOLD_MS: u128 = 100;

/// Stitch context for query evaluation
#[derive(Debug, Clone)]
pub struct StitchContext {
    pub stitch_id: String,
    pub project: String,
    pub kind: String,
    pub title: String,
    pub labels: Vec<String>,
}

/// Result of pattern query evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryEvaluationResult {
    pub pattern_id: String,
    pub matched: bool,
    pub query_duration_ms: u128,
    pub is_slow: bool,
}

/// Event emitted when a pattern's saved query is synced
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternSavedQuerySyncedEvent {
    pub pattern_id: String,
    pub stitch_id: String,
    pub query: String,
    pub matched: bool,
    pub query_duration_ms: u128,
    pub synced_at: String,
}

/// Parsed query expression
#[derive(Debug, Clone, PartialEq)]
enum QueryExpr {
    /// Title regex match
    TitleRegex(String),
    /// Label match
    Label(String),
    /// Project exact match
    Project(String),
    /// Kind exact match
    Kind(String),
    /// AND - all must match
    And(Vec<QueryExpr>),
    /// OR - any must match
    Or(Vec<QueryExpr>),
    /// Not - negation
    Not(Box<QueryExpr>),
}

/// Parse a query string into a QueryExpr
fn parse_query(query: &str) -> Result<QueryExpr> {
    let tokens = tokenize(query)?;
    let (expr, remaining) = parse_or_expr(&tokens)?;
    if !remaining.is_empty() {
        return anyhow::bail!("Unexpected tokens after query: {:?}", remaining);
    }
    Ok(expr)
}

/// Token type for query parsing
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Word(String),
    Colon,
    And,
    Or,
    Not,
    LParen,
    RParen,
}

/// Tokenize the query string
fn tokenize(query: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut chars = query.chars().peekable();
    let mut in_word = false;
    let mut current_word = String::new();

    while let Some(&c) = chars.peek() {
        match c {
            ':' => {
                if in_word {
                    tokens.push(Token::Word(current_word.clone()));
                    current_word.clear();
                    in_word = false;
                }
                tokens.push(Token::Colon);
                chars.next();
            }
            '(' => {
                if in_word {
                    tokens.push(Token::Word(current_word.clone()));
                    current_word.clear();
                    in_word = false;
                }
                tokens.push(Token::LParen);
                chars.next();
            }
            ')' => {
                if in_word {
                    tokens.push(Token::Word(current_word.clone()));
                    current_word.clear();
                    in_word = false;
                }
                tokens.push(Token::RParen);
                chars.next();
            }
            ' ' | '\t' | '\n' => {
                if in_word {
                    tokens.push(Token::Word(current_word.clone()));
                    current_word.clear();
                    in_word = false;
                }
                chars.next();
            }
            _ => {
                current_word.push(c);
                in_word = true;
                chars.next();
            }
        }
    }

    if in_word {
        tokens.push(Token::Word(current_word));
    }

    // Convert words to keywords
    let mut processed = Vec::new();
    for token in tokens {
        match token {
            Token::Word(w) if w.eq_ignore_ascii_case("AND") => processed.push(Token::And),
            Token::Word(w) if w.eq_ignore_ascii_case("OR") => processed.push(Token::Or),
            Token::Word(w) if w.eq_ignore_ascii_case("NOT") => processed.push(Token::Not),
            _ => processed.push(token),
        }
    }

    Ok(processed)
}

/// Parse OR expressions (lowest precedence)
fn parse_or_expr(tokens: &[Token]) -> Result<(QueryExpr, &[Token])> {
    let (mut expr, mut remaining) = parse_and_expr(tokens)?;

    while !remaining.is_empty() && remaining[0] == Token::Or {
        remaining = &remaining[1..];
        let (right, rest) = parse_and_expr(remaining)?;
        expr = QueryExpr::Or(vec![expr, right]);
        remaining = rest;
    }

    Ok((expr, remaining))
}

/// Parse AND expressions
fn parse_and_expr(tokens: &[Token]) -> Result<(QueryExpr, &[Token])> {
    let (mut expr, mut remaining) = parse_not_expr(tokens)?;

    while !remaining.is_empty() && remaining[0] == Token::And {
        remaining = &remaining[1..];
        let (right, rest) = parse_not_expr(remaining)?;
        expr = QueryExpr::And(vec![expr, right]);
        remaining = rest;
    }

    Ok((expr, remaining))
}

/// Parse NOT expressions
fn parse_not_expr(tokens: &[Token]) -> Result<(QueryExpr, &[Token])> {
    if !tokens.is_empty() && tokens[0] == Token::Not {
        let (expr, remaining) = parse_primary_expr(&tokens[1..])?;
        Ok((QueryExpr::Not(Box::new(expr)), remaining))
    } else {
        parse_primary_expr(tokens)
    }
}

/// Parse primary expressions (literals, parenthesized expressions)
fn parse_primary_expr(tokens: &[Token]) -> Result<(QueryExpr, &[Token])> {
    if tokens.is_empty() {
        return anyhow::bail!("Unexpected end of input");
    }

    match &tokens[0] {
        Token::LParen => {
            let (expr, remaining) = parse_or_expr(&tokens[1..])?;
            if remaining.is_empty() || remaining[0] != Token::RParen {
                return anyhow::bail!("Expected closing parenthesis");
            }
            Ok((expr, &remaining[1..]))
        }
        Token::Word(field) => {
            if tokens.len() < 2 || tokens[1] != Token::Colon {
                // Standalone word - treat as label filter for backward compatibility
                return Ok((QueryExpr::Label(field.clone()), &tokens[1..]));
            }
            if tokens.len() < 3 {
                return anyhow::bail!("Expected value after colon");
            }
            match &tokens[2] {
                Token::Word(value) => {
                    let expr = match field.as_str() {
                        "title" => QueryExpr::TitleRegex(value.clone()),
                        "label" => QueryExpr::Label(value.clone()),
                        "project" => QueryExpr::Project(value.clone()),
                        "kind" => QueryExpr::Kind(value.clone()),
                        _ => return anyhow::bail!("Unknown field: {}", field),
                    };
                    Ok((expr, &tokens[3..]))
                }
                _ => anyhow::bail!("Expected word value after colon"),
            }
        }
        _ => anyhow::bail!("Unexpected token: {:?}", tokens[0]),
    }
}

/// Evaluate a query expression against a stitch context
fn evaluate_query(expr: &QueryExpr, ctx: &StitchContext) -> Result<bool> {
    match expr {
        QueryExpr::TitleRegex(pattern) => {
            let regex = Regex::new(pattern)
                .map_err(|e| anyhow!("Invalid regex '{}': {}", pattern, e))?;
            Ok(regex.is_match(&ctx.title))
        }
        QueryExpr::Label(label) => {
            // Check if any bead has this label (including stitch:* labels)
            Ok(ctx.labels.iter().any(|l| l == label || l == &format!("stitch:{}", label)))
        }
        QueryExpr::Project(project) => Ok(ctx.project == *project),
        QueryExpr::Kind(kind) => Ok(ctx.kind == *kind),
        QueryExpr::And(exprs) => {
            for expr in exprs {
                if !evaluate_query(expr, ctx)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        QueryExpr::Or(exprs) => {
            for expr in exprs {
                if evaluate_query(expr, ctx)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        QueryExpr::Not(expr) => Ok(!evaluate_query(expr, ctx)?),
    }
}

/// Get all labels for beads linked to a stitch
///
/// Queries the stitch_beads table and looks up bead labels via br CLI
fn get_stitch_labels(stitch_id: &str) -> Result<Vec<String>> {
    let conn = Connection::open(fleet::db_path())?;

    let mut stmt = conn.prepare(
        "SELECT workspace, bead_id FROM stitch_beads WHERE stitch_id = ?1"
    )?;

    let rows = stmt.query_map(params![stitch_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    let mut labels = Vec::new();
    for row in rows {
        let (workspace, bead_id) = row?;
        // Look up bead labels via br CLI
        match lookup_bead_labels(std::path::Path::new(&workspace), &bead_id) {
            Ok(bead_labels) => labels.extend(bead_labels),
            Err(e) => {
                debug!("Failed to look up labels for bead {}: {}", bead_id, e);
            }
        }
    }

    Ok(labels)
}

/// Look up a bead's labels via `br get --json`
fn lookup_bead_labels(project_path: &std::path::Path, bead_id: &str) -> Result<Vec<String>> {
    use crate::br_verbs::invoke_br_read;
    use crate::br_verbs::ReadVerb;

    let mut cmd = invoke_br_read(ReadVerb::Get, &[bead_id, "--json"]);
    cmd.current_dir(project_path);

    let output = cmd.output()
        .map_err(|e| anyhow!("Failed to run br get: {}", e))?;

    if !output.status.success() {
        return Err(anyhow!("br get failed for bead {}", bead_id));
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| anyhow!("Failed to parse br get output: {}", e))?;

    json.get("labels")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .ok_or_else(|| anyhow!("No labels field on bead"))
}

/// Evaluate all pattern queries against a stitch
///
/// Returns the set of pattern IDs that matched, along with evaluation results
pub fn evaluate_pattern_queries(stitch_ctx: &StitchContext) -> Result<Vec<QueryEvaluationResult>> {
    let conn = Connection::open(fleet::db_path())?;

    // Get all pattern_queries
    let mut stmt = conn.prepare(
        "SELECT pattern_id, saved_query FROM pattern_queries"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    let mut results = Vec::new();
    let mut pattern_queries: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

    for row in rows {
        let (pattern_id, query) = row?;
        pattern_queries.entry(pattern_id).or_default().push(query);
    }

    for (pattern_id, queries) in pattern_queries {
        let start = Instant::now();
        let mut matched = false;

        for query in queries {
            match parse_query(&query) {
                Ok(expr) => {
                    match evaluate_query(&expr, stitch_ctx) {
                        Ok(true) => {
                            matched = true;
                            break;
                        }
                        Ok(false) => continue,
                        Err(e) => {
                            warn!("Failed to evaluate query '{}' for pattern {}: {}", query, pattern_id, e);
                            continue;
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to parse query '{}' for pattern {}: {}", query, pattern_id, e);
                    continue;
                }
            }
        }

        let duration = start.elapsed().as_millis();
        let is_slow = duration > SLOW_QUERY_THRESHOLD_MS;

        results.push(QueryEvaluationResult {
            pattern_id: pattern_id.clone(),
            matched,
            query_duration_ms: duration,
            is_slow,
        });
    }

    Ok(results)
}

/// Insert stitch into pattern_members idempotently
///
/// Only inserts if the (pattern_id, stitch_id) pair doesn't already exist
pub fn insert_pattern_member(pattern_id: &str, stitch_id: &str) -> Result<bool> {
    let conn = Connection::open(fleet::db_path())?;

    // Check if already exists
    let exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM pattern_members WHERE pattern_id = ?1 AND stitch_id = ?2",
        params![pattern_id, stitch_id],
        |row| row.get::<_, i64>(0).map(|c| c > 0),
    )?;

    if exists {
        return Ok(false);
    }

    conn.execute(
        "INSERT INTO pattern_members (pattern_id, stitch_id) VALUES (?1, ?2)",
        params![pattern_id, stitch_id],
    )?;

    Ok(true)
}

/// Main entry point: evaluate pattern queries for a newly created/updated stitch
///
/// 1. Builds stitch context (project, kind, title, labels)
/// 2. Evaluates all pattern queries
/// 3. Inserts matches into pattern_members idempotently
/// 4. Logs slow queries
/// 5. Returns results for event emission
pub fn sync_pattern_queries_for_stitch(
    stitch_id: &str,
    project: &str,
    kind: &str,
    title: &str,
) -> Result<Vec<QueryEvaluationResult>> {
    let labels = get_stitch_labels(stitch_id)?;

    let ctx = StitchContext {
        stitch_id: stitch_id.to_string(),
        project: project.to_string(),
        kind: kind.to_string(),
        title: title.to_string(),
        labels,
    };

    let results = evaluate_pattern_queries(&ctx)?;

    let mut matched_patterns = Vec::new();
    for result in &results {
        if result.matched {
            if insert_pattern_member(&result.pattern_id, stitch_id)? {
                info!(
                    "Added stitch {} to pattern {} via saved query ({}ms)",
                    stitch_id, result.pattern_id, result.query_duration_ms
                );
                matched_patterns.push(result.pattern_id.clone());
            }
        }

        if result.is_slow {
            warn!(
                "Slow pattern query for pattern {}: {}ms",
                result.pattern_id, result.query_duration_ms
            );
        }
    }

    Ok(results)
}

/// Evaluate pattern queries for a stitch and emit events for each match
///
/// This is the main entry point called after stitch creation/update. It:
/// 1. Evaluates all pattern queries against the stitch
/// 2. Inserts matches into pattern_members idempotently
/// 3. Emits pattern.saved_query_synced events for UI updates
/// 4. Logs slow queries
///
/// Returns the number of patterns matched (for metrics/logging)
pub fn sync_and_emit_pattern_queries(
    stitch_id: &str,
    project: &str,
    kind: &str,
    title: &str,
    pattern_tx: &broadcast::Sender<PatternSavedQuerySyncedData>,
) -> Result<usize> {
    let results = sync_pattern_queries_for_stitch(stitch_id, project, kind, title)?;

    let synced_at = Utc::now().to_rfc3339();
    let mut matched_count = 0;

    for result in &results {
        if result.matched {
            // Emit pattern.saved_query_synced event for UI
            let event = PatternSavedQuerySyncedData {
                pattern_id: result.pattern_id.clone(),
                stitch_id: stitch_id.to_string(),
                project: project.to_string(),
                title: title.to_string(),
                query_duration_ms: result.query_duration_ms,
                synced_at: synced_at.clone(),
            };
            let _ = pattern_tx.send(event);
            matched_count += 1;
        }
    }

    Ok(matched_count)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let tokens = tokenize("title:fix").unwrap();
        assert_eq!(tokens, vec![
            Token::Word("title".to_string()),
            Token::Colon,
            Token::Word("fix".to_string()),
        ]);
    }

    #[test]
    fn test_tokenize_with_spaces() {
        let tokens = tokenize("title:fix AND label:urgent").unwrap();
        assert_eq!(tokens, vec![
            Token::Word("title".to_string()),
            Token::Colon,
            Token::Word("fix".to_string()),
            Token::And,
            Token::Word("label".to_string()),
            Token::Colon,
            Token::Word("urgent".to_string()),
        ]);
    }

    #[test]
    fn test_parse_title_regex() {
        let expr = parse_query("title:fix.*").unwrap();
        assert_eq!(expr, QueryExpr::TitleRegex("fix.*".to_string()));
    }

    #[test]
    fn test_parse_label_filter() {
        let expr = parse_query("label:urgent").unwrap();
        assert_eq!(expr, QueryExpr::Label("urgent".to_string()));
    }

    #[test]
    fn test_parse_and_expression() {
        let expr = parse_query("title:fix AND label:urgent").unwrap();
        assert_eq!(expr, QueryExpr::And(vec![
            QueryExpr::TitleRegex("fix".to_string()),
            QueryExpr::Label("urgent".to_string()),
        ]));
    }

    #[test]
    fn test_parse_or_expression() {
        let expr = parse_query("label:bug OR label:hotfix").unwrap();
        assert!(matches!(expr, QueryExpr::Or(_)));
    }

    #[test]
    fn test_parse_parentheses() {
        let expr = parse_query("(label:urgent OR label:p0) AND kind:operator").unwrap();
        assert!(matches!(expr, QueryExpr::And(_)));
    }

    #[test]
    fn test_evaluate_title_regex() {
        let ctx = StitchContext {
            stitch_id: "test".to_string(),
            project: "HOOP".to_string(),
            kind: "operator".to_string(),
            title: "fix the bug".to_string(),
            labels: vec![],
        };

        let expr = QueryExpr::TitleRegex("fix.*".to_string());
        assert!(evaluate_query(&expr, &ctx).unwrap());

        let expr = QueryExpr::TitleRegex("feature.*".to_string());
        assert!(!evaluate_query(&expr, &ctx).unwrap());
    }

    #[test]
    fn test_evaluate_label_match() {
        let ctx = StitchContext {
            stitch_id: "test".to_string(),
            project: "HOOP".to_string(),
            kind: "operator".to_string(),
            title: "fix the bug".to_string(),
            labels: vec!["urgent".to_string(), "bug".to_string()],
        };

        let expr = QueryExpr::Label("urgent".to_string());
        assert!(evaluate_query(&expr, &ctx).unwrap());

        let expr = QueryExpr::Label("p0".to_string());
        assert!(!evaluate_query(&expr, &ctx).unwrap());
    }

    #[test]
    fn test_evaluate_project_match() {
        let ctx = StitchContext {
            stitch_id: "test".to_string(),
            project: "HOOP".to_string(),
            kind: "operator".to_string(),
            title: "fix the bug".to_string(),
            labels: vec![],
        };

        let expr = QueryExpr::Project("HOOP".to_string());
        assert!(evaluate_query(&expr, &ctx).unwrap());

        let expr = QueryExpr::Project("OTHER".to_string());
        assert!(!evaluate_query(&expr, &ctx).unwrap());
    }

    #[test]
    fn test_evaluate_kind_match() {
        let ctx = StitchContext {
            stitch_id: "test".to_string(),
            project: "HOOP".to_string(),
            kind: "operator".to_string(),
            title: "fix the bug".to_string(),
            labels: vec![],
        };

        let expr = QueryExpr::Kind("operator".to_string());
        assert!(evaluate_query(&expr, &ctx).unwrap());

        let expr = QueryExpr::Kind("worker".to_string());
        assert!(!evaluate_query(&expr, &ctx).unwrap());
    }

    #[test]
    fn test_evaluate_and_expression() {
        let ctx = StitchContext {
            stitch_id: "test".to_string(),
            project: "HOOP".to_string(),
            kind: "operator".to_string(),
            title: "fix the bug".to_string(),
            labels: vec!["urgent".to_string()],
        };

        let expr = QueryExpr::And(vec![
            QueryExpr::TitleRegex("fix.*".to_string()),
            QueryExpr::Label("urgent".to_string()),
        ]);
        assert!(evaluate_query(&expr, &ctx).unwrap());

        let expr = QueryExpr::And(vec![
            QueryExpr::TitleRegex("feature.*".to_string()),
            QueryExpr::Label("urgent".to_string()),
        ]);
        assert!(!evaluate_query(&expr, &ctx).unwrap());
    }

    #[test]
    fn test_evaluate_or_expression() {
        let ctx = StitchContext {
            stitch_id: "test".to_string(),
            project: "HOOP".to_string(),
            kind: "operator".to_string(),
            title: "fix the bug".to_string(),
            labels: vec!["urgent".to_string()],
        };

        let expr = QueryExpr::Or(vec![
            QueryExpr::Label("p0".to_string()),
            QueryExpr::Label("urgent".to_string()),
        ]);
        assert!(evaluate_query(&expr, &ctx).unwrap());
    }

    #[test]
    fn test_evaluate_not_expression() {
        let ctx = StitchContext {
            stitch_id: "test".to_string(),
            project: "HOOP".to_string(),
            kind: "operator".to_string(),
            title: "fix the bug".to_string(),
            labels: vec!["urgent".to_string()],
        };

        let expr = QueryExpr::Not(Box::new(QueryExpr::Label("p0".to_string())));
        assert!(evaluate_query(&expr, &ctx).unwrap());

        let expr = QueryExpr::Not(Box::new(QueryExpr::Label("urgent".to_string())));
        assert!(!evaluate_query(&expr, &ctx).unwrap());
    }

    #[test]
    fn test_standalone_word_as_label() {
        let expr = parse_query("urgent").unwrap();
        assert_eq!(expr, QueryExpr::Label("urgent".to_string()));
    }
}
