# ScriptRunRequest Struct Documentation

**Bead:** bf-20qqe  
**File:** hoop-daemon/src/api_scripts.rs:162-169

## Struct Definition

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

## Analysis

### Current Derives
- `Debug` - enables debug formatting
- `Serialize` - enables serialization (via serde)
- `Deserialize` - enables deserialization (via serde)

### cfg_attr for ToSchema
- `#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]`
- Conditionally derives `utoipa::ToSchema` only when the `openapi` feature is enabled
- This allows OpenAPI schema generation without requiring the dependency in all builds

### Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `args` | `Vec<String>` | `Vec::new()` (via `#[serde(default)]`) | Arguments to pass to the script |
| `project` | `Option<String>` | `None` | Optional project context for project-scoped scripts |

## Purpose

`ScriptRunRequest` is the request body for script execution endpoints. It accepts:
- An array of command-line arguments to pass to the script
- An optional project identifier for scripts that operate within a project context
