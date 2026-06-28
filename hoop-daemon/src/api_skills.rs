//! Skills API — agent-invocable skills with manifest schema and JSON-Schema validation (§22.2)
//!
//! Skills at `~/.hoop/skills/<name>/` are executables with a `manifest.yml` that declares
//! their arguments schema. Skills are discovered and exposed as tools to agents via MCP.
//! Before invoking the skill's `run` executable, arguments are validated against the
//! declared `args_schema` in the manifest.
//!
//! ## Manifest Schema
//!
//! ```yaml
//! name: skill-name          # Required: matches directory name
//! description: One-liner    # Required: agent-facing description
//! summary: Human summary    # Required: human-readable purpose
//! scope: global             # Required: global|project|pattern
//! args_schema:              # Required: JSON Schema for arguments
//!   type: object
//!   properties:
//!     url:
//!       type: string
//!   required: ["url"]
//! ```
//!
//! ## Directory Structure
//!
//! ~/.hoop/skills/<name>/
//!   manifest.yml    # Skill manifest (required)
//!   run             # Executable invoked with args JSON on stdin (required, +x)
//!
//! ## Execution
//!
//! 1. Agent invokes skill via MCP tool
//! 2. Daemon validates args against manifest's args_schema
//! 3. If valid: runs `~/.hoop/skills/<name>/run` with args JSON on stdin
//! 4. If invalid: returns validation error without executing

use anyhow::{anyhow, Result};
use fnv::FnvBuildHasher;
use jsonschema::JSONSchema;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::net::SocketAddr;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path as StdPath, PathBuf};
use std::sync::Arc;
use tracing::{debug, info, warn};

// Axum imports for REST API
use axum::{
    extract::{ConnectInfo, Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

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
#[derive(Debug, Clone, Serialize)]
pub struct SkillEntry {
    /// Skill name
    pub name: String,
    /// Path to skill directory
    pub path: PathBuf,
    /// Path to run executable
    pub run_path: PathBuf,
    /// Manifest metadata
    pub manifest: SkillManifest,
    /// Whether run executable exists and has +x bit
    pub executable: bool,
}

/// Skill execution request
#[derive(Debug, Serialize, Deserialize)]
pub struct SkillRunRequest {
    /// Arguments to pass to the skill (validated against args_schema)
    pub args: Value,
    /// Optional project context (for scoping)
    pub project: Option<String>,
    /// Optional bead context (for pattern scoping)
    pub bead_id: Option<String>,
}

/// Skill execution response
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SkillRunResponse {
    /// Skill name
    pub skill: String,
    /// Exit code (0 = success)
    pub exit_code: Option<i32>,
    /// Whether execution timed out
    pub timed_out: bool,
    /// Stdout output
    pub stdout: String,
    /// Stderr output
    pub stderr: String,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Human-readable status message
    pub status: String,
    /// Validation errors (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_errors: Option<Vec<String>>,
}

/// Validation error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error message
    pub message: String,
    /// Invalid property path (JSON Pointer)
    pub instance_path: String,
    /// Expected schema type/constraint
    pub schema_path: String,
}

/// Discover all skills in the configured skills directory
pub fn discover_skills(skills_dir: &StdPath) -> Vec<SkillEntry> {
    let mut skills = Vec::new();

    let Ok(entries) = fs::read_dir(skills_dir) else {
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
            path,
            run_path,
            manifest,
            executable,
        });
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills
}

/// Validate arguments against a JSON Schema
pub fn validate_args_against_schema(
    args: &Value,
    schema: &Value,
) -> Result<(), Vec<ValidationError>> {
    // Compile the JSON Schema
    let compiled = match JSONSchema::compile(schema) {
        Ok(s) => s,
        Err(e) => {
            return Err(vec![ValidationError {
                message: format!("Invalid args_schema in manifest: {}", e),
                instance_path: String::new(),
                schema_path: String::new(),
            }]);
        }
    };

    // Validate the instance
    let result = compiled.validate(args);

    if let Err(errors) = result {
        let validation_errors: Vec<ValidationError> = errors
            .map(|e| ValidationError {
                message: e.to_string(),
                instance_path: e.instance_path.to_string(),
                schema_path: e.schema_path.to_string(),
            })
            .collect();

        return Err(validation_errors);
    }

    Ok(())
}

/// Execute a skill and capture its output
pub fn execute_skill(
    skill_path: &StdPath,
    args: &Value,
    timeout_secs: u64,
) -> Result<SkillRunResponse, String> {
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::{Duration, Instant};

    let start = Instant::now();

    debug!(
        "Executing skill: {} with args: {}",
        skill_path.display(),
        args
    );

    let mut child = Command::new(skill_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn skill: {}", e))?;

    // Write args JSON to stdin
    if let Some(mut stdin) = child.stdin.take() {
        let args_json = serde_json::to_string(args)
            .map_err(|e| format!("Failed to serialize args: {}", e))?;
        stdin
            .write_all(args_json.as_bytes())
            .map_err(|e| format!("Failed to write to stdin: {}", e))?;
        stdin
            .flush()
            .map_err(|e| format!("Failed to flush stdin: {}", e))?;
    }

    let timeout = Duration::from_secs(timeout_secs);

    // Take stdout and stderr once
    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

    use std::sync::mpsc;

    let (stdout_tx, stdout_rx) = mpsc::channel();
    let (stderr_tx, stderr_rx) = mpsc::channel();

    // Spawn thread to collect stdout
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(l) = line {
                let _ = stdout_tx.send(l);
            }
        }
    });

    // Spawn thread to collect stderr
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(l) = line {
                let _ = stderr_tx.send(l);
            }
        }
    });

    // Wait for completion with timeout
    let start_time = Instant::now();
    let mut timed_out = false;

    loop {
        let elapsed = start_time.elapsed();
        if elapsed >= timeout {
            let _ = child.kill();
            timed_out = true;
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

                let status = if exit_code == Some(0) {
                    "Skill completed successfully".to_string()
                } else {
                    format!("Skill exited with code: {:?}", exit_code)
                };

                return Ok(SkillRunResponse {
                    skill: skill_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    exit_code,
                    timed_out: false,
                    stdout,
                    stderr,
                    duration_ms,
                    status,
                    validation_errors: None,
                });
            }
            Ok(None) => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(format!("Failed to wait for skill: {}", e)),
        }
    }

    // Timed out
    let duration_ms = timeout.as_millis() as u64;
    let stdout_lines: Vec<_> = stdout_rx.try_iter().collect();
    let stderr_lines: Vec<_> = stderr_rx.try_iter().collect();

    Ok(SkillRunResponse {
        skill: skill_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        exit_code: None,
        timed_out: true,
        stdout: stdout_lines.join("\n"),
        stderr: stderr_lines.join("\n"),
        duration_ms,
        status: format!("Skill timed out after {} seconds", timeout_secs),
        validation_errors: None,
    })
}

/// Resolve the actor identity for audit purposes.
///
/// Per §13: identity from Tailscale whois where available,
/// falling back to the OS user running the HOOP process.
fn resolve_actor(remote_addr: Option<SocketAddr>) -> String {
    if let Some(addr) = remote_addr {
        let ip = addr.ip();
        let output = std::process::Command::new("tailscale")
            .arg("whois")
            .arg(ip.to_string())
            .output();
        if let Ok(out) = output {
            if let Ok(s) = String::from_utf8(out.stdout) {
                let user = s
                    .lines()
                    .find(|line| line.starts_with("User["))
                    .and_then(|line| line.strip_prefix("User["))
                    .and_then(|s| s.split(' ').next())
                    .unwrap_or("unknown")
                    .to_string();
                return format!("tailscale:{}", user);
            }
        }
    }
    // Fallback to OS user
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    format!("os:{}", user)
}

/// Compute SHA-256 hash of skill arguments for audit integrity
fn hash_args(args: &Value) -> String {
    let json_str = serde_json::to_string(args).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(json_str.as_bytes());
    hex::encode(hasher.finalize())
}

/// Get skills directory path
pub fn skills_dir() -> Result<PathBuf> {
    let mut path = dirs::home_dir()
        .ok_or_else(|| anyhow!("Cannot determine home directory"))?;
    path.push(".hoop");
    path.push("skills");
    Ok(path)
}

/// Cache for compiled JSON schemas
pub struct SchemaCache {
    cache: HashMap<String, Arc<JSONSchema>, FnvBuildHasher>,
}

impl SchemaCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::default(),
        }
    }

    pub fn get_or_compile(&mut self, skill_name: &str, schema: &Value) -> Result<Arc<JSONSchema>> {
        if let Some(compiled) = self.cache.get(skill_name) {
            return Ok(Arc::clone(compiled));
        }

        let compiled = JSONSchema::compile(schema)
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

/// Convert skill entries to MCP tool definitions
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs::{self, File};
    use std::os::unix::fs::PermissionsExt as UnixPermissionsExt;
    use tempfile::TempDir;

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
        let schema = json!({
            "type": "object",
            "properties": {
                "url": {"type": "string"},
                "count": {"type": "number"}
            },
            "required": ["url"]
        });

        let args = json!({
            "url": "https://example.com",
            "count": 42
        });

        assert!(validate_args_against_schema(&args, &schema).is_ok());
    }

    #[test]
    fn test_validate_args_missing_required() {
        let schema = json!({
            "type": "object",
            "properties": {
                "url": {"type": "string"}
            },
            "required": ["url"]
        });

        let args = json!({
            "count": 42
        });

        let result = validate_args_against_schema(&args, &schema);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("url") || errors[0].instance_path.contains("url"));
    }

    #[test]
    fn test_validate_args_wrong_type() {
        let schema = json!({
            "type": "object",
            "properties": {
                "count": {"type": "number"}
            }
        });

        let args = json!({
            "count": "not a number"
        });

        let result = validate_args_against_schema(&args, &schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_discover_skills_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let skills = discover_skills(temp_dir.path());
        assert!(skills.is_empty());
    }

    #[test]
    fn test_discover_skills_with_manifest() {
        let temp_dir = TempDir::new().unwrap();

        // Create skill directory
        let skill_dir = temp_dir.path().join("test-skill");
        fs::create_dir(&skill_dir).unwrap();

        // Write manifest
        let manifest_path = skill_dir.join("manifest.yml");
        let manifest = r#"
name: test-skill
description: A test skill
summary: Test summary for testing
scope: global
args_schema:
  type: object
  properties:
    input:
      type: string
"#;
        fs::write(&manifest_path, manifest).unwrap();

        let skills = discover_skills(temp_dir.path());
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "test-skill");
        assert_eq!(skills[0].manifest.description, "A test skill");
        assert!(!skills[0].executable); // No run file
    }

    #[test]
    fn test_discover_skills_executable_check() {
        let temp_dir = TempDir::new().unwrap();

        let skill_dir = temp_dir.path().join("executable-skill");
        fs::create_dir(&skill_dir).unwrap();

        let manifest_path = skill_dir.join("manifest.yml");
        fs::write(
            &manifest_path,
            r#"
name: executable-skill
description: An executable skill
summary: Test
scope: global
args_schema:
  type: object
"#,
        ).unwrap();

        let run_path = skill_dir.join("run");
        File::create(&run_path).unwrap();

        // Not executable yet
        let skills = discover_skills(temp_dir.path());
        assert_eq!(skills[0].executable, false);

        // Make executable
        let mut perms = fs::metadata(&run_path).unwrap().permissions();
        perms.set_mode(perms.mode() | 0o111);
        fs::set_permissions(&run_path, perms).unwrap();

        let skills = discover_skills(temp_dir.path());
        assert_eq!(skills[0].executable, true);
    }

    #[test]
    fn test_discover_skills_name_mismatch() {
        let temp_dir = TempDir::new().unwrap();

        let skill_dir = temp_dir.path().join("actual-name");
        fs::create_dir(&skill_dir).unwrap();

        // Manifest has different name
        let manifest_path = skill_dir.join("manifest.yml");
        fs::write(
            &manifest_path,
            r#"
name: different-name
description: Test
summary: Test
scope: global
args_schema:
  type: object
"#,
        ).unwrap();

        let skills = discover_skills(temp_dir.path());
        // Skill should be ignored due to name mismatch
        assert!(skills.is_empty());
    }

    #[test]
    fn test_skills_to_mcp_tools() {
        let skills = vec![
            SkillEntry {
                name: "fetch".to_string(),
                path: PathBuf::from("/skills/fetch"),
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
                path: PathBuf::from("/skills/incomplete"),
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
                executable: false, // Not executable
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

        // Both should validate successfully
        assert!(compiled1.validate(&json!({"input": "test"})).is_ok());
        assert!(compiled2.validate(&json!({"input": "test"})).is_ok());
    }

    #[test]
    fn test_skill_scope_serialization() {
        let global = SkillScope::Global;
        assert_eq!(serde_json::to_value(global).unwrap(), "global");

        let project = SkillScope::Project;
        assert_eq!(serde_json::to_value(project).unwrap(), "project");

        let pattern = SkillScope::Pattern;
        assert_eq!(serde_json::to_value(pattern).unwrap(), "pattern");
    }

    #[test]
    fn test_skill_manifest_deserialization() {
        let yaml = r#"
name: test-skill
description: Test description
summary: Test summary
scope: project
projects:
  - project-a
  - project-b
args_schema:
  type: object
  properties:
    input:
      type: string
timeout_secs: 120
"#;

        let manifest: SkillManifest = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(manifest.name, "test-skill");
        assert_eq!(manifest.description, "Test description");
        assert_eq!(manifest.summary, "Test summary");
        assert_eq!(manifest.scope, SkillScope::Project);
        assert_eq!(manifest.projects, vec!["project-a", "project-b"]);
        assert_eq!(manifest.timeout_secs, 120);
    }

    #[test]
    fn test_validation_error_serialization() {
        let error = ValidationError {
            message: "Required property missing".to_string(),
            instance_path: "/url".to_string(),
            schema_path: "/properties/url".to_string(),
        };

        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("Required property missing"));
        assert!(json.contains("/url"));
    }
}

// ---------------------------------------------------------------------------
// Skill Library and Store
// ---------------------------------------------------------------------------

/// Shared skill store behind an RwLock.
pub type SkillStore = Arc<std::sync::RwLock<SkillLibrary>>;

/// In-memory collection of discovered skills.
#[derive(Debug, Clone, Default)]
pub struct SkillLibrary {
    /// All discovered skills
    skills: Vec<SkillEntry>,
}

impl SkillLibrary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load all skills from the skills directory.
    pub fn load(&mut self, skills_dir: &StdPath) -> Result<()> {
        self.skills = discover_skills(skills_dir);
        Ok(())
    }

    /// Return all skills.
    pub fn list(&self) -> Vec<SkillEntry> {
        self.skills.clone()
    }

    /// Get a single skill by name.
    pub fn get(&self, name: &str) -> Option<SkillEntry> {
        self.skills.iter().find(|s| s.name == name).cloned()
    }

    /// Return only executable skills.
    pub fn executable(&self) -> Vec<SkillEntry> {
        self.skills.iter().filter(|s| s.executable).cloned().collect()
    }
}

/// Ensure the skills directory exists.
pub fn ensure_skills_dir(home: &StdPath) -> PathBuf {
    let skills_dir = home.join(".hoop").join("skills");

    if !skills_dir.exists() {
        std::fs::create_dir_all(&skills_dir).unwrap_or_else(|e| {
            warn!("Failed to create skills directory: {}", e);
        });
    }

    // Seed example skill if directory is empty
    let has_files = std::fs::read_dir(&skills_dir)
        .map(|mut d| d.next().is_some())
        .unwrap_or(false);

    if !has_files {
        seed_example_skill(&skills_dir);
    }

    skills_dir
}

fn seed_example_skill(dir: &StdPath) {
    // Seed echo skill (for testing)
    let echo_dir = dir.join("echo");
    std::fs::create_dir_all(&echo_dir).unwrap_or_else(|e| {
        warn!("Failed to create echo skill directory: {}", e);
    });

    let echo_manifest = r#"name: echo
description: Echo back the input message for testing
summary: Simple echo skill for testing the skills system
scope: global
args_schema:
  type: object
  properties:
    message:
      type: string
  required:
    - message
timeout_secs: 30
"#;

    let echo_run = r#"#!/bin/bash
INPUT=$(cat)
if command -v jq >/dev/null 2>&1; then
    MESSAGE=$(echo "$INPUT" | jq -r '.message')
    echo "{\"output\": \"$MESSAGE\"}"
else
    echo "{\"output\": \"$INPUT\"}"
fi
exit 0
"#;

    let echo_readme = r#"# Echo Skill

A simple skill that echoes back the input message. Useful for testing the skills system.

## Usage

```json
{
  "message": "Hello, HOOP!"
}
```
"#;

    crate::atomic_write::atomic_write_file_str(&echo_dir.join("manifest.yml"), echo_manifest)
        .unwrap_or_else(|e| warn!("Failed to write echo skill manifest: {}", e));
    crate::atomic_write::atomic_write_file_str(&echo_dir.join("run"), echo_run)
        .unwrap_or_else(|e| warn!("Failed to write echo skill run: {}", e));
    crate::atomic_write::atomic_write_file_str(&echo_dir.join("README.md"), echo_readme)
        .unwrap_or_else(|e| warn!("Failed to write echo skill README: {}", e));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let run_path = echo_dir.join("run");
        if let Ok(metadata) = std::fs::metadata(&run_path) {
            let mut perms = metadata.permissions();
            perms.set_mode(perms.mode() | 0o111);
            let _ = std::fs::set_permissions(&run_path, perms);
        }
    }

    info!("Seeded example skill: echo");

    // Seed lookup-git-log skill (practical example)
    let gitlog_dir = dir.join("lookup-git-log");
    std::fs::create_dir_all(&gitlog_dir).unwrap_or_else(|e| {
        warn!("Failed to create lookup-git-log skill directory: {}", e);
    });

    let gitlog_manifest = r#"name: lookup-git-log
description: Query git log history with filtering options
summary: Look up git commit history with optional filters for author, date range, file path, and commit message pattern
scope: global
args_schema:
  type: object
  properties:
    project_path:
      type: string
      description: Path to the git repository (defaults to current working directory)
    max_count:
      type: integer
      description: Maximum number of commits to return (default: 20)
      minimum: 1
      maximum: 100
    author:
      type: string
      description: Filter by author name or email
    since:
      type: string
      description: Show commits since this date (e.g., "2 weeks ago", "2024-01-01")
    path:
      type: string
      description: Filter commits affecting a specific file or directory
    grep:
      type: string
      description: Filter commits by message pattern
  required: []
timeout_secs: 60
"#;

    let gitlog_run = r#"#!/bin/bash
# lookup-git-log skill - Query git history
# Reads JSON arguments from stdin, outputs formatted results

INPUT=$(cat)

# Parse arguments with fallbacks
PROJECT_PATH=$(echo "$INPUT" | jq -r '.project_path // "."')
MAX_COUNT=$(echo "$INPUT" | jq -r '.max_count // "20"')
AUTHOR=$(echo "$INPUT" | jq -r '.author // ""')
SINCE=$(echo "$INPUT" | jq -r '.since // ""')
PATH_FILTER=$(echo "$INPUT" | jq -r '.path // ""')
GREP=$(echo "$INPUT" | jq -r '.grep // ""')

# Build git log command
CMD=("git" "-C" "$PROJECT_PATH" "log" "--max-count=$MAX_COUNT" "--format=%H|%an|%ae|%ad|%s" "--date=iso")

if [ -n "$AUTHOR" ]; then
    CMD+=("--author=$AUTHOR")
fi

if [ -n "$SINCE" ]; then
    CMD+=("--since=$SINCE")
fi

if [ -n "$PATH_FILTER" ]; then
    CMD+=("--" "$PATH_FILTER")
fi

if [ -n "$GREP" ]; then
    CMD+=("--grep=$GREP")
fi

# Execute git log
if OUTPUT=$("${CMD[@]}" 2>&1); then
    # Parse and format as JSON
    echo "$OUTPUT" | awk -F'|' 'BEGIN {print "["}
    NR > 1 {print ","}
    {
        printf "  {\n"
        printf "    \"hash\": \"%s\",\n", $1
        printf "    \"author_name\": \"%s\",\n", $2
        printf "    \"author_email\": \"%s\",\n", $3
        printf "    \"date\": \"%s\",\n", $4
        printf "    \"message\": \"%s\"\n", $5
        for (i = 6; i <= NF; i++) printf " %s", $i
        printf "  }\n"
    }
    END {print "]"}'
    exit 0
else
    # Error occurred
    echo "{\"error\": \"git log failed\", \"details\": \"$OUTPUT\"}"
    exit 1
fi
"#;

    let gitlog_readme = r#"# lookup-git-log Skill

A practical skill for querying git commit history with filtering options.

## Use Cases

- Investigating recent changes in a project
- Finding commits by a specific author
- Checking when a file was last modified
- Searching for commits matching a pattern

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `project_path` | string | No | Path to git repository (default: current directory) |
| `max_count` | integer | No | Maximum commits to return (1-100, default: 20) |
| `author` | string | No | Filter by author name or email |
| `since` | string | No | Show commits since date (e.g., "2 weeks ago") |
| `path` | string | No | Filter commits affecting a specific file/directory |
| `grep` | string | No | Filter commits by message pattern |

## Examples

```json
{
  "project_path": "/home/coding/HOOP",
  "max_count": 10,
  "author": "jedarden"
}
```

```json
{
  "project_path": "/home/coding/HOOP",
  "since": "1 week ago",
  "path": "hoop-daemon/src"
}
```
"#;

    crate::atomic_write::atomic_write_file_str(&gitlog_dir.join("manifest.yml"), gitlog_manifest)
        .unwrap_or_else(|e| warn!("Failed to write lookup-git-log skill manifest: {}", e));
    crate::atomic_write::atomic_write_file_str(&gitlog_dir.join("run"), gitlog_run)
        .unwrap_or_else(|e| warn!("Failed to write lookup-git-log skill run: {}", e));
    crate::atomic_write::atomic_write_file_str(&gitlog_dir.join("README.md"), gitlog_readme)
        .unwrap_or_else(|e| warn!("Failed to write lookup-git-log skill README: {}", e));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let run_path = gitlog_dir.join("run");
        if let Ok(metadata) = std::fs::metadata(&run_path) {
            let mut perms = metadata.permissions();
            perms.set_mode(perms.mode() | 0o111);
            let _ = std::fs::set_permissions(&run_path, perms);
        }
    }

    info!("Seeded example skill: lookup-git-log");
}

/// Start file watcher for the skills directory.
/// Returns the watcher (must be kept alive for watching to work).
pub fn start_watcher(
    skills_dir: PathBuf,
    store: SkillStore,
) -> notify::RecommendedWatcher {
    let watcher_skills_dir = skills_dir.clone();
    let mut watcher =
        notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
            match res {
                Ok(_event) => {
                    debug!("Skills directory changed, reloading");
                    let mut lib = store.write().unwrap();
                    if let Err(e) = lib.load(&watcher_skills_dir) {
                        warn!("Skills reload failed: {}", e);
                    }
                }
                Err(e) => {
                    warn!("Skills watch error: {}", e);
                }
            }
        })
        .expect("failed to create skills file watcher");

    if skills_dir.exists() {
        watcher
            .watch(&skills_dir, RecursiveMode::NonRecursive)
            .unwrap_or_else(|e| warn!("Cannot watch skills dir: {}", e));
    }

    info!("Skills file watcher started");
    watcher
}

// ---------------------------------------------------------------------------
// REST API
// ---------------------------------------------------------------------------

/// GET /api/skills — list all skills
async fn list_skills(
    State(state): State<crate::DaemonState>,
) -> Json<Vec<SkillEntry>> {
    let lib = state.skill_library.read().unwrap();
    Json(lib.list())
}

/// GET /api/skills/:name — get a single skill by name
async fn get_skill(
    Path(name): Path<String>,
    State(state): State<crate::DaemonState>,
) -> Result<Json<SkillEntry>, (StatusCode, String)> {
    let lib = state.skill_library.read().unwrap();
    lib.get(&name)
        .map(Json)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Skill '{}' not found", name)))
}

/// POST /api/skills/:name/run — execute a skill
#[axum::debug_handler]
async fn run_skill(
    State(state): State<crate::DaemonState>,
    Path(name): Path<String>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Json(req): Json<SkillRunRequest>,
) -> Result<Json<SkillRunResponse>, (StatusCode, String)> {
    // Clone needed data from skill library before any await points
    let (skill_run_path, skill_timeout, args_schema) = {
        let lib = state.skill_library.read().unwrap();
        let skill = lib.get(&name)
            .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Skill '{}' not found", name)))?;

        if !skill.executable {
            return Err((
                StatusCode::FORBIDDEN,
                format!("Skill '{}' is not executable", name),
            ));
        }

        (
            skill.run_path.clone(),
            skill.manifest.timeout_secs,
            skill.manifest.args_schema.clone(),
        )
        // Lock guard dropped here at end of scope
    };

    // Validate arguments against schema
    if let Err(validation_errors) = validate_args_against_schema(&req.args, &args_schema) {
        let error_messages: Vec<String> = validation_errors
            .iter()
            .map(|e| e.message.clone())
            .collect();
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Argument validation failed: {}", error_messages.join("; ")),
        ));
    }

    // Execute the skill (blocking call in spawn_blocking)
    let args = req.args.clone();
    let result = tokio::task::spawn_blocking(move || execute_skill(&skill_run_path, &args, skill_timeout))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to join skill execution task: {}", e),
            )
        })?
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Skill execution failed: {}", e),
            )
        })?;

    // Write audit row for skill execution
    let actor = resolve_actor(connect_info.map(|ci| ci.0));
    let args_json = serde_json::to_string(&req.args).ok();
    let args_hash = hash_args(&req.args);
    let audit_result = if result.exit_code == Some(0) {
        crate::fleet::ActionResult::Success
    } else {
        crate::fleet::ActionResult::Failure
    };
    let audit_error = if result.exit_code != Some(0) {
        Some(result.status.clone())
    } else {
        None
    };

    if let Err(e) = crate::fleet::write_audit_row(
        &actor,
        crate::fleet::ActionKind::SkillInvoked,
        &name,
        req.project.as_deref(),
        args_json,
        audit_result,
        audit_error,
        Some("api"),
        None,
        Some(&args_hash),
    ) {
        warn!("Failed to write audit row for skill execution: {}", e);
    }

    info!("Skill '{}' completed with status: {}", name, result.status);

    Ok(Json(result))
}

pub fn router() -> Router<crate::DaemonState> {
    Router::new()
        .route("/api/skills", get(list_skills))
        .route("/api/skills/{name}", get(get_skill))
        .route("/api/skills/{name}/run", post(run_skill))
}
