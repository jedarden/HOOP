# ScriptRunRequest ToSchema Field Type Analysis

## Task Summary
Analyzed whether `ScriptRunRequest` field types need explicit `ToSchema` derives.

## ScriptRunRequest Definition

```rust
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

## Field Type Analysis

### 1. `Vec<String>` field

**Type**: `Vec<String>`

**ToSchema Support**: ✅ Automatic - No derive needed

**Rationale**: 
- `String` implements `ToSchema` via utoipa's primitive type implementations
- `Vec<T>` has a blanket implementation: `impl<T: ToSchema> ToSchema for Vec<T>`
- Therefore, `Vec<String>` automatically implements `ToSchema` through the blanket implementation

**OpenAPI Schema Representation**:
```yaml
args:
  type: array
  items:
    type: string
  default: []
```

### 2. `Option<String>` field

**Type**: `Option<String>`

**ToSchema Support**: ✅ Automatic - No derive needed

**Rationale**:
- `String` implements `ToSchema` via utoipa's primitive type implementations  
- `Option<T>` has a blanket implementation: `impl<T: ToSchema> ToSchema for Option<T>`
- Therefore, `Option<String>` automatically implements `ToSchema` through the blanket implementation

**OpenAPI Schema Representation**:
```yaml
project:
  type: string
  nullable: true
```

## Potential Issues

**None identified**. Both field types are standard Rust library types with utoipa blanket implementations.

The only requirement is that the inner type (`String`) implements `ToSchema`, which it does via utoipa's primitive type support.

## Usage Verification

The struct is successfully registered in the OpenAPI schema registry in `openapi.rs` at line 453:

```rust
components(schemas(
    // ...
    crate::api_scripts::ScriptRunRequest,
    // ...
))
```

This confirms that the ToSchema derivation works correctly for `ScriptRunRequest` and its field types.

## Conclusion

Both `Vec<String>` and `Option<String>` field types in `ScriptRunRequest` support `ToSchema` automatically through utoipa's blanket implementations. No additional derives or custom implementations are needed.
