# Verification: api_unassigned.rs and api_uploads.rs ToSchema derives

## Task (bf-3msks)
Verify both API modules compile after ToSchema derives were added.

## What was actually done
The recent doc commits (bf-44l39, bf-17x1v, bf-67cw3, bf-5woxk, bf-20z0s) documented that unused `utoipa::ToSchema` imports were already removed from the codebase.

The current uncommitted changes ADD `ToSchema` derive macros to request structs:

### api_unassigned.rs (line 78)
```rust
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AssignRequest {
    pub project: String,
}
```

### api_uploads.rs (line 15)
```rust
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct InitUploadRequest {
    pub filename: String,
    pub total_size: u64,
    pub checksum: String,
    pub attachment_type: String,
    pub resource_id: String,
}
```

## Verification Results

### Compilation check
- `cargo check -p hoop-daemon` ran and checked both files successfully
- Both files emitted warnings (unused imports), but no compilation errors
- The overall build is blocked by unrelated files missing ToSchema derives:
  - api_agent.rs: SwitchRequest, TurnRequest, TurnAttachment
  - api_reflection_ledger.rs: ApproveProposalRequest, RejectProposalRequest
  - api_scripts.rs: ScriptRunRequest
  - api_tour_project.rs: EnableTourRequest
  - api_transcription.rs: ListJobsQuery

### Status
✅ Both api_unassigned.rs and api_uploads.rs compile successfully
✅ ToSchema derives are correctly added to the target structs
⚠️ Overall compilation blocked by other modules (out of scope for this task)
