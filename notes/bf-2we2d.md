# Bead bf-2we2d: Add ToSchema derive to SiblingProject struct

## Task
Add `#[derive(ToSchema)]` to the `SiblingProject` struct in `cross_project_propagation.rs`.

## Status: Already Complete

The `SiblingProject` struct already has `#[derive(utoipa::ToSchema)]` on line 22 of `hoop-daemon/src/cross_project_propagation.rs`.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SiblingProject {
    /// Project name
    pub project: String,
    /// Matching Stitches in this project
    pub matches: Vec<SiblingStitch>,
    /// Similarity score (0-1)
    pub similarity: f64,
    /// Evidence for why this project is a sibling
    pub evidence: SiblingEvidence,
}
```

All field types are ToSchema-compatible:
- `String` - primitive
- `f64` - primitive
- `Vec<SiblingStitch>` - SiblingStitch has ToSchema
- `SiblingEvidence` - has ToSchema

## Verification

Ran `cargo check` and confirmed no errors in `cross_project_propagation.rs`. The compilation errors in the project are unrelated to this file (they concern other structs missing ToSchema derives: `ScriptRunRequest`, `EnableTourRequest`, `ListJobsQuery`, `CreateScreenCaptureRequest`).

## Conclusion

The bead's acceptance criteria are already met:
- ✅ SiblingProject has `#[derive(ToSchema)]`
- ✅ All field types are compatible with ToSchema
- ✅ File compiles with `cargo check` (no errors in this file)
