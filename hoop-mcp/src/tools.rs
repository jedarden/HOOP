//! MCP tool implementations
//!
//! All tools exposed by the HOOP MCP server.

use crate::audit::{AuditLog, AuditResult};
use crate::protocol::{Content, InputSchema, OutputSchema, Tool, ToolCallResult};
use anyhow::{anyhow, Result};
use regex::Regex;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Worker-steering verbs that MUST NOT be exposed via MCP.
///
/// These represent actions that belong to NEEDLE, not HOOP. HOOP observes
/// workers and fleets; it does not control them. When an agent attempts to
/// invoke these tools, return a clear error explaining the non-goal.
pub const FORBIDDEN_WORKER_STEERING_VERBS: &[&str] = &[
    "launch_fleet",
    "stop_fleet",
    "release_claim",
    "boost_priority",
    "close_stitch",
    "close_bead",
    "signal_worker",
    "kill_worker",
    "pause_worker",
];

/// Check if a tool name is a forbidden worker-steering verb.
pub fn is_forbidden_worker_steering_verb(tool_name: &str) -> bool {
    FORBIDDEN_WORKER_STEERING_VERBS.contains(&tool_name)
}

/// Error message for forbidden worker-steering actions.
pub fn forbidden_worker_steering_error(tool_name: &str) -> String {
    format!(
        "HOOP cannot perform worker-steering actions: '{}' is not available. \
        To close a bead, use `br close` directly. To stop a worker, use NEEDLE's tooling.",
        tool_name
    )
}

/// MCP server state
pub struct McpServerState {
    pub audit_log: AuditLog,
    pub actor: String,
    /// Project registry: name -> path
    pub projects: HashMap<String, String>,
    /// Fleet database path
    pub fleet_db_path: PathBuf,
}

impl McpServerState {
    /// Create a new MCP server state
    pub fn new(actor: String) -> Result<Self> {
        let audit_log = AuditLog::open()?;

        // Load project registry from ~/.hoop/projects.yaml
        let projects = Self::load_projects()?;

        // Fleet db path
        let mut home = dirs::home_dir()
            .ok_or_else(|| anyhow!("Cannot determine home directory"))?;
        home.push(".hoop");
        home.push("fleet.db");
        let fleet_db_path = home;

        Ok(Self {
            audit_log,
            actor,
            projects,
            fleet_db_path,
        })
    }

    /// Load projects from ~/.hoop/projects.yaml
    fn load_projects() -> Result<HashMap<String, String>> {
        let mut path = dirs::home_dir()
            .ok_or_else(|| anyhow!("Cannot determine home directory"))?;
        path.push(".hoop");
        path.push("projects.yaml");

        let mut projects = HashMap::new();

        if !path.exists() {
            return Ok(projects);
        }

        let contents = fs::read_to_string(&path)?;
        let yaml: Value = serde_yaml::from_str(&contents)?;

        if let Some(project_list) = yaml.get("projects").and_then(|p| p.as_array()) {
            for project in project_list {
                if let Some(name) = project.get("name").and_then(|n| n.as_str()) {
                    // Try shorthand single-workspace form
                    if let Some(p) = project.get("path").and_then(|p| p.as_str()) {
                        projects.insert(name.to_string(), p.to_string());
                        continue;
                    }
                    // Try multi-workspace form (use primary workspace)
                    if let Some(workspaces) = project.get("workspaces").and_then(|w| w.as_array()) {
                        for ws in workspaces {
                            if let Some(role) = ws.get("role").and_then(|r| r.as_str()) {
                                if role == "primary" {
                                    if let Some(p) = ws.get("path").and_then(|p| p.as_str()) {
                                        projects.insert(name.to_string(), p.to_string());
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(projects)
    }

    /// Get all available tools with input and output schemas
    pub fn get_tools() -> Vec<Tool> {
        vec![
            // Read tools
            Tool {
                name: "find_stitches".to_string(),
                description: "List stitches in a project with optional filtering. Returns aggregated stitch data including linked beads, participants, and activity status.".to_string(),
                input_schema: input_schema_find_stitches(),
                output_schema: Some(output_schema_find_stitches()),
            },
            Tool {
                name: "read_stitch".to_string(),
                description: "Get detailed information about a single stitch by ID, including all messages, linked beads, and participant metadata.".to_string(),
                input_schema: input_schema_read_stitch(),
                output_schema: Some(output_schema_read_stitch()),
            },
            Tool {
                name: "find_beads".to_string(),
                description: "List beads in a project with optional filtering. Returns bead summaries with status, priority, and dependencies.".to_string(),
                input_schema: input_schema_find_beads(),
                output_schema: Some(output_schema_find_beads()),
            },
            Tool {
                name: "read_bead".to_string(),
                description: "Get detailed information about a single bead by ID, including title, description, status, and dependencies.".to_string(),
                input_schema: input_schema_read_bead(),
                output_schema: Some(output_schema_read_bead()),
            },
            Tool {
                name: "read_file".to_string(),
                description: "Read a file from a project's repository. Supports optional git revision for historical views.".to_string(),
                input_schema: input_schema_read_file(),
                output_schema: Some(output_schema_read_file()),
            },
            Tool {
                name: "grep".to_string(),
                description: "Search for a pattern across files in a project using grep regex syntax.".to_string(),
                input_schema: input_schema_grep(),
                output_schema: Some(output_schema_grep()),
            },
            Tool {
                name: "search_conversations".to_string(),
                description: "Search conversation transcripts for a query string. Returns matching message excerpts with context.".to_string(),
                input_schema: input_schema_search_conversations(),
                output_schema: Some(output_schema_search_conversations()),
            },
            Tool {
                name: "summarize_project".to_string(),
                description: "Get a summary of project activity including open beads, active stitches, cost today, and recent events.".to_string(),
                input_schema: input_schema_summarize_project(),
                output_schema: Some(output_schema_summarize_project()),
            },
            Tool {
                name: "summarize_day".to_string(),
                description: "Get a daily summary across all projects including total beads closed, stitches created, and cost aggregated.".to_string(),
                input_schema: input_schema_summarize_day(),
                output_schema: Some(output_schema_summarize_day()),
            },
            // Write tools
            Tool {
                name: "create_stitch".to_string(),
                description: "Submit a stitch draft for operator review. Creates a draft in the preview queue — the operator must approve before any beads are created. No silent writes.".to_string(),
                input_schema: input_schema_create_stitch(),
                output_schema: Some(output_schema_create_stitch()),
            },
            Tool {
                name: "create_bead".to_string(),
                description: "Create a follow-up bead in a project. When parent_bead_id is set, stitch:* labels are automatically inherited from the parent bead (Hook 4).".to_string(),
                input_schema: input_schema_create_bead(),
                output_schema: Some(output_schema_create_bead()),
            },
            // Utility tools
            Tool {
                name: "escalate_to_operator".to_string(),
                description: "Send a message to the operator that will be displayed as a UI banner. Use when human intervention is needed. No automatic actions are taken.".to_string(),
                input_schema: input_schema_escalate_to_operator(),
                output_schema: Some(output_schema_escalate_to_operator()),
            },
        ]
    }

    /// Handle a tool call
    pub fn call_tool(&self, name: &str, args: &Map<String, Value>) -> Result<ToolCallResult, String> {
        // Runtime guard: reject worker-steering verbs (these are NOT exposed via MCP)
        if is_forbidden_worker_steering_verb(name) {
            return Err(forbidden_worker_steering_error(name));
        }

        let result = match name {
            // Read tools
            "find_stitches" => self.find_stitches(args),
            "read_stitch" => self.read_stitch(args),
            "find_beads" => self.find_beads(args),
            "read_bead" => self.read_bead(args),
            "read_file" => self.read_file(args),
            "grep" => self.grep(args),
            "search_conversations" => self.search_conversations(args),
            "summarize_project" => self.summarize_project(args),
            "summarize_day" => self.summarize_day(args),
            // Write tools
            "create_stitch" => self.create_stitch(args),
            "create_bead" => self.create_bead(args),
            // Utility tools
            "escalate_to_operator" => self.escalate_to_operator(args),
            _ => Err(format!("Unknown tool: {}", name)),
        };

        // Record in audit log
        let audit_result = match &result {
            Ok(_) => AuditResult::Success,
            Err(e) => AuditResult::Failure(e.clone()),
        };

        let args_value = Value::Object(args.clone());
        let _ = self.audit_log.record(&self.actor, name, Some(&args_value), &audit_result);

        result
    }

    // -----------------------------------------------------------------------
    // Read tools
    // -----------------------------------------------------------------------

    fn find_stitches(&self, args: &Map<String, Value>) -> Result<ToolCallResult, String> {
        let project = args.get("project")
            .and_then(|v| v.as_str())
            .ok_or("project parameter is required")?;

        let _project_path = self.projects.get(project)
            .ok_or(format!("Project '{}' not found", project))?;

        // Read stitches from fleet.db
        let stitches = self.query_stitches_from_db(project, args)?;

        let content = serde_json::to_string_pretty(&stitches)
            .map_err(|e| format!("Failed to serialize result: {}", e))?;

        Ok(ToolCallResult {
            content: vec![Content::Text { text: content }],
            is_error: None,
        })
    }

    fn read_stitch(&self, args: &Map<String, Value>) -> Result<ToolCallResult, String> {
        let stitch_id = args.get("id")
            .and_then(|v| v.as_str())
            .ok_or("id parameter is required")?;

        crate::id_validators::validate_stitch_id(stitch_id)
            .map_err(|e| format!("id: {}", e))?;

        // Try the daemon's aggregated-read endpoint first (full data).
        // Fall back to direct DB query if daemon is not reachable.
        match self.read_stitch_via_daemon(stitch_id) {
            Ok(result) => {
                let content = serde_json::to_string_pretty(&result)
                    .map_err(|e| format!("Failed to serialize result: {}", e))?;
                Ok(ToolCallResult {
                    content: vec![Content::Text { text: content }],
                    is_error: None,
                })
            }
            Err(_) => {
                // Daemon not available — fall back to direct DB query
                let stitch = self.query_stitch_detail_from_db(stitch_id)
                    .map_err(|e| format!("Failed to read stitch: {}", e))?;

                let content = serde_json::to_string_pretty(&stitch)
                    .map_err(|e| format!("Failed to serialize result: {}", e))?;

                Ok(ToolCallResult {
                    content: vec![Content::Text { text: content }],
                    is_error: None,
                })
            }
        }
    }

    fn find_beads(&self, args: &Map<String, Value>) -> Result<ToolCallResult, String> {
        let project = args.get("project")
            .and_then(|v| v.as_str())
            .ok_or("project parameter is required")?;

        let project_path = self.projects.get(project)
            .ok_or(format!("Project '{}' not found", project))?;

        // Read beads using br list
        let beads = self.list_beads_via_br(project_path, args)?;

        let content = serde_json::to_string_pretty(&beads)
            .map_err(|e| format!("Failed to serialize result: {}", e))?;

        Ok(ToolCallResult {
            content: vec![Content::Text { text: content }],
            is_error: None,
        })
    }

    fn read_bead(&self, args: &Map<String, Value>) -> Result<ToolCallResult, String> {
        let project = args.get("project")
            .and_then(|v| v.as_str())
            .ok_or("project parameter is required")?;

        let bead_id = args.get("id")
            .and_then(|v| v.as_str())
            .ok_or("id parameter is required")?;

        crate::id_validators::validate_bead_id(bead_id)
            .map_err(|e| format!("id: {}", e))?;

        let project_path = self.projects.get(project)
            .ok_or(format!("Project '{}' not found", project))?;

        // Read bead details using br get
        let bead = self.get_bead_via_br(project_path, bead_id)
            .map_err(|e| format!("Failed to read bead: {}", e))?;

        let content = serde_json::to_string_pretty(&bead)
            .map_err(|e| format!("Failed to serialize result: {}", e))?;

        Ok(ToolCallResult {
            content: vec![Content::Text { text: content }],
            is_error: None,
        })
    }

    fn read_file(&self, args: &Map<String, Value>) -> Result<ToolCallResult, String> {
        let project = args.get("project")
            .and_then(|v| v.as_str())
            .ok_or("project parameter is required")?;

        let file_path = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or("path parameter is required")?;

        let project_path = self.projects.get(project)
            .ok_or(format!("Project '{}' not found", project))?;

        let full_path = PathBuf::from(project_path).join(file_path);

        // Security check: ensure path is within project
        let canonical = full_path.canonicalize()
            .map_err(|e| format!("Path error: {}", e))?;
        let canonical_project = PathBuf::from(project_path).canonicalize()
            .map_err(|e| format!("Project path error: {}", e))?;

        if !canonical.starts_with(&canonical_project) {
            return Err("Path traversal detected".to_string());
        }

        let content = fs::read_to_string(&canonical)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        Ok(ToolCallResult {
            content: vec![Content::Text { text: content }],
            is_error: None,
        })
    }

    fn grep(&self, args: &Map<String, Value>) -> Result<ToolCallResult, String> {
        let project = args.get("project")
            .and_then(|v| v.as_str())
            .ok_or("project parameter is required")?;

        let pattern = args.get("pattern")
            .and_then(|v| v.as_str())
            .ok_or("pattern parameter is required")?;

        let project_path = self.projects.get(project)
            .ok_or(format!("Project '{}' not found", project))?;

        let results = self.grep_in_project(project_path, pattern, args)
            .map_err(|e| format!("Grep error: {}", e))?;

        let content = serde_json::to_string_pretty(&results)
            .map_err(|e| format!("Failed to serialize result: {}", e))?;

        Ok(ToolCallResult {
            content: vec![Content::Text { text: content }],
            is_error: None,
        })
    }

    fn search_conversations(&self, args: &Map<String, Value>) -> Result<ToolCallResult, String> {
        let query = args.get("query")
            .and_then(|v| v.as_str())
            .ok_or("query parameter is required")?;

        let project = args.get("project")
            .and_then(|v| v.as_str());

        let results = self.search_conversations_in_db(query, project)
            .map_err(|e| format!("Search error: {}", e))?;

        let content = serde_json::to_string_pretty(&results)
            .map_err(|e| format!("Failed to serialize result: {}", e))?;

        Ok(ToolCallResult {
            content: vec![Content::Text { text: content }],
            is_error: None,
        })
    }

    fn summarize_project(&self, args: &Map<String, Value>) -> Result<ToolCallResult, String> {
        let project = args.get("project")
            .and_then(|v| v.as_str())
            .ok_or("project parameter is required")?;

        let _project_path = self.projects.get(project)
            .ok_or(format!("Project '{}' not found", project))?;

        let summary = self.generate_project_summary(project)
            .map_err(|e| format!("Failed to generate summary: {}", e))?;

        let content = serde_json::to_string_pretty(&summary)
            .map_err(|e| format!("Failed to serialize result: {}", e))?;

        Ok(ToolCallResult {
            content: vec![Content::Text { text: content }],
            is_error: None,
        })
    }

    fn summarize_day(&self, _args: &Map<String, Value>) -> Result<ToolCallResult, String> {
        let summary = self.generate_day_summary()
            .map_err(|e| format!("Failed to generate summary: {}", e))?;

        let content = serde_json::to_string_pretty(&summary)
            .map_err(|e| format!("Failed to serialize result: {}", e))?;

        Ok(ToolCallResult {
            content: vec![Content::Text { text: content }],
            is_error: None,
        })
    }

    // -----------------------------------------------------------------------
    // Write tools
    // -----------------------------------------------------------------------

    fn create_stitch(&self, args: &Map<String, Value>) -> Result<ToolCallResult, String> {
        let project = args.get("project")
            .and_then(|v| v.as_str())
            .ok_or("project parameter is required")?;

        let title = args.get("title")
            .and_then(|v| v.as_str())
            .ok_or("title parameter is required")?;

        let kind = args.get("kind")
            .and_then(|v| v.as_str())
            .unwrap_or("investigation");

        let description = args.get("description")
            .and_then(|v| v.as_str());

        let priority = args.get("priority")
            .and_then(|v| v.as_i64());

        // Validate project exists
        let _project_path = self.projects.get(project)
            .ok_or(format!("Project '{}' not found", project))?;

        // Call the daemon's draft API which performs deduplication check
        let result = self.create_stitch_via_daemon(project, title, description, kind, priority)
            .map_err(|e| format!("Failed to create stitch draft: {}", e))?;

        let content = serde_json::to_string_pretty(&result)
            .map_err(|e| format!("Failed to serialize result: {}", e))?;

        Ok(ToolCallResult {
            content: vec![Content::Text { text: content }],
            is_error: None,
        })
    }

    /// Create a follow-up bead via `br create` with Hook 4 label propagation.
    ///
    /// When `parent_bead_id` is set, reads the parent bead's labels via `br get`,
    /// extracts all `stitch:*` labels, and appends them to the new bead (deduplicated).
    fn create_bead(&self, args: &Map<String, Value>) -> Result<ToolCallResult, String> {
        // Zero-write guard
        #[cfg(feature = "zero-write-v01")]
        {
            let _ = args;
            return Err("Bead creation is disabled in zero-write mode".to_string());
        }

        let project = args.get("project")
            .and_then(|v| v.as_str())
            .ok_or("project parameter is required")?;

        let title = args.get("title")
            .and_then(|v| v.as_str())
            .ok_or("title parameter is required")?;

        let description = args.get("description")
            .and_then(|v| v.as_str());

        let issue_type = args.get("issue_type")
            .and_then(|v| v.as_str())
            .unwrap_or("task");

        let priority = args.get("priority")
            .and_then(|v| v.as_i64());

        let labels: Vec<String> = args.get("labels")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let parent_bead_id = args.get("parent_bead_id")
            .and_then(|v| v.as_str());

        // Validate parent bead ID
        if let Some(pid) = parent_bead_id {
            crate::id_validators::validate_bead_id(pid)
                .map_err(|e| format!("parent_bead_id: {}", e))?;
        }

        let project_path = self.projects.get(project)
            .ok_or(format!("Project '{}' not found", project))?;

        // Build label list
        let mut all_labels = labels;

        // Hook 4: Inherit stitch labels from parent bead
        if let Some(pid) = parent_bead_id {
            if let Ok(parent_labels) = self.lookup_bead_labels(project_path, pid) {
                crate::br_verbs::propagate_stitch_labels(&mut all_labels, &parent_labels);
            }
        }

        // Execute br create
        #[cfg(not(feature = "zero-write-v01"))]
        {
            let mut cmd = crate::br_verbs::invoke_br_create(&[]);
            cmd.current_dir(project_path);
            cmd.arg(title);
            cmd.arg("--type").arg(issue_type);

            if let Some(desc) = description {
                if !desc.is_empty() {
                    cmd.arg("--description").arg(desc);
                }
            }

            if let Some(p) = priority {
                cmd.arg("--priority").arg(p.to_string());
            }

            if !all_labels.is_empty() {
                cmd.arg("--labels").arg(all_labels.join(","));
            }

            cmd.arg("--actor").arg(&self.actor);
            cmd.arg("--silent");

            let output = cmd.output()
                .map_err(|e| format!("Failed to run br create: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("br create failed: {}", stderr.trim()));
            }

            let bead_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if bead_id.is_empty() {
                return Err("br create did not return a bead ID".to_string());
            }

            let result = json!({
                "id": bead_id,
                "title": title,
                "project": project,
                "labels": all_labels,
                "parent_bead_id": parent_bead_id,
            });

            let content = serde_json::to_string_pretty(&result)
                .map_err(|e| format!("Failed to serialize result: {}", e))?;

            Ok(ToolCallResult {
                content: vec![Content::Text { text: content }],
                is_error: None,
            })
        }
    }

    // -----------------------------------------------------------------------
    // Utility tools
    // -----------------------------------------------------------------------

    fn escalate_to_operator(&self, args: &Map<String, Value>) -> Result<ToolCallResult, String> {
        let message = args.get("message")
            .and_then(|v| v.as_str())
            .ok_or("message parameter is required")?;

        // Write escalation to ~/.hoop/escalations.jsonl
        let mut path = dirs::home_dir()
            .ok_or("Cannot determine home directory")?;
        path.push(".hoop");
        path.push("escalations.jsonl");

        let timestamp = chrono::Utc::now().to_rfc3339();
        let entry = json!({
            "timestamp": timestamp,
            "actor": &self.actor,
            "message": message,
        });

        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| format!("Failed to open escalations file: {}", e))?;

        use std::io::Write;
        writeln!(file, "{}", entry)
            .map_err(|e| format!("Failed to write escalation: {}", e))?;

        Ok(ToolCallResult {
            content: vec![Content::Text {
                text: format!("Escalation sent: {}", message)
            }],
            is_error: None,
        })
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn open_fleet_db(&self) -> Result<rusqlite::Connection, String> {
        if !self.fleet_db_path.exists() {
            return Err("fleet.db not found — daemon may not have been started yet".to_string());
        }
        rusqlite::Connection::open(&self.fleet_db_path)
            .map_err(|e| format!("Failed to open fleet.db: {}", e))
    }

    /// Call the daemon's aggregated-read endpoint for a stitch.
    /// Returns the full enriched response (messages, live beads, cost/duration, link graph).
    fn read_stitch_via_daemon(&self, stitch_id: &str) -> Result<Value, String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| format!("HTTP client error: {}", e))?;

        let url = format!("http://127.0.0.1:3000/api/stitches/{}", stitch_id);
        let response = client
            .get(&url)
            .send()
            .map_err(|e| format!("Daemon unreachable: {}", e))?;

        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(format!("Stitch '{}' not found", stitch_id));
        }
        if !status.is_success() {
            let error_text = response.text().unwrap_or_else(|_| format!("HTTP {}", status.as_u16()));
            return Err(format!("Daemon returned error: {}", error_text));
        }

        let data: Value = response
            .json()
            .map_err(|e| format!("Failed to parse daemon response: {}", e))?;

        // §18.3: Redact secrets in message content before forwarding to the agent.
        Ok(redact_stitch_response(data))
    }

    fn query_stitches_from_db(&self, project: &str, args: &Map<String, Value>) -> Result<Vec<Value>, String> {
        let conn = self.open_fleet_db()?;

        let kind_filter = args.get("kind").and_then(|v| v.as_str());
        let limit = args.get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(50) as i64;

        let (sql, params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = if let Some(kind) = kind_filter {
            (
                "SELECT id, project, kind, title, created_by, created_at, last_activity_at, participants
                 FROM stitches
                 WHERE project = ?1 AND kind = ?2
                 ORDER BY last_activity_at DESC
                 LIMIT ?3".to_string(),
                vec![Box::new(project.to_string()), Box::new(kind.to_string()), Box::new(limit)],
            )
        } else {
            (
                "SELECT id, project, kind, title, created_by, created_at, last_activity_at, participants
                 FROM stitches
                 WHERE project = ?1
                 ORDER BY last_activity_at DESC
                 LIMIT ?2".to_string(),
                vec![Box::new(project.to_string()), Box::new(limit)],
            )
        };

        let mut stmt = conn.prepare(&sql)
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            let participants: String = row.get(7).unwrap_or_default();
            Ok(json!({
                "id": row.get::<_, String>(0)?,
                "project": row.get::<_, String>(1)?,
                "kind": row.get::<_, String>(2)?,
                "title": row.get::<_, String>(3)?,
                "created_by": row.get::<_, String>(4)?,
                "created_at": row.get::<_, String>(5)?,
                "last_activity_at": row.get::<_, String>(6)?,
                "participants": participants,
            }))
        }).map_err(|e| format!("Failed to execute query: {}", e))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| format!("Row error: {}", e))?);
        }

        Ok(results)
    }

    fn query_stitch_detail_from_db(&self, stitch_id: &str) -> Result<Value, String> {
        let conn = self.open_fleet_db()?;

        // Get stitch info
        let stitch = conn.query_row(
            "SELECT id, project, kind, title, created_by, created_at, last_activity_at
             FROM stitches WHERE id = ?1",
            [stitch_id],
            |row| {
                Ok(json!({
                    "id": row.get::<_, String>(0)?,
                    "project": row.get::<_, String>(1)?,
                    "kind": row.get::<_, String>(2)?,
                    "title": row.get::<_, String>(3)?,
                    "created_by": row.get::<_, String>(4)?,
                    "created_at": row.get::<_, String>(5)?,
                    "last_activity_at": row.get::<_, String>(6)?,
                }))
            },
        ).map_err(|e| format!("Stitch not found: {}", e))?;

        // Get messages for this stitch
        let mut stmt = conn.prepare(
            "SELECT id, ts, role, content
             FROM stitch_messages
             WHERE stitch_id = ?1
             ORDER BY ts"
        ).map_err(|e| format!("Failed to prepare messages query: {}", e))?;

        let messages: Vec<Value> = stmt.query_map([stitch_id], |row| {
            let content: String = row.get(3)?;
            // §18.3: Redact secrets before forwarding to the agent.
            let content = crate::redaction::redact_text(&content);
            Ok(json!({
                "id": row.get::<_, String>(0)?,
                "ts": row.get::<_, String>(1)?,
                "role": row.get::<_, String>(2)?,
                "content": content,
            }))
        }).map_err(|e| format!("Failed to execute messages query: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Message row error: {}", e))?;

        // Get linked beads
        let mut stmt = conn.prepare(
            "SELECT bead_id, workspace, relationship
             FROM stitch_beads
             WHERE stitch_id = ?1"
        ).map_err(|e| format!("Failed to prepare beads query: {}", e))?;

        let beads: Vec<Value> = stmt.query_map([stitch_id], |row| {
            Ok(json!({
                "bead_id": row.get::<_, String>(0)?,
                "workspace": row.get::<_, String>(1)?,
                "relationship": row.get::<_, String>(2)?,
            }))
        }).map_err(|e| format!("Failed to execute beads query: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Bead row error: {}", e))?;

        Ok(json!({
            "stitch": stitch,
            "messages": messages,
            "linked_beads": beads,
        }))
    }

    fn list_beads_via_br(&self, project_path: &str, args: &Map<String, Value>) -> Result<Vec<Value>, String> {
        let limit = args.get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(50);

        let status_filter = args.get("status")
            .and_then(|v| v.as_str());

        // Call br list --json
        let mut cmd = crate::br_verbs::invoke_br_read(
            crate::br_verbs::ReadVerb::List,
            &["--json"],
        );
        let output = cmd.current_dir(project_path)
            .output()
            .map_err(|e| format!("Failed to execute br: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("br list failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut beads: Vec<Value> = serde_json::from_str(&stdout)
            .unwrap_or_default();

        // Apply filters
        if let Some(status) = status_filter {
            beads.retain(|b| {
                b.get("status")
                    .and_then(|s| s.as_str())
                    .map(|s| s == status)
                    .unwrap_or(false)
            });
        }

        // Apply limit
        beads.truncate(limit as usize);

        Ok(beads)
    }

    fn get_bead_via_br(&self, project_path: &str, bead_id: &str) -> Result<Value, String> {
        let mut cmd = crate::br_verbs::invoke_br_read(
            crate::br_verbs::ReadVerb::Get,
            &[bead_id, "--json"],
        );
        let output = cmd.current_dir(project_path)
            .output()
            .map_err(|e| format!("Failed to execute br: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("br get failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        serde_json::from_str(&stdout)
            .map_err(|e| format!("Failed to parse br output: {}", e))
    }

    /// Look up a bead's labels via `br get --json`.
    ///
    /// Used by Hook 4 to inherit stitch labels from a parent bead.
    fn lookup_bead_labels(&self, project_path: &str, bead_id: &str) -> Result<Vec<String>, String> {
        let bead_json = self.get_bead_via_br(project_path, bead_id)?;
        bead_json
            .get("labels")
            .and_then(|l| l.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .ok_or_else(|| "No labels field on bead".to_string())
    }

    fn grep_in_project(&self, project_path: &str, pattern: &str, args: &Map<String, Value>) -> Result<Vec<Value>, String> {
        let max_results = args.get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(100) as usize;

        let path_arg = args.get("path")
            .and_then(|v| v.as_str());

        let base_path = if let Some(p) = path_arg {
            PathBuf::from(project_path).join(p)
        } else {
            PathBuf::from(project_path)
        };

        let regex = Regex::new(pattern)
            .map_err(|e| format!("Invalid regex: {}", e))?;

        let mut results = Vec::new();

        // Use ignore crate for efficient file walking
        let walker = ignore::WalkBuilder::new(&base_path)
            .hidden(true)
            .git_ignore(true)
            .build();

        for entry in walker.flatten().take(max_results * 10) {
            if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    for (line_num, line) in content.lines().enumerate() {
                        if regex.is_match(line) {
                            let display_path = entry.path()
                                .strip_prefix(project_path)
                                .unwrap_or(entry.path())
                                .display()
                                .to_string();
                            results.push(json!({
                                "path": display_path,
                                "line_number": line_num + 1,
                                "line": line,
                            }));

                            if results.len() >= max_results {
                                break;
                            }
                        }
                    }
                }
            }

            if results.len() >= max_results {
                break;
            }
        }

        Ok(results)
    }

    fn search_conversations_in_db(&self, query: &str, project: Option<&str>) -> Result<Vec<Value>, String> {
        let conn = self.open_fleet_db()?;

        let pattern = format!("%{}%", query);

        // §18.3: redact content before forwarding to the agent.
        let redact = |raw: String| crate::redaction::redact_text(&raw);

        if let Some(proj) = project {
            let mut stmt = conn.prepare(
                "SELECT sm.stitch_id, sm.ts, sm.role, sm.content, s.project
                 FROM stitch_messages sm
                 JOIN stitches s ON sm.stitch_id = s.id
                 WHERE s.project = ?1 AND sm.content LIKE ?2
                 ORDER BY sm.ts DESC
                 LIMIT 50"
            ).map_err(|e| format!("Failed to prepare query: {}", e))?;

            let rows: Result<Vec<Value>, _> = stmt.query_map([proj, &pattern], |row| {
                let content: String = row.get(3)?;
                Ok(json!({
                    "stitch_id": row.get::<_, String>(0)?,
                    "timestamp": row.get::<_, String>(1)?,
                    "role": row.get::<_, String>(2)?,
                    "content": redact(content),
                    "project": row.get::<_, String>(4)?,
                }))
            }).map_err(|e| format!("Failed to execute query: {}", e))?
                .collect();

            Ok(rows.map_err(|e| format!("Row error: {}", e))?)
        } else {
            let mut stmt = conn.prepare(
                "SELECT sm.stitch_id, sm.ts, sm.role, sm.content, s.project
                 FROM stitch_messages sm
                 JOIN stitches s ON sm.stitch_id = s.id
                 WHERE sm.content LIKE ?1
                 ORDER BY sm.ts DESC
                 LIMIT 50"
            ).map_err(|e| format!("Failed to prepare query: {}", e))?;

            let rows: Result<Vec<Value>, _> = stmt.query_map([&pattern], |row| {
                let content: String = row.get(3)?;
                Ok(json!({
                    "stitch_id": row.get::<_, String>(0)?,
                    "timestamp": row.get::<_, String>(1)?,
                    "role": row.get::<_, String>(2)?,
                    "content": redact(content),
                    "project": row.get::<_, String>(4)?,
                }))
            }).map_err(|e| format!("Failed to execute query: {}", e))?
                .collect();

            Ok(rows.map_err(|e| format!("Row error: {}", e))?)
        }
    }

    fn generate_project_summary(&self, project: &str) -> Result<Value, String> {
        let conn = self.open_fleet_db()?;

        // Get stitch counts
        let stitch_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM stitches WHERE project = ?1",
            [project],
            |row| row.get(0),
        ).unwrap_or(0);

        // Get recent activity — try with project column first, fall back to count all
        let recent_actions: i64 = conn.query_row(
            "SELECT COUNT(*) FROM actions WHERE project = ?1 AND ts > datetime('now', '-24 hours')",
            [project],
            |row| row.get(0),
        ).unwrap_or_else(|_| {
            conn.query_row(
                "SELECT COUNT(*) FROM actions WHERE ts > datetime('now', '-24 hours')",
                [],
                |row| row.get(0),
            ).unwrap_or(0)
        });

        Ok(json!({
            "project": project,
            "stitch_count": stitch_count,
            "recent_actions_24h": recent_actions,
            "generated_at": chrono::Utc::now().to_rfc3339(),
        }))
    }

    fn generate_day_summary(&self) -> Result<Value, String> {
        let conn = self.open_fleet_db()?;

        // Get today's action count
        let today_actions: i64 = conn.query_row(
            "SELECT COUNT(*) FROM actions WHERE ts > datetime('now', 'start of day')",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        // Get stitch count
        let total_stitches: i64 = conn.query_row(
            "SELECT COUNT(*) FROM stitches",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        // Get active stitches (last 24h activity)
        let active_stitches: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT stitch_id) FROM stitch_messages WHERE ts > datetime('now', '-24 hours')",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        Ok(json!({
            "date": chrono::Utc::now().format("%Y-%m-%d").to_string(),
            "total_stitches": total_stitches,
            "active_stitches_24h": active_stitches,
            "today_actions": today_actions,
            "generated_at": chrono::Utc::now().to_rfc3339(),
        }))
    }

    /// Create a stitch draft by calling the daemon's draft API.
    ///
    /// The daemon performs deduplication checking before creating the draft.
    /// If a duplicate is detected, a 409 CONFLICT response is returned.
    /// This ensures agents never silently create duplicate work (§3.10 read-first principle).
    fn create_stitch_via_daemon(
        &self,
        project: &str,
        title: &str,
        description: Option<&str>,
        kind: &str,
        priority: Option<i64>,
    ) -> Result<Value, String> {
        // Build the request body matching CreateDraftRequest
        let request_body = json!({
            "project": project,
            "title": title,
            "kind": kind,
            "description": description,
            "has_acceptance_criteria": false,
            "priority": priority,
            "labels": [],
            "source": "agent",
        });

        // Call the daemon's draft API
        let client = reqwest::blocking::Client::new();
        let response = client
            .post("http://127.0.0.1:3000/api/drafts")
            .json(&request_body)
            .send()
            .map_err(|e| format!("Failed to connect to daemon: {}. Is hoop-daemon running on 127.0.0.1:3000?", e))?;

        let status = response.status();

        if status == reqwest::StatusCode::CONFLICT {
            // Deduplication check failed - a similar stitch/bead exists
            let error_msg = response
                .text()
                .unwrap_or_else(|_| "Duplicate detected".to_string());
            return Err(error_msg);
        }

        if !status.is_success() {
            let error_text = response
                .text()
                .unwrap_or_else(|_| format!("HTTP {}", status.as_u16()));
            return Err(format!("Daemon returned error: {}", error_text));
        }

        let response_json: Value = response
            .json()
            .map_err(|e| format!("Failed to parse daemon response: {}", e))?;

        Ok(response_json)
    }
}

// -----------------------------------------------------------------------
// Redaction helpers
// -----------------------------------------------------------------------

/// Redact secrets in message content within a stitch response object.
///
/// The daemon returns `{ stitch: {...}, messages: [{..., content: "..."}, ...], ... }`.
/// We walk the `messages` array and redact the `content` field of each message
/// before forwarding to the agent (§18.3).
fn redact_stitch_response(mut value: Value) -> Value {
    if let Some(messages) = value.get_mut("messages").and_then(|m| m.as_array_mut()) {
        for msg in messages.iter_mut() {
            if let Some(content) = msg.get_mut("content") {
                let redacted = match content.take() {
                    Value::String(s) => Value::String(crate::redaction::redact_text(&s)),
                    other => {
                        // Recursively redact nested JSON (content blocks, tool results, etc.)
                        redact_json_value_mcp(other)
                    }
                };
                *content = redacted;
            }
        }
    }
    value
}

/// Recursively redact all string leaves in a JSON value.
fn redact_json_value_mcp(value: Value) -> Value {
    match value {
        Value::String(s) => Value::String(crate::redaction::redact_text(&s)),
        Value::Array(arr) => Value::Array(arr.into_iter().map(redact_json_value_mcp).collect()),
        Value::Object(mut obj) => {
            for v in obj.values_mut() {
                *v = redact_json_value_mcp(v.take());
            }
            Value::Object(obj)
        }
        other => other,
    }
}

// -----------------------------------------------------------------------
// Input schema builders
// -----------------------------------------------------------------------

fn input_schema_find_stitches() -> InputSchema {
    InputSchema {
        schema_type: "object".to_string(),
        properties: {
            let mut props = serde_json::Map::new();
            props.insert("project".to_string(), json!({
                "type": "string",
                "description": "Project name to search within"
            }));
            props.insert("limit".to_string(), json!({
                "type": "number",
                "description": "Maximum number of results to return (default: 50)"
            }));
            props.insert("kind".to_string(), json!({
                "type": "string",
                "description": "Filter by stitch kind (operator, dictated, worker, ad-hoc)",
                "enum": ["operator", "dictated", "worker", "ad-hoc"]
            }));
            props
        },
        required: Some(vec!["project".to_string()]),
    }
}

fn input_schema_read_stitch() -> InputSchema {
    InputSchema {
        schema_type: "object".to_string(),
        properties: {
            let mut props = serde_json::Map::new();
            props.insert("id".to_string(), json!({
                "type": "string",
                "description": "Stitch ID to read"
            }));
            props
        },
        required: Some(vec!["id".to_string()]),
    }
}

fn input_schema_find_beads() -> InputSchema {
    InputSchema {
        schema_type: "object".to_string(),
        properties: {
            let mut props = serde_json::Map::new();
            props.insert("project".to_string(), json!({
                "type": "string",
                "description": "Project name to search within"
            }));
            props.insert("limit".to_string(), json!({
                "type": "number",
                "description": "Maximum number of results to return (default: 50)"
            }));
            props.insert("status".to_string(), json!({
                "type": "string",
                "description": "Filter by bead status (open, closed)",
                "enum": ["open", "closed"]
            }));
            props
        },
        required: Some(vec!["project".to_string()]),
    }
}

fn input_schema_read_bead() -> InputSchema {
    InputSchema {
        schema_type: "object".to_string(),
        properties: {
            let mut props = serde_json::Map::new();
            props.insert("project".to_string(), json!({
                "type": "string",
                "description": "Project name"
            }));
            props.insert("id".to_string(), json!({
                "type": "string",
                "description": "Bead ID to read"
            }));
            props
        },
        required: Some(vec!["project".to_string(), "id".to_string()]),
    }
}

fn input_schema_read_file() -> InputSchema {
    InputSchema {
        schema_type: "object".to_string(),
        properties: {
            let mut props = serde_json::Map::new();
            props.insert("project".to_string(), json!({
                "type": "string",
                "description": "Project name"
            }));
            props.insert("path".to_string(), json!({
                "type": "string",
                "description": "File path relative to project root"
            }));
            props.insert("revision".to_string(), json!({
                "type": "string",
                "description": "Optional git revision (e.g., commit hash, branch name)"
            }));
            props
        },
        required: Some(vec!["project".to_string(), "path".to_string()]),
    }
}

fn input_schema_grep() -> InputSchema {
    InputSchema {
        schema_type: "object".to_string(),
        properties: {
            let mut props = serde_json::Map::new();
            props.insert("project".to_string(), json!({
                "type": "string",
                "description": "Project name to search within"
            }));
            props.insert("pattern".to_string(), json!({
                "type": "string",
                "description": "Regex pattern to search for"
            }));
            props.insert("path".to_string(), json!({
                "type": "string",
                "description": "Optional subdirectory path to limit search"
            }));
            props.insert("max_results".to_string(), json!({
                "type": "number",
                "description": "Maximum number of matches to return (default: 100)"
            }));
            props
        },
        required: Some(vec!["project".to_string(), "pattern".to_string()]),
    }
}

fn input_schema_search_conversations() -> InputSchema {
    InputSchema {
        schema_type: "object".to_string(),
        properties: {
            let mut props = serde_json::Map::new();
            props.insert("query".to_string(), json!({
                "type": "string",
                "description": "Search query string"
            }));
            props.insert("project".to_string(), json!({
                "type": "string",
                "description": "Optional project filter"
            }));
            props
        },
        required: Some(vec!["query".to_string()]),
    }
}

fn input_schema_summarize_project() -> InputSchema {
    InputSchema {
        schema_type: "object".to_string(),
        properties: {
            let mut props = serde_json::Map::new();
            props.insert("project".to_string(), json!({
                "type": "string",
                "description": "Project name to summarize"
            }));
            props
        },
        required: Some(vec!["project".to_string()]),
    }
}

fn input_schema_summarize_day() -> InputSchema {
    InputSchema {
        schema_type: "object".to_string(),
        properties: serde_json::Map::new(),
        required: None,
    }
}

fn input_schema_create_stitch() -> InputSchema {
    InputSchema {
        schema_type: "object".to_string(),
        properties: {
            let mut props = serde_json::Map::new();
            props.insert("project".to_string(), json!({
                "type": "string",
                "description": "Target project for the stitch"
            }));
            props.insert("title".to_string(), json!({
                "type": "string",
                "description": "Stitch title"
            }));
            props.insert("description".to_string(), json!({
                "type": "string",
                "description": "Optional stitch description"
            }));
            props.insert("kind".to_string(), json!({
                "type": "string",
                "description": "Stitch kind",
                "enum": ["investigation", "fix", "feature"],
                "default": "investigation"
            }));
            props.insert("priority".to_string(), json!({
                "type": "number",
                "description": "Bead priority (0-9, default: 2)",
                "minimum": 0,
                "maximum": 9
            }));
            props
        },
        required: Some(vec!["project".to_string(), "title".to_string()]),
    }
}

fn input_schema_create_bead() -> InputSchema {
    InputSchema {
        schema_type: "object".to_string(),
        properties: {
            let mut props = serde_json::Map::new();
            props.insert("project".to_string(), json!({
                "type": "string",
                "description": "Target project for the bead"
            }));
            props.insert("title".to_string(), json!({
                "type": "string",
                "description": "Bead title"
            }));
            props.insert("description".to_string(), json!({
                "type": "string",
                "description": "Optional bead description"
            }));
            props.insert("issue_type".to_string(), json!({
                "type": "string",
                "description": "Bead issue type",
                "enum": ["task", "bug", "epic", "genesis", "review", "fix"],
                "default": "task"
            }));
            props.insert("priority".to_string(), json!({
                "type": "number",
                "description": "Bead priority (0-9, default: 2)",
                "minimum": 0,
                "maximum": 9
            }));
            props.insert("labels".to_string(), json!({
                "type": "array",
                "items": { "type": "string" },
                "description": "Labels for the bead"
            }));
            props.insert("parent_bead_id".to_string(), json!({
                "type": "string",
                "description": "Parent bead ID to inherit stitch:* labels from (Hook 4). When set, stitch labels are automatically propagated to the new bead."
            }));
            props
        },
        required: Some(vec!["project".to_string(), "title".to_string()]),
    }
}

fn input_schema_escalate_to_operator() -> InputSchema {
    InputSchema {
        schema_type: "object".to_string(),
        properties: {
            let mut props = serde_json::Map::new();
            props.insert("message".to_string(), json!({
                "type": "string",
                "description": "Message to display to the operator"
            }));
            props
        },
        required: Some(vec!["message".to_string()]),
    }
}

// -----------------------------------------------------------------------
// Output schema builders
// -----------------------------------------------------------------------

fn output_schema_find_stitches() -> OutputSchema {
    OutputSchema {
        schema_type: "object".to_string(),
        properties: {
            let mut props = serde_json::Map::new();
            props.insert("stitches".to_string(), json!({
                "type": "array",
                "description": "Array of stitch objects",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "string", "description": "Stitch UUID" },
                        "project": { "type": "string", "description": "Project name" },
                        "kind": { "type": "string", "description": "Stitch kind (operator, dictated, worker, ad-hoc)" },
                        "title": { "type": "string", "description": "Stitch title" },
                        "created_by": { "type": "string", "description": "Creator identifier" },
                        "created_at": { "type": "string", "description": "ISO 8601 timestamp" },
                        "last_activity_at": { "type": "string", "description": "ISO 8601 timestamp of last activity" }
                    }
                }
            }));
            props
        },
        required: Some(vec!["stitches".to_string()]),
    }
}

fn output_schema_read_stitch() -> OutputSchema {
    OutputSchema {
        schema_type: "object".to_string(),
        properties: {
            let mut props = serde_json::Map::new();
            props.insert("stitch".to_string(), json!({
                "type": "object",
                "description": "Stitch metadata",
                "properties": {
                    "id": { "type": "string" },
                    "project": { "type": "string" },
                    "kind": { "type": "string" },
                    "title": { "type": "string" },
                    "created_by": { "type": "string" },
                    "created_at": { "type": "string" },
                    "last_activity_at": { "type": "string" },
                    "participants": { "type": "array", "items": { "type": "string" } }
                }
            }));
            props.insert("messages".to_string(), json!({
                "type": "array",
                "description": "Messages in this stitch",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "string" },
                        "ts": { "type": "string" },
                        "role": { "type": "string" },
                        "content": { "type": "string" },
                        "tokens": { "type": "number" }
                    }
                }
            }));
            props.insert("linked_beads".to_string(), json!({
                "type": "array",
                "description": "Beads linked to this stitch with live status",
                "items": {
                    "type": "object",
                    "properties": {
                        "bead_id": { "type": "string" },
                        "workspace": { "type": "string" },
                        "relationship": { "type": "string" },
                        "live_status": {
                            "type": "object",
                            "description": "Live status from in-memory bead store",
                            "properties": {
                                "title": { "type": "string" },
                                "status": { "type": "string" },
                                "priority": { "type": "number" },
                                "issue_type": { "type": "string" },
                                "created_by": { "type": "string" },
                                "dependencies": { "type": "array", "items": { "type": "string" } }
                            }
                        }
                    }
                }
            }));
            props.insert("touched_files".to_string(), json!({
                "type": "array",
                "description": "Files mentioned in stitch messages, sorted by mention count",
                "items": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" },
                        "mention_count": { "type": "number" }
                    }
                }
            }));
            props.insert("cost_duration".to_string(), json!({
                "type": "object",
                "description": "Token and wall-clock cost/duration roll-up",
                "properties": {
                    "total_tokens": { "type": "number" },
                    "message_count": { "type": "number" },
                    "wall_clock": { "type": "string" },
                    "first_message_ts": { "type": "string" },
                    "last_message_ts": { "type": "string" }
                }
            }));
            props.insert("link_graph".to_string(), json!({
                "type": "object",
                "description": "Stitch-to-stitch link graph (incoming and outgoing)",
                "properties": {
                    "outgoing": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "stitch_id": { "type": "string" },
                                "kind": { "type": "string" },
                                "title": { "type": "string" }
                            }
                        }
                    },
                    "incoming": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "stitch_id": { "type": "string" },
                                "kind": { "type": "string" },
                                "title": { "type": "string" }
                            }
                        }
                    }
                }
            }));
            props.insert("elapsed_ms".to_string(), json!({
                "type": "number",
                "description": "Server-side processing time in milliseconds"
            }));
            props
        },
        required: Some(vec![
            "stitch".to_string(),
            "messages".to_string(),
            "linked_beads".to_string(),
            "touched_files".to_string(),
            "cost_duration".to_string(),
            "link_graph".to_string(),
        ]),
    }
}

fn output_schema_find_beads() -> OutputSchema {
    OutputSchema {
        schema_type: "object".to_string(),
        properties: {
            let mut props = serde_json::Map::new();
            props.insert("beads".to_string(), json!({
                "type": "array",
                "description": "Array of bead objects",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "string", "description": "Bead ID" },
                        "title": { "type": "string", "description": "Bead title" },
                        "status": { "type": "string", "description": "open or closed" },
                        "priority": { "type": "number", "description": "Priority 0-9" },
                        "issue_type": { "type": "string", "description": "task, bug, epic, genesis, review, fix" }
                    }
                }
            }));
            props
        },
        required: Some(vec!["beads".to_string()]),
    }
}

fn output_schema_read_bead() -> OutputSchema {
    OutputSchema {
        schema_type: "object".to_string(),
        properties: {
            let mut props = serde_json::Map::new();
            props.insert("id".to_string(), json!({ "type": "string" }));
            props.insert("title".to_string(), json!({ "type": "string" }));
            props.insert("description".to_string(), json!({ "type": "string" }));
            props.insert("status".to_string(), json!({ "type": "string" }));
            props.insert("priority".to_string(), json!({ "type": "number" }));
            props.insert("issue_type".to_string(), json!({ "type": "string" }));
            props.insert("dependencies".to_string(), json!({
                "type": "array",
                "items": { "type": "string" }
            }));
            props
        },
        required: None,
    }
}

fn output_schema_read_file() -> OutputSchema {
    OutputSchema {
        schema_type: "object".to_string(),
        properties: {
            let mut props = serde_json::Map::new();
            props.insert("content".to_string(), json!({
                "type": "string",
                "description": "File contents as UTF-8 text"
            }));
            props
        },
        required: Some(vec!["content".to_string()]),
    }
}

fn output_schema_grep() -> OutputSchema {
    OutputSchema {
        schema_type: "object".to_string(),
        properties: {
            let mut props = serde_json::Map::new();
            props.insert("matches".to_string(), json!({
                "type": "array",
                "description": "Array of grep match objects",
                "items": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Relative file path" },
                        "line_number": { "type": "number", "description": "1-based line number" },
                        "line": { "type": "string", "description": "Matched line content" }
                    }
                }
            }));
            props
        },
        required: Some(vec!["matches".to_string()]),
    }
}

fn output_schema_search_conversations() -> OutputSchema {
    OutputSchema {
        schema_type: "object".to_string(),
        properties: {
            let mut props = serde_json::Map::new();
            props.insert("results".to_string(), json!({
                "type": "array",
                "description": "Array of matching conversation messages",
                "items": {
                    "type": "object",
                    "properties": {
                        "stitch_id": { "type": "string" },
                        "timestamp": { "type": "string" },
                        "role": { "type": "string" },
                        "content": { "type": "string" },
                        "project": { "type": "string" }
                    }
                }
            }));
            props
        },
        required: Some(vec!["results".to_string()]),
    }
}

fn output_schema_summarize_project() -> OutputSchema {
    OutputSchema {
        schema_type: "object".to_string(),
        properties: {
            let mut props = serde_json::Map::new();
            props.insert("project".to_string(), json!({ "type": "string" }));
            props.insert("stitch_count".to_string(), json!({ "type": "number" }));
            props.insert("recent_actions_24h".to_string(), json!({ "type": "number" }));
            props.insert("generated_at".to_string(), json!({ "type": "string" }));
            props
        },
        required: Some(vec!["project".to_string(), "stitch_count".to_string(), "recent_actions_24h".to_string()]),
    }
}

fn output_schema_summarize_day() -> OutputSchema {
    OutputSchema {
        schema_type: "object".to_string(),
        properties: {
            let mut props = serde_json::Map::new();
            props.insert("date".to_string(), json!({ "type": "string" }));
            props.insert("total_stitches".to_string(), json!({ "type": "number" }));
            props.insert("active_stitches_24h".to_string(), json!({ "type": "number" }));
            props.insert("today_actions".to_string(), json!({ "type": "number" }));
            props.insert("generated_at".to_string(), json!({ "type": "string" }));
            props
        },
        required: Some(vec!["date".to_string(), "total_stitches".to_string(), "active_stitches_24h".to_string()]),
    }
}

fn output_schema_create_stitch() -> OutputSchema {
    OutputSchema {
        schema_type: "object".to_string(),
        properties: {
            let mut props = serde_json::Map::new();
            props.insert("draft_id".to_string(), json!({
                "type": "string",
                "description": "ID of the created draft"
            }));
            props.insert("status".to_string(), json!({
                "type": "string",
                "description": "Draft status (always 'pending' on creation)"
            }));
            props.insert("title".to_string(), json!({ "type": "string" }));
            props.insert("kind".to_string(), json!({ "type": "string" }));
            props.insert("project".to_string(), json!({ "type": "string" }));
            props.insert("created_at".to_string(), json!({ "type": "string" }));
            props.insert("message".to_string(), json!({
                "type": "string",
                "description": "Explanation that the draft is pending operator review"
            }));
            props
        },
        required: Some(vec!["draft_id".to_string(), "status".to_string(), "title".to_string()]),
    }
}

fn output_schema_create_bead() -> OutputSchema {
    OutputSchema {
        schema_type: "object".to_string(),
        properties: {
            let mut props = serde_json::Map::new();
            props.insert("id".to_string(), json!({
                "type": "string",
                "description": "ID of the created bead"
            }));
            props.insert("title".to_string(), json!({ "type": "string" }));
            props.insert("project".to_string(), json!({ "type": "string" }));
            props.insert("labels".to_string(), json!({
                "type": "array",
                "items": { "type": "string" },
                "description": "Labels on the created bead (including inherited stitch labels)"
            }));
            props.insert("parent_bead_id".to_string(), json!({
                "type": "string",
                "description": "Parent bead ID if stitch labels were inherited"
            }));
            props
        },
        required: Some(vec!["id".to_string(), "title".to_string(), "project".to_string()]),
    }
}

fn output_schema_escalate_to_operator() -> OutputSchema {
    OutputSchema {
        schema_type: "object".to_string(),
        properties: {
            let mut props = serde_json::Map::new();
            props.insert("message".to_string(), json!({
                "type": "string",
                "description": "Confirmation message"
            }));
            props
        },
        required: Some(vec!["message".to_string()]),
    }
}
