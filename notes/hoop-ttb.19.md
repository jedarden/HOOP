# Extensibility Implementation Summary (hoop-ttb.19)

## Status: COMPLETE ✓

All four extensibility systems from plan §22 are fully implemented and operational.

## Implementation Summary

### Skills (`~/.hoop/skills/<name>/`)
- **Files**: `hoop-daemon/src/api_skills.rs`, `hoop-mcp/src/skills.rs`
- **Features**:
  - Manifest-based discovery with JSON Schema argument validation
  - Executable `run` files with stdin/stdout JSON communication
  - Hot-reload via file watcher
  - MCP integration exposing skills as agent tools
  - Audit logging to fleet.db actions table
  - Example skills: `echo`, `lookup-git-log`

### Scripts (`~/.hoop/scripts/<name>`)
- **Files**: `hoop-daemon/src/api_scripts.rs`, `hoop-daemon/src/script_scheduler.rs`, `hoop-daemon/src/script_trigger.rs`
- **Features**:
  - Single executable files with optional manifest.yml
  - Three trigger types: manual (CLI/UI), event-driven, cron-scheduled
  - Overlap policies: skip, queue, parallel
  - Hot-reload via file watcher
  - Audit logging with actor identity (Tailscale whois or OS user)
  - Example script: `hello-world`

### Notes (`~/.hoop/notes/<name>.md`)
- **Files**: `hoop-daemon/src/api_notes.rs`, `hoop-mcp/src/notes.rs`
- **Features**:
  - Plain markdown with YAML frontmatter
  - Global and project-scoped notes
  - Hot-reload via file watcher
  - MCP integration for agent `read_note()` tool
  - Example notes: `team-conventions.md`, `glossary.md`

### Prompts (`~/.hoop/prompts/<name>.md`)
- **Files**: `hoop-daemon/src/api_prompts.rs`, `hoop-daemon/src/prompt_substitute.rs`
- **Features**:
  - Plain markdown with YAML frontmatter
  - Handlebars-style `{{var}}` substitution
  - Built-in variables: project, file, stitch, now
  - Custom operator-passed arguments
  - Hot-reload via file watcher
  - Example prompts: `fix-linting.md`, `write-plan-stub.md`, `investigate-error.md`

## Closing Criteria Met

✅ All four plugin dirs hot-reloadable
✅ Skill manifest schema documented with examples
✅ Script triggers (manual/event/cron) all functional
✅ Audit log captures every skill invocation
✅ Quickstart examples seeded: 2 skills, 1 script, 2 notes, 3 prompts

## Documentation

- User guide: `docs/concepts/extensibility.md`
- Verification: `docs/verification/hoop-ttb.19_extensibility_verification.md`
