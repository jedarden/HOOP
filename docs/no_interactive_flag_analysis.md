# no_interactive Flag Analysis - HOOP CLI

**Date:** 2026-08-13
**Bead:** bf-61jjp7
**Task:** Identify all nested subcommands that use the global no_interactive flag

---

## Summary

The `no_interactive` flag is a global clap flag (`--no-interactive` / `-y`) that suppresses all interactive prompts in the HOOP CLI. It is defined with `global = true`, making it available to all subcommands automatically.

**Call chain:** `main.rs` extracts the flag once at parse time → passes to command handlers → handlers use it to skip prompts or require explicit confirmation.

---

## Commands That USE no_interactive

### Top-Level Commands (main.rs)

| Command | Handler | no_interactive Usage |
|---------|---------|---------------------|
| `scan <root>` | `projects::scan_projects(root, no_interactive \|\| auto_confirm)` | Auto-registers all discovered workspaces without prompting |
| `remove <name> --confirm` | `projects::remove_project(name, no_interactive, confirm)` | Requires `--confirm` when `no_interactive=true` |
| `restore --from <uri> --confirm` | `restore::run_restore(from, dry_run, no_interactive, confirm)` | Requires `--confirm` when `no_interactive=true` |
| `init` | `init::run_init_wizard(no_interactive)` | Exits with error when `no_interactive=true` |

### Nested Commands (via handle_projects)

| Projects Subcommand | Handler | no_interactive Usage |
|--------------------|---------|---------------------|
| `projects scan <root>` | `projects::scan_projects(root, no_interactive \|\| yes)` | Auto-registers all discovered workspaces without prompting |
| `projects remove <name> --confirm` | `projects::remove_project(name, no_interactive, confirm)` | Requires `--confirm` when `no_interactive=true` |
| `projects add <path>` | `projects::add_project(path)` | **Does NOT use no_interactive** (no prompts) |
| `projects list [--json]` | `projects::list_projects()` | **Does NOT use no_interactive** (no prompts) |
| `projects show <name>` | `projects::show_project(name)` | **Does NOT use no_interactive** (no prompts) |

---

## Commands That Do NOT Use (or Need) no_interactive

These commands either:
1. Have no interactive prompts to suppress, OR
2. Are handled by functions that don't accept the no_interactive parameter

| Command | Reason |
|---------|--------|
| `serve` | No interactive prompts |
| `add <path>` (top-level) | No interactive prompts |
| `list` | No interactive prompts |
| `status [--project] [--json]` | No interactive prompts |
| `audit check` | No interactive prompts |
| `audit verify` | No interactive prompts |
| `agent` | Not yet implemented |
| `new <project> --dry-run` | No interactive prompts |
| `stitch [--project]` | Not yet implemented |
| `install-systemd` | No interactive prompts |
| `backup trigger` | No interactive prompts (API call) |
| `backup status` | No interactive prompts (API call) |
| `migrate run --confirm` | Has its own `--confirm` flag logic (independent) |
| `migrate status` | No interactive prompts |
| `migrate major-upgrade --confirm` | Has its own `--confirm` flag logic (independent) |
| `migrate rollback --confirm` | Has its own `--confirm` flag logic (independent) |
| `migrate rebuild-percentile-index` | No interactive prompts |
| All `script` subcommands | No interactive prompts (no references) |
| All `config` subcommands | No interactive prompts (no references) |
| All `risk-patterns` subcommands | No interactive prompts (no references) |
| All `skills` subcommands | No interactive prompts (no references) |
| All `pattern` subcommands | No interactive prompts (no references) |
| All `reflection` subcommands | No interactive prompts (no references) |

---

## Flag Propagation Path Through Call Chain

```
main.rs (line 366):
  let no_interactive = cli.no_interactive;

↓

Projects command group (line 394-398):
  Commands::Projects(cmd) => handle_projects(cmd, no_interactive)

↓

handle_projects (line 554):
  ProjectsCommands::Scan { root, yes } 
    => projects::scan_projects(&root, no_interactive || yes)
  
  ProjectsCommands::Remove { name, confirm }
    => projects::remove_project(&name, no_interactive, confirm)

↓

Implementation in projects.rs:
  scan_projects(root, no_interactive) {
    if no_interactive {
      // Auto-register without prompting
    } else {
      // Prompt for each discovery
    }
  }

  remove_project(name, no_interactive, confirm) {
    if no_interactive && !confirm {
      bail!("--confirm is required in non-interactive mode");
    }
    if !no_interactive {
      // Prompt for confirmation
    }
  }
```

---

## Safety Patterns for Destructive Operations

Both `remove` and `restore` follow the same safety pattern:

1. **Require explicit `--confirm` flag** when `no_interactive=true`
2. **Require interactive confirmation** when `no_interactive=false`
3. **Error early** if `--confirm` is missing in non-interactive mode
4. **Provide clear error messages** showing the correct command to run

Example error message from `projects remove`:
```
--confirm is required in non-interactive mode.
 Re-run with: hoop projects remove my-project --no-interactive --confirm
```

---

## Commands That Need Testing vs Commands That Don't

### NEED TESTING (use no_interactive):

- [ ] `scan --no-interactive <root>` - auto-registers all discoveries
- [ ] `scan <root> --yes` - local flag has same effect
- [ ] `scan --no-interactive <root> --yes` - both flags work together
- [ ] `remove --no-interactive <name> --confirm` - requires --confirm
- [ ] `remove --no-interactive <name>` (no --confirm) - should error
- [ ] `remove <name> --confirm` (no --no-interactive) - prompts normally
- [ ] `restore --no-interactive --from <uri> --confirm` - requires --confirm
- [ ] `restore --no-interactive --from <uri>` (no --confirm) - should error
- [ ] `restore --from <uri> --confirm` (no --no-interactive) - prompts normally
- [ ] `init --no-interactive` - should error with clear message
- [ ] `projects scan --no-interactive <root>` - auto-registers all discoveries
- [ ] `projects scan <root> --yes` - local flag has same effect
- [ ] `projects remove --no-interactive <name> --confirm` - requires --confirm
- [ ] `projects remove --no-interactive <name>` (no --confirm) - should error

### DON'T NEED TESTING (don't use no_interactive):

All other commands (serve, add, list, status, audit, backup, migrate, script, config, risk-patterns, skills, pattern, reflection, new, install-systemd) - they either have no interactive prompts or use independent confirmation patterns.

---

## Files That Reference no_interactive

1. **cli.rs** - Defines the CLI structure with the global flag
2. **main.rs** - Extracts and passes the flag to handlers
3. **init.rs** - Uses the flag to exit early with error
4. **restore.rs** - Uses the flag for confirmation logic
5. **projects.rs** - Uses the flag in `scan_projects` and `remove_project`

All other command handler files do NOT reference `no_interactive`:
- backup.rs
- config.rs
- script.rs
- risk_patterns.rs
- skills.rs
- patterns.rs
- reflection.rs
- new.rs
- status.rs

---

## Implementation Pattern Summary

**Safe operations (scan):**
```rust
if no_interactive {
    // Auto-proceed
} else {
    // Prompt normally
}
```

**Destructive operations (remove, restore):**
```rust
if no_interactive && !confirm {
    bail!("--confirm is required in non-interactive mode");
}

if !no_interactive {
    // Prompt for confirmation
}

// Proceed with operation
```

**Wizards (init):**
```rust
if no_interactive {
    bail!("Cannot run in non-interactive mode");
}

// Run wizard prompts
```

---

## Conclusion

The `no_interactive` flag is used consistently across commands that have interactive prompts:

- **4 top-level commands** use it: `scan`, `remove`, `restore`, `init`
- **2 nested commands** use it: `projects scan`, `projects remove`
- **3 command handlers** implement the logic: `scan_projects`, `remove_project`, `run_restore`, `run_init_wizard`

All other commands either have no interactive prompts or use independent confirmation mechanisms (like `migrate` commands).
