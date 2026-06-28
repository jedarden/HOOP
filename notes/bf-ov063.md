# Task bf-ov063: Remove unused utoipa imports (first half)

## Investigation

The manifest `.claude/utoipa-unused-imports.txt` listed 12 API modules with supposedly unused `utoipa::ToSchema` imports at specific line numbers. However, upon investigation:

1. **None of the 12 target files had standalone `use utoipa::ToSchema;` imports**
2. The files use `#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]` attributes directly on structs, which is the correct pattern - no separate import needed
3. Only ONE file had an actual unused utoipa import: `hoop-daemon/src/api_beads.rs:28` with `use utoipa::path;`

## Action Taken

Removed the unused `use utoipa::path;` import from `hoop-daemon/src/api_beads.rs` line 28.

## Verification

- `cargo check` passes (exit code 0)
- `cargo clippy` shows NO unused utoipa imports in the 12 target files
- Remaining clippy warnings in target files are for other unused imports (State, Connection, params, etc.), not utoipa-related

## Conclusion

The manifest appears to have been generated from an outdated clippy run where line numbers had shifted. The actual unused utoipa import was `utoipa::path` in api_beads.rs, which has now been removed.
