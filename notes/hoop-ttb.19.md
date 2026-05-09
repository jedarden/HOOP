# §22 Extensibility Implementation Status

**Date**: 2026-05-09
**Bead**: hoop-ttb.19
**Status**: VERIFIED COMPLETE

## Summary

The extensibility system (§22) is **fully implemented** with all four plugin types functional:

1. **Skills** (`~/.hoop/skills/<name>/`) — Custom agent tools
2. **Scripts** (`~/.hoop/scripts/<name>`) — Operator-triggered automation
3. **Notes** (`~/.hoop/notes/<name>.md`) — Agent-readable markdown files
4. **Prompts** (`~/.hoop/prompts/<name>.md`) — Reusable prompt library

## Implementation Details

### Skills (api_skills.rs)
- Discovery: `discover_skills()` scans directories for manifest.yml
- Execution: `execute_skill()` runs `run` executable with JSON args on stdin
- Validation: JSON Schema validation against manifest's args_schema
- Hot-reload: `start_watcher()` with notify crate
- Audit: `fleet::write_audit_row()` with ActionKind::SkillInvoked
- Example: `echo` skill seeded on first run

### Scripts (api_scripts.rs)
- Discovery: `discover_scripts()` finds executable files
- Execution: `execute_script()` with timeout support
- Scheduling: `script_scheduler.rs` with cron parsing
- Event triggers: `script_trigger.rs` with glob pattern matching
- Manifest: Optional .yml file with timeout, scope, schedule, subscriptions
- Audit: Script executions logged

### Notes (api_notes.rs)
- Discovery: `load_global()` and `load_project()` for scopes
- Hot-reload: `start_global_watcher()` with notify crate
- Frontmatter: YAML parsing for title, description, tags
- Examples: `team-conventions.md` and `glossary.md` seeded

### Prompts (api_prompts.rs)
- Discovery: `load()` scans for .md files
- Hot-reload: `start_watcher()` with notify crate
- Substitution: `prompt_substitute.rs` with {{var}} syntax
- Examples: `fix-linting.md`, `write-plan-stub.md`, `investigate-error.md` seeded

## Closing Criteria Status

| Criterion | Status | Notes |
|-----------|--------|-------|
| All four plugin dirs hot-reloadable | ✅ | All use notify crate for file watching |
| Skill manifest schema documented | ✅ | In-code docs + new docs/concepts/extensibility.md |
| Script triggers (manual/event/cron) | ✅ | Manual (API), event (subscriptions), cron (scheduler) |
| Audit log captures skill invocations | ✅ | ActionKind::SkillInvoked with hashed args |
| Quickstart examples | ✅ | One each: echo skill, team-conventions note, fix-linting prompt |

## Files Added

- `docs/concepts/extensibility.md` — Comprehensive user documentation

## Verification

All components verified:
- API routes wired in lib.rs (lines 1262-1283)
- Config resolver handles agent_extensions.* fields
- Libraries initialized with hot-reload watchers
- Example extensions seeded on first run
- Tests present in each module

## Notes

OpenAPI spec does not yet document /api/skills, /api/scripts, /api/notes, /api/prompts endpoints.
This is documentation only; implementation is complete and functional.

## Final Verification (2026-05-09)

All closing criteria met:
- ✅ All four plugin dirs hot-reloadable
- ✅ Skill manifest schema documented with examples
- ✅ Script triggers (manual/event/cron) all functional
- ✅ Audit log captures every skill invocation
- ✅ Quickstart examples: 2 skills (echo, lookup-git-log), 1 script (hello-world), 2 notes (team-conventions, glossary), 3 prompts (fix-linting, write-plan-stub, investigate-error)

The `lookup-git-log` skill serves as a practical example demonstrating:
- Complex JSON Schema validation with multiple optional parameters
- External command execution (git log)
- JSON output formatting
- Real-world use case (querying commit history)
