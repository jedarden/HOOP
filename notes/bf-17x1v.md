# Task bf-17x1v: Remove unused utoipa::ToSchema import from api_unassigned.rs

## Finding
The unused `utoipa::ToSchema` import was **already removed** from `hoop-daemon/src/api_unassigned.rs`.

## Evidence
1. No `use utoipa::ToSchema;` import exists in the file
2. Line 37 is `const MAX_UNASSIGNED_SESSIONS: usize = 100;` (not an import)
3. The file uses `utoipa::ToSchema` only in derive attributes with full paths:
   - `#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]`
4. `cargo check` passes with no errors or warnings for `api_unassigned.rs`

## Conclusion
The task was already completed in a previous change. No action needed.
