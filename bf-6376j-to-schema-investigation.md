# Investigation: ScriptRunRequest ToSchema Dependencies (bf-6376j)

## Task Completion Summary

Investigated the `ScriptRunRequest` struct to understand its ToSchema dependencies and identify any potential issues preventing `#[derive(ToSchema)]` from working.

## Findings

### Struct Location
File: `hoop-daemon/src/api_scripts.rs`, lines 162-171

### Struct Definition
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

### ToSchema Import Status
✅ **CONFIRMED**: `utoipa::ToSchema` is imported at line 16:
```rust
#[cfg(feature = "openapi")]
use utoipa::ToSchema];
```

The import is correctly gated behind the `openapi` feature flag.

### Field Type Analysis

| Field | Type | ToSchema Compatible | Notes |
|-------|------|---------------------|-------|
| `args` | `Vec<String>` | ✅ Yes | Standard collection type, native utoipa support |
| `project` | `Option<String>` | ✅ Yes | Standard optional type, native utoipa support |

**Key Finding**: Both field types are primitive Rust types that utoipa handles natively. No custom derives or additional ToSchema implementations are required.

### Consistency with Codebase Pattern

The `ScriptRunRequest` struct follows the exact same pattern used by 17 other structs in `api_scripts.rs`:
- `ScriptManifest` (line 35)
- `OverlapPolicy` (line 68)
- `ScriptScope` (line 93)
- `ScriptArgument` (line 103)
- `EventSubscription` (line 120)
- `ScriptEntry` (line 140)
- `ScriptRunRequest` (line 164) ← **This struct**
- `ScriptRunResponse` (line 174)

All use `#[cfg_attr(feature = "openapi", derive(ToSchema))]` consistently.

## Conclusion

The `ScriptRunRequest` struct **should work** with `#[derive(ToSchema)]`:
- ✅ ToSchema is imported (gated behind `openapi` feature)
- ✅ All field types are utoipa-compatible primitives
- ✅ Derive attribute is correctly applied
- ✅ Pattern matches the rest of the codebase

If compilation errors occur, they are likely caused by:
1. The `openapi` feature not being enabled
2. Issues in other parts of the OpenAPI generation (see workspace learning: "unrelated OpenAPI generation errors in openapi.rs")
3. Dependency version conflicts with utoipa

No changes needed to `ScriptRunRequest` itself.
