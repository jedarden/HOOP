//! `hoop new <project>` — open a Stitch draft in $EDITOR, validate, and submit.
//!
//! The draft is a markdown file with YAML frontmatter. On save the frontmatter
//! is validated and the draft is POSTed to the running daemon with source="cli".

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::io::Write as _;
use std::process::Command;

// ---------------------------------------------------------------------------
// Template
// ---------------------------------------------------------------------------

const TEMPLATE: &str = "\
---\n\
project: {project}\n\
title: \"\"\n\
kind: feature\n\
description: |\n\
  Describe the work here.\n\
labels: []\n\
priority: null\n\
has_acceptance_criteria: false\n\
---\n\
\n\
<!-- Edit the YAML frontmatter above, then save and quit the editor. -->\n\
<!-- Everything after the second --- is ignored.                       -->\n\
<!-- Valid kinds: investigation, fix, feature                          -->\n\
";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct Frontmatter {
    project: String,
    title: String,
    kind: String,
    description: Option<String>,
    labels: Option<Vec<String>>,
    priority: Option<i64>,
    has_acceptance_criteria: Option<bool>,
}

#[derive(Debug, Serialize)]
struct CreateDraftPayload {
    project: String,
    title: String,
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    has_acceptance_criteria: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    labels: Option<Vec<String>>,
    source: String,
    force_create: bool,
}

#[derive(Debug, Deserialize)]
struct CreateDraftResponse {
    draft_id: String,
    status: String,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub async fn run(project: &str, dry_run: bool) -> Result<()> {
    // 1. Validate project exists in registry
    let registry = crate::projects::ProjectsRegistry::load()?;
    if registry.get(project).is_none() {
        bail!(
            "Project '{}' not in registry.\nRun `hoop projects list` to see registered projects.",
            project
        );
    }

    // 2. Write the template to a named temp file
    let template = TEMPLATE.replace("{project}", project);
    let mut tmp = tempfile::Builder::new()
        .prefix("hoop-draft-")
        .suffix(".md")
        .tempfile()
        .context("Failed to create temp file")?;
    tmp.write_all(template.as_bytes())
        .context("Failed to write draft template")?;
    tmp.flush()?;
    let tmp_path = tmp.path().to_path_buf();

    // 3. Launch $EDITOR (fall back to $VISUAL, then vi)
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string());

    let exit_status = Command::new(&editor)
        .arg(&tmp_path)
        .status()
        .with_context(|| format!("Failed to launch editor '{}'", editor))?;

    if !exit_status.success() {
        bail!(
            "Editor '{}' exited with a non-zero status; draft not submitted",
            editor
        );
    }

    // 4. Read saved file
    let contents = std::fs::read_to_string(&tmp_path)
        .context("Failed to read draft file after editing")?;

    // 5. Parse and validate YAML frontmatter
    let fm = parse_frontmatter(&contents)?;

    if fm.title.trim().is_empty() {
        bail!("title is required — set it in the YAML frontmatter");
    }

    let payload = CreateDraftPayload {
        project: fm.project,
        title: fm.title.trim().to_string(),
        kind: fm.kind,
        description: fm
            .description
            .map(|d| d.trim().to_string())
            .filter(|d| !d.is_empty()),
        has_acceptance_criteria: fm.has_acceptance_criteria,
        priority: fm.priority,
        labels: fm.labels,
        source: "cli".to_string(),
        force_create: false,
    };

    // 6. Dry-run: print what would be submitted and exit
    if dry_run {
        println!("--dry-run: would POST to http://127.0.0.1:3000/api/drafts");
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    // 7. Submit to the daemon REST API
    let body = serde_json::to_string(&payload)?;
    let client = reqwest::Client::new();
    let resp = client
        .post("http://127.0.0.1:3000/api/drafts")
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .context("Failed to reach daemon — is `hoop serve` running?")?;

    let http_status = resp.status();
    if !http_status.is_success() {
        let err_body = resp.text().await.unwrap_or_default();
        bail!("Daemon returned {}: {}", http_status, err_body);
    }

    let resp_text = resp.text().await.context("Failed to read daemon response")?;
    let created: CreateDraftResponse =
        serde_json::from_str(&resp_text).context("Failed to parse daemon response")?;

    println!("Draft created: {}", created.draft_id);
    println!("Status:        {}", created.status);
    println!("Approve in the UI at http://localhost:3000");

    Ok(())
}

// ---------------------------------------------------------------------------
// Frontmatter parser
// ---------------------------------------------------------------------------

fn parse_frontmatter(contents: &str) -> Result<Frontmatter> {
    let body = contents
        .strip_prefix("---\n")
        .or_else(|| contents.strip_prefix("---\r\n"))
        .context("Draft must start with --- (YAML frontmatter)")?;

    let end = body
        .find("\n---")
        .context("YAML frontmatter closing --- not found")?;

    let yaml = &body[..end];
    let fm: Frontmatter = serde_yaml::from_str(yaml).context("Invalid YAML frontmatter")?;
    Ok(fm)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_frontmatter() {
        let contents = "---\nproject: HOOP\ntitle: Add caching layer\nkind: feature\ndescription: |\n  Speed up reads.\nlabels: [perf]\npriority: 2\nhas_acceptance_criteria: true\n---\n\nsome body text\n";
        let fm = parse_frontmatter(contents).unwrap();
        assert_eq!(fm.project, "HOOP");
        assert_eq!(fm.title, "Add caching layer");
        assert_eq!(fm.kind, "feature");
        assert_eq!(fm.has_acceptance_criteria, Some(true));
        assert_eq!(fm.priority, Some(2));
        assert_eq!(fm.labels, Some(vec!["perf".to_string()]));
    }

    #[test]
    fn parse_minimal_frontmatter() {
        let contents = "\
---\n\
project: PROJ\n\
title: Fix the bug\n\
kind: fix\n\
description: null\n\
labels: []\n\
priority: null\n\
has_acceptance_criteria: false\n\
---\n\
";
        let fm = parse_frontmatter(contents).unwrap();
        assert_eq!(fm.title, "Fix the bug");
        assert_eq!(fm.kind, "fix");
    }

    #[test]
    fn parse_missing_opening_fence_errors() {
        let result = parse_frontmatter("title: oops\n---\n");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("---"));
    }

    #[test]
    fn parse_missing_closing_fence_errors() {
        let result = parse_frontmatter("---\ntitle: oops\n");
        assert!(result.is_err());
    }

    #[test]
    fn template_contains_project() {
        let out = TEMPLATE.replace("{project}", "MY-PROJECT");
        assert!(out.contains("project: MY-PROJECT"));
        assert!(out.contains("kind: feature"));
    }
}
