# Bead bf-4g83z: Remove unused utoipa imports from hoop-daemon API modules (second half)

## Status: Already Complete

This bead's work was completed in commit `702df8c` titled "refactor: remove unused utoipa::ToSchema from request-only structs".

## What Was Done

The commit removed `#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]` from request-only structs (structs that only implement `Deserialize` but not `Serialize`). ToSchema is only needed for response types that implement Serialize.

### Files Modified (8 of 10)

1. `api_transcription.rs` - Removed from `ListJobsQuery` (query params)
2. `api_uploads.rs` - Removed from `InitUploadRequest` (request body)
3. `api_reflection_ledger.rs` - Removed from `ApproveProposalRequest`, `RejectProposalRequest` (request bodies)
4. `api_screen_capture.rs` - Removed from `CreateScreenCaptureRequest`, `StartStreamingUploadRequest`, `CompleteStreamingUploadRequest` (request bodies)
5. `api_scripts.rs` - Removed from `ScriptRunRequest` (request body)
6. `api_skills.rs` - Removed from `SkillRunRequest` (request body)
7. `api_tour_project.rs` - Removed from `EnableTourRequest` (request body)
8. `api_unassigned.rs` - Removed from `AssignRequest` (request body)

### Files Not Modified (2 of 10)

9. `api_stitch_traversal.rs` - No unused ToSchema derives (all derives are on response structs)
10. `api_timeline.rs` - No unused ToSchema derives (all derives are on response structs)

Both remaining files only have ToSchema derives on response structs that implement `Serialize`, which is correct.

## Why No Import Statements

The bead description mentioned "unused utoipa::ToSchema imports," but there were never any `use utoipa::ToSchema;` import statements in these files. All files used the full path `#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]` directly in the attribute. The "imports" in the manifest referred to these derive attributes, not import statements.

## Acceptance Criteria Met

- ✅ All unused ToSchema derives removed from the 8 affected files
- ✅ Each file still compiles (commit was tested)
- ✅ Only ToSchema removed from request-only structs; response structs kept their derives
