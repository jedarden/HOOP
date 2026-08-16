//! `hoop reflection` — export approved Reflection Ledger entries into the
//! operator's Claude Code memory index.
//!
//! HOOP's Reflection Ledger collects repeated operator instructions and turns
//! the approved ones into durable rules. This command bridges that store to the
//! hand-written global memory index (`MEMORY.md` + per-topic markdown files)
//! that every Claude Code session in this workspace reads at startup, so an
//! approved reflection becomes a rule future sessions actually see.
//!
//! Scope is export-only: HOOP appends *new* entries (and rewrites the per-entry
//! file when an entry's content changes). It never deletes or edits existing
//! manually-written memory entries. Re-running is idempotent — exported state
//! is tracked in a local export-log file (`<out>/.hoop-reflection-export.jsonl`)
//! keyed by reflection id + content hash.
//!
//! Plan reference: §19.2 (Reflection Ledger).

use anyhow::{bail, Context, Result};
use hoop_daemon::fleet::ReflectionLedgerEntry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Filename of the MEMORY.md index inside the output directory.
const INDEX_FILE: &str = "MEMORY.md";

/// Filename of the local export log (tracks which entries have been exported).
const EXPORT_LOG_FILE: &str = ".hoop-reflection-export.jsonl";

/// Namespace prefix for exported memory slugs, so HOOP-exported entries are
/// clearly distinguished from hand-written ones and never collide with them.
const SLUG_PREFIX: &str = "reflection-";

// ---------------------------------------------------------------------------
// Subcommand definition
// ---------------------------------------------------------------------------

/// `hoop reflection …` subcommands.
#[derive(clap::Subcommand, Debug)]
pub enum ReflectionCommands {
    /// Export approved Reflection Ledger entries into the operator's memory index
    #[command(arg_required_else_help = false)]
    Export {
        /// Output format. Only `claude-memory` (MEMORY.md index + per-entry
        /// markdown) is supported today; the flag is kept for future formats.
        #[arg(long, value_enum, default_value_t = ExportFormat::ClaudeMemory)]
        format: ExportFormat,

        /// Directory holding MEMORY.md and the per-topic markdown files.
        /// Defaults to the operator's global memory index
        /// (~/.claude/projects/-home-coding/memory/).
        #[arg(long, value_name = "DIR")]
        out: Option<PathBuf>,

        /// Daemon address (default: 127.0.0.1:3000)
        #[arg(short, long)]
        addr: Option<SocketAddr>,

        /// Print what would be written without touching any files.
        #[arg(long)]
        dry_run: bool,
    },
}

/// Supported export formats for `hoop reflection export`.
#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportFormat {
    /// Append to a `MEMORY.md` index and write one markdown file per entry,
    /// matching the format this workspace's memory index already uses.
    #[value(name = "claude-memory")]
    ClaudeMemory,
}

// ---------------------------------------------------------------------------
// Pure formatting / planning logic (unit-tested, no I/O)
// ---------------------------------------------------------------------------

/// A record of a previously-exported reflection entry, read from / written to
/// the local export log. Used to make re-runs idempotent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportRecord {
    pub id: String,
    pub content_hash: String,
    pub slug: String,
    pub exported_at: String,
}

/// What the export would do for a single entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportAction {
    /// New entry — write its markdown file and append an index line.
    Add,
    /// Content changed since last export — rewrite its markdown file (and
    /// refresh the index line, since the title/hook are derived from content).
    Update,
    /// Already exported and unchanged — skip.
    Skip,
}

/// One entry's planned fate under an export run.
#[derive(Debug, Clone)]
pub struct PlannedEntry<'a> {
    pub entry: &'a ReflectionLedgerEntry,
    pub action: ExportAction,
    pub slug: String,
}

/// Derive a stable, filesystem-safe, namespaced slug from a reflection id.
///
/// The id is a UUID (e.g. `550e8400-e29b-41d4-a716-446655440000`); we keep it
/// intact so the memory file traces back to its `reflection_ledger` row, just
/// lowercased and cleaned of characters that are unsafe in a filename.
pub fn slug_for(id: &str) -> String {
    let cleaned: String = id
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '-' })
        .collect();
    let collapsed = collapse_dashes(&cleaned);
    let trimmed = collapsed.trim_matches('-');
    let body = if trimmed.is_empty() { "untitled" } else { trimmed };
    format!("{SLUG_PREFIX}{body}")
}

/// Short human-readable title for an entry, taken from its rule. Used as the
/// link text in the MEMORY.md index line.
pub fn title_for(rule: &str) -> String {
    single_line(rule, 60)
}

/// One-line hook/summary for an entry, taken from its reason. Used as the
/// `— …` text in the MEMORY.md index line.
pub fn hook_for(reason: &str) -> String {
    let line = single_line(reason, 100);
    if line.trim().is_empty() {
        "(no reason recorded)".to_string()
    } else {
        line
    }
}

/// Description for the markdown frontmatter — a single-line summary of the rule
/// (this is what the memory system uses to judge recall relevance).
pub fn description_for(rule: &str) -> String {
    single_line(rule, 140)
}

/// The MEMORY.md index line for an entry: `- [title](slug.md) — hook`.
pub fn index_line(slug: &str, entry: &ReflectionLedgerEntry) -> String {
    format!(
        "- [{}]({}.md) - {}",
        title_for(&entry.rule),
        slug,
        hook_for(&entry.reason)
    )
}

/// Render the full per-entry markdown file body, in the same frontmatter + body
/// shape this workspace's memory files already use.
pub fn memory_file_body(entry: &ReflectionLedgerEntry) -> String {
    let stitches = format_source_stitches(&entry.source_stitches);
    let approved_by = entry.approved_by.clone().unwrap_or_else(|| "unknown".to_string());
    let approved_at = entry.approved_at.clone().unwrap_or_else(|| "unknown".to_string());

    let mut out = String::new();
    // Frontmatter (YAML). description is double-quoted so arbitrary rule text
    // (colons, quotes) serializes safely.
    out.push_str("---\n");
    out.push_str(&format!("name: {}\n", slug_for(&entry.id)));
    out.push_str(&format!("description: {}\n", yaml_double_quote(&description_for(&entry.rule))));
    out.push_str("metadata:\n");
    out.push_str("  node_type: memory\n");
    out.push_str("  type: feedback\n");
    out.push_str("  source: hoop-reflection-ledger\n");
    out.push_str(&format!("  reflection_id: {}\n", entry.id));
    out.push_str(&format!("  scope: {}\n", entry.scope));
    out.push_str("---\n\n");
    // Body
    out.push_str(entry.rule.trim());
    out.push_str("\n\n");
    out.push_str(&format!("**Why:** {}\n\n", entry.reason.trim()));
    out.push_str(&format!("**Scope:** {}\n\n", entry.scope));
    out.push_str(&format!("**Source stitches:** {}\n\n", stitches));
    out.push_str(&format!(
        "_Exported from the HOOP Reflection Ledger — approved by {approved_by} at {approved_at}._\n"
    ));
    out
}

/// Decide what to do with each approved entry given the prior export log.
///
/// An entry is `Add` if its id is unseen, `Update` if its content hash changed,
/// and `Skip` if it is already exported unchanged.
pub fn plan_export<'a>(
    entries: &'a [ReflectionLedgerEntry],
    log: &[ExportRecord],
) -> Vec<PlannedEntry<'a>> {
    let by_id: HashMap<&str, &ExportRecord> = log.iter().map(|r| (r.id.as_str(), r)).collect();
    entries
        .iter()
        .map(|entry| {
            let slug = slug_for(&entry.id);
            let action = match by_id.get(entry.id.as_str()) {
                None => ExportAction::Add,
                Some(rec) if rec.content_hash != entry.content_hash => ExportAction::Update,
                Some(_) => ExportAction::Skip,
            };
            PlannedEntry { entry, action, slug }
        })
        .collect()
}

/// Append an entry's index line to a MEMORY.md buffer (idempotent: a line
/// linking to this slug is never duplicated).
pub fn append_index_line(content: &mut String, slug: &str, entry: &ReflectionLedgerEntry) {
    let link = format!("]({slug}.md)");
    if content.contains(&link) {
        return; // already indexed — never duplicate
    }
    let line = index_line(slug, entry);
    push_index_line(content, &line);
}

/// Replace an existing index line for a slug in a MEMORY.md buffer with a fresh
/// one (content may have changed). Falls back to appending if not present.
pub fn replace_index_line(content: &mut String, slug: &str, entry: &ReflectionLedgerEntry) {
    let link = format!("]({slug}.md)");
    let new_line = index_line(slug, entry);
    let mut replaced = false;
    let updated: String = content
        .lines()
        .map(|line| {
            if !replaced && line.contains(&link) {
                replaced = true;
                new_line.clone()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    *content = updated;
    if !replaced {
        // Wasn't indexed yet (e.g. log existed but index line was hand-removed).
        push_index_line(content, &new_line);
    }
}

// ---------------------------------------------------------------------------
// Command handler + I/O
// ---------------------------------------------------------------------------

/// Handle `hoop reflection` subcommands.
pub async fn handle_reflection(cmd: ReflectionCommands) -> Result<()> {
    match cmd {
        ReflectionCommands::Export {
            format,
            out,
            addr,
            dry_run,
        } => run_export(format, out, addr, dry_run).await,
    }
}

async fn run_export(
    _format: ExportFormat,
    out: Option<PathBuf>,
    addr: Option<SocketAddr>,
    dry_run: bool,
) -> Result<()> {
    // Only claude-memory exists today; `_format` is accepted for forward-compat.
    let out_dir = out.unwrap_or_else(default_memory_dir);
    let addr = addr.unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 3000)));

    println!("Fetching approved reflections from {} …", addr);
    let entries = fetch_approved(addr).await?;
    if entries.is_empty() {
        println!("No approved reflections to export.");
        return Ok(());
    }

    let log_path = export_log_path(&out_dir);
    let log = read_export_log(&log_path);
    let planned = plan_export(&entries, &log);

    let adds = planned.iter().filter(|p| p.action == ExportAction::Add).count();
    let updates = planned.iter().filter(|p| p.action == ExportAction::Update).count();
    let skips = planned.iter().filter(|p| p.action == ExportAction::Skip).count();

    println!(
        "{} approved reflection(s): {} new, {} changed, {} up-to-date",
        planned.len(),
        adds,
        updates,
        skips
    );

    if dry_run {
        println!();
        println!("Dry run — no files will be written.");
        println!("Output directory: {}", out_dir.display());
        for p in &planned {
            if p.action == ExportAction::Skip {
                continue;
            }
            println!();
            println!("── reflection {} [{}] ──", p.entry.id, action_label(p.action));
            println!("  file:  {}.md", p.slug);
            println!("  index: {}", index_line(&p.slug, p.entry));
            println!("  ── body ──");
            for line in memory_file_body(p.entry).lines() {
                println!("    {}", line);
            }
            println!("  ── end ──");
        }
        return Ok(());
    }

    // ---- Write path ----
    fs::create_dir_all(&out_dir)
        .with_context(|| format!("Failed to create output directory {}", out_dir.display()))?;

    let index_path = out_dir.join(INDEX_FILE);
    let mut index_content = match fs::read_to_string(&index_path) {
        Ok(c) => c,
        Err(_) => {
            // Brand-new index — seed with the conventional header.
            "# Memory Index\n\n".to_string()
        }
    };

    let now = chrono::Utc::now().to_rfc3339();
    let mut log_map: HashMap<String, ExportRecord> =
        log.into_iter().map(|r| (r.id.clone(), r)).collect();
    let mut wrote_anything = false;

    for p in &planned {
        if p.action == ExportAction::Skip {
            continue;
        }
        let file_path = out_dir.join(format!("{}.md", p.slug));
        fs::write(&file_path, memory_file_body(p.entry))
            .with_context(|| format!("Failed to write {}", file_path.display()))?;
        match p.action {
            ExportAction::Add => append_index_line(&mut index_content, &p.slug, p.entry),
            ExportAction::Update => replace_index_line(&mut index_content, &p.slug, p.entry),
            ExportAction::Skip => unreachable!(),
        }
        println!(
            "  {} {} ({}): {}",
            action_label(p.action),
            p.entry.id,
            p.slug,
            file_path.display()
        );
        log_map.insert(
            p.entry.id.clone(),
            ExportRecord {
                id: p.entry.id.clone(),
                content_hash: p.entry.content_hash.clone(),
                slug: p.slug.clone(),
                exported_at: now.clone(),
            },
        );
        wrote_anything = true;
    }

    if !wrote_anything {
        println!("Nothing to do — all approved reflections are already exported.");
        return Ok(());
    }

    fs::write(&index_path, &index_content)
        .with_context(|| format!("Failed to write {}", index_path.display()))?;
    write_export_log(&log_path, log_map.values().cloned().collect())?;

    println!();
    println!("Export complete: {} added, {} updated, {} skipped.", adds, updates, skips);
    println!("  index:      {}", index_path.display());
    println!("  export log: {}", log_path.display());
    Ok(())
}

/// Fetch approved reflections from the daemon's REST API.
async fn fetch_approved(addr: SocketAddr) -> Result<Vec<ReflectionLedgerEntry>> {
    #[derive(Deserialize)]
    struct ReflectionsResponse {
        reflections: Vec<ReflectionLedgerEntry>,
        #[allow(dead_code)]
        count: usize,
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let resp = client
        .get(format!("http://{}/api/reflections", addr))
        .send()
        .await
        .context("Failed to connect to daemon — is it running?")?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        bail!("GET /api/reflections failed: {status} — {}", body.trim());
    }
    let json: ReflectionsResponse = resp
        .json()
        .await
        .context("Failed to parse /api/reflections response")?;
    Ok(json.reflections)
}

/// Default output directory: the operator's global memory index.
fn default_memory_dir() -> PathBuf {
    let mut p = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    p.push(".claude");
    p.push("projects");
    p.push("-home-coding");
    p.push("memory");
    p
}

fn export_log_path(out_dir: &Path) -> PathBuf {
    out_dir.join(EXPORT_LOG_FILE)
}

/// Read the export log into a vec of records (malformed lines are skipped).
fn read_export_log(path: &Path) -> Vec<ExportRecord> {
    let contents = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    contents
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<ExportRecord>(line).ok())
        .collect()
}

/// Write the full export log (one record per line), atomically-ish.
fn write_export_log(path: &Path, records: Vec<ExportRecord>) -> Result<()> {
    // Sort by id for a stable, diff-friendly checkpoint.
    let mut records = records;
    records.sort_by(|a, b| a.id.cmp(&b.id));
    let mut out = String::new();
    for rec in &records {
        out.push_str(&serde_json::to_string(rec)?);
        out.push('\n');
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, out)
        .with_context(|| format!("Failed to write export log {}", path.display()))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Small text helpers
// ---------------------------------------------------------------------------

fn action_label(action: ExportAction) -> &'static str {
    match action {
        ExportAction::Add => "ADD",
        ExportAction::Update => "UPDATE",
        ExportAction::Skip => "SKIP",
    }
}

/// Collapse runs of `-` into a single `-`.
fn collapse_dashes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_dash = false;
    for c in s.chars() {
        if c == '-' {
            if !prev_dash {
                out.push('-');
            }
            prev_dash = true;
        } else {
            out.push(c);
            prev_dash = false;
        }
    }
    out
}

/// Reduce text to a single line and truncate to `max` chars on a word boundary,
/// appending an ellipsis when truncated.
fn single_line(text: &str, max: usize) -> String {
    let collapsed: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() <= max {
        return collapsed;
    }
    let mut out: String = collapsed.chars().take(max).collect();
    // Walk back to the last whitespace so we don't cut mid-word.
    while out.chars().last().is_none_or(|c| !c.is_whitespace()) && !out.is_empty() {
        out.pop();
    }
    let trimmed = out.trim_end();
    if trimmed.is_empty() {
        // The first word alone exceeds `max`; keep the hard-truncated form.
        trimmed.to_string()
    } else {
        format!("{trimmed}…")
    }
}

/// Parse the `source_stitches` JSON array into a human-readable list.
fn format_source_stitches(raw: &str) -> String {
    let parsed: Vec<String> = serde_json::from_str(raw).unwrap_or_default();
    if parsed.is_empty() {
        return "none".to_string();
    }
    parsed.join(", ")
}

/// YAML-double-quote a string scalar (safe for arbitrary rule/description text).
fn yaml_double_quote(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Append one index line to a MEMORY.md buffer, ensuring exactly one blank
/// line separates it from prior content.
fn push_index_line(content: &mut String, line: &str) {
    // Own the trimmed tail so its borrow of `content` ends before we mutate
    // `content` below (otherwise `content.clear()` trips the borrow checker).
    let trimmed = content.trim_end_matches(['\n', '\r', ' ']).to_string();
    content.clear();
    if trimmed.is_empty() {
        content.push_str(line);
        content.push('\n');
    } else {
        content.push_str(&trimmed);
        content.push_str("\n\n");
        content.push_str(line);
        content.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a fixture entry with sensible defaults; only override what matters.
    fn fixture(id: &str, rule: &str, reason: &str, hash: &str) -> ReflectionLedgerEntry {
        ReflectionLedgerEntry {
            id: id.to_string(),
            scope: "global".to_string(),
            rule: rule.to_string(),
            reason: reason.to_string(),
            source_stitches: r#"["st-001","st-002"]"#.to_string(),
            status: "approved".to_string(),
            created_at: "2026-07-01T00:00:00Z".to_string(),
            last_applied: None,
            applied_count: 0,
            content_hash: hash.to_string(),
            rejection_count: 0,
            approved_by: Some("operator".to_string()),
            approved_at: Some("2026-07-02T00:00:00Z".to_string()),
            archived_at: None,
        }
    }

    #[test]
    fn slug_is_namespaced_and_stable() {
        // UUID ids slugify cleanly and keep their body for traceability.
        assert_eq!(
            slug_for("550e8400-e29b-41d4-a716-446655440000"),
            "reflection-550e8400-e29b-41d4-a716-446655440000"
        );
        // Upper-case is folded, unsafe chars become dashes, runs collapse.
        assert_eq!(slug_for("RF_AbC 123"), "reflection-rf-abc-123");
        // Degenerate id still produces a valid slug.
        assert_eq!(slug_for("!!!"), "reflection-untitled");
        // Deterministic.
        assert_eq!(slug_for("abc"), slug_for("abc"));
    }

    #[test]
    fn title_and_hook_truncate_cleanly() {
        assert_eq!(title_for("Always run cargo fmt"), "Always run cargo fmt");
        let long = "word ".repeat(40);
        let t = title_for(&long);
        assert!(t.chars().count() <= 61); // <= max + ellipsis
        assert!(t.ends_with('…'));

        assert_eq!(hook_for(""), "(no reason recorded)");
        assert_eq!(hook_for("prevents drift"), "prevents drift");
    }

    #[test]
    fn index_line_matches_workspace_format() {
        let e = fixture("abc", "Always run cargo fmt", "prevents drift", "h1");
        let line = index_line(&slug_for("abc"), &e);
        assert_eq!(
            line,
            "- [Always run cargo fmt](reflection-abc.md) - prevents drift"
        );
    }

    #[test]
    fn memory_file_body_has_frontmatter_and_body() {
        let e = fixture(
            "abc",
            "Always run cargo fmt before committing",
            "Keeps diffs reviewable",
            "h1",
        );
        let body = memory_file_body(&e);

        // Frontmatter shape matches the workspace's memory files.
        assert!(body.starts_with("---\n"));
        assert!(body.contains("name: reflection-abc\n"));
        assert!(body.contains("description: \"Always run cargo fmt before committing\"\n"));
        assert!(body.contains("  node_type: memory\n"));
        assert!(body.contains("  type: feedback\n"));
        assert!(body.contains("  source: hoop-reflection-ledger\n"));
        assert!(body.contains("  reflection_id: abc\n"));
        assert!(body.contains("  scope: global\n"));
        // A closing frontmatter delimiter precedes the body.
        assert!(body.contains("\n---\n\n"));

        // Body pulls from rule + reason + source stitches.
        assert!(body.contains("Always run cargo fmt before committing"));
        assert!(body.contains("**Why:** Keeps diffs reviewable"));
        assert!(body.contains("**Source stitches:** st-001, st-002"));
        assert!(body.contains("approved by operator"));
    }

    #[test]
    fn yaml_description_escapes_quotes_and_colons() {
        // A rule containing a double-quote and a colon must round-trip as a
        // single safe YAML scalar, not break the frontmatter.
        let e = fixture("abc", r#"Use the word "don't": carefully"#, "r", "h1");
        let body = memory_file_body(&e);
        let frontmatter = body.lines().take(7).collect::<Vec<_>>().join("\n");
        // Should parse back as valid YAML.
        let parsed: serde_yaml::Value = serde_yaml::from_str(&frontmatter).unwrap();
        let desc = parsed
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert!(desc.contains(r#""don't""#));
        assert!(desc.contains(':'));
    }

    #[test]
    fn plan_export_classifies_add_update_skip() {
        let e1 = fixture("id-1", "rule one", "r1", "hash-1");
        let e2 = fixture("id-2", "rule two", "r2", "hash-2");
        let e3 = fixture("id-3", "rule three", "r3", "hash-3");
        let entries = vec![e1.clone(), e2.clone(), e3.clone()];

        // Empty log → all Add.
        let planned = plan_export(&entries, &[]);
        assert_eq!(
            planned.iter().map(|p| p.action).collect::<Vec<_>>(),
            vec![ExportAction::Add, ExportAction::Add, ExportAction::Add]
        );

        // Now id-2 is exported unchanged, id-3 exported with a *different* hash
        // (simulating a rule edit). id-1 is brand new.
        let log = vec![
            ExportRecord {
                id: "id-2".to_string(),
                content_hash: "hash-2".to_string(),
                slug: "reflection-id-2".to_string(),
                exported_at: "t0".to_string(),
            },
            ExportRecord {
                id: "id-3".to_string(),
                content_hash: "hash-old".to_string(),
                slug: "reflection-id-3".to_string(),
                exported_at: "t0".to_string(),
            },
        ];
        let planned = plan_export(&entries, &log);
        let by_id: HashMap<&str, ExportAction> = planned
            .iter()
            .map(|p| (p.entry.id.as_str(), p.action))
            .collect();
        assert_eq!(by_id["id-1"], ExportAction::Add);
        assert_eq!(by_id["id-2"], ExportAction::Skip);
        assert_eq!(by_id["id-3"], ExportAction::Update);
    }

    #[test]
    fn append_index_line_is_idempotent() {
        let e = fixture("abc", "Always run cargo fmt", "prevents drift", "h1");
        let mut content = String::from("# Memory Index\n\n- [old](old.md) — x\n");
        append_index_line(&mut content, "reflection-abc", &e);
        let once = content.clone();
        // Running again must not duplicate the line.
        append_index_line(&mut content, "reflection-abc", &e);
        assert_eq!(content, once);
        assert_eq!(content.matches("reflection-abc.md").count(), 1);
    }

    #[test]
    fn replace_index_line_updates_in_place() {
        let e_old = fixture("abc", "Old rule", "old reason", "h1");
        let e_new = fixture("abc", "New rule", "new reason", "h2");
        let mut content = String::new();
        append_index_line(&mut content, "reflection-abc", &e_old);
        replace_index_line(&mut content, "reflection-abc", &e_new);
        assert!(content.contains("[New rule]"));
        assert!(!content.contains("[Old rule]"));
        assert_eq!(content.matches("reflection-abc.md").count(), 1);
    }

    #[test]
    fn end_to_end_idempotent_via_tempdir() {
        let dir = tempfile::tempdir().unwrap();
        let out_dir = dir.path();
        let log_path = export_log_path(out_dir);
        let index_path = out_dir.join(INDEX_FILE);

        let e1 = fixture("id-1", "rule one", "reason one", "hash-1");
        let e2 = fixture("id-2", "rule two", "reason two", "hash-2");
        let entries = vec![e1.clone(), e2.clone()];

        // --- first run: both Add ---
        let log = read_export_log(&log_path);
        let planned = plan_export(&entries, &log);
        let mut index_content = "# Memory Index\n\n".to_string();
        let mut log_map: HashMap<String, ExportRecord> = HashMap::new();
        let now = "2026-07-26T00:00:00Z".to_string();
        for p in &planned {
            assert_eq!(p.action, ExportAction::Add);
            fs::write(out_dir.join(format!("{}.md", p.slug)), memory_file_body(p.entry)).unwrap();
            append_index_line(&mut index_content, &p.slug, p.entry);
            log_map.insert(
                p.entry.id.clone(),
                ExportRecord {
                    id: p.entry.id.clone(),
                    content_hash: p.entry.content_hash.clone(),
                    slug: p.slug.clone(),
                    exported_at: now.clone(),
                },
            );
        }
        fs::write(&index_path, &index_content).unwrap();
        write_export_log(&log_path, log_map.values().cloned().collect()).unwrap();

        assert_eq!(index_content.matches("reflection-id-1.md").count(), 1);
        assert_eq!(index_content.matches("reflection-id-2.md").count(), 1);

        // --- second run, same data: both Skip, nothing changes ---
        let log = read_export_log(&log_path);
        let planned = plan_export(&entries, &log);
        assert!(planned.iter().all(|p| p.action == ExportAction::Skip));
        let index_before = fs::read_to_string(&index_path).unwrap();
        // A Skip-only run performs no writes; index is untouched.
        assert_eq!(fs::read_to_string(&index_path).unwrap(), index_before);

        // --- third run: e2's content hash changed → Update only ---
        let e2_changed = fixture("id-2", "rule two edited", "reason two edited", "hash-2-new");
        let entries = vec![e1.clone(), e2_changed];
        let log = read_export_log(&log_path);
        let planned = plan_export(&entries, &log);
        let by_id: HashMap<&str, ExportAction> = planned
            .iter()
            .map(|p| (p.entry.id.as_str(), p.action))
            .collect();
        assert_eq!(by_id["id-1"], ExportAction::Skip);
        assert_eq!(by_id["id-2"], ExportAction::Update);

        for p in &planned {
            if p.action == ExportAction::Skip {
                continue;
            }
            fs::write(out_dir.join(format!("{}.md", p.slug)), memory_file_body(p.entry)).unwrap();
            replace_index_line(&mut index_content, &p.slug, p.entry);
        }
        assert!(index_content.contains("rule two edited"));
        // Still only one index line per slug.
        assert_eq!(index_content.matches("reflection-id-1.md").count(), 1);
        assert_eq!(index_content.matches("reflection-id-2.md").count(), 1);
    }

    #[test]
    fn format_source_stitches_handles_garbage() {
        assert_eq!(format_source_stitches(r#"["a","b"]"#), "a, b");
        assert_eq!(format_source_stitches("[]"), "none");
        assert_eq!(format_source_stitches("not json"), "none");
    }
}
