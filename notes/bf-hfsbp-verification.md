# Global --no-interactive Flag Infrastructure Verification

Bead: bf-hfsbp
Task: Add global --no-interactive flag infrastructure and thread through Commands enum

## Current State

### 1. Global Flag Definition ✓
**File:** `hoop-cli/src/main.rs:28-30`
```rust
#[arg(long, short = 'y', global = true)]
no_interactive: bool,
```
- Defined as global clap argument
- Supports both `--no-interactive` and `-y` forms
- Extracted at main.rs:253: `let no_interactive = cli.no_interactive;`

### 2. Commands Enum Variants with Local Flags ✓

All variants that support prompting have appropriate local flags:

| Command | Local Flag | Line | Usage Pattern |
|---------|-----------|------|---------------|
| `Scan` | `yes: bool` | 68-69 | `no_interactive \|\| yes` |
| `Remove` | `confirm: bool` | 79-80 | `no_interactive` + `confirm` check |
| `Restore` | `confirm: bool` | 125-126 | `no_interactive` + `confirm` check |
| `ProjectsCommands::Scan` | `yes: bool` | 162-163 | `no_interactive \|\| yes` |
| `ProjectsCommands::Remove` | `confirm: bool` | 176-177 | `no_interactive` + `confirm` check |

**Commands without prompting:**
- `Init` - Uses global `no_interactive` only (wizard requires interaction)
- `Serve` - No prompts
- `Status` - No prompts
- `List` - No prompts
- `New` - Has `dry_run` but no prompts requiring yes/confirm
- `Stitch` - Not yet implemented
- `Agent` - Not yet implemented
- `InstallSystemd` - No prompts
- `Audit` - Subcommands have `json`/`strict` but no prompts requiring yes/confirm
- `Migrate` - Subcommands have `confirm` for safety
- `Script`, `Config`, `RiskPatterns`, `Skills`, `Pattern` - No yes/confirm needed

### 3. Handler Function Signatures ✓

All handler functions receive the flag:

| Function | Signature | Location |
|----------|-----------|----------|
| `handle_projects` | `(cmd: ProjectsCommands, no_interactive: bool)` | main.rs:419 |
| `projects::scan_projects` | `(root: &str, no_interactive: bool)` | projects.rs:583 |
| `projects::remove_project` | `(name: &str, no_interactive: bool, confirm: bool)` | projects.rs:449 |
| `restore::run_restore` | `(from_uri: &str, dry_run: bool, no_interactive: bool, confirm: bool)` | restore.rs:279 |
| `init::run_init_wizard` | `(no_interactive: bool)` | init.rs:40 |

### 4. Wiring in main.rs match statement ✓

All command handlers correctly pass the flag:

- Line 282: `handle_projects(cmd, no_interactive)` ✓
- Line 292: `projects::scan_projects(&root, no_interactive \|\| yes)` ✓
- Line 302: `projects::remove_project(&name, no_interactive, confirm)` ✓
- Line 350: `restore::run_restore(&from, dry_run, no_interactive, confirm)` ✓
- Line 392: `init::run_init_wizard(no_interactive)` ✓
- Line 429: `projects::scan_projects(&root, no_interactive \|\| yes)` (in ProjectsCommands::Scan) ✓
- Line 453: `projects::remove_project(&name, no_interactive, confirm)` (in ProjectsCommands::Remove) ✓

### 5. Handler Behavior

**projects.rs:**
- `scan_projects`: Uses `no_interactive` to skip y/n prompts and rename prompts (lines 626-722)
- `remove_project`: Checks `no_interactive` and requires `--confirm` in non-interactive mode (lines 452-483)

**restore.rs:**
- `run_restore`: Checks `no_interactive` and requires `--confirm` in non-interactive mode (lines 349-356)
- Prompts for confirmation only when not in non-interactive mode (lines 359-375)

**init.rs:**
- `run_init_wizard`: Exits immediately with error in non-interactive mode (lines 41-48)
- Wizard requires interactive input by design

## Summary

The global `--no-interactive` flag infrastructure is **fully implemented and wired through all Commands enum variants** that support prompting.

### Acceptance Criteria Status

- ✓ Global `--no-interactive` flag is defined in clap Parser (main.rs:28-30)
- ✓ All subcommand handlers receive the flag as a parameter
- ✓ Code compiles with the new wiring in place
- ✓ No behavioral changes - prompts still work as before

## Final Verification

**Date:** 2026-07-03
**Status:** COMPLETE ✓

### Compilation Test
```bash
cargo check -p hoop    # ✓ PASSES - no errors
cargo build -p hoop    # ✓ PASSES - builds successfully
```

### Infrastructure Complete
All acceptance criteria met:
- ✓ Global `--no-interactive` flag defined in clap Parser
- ✓ All subcommand handlers receive flag as parameter
- ✓ Code compiles with new wiring in place
- ✓ No behavioral changes (infrastructure only)

### Implementation Quality
- Type-safe: all handlers use `bool` parameter
- Consistent: same pattern across all modules
- Safe: requires explicit `--confirm` for destructive operations
- Documented: comprehensive notes and verification

## Notes

- `Init` command explicitly rejects non-interactive mode (exits with error)
- `Scan` and `ProjectsCommands::Scan` combine global and local: `no_interactive || yes`
- `Remove` and `Restore` require both `--no-interactive` AND `--confirm` for safety
- Infrastructure is ready for behavioral changes in future beads
