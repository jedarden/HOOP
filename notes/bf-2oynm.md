# Task bf-2oynm: Add ToSchema derive to RejectProposalRequest

## Task Description
Add `#[derive(ToSchema)]` to the `RejectProposalRequest` struct in `api_reflection_ledger.rs`.

## Findings
The `RejectProposalRequest` struct already has the required `#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]` attribute (lines 58-64). This is the correct pattern used consistently throughout the file for all request/response types.

```rust
/// Request to reject a proposal
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RejectProposalRequest {
    /// Optional reason for rejection
    pub reason: Option<String>,
}
```

## Verification
- ✅ Struct has `#[derive(ToSchema)]` via cfg_attr
- ✅ Field type `Option<String>` is fully compatible with utoipa's ToSchema
- ✅ `cargo check` passes (no openapi feature)
- ✅ `cargo check --features openapi` passes (with openapi feature)

## Conclusion
The task is already complete. The struct uses the proper conditional derive pattern that matches all other API types in the file.
