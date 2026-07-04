# `no_interactive` Flag Audit

**Purpose:** Document all CLI commands and their relationship to the global `--no-interactive` flag.

**Definition:** The `no_interactive` flag is a global flag (`--no-interactive` / `-y`) defined in main.rs that suppresses all interactive prompts. When set, commands that normally prompt for confirmation must either require an explicit `--confirm` flag or auto-proceed safely.

## Flag Definition

**Location:** `hoop-cli/src/main.rs:34-35`

```rust
#[arg(short = 'y', long = "no-interactive", global = true)]
no_interactive: bool,
```

**Extraction:** Line 258: `let no_interactive = cli.no_interactive;`

**Behavior:** Commands that have interactive prompts should:
1. Accept the `no_interactive` parameter in their handler
2. When `no_interactive=false`: prompt the user interactively
3. When `no_interactive=true`: either auto-proceed OR require `--confirm` flag

## Inventory

### Top-Level Commands

| Command | Uses `no_interactive` | Interactive Prompts | Should Use Flag | Notes |
|---------|----------------------|-------------------|----------------|-------|
| `Serve` | ❌ No | ❌ No | ❌ No | Pure daemon startup |
| `Add` | ❌ No | ❌ No | ❌ No | Direct operation |
| `Scan` | ✅ Yes | ✅ Yes | ✅ Yes | Currently: `no_interactive \|\| yes` |
| `List` | ❌ No | ❌ No | ❌ No | Read-only |
| `Remove` | ✅ Yes | ✅ Yes | ✅ Yes | Requires `--confirm` when `no_interactive=true` |
| `Status` | ❌ No | ❌ No | ❌ No | Read-only |
| `Agent` | ❌ No | ❌ No | ❌ No | Not yet implemented |
| `New` | ❌ No | ❌ No (uses $EDITOR) | ❌ No | Opens editor, no CLI prompts |
| `Stitch` | ❌ No | ❌ No | ❌ No | Not yet implemented |
| `InstallSystemd` | ❌ No | ❌ No | ❌ No | Pure file write |
| `Init` | ✅ Yes (rejects) | ✅ Yes | ❌ No | **EXPLICITLY REJECTS** `no_interactive` mode (init.rs:40-48) |
| `Restore` | ✅ Yes | ✅ Yes | ✅ Yes | Requires `--confirm` when `no_interactive=true` |

### Nested Subcommands

#### `Projects::*` (already covered in top-level, but defined separately)

| Command | Uses `no_interactive` | Interactive Prompts | Should Use Flag | Implementation |
|---------|----------------------|-------------------|----------------|----------------|
| `Projects::Add` | ✅ Yes | ❌ No | ⚠️ Questionable | Handled via main::handle_projects |
| `Projects::Scan` | ✅ Yes | ✅ Yes | ✅ Yes | `no_interactive \|\| yes` (line 434) |
| `Projects::List` | ❌ No | ❌ No | ❌ No | Read-only |
| `Projects::Show` | ❌ No | ❌ No | ❌ No | Read-only |
| `Projects::Remove` | ✅ Yes | ✅ Yes | ✅ Yes | Requires `--confirm` when `no_interactive=true` (projects.rs:449-459) |

#### `Audit::*`

| Command | Uses `no_interactive` | Interactive Prompts | Should Use Flag | Notes |
|---------|----------------------|-------------------|----------------|-------|
| `Audit::Check` | ❌ No | ❌ No | ❌ No | Read-only audit |
| `Audit::Verify` | ❌ No | ❌ No | ❌ No | Read-only verification |

#### `Migrate::*`

| Command | Uses `no_interactive` | Interactive Prompts | Should Use Flag | Current Pattern |
|---------|----------------------|-------------------|----------------|-----------------|
| `Migrate::Run` | ❌ No | ❌ No (requires `--confirm`) | ⚠️ Maybe | Requires `--confirm` flag (main.rs:564-570) |
| `Migrate::Status` | ❌ No | ❌ No | ❌ No | Read-only |
| `Migrate::MajorUpgrade` | ❌ No | ❌ No (requires `--confirm`) | ⚠️ Maybe | Requires `--confirm` flag (main.rs:620-626) |
| `Migrate::Rollback` | ❌ No | ❌ No (requires `--confirm`) | ⚠️ Maybe | Requires `--confirm` flag (main.rs:655-661) |
| `Migrate::RebuildPercentileIndex` | ❌ No | ❌ No | ❌ No | No prompts |

**Note:** All migrate commands use `--confirm` flag pattern instead of `no_interactive`. This is a DESTRUCTIVE OPERATIONS pattern that's consistent but different from the `no_interactive` approach.

#### `Backup::*`

| Command | Uses `no_interactive` | Interactive Prompts | Should Use Flag | Notes |
|---------|----------------------|-------------------|----------------|-------|
| `Backup::Trigger` | ❌ No | ❌ No | ❌ No | Direct API call |
| `Backup::Status` | ❌ No | ❌ No | ❌ No | Read-only |

#### `Script::*`

| Command | Uses `no_interactive` | Interactive Prompts | Should Use Flag | Notes |
|---------|----------------------|-------------------|----------------|-------|
| `Script::Run` | ❌ No | ❌ No | ❌ No | Direct execution |
| `Script::List` | ❌ No | ❌ No | ❌ No | Read-only |
| `Script::Show` | ❌ No | ❌ No | ❌ No | Read-only |

#### `Config::*`

| Command | Uses `no_interactive` | Interactive Prompts | Should Use Flag | Notes |
|---------|----------------------|-------------------|----------------|-------|
| `Config::Diff` | ❌ No | ❌ No | ❌ No | Read-only diff |
| `Config::Validate` | ❌ No | ❌ No | ❌ No | Read-only validation |

#### `RiskPatterns::*`

| Command | Uses `no_interactive` | Interactive Prompts | Should Use Flag | Notes |
|---------|----------------------|-------------------|----------------|-------|
| `RiskPatterns::Add` | ❌ No | ❌ No | ❌ No | Direct add |
| `RiskPatterns::List` | ❌ No | ❌ No | ❌ No | Read-only |
| `RiskPatterns::Seed` | ❌ No | ⚠️ Yes (overwrite check) | ⚠️ Maybe | Requires `--force` to overwrite existing (risk_patterns.rs:246-254) |

#### `Skills::*`

| Command | Uses `no_interactive` | Interactive Prompts | Should Use Flag | Notes |
|---------|----------------------|-------------------|----------------|-------|
| `Skills::Import` | ❌ No | ❌ No | ❌ No | Direct import to quarantine |
| `Skills::Enable` | ❌ No | ❌ No | ❌ No | Direct enable |
| `Skills::Disable` | ❌ No | ❌ No | ❌ No | Direct disable |
| `Skills::List` | ❌ No | ❌ No | ❌ No | Read-only |
| `Skills::Show` | ❌ No | ❌ No | ❌ No | Read-only |
| `Skills::Remove` | ❌ No | ❌ No | ❌ No | Direct removal |

#### `Pattern::*`

| Command | Uses `no_interactive` | Interactive Prompts | Should Use Flag | Notes |
|---------|----------------------|-------------------|----------------|-------|
| `Pattern::New` | ❌ No | ❌ No | ❌ No | Direct create via API |
| `Pattern::List` | ❌ No | ❌ No | ❌ No | Read-only |
| `Pattern::Show` | ❌ No | ❌ No | ❌ No | Read-only |
| `Pattern::Update` | ❌ No | ❌ No | ❌ No | Direct update via API |
| `Pattern::Close` | ❌ No | ❌ No | ❌ No | Direct update via API |
| `Pattern::Delete` | ❌ No | ✅ **Yes** | ✅ **Yes** | **MISSING**: Has confirmation prompt (patterns.rs:365-378) |
| `Pattern::AddMember` | ❌ No | ❌ No | ❌ No | Direct add via API |
| `Pattern::RemoveMember` | ❌ No | ❌ No | ❌ No | Direct remove via API |
| `Pattern::AddQuery` | ❌ No | ❌ No | ❌ No | Direct add via API |
| `Pattern::RemoveQuery` | ❌ No | ❌ No | ❌ No | Direct remove via API |

## Critical Findings

### ❌ Missing Implementation

**`Pattern::Delete`** (patterns.rs:365-378)
- **Has interactive confirmation prompt:** `print!("Confirm (yes/no): ")` then reads stdin
- **Does NOT accept `no_interactive` parameter**
- **Should:** Either accept `--confirm` flag when `no_interactive=true`, OR respect the global `no_interactive` flag

**Current code:**
```rust
PatternCommands::Delete { id, confirm, addr } => {
    if !confirm {
        println!("Are you sure you want to delete pattern '{}'?", id);
        println!("This will cascade to all members and queries.");
        print!("Confirm (yes/no): ");
        // ... reads stdin ...
        if input.trim() != "yes" {
            println!("Deletion cancelled");
            return Ok(());
        }
    }
    // ... proceeds with deletion ...
}
```

**Issue:** The `confirm` field exists but is not passed from the main handler. The `handle_patterns` function doesn't accept `no_interactive`.

### ⚠️ Inconsistencies

1. **Two patterns for non-interactive mode:**
   - `no_interactive` + `--confirm` (used by `Remove`, `Projects::Remove`, `Restore`)
   - `--confirm` alone (used by `Migrate::*` commands)
   
   **Recommendation:** Standardize on `no_interactive` + `--confirm` for destructive operations.

2. **`Init` command explicitly rejects `no_interactive`:**
   - This is correct behavior (init wizard requires interaction)
   - But it's the only command that does this

### ✅ Well-Implemented Commands

These commands properly handle `no_interactive` mode:

1. **`Projects::Scan`** - Uses `no_interactive || yes` to bypass prompts
2. **`Projects::Remove`** - Requires `--confirm` when `no_interactive=true` with clear error message
3. **`Remove` (top-level)** - Same as `Projects::Remove`
4. **`Restore`** - Requires `--confirm` when `no_interactive=true` with clear error message

## Recommendations

### High Priority

1. **Fix `Pattern::Delete`** to accept `no_interactive` parameter:
   - Modify `handle_patterns` signature to accept `no_interactive`
   - When `no_interactive=true` and `confirm=false`, error with clear message
   - Follow the pattern used by `Projects::Remove` and `Restore`

### Medium Priority

2. **Consider standardizing `Migrate::*` commands** to use `no_interactive` pattern:
   - These already require `--confirm` which is good
   - Could also check `no_interactive` for consistency
   - Update `handle_migrate` signature to accept `no_interactive`

3. **Document `RiskPatterns::Seed` force requirement:**
   - Current behavior is correct (requires `--force` to overwrite)
   - Could add `no_interactive` support for consistency

### Low Priority

4. **`Projects::Add` currently receives `no_interactive` but doesn't use it:**
   - No interactive prompts, so not needed
   - Could remove parameter from signature for clarity

## Code Locations Reference

- **Flag definition:** `hoop-cli/src/main.rs:34-35`
- **Flag extraction:** `hoop-cli/src/main.rs:258`
- **Init handler:** `hoop-cli/src/init.rs:40-48` (rejects `no_interactive`)
- **Projects commands:** `hoop-cli/src/projects.rs` (scan, remove)
- **Restore command:** `hoop-cli/src/restore.rs:279-356`
- **Pattern commands:** `hoop-cli/src/patterns.rs:365-378` (**missing implementation**)
- **Migrate handler:** `hoop-cli/src/main.rs:559-714`
- **RiskPatterns seed:** `hoop-cli/src/risk_patterns.rs:246-254`

## Testing Checklist

For each command that should support `no_interactive`:

- [ ] Verify command works in interactive mode (prompts user)
- [ ] Verify command works with `--no-interactive` (auto-proceeds)
- [ ] Verify command with `--no-interactive` requires `--confirm` for destructive ops
- [ ] Verify clear error message when `--no-interactive` without `--confirm` on destructive ops

**Current state:**
- ✅ `Projects::Scan`
- ✅ `Projects::Remove`
- ✅ `Remove` (top-level)
- ✅ `Restore`
- ❌ `Pattern::Delete` (not implemented)
