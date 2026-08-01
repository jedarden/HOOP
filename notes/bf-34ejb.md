# Bead bf-34ejb: Document and verify stderr output for prompts

## Task
Ensure all interactive prompts in projects commands go to stderr, not stdout.

## Verification Summary

### remove_project (lines 455-528 in hoop-cli/src/projects.rs)

✅ **All prompts correctly use stderr:**
- Line 473: `eprintln!("Removing project '{}'", name);` - project info
- Line 475: `eprintln!("  Workspace: {} ({})", ws.path.display(), ws.role);` - workspace list
- Line 478: `eprint!("Confirm removal? [y/N] ");` - confirmation prompt
- Line 479: `std::io::stderr().flush()?;` - explicit flush
- Line 486: `eprintln!("Removal cancelled");` - cancellation message

✅ **Documentation:**
- Lines 449-451: Function doc explicitly states "All prompts go to stderr"
- Line 470: Inline comment added: "All prompts go to stderr (not stdout) to avoid interfering with data output"

### scan_projects (lines 589-753 in hoop-cli/src/projects.rs)

✅ **All prompts correctly use stderr:**
- Line 664: `eprintln!("    Failed to register {}: {}", path.display(), e);` - error messages
- Line 671: `eprint!("  {} — register? [y/N] ", default_name);` - registration prompt
- Line 672: `std::io::stderr().flush()?;` - explicit flush
- Line 683: `eprint!("    name [{}]: ", default_name);` - rename prompt
- Line 684: `std::io::stderr().flush()?;` - explicit flush
- Line 724: `eprintln!("    Failed to register {}: {}", path.display(), e);` - error messages

✅ **Documentation:**
- Lines 581-587: Function doc updated to mention "All interactive prompts go to stderr; registration results and errors go to stdout."
- Line 670: Inline comment updated: "All interactive prompts go to stderr (not stdout) to avoid interfering with data output"
- Line 682: Inline comment updated: "Offer rename (prompt also goes to stderr)"

### Data Output (uses stdout, as expected)

Both functions correctly use `println!` for data output:
- `scan_projects`: Lines 602-618, 627, 633, 636, 696, 734-750 - registration results and summaries
- `remove_project`: No direct stdout output (returns bool via Result)

## Acceptance Criteria Met

✅ All interactive prompts output to stderr (verified with code inspection)
✅ Behavior is documented in code comments (function docs + inline comments)
✅ No prompts leak to stdout (clean separation: prompts to stderr, data to stdout)
✅ Verification shows proper `eprint!/eprintln!` usage with explicit `std::io::stderr().flush()`

## Changes Made

1. Updated `scan_projects` function documentation (lines 581-587)
2. Added inline comment at remove_project prompt site (line 470)
3. Updated inline comments at scan_projects prompt sites (lines 670, 682)

## Verification Method

- Code inspection of all `eprint!/eprintln!` calls in both functions
- Verification that `std::io::stderr().flush()` is called after each `eprint!` prompt
- Confirmed no `print!` calls are used for interactive prompts
- Checked that `println!` is only used for data output (registration results, summaries)

## Conclusion

Both `remove_project` and `scan_projects` correctly direct all interactive prompts to stderr, ensuring clean separation from data output. This behavior is now documented in both function-level doc comments and inline comments at each prompt site.
