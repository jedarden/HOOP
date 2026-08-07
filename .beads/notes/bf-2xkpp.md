# Bead bf-2xkpp: Filter clippy output for unused utoipa::ToSchema warnings

## Summary
Filtered clippy raw output (`.beads/clippy-raw-output.txt`) for unused utoipa::ToSchema import warnings.

## Result
**No unused utoipa::ToSchema warnings found** in the clippy output.

The clippy run contained 88 errors/warnings covering:
- Compilation errors (missing modules, type mismatches, missing fields)
- Unused imports (AtomicBool, Ordering, tag_join, PathBuf, etc.)
- Dead code (unused functions, constants, fields)
- Disallowed methods (std::fs::write, File::create)
- Style lints (manual_clamp, unnecessary_sort_by, etc.)
- Type complexity warnings

However, none of these were related to unused utoipa::ToSchema imports specifically.

## Output
Created `.beads/clippy-utoipa-filtered.txt` documenting that no such warnings were found.

## Next step
This filtered result feeds into the final bead which will sort and save to the reference file.
