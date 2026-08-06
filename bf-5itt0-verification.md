# ToSchema Derive Verification (bf-5itt0)

## Date: 2026-08-06

## Task
Verify all ToSchema derives with cargo check

## Results

### All Three Structs Have ToSchema Derive ✓

1. **`EnableTourRequest`** (hoop-daemon/src/api_tour_project.rs:35)
   ```rust
   #[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
   pub struct EnableTourRequest {
   ```

2. **`ScriptRunRequest`** (hoop-daemon/src/api_scripts.rs:165)
   ```rust
   #[cfg_attr(feature = "openapi", derive(ToSchema))]
   pub struct ScriptRunRequest {
   ```

3. **`ListJobsQuery`** (hoop-daemon/src/api_transcription.rs:20)
   ```rust
   #[derive(Debug, Deserialize, ToSchema)]
   pub struct ListJobsQuery {
   ```

### cargo check Results ✓
- **Status**: Completed successfully (exit 0)
- **ToSchema errors**: None
- **Total warnings**: 25 (14 hoop-daemon, 9 hoop-cli)
- **New compilation errors**: 0

## Conclusion
All acceptance criteria met:
- ✓ All three structs have #[derive(ToSchema)]
- ✓ cargo check completes without ToSchema-related errors
- ✓ No new compilation errors introduced

The ToSchema derives are properly configured and working correctly.
