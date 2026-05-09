# HOOP Extensibility Guide

HOOP is extensible without code changes via four directory-based plugin types. This guide explains how to create and use each type of extension.

## Overview

| Type | Purpose | Directory |
|---|---|---|
| **Skills** | Custom tools the human-interface agent can call | `~/.hoop/skills/<name>/` |
| **Scripts** | Operator-triggered or event-triggered automation | `~/.hoop/scripts/<name>` |
| **Notes** | Structured knowledge files the agent can read | `~/.hoop/notes/<name>.md` |
| **Prompts** | Reusable prompt library referenced by name | `~/.hoop/prompts/<name>.md` |

All four directories are **hot-reloaded** — changes take effect within seconds without restarting HOOP.

---

## Skills

A Skill is a directory with a `manifest.yml` and an executable `run` file. Skills are automatically discovered and exposed as tools to the human-interface agent via MCP.

### Directory Structure

```
~/.hoop/skills/<skill-name>/
├── manifest.yml    # Required: skill metadata
├── run             # Required: executable (any language)
└── README.md       # Optional: human documentation
```

### Manifest Schema

```yaml
name: skill-name              # Required: must match directory name
description: One-line summary # Required: agent-facing description
summary: Human-readable purpose # Required: longer description
scope: global                 # Required: global|project|pattern
timeout_secs: 300            # Optional: execution timeout (default: 300)
args_schema:                  # Required: JSON Schema for arguments
  type: object
  properties:
    url:
      type: string
      description: URL to fetch
  required:
    - url
```

### Example Skill

**~/.hoop/skills/fetch-url/manifest.yml**
```yaml
name: fetch-url
description: Fetch a URL and return the response body
summary: Retrieves content from HTTP/HTTPS URLs
scope: global
timeout_secs: 60
args_schema:
  type: object
  properties:
    url:
      type: string
      description: The URL to fetch
  required:
    - url
```

**~/.hoop/skills/fetch-url/run**
```bash
#!/bin/bash
# Read JSON arguments from stdin
INPUT=$(cat)
URL=$(echo "$INPUT" | jq -r '.url')

# Fetch the URL
if curl -sSf "$URL" > /tmp/response.$$; then
    echo "{\"status\": \"success\", \"body\": \"$(cat /tmp/response.$$ | jq -Rs .)\"}"
    rm -f /tmp/response.$$
    exit 0
else
    echo "{\"status\": \"error\", \"message\": \"Failed to fetch $URL\"}"
    exit 1
fi
```

### Skill Execution

1. Agent invokes skill via MCP tool `skill_<name>`
2. Daemon validates arguments against `args_schema`
3. If valid: runs `~/.hoop/skills/<name>/run` with args JSON on stdin
4. If invalid: returns validation error without executing

### Scopes

- **global**: Available to all agents (default)
- **project**: Only available when working in specified projects
- **pattern**: Available when bead title/pattern matches

---

## Scripts

A Script is a single executable file that can be triggered manually, by events, or on a schedule.

### Script Structure

```
~/.hoop/scripts/<script-name>       # Executable file (any language)
~/.hoop/scripts/<script-name>.yml   # Optional manifest
```

### Manifest Schema

```yaml
name: script-name          # Required: must match filename
description: Summary       # Optional: human-readable description
scope: global              # Optional: global|project (default: global)
timeout_secs: 300         # Optional: execution timeout (default: 300)
schedule: "0 4 * * *"     # Optional: cron schedule (5-field format)
overlap_policy: skip      # Optional: skip|queue|parallel (default: skip)
arguments:                # Optional: argument schema for UI
  - name: input
    description: Input value
    required: true
on:                        # Optional: event subscriptions
  - event: "bead.closed"
    project: "my-project"
    result: "failure"
```

### Example Scripts

**~/.hoop/scripts/backup-database**
```bash
#!/bin/bash
# Backup the project database
pg_dump "$DATABASE_URL" > "/backups/db-$(date +%Y%m%d).sql"
echo "Backup completed"
```

**~/.hoop/scripts/backup-database.yml**
```yaml
name: backup-database
description: Backup project database to daily file
scope: global
timeout_secs: 600
schedule: "0 2 * * *"  # Run daily at 2 AM
overlap_policy: skip
```

### Triggers

Scripts can be triggered in three ways:

1. **Manual**: Via UI button or `POST /api/scripts/<name>/run`
2. **Scheduled**: Via cron expression in manifest
3. **Event**: Via `on:` subscriptions in manifest

### Event Subscriptions

Subscribe to NEEDLE events with glob patterns:

```yaml
on:
  - event: "bead.*"              # Glob pattern on event type
    project: "my-project"        # Glob pattern on project
    kind: "bug"                   # Exact match on kind
    adapter: "claude"             # Exact match on adapter
    result: "failure"             # "success" or "failure"
```

When a matching event fires, the script is executed with the event JSON on stdin.

---

## Notes

Notes are plain markdown files that the agent can read via the `read_note` tool.

### Note Format

```markdown
---
title: Note Title
description: Optional description
tags: [optional, tags]
---

# Markdown Content

Your note content here.
```

### Locations

- **Global notes**: `~/.hoop/notes/<name>.md`
- **Project notes**: `<workspace>/.hoop/notes/<name>.md`

### Example Note

**~/.hoop/notes/api-conventions.md**
```markdown
---
title: API Conventions
description: How we design REST APIs
tags: [api, conventions]
---

# API Conventions

## URL Structure
- Use kebab-case for resource names: `/api/users/:id`
- Pluralize resource names: `/api/users`, not `/api/user`

## Response Format
All responses follow this structure:
```json
{
  "data": { ... },
  "error": null,
  "meta": { "page": 1 }
}
```
```

### Agent Access

The agent can read notes using the `read_note` tool:

```
Agent: What are our API conventions?
Tool Call: read_note(name="api-conventions")
```

---

## Prompts

Prompts are reusable prompt bodies with parameter substitution.

### Prompt Format

```markdown
---
name: prompt-name
description: What this prompt does
args:
  - var_name
---

## Task
Do something with {{var_name}}.

## Requirements
- Must handle {{var_name}} correctly
- Should not exceed limits
```

### Built-in Variables

- `{{project}}` - Current project name
- `{{file}}` - Current file path
- `{{stitch}}` - Current stitch ID
- `{{now}}` - Current timestamp

### Example Prompt

**~/.hoop/prompts/investigate-error.md**
```markdown
---
name: investigate-error
description: Investigate an error in a codebase
args:
  - error_message
  - file_path
---

## Task
Investigate error in {{file_path}}: {{error_message}}

## Steps
1. Read the file to understand context
2. Identify the root cause
3. Propose a fix
4. Check for similar issues elsewhere

## Acceptance Criteria
- Root cause identified
- Fix proposed
- No similar issues remain
```

### Using Prompts

Reference prompts by name with `@prompt:<name>`:

```
Agent: Use the investigate-error prompt for the parse error in parser.rs
Tool Call: (substitutes {{error_message}} and {{file_path}})
```

Or via API:

```bash
curl -X POST http://localhost:8080/api/prompts/investigate-error/substitute \
  -H "Content-Type: application/json" \
  -d '{
    "file": "src/parser.rs",
    "args": {
      "error_message": "unexpected token at line 42",
      "file_path": "src/parser.rs"
    }
  }'
```

---

## Hot-Reload

All four plugin directories are file-watched. Changes take effect within seconds:

- **Skills**: New tools appear in agent's next turn
- **Scripts**: New scripts appear in UI immediately
- **Notes**: Agent can read updated notes immediately
- **Prompts**: Updated templates available immediately

No restart required.

---

## Security

- Skills and scripts run with HOOP user's privileges
- No sandboxing — operator owns their extensions
- Audit log records every skill/script invocation with arguments
- Skills validate arguments against JSON Schema before execution

---

## Sharing

Extensions are plain files. Share via:

- **Git**: `git clone` into the target directory
- **Tarballs**: Extract to `~/.hoop/skills/` or `~/.hoop/scripts/`
- **Community registry**: (future) not built by HOOP

---

## REST API

### Skills

- `GET /api/skills` — List all skills
- `GET /api/skills/:name` — Get skill manifest
- `POST /api/skills/:name/run` — Execute a skill

### Scripts

- `GET /api/scripts` — List all scripts
- `GET /api/scripts/:name` — Get script manifest
- `POST /api/scripts/:name/run` — Execute a script

### Notes

- `GET /api/notes` — List all notes
- `GET /api/notes/global` — List global notes only
- `GET /api/notes/project/:project` — List project-scoped notes
- `GET /api/notes/:name` — Get a single note

### Prompts

- `GET /api/prompts` — List all prompts
- `GET /api/prompts/:name` — Get a single prompt
- `POST /api/prompts/:name/substitute` — Substitute variables in a prompt

---

## Quickstart Examples

HOOP seeds example extensions on first run:

### Skills
- `echo` — Simple echo skill for testing
- `lookup-git-log` — Query git commit history with filters

### Scripts
- `hello-world` — Print a friendly message (seeded if scripts dir is empty)

### Notes
- `team-conventions.md` — Team workflow guidelines
- `glossary.md` — Common terms and acronyms

### Prompts
- `fix-linting.md` — Fix linting violations
- `write-plan-stub.md` — Create a plan.md stub
- `investigate-error.md` — Investigate errors in code

---

## Tips

1. **Start simple**: Begin with a basic skill or script, then add features
2. **Use jq**: For JSON parsing in bash scripts, `jq` is your friend
3. **Set timeouts**: Use `timeout_secs` to prevent runaway scripts
4. **Validate inputs**: Use JSON Schema in skill manifests to catch errors early
5. **Log output**: Script stdout/stderr is captured and returned to the caller
6. **Test locally**: Run skills/scripts directly before installing to HOOP
7. **Use overlap policies**: For scheduled scripts, choose `skip` (default), `queue`, or `parallel`
8. **Leverage events**: Use event subscriptions to trigger scripts on important system events

---

## Troubleshooting

### Skill not appearing to agent
- Check that `run` file is executable: `chmod +x ~/.hoop/skills/<name>/run`
- Verify manifest `name` matches directory name
- Check daemon logs for parse errors

### Script timing out
- Increase `timeout_secs` in manifest
- Check for infinite loops or blocking operations
- Verify the script doesn't require interactive input

### Notes not found
- Verify file ends in `.md`
- Check YAML frontmatter is valid
- Ensure global notes are in `~/.hoop/notes/`

### Prompt substitution failing
- Verify variable names match between args and body
- Check for typos in `{{variable}}` syntax
- Ensure required args are provided
