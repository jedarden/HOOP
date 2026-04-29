# HOOP Screenshots

This directory contains screenshots of the HOOP UI for the README and documentation.

## Current Screenshots (v1.0.0)

The following screenshots are available and used in the main README.md:

1. **Dashboard** (`dashboard.png`) — Project cards showing active work, cost today, and alerts
2. **Project Detail** (`project-detail.png`) — Stitch list with worker sessions, operator chats, and dictated notes
3. **Agent Chat** (`agent-chat.png`) — Persistent conversation pane for asking questions and drafting work
4. **File Browser** (`file-browser.png`) — Code viewer with syntax highlighting and navigation

## UI Overview

The HOOP UI consists of:

- **Dashboard** (`/`) — Cross-project overview with cards for each registered project
- **Project Detail** (`/project/:id`) — Single-project view with Stitch timeline, files, drafts, morning brief
- **Agent Chat** — Persistent conversation pane for asking questions and drafting work
- **File Browser** — Navigate project source code with syntax highlighting

The UI is responsive and works on desktop, tablet, and mobile form factors. Default theme is dark mode.

## Generating New Screenshots

To update or regenerate screenshots:

```bash
# From the HOOP repository root
cd hoop-ui/web
pnpm tsx scripts/take-screenshots.ts
```

This will:
1. Start the HOOP daemon with testrepo data
2. Capture screenshots of each major view
3. Save them to this directory

## Guidelines

When capturing screenshots:

- **Anonymize all sensitive data** (project names, file paths, bead IDs, API keys)
- **Use dark theme** for consistency (default)
- **Capture at 1920x1080 resolution** for desktop, 390x844 for mobile
- **Show realistic data** — use testrepo synthetic data rather than "foo/bar/baz"
- **Optimize for web** — PNG at 80% quality, keep files under 500KB

## File Naming Convention

```
<screenshot-name>-<theme>.png

Examples:
- dashboard-dark.png
- project-detail-populated-dark.png
- file-browser-source-dark.png
```

## Demo Video (Planned for v1.1)

A demo video walkthrough is planned for v1.1 covering:
- Installation and first-time setup
- Project registration and scanning
- Creating a Stitch via the agent
- Morning brief review
- Pattern creation and management
- Cost tracking and anomaly detection
- Push-to-talk dictation workflow
