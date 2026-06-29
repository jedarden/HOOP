# Task bf-hw421: Add ToSchema derive to ApproveProposalRequest

## Finding
The `ToSchema` derive macro was **already present** on `ApproveProposalRequest` at the time this task was claimed.

## Verification
```bash
$ grep -B1 "pub struct ApproveProposalRequest" hoop-daemon/src/api_reflection_ledger.rs
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ApproveProposalRequest {
```

## Acceptance Criteria Status
- ✅ ApproveProposalRequest has `#[derive(ToSchema)]` present (as `utoipa::ToSchema`)
- ✅ The derive macro is properly formatted

## Conclusion
Task is complete. No file changes were needed.
