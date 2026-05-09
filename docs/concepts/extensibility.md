# Extensibility — Skills, Scripts, Notes, Prompts

HOOP is extensible without code changes via four directory-based plugin types. All four directories are hot-reloaded—changes take effect within seconds without restart.

## Overview

| Type | Purpose | Directory |
|------|---------|-----------|
| **Skills** | Custom tools the human-interface agent can call | `~/.hoop/skills/<name>/` |
| **Scripts** | Operator-triggered or event-triggered automation | `~/.hoop/scripts/<name>` |
| **Notes** | Structured knowledge files the agent can read | `~/.hoop/notes/<name>.md` |
| **Prompts** | Reusable prompt library referenced by name | `~/.hoop/prompts/<name>.md` |

## Skills

A Skill is a directory with:
- `manifest.yml` — name, description, argument schema (JSON Schema)
- `run` — executable (any language) that reads args from stdin as JSON, writes result to stdout as JSON
- Optional `README.md` — human documentation

### Example Skill: Echo

`~/.hoop/skills/echo/manifest.yml`:
```yaml
name: echo
description: Echo back the input message
summary: Simple echo skill for testing
scope: global
args_schema:
  type: object
  properties:
    message:
      type: string
  required: ["message"]
timeout_secs: 30
```

`~/.hoop/skills/echo/run`:
```bash
#!/bin/bash
INPUT=$(cat)
if command -v jq >/dev/null 2>&1; then
    MESSAGE=$(echo "$INPUT" | jq -r '.message')
    echo "{\"output\": \"$MESSAGE\"}"
else
    echo "{\"output\": \"$INPUT\"}"
fi
exit 0
```

### Example Skill: lookup-git-log

A practical skill for querying git commit history:

`~/.hoop/skills/lookup-git-log/manifest.yml`:
```yaml
name: lookup-git-log
description: Query git log history with filtering options
summary: Look up git commit history with optional filters
scope: global
args_schema:
  type: object
  properties:
    project_path:
      type: string
      description: Path to the git repository
    max_count:
      type: integer
      description: Maximum commits to return (1-100)
      minimum: 1
      maximum: 100
    author:
      type: string
      description: Filter by author name or email
    since:
      type: string
      description: Show commits since date (e.g. "2 weeks ago")
    path:
      type: string
      description: Filter commits affecting a file/directory
  required: []
timeout_secs: 60
```

Usage:
```bash
curl -X POST http://localhost:3000/api/skills/lookup-git-log/run \
  -H "Content-Type: application/json" \
  -d '{
    "args": {
      "project_path": "/home/coding/HOOP",
      "max_count": 10,
      "author": "jedarden"
    }
  }'
```

### Skill Manifest Schema

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Must match directory name |
| `description` | string | Yes | Agent-facing one-line summary |
| `summary` | string | Yes | Human-readable purpose |
| `scope` | string | No | `global` (default), `project`, or `pattern` |
| `projects` | array | No | Project names for `project` scope |
| `pattern` | string | No | Pattern for `pattern` scope (e.g., `fix-*`) |
| `args_schema` | object | Yes | JSON Schema for argument validation |
| `timeout_secs` | number | No | Execution timeout (default: 300) |

### Using Skills

Skills are automatically discovered and exposed as tools to the human-interface agent. The agent can invoke them based on the task at hand. Skills can also be executed via the REST API:

```bash
# List all skills
curl http://localhost:3000/api/skills

# Execute a skill
curl -X POST http://localhost:3000/api/skills/echo/run \
  -H "Content-Type: application/json" \
  -d '{"args": {"message": "Hello, HOOP!"}}'
```

## Scripts

A Script is a single executable file that can be triggered:
- **Operator-invoked**: Via UI button or `hoop script run <name>`
- **Event-triggered**: Via manifest.yml subscriptions (e.g., "when a Stitch is archived")
- **Scheduled**: Via cron-style schedule in manifest.yml

Scripts run with full HOOP user privileges and are the escape hatch for external integrations (webhooks, notifications, browser automation).

### Example Script: Notify on Failure

`~/.hoop/scripts/notify-on-failure`:
```bash
#!/bin/bash
# Read event JSON from stdin
EVENT=$(cat)
# Extract relevant fields
BEAD_ID=$(echo "$EVENT" | jq -r '.bead_id // empty')
ERROR=$(echo "$EVENT" | jq -r '.error // "Unknown error"')
# Send pushover notification
curl -s \
  --form-string "token=$PUSHOVER_TOKEN" \
  --form-string "user=$PUSHOVER_USER" \
  --form-string "message=HOOP: Bead $BEAD_ID failed: $ERROR" \
  https://api.pushover.net/1/messages.json
```

`~/.hoop/scripts/notify-on-failure.yml`:
```yaml
name: notify-on-failure
description: Send push notification when a bead fails
scope: global
timeout_secs: 60
on:
  - event: "fail"
    result: "failure"
```

### Script Manifest Schema

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Must match script filename |
| `description` | string | No | Human-readable summary |
| `scope` | string | No | `global` (default) or `project` |
| `projects` | array | No | Project names for `project` scope |
| `timeout_secs` | number | No | Execution timeout (default: 300) |
| `arguments` | array | No | Argument schema for UI prompts |
| `schedule` | string | No | Cron schedule (5-field format) |
| `overlap_policy` | string | No | `skip` (default), `queue`, `parallel` |
| `on` | array | No | Event subscriptions |

### Event Subscriptions

Scripts can subscribe to NEEDLE events via the `on` field:

```yaml
on:
  - event: "bead.*"          # Glob pattern for event type
    project: "my-project"    # Optional project filter
    kind: "fix"              # Optional bead kind filter
    adapter: "claude"        # Optional adapter filter
    result: "failure"        # Optional result filter
```

### Using Scripts

```bash
# List all scripts
curl http://localhost:3000/api/scripts

# Execute a script manually
curl -X POST http://localhost:3000/api/scripts/backup-db/run \
  -H "Content-Type: application/json" \
  -d '{"args": ["--full"]}'
```

## Notes

Notes are plain markdown files the agent can read via its `read_note(name)` tool. Use cases:
- Project glossaries
- Team conventions ("we always prefer A over B")
- Reference material for the agent

### Example Note: Team Conventions

`~/.hoop/notes/team-conventions.md`:
```markdown
---
title: Team Conventions
description: How we work together
tags: [conventions, workflow]
---

# Team Conventions

## Code Review
- All PRs require at least one approval
- Use "Request Changes" for blocking issues
- Comments should be constructive and specific

## Testing
- Write tests for new features
- Run tests before committing
- Test coverage should not decrease
```

### Project-Scoped Notes

Notes can also be project-scoped at `<workspace>/.hoop/notes/<name>.md`:

```bash
# Global note
~/.hoop/notes/glossary.md

# Project-scoped note
~/my-project/.hoop/notes/project-glossary.md
```

### Using Notes

```bash
# List all notes
curl http://localhost:3000/api/notes

# List global notes only
curl http://localhost:3000/api/notes/global

# List project-scoped notes
curl http://localhost:3000/api/notes/project/my-project

# Get a specific note
curl http://localhost:3000/api/notes/team-conventions
```

## Prompts

Prompts are reusable prompt bodies referenced by name (`@prompt:<name>`) with parameter substitution.

### Example Prompt: Fix Linting

`~/.hoop/prompts/fix-linting.md`:
```markdown
---
name: fix-linting
description: "Fix a linting violation in a file"
args:
  - lint_type
  - severity
---

## Task
Fix {{lint_type}} linting error ({{severity}}) in {{file}}.

## Acceptance
- Linter passes for {{file}}
- No new violations introduced
```

### Built-in Variables

| Variable | Description |
|----------|-------------|
| `{{project}}` | Project name |
| `{{file}}` | File path |
| `{{stitch}}` | Stitch ID |
| `{{now}}` | Current timestamp (ISO 8601) |

Plus any custom arguments passed via `args`.

### Using Prompts

```bash
# List all prompts
curl http://localhost:3000/api/prompts

# Get a specific prompt
curl http://localhost:3000/api/prompts/fix-linting

# Substitute variables in a prompt
curl -X POST http://localhost:3000/api/prompts/fix-linting/substitute \
  -H "Content-Type: application/json" \
  -d '{
    "file": "src/main.rs",
    "args": {
      "lint_type": "clippy::unwrap_used",
      "severity": "warning"
    }
  }'
```

## Configuration

Extension directories are configurable via `~/.hoop/config.yml`:

```yaml
agent_extensions:
  skills: null      # Default: ~/.hoop/skills
  scripts: null     # Default: ~/.hoop/scripts
  notes: null       # Default: ~/.hoop/notes
  prompts: null     # Default: ~/.hoop/prompts
```

Or via environment variables:
- `HOOP_AGENT_EXTENSIONS_SKILLS`
- `HOOP_AGENT_EXTENSIONS_SCRIPTS`
- `HOOP_AGENT_EXTENSIONS_NOTES`
- `HOOP_AGENT_EXTENSIONS_PROMPTS`

## Hot-Reload

All four extension directories are file-watched. Changes take effect within seconds:

- New skill → agent's next turn has the new tool
- Script schedule change → picked up on next scheduler tick (60s)
- Note update → agent immediately sees updated content
- Prompt update → immediately available for use

## Security

Skills and scripts run with HOOP user's privileges. There is no sandboxing—the operator owns their extensions.

**Audit logging:**
- Every skill invocation is logged with arguments (hashed)
- Script executions are logged but not stdout/stderr (script's responsibility)
- Actor identity is captured via Tailscale whois or OS username

**Best practices:**
- Validate all input in scripts/skills
- Don't log sensitive data
- Use appropriate file permissions
- Review scripts/skills before adding to shared environments

## Sharing

Extension directories are plain files. Share via:
- `git clone` into the directory
- Tarballs
- Community registry (not built by HOOP)

Example:
```bash
# Share a skill collection
git clone https://github.com/example/hoop-skills.git ~/.hoop/skills/
```

## Quickstart

HOOP seeds example extensions on first run:

- **Skill**: `echo` — Simple echo skill for testing
- **Skill**: `lookup-git-log` — Query git commit history with filtering
- **Note**: `team-conventions` — Example team conventions
- **Note**: `glossary` — HOOP terminology glossary
- **Prompt**: `fix-linting` — Fix linting violations
- **Prompt**: `write-plan-stub` — Create plan.md stub
- **Prompt**: `investigate-error` — Error investigation template

Test the echo skill:
```bash
curl -X POST http://localhost:3000/api/skills/echo/run \
  -H "Content-Type: application/json" \
  -d '{"args": {"message": "Hello from HOOP!"}}'
```

Test the git log lookup skill:
```bash
curl -X POST http://localhost:3000/api/skills/lookup-git-log/run \
  -H "Content-Type: application/json" \
  -d '{"args": {"project_path": "/home/coding/HOOP", "max_count": 5}}'
```

## API Reference

### Skills API
- `GET /api/skills` — List all skills
- `GET /api/skills/{name}` — Get skill manifest
- `POST /api/skills/{name}/run` — Execute a skill

### Scripts API
- `GET /api/scripts` — List all scripts
- `GET /api/scripts/{name}` — Get script manifest
- `POST /api/scripts/{name}/run` — Execute a script

### Notes API
- `GET /api/notes` — List all notes
- `GET /api/notes/global` — List global notes
- `GET /api/notes/project/{project}` — List project notes
- `GET /api/notes/{name}` — Get a note

### Prompts API
- `GET /api/prompts` — List all prompts
- `GET /api/prompts/{name}` — Get a prompt
- `POST /api/prompts/{name}/substitute` — Substitute variables
