# Bead bf-1zs72: Add no_interactive skip to scan_projects

## Task Completion Status: ✅ ALREADY IMPLEMENTED

The functionality described in this bead was already implemented in the codebase prior to this task.

## Verification

### Implementation Location
The `scan_projects` function in `hoop-cli/src/projects.rs` (lines 589-753) already includes:

1. **Function signature** (line 589):
   ```rust
   pub fn scan_projects(root: &str, no_interactive: bool) -> Result<()>
   ```

2. **Non-interactive mode** (lines 632-666):
   - When `no_interactive == true`, automatically registers all discovered workspaces
   - No prompts, no user input required
   - Prints progress to stdout

3. **Interactive mode** (lines 667-727):
   - When `no_interactive == false`, prompts user per discovery
   - Allows optional project renaming
   - Prompts go to stderr

### Command Integration
Both command invocations correctly pass the flag:

**Main command** (`main.rs` line 405):
```rust
Commands::Scan { root, yes } => {
    projects::scan_projects(&root, no_interactive || yes)
}
```

**Projects subcommand** (`main.rs` line 548):
```rust
ProjectsCommands::Scan { root, yes } => {
    projects::scan_projects(&root, no_interactive || yes)
}
```

### Usage Examples

All of these work correctly:

```bash
# Global flag
hoop --no-interactive scan /path/to/projects
hoop -y scan /path/to/projects

# Command-specific flag
hoop scan --yes /path/to/projects
hoop projects scan --yes /path/to/projects

# Projects subcommand with global flag
hoop --no-interactive projects scan /path/to/projects
```

## Acceptance Criteria Met

- ✅ `scan_projects` respects `no_interactive` flag and skips all prompts
- ✅ Command completes successfully without user interaction when `no_interactive` is true
- ✅ Default behavior (interactive mode) is unchanged
- ✅ User can run: `hoop --no-interactive scan /path/to/projects` (should not prompt)

## Conclusion

This bead's requirements were already satisfied by existing implementation. No code changes were needed.
