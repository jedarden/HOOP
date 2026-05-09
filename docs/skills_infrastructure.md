# Skills Plugin Infrastructure (§22)

## Overview

The skills plugin infrastructure allows operators to extend HOOP with custom tools that the human-interface agent can invoke. Skills are discovered from `~/.hoop/skills/<name>/` and automatically exposed as MCP tools.

## Acceptance Status

| Requirement | Status | Implementation |
|-------------|--------|----------------|
| Manifest schema documented with example | ✅ Complete | `hoop-daemon/src/api_skills.rs` lines 8-34 |
| Skill registration auto-augments agent's tool-belt | ✅ Complete | `hoop-mcp/src/tools.rs` via `skills_to_mcp_tools()` |
| Hot-reload on directory change | ✅ Complete | `start_watcher()` in `api_skills.rs`, called in `lib.rs` |
| Audit logs every invocation with args + result | ✅ Complete | `run_skill()` writes to `actions` table in fleet.db |
| Example skill shipped | ✅ Complete | `echo` and `lookup-git-log` seeded in `seed_example_skill()` |

## Directory Structure

```
~/.hoop/skills/<name>/
  manifest.yml    # Skill manifest (required)
  run             # Executable invoked with args JSON on stdin (required, +x)
  README.md       # Human documentation (optional)
```

## Manifest Schema

```yaml
name: skill-name          # Required: matches directory name
description: One-liner    # Required: agent-facing description
summary: Human summary    # Required: human-readable purpose
scope: global             # Required: global|project|pattern
projects:                 # Optional: projects where skill is available (scope=project)
  - project-a
pattern: fix-*            # Optional: bead title pattern (scope=pattern)
args_schema:              # Required: JSON Schema for arguments
  type: object
  properties:
    url:
      type: string
  required:
    - url
timeout_secs: 300        # Optional: execution timeout (default: 300)
```

## Execution Flow

1. Agent invokes skill via MCP tool (`skill_<name>`)
2. Arguments validated against manifest's `args_schema`
3. If valid: `run` executable spawned with args JSON on stdin
4. If invalid: validation error returned without execution
5. Result captured (stdout/stderr/exit code/timeout)
6. Audit row written to `fleet.db` actions table

## REST API Endpoints

- `GET /api/skills` — List all discovered skills
- `GET /api/skills/:name` — Get single skill manifest
- `POST /api/skills/:name/run` — Execute a skill

## Example Skills

### echo
Simple echo skill for testing the skills system.

```bash
echo '{"message": "Hello, HOOP!"}' | ~/.hoop/skills/echo/run
# Output: {"output": "Hello, HOOP!"}
```

### lookup-git-log
Practical skill for querying git commit history.

```bash
echo '{"project_path": "/home/coding/HOOP", "max_count": 10, "author": "jedarden"}' | \
  ~/.hoop/skills/lookup-git-log/run
```

## Code Locations

- **Daemon skills library**: `hoop-daemon/src/api_skills.rs`
- **MCP skills integration**: `hoop-mcp/src/skills.rs`
- **Daemon initialization**: `hoop-daemon/src/lib.rs` (lines 2717-2734)
- **MCP tool registration**: `hoop-mcp/src/tools.rs` (lines 263-282, 334-407)

## Testing

Run skills tests with:

```bash
cd hoop-daemon && cargo test --package hoop-daemon skills
cd hoop-mcp && cargo test --package hoop-mcp skills
```

## Security Considerations

- Skills run with the HOOP user's privileges (no sandboxing)
- Arguments validated against JSON Schema before execution
- Execution timeout prevents runaway processes
- Audit trail recorded for every invocation
- Path traversal hardening applies to file operations
