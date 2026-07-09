# ScriptRunRequest ToSchema Investigation Summary

**Bead ID:** bf-48l13  
**Date:** 2026-07-09  
**Investigation Scope:** ScriptRunRequest struct ToSchema compilation and field type compatibility

## Overview

This document compiles comprehensive findings from the investigation of `ScriptRunRequest` and its `utoipa::ToSchema` derive implementation. The investigation identified and resolved a critical missing derive that prevented successful OpenAPI schema generation.

## Struct Definition Details

### Location
**File:** `hoop-daemon/src/api_scripts.rs:162-169`

### Current Definition
```rust
/// Script execution request
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ScriptRunRequest {
    /// Arguments to pass to the script
    #[serde(default)]
    pub args: Vec<String>,
    /// Optional project context (for project-scoped scripts)
    pub project: Option<String>,
}
```

### Field Breakdown

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `args` | `Vec<String>` | `Vec::new()` (via `#[serde(default)]`) | Command-line arguments to pass to the script |
| `project` | `Option<String>` | `None` | Optional project identifier for project-scoped scripts |

### Purpose
Request body for script execution endpoints. Accepts command-line arguments and optional project context for scripts that operate within a project scope.

## Import Analysis Results

### No Explicit utoipa Import Found
The file does **NOT** contain an explicit `use utoipa::ToSchema;` import. The import section includes:
- axum
- notify  
- serde (Serialize, Deserialize)
- std library components
- sha2
- tracing
- crate-specific imports

### Full Path Usage Pattern
`ToSchema` is applied using the **full path** `utoipa::ToSchema` in the derive attribute:
```rust
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
```

This eliminates the need for a separate import statement and makes it explicit that utoipa is only used when the openapi feature is enabled.

### Feature-Gated Compilation
The derive is conditionally compiled with `feature = "openapi"`:
- When enabled: `utoipa::ToSchema` is derived and OpenAPI schema is generated
- When disabled: The derive macro is not applied at all

### Other Structs with Same Pattern
This pattern is used consistently across 8 structs in api_scripts.rs:
1. ScriptManifest
2. OverlapPolicy
3. ScriptScope
4. ScriptArgument
5. EventSubscription
6. ScriptEntry
7. **ScriptRunRequest** (line 161-162)
8. ScriptRunResponse

## Field Type Compatibility Assessment

### `Vec<String>` Analysis

**Type:** `Vec<String>`  
**ToSchema Support:** ✅ **Automatic - No derive needed**

**Rationale:**
- `String` implements `ToSchema` via utoipa's primitive type implementations
- `Vec<T>` has blanket implementation: `impl<T: ToSchema> ToSchema for Vec<T>`
- Therefore, `Vec<String>` automatically implements `ToSchema` through the blanket implementation

**OpenAPI Schema Representation:**
```yaml
args:
  type: array
  items:
    type: string
  default: []
```

### `Option<String>` Analysis

**Type:** `Option<String>`  
**ToSchema Support:** ✅ **Automatic - No derive needed**

**Rationale:**
- `String` implements `ToSchema` via utoipa's primitive type implementations
- `Option<T>` has blanket implementation: `impl<T: ToSchema> ToSchema for Option<T>`
- Therefore, `Option<String>` automatically implements `ToSchema` through the blanket implementation

**OpenAPI Schema Representation:**
```yaml
project:
  type: string
  nullable: true
```

### Field Type Compatibility Conclusion

**Both field types support ToSchema automatically** through utoipa's blanket implementations. No additional derives or custom implementations are needed for the field types themselves.

The only requirement is that the inner type (`String`) implements `ToSchema`, which it does via utoipa's primitive type support.

## Issues Found and Resolution

### Root Cause: Missing Serialize Derive ⚠️

**Issue:** The struct was missing the `Serialize` derive, which is required by `utoipa::ToSchema`.

**Before Fix:**
```rust
#[derive(Debug, Deserialize)]
pub struct ScriptRunRequest { ... }
```

**After Fix (commit cd9d354):**
```rust
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ScriptRunRequest { ... }
```

**Why This Matters:**
`utoipa::ToSchema` requires **both** `Serialize` AND `Deserialize` traits to be present. The struct only had `Deserialize`, causing compilation errors when ToSchema was added.

### Resolution Details

**Fixed in:** commit `cd9d354` - "fix(api_scripts): Add missing Serialize derive to ScriptRunRequest"

**Impact:** The missing `Serialize` derive prevented successful OpenAPI schema generation. After adding the derive, the struct compiles successfully and generates proper OpenAPI documentation.

## Verification and Registration

### OpenAPI Schema Registration
The struct is successfully registered in the OpenAPI schema registry in `openapi.rs` at line 453:

```rust
components(schemas(
    // ...
    crate::api_scripts::ScriptRunRequest,
    // ...
))
```

This confirms that the ToSchema derivation works correctly for `ScriptRunRequest` and its field types.

### Current Compilation Status
✅ **Clean build verified** - `cargo check -p hoop-daemon` exits with no E0277 errors related to ToSchema derives

### Related Investigation Work
The investigation built on prior beads:
- **bf-20qqe**: Initial struct definition documentation
- **bf-3wrny**: utoipa ToSchema usage pattern documentation  
- **bf-6376j**: Root cause identification (missing Serialize derive)
- **bf-14ngz**: Field type compatibility analysis
- **bf-2e233**: Verification of ToSchema derives across 12 structs

## Lessons Learned

### Pattern for Future ToSchema Issues

When adding `#[derive(utoipa::ToSchema)]` to a struct, ensure the struct has **both**:

1. `#[derive(Serialize)]`  
2. `#[derive(Deserialize)]`

Or combined: `#[derive(Serialize, Deserialize)]`

**Without both derives, ToSchema will fail to compile.**

### Field Type Checklist

For standard Rust library types used as struct fields:
- ✅ `String` - Automatic ToSchema support
- ✅ `Vec<T>` - Automatic when T: ToSchema  
- ✅ `Option<T>` - Automatic when T: ToSchema
- ✅ `HashMap<K, V>` - Automatic when K: ToSchema and V: ToSchema

Custom types require explicit ToSchema derives or implementations.

### Feature-Gated Derives Pattern

Using `#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]` with full path `utoipa::ToSchema`:
- Eliminates need for explicit imports
- Makes dependency on openapi feature explicit
- Keeps imports section cleaner
- Standard pattern across the codebase

## Conclusion

The ScriptRunRequest ToSchema investigation successfully identified and resolved a critical missing derive issue. The struct now properly derives both `Serialize` and `Deserialize`, enabling successful `utoipa::ToSchema` derivation and OpenAPI schema generation. Both field types (`Vec<String>` and `Option<String>`) support ToSchema automatically through utoipa's blanket implementations, requiring no additional custom code.

**Status:** ✅ **RESOLVED** - All ToSchema derives working correctly, clean compilation verified.
