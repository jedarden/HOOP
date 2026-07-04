# Bead bf-hfsbp: Global --no-interactive Flag Infrastructure

## Summary
Implemented global `--no-interactive` / `-y` flag infrastructure and threaded it through all relevant Commands enum handlers.

## Changes Made

### hoop-cli/src/main.rs
- Added global `no_interactive` flag to Cli struct (line 28-30)
- Flag is global with short alias `-y`
- Extracted flag value at main() entry (line 253)
- Threading through commands:
  - `handle_projects(cmd, no_interactive)` - line 282
  - `scan_projects(&root, no_interactive || yes)` - line 292
  - `remove_project(&name, no_interactive, confirm)` - line 302
  - `run_restore(&from, dry_run, no_interactive, confirm)` - line 350
  - `run_init_wizard(no_interactive)` - line 392

### hoop-cli/src/init.rs
- Updated `run_init_wizard()` signature to accept `no_interactive: bool`
- Rejects non-interactive mode with clear error message (init wizard requires prompts)

### hoop-cli/src/projects.rs
- Updated `scan_projects()` signature to accept `no_interactive: bool`
- Updated `remove_project()` signature to accept `no_interactive: bool` and `confirm: bool`
- Both handlers check for `--confirm` requirement when `no_interactive` is true

### hoop-cli/src/restore.rs
- Updated `run_restore()` signature to accept `no_interactive: bool` and `confirm: bool`
- Handler checks for `--confirm` requirement when `no_interactive` is true

## Commands Enum Confirmation Fields
Commands that support prompting now have explicit confirmation fields:
- `Scan { yes: bool }` - local `-y` flag for auto-confirmation
- `Remove { confirm: bool }` - requires `--confirm` in non-interactive mode
- `Restore { confirm: bool }` - requires `--confirm` in non-interactive mode
- `ProjectsCommands::Scan { yes: bool }` - local `-y` flag for auto-confirmation
- `ProjectsCommands::Remove { confirm: bool }` - requires `--confirm` in non-interactive mode
- `Init` - rejects non-interactive mode (wizard requires interactive input)

## Acceptance Criteria Met
✓ Global `--no-interactive` flag defined in clap Parser
✓ All subcommand handlers receive flag as parameter
✓ Code compiles with new wiring in place
✓ No behavioral changes (infrastructure only)

## Usage
```bash
# Non-interactive mode with global flag
hoop --no-interactive scan ~/ --no-interactive
hoop -y projects scan ~/ --confirm

# Local flags still work
hoop scan ~/ -y
hoop projects remove my-project --confirm
```

## Next Steps
This infrastructure enables future implementation of actual non-interactive behavior in individual handlers.
