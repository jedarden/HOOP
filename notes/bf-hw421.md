# bf-hw421: ToSchema derive already present on ApproveProposalRequest

## Task
Add `#[derive(ToSchema)]` to the `ApproveProposalRequest` struct in `api_reflection_ledger.rs`.

## Finding
The `ToSchema` derive is **already present** on `ApproveProposalRequest`.

At line 41-45 of `hoop-daemon/src/api_reflection_ledger.rs`:

```rust
/// Request to approve a proposal
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ApproveProposalRequest {
    /// Optional comment on the approval
    pub comment: Option<String>,
}
```

The struct already has `utoipa::ToSchema` in its derive attribute.

## Note
This is the sixth attempt to document this finding (see git commits `1155149`, `aa1c625`, `9280783`, `c894344`, `34a9b39`). The derive was already present when the bead was created.

## Recommendation
The bead should be closed with no code changes needed. The `ToSchema` derive has been present since the initial implementation.
