# utoipa ToSchema Usage in api_scripts.rs

## Findings

### No Explicit Import
The file does NOT contain an explicit `use utoipa::ToSchema;` import. The import section (lines 7-26) includes:
- axum
- notify
- serde
- std (fs, io, os, path, process, sync, time)
- sha2
- tracing
- crate::DaemonState and crate::fleet

But no `utoipa` import.

### ToSchema Applied via cfg_attr
ToSchema is applied using `#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]` on the following structs:
1. `ScriptManifest` (line 32-33)
2. `OverlapPolicy` (line 64-66)
3. `ScriptScope` (line 89-91)
4. `ScriptArgument` (line 100-101)
5. `EventSubscription` (line 117-118)
6. `ScriptEntry` (line 137-138)
7. `ScriptRunRequest` (line 161-162)
8. `ScriptRunResponse` (line 171-173)

### Feature-Gated Behind 'openapi'
The derive is conditionally compiled with `feature = "openapi"`. This means:
- When the "openapi" feature is enabled in Cargo.toml, `utoipa::ToSchema` is derived
- When disabled, the derive macro is not applied at all

### Full Path Usage
The derive uses the full path `utoipa::ToSchema` rather than just `ToSchema`. This is why no import is needed - the full path resolves to the trait in the utoipa crate directly in the attribute.

## Notes
This is the standard pattern for feature-gated derives in Rust. Using the full path (`utoipa::ToSchema`) in the cfg_attr eliminates the need for a separate import statement, which keeps the imports section cleaner and makes it clear that utoipa is only used when the openapi feature is enabled.
