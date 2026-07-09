//! MCP integration for HOOP skills
//!
//! Discovers skills from `~/.hoop/skills/<name>/` and exposes them as MCP tools.
//! Validates skill arguments against the manifest's args_schema before invocation.

use anyhow::{anyhow, Result};
use chrono::Utc;
use fnv::FnvBuildHasher;
use jsonschema::Validator;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, warn};
use uuid::Uuid;

/// Skill manifest metadata (from manifest.yml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillManifest {
    /// Skill name (must match directory name)
    pub name: String,
    /// Agent-facing one-line description
    pub description: String,
    /// Human-readable summary
    pub summary: String,
    /// Visibility scope
    #[serde(default = "default_scope")]
    pub scope: SkillScope,
    /// Projects where this skill is available (when scope=project)
    #[serde(default)]
    pub projects: Vec<String>,
    /// Pattern for scope=pattern (e.g., "fix-*", "investigate:*")
    #[serde(default)]
    pub pattern: Option<String>,
    /// JSON Schema for validating invocation arguments
    #[serde(default)]
    pub args_schema: Value,
    /// Execution timeout in seconds (default: 300)
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

fn default_scope() -> SkillScope {
    SkillScope::Global
}

fn default_timeout_secs() -> u64 {
    300
}

/// Skill visibility scope
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SkillScope {
    /// Skill appears globally (available to all agents)
    Global,
    /// Skill only appears on matching projects
    Project,
    /// Skill appears when bead title/pattern matches
    Pattern,
}

/// Discovered skill entry
#[derive(Debug, Clone)]
pub struct SkillEntry {
    /// Skill name
    pub name: String,
    /// Path to run executable
    pub run_path: PathBuf,
    /// Manifest metadata
    pub manifest: SkillManifest,
    /// Whether run executable exists and has +x bit
    pub executable: bool,
}

/// Skill execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillResult {
    /// Skill name
    pub skill: String,
    /// Whether execution succeeded
    pub success: bool,
    /// Stdout output
    pub stdout: String,
    /// Stderr output
    pub stderr: String,
    /// Exit code (if available)
    pub exit_code: Option<i32>,
    /// Whether execution timed out
    pub timed_out: bool,
    /// Human-readable status message
    pub status: String,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
}

/// Validation error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error message
    pub message: String,
    /// Invalid property path (JSON Pointer)
    pub instance_path: String,
}

/// Get skills directory path
pub fn skills_dir() -> Result<PathBuf> {
    let mut path = dirs::home_dir()
        .ok_or_else(|| anyhow!("Cannot determine home directory"))?;
    path.push(".hoop");
    path.push("skills");
    Ok(path)
}

/// Discover all skills in the configured skills directory
pub fn discover_skills() -> Vec<SkillEntry> {
    let skills_dir = match skills_dir() {
        Ok(dir) => dir,
        Err(e) => {
            debug!("Failed to determine skills directory: {}", e);
            return Vec::new();
        }
    };

    let mut skills = Vec::new();

    let Ok(entries) = fs::read_dir(&skills_dir) else {
        debug!("Skills directory not readable: {}", skills_dir.display());
        return skills;
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();

        // Skip non-directories and hidden
        if !path.is_dir() || path.file_name().is_none_or(|n| n.to_string_lossy().starts_with('.')) {
            continue;
        }

        let name = path.file_name().unwrap().to_string_lossy().to_string();

        // Load manifest.yml
        let manifest_path = path.join("manifest.yml");
        let manifest = match fs::read_to_string(&manifest_path) {
            Ok(content) => match serde_yaml::from_str::<SkillManifest>(&content) {
                Ok(m) => m,
                Err(e) => {
                    warn!(
                        "Failed to parse skill manifest '{}': {}",
                        manifest_path.display(),
                        e
                    );
                    continue;
                }
            },
            Err(e) => {
                debug!(
                    "No manifest.yml found for skill '{}': {}",
                    name,
                    e
                );
                continue;
            }
        };

        // Validate manifest name matches directory name
        if manifest.name != name {
            warn!(
                "Skill manifest name '{}' does not match directory '{}', ignoring skill",
                manifest.name, name
            );
            continue;
        }

        // Check for run executable
        let run_path = path.join("run");
        let executable = run_path.exists()
            && fs::metadata(&run_path)
                .map(|m| m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false);

        skills.push(SkillEntry {
            name,
            run_path,
            manifest,
            executable,
        });
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills
}

/// Cache for compiled JSON schemas
pub struct SchemaCache {
    cache: HashMap<String, Arc<Validator>, FnvBuildHasher>,
}

impl SchemaCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::default(),
        }
    }

    pub fn get_or_compile(&mut self, skill_name: &str, schema: &Value) -> Result<Arc<Validator>> {
        if let Some(compiled) = self.cache.get(skill_name) {
            return Ok(Arc::clone(compiled));
        }

        let compiled = Validator::new(schema)
            .map_err(|e| anyhow!("Failed to compile schema for skill '{}': {}", skill_name, e))?;

        let compiled = Arc::new(compiled);
        self.cache.insert(skill_name.to_string(), Arc::clone(&compiled));
        Ok(compiled)
    }
}

impl Default for SchemaCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate arguments against a JSON Schema
pub fn validate_args(schema: &Validator, args: &Value) -> Result<(), Vec<ValidationError>> {
    // Use iter_errors to get all validation errors (not just the first one)
    let errors_iter = schema.iter_errors(args);

    // Collect all validation errors
    let validation_errors: Vec<ValidationError> = errors_iter
        .map(|e| ValidationError {
            message: e.to_string(),
            instance_path: e.instance_path.to_string(),
        })
        .collect();

    if !validation_errors.is_empty() {
        return Err(validation_errors);
    }

    Ok(())
}

/// Execute a skill with validated arguments
pub fn execute_skill(skill: &SkillEntry, args: &Value) -> Result<SkillResult> {
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::{Duration, Instant};

    let _start = Instant::now();

    debug!(
        "Executing skill: {} with args: {}",
        skill.name,
        args
    );

    let mut child = Command::new(&skill.run_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| anyhow!("Failed to spawn skill: {}", e))?;

    // Write args JSON to stdin
    if let Some(mut stdin) = child.stdin.take() {
        let args_json = serde_json::to_string(args)
            .map_err(|e| anyhow!("Failed to serialize args: {}", e))?;
        stdin
            .write_all(args_json.as_bytes())
            .map_err(|e| anyhow!("Failed to write to stdin: {}", e))?;
        stdin
            .flush()
            .map_err(|e| anyhow!("Failed to flush stdin: {}", e))?;
    }

    let timeout = Duration::from_secs(skill.manifest.timeout_secs);

    // Take stdout and stderr once
    let stdout = child.stdout.take().ok_or_else(|| anyhow!("Failed to capture stdout"))?;
    let stderr = child.stderr.take().ok_or_else(|| anyhow!("Failed to capture stderr"))?;

    use std::sync::mpsc;

    let (stdout_tx, stdout_rx) = mpsc::channel();
    let (stderr_tx, stderr_rx) = mpsc::channel();

    // Spawn thread to collect stdout
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for l in reader.lines().map_while(Result::ok) {
            let _ = stdout_tx.send(l);
        }
    });

    // Spawn thread to collect stderr
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for l in reader.lines().map_while(Result::ok) {
            let _ = stderr_tx.send(l);
        }
    });

    // Wait for completion with timeout
    let start_time = Instant::now();
    let timed_out = false;

    loop {
        let elapsed = start_time.elapsed();
        if elapsed >= timeout {
            let _ = child.kill();
            break;
        }

        match child.try_wait() {
            Ok(Some(status)) => {
                let exit_code = status.code();
                let duration_ms = elapsed.as_millis() as u64;

                // Collect remaining output with timeout
                let collect_start = Instant::now();
                let mut stdout_lines = Vec::new();
                let mut stderr_lines = Vec::new();

                while stdout_lines.len() + stderr_lines.len() < 1000
                    && collect_start.elapsed() < Duration::from_secs(1)
                {
                    let received = match stdout_rx.recv_timeout(Duration::from_millis(50)) {
                        Ok(l) => {
                            stdout_lines.push(l);
                            true
                        }
                        Err(_) => false,
                    };
                    let _ = stderr_rx.recv_timeout(Duration::from_millis(50)).ok().map(|l| {
                        stderr_lines.push(l);
                    });
                    if !received {
                        break;
                    }
                }

                // Drain any remaining
                while let Ok(line) = stdout_rx.try_recv() {
                    stdout_lines.push(line);
                }
                while let Ok(line) = stderr_rx.try_recv() {
                    stderr_lines.push(line);
                }

                let stdout = stdout_lines.join("\n");
                let stderr = stderr_lines.join("\n");
                let success = exit_code == Some(0);
                let status = if success {
                    "Skill completed successfully".to_string()
                } else {
                    format!("Skill exited with code: {:?}", exit_code)
                };

                return Ok(SkillResult {
                    skill: skill.name.clone(),
                    success,
                    stdout,
                    stderr,
                    exit_code,
                    timed_out,
                    status,
                    duration_ms,
                });
            }
            Ok(None) => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(anyhow!("Failed to wait for skill: {}", e)),
        }
    }

    // Timed out
    let stdout_lines: Vec<_> = stdout_rx.try_iter().collect();
    let stderr_lines: Vec<_> = stderr_rx.try_iter().collect();
    let duration_ms = timeout.as_millis() as u64;

    Ok(SkillResult {
        skill: skill.name.clone(),
        success: false,
        stdout: stdout_lines.join("\n"),
        stderr: stderr_lines.join("\n"),
        exit_code: None,
        timed_out: true,
        status: format!("Skill timed out after {} seconds", skill.manifest.timeout_secs),
        duration_ms,
    })
}

/// Convert skill entries to MCP tool definitions
#[cfg(test)]
pub fn skills_to_mcp_tools(skills: &[SkillEntry]) -> Vec<Value> {
    skills
        .iter()
        .filter(|s| s.executable)
        .map(|skill| {
            json!({
                "name": format!("skill_{}", skill.name),
                "description": skill.manifest.description.clone(),
                "inputSchema": skill.manifest.args_schema,
            })
        })
        .collect()
}

/// Find a skill by tool name (e.g., "skill_fetch" -> "fetch")
#[cfg(test)]
pub fn find_skill_by_tool_name<'a>(skills: &'a [SkillEntry], tool_name: &str) -> Option<&'a SkillEntry> {
    let skill_name = tool_name.strip_prefix("skill_")?;
    skills.iter().find(|s| s.name == skill_name)
}

/// Write a skill invocation audit row to fleet.db
///
/// Records skill_name, args_json, invoked_by, ts, duration_ms, and result
/// to the actions table with hash-chain integrity (§13).
pub fn write_skill_audit(
    fleet_db_path: &Path,
    skill_name: &str,
    args_json: Option<&Value>,
    invoked_by: &str,
    duration_ms: u64,
    success: bool,
    error_msg: Option<&str>,
) -> Result<()> {
    if !fleet_db_path.exists() {
        // fleet.db may not exist yet if daemon hasn't started
        warn!("fleet.db not found, skipping skill audit");
        return Ok(());
    }

    let conn = rusqlite::Connection::open(fleet_db_path)
        .map_err(|e| anyhow!("Failed to open fleet.db: {}", e))?;

    // Fetch the most recent hash_self for hash chaining
    let hash_prev: String = conn
        .query_row(
            "SELECT hash_self FROM actions ORDER BY ts DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "0000000000000000000000000000000000000000000000000000000000000000".to_string());

    // Generate audit row ID and timestamp
    let id = Uuid::new_v4().to_string();
    let ts = Utc::now().to_rfc3339();

    // Build args_json with skill_name and duration_ms included
    let mut audit_args = json!({
        "skill_name": skill_name,
        "duration_ms": duration_ms,
    });
    if let Some(args) = args_json {
        if let Some(obj) = audit_args.as_object_mut() {
            obj.insert("args".to_string(), args.clone());
        }
    }

    let args_json_str = serde_json::to_string(&audit_args)?;

    // Compute hash_self from row content (must match verify_hash_chain in fleet.rs)
    // Format: id, ts, actor, kind, target, project, args_json
    // Note: result is NOT included in hash computation (see verify_hash_chain)
    let project: Option<String> = None;
    let hash_input = format!(
        "{}{}{}{}{}{:?}{}",
        id,
        ts,
        invoked_by,
        "skill_invoked",
        skill_name,
        project,
        args_json_str
    );
    let mut hasher = Sha256::new();
    hasher.update(hash_input.as_bytes());
    let hash_self = hex::encode(hasher.finalize());

    // Insert the audit row
    conn.execute(
        r#"
        INSERT INTO actions (id, ts, actor, kind, target, project, args_json, result, error, source, stitch_id, args_hash, hash_prev, hash_self)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
        "#,
        params![
            id,
            ts,
            invoked_by,
            "skill_invoked",
            skill_name,
            Option::<String>::None,  // project
            args_json_str,
            if success { "success" } else { "failure" },
            error_msg,
            Option::<String>::None,  // source
            Option::<String>::None,  // stitch_id
            Option::<String>::None,  // args_hash
            hash_prev,
            hash_self,
        ],
    )
    .map_err(|e| anyhow!("Failed to insert skill audit row: {}", e))?;

    debug!(
        "Skill audit written: {} invoked by {} in {}ms",
        skill_name, invoked_by, duration_ms
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_scope_default() {
        let manifest = SkillManifest {
            name: "test".to_string(),
            description: "Test skill".to_string(),
            summary: "Summary".to_string(),
            scope: default_scope(),
            projects: Vec::new(),
            pattern: None,
            args_schema: json!({"type": "object"}),
            timeout_secs: default_timeout_secs(),
        };
        assert_eq!(manifest.scope, SkillScope::Global);
        assert_eq!(manifest.timeout_secs, 300);
    }

    #[test]
    fn test_validate_args_valid() {
        let schema = Validator::new(&json!({
            "type": "object",
            "properties": {
                "url": {"type": "string"},
                "count": {"type": "number"}
            },
            "required": ["url"]
        })).unwrap();

        let args = json!({
            "url": "https://example.com",
            "count": 42
        });

        assert!(validate_args(&schema, &args).is_ok());
    }

    #[test]
    fn test_validate_args_missing_required() {
        let schema = Validator::new(&json!({
            "type": "object",
            "properties": {
                "url": {"type": "string"}
            },
            "required": ["url"]
        })).unwrap();

        let args = json!({
            "count": 42
        });

        let result = validate_args(&schema, &args);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_find_skill_by_tool_name() {
        let skills = vec![
            SkillEntry {
                name: "fetch".to_string(),
                run_path: PathBuf::from("/skills/fetch/run"),
                manifest: SkillManifest {
                    name: "fetch".to_string(),
                    description: "Fetch a URL".to_string(),
                    summary: "URL fetcher".to_string(),
                    scope: SkillScope::Global,
                    projects: Vec::new(),
                    pattern: None,
                    args_schema: json!({}),
                    timeout_secs: 60,
                },
                executable: true,
            },
        ];

        assert_eq!(
            find_skill_by_tool_name(&skills, "skill_fetch").map(|s| s.name.as_str()),
            Some("fetch")
        );
        assert!(find_skill_by_tool_name(&skills, "skill_unknown").is_none());
        assert!(find_skill_by_tool_name(&skills, "fetch").is_none());
    }

    #[test]
    fn test_skills_to_mcp_tools() {
        let skills = vec![
            SkillEntry {
                name: "fetch".to_string(),
                run_path: PathBuf::from("/skills/fetch/run"),
                manifest: SkillManifest {
                    name: "fetch".to_string(),
                    description: "Fetch a URL".to_string(),
                    summary: "URL fetcher".to_string(),
                    scope: SkillScope::Global,
                    projects: Vec::new(),
                    pattern: None,
                    args_schema: json!({
                        "type": "object",
                        "properties": {
                            "url": {"type": "string"}
                        },
                        "required": ["url"]
                    }),
                    timeout_secs: 60,
                },
                executable: true,
            },
            SkillEntry {
                name: "incomplete".to_string(),
                run_path: PathBuf::from("/skills/incomplete/run"),
                manifest: SkillManifest {
                    name: "incomplete".to_string(),
                    description: "Incomplete".to_string(),
                    summary: "Summary".to_string(),
                    scope: SkillScope::Global,
                    projects: Vec::new(),
                    pattern: None,
                    args_schema: json!({}),
                    timeout_secs: 300,
                },
                executable: false,
            },
        ];

        let tools = skills_to_mcp_tools(&skills);
        assert_eq!(tools.len(), 1); // Only executable skills
        assert_eq!(tools[0]["name"], "skill_fetch");
        assert_eq!(tools[0]["description"], "Fetch a URL");
    }

    #[test]
    fn test_schema_cache() {
        let schema = json!({
            "type": "object",
            "properties": {
                "value": {"type": "string"}
            }
        });

        let mut cache = SchemaCache::new();

        // First call compiles
        let compiled1 = cache.get_or_compile("test", &schema).unwrap();
        // Second call uses cache
        let compiled2 = cache.get_or_compile("test", &schema).unwrap();

        // Same reference (Arc pointing to same allocation)
        assert!(Arc::ptr_eq(&compiled1, &compiled2));
    }

    #[test]
    fn test_skill_invocation_rejects_missing_required_arg() {
        let schema = Validator::new(&json!({
            "type": "object",
            "properties": {
                "url": {"type": "string"}
            },
            "required": ["url"]
        })).unwrap();

        // Missing required 'url' argument
        let args = json!({"count": 42});
        let result = validate_args(&schema, &args);

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
        // Error should mention the missing required property
        let error_msg = errors.iter().map(|e| e.message.as_str()).collect::<Vec<_>>().join(" ");
        assert!(error_msg.contains("url") || error_msg.contains("required"));
    }

    #[test]
    fn test_skill_invocation_rejects_wrong_type() {
        let schema = Validator::new(&json!({
            "type": "object",
            "properties": {
                "count": {"type": "number"}
            }
        })).unwrap();

        // 'count' should be a number, not a string
        let args = json!({"count": "not a number"});
        let result = validate_args(&schema, &args);

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
        // Error should mention the type mismatch
        let error_msg = errors[0].message.to_lowercase();
        assert!(error_msg.contains("type") || error_msg.contains("number") || error_msg.contains("integer"));
    }

    #[test]
    fn test_skill_invocation_accepts_valid_args() {
        let schema = Validator::new(&json!({
            "type": "object",
            "properties": {
                "url": {"type": "string"},
                "count": {"type": "number"}
            },
            "required": ["url"]
        })).unwrap();

        // Valid arguments
        let args = json!({
            "url": "https://example.com",
            "count": 42
        });
        let result = validate_args(&schema, &args);

        assert!(result.is_ok());
    }

    #[test]
    fn test_skill_invocation_rejects_invalid_schema() {
        // Test that an invalid schema in manifest is caught
        let invalid_schema = json!({
            "type": "object",
            "properties": {
                "url": {"type": "invalid-type"}  // Invalid JSON Schema type
            }
        });

        let result = Validator::new(&invalid_schema);
        // The jsonschema crate should reject this invalid schema
        assert!(result.is_err());
    }
}
