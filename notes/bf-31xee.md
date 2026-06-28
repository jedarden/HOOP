# bead bf-31xee: Unused utoipa import cleanup verification

## Task
Remove unused `utoipa::ToSchema` imports from:
- hoop-daemon/src/api_reflection_ledger.rs:21
- hoop-daemon/src/api_scripts.rs:29
- hoop-daemon/src/api_screen_capture.rs:28

## Finding
**Task already complete.** These 3 files do not have `use utoipa::ToSchema;` import statements.

## Investigation
All 3 files correctly use the full path in their derive attributes:
```rust
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
```

This pattern works without a separate `use utoipa::ToSchema;` import statement. The derive attribute uses the fully-qualified path `utoipa::ToSchema`, which resolves correctly without an import.

## Verification
- `cargo check --package hoop-daemon` compiles successfully
- No `use utoipa::ToSchema;` imports found in any of the 3 target files
- The files use `#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]` which is the correct pattern

## Likely cleanup date
Based on git history, similar cleanup was done in:
- Commit `0e24592` - removed unused imports from core modules
- Commit `16086f9` - removed unused ToSchema derives from request-only structs

These 3 files were likely cleaned up in or before those commits.
