# HOOP Screenshots

This directory will contain screenshots of the HOOP UI for the README and documentation.

## Current Status

Screenshots are planned for v1.0.1. The v1.0.0 release focuses on core functionality; visual documentation will follow in a patch release.

## Planned Screenshots

1. **Dashboard** — Project cards showing active work, cost today, and alerts
2. **Project View** — Stitch list with worker sessions, operator chats, and dictated notes
3. **Stitch Detail** — Message timeline, linked beads, touched files
4. **Pattern Library** — Cross-project goal organization and progress
5. **Morning Brief** — Daily summary with drafts and headlines
6. **File Browser** — Code viewer with Stitch-provenance annotations
7. **Fleet Map** — Live worker layout visualization
8. **Mobile View** — Responsive UI on phone form factor
9. **ADB Dictation** — Push-to-talk voice note capture
10. **Settings Panel** — Configuration UI with hot-reload indicators

## UI Overview

The HOOP UI consists of:

- **Dashboard** (`/`) — Cross-project overview with cards for each registered project
- **Project Detail** (`/project/:id`) — Single-project view with Stitch timeline, files, drafts, morning brief
- **Agent Chat** — Persistent conversation pane for asking questions and drafting work
- **Fleet Map** — Visualization of active NEEDLE workers across projects
- **Settings** — Configuration UI for themes, backups, and agent preferences

The UI is responsive and works on desktop, tablet, and mobile form factors. Default theme is dark mode.

## Guidelines

When capturing screenshots:

- **Anonymize all sensitive data** (project names, file paths, bead IDs, API keys)
- **Use dark theme** for consistency (default)
- **Capture at 1920x1080 resolution** for desktop, 390x844 for mobile
- **Include realistic but synthetic data** — don't use "foo/bar/baz"
- **Show both empty and populated states** where relevant
- **Highlight key UI elements** with subtle annotations when needed
- **Optimize for web** — PNG at 80% quality, keep files under 500KB

## Demo Video

A demo video walkthrough is planned for v1.1 covering:
- Installation and first-time setup
- Project registration and scanning
- Creating a Stitch via the agent
- Morning brief review
- Pattern creation and management
- Cost tracking and anomaly detection
- Push-to-talk dictation workflow

## File Naming Convention

```
<screenshot-name>-<theme>.png

Examples:
- dashboard-dark.png
- project-view-populated-dark.png
- morning-brief-empty-dark.png
- fleet-map-mobile-dark.png
```
