# `no_interactive` Flag Survey - HOOP CLI

**Bead:** bf-61jjp7  
**Date:** 2026-08-13  
**Purpose:** Identify all nested subcommands that use the `no_interactive` flag

## Overview

The `no_interactive` flag is defined as a **global flag** in the top-level `Cli` struct (main.rs:139):

```rust
#[arg(short = 'y', long = "no-interactive", global = true)]
no_interactive: bool,
```

Because of `global = true`, this flag is automatically available to all subcommands without redefining it. The flag is extracted once at parse time (main.rs:366) and passed down to command handlers:

```rust
let no_interactive = cli.no_interactive;
```

## Call Chain Flow

```
CLI Parsing → main.rs extraction → Command Handlers
     ↓              ↓                    ↓
  Cli struct    no_interactive      Handler functions
                 variable
```

## Commands That USE the `no_interactive` Flag

### 1. Scan (top-level and projects scan)
- **Path:** `projects::scan_projects` (projects.rs:633)
- **Handler signature:** `pub fn scan_projects(root: &str, no_interactive: bool)`
- **Usage:** 
  - When `no_interactive=true`: Auto-registers all discovered workspaces without prompting
  - When `no_interactive=false`: Prompts for each discovery with `y/N`
- **Call chain:**
  ```
  main.rs:407 → projects::scan_projects(&root, no_interactive || auto_confirm)
  handle_projects:564 → projects::scan_projects(&root, no_interactive || yes)
  ```
  - Note: Combines `no_interactive` with local `--yes` flag

### 2. Remove (top-level and projects remove)
- **Path:** `projects::remove_project` (projects.rs:487)
- **Handler signature:** `pub fn remove_project(name: &str, no_interactive: bool, confirm: bool)`
- **Usage:**
  - When `no_interactive=true`: Requires `--confirm` flag (safety check)
  - When `no_interactive=false`: Prompts for confirmation
- **Call chain:**
  ```
  main.rs:431 → projects::remove_project(&name, no_interactive, confirm)
  handle_projects:588 → projects::remove_project(&name, no_interactive, confirm)
  ```
  - Safety: Errors with `--confirm is required in non-interactive mode` if `confirm=false`

### 3. Restore
- **Path:** `restore::run_restore` (restore.rs:279)
- **Handler signature:** `pub async fn run_restore(from_uri: &str, dry_run: bool, no_interactive: bool, confirm: bool)`
- **Usage:**
  - When `no_interactive=true`: Requires `--confirm` flag (DESTRUCTIVE operation)
  - When `no_interactive=false`: Prompts for confirmation with warning
- **Call chain:**
  ```
  main.rs:479 → restore::run_restore(&from, dry_run, no_interactive, confirm)
  ```
  - Safety: Errors with `--confirm is required in non-interactive mode` if `confirm=false`
  - Destructive nature: Replaces entire `~/.hoop/` directory

### 4. Init
- **Path:** `init::run_init_wizard` (init.rs:40)
- **Handler signature:** `pub fn run_init_wizard(no_interactive: bool)`
- **Usage:**
  - When `no_interactive=true`: **EXPLICITLY REJECTS** and exits with code 2
  - When `no_interactive=false`: Runs interactive wizard
- **Call chain:**
  ```
  main.rs:521 → init::run_init_wizard(no_interactive)
  ```
  - Special case: The only command that **errors** when `no_interactive=true`
  - Reason: Init wizard requires interactive input for configuration

## Commands That Do NOT Use the `no_interactive` Flag

The following command handlers exist but do NOT have `no_interactive` in their function signature:

### Backup Commands
- **File:** `backup.rs`
- **Handler:** `handle_backup(cmd: BackupCommands)` (line 26)
- **Subcommands:** `Trigger`, `Status`
- **Current behavior:** No interactive prompts, flag accepted but unused

### Script Commands
- **File:** `script.rs`
- **Handler:** `handle_script(cmd: ScriptCommands)` (line 257)
- **Subcommands:** `Run`, `List`, `Show`
- **Current behavior:** No interactive prompts, flag accepted but unused

### Config Commands
- **File:** `config.rs`
- **Handler:** `handle_config(cmd: ConfigCommands)` (line 22)
- **Subcommands:** `Diff`, `Validate`
- **Current behavior:** No interactive prompts, flag accepted but unused

### RiskPatterns Commands
- **File:** `risk_patterns.rs`
- **Handler:** `handle_risk_patterns(cmd: RiskPatternsCommands)` (line 56)
- **Subcommands:** `Add`, `List`, `Seed`
- **Current behavior:** No interactive prompts, flag accepted but unused

### Skills Commands
- **File:** `skills.rs`
- **Handler:** `handle_skills(cmd: SkillsCommands)` (line 650)
- **Subcommands:** `Import`, `Enable`, `Disable`, `List`, `Show`, `Remove`
- **Current behavior:** No interactive prompts, flag accepted but unused

### Pattern Commands
- **File:** `patterns.rs`
- **Handler:** `handle_patterns(cmd: PatternCommands)` (line 152)
- **Subcommands:** `New`, `List`, `Show`, `Update`, `Close`, `Delete`, `AddMember`, `RemoveMember`, `AddQuery`, `RemoveQuery`
- **Current behavior:** No interactive prompts (Delete has `--confirm` but doesn't use `no_interactive`)

### Reflection Commands
- **File:** `reflection.rs`
- **Handler:** `handle_reflection(cmd: ReflectionCommands)` (line 258)
- **Subcommands:** `Export`
- **Current behavior:** No interactive prompts, flag accepted but unused

### Migrate Commands (via main.rs)
- **File:** `main.rs` (inline handler)
- **Function:** `handle_migrate(cmd: MigrateCommands)` (line 690)
- **Subcommands:** `Run`, `Status`, `MajorUpgrade`, `Rollback`, `RebuildPercentileIndex`
- **Current behavior:** Has `--confirm` flags but doesn't use `no_interactive`

### Audit Commands (via main.rs)
- **File:** `main.rs` (inline handler)
- **Function:** `handle_audit(cmd: AuditCommands)` (line 629)
- **Subcommands:** `Check`, `Verify`
- **Current behavior:** No interactive prompts, flag accepted but unused

### Other Commands (via main.rs)
- **Status** (line 440): No interactive prompts
- **Agent** (line 452): Not yet implemented
- **New** (line 456): No interactive prompts
- **Stitch** (line 462): Not yet implemented
- **InstallSystemd** (line 466): No interactive prompts
- **List** (line 412): No interactive prompts
- **Add** (line 400): No interactive prompts

## Testing Recommendations

### Commands Requiring `no_interactive` Testing
These commands have different behavior based on the flag value:

1. **Scan** (`projects scan` / `hoop scan`)
   - Test auto-registration with `no_interactive=true`
   - Test interactive prompts with `no_interactive=false`

2. **Remove** (`projects remove` / `hoop remove`)
   - Test `--confirm` requirement with `no_interactive=true`
   - Test interactive prompts with `no_interactive=false`
   - Test error when `no_interactive=true` without `--confirm`

3. **Restore** (`hoop restore`)
   - Test `--confirm` requirement with `no_interactive=true`
   - Test interactive prompts with `no_interactive=false`
   - Test error when `no_interactive=true` without `--confirm`
   - Test destructive nature and rollback

4. **Init** (`hoop init`)
   - Test early exit with error when `no_interactive=true`
   - Test wizard continues when `no_interactive=false`
   - Test error message quality

### Commands NOT Requiring `no_interactive` Testing
These commands do not change behavior based on the flag:

- All backup, script, config, risk_patterns, skills, pattern, reflection commands
- Migrate commands (have their own `--confirm` flags)
- Audit commands
- Status, agent, new, stitch, install-systemd, list, add

**Note:** These commands accept the global flag (because it's global) but do not use it in their logic.

## Pattern Summary

### Implementation Pattern
Commands that use `no_interactive` follow this pattern:

1. **Accept parameter:** Handler includes `no_interactive: bool` in signature
2. **Check before prompting:** Test flag before interactive prompts
3. **Require explicit confirm:** For destructive operations, require `--confirm` when `no_interactive=true`
4. **Exit early:** Some commands (like init) exit immediately when `no_interactive=true`

### Flag Propagation Path
```
main.rs (parse time)
  ↓ extract: let no_interactive = cli.no_interactive;
  ↓ match cli.command
  ↓
  ├─► Scan → projects::scan_projects(no_interactive)
  ├─► Remove → projects::remove_project(no_interactive, confirm)
  ├─► Restore → restore::run_restore(no_interactive, confirm)
  ├─► Init → init::run_init_wizard(no_interactive)
  └─► Others → handler functions (flag unused)
```

## Conclusion

**Total commands surveyed:** 20+ command groups  
**Commands using `no_interactive`:** 4 (Scan, Remove, Restore, Init)  
**Commands not using `no_interactive`:** 16+ (all others)

The `no_interactive` flag is a global flag that flows through the CLI parsing infrastructure but is only actively used by a small subset of commands. Most commands accept the flag (because it's global) but don't change their behavior based on its value.

For subsequent testing beads, focus on the 4 commands that actually use the flag: Scan, Remove, Restore, and Init.
