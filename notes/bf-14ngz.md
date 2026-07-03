# Analysis: ScriptRunRequest Field Types for ToSchema

**Bead ID:** bf-14ngz  
**Date:** 2026-07-03

## Task Summary

Analyze whether `ScriptRunRequest` field types need `ToSchema` derives themselves or support utoipa automatically.

## Field Types Analyzed

The `ScriptRunRequest` struct (hoop-daemon/src/api_scripts.rs:160-169) contains:

```rust
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ScriptRunRequest {
    #[serde(default)]
    pub args: Vec<String>,
    pub project: Option<String>,
}
```

### Field 1: `Vec<String>`

**Analysis:**
- `Vec<T>` implements `ToSchema` automatically when `T: ToSchema`
- `String` is a primitive type that utoipa recognizes and handles automatically
- No additional derive needed on `Vec` or `String`

**Conclusion:** ✅ Works automatically - no derives needed

### Field 2: `Option<String>`

**Analysis:**
- `Option<T>` implements `ToSchema` automatically when `T: ToSchema`
- `String` is a primitive type known to utoipa
- In OpenAPI spec, `Option<T>` renders as a nullable field

**Conclusion:** ✅ Works automatically - no derives needed

## Evidence from Codebase

Multiple structs in HOOP use these patterns successfully with `ToSchema`:

**Example from `api_beads.rs` (lines 43-66):**
```rust
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateBeadRequest {
    pub dependencies: Option<Vec<String>>,  // Works ✓
    pub labels: Option<Vec<String>>,         // Works ✓
    pub stitch_id: Option<String>,           // Works ✓
    pub parent_bead_id: Option<String>,      // Works ✓
}
```

**Example from `api_scripts.rs` (lines 32-61):**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ScriptManifest {
    pub projects: Vec<String>,     // Works ✓
    pub schedule: Option<String>,  // Works ✓
}
```

## How utoipa Handles These Types

1. **Primitive types**: `String`, `i32`, `i64`, `bool`, etc. are built-in and work automatically
2. **`Vec<T>`**: Generic container that implements `ToSchema` for any `T: ToSchema`
3. **`Option<T>`**: Generic container that implements `ToSchema` for any `T: ToSchema` (renders as nullable in OpenAPI)
4. **`HashMap<K,V>`**: Works automatically when both `K` and `V` implement `ToSchema`

## Potential Issues

**No issues identified** for these specific types. The only cases where manual derives would be needed are:

1. **Custom structs/enums**: Must have `#[derive(ToSchema)]` if used as a field type
2. **Complex generics**: Nested types like `Result<Vec<Option<T>>, E>` where the inner type is custom
3. **Third-party types**: Types from external crates that don't already implement `ToSchema`

## Recommendations

✅ **No action needed** - The current `ScriptRunRequest` implementation is correct:
- Only the struct itself needs `#[derive(utoipa::ToSchema)]`
- Field types `Vec<String>` and `Option<String>` work automatically
- This is the idiomatic pattern used throughout the HOOP codebase

## References

- utoipa documentation: https://docs.rs/utoipa/
- Similar patterns in: `api_beads.rs`, `api_draft_queue.rs`, `api_files.rs`
