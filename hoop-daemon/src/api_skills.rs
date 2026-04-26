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
use jsonschema::Validator;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, warn};

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
#[derive(Debug, Deserialize)]
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
pub fn discover_skills(skills_dir: &Path) -> Vec<SkillEntry> {
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
    let compiled = match Validator::new(schema) {
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
    skill_path: &Path,
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
