# Bead bf-5yhfp: ToSchema Already Present

## Finding

The `ScriptRunRequest` struct in `hoop-daemon/src/api_scripts.rs:162` **already has** the `#[derive(ToSchema)]` attribute applied.

## Current State

```rust
/// Script execution request
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ScriptRunRequest {
    /// Arguments to pass to the script
    #[serde(default)]
    pub args: Vec<String>,
    /// Optional project context (for project-scoped scripts)
    pub project: Option<String>,
}
```

## History

The ToSchema derive was added/updated in commit `60c9f01` on 2026-07-09:
- Title: "fix: Add ToSchema trait import to api_scripts.rs"
- This commit added the proper `use utoipa::ToSchema;` import
- Updated all derives to use the imported trait name
- The `ScriptRunRequest` struct has had the derive since at least that commit

## Verification

- `cargo check --package hoop-daemon` passes with zero errors
- The derive pattern `#[cfg_attr(feature = "openapi", derive(ToSchema))]` is consistent with all other structs in the file
- No changes are needed

## Conclusion

The bead acceptance criteria are already met:
- ✅ ToSchema derive is present (line 164)
- ✅ Existing derives (Debug, Serialize, Deserialize) are preserved
- ✅ Proper formatting is in place
