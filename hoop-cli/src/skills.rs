//! Skills CLI — import, enable, list, and inspect skills
//!
//! Skills are executable plugins with manifest.yml that extend agent capabilities.
//! This module implements the quarantine/inspection workflow for skill imports (§22.7, §22.8).

use anyhow::{anyhow, Result};
use chrono::Utc;
use clap::{ArgGroup, Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Skill import summary shown to operator before enabling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillImportSummary {
    /// Skill name from manifest
    pub name: String,
    /// Agent-facing description
    pub description: String,
    /// Human-readable summary
    pub summary: String,
    /// Visibility scope
    pub scope: String,
    /// Projects (if scope=project)
    pub projects: Vec<String>,
    /// Pattern (if scope=pattern)
    pub pattern: Option<String>,
    /// Run script file info
    pub run_info: RunScriptInfo,
    /// Full manifest YAML for inspection
    pub manifest_yaml: String,
    /// Import timestamp
    pub imported_at: String,
    /// Import source path
    pub source_path: PathBuf,
    /// Pending directory path
    pub pending_path: PathBuf,
}

/// Run script information for security inspection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunScriptInfo {
    /// Whether run file exists
    pub exists: bool,
    /// File size in bytes
    pub size: Option<u64>,
    /// Executable bit status
    pub executable: bool,
    /// Shebang detected (first line if starts with #!)
    pub shebang: Option<String>,
    /// File extension (if any)
    pub extension: Option<String>,
    /// SHA-256 hash of run script contents
    pub sha256: Option<String>,
}

/// Skill enable event for audit log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillEnableEvent {
    /// Event ID
    pub id: String,
    /// Timestamp
    pub ts: String,
    /// Operator who enabled the skill
    pub actor: String,
    /// Skill name
    pub skill_name: String,
    /// Previous SHA-256 (if re-enabling)
    pub prev_sha256: Option<String>,
    /// New SHA-256
    pub new_sha256: String,
}

/// Skills commands
#[derive(clap::Subcommand, Debug)]
pub enum SkillsCommands {
    /// Import a skill from a path (quarantined until enabled)
    Import {
        /// Path to skill directory or tarball
        path: String,
    },
    /// Enable a pending skill (move from .pending/ to active)
    Enable {
        /// Skill name to enable
        name: String,
    },
    /// Disable an active skill (move to .pending/)
    Disable {
        /// Skill name to disable
        name: String,
    },
    /// List all skills (active and pending)
    List {
        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },
    /// Show detailed information about a skill
    Show {
        /// Skill name
        name: String,
    },
    /// Remove a skill (from active or pending)
    Remove {
        /// Skill name to remove
        name: String,
    },
}

/// Get the skills directory
pub fn skills_dir() -> Result<PathBuf> {
    let mut path = dirs::home_dir()
        .ok_or_else(|| anyhow!("Cannot determine home directory"))?;
    path.push(".hoop");
    path.push("skills");
    Ok(path)
}

/// Get the pending skills directory
pub fn pending_dir() -> Result<PathBuf> {
    let mut path = skills_dir()?;
    path.push(".pending");
    Ok(path)
}

/// Import a skill from a path to the .pending/ quarantine directory
pub fn import_skill(path: &str) -> Result<SkillImportSummary> {
    let source_path = PathBuf::from(path);

    if !source_path.exists() {
        return Err(anyhow!("Source path does not exist: {}", path));
    }

    // Determine if this is a directory or tarball
    let temp_skill_dir = if source_path.is_dir() {
        // It's a directory, use it directly
        source_path.clone()
    } else {
        // Assume tarball - extract to temp dir
        return Err(anyhow!("Tarball import not yet implemented - please extract and import from directory"));
    };

    // Read manifest.yml
    let manifest_path = temp_skill_dir.join("manifest.yml");
    if !manifest_path.exists() {
        return Err(anyhow!("manifest.yml not found in {}", temp_skill_dir.display()));
    }

    let manifest_yaml = fs::read_to_string(&manifest_path)
        .map_err(|e| anyhow!("Failed to read manifest.yml: {}", e))?;

    // Parse manifest to get name
    let manifest: SkillManifest = serde_yaml::from_str(&manifest_yaml)
        .map_err(|e| anyhow!("Failed to parse manifest.yml: {}", e))?;

    let skill_name = manifest.name.clone();

    // Gather run script info
    let run_path = temp_skill_dir.join("run");
    let run_info = analyze_run_script(&run_path)?;

    // Create pending directory if it doesn't exist
    let pending_base = pending_dir()?;
    fs::create_dir_all(&pending_base)
        .map_err(|e| anyhow!("Failed to create pending directory: {}", e))?;

    let pending_path = pending_base.join(&skill_name);

    // Check if skill already exists in pending or active
    if pending_path.exists() {
        return Err(anyhow!("Skill '{}' is already pending. Use `hoop skills enable {}` or `hoop skills remove {}` first.", skill_name, skill_name, skill_name));
    }

    let active_path = skills_dir()?.join(&skill_name);
    if active_path.exists() {
        return Err(anyhow!("Skill '{}' is already active. Use `hoop skills disable {}` first.", skill_name, skill_name));
    }

    // Copy skill directory to pending
    copy_directory(&temp_skill_dir, &pending_path)
        .map_err(|e| anyhow!("Failed to copy skill to pending: {}", e))?;

    let summary = SkillImportSummary {
        name: skill_name,
        description: manifest.description,
        summary: manifest.summary,
        scope: format!("{:?}", manifest.scope).to_lowercase(),
        projects: manifest.projects,
        pattern: manifest.pattern,
        run_info,
        manifest_yaml,
        imported_at: Utc::now().to_rfc3339(),
        source_path: temp_skill_dir,
        pending_path,
    };

    Ok(summary)
}

/// Enable a pending skill by moving it from .pending/ to active
pub fn enable_skill(name: &str) -> Result<SkillEnableEvent> {
    let pending_path = pending_dir()?.join(name);
    let active_path = skills_dir()?.join(name);

    if !pending_path.exists() {
        return Err(anyhow!("Skill '{}' is not pending. Import it first with `hoop skills import <path>`", name));
    }

    if active_path.exists() {
        return Err(anyhow!("Skill '{}' is already active. Disable it first with `hoop skills disable {}`.", name, name));
    }

    // Read run file for SHA-256
    let run_path = pending_path.join("run");
    let new_sha256 = compute_sha256(&run_path)?;

    // Get previous SHA-256 if this was previously enabled
    let prev_sha256 = get_previous_sha256(name)?;

    // Move from pending to active
    fs::rename(&pending_path, &active_path)
        .map_err(|e| anyhow!("Failed to move skill from pending to active: {}", e))?;

    // Write audit log entry
    let actor = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
    let event = SkillEnableEvent {
        id: Uuid::new_v4().to_string(),
        ts: Utc::now().to_rfc3339(),
        actor: actor.clone(),
        skill_name: name.to_string(),
        prev_sha256,
        new_sha256,
    };

    write_skill_enable_audit(&event)?;

    Ok(event)
}

/// Disable an active skill by moving it to .pending/
pub fn disable_skill(name: &str) -> Result<()> {
    let active_path = skills_dir()?.join(name);
    let pending_path = pending_dir()?.join(name);

    if !active_path.exists() {
        return Err(anyhow!("Skill '{}' is not active", name));
    }

    if pending_path.exists() {
        return Err(anyhow!("Skill '{}' is already pending. Remove it first with `hoop skills remove {}`.", name, name));
    }

    // Move from active to pending
    fs::rename(&active_path, &pending_path)
        .map_err(|e| anyhow!("Failed to move skill from active to pending: {}", e))?;

    Ok(())
}

/// List all skills (active and pending)
pub fn list_skills() -> Result<Vec<SkillListEntry>> {
    let mut entries = Vec::new();

    let skills_base = skills_dir()?;
    let pending_base = pending_dir()?;

    // List active skills
    if let Ok(dir) = fs::read_dir(&skills_base) {
        for entry in dir.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() && !path.file_name().is_none_or(|n| n.to_string_lossy().starts_with('.')) {
                if let Some(entry) = read_skill_entry(&path, SkillState::Active) {
                    entries.push(entry);
                }
            }
        }
    }

    // List pending skills
    if pending_base.exists() {
        if let Ok(dir) = fs::read_dir(&pending_base) {
            for entry in dir.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(entry) = read_skill_entry(&path, SkillState::Pending) {
                        entries.push(entry);
                    }
                }
            }
        }
    }

    entries.sort_by(|a, b| {
        match (&a.state, &b.state) {
            (SkillState::Pending, SkillState::Active) => std::cmp::Ordering::Less,
            (SkillState::Active, SkillState::Pending) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });

    Ok(entries)
}

/// Show detailed information about a skill
pub fn show_skill(name: &str) -> Result<SkillDetail> {
    let skills_base = skills_dir()?;
    let pending_base = pending_dir()?;

    let (skill_path, state) = if skills_base.join(name).exists() {
        (skills_base.join(name), SkillState::Active)
    } else if pending_base.join(name).exists() {
        (pending_base.join(name), SkillState::Pending)
    } else {
        return Err(anyhow!("Skill '{}' not found", name));
    };

    let manifest_path = skill_path.join("manifest.yml");
    let manifest_yaml = fs::read_to_string(&manifest_path)?;
    let manifest: SkillManifest = serde_yaml::from_str(&manifest_yaml)?;

    let run_path = skill_path.join("run");
    let run_info = analyze_run_script(&run_path)?;

    // Read README if it exists
    let readme = skill_path.join("README.md");
    let readme_content = if readme.exists() {
        Some(fs::read_to_string(&readme)?)
    } else {
        None
    };

    Ok(SkillDetail {
        name: name.to_string(),
        state,
        manifest,
        manifest_yaml,
        run_info,
        readme: readme_content,
        path: skill_path,
    })
}

/// Remove a skill (from active or pending)
pub fn remove_skill(name: &str) -> Result<()> {
    let skills_base = skills_dir()?;
    let pending_base = pending_dir()?;

    let active_path = skills_base.join(name);
    let pending_path = pending_base.join(name);

    let target_path = if active_path.exists() {
        active_path
    } else if pending_path.exists() {
        pending_path
    } else {
        return Err(anyhow!("Skill '{}' not found", name));
    };

    fs::remove_dir_all(&target_path)
        .map_err(|e| anyhow!("Failed to remove skill '{}': {}", name, e))?;

    Ok(())
}

// Internal types

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SkillManifest {
    name: String,
    description: String,
    summary: String,
    #[serde(default)]
    scope: SkillScope,
    #[serde(default)]
    projects: Vec<String>,
    pattern: Option<String>,
    #[serde(default)]
    args_schema: serde_json::Value,
    #[serde(default)]
    timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum SkillScope {
    Global,
    Project,
    Pattern,
}

impl Default for SkillScope {
    fn default() -> Self {
        SkillScope::Global
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillListEntry {
    pub name: String,
    pub state: SkillState,
    pub description: String,
    pub executable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SkillState {
    Active,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDetail {
    pub name: String,
    pub state: SkillState,
    pub manifest: SkillManifestPublic,
    pub manifest_yaml: String,
    pub run_info: RunScriptInfo,
    pub readme: Option<String>,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillManifestPublic {
    pub name: String,
    pub description: String,
    pub summary: String,
    pub scope: String,
    pub projects: Vec<String>,
    pub pattern: Option<String>,
    pub timeout_secs: u64,
}

// Internal helpers

fn analyze_run_script(path: &Path) -> Result<RunScriptInfo> {
    let exists = path.exists();

    if !exists {
        return Ok(RunScriptInfo {
            exists: false,
            size: None,
            executable: false,
            shebang: None,
            extension: None,
            sha256: None,
        });
    }

    let metadata = fs::metadata(path)
        .map_err(|e| anyhow!("Failed to read run script metadata: {}", e))?;
    let size = metadata.len();
    let executable = metadata.permissions().mode() & 0o111 != 0;

    let extension = path.extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_string());

    let content = fs::read(path)
        .map_err(|e| anyhow!("Failed to read run script: {}", e))?;

    let shebang = if content.starts_with(b"#!") {
        content.iter()
            .take_while(|&&b| b != b'\n')
            .map(|&b| b as char)
            .collect::<String>()
    } else {
        None
    };

    let sha256 = Some(compute_sha256_from_bytes(&content));

    Ok(RunScriptInfo {
        exists: true,
        size: Some(size),
        executable,
        shebang,
        extension,
        sha256,
    })
}

fn compute_sha256(path: &Path) -> Result<String> {
    let content = fs::read(path)
        .map_err(|e| anyhow!("Failed to read file for SHA-256: {}", e))?;
    Ok(compute_sha256_from_bytes(&content))
}

fn compute_sha256_from_bytes(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
}

fn copy_directory(source: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)
        .map_err(|e| anyhow!("Failed to create destination directory: {}", e))?;

    for entry in fs::read_dir(source)
        .map_err(|e| anyhow!("Failed to read source directory: {}", e))?
    {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src = entry.path();
        let dst = dest.join(entry.file_name());

        if ty.is_dir() {
            copy_directory(&src, &dst)?;
        } else {
            fs::copy(&src, &dst)
                .map_err(|e| anyhow!("Failed to copy file {}: {}", src.display(), e))?;
        }
    }

    Ok(())
}

fn read_skill_entry(path: &Path, state: SkillState) -> Option<SkillListEntry> {
    let manifest_path = path.join("manifest.yml");
    let manifest_content = fs::read_to_string(&manifest_path).ok()?;
    let manifest: SkillManifest = serde_yaml::from_str(&manifest_content).ok()?;

    let run_path = path.join("run");
    let executable = run_path.exists()
        && fs::metadata(&run_path)
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false);

    Some(SkillListEntry {
        name: manifest.name,
        state,
        description: manifest.description,
        executable,
    })
}

fn write_skill_enable_audit(event: &SkillEnableEvent) -> Result<()> {
    use rusqlite::params;

    let db_path = fleet_db_path()?;
    if !db_path.exists() {
        // fleet.db may not exist yet - skip audit
        return Ok(());
    }

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| anyhow!("Failed to open fleet.db: {}", e))?;

    // Fetch the most recent hash_self for hash chaining
    let hash_prev: String = conn
        .query_row(
            "SELECT hash_self FROM actions ORDER BY ts DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "0000000000000000000000000000000000000000000000000000000000000000".to_string());

    // Generate audit row ID and timestamp (use event's timestamp)
    let id = event.id.clone();
    let ts = event.ts.clone();

    // Build args_json with skill enable details
    let args_json = serde_json::json!({
        "skill_name": event.skill_name,
        "prev_sha256": event.prev_sha256,
        "new_sha256": event.new_sha256,
    });
    let args_json_str = serde_json::to_string(&args_json)?;

    // Compute hash_self from row content
    let hash_input = format!(
        "{}{}{}{}{}{}{}",
        id,
        ts,
        event.actor,
        "skill_enabled",
        event.skill_name,
        Option::<String>::None, // project
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
            event.actor,
            "skill_enabled",
            event.skill_name,
            Option::<String>::None,  // project
            args_json_str,
            "enabled",
            Option::<String>::None,  // error
            Option::<String>::None,  // source
            Option::<String>::None,  // stitch_id
            Option::<String>::None,  // args_hash
            hash_prev,
            hash_self,
        ],
    )
    .map_err(|e| anyhow!("Failed to insert skill enable audit row: {}", e))?;

    Ok(())
}

fn get_previous_sha256(name: &str) -> Result<Option<String>> {
    // This would check a history of enabled skills
    // For now, just return None - could be extended to track enable history
    Ok(None)
}

fn fleet_db_path() -> Result<PathBuf> {
    let mut path = dirs::home_dir()
        .ok_or_else(|| anyhow!("Cannot determine home directory"))?;
    path.push(".hoop");
    path.push("fleet.db");
    Ok(path)
}

impl From<SkillManifest> for SkillManifestPublic {
    fn from(m: SkillManifest) -> Self {
        SkillManifestPublic {
            name: m.name,
            description: m.description,
            summary: m.summary,
            scope: format!("{:?}", m.scope).to_lowercase(),
            projects: m.projects,
            pattern: m.pattern,
            timeout_secs: m.timeout_secs,
        }
    }
}

/// Handle skills commands
pub async fn handle_skills(cmd: SkillsCommands) -> anyhow::Result<()> {
    match cmd {
        SkillsCommands::Import { path } => {
            match import_skill(&path) {
                Ok(summary) => {
                    println!("Skill imported to quarantine:");
                    println!();
                    println!("  Name: {}", summary.name);
                    println!("  Description: {}", summary.description);
                    println!("  Summary: {}", summary.summary);
                    println!("  Scope: {}", summary.scope);
                    if !summary.projects.is_empty() {
                        println!("  Projects: {}", summary.projects.join(", "));
                    }
                    if let Some(pattern) = &summary.pattern {
                        println!("  Pattern: {}", pattern);
                    }
                    println!();
                    println!("  Run script:");
                    println!("    Exists: {}", summary.run_info.exists);
                    if let Some(size) = summary.run_info.size {
                        println!("    Size: {} bytes", size);
                    }
                    println!("    Executable: {}", summary.run_info.executable);
                    if let Some(shebang) = &summary.run_info.shebang {
                        println!("    Shebang: {}", shebang);
                    }
                    if let Some(ext) = &summary.run_info.extension {
                        println!("    Extension: {}", ext);
                    }
                    if let Some(hash) = &summary.run_info.sha256 {
                        println!("    SHA-256: {}", hash);
                    }
                    println!();
                    println!("  Manifest YAML:");
                    for line in summary.manifest_yaml.lines() {
                        println!("    {}", line);
                    }
                    println!();
                    println!("Imported at: {}", summary.imported_at);
                    println!("Pending path: {}", summary.pending_path.display());
                    println!();
                    println!("Review the manifest and run script above, then enable with:");
                    println!("  hoop skills enable {}", summary.name);
                }
                Err(e) => {
                    eprintln!("Import failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        SkillsCommands::Enable { name } => {
            match enable_skill(&name) {
                Ok(event) => {
                    println!("Skill '{}' enabled successfully", name);
                    println!("  Enabled by: {}", event.actor);
                    println!("  At: {}", event.ts);
                    if let Some(prev) = &event.prev_sha256 {
                        println!("  Previous SHA-256: {}", prev);
                    }
                    println!("  New SHA-256: {}", event.new_sha256);
                }
                Err(e) => {
                    eprintln!("Enable failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        SkillsCommands::Disable { name } => {
            match disable_skill(&name) {
                Ok(()) => {
                    println!("Skill '{}' disabled", name);
                    println!("Moved to pending. Re-enable with: hoop skills enable {}", name);
                }
                Err(e) => {
                    eprintln!("Disable failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        SkillsCommands::List { json } => {
            match list_skills() {
                Ok(entries) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&entries)?);
                    } else {
                        if entries.is_empty() {
                            println!("No skills found");
                        } else {
                            let mut active_count = 0;
                            let mut pending_count = 0;

                            for entry in &entries {
                                if entry.state == SkillState::Active {
                                    active_count += 1;
                                } else {
                                    pending_count += 1;
                                }
                            }

                            println!("Skills ({} active, {} pending):", active_count, pending_count);
                            println!();

                            let mut current_state = None;
                            for entry in &entries {
                                if current_state != Some(entry.state.clone()) {
                                    println!("  [{}]", format!("{:?}", entry.state).to_lowercase());
                                    current_state = Some(entry.state.clone());
                                }
                                let exec_marker = if entry.executable { "" } else { " (not executable)" };
                                println!("    {}{} - {}", entry.name, exec_marker, entry.description);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("List failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        SkillsCommands::Show { name } => {
            match show_skill(&name) {
                Ok(detail) => {
                    println!("Skill: {}", detail.name);
                    println!("State: {}", format!("{:?}", detail.state).to_lowercase());
                    println!();
                    println!("Manifest:");
                    println!("  Name: {}", detail.manifest.name);
                    println!("  Description: {}", detail.manifest.description);
                    println!("  Summary: {}", detail.manifest.summary);
                    println!("  Scope: {}", detail.manifest.scope);
                    if !detail.manifest.projects.is_empty() {
                        println!("  Projects: {}", detail.manifest.projects.join(", "));
                    }
                    if let Some(pattern) = &detail.manifest.pattern {
                        println!("  Pattern: {}", pattern);
                    }
                    println!("  Timeout: {} seconds", detail.manifest.timeout_secs);
                    println!();
                    println!("Run script:");
                    println!("  Path: {}", detail.path.join("run").display());
                    println!("  Exists: {}", detail.run_info.exists);
                    if let Some(size) = detail.run_info.size {
                        println!("  Size: {} bytes", size);
                    }
                    println!("  Executable: {}", detail.run_info.executable);
                    if let Some(shebang) = &detail.run_info.shebang {
                        println!("  Shebang: {}", shebang);
                    }
                    if let Some(ext) = &detail.run_info.extension {
                        println!("  Extension: {}", ext);
                    }
                    if let Some(hash) = &detail.run_info.sha256 {
                        println!("  SHA-256: {}", hash);
                    }
                    println!();
                    println!("Full manifest YAML:");
                    for line in detail.manifest_yaml.lines() {
                        println!("  {}", line);
                    }
                    if let Some(readme) = &detail.readme {
                        println!();
                        println!("README.md:");
                        for line in readme.lines() {
                            println!("  {}", line);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Show failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        SkillsCommands::Remove { name } => {
            match remove_skill(&name) {
                Ok(()) => {
                    println!("Skill '{}' removed", name);
                }
                Err(e) => {
                    eprintln!("Remove failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}
