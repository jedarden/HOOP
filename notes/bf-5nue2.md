# bf-5nue2: Add ToSchema derive to SiblingProject

## Verification

The `SiblingProject` struct in `src/cross_project_propagation.rs` already has `#[derive(utoipa::ToSchema)]` on line 22.

### Field type verification

All field types support ToSchema:
- `project: String` - primitive type supported by utoipa
- `matches: Vec<SiblingStitch>` - `SiblingStitch` has ToSchema derive (line 35)
- `similarity: f64` - primitive type supported by utoipa
- `evidence: SiblingEvidence` - `SiblingEvidence` has ToSchema derive (line 52)

### Compilation

Verified with `cargo check` - compiles successfully with only warnings (no errors).
