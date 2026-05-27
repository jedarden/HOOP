# hoop(1) — HOOP CLI Reference

Reference manual for the `hoop` command-line interface.

## Exit codes

- `0` — Success
- `1` — Partial failure (some subcommand data unavailable)
- `2` — Fatal / precondition not met (e.g., missing `--confirm` for destructive operations)

## Global behavior

**Non-interactive / automation policy:** Every CLI command that queries or reads supports `--json` for machine-readable output (JSON on stdout, no color). Every CLI command that might prompt supports `--no-interactive` / `-y` to accept safe defaults without prompting. Destructive operations additionally require `--confirm` when `--no-interactive` is set; without it, they exit with code 2 and an error message. No prompt is ever emitted to stdout (only stderr) — stdout is always either clean JSON or plain human-readable text suitable for piping.

## Subcommands

### `hoop serve`

Run the daemon (web UI + WebSocket + REST API).

```bash
hoop serve [OPTIONS]
```

**Options:**

- `-a, --addr <ADDR>` — Bind address (default: `127.0.0.1:3000`)
- `--observer` — Observer mode: read-only attach to primary daemon
- `--primary-addr <ADDR>` — Primary daemon address for observer mode (default: `127.0.0.1:3000`)
- `--allow-br-mismatch` — Skip br version compatibility check (dev override)

**Exit codes:** 0 on clean shutdown, 1 on startup error

**Notes:** This is the long-lived process. Other subcommands are clients over a Unix socket (`~/.hoop/control.sock`).

---

### `hoop projects`

Manage the project registry.

#### `hoop projects add <path>`

Add a project to the registry.

```bash
hoop projects add <path>
```

**Arguments:**

- `<path>` — Path to the workspace directory containing `.beads/`

**Exit codes:** 0 on success, 1 if path invalid or already registered

**Example:**

```bash
hoop projects add /home/coding/HOOP/testrepo --name testrepo
```

#### `hoop projects scan <root>`

Auto-register every workspace with `.beads/` under a root path.

```bash
hoop projects scan <root> [OPTIONS]
```

**Arguments:**

- `<root>` — Root path to scan for `.beads/` directories

**Options:**

- `-y, --yes` — Auto-register all discoveries without prompting

**Exit codes:** 0 on success, 1 if no valid workspaces found

**Example:**

```bash
hoop projects scan ~/ --yes
```

#### `hoop projects list`

List registered projects.

```bash
hoop projects list [OPTIONS]
```

**Options:**

- `-j, --json` — Output as JSON

**Exit codes:** 0 on success

**JSON output format:**

```json
[
  {
    "name": "testrepo",
    "workspaces": [
      {
        "path": "/home/coding/HOOP/testrepo",
        "role": "primary"
      }
    ]
  }
]
```

#### `hoop projects remove <name>`

Remove a project from the registry.

```bash
hoop projects remove <name>
```

**Arguments:**

- `<name>` — Project name to remove

**Exit codes:** 0 on success, 1 if project not found

**Note:** Workspace data remains intact at its original location.

#### `hoop projects show <name>`

Show details for a single project.

```bash
hoop projects show <name>
```

**Arguments:**

- `<name>` — Project name

**Exit codes:** 0 on success, 1 if project not found

---

### `hoop status`

CLI overview of fleets, beads, and cost.

```bash
hoop status [OPTIONS] [project]
```

**Arguments:**

- `[project]` — Optional project filter

**Options:**

- `-j, --json` — Output as JSON

**Exit codes:** 0 on success, 1 if daemon not running

**JSON output format:**

```json
{
  "projects": [
    {
      "name": "testrepo",
      "workspaces": [
        {
          "path": "/home/coding/HOOP/testrepo",
          "beads": {
            "total": 42,
            "open": 5,
            "claimed": 3,
            "closed": 34
          }
        }
      ]
    }
  ],
  "daemon": {
    "running": true,
    "pid": 12345,
    "version": "1.0.0"
  }
}
```

---

### `hoop audit`

Audit operations for runtime health and log integrity.

#### `hoop audit check`

Startup binary and environment audit.

```bash
hoop audit check [OPTIONS]
```

**Options:**

- `-j, --json` — Output as JSON
- `--strict` — Skip optional checks (Tailscale, systemd)

**Exit codes:** 0 if all critical checks pass, 1 if any critical check fails

**Checks performed:**

- `br` binary availability and version
- Port availability (default: 3000)
- Tailscale status (optional, skipped with `--strict`)
- systemd availability (optional, skipped with `--strict`)

#### `hoop audit verify`

Verify audit log hash chain integrity.

```bash
hoop audit verify [OPTIONS]
```

**Options:**

- `-j, --json` — Output as JSON

**Exit codes:** 0 if hash chain is intact, 1 if verification fails

---

### `hoop agent`

Attach to or start the human-interface agent conversation.

```bash
hoop agent
```

**Exit codes:** 0 on normal exit, 1 on connection error

**Note:** Requires the daemon to be running (`hoop serve`).

---

### `hoop new`

CLI shortcut to draft and submit a Stitch.

```bash
hoop new <project> [OPTIONS]
```

**Arguments:**

- `<project>` — Target project name

**Options:**

- `--dry-run` — Validate and print the payload without submitting

**Exit codes:** 0 on success, 1 on validation error

**Note:** This is a convenience wrapper for creating Stitches via the CLI instead of the web UI.

---

### `hoop stitch`

List open Stitches.

#### `hoop stitch list [project]`

List open Stitches with optional project filter.

```bash
hoop stitch list [project]
```

**Arguments:**

- `[project]` — Optional project filter

**Exit codes:** 0 on success

**Note:** For underlying bead view, use `--beads` flag.

---

### `hoop restore`

Restore from a prior snapshot (requires daemon stopped).

```bash
hoop restore --from <s3-uri> [OPTIONS]
```

**Options:**

- `--from <s3-uri>` — S3 URI: `s3://<bucket>/<prefix>/<snapshot-id>` (required)
- `--dry-run` — Validate manifest and show what would be restored without making changes

**Exit codes:** 0 on success, 1 on restore failure, 2 if `--confirm` missing in non-interactive mode

**Example:**

```bash
hoop restore --from s3://hoop-backups/prod/snapshot-20250527 --dry-run
```

---

### `hoop migrate`

Manage schema migrations.

#### `hoop migrate run --confirm`

Run pending migrations (minor version upgrades only).

```bash
hoop migrate run --confirm
```

**Options:**

- `--confirm` — Required safety confirmation

**Exit codes:** 0 on success, 1 on migration failure, 2 if `--confirm` missing

**Note:** Verify you have a current backup before running.

#### `hoop migrate status [--json]`

Show migration status and pending migrations.

```bash
hoop migrate status [OPTIONS]
```

**Options:**

- `-j, --json` — Output as JSON

**Exit codes:** 0 on success

#### `hoop migrate major-upgrade --from <version> --confirm`

Perform a major version upgrade (e.g., 1.x → 2.x).

```bash
hoop migrate major-upgrade [OPTIONS] --confirm
```

**Options:**

- `--from <version>` — Source major version (e.g., `1` for 1.x → 2.x) as a safety check
- `--confirm` — Required safety confirmation

**Exit codes:** 0 on success, 1 on upgrade failure, 2 if `--confirm` missing or version mismatch

#### `hoop migrate rollback <version> --confirm`

Rollback to a previous minor version.

```bash
hoop migrate rollback <version> --confirm
```

**Arguments:**

- `<version>` — Target version to rollback to

**Options:**

- `--confirm` — Required safety confirmation

**Exit codes:** 0 on success, 1 if rollback not supported, 2 if `--confirm` missing

**Note:** Major version upgrades cannot be rolled back.

#### `hoop migrate rebuild-percentile-index`

Rebuild the percentile index from closed Stitches.

```bash
hoop migrate rebuild-percentile-index
```

**Exit codes:** 0 on success, 1 if table doesn't exist

---

### `hoop risk-patterns`

Manage risk patterns for automatic issue detection.

#### `hoop risk-patterns list`

List all configured risk patterns.

```bash
hoop risk-patterns list [OPTIONS]
```

**Options:**

- `-j, --json` — Output as JSON

**Exit codes:** 0 on success

#### `hoop risk-patterns add`

Add a new risk pattern.

```bash
hoop risk-patterns add --id <id> --name <name> --description <desc> \
  --keywords <kw> --label-keywords <kw> --fix-recommendation <fix> \
  --severity <sev> --category <cat>
```

**Options:**

- `--id <id>` — Pattern ID (unique identifier, required)
- `--name <name>` — Pattern name (required)
- `--description <desc>` — Pattern description (required)
- `--keywords <kw>` — Comma-separated keywords that trigger this pattern (required)
- `--label-keywords <kw>` — Comma-separated label keywords that increase confidence
- `--fix-recommendation <fix>` — Recommended fix approach (required)
- `--severity <sev>` — Severity level: `low`, `medium`, `high`, `critical` (required)
- `--category <cat>` — Pattern category: `performance`, `correctness`, `security`, `integration`, `code_quality`, `infrastructure` (required)

**Exit codes:** 0 on success, 1 on validation error, 2 if pattern ID already exists

**Example:**

```bash
hoop risk-patterns add \
  --id perf-n-plus-1 \
  --name "N+1 Query Pattern" \
  --description "Detects potential N+1 query issues" \
  --keywords "query,loop,database,n+1" \
  --label-keywords "performance,slow" \
  --fix-recommendation "Use batch loading or a data loader" \
  --severity high \
  --category performance
```

#### `hoop risk-patterns seed [--force]`

Seed initial risk patterns (first-run setup).

```bash
hoop risk-patterns seed [OPTIONS]
```

**Options:**

- `--force` — Force re-seed even if patterns exist

**Exit codes:** 0 on success, 2 if patterns already exist (without `--force`)

---

### `hoop script`

Manage and run operator-invoked scripts.

#### `hoop script run <name> [args...]`

Run a script by name.

```bash
hoop script run <name> [args...]
```

**Arguments:**

- `<name>` — Script name to run
- `[args...]` — Arguments to pass to the script

**Exit codes:** 0 on success, script's exit code if non-zero, 1 on execution error

#### `hoop script list [--project <proj>]`

List all available scripts.

```bash
hoop script list [OPTIONS]
```

**Options:**

- `-p, --project <proj>` — Filter by project (shows global + matching project scripts)

**Exit codes:** 0 on success

#### `hoop script show <name>`

Show details for a specific script.

```bash
hoop script show <name>
```

**Arguments:**

- `<name>` — Script name

**Exit codes:** 0 on success, 1 if script not found

---

### `hoop init`

First-time setup wizard.

```bash
hoop init
```

**Exit codes:** 0 on success, 1 if any step fails

**Wizard stages:**

1. Dependency check (runs `hoop audit check`)
2. First project registration (offers `scan ~/` preview)
3. Agent adapter setup (optional; Claude, ZAI)
4. systemd install (optional)
5. Health check and URL display

---

### `hoop config`

Manage daemon configuration.

#### `hoop config diff`

Show configuration diff (running vs config.yml).

```bash
hoop config diff
```

**Exit codes:** 0 on success, 1 if config file not found

---

### `hoop backup`

Manage backups.

#### `hoop backup create`

Create a backup snapshot.

```bash
hoop backup create [OPTIONS]
```

**Options:**

- `--description <desc>` — Backup description

**Exit codes:** 0 on success, 1 on backup failure

#### `hoop backup list`

List available backups.

```bash
hoop backup list [OPTIONS]
```

**Options:**

- `-j, --json` — Output as JSON

**Exit codes:** 0 on success

---

### `hoop skills`

Manage agent-invocable skills.

#### `hoop skills list`

List available skills.

```bash
hoop skills list [OPTIONS]
```

**Options:**

- `-j, --json` — Output as JSON

**Exit codes:** 0 on success

---

### `hoop pattern`

Manage patterns (operator-curated groups of Stitches).

#### `hoop pattern list`

List available patterns.

```bash
hoop pattern list [OPTIONS]
```

**Options:**

- `-j, --json` — Output as JSON

**Exit codes:** 0 on success

---

## Deprecated / removed commands

The following commands are **not** part of HOOP and belong to NEEDLE:

- `hoop launch` — Use NEEDLE directly
- `hoop stop` — Use NEEDLE directly
- `hoop salvage` — Use `br` directly for bead recovery
- `hoop steer` — Capacity management is NEEDLE's concern

---

## See also

- [README.md](../README.md) — Project overview
- [docs/operations.md](operations.md) — Systemd service, logs, upgrades, backups
- [docs/troubleshooting.md](troubleshooting.md) — Common failures mapped to `hoop audit` output
- [docs/plan/plan.md](plan/plan.md) — Full implementation plan
