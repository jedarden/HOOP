# Bead bf-31xee: Unused utoipa imports already removed

## Task
Remove unused `utoipa::ToSchema` imports from:
- `hoop-daemon/src/api_reflection_ledger.rs:21`
- `hoop-daemon/src/api_scripts.rs:29`
- `hoop-daemon/src/api_screen_capture.rs:28`

## Status: Already Complete

The unused imports were already removed in commit `0e24592 refactor: remove unused utoipa::ToSchema imports from core modules` (2026-06-28).

## Verification

All three files were examined:
1. `api_reflection_ledger.rs` - No direct `use utoipa::ToSchema` import present
2. `api_scripts.rs` - No direct `use utoipa::ToSchema` import present  
3. `api_screen_capture.rs` - No direct `use utoipa::ToSchema` import present

All files compile successfully (verified with `cargo check --package hoop-daemon`).

The structs in these files still use `#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]` where appropriate for OpenAPI schema generation, but no standalone unused imports exist.
