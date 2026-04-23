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
                description: "Create a new stitch with one or more beads. This is the ONLY write operation exposed via MCP. Internally issues br create calls.".to_string(),
                input_schema: input_schema_create_stitch(),
                output_schema: Some(output_schema_create_stitch()),
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

        // Read stitch details from fleet.db
        let stitch = self.query_stitch_detail_from_db(stitch_id)
            .map_err(|e| format!("Failed to read stitch: {}", e))?;

        let content = serde_json::to_string_pretty(&stitch)
            .map_err(|e| format!("Failed to serialize result: {}", e))?;

        Ok(ToolCallResult {
            content: vec![Content::Text { text: content }],
            is_error: None,
        })
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

        let project_path = self.projects.get(project)
            .ok_or(format!("Project '{}' not found", project))?;

        // This is the ONE write operation: create stitch via the daemon's API
        // or via br create directly
        let result = self.create_stitch_via_br(project_path, title, description, kind, args)
            .map_err(|e| format!("Failed to create stitch: {}", e))?;

        let content = serde_json::to_string_pretty(&result)
            .map_err(|e| format!("Failed to serialize result: {}", e))?;

        Ok(ToolCallResult {
            content: vec![Content::Text { text: content }],
            is_error: None,
        })
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
        writeln!(file, "{}", entry.to_string())
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

    fn query_stitches_from_db(&self, project: &str, _args: &Map<String, Value>) -> Result<Vec<Value>, String> {
        use rusqlite::Connection;

        let conn = Connection::open(&self.fleet_db_path)
            .map_err(|e| format!("Failed to open fleet.db: {}", e))?;

        let mut stmt = conn.prepare(
            "SELECT id, project, kind, title, created_by, created_at, last_activity_at
             FROM stitches
             WHERE project = ?1
             ORDER BY last_activity_at DESC"
        ).map_err(|e| format!("Failed to prepare query: {}", e))?;

        let rows = stmt.query_map([project], |row| {
            Ok(json!({
                "id": row.get::<_, String>(0)?,
                "project": row.get::<_, String>(1)?,
                "kind": row.get::<_, String>(2)?,
                "title": row.get::<_, String>(3)?,
                "created_by": row.get::<_, String>(4)?,
                "created_at": row.get::<_, String>(5)?,
                "last_activity_at": row.get::<_, String>(6)?,
            }))
        }).map_err(|e| format!("Failed to execute query: {}", e))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| format!("Row error: {}", e))?);
        }

        Ok(results)
    }

    fn query_stitch_detail_from_db(&self, stitch_id: &str) -> Result<Value, String> {
        use rusqlite::Connection;

        let conn = Connection::open(&self.fleet_db_path)
            .map_err(|e| format!("Failed to open fleet.db: {}", e))?;

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
            Ok(json!({
                "id": row.get::<_, String>(0)?,
                "ts": row.get::<_, String>(1)?,
                "role": row.get::<_, String>(2)?,
                "content": row.get::<_, String>(3)?,
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
        use rusqlite::Connection;

        let conn = Connection::open(&self.fleet_db_path)
            .map_err(|e| format!("Failed to open fleet.db: {}", e))?;

        let pattern = format!("%{}%", query);

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
                Ok(json!({
                    "stitch_id": row.get::<_, String>(0)?,
                    "timestamp": row.get::<_, String>(1)?,
                    "role": row.get::<_, String>(2)?,
                    "content": row.get::<_, String>(3)?,
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
                Ok(json!({
                    "stitch_id": row.get::<_, String>(0)?,
                    "timestamp": row.get::<_, String>(1)?,
                    "role": row.get::<_, String>(2)?,
                    "content": row.get::<_, String>(3)?,
                    "project": row.get::<_, String>(4)?,
                }))
            }).map_err(|e| format!("Failed to execute query: {}", e))?
                .collect();

            Ok(rows.map_err(|e| format!("Row error: {}", e))?)
        }
    }

    fn generate_project_summary(&self, project: &str) -> Result<Value, String> {
        use rusqlite::Connection;

        let conn = Connection::open(&self.fleet_db_path)
            .map_err(|e| format!("Failed to open fleet.db: {}", e))?;

        // Get stitch counts
        let stitch_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM stitches WHERE project = ?1",
            [project],
            |row| row.get(0),
        ).unwrap_or(0);

        // Get recent activity from audit log
        let recent_actions: i64 = conn.query_row(
            "SELECT COUNT(*) FROM actions WHERE project = ?1 AND ts > datetime('now', '-24 hours')",
            [project],
            |row| row.get(0),
        ).unwrap_or(0);

        Ok(json!({
            "project": project,
            "stitch_count": stitch_count,
            "recent_actions_24h": recent_actions,
            "generated_at": chrono::Utc::now().to_rfc3339(),
        }))
    }

    fn generate_day_summary(&self) -> Result<Value, String> {
        use rusqlite::Connection;

        let conn = Connection::open(&self.fleet_db_path)
            .map_err(|e| format!("Failed to open fleet.db: {}", e))?;

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

    fn create_stitch_via_br(&self, project_path: &str, title: &str, description: Option<&str>, kind: &str, _args: &Map<String, Value>) -> Result<Value, String> {
        // This creates a stitch by calling br create — the ONE allowed write verb
        #[cfg(any(
            feature = "create-only-write",
            not(any(feature = "zero-write-v01", feature = "create-only-write"))
        ))]
        {
            let mut cmd = crate::br_verbs::invoke_br_create(&[]);
            cmd.arg(title)
                .arg("--type").arg("task")
                .arg("--labels").arg(format!("stitch-kind:{}", kind))
                .arg("--actor").arg(format!("hoop-mcp:{}", self.actor))
                .arg("--silent");

            if let Some(desc) = description {
                if !desc.is_empty() {
                    cmd.arg("--description").arg(desc);
                }
            }

            let output = cmd.current_dir(project_path)
                .output()
                .map_err(|e| format!("Failed to execute br: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("br create failed: {}", stderr));
            }

            let bead_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

            Ok(json!({
                "bead_id": bead_id,
                "title": title,
                "kind": kind,
                "created_at": chrono::Utc::now().to_rfc3339(),
            }))
        }

        #[cfg(feature = "zero-write-v01")]
        {
            let _ = (project_path, title, description, kind);
            Err("create_stitch is not available under zero-write-v01".to_string())
        }
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
                    "last_activity_at": { "type": "string" }
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
                        "content": { "type": "string" }
                    }
                }
            }));
            props.insert("linked_beads".to_string(), json!({
                "type": "array",
                "description": "Beads linked to this stitch",
                "items": {
                    "type": "object",
                    "properties": {
                        "bead_id": { "type": "string" },
                        "workspace": { "type": "string" },
                        "relationship": { "type": "string" }
                    }
                }
            }));
            props
        },
        required: Some(vec!["stitch".to_string(), "messages".to_string(), "linked_beads".to_string()]),
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
            props.insert("bead_id".to_string(), json!({
                "type": "string",
                "description": "ID of the created bead"
            }));
            props.insert("title".to_string(), json!({ "type": "string" }));
            props.insert("kind".to_string(), json!({ "type": "string" }));
            props.insert("created_at".to_string(), json!({ "type": "string" }));
            props
        },
        required: Some(vec!["bead_id".to_string(), "title".to_string()]),
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
