# Extensibility Implementation Verification (hoop-ttb.19)

## Summary

All four extensibility systems specified in plan section §22 are fully implemented and integrated.

## Implementation Status

### 1. Skills (`~/.hoop/skills/<name>/`) ✓

**Location:** `hoop-daemon/src/api_skills.rs`, `hoop-mcp/src/skills.rs`

**Features:**
- Manifest-based discovery with `manifest.yml`
- JSON Schema argument validation
- Executable `run` file execution with timeout
- Hot-reload via file watcher
- MCP integration for agent access
- Audit logging to `fleet.db` actions table
- Example `echo` skill seeded on first run

**API Endpoints:**
- `GET /api/skills` - List all skills
- `GET /api/skills/{name}` - Get skill manifest
- `POST /api/skills/{name}/run` - Execute a skill

**Manifest Schema:**
```yaml
name: skill-name          # Required: matches directory name
description: One-liner    # Required: agent-facing description
summary: Human summary    # Required: human-readable purpose
scope: global             # Required: global|project|pattern
args_schema:              # Required: JSON Schema for arguments
  type: object
timeout_secs: 300         # Optional: execution timeout
```

### 2. Scripts (`~/.hoop/scripts/<name>`) ✓

**Location:** `hoop-daemon/src/api_scripts.rs`, `hoop-daemon/src/script_scheduler.rs`, `hoop-daemon/src/script_trigger.rs`

**Features:**
- Single executable file with optional `manifest.yml`
- Operator-invoked execution (UI button, CLI, REST API)
- Event-triggered execution via `on:` subscriptions
- Cron-based scheduling with overlap policies
- Hot-reload via file watcher
- Audit logging to `fleet.db` actions table
- Example `hello-world` script seeded on first run

**API Endpoints:**
- `GET /api/scripts` - List all scripts
- `GET /api/scripts/{name}` - Get script manifest
- `POST /api/scripts/{name}/run` - Execute a script

**Manifest Schema:**
```yaml
name: script-name        # Required: matches filename
description: Summary     # Optional: human-readable
scope: global            # Optional: global|project
timeout_secs: 300        # Optional: execution timeout
schedule: "0 4 * * *"    # Optional: cron schedule
overlap_policy: skip     # Optional: skip|queue|parallel
on:                      # Optional: event subscriptions
  - event: "fail"
    result: "failure"
```

**Trigger Types:**
1. **Manual:** Via UI button or `hoop script run <name>`
2. **Event:** Automatic on matching NEEDLE events
3. **Scheduled:** Cron-style with 60s tick interval

### 3. Notes (`~/.hoop/notes/<name>.md`) ✓

**Location:** `hoop-daemon/src/api_notes.rs`, `hoop-mcp/src/notes.rs`

**Features:**
- Plain markdown with YAML frontmatter
- Global scope at `~/.hoop/notes/`
- Project scope at `<workspace>/.hoop/notes/`
- Hot-reload via file watcher
- MCP integration for agent `read_note()` tool
- Example notes seeded: `team-conventions.md`, `glossary.md`

**API Endpoints:**
- `GET /api/notes` - List all notes (global + projects)
- `GET /api/notes/global` - List global notes only
- `GET /api/notes/project/{project}` - List project notes
- `GET /api/notes/{name}` - Get a specific note

**Note Format:**
```markdown
---
title: Note Title
description: Optional description
tags: [tag1, tag2]
---

# Markdown content here
```

### 4. Prompts (`~/.hoop/prompts/<name>.md`) ✓

**Location:** `hoop-daemon/src/api_prompts.rs`, `hoop-daemon/src/prompt_substitute.rs`

**Features:**
- Plain markdown with YAML frontmatter
- Parameter substitution with `{{var}}` syntax
- Built-in variables: `{{project}}`, `{{file}}`, `{{stitch}}`, `{{now}}`
- Custom operator-passed arguments
- Hot-reload via file watcher
- Example prompts seeded: `fix-linting.md`, `write-plan-stub.md`, `investigate-error.md`

**API Endpoints:**
- `GET /api/prompts` - List all prompts
- `GET /api/prompts/{name}` - Get a prompt
- `POST /api/prompts/{name}/substitute` - Substitute variables

**Prompt Format:**
```markdown
---
name: prompt-name
description: Optional description
args:
  - custom_var
---

## Task
Work on {{project}} with {{custom_var}} in {{file}}.
```

## Hot-Reload Implementation

All four systems use the `notify` crate for file watching:

```rust
// Skills
let _skills_watcher = api_skills::start_watcher(skills_dir, skill_library);

// Scripts
let _scripts_watcher = api_scripts::start_watcher(scripts_dir, script_library);

// Notes
let _notes_watcher = api_notes::start_global_watcher(notes_dir, note_library);

// Prompts
let _prompt_watcher = api_prompts::start_watcher(prompts_dir, prompt_library);
```

Changes take effect within seconds without daemon restart.

## Security

1. **No sandboxing:** Skills and scripts run with HOOP user privileges
2. **Audit logging:**
   - Every skill invocation logged with arguments (hashed)
   - Script executions logged with actor identity
   - Actor identity via Tailscale whois or OS username
3. **Operator responsibility:** Extensions are trusted code

## Documentation

Comprehensive documentation at `docs/concepts/extensibility.md`:
- Overview table
- Manifest schemas
- Quickstart examples
- API reference
- Security best practices
- Sharing instructions

## Closing Criteria Met

- [x] All four plugin dirs hot-reloadable
- [x] Skill manifest schema documented with examples
- [x] Script triggers (manual/event/cron) all functional
- [x] Audit log captures every skill invocation
- [x] Quickstart examples seeded: 1 skill, 1 script, 2 notes, 3 prompts

## Integration Points

1. **Daemon State** (`lib.rs`):
   - `skill_library: SkillStore`
   - `script_library: ScriptStore`
   - `note_library: NoteStore`
   - `prompt_library: PromptStore`
   - `script_scheduler: Option<Arc<ScriptScheduler>>`

2. **Router** (`lib.rs`):
   - `.merge(api_skills::router())`
   - `.merge(api_scripts::router())`
   - `.merge(api_notes::router())`
   - `.merge(api_prompts::router())`

3. **MCP Server** (`hoop-mcp/src/`):
   - `skills.rs` - Agent tool integration
   - `notes.rs` - Agent `read_note()` tool

## Verified Files

- `hoop-daemon/src/api_skills.rs` - Skills REST API, library, watcher
- `hoop-daemon/src/api_scripts.rs` - Scripts REST API, library, watcher
- `hoop-daemon/src/api_notes.rs` - Notes REST API, library, watcher
- `hoop-daemon/src/api_prompts.rs` - Prompts REST API, library, watcher
- `hoop-daemon/src/script_scheduler.rs` - Cron-based script scheduling
- `hoop-daemon/src/script_trigger.rs` - Event-triggered scripts
- `hoop-daemon/src/prompt_substitute.rs` - Parameter substitution engine
- `hoop-mcp/src/skills.rs` - MCP integration for skills
- `hoop-mcp/src/notes.rs` - MCP integration for notes
- `docs/concepts/extensibility.md` - User documentation

## Status: COMPLETE ✓

All extensibility features from plan section §22 are implemented, tested, and documented.
