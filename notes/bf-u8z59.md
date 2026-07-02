# HOOP Debug Build Output

**Bead:** bf-u8z59  
**Task:** Gather HOOP debug build output  
**Date:** 2026-07-02  
**Build Command:** `cargo build`

## Summary

The debug build fails with **22 compilation errors** and **74 warnings**.

## Errors (22 total)

All errors are related to missing `ToSchema` trait implementations for OpenAPI documentation:

1. **`ScriptRunRequest`** (hoop-daemon/src/api_scripts.rs:162) - Missing `ToSchema` and `PartialSchema` (2 occurrences)
2. **`EnableTourRequest`** (hoop-daemon/src/api_tour_project.rs:34) - Missing `ToSchema` and `PartialSchema` (4 occurrences)
3. **`ListJobsQuery`** (hoop-daemon/src/api_transcription.rs:19) - Missing `ToSchema` and `PartialSchema` (2 occurrences)
4. **`CreateScreenCaptureRequest`** (hoop-daemon/src/api_screen_capture.rs:34) - Missing `ToSchema` and `PartialSchema` (2 occurrences)
5. **`StartStreamingUploadRequest`** (hoop-daemon/src/api_screen_capture.rs:352) - Missing `ToSchema` and `PartialSchema` (2 occurrences)
6. **`CompleteStreamingUploadRequest`** (hoop-daemon/src/api_screen_capture.rs:469) - Missing `ToSchema` and `PartialSchema` (2 occurrences)

**Error Locations:**
- hoop-daemon/src/openapi.rs:453 (ScriptRunRequest)
- hoop-daemon/src/openapi.rs:497 (EnableTourRequest)
- hoop-daemon/src/openapi.rs:500 (ListJobsQuery)
- hoop-daemon/src/api_tour_project.rs:73 (EnableTourRequest)
- hoop-daemon/src/api_screen_capture.rs:84 (CreateScreenCaptureRequest)
- hoop-daemon/src/api_screen_capture.rs:366 (StartStreamingUploadRequest)
- hoop-daemon/src/api_screen_capture.rs:484 (CompleteStreamingUploadRequest)

These structs are used in OpenAPI path definitions but lack the required `#[derive(ToSchema)]` attribute.

## Warnings (74 total)

### Unused Imports (30+ warnings)
Files affected:
- accounts_config.rs: PathBuf, warn
- api_bead_files.rs: State, Connection, params, Deserialize
- api_pattern_mutations.rs: get
- api_stitch_decompose.rs: Arc
- api_stitch_replay.rs: ReplayOptions
- api_unassigned.rs: ParsedSessionKind
- api_skills.rs: RecommendedWatcher
- atomic_write.rs: PathBuf
- capacity.rs: Duration, OpenCodeLimits, Utc
- content_blocks.rs: Utc
- api_presence.rs: HashMap
- api_tour_project.rs: get
- migrations.rs: Serialize
- stitch_reconstruction.rs: anyhow, HashMap
- stuck_detector.rs: Result
- prompt_substitute.rs: anyhow, bail, json
- api_prompts.rs: SubstitutionContext
- config_backup.rs: warn
- cross_project_propagation.rs: SimilarStitch, DateTime
- api_fix_patterns.rs: delete, put
- api_screen_capture.rs: self
- screen_capture.rs: Path
- saturation_detector.rs: Deserialize, Serialize
- observer.rs: log_rotation, TcpStream
- lib.rs: AgentConfigChanged

### Unused Variables (25+ warnings)
Files affected:
- backup_pipeline.rs: start
- auth.rs: remote_addr, required_role (2)
- api_scripts.rs: start, timed_out
- api_stitch_links.rs: elapsed_ms
- api_skills.rs: start, timed_out
- capacity.rs: config, event_type
- config_watcher.rs: initial_hash, cfg
- fleet.rs: start
- stitch_traversal.rs: link_kind
- script_scheduler.rs: schedule, overlap_policy
- stitch_reconstruction.rs: workspace
- stuck_detector.rs: transition_secs
- cross_project_propagation.rs: created_by, conn, sim, source_labels
- api_fix_patterns.rs: create_req
- screen_capture.rs: attachments_dir
- observer.rs: dashboard
- lib.rs: abs_path, project, synthesis_callback, semaphore_ref

### Unused Mut (8+ warnings)
Files affected:
- api_tour_project.rs: conn
- api_fix_patterns.rs: conn
- lib.rs: shutdown_rx
- capacity.rs: gemini_dirs, opencode_dirs
- cross_project_propagation.rs: shared_files, shared_labels
- fix_patterns.rs: conn (2)

## Fix Strategy

### Errors
Add `#[derive(ToSchema)]` to the following request structs:
- `ScriptRunRequest` in api_scripts.rs
- `EnableTourRequest` in api_tour_project.rs
- `ListJobsQuery` in api_transcription.rs
- `CreateScreenCaptureRequest` in api_screen_capture.rs
- `StartStreamingUploadRequest` in api_screen_capture.rs
- `CompleteStreamingUploadRequest` in api_screen_capture.rs

This requires adding the derive macro and ensuring all field types also implement ToSchema.

### Warnings
Run `cargo clippy --fix` to automatically fix unused imports and variables, or manually prefix with underscore for intentionally unused variables.

## Full Build Output

The complete build log has been saved to `/tmp/hoop-debug-build.log`.

## Acceptance Criteria Verification

- ✅ Full compilation output saved to file
- ✅ Output contains all error information (22 errors)
- ✅ Output contains all warning information (74 warnings)
- ✅ Build artifacts captured in `/tmp/hoop-debug-build.log`
