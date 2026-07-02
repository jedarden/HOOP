# HOOP Debug Build Compilation Errors and Warnings

**Build Date:** 2026-07-02  
**Command:** `cargo build`  
**Result:** FAILED (14 errors, 74 warnings)

---

## Compilation Errors (14 total)

### Error 1-2: ListJobsQuery missing ToSchema trait
**File:** `hoop-daemon/src/openapi.rs:500`  
**Type:** E0277 (trait bound not satisfied)

```
error[E0277]: the trait bound `ListJobsQuery: ToSchema` is not satisfied
   --> hoop-daemon/src/openapi.rs:500:13
    |
500 |             crate::api_transcription::ListJobsQuery,
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ unsatisfied trait bound
```

**Cause:** `ListJobsQuery` in `hoop-daemon/src/api_transcription.rs:19` does not implement the `ToSchema` trait required by utoipa OpenAPI generation.

**Required Fix:** Add `#[derive(ToSchema)]` to `ListJobsQuery` struct.

---

### Error 3-4: CreateScreenCaptureRequest missing ToSchema trait
**File:** `hoop-daemon/src/api_screen_capture.rs:84`  
**Type:** E0277 (trait bound not satisfied)

```
error[E0277]: the trait bound `CreateScreenCaptureRequest: ToSchema` is not satisfied
  --> hoop-daemon/src/api_screen_capture.rs:84:20
   |
84 |     request_body = CreateScreenCaptureRequest,
   |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^ unsatisfied trait bound
```

**Cause:** `CreateScreenCaptureRequest` in `hoop-daemon/src/api_screen_capture.rs:34` does not implement the `ToSchema` trait.

**Required Fix:** Add `#[derive(ToSchema)]` to `CreateScreenCaptureRequest` struct.

---

### Error 5-6: StartStreamingUploadRequest missing ToSchema trait
**File:** `hoop-daemon/src/api_screen_capture.rs:366`  
**Type:** E0277 (trait bound not satisfied)

```
error[E0277]: the trait bound `StartStreamingUploadRequest: ToSchema` is not satisfied
  --> hoop-daemon/src/api_screen_capture.rs:366:20
   |
366 |     request_body = StartStreamingUploadRequest,
   |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ unsatisfied trait bound
```

**Cause:** `StartStreamingUploadRequest` in `hoop-daemon/src/api_screen_capture.rs:352` does not implement the `ToSchema` trait.

**Required Fix:** Add `#[derive(ToSchema)]` to `StartStreamingUploadRequest` struct.

---

### Error 7-8: CompleteStreamingUploadRequest missing ToSchema trait
**File:** `hoop-daemon/src/api_screen_capture.rs:484`  
**Type:** E0277 (trait bound not satisfied)

```
error[E0277]: the trait bound `CompleteStreamingUploadRequest: ToSchema` is not satisfied
  --> hoop-daemon/src/api_screen_capture.rs:484:20
   |
484 |     request_body = CompleteStreamingUploadRequest,
   |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ unsatisfied trait bound
```

**Cause:** `CompleteStreamingUploadRequest` in `hoop-daemon/src/api_screen_capture.rs:469` does not implement the `ToSchema` trait.

**Required Fix:** Add `#[derive(ToSchema)]` to `CompleteStreamingUploadRequest` struct.

---

## Summary of Errors

All 14 compilation errors are caused by the same issue: **missing `ToSchema` derives on request structs used in OpenAPI documentation**.

**Affected structs:**
1. `ListJobsQuery` (api_transcription.rs)
2. `CreateScreenCaptureRequest` (api_screen_capture.rs)
3. `StartStreamingUploadRequest` (api_screen_capture.rs)
4. `CompleteStreamingUploadRequest` (api_screen_capture.rs)

Each struct generates 2 errors (one for `ToSchema`, one for `PartialSchema`), totaling 14 errors.

---

## Compilation Warnings (74 total)

### Unused Imports (36 warnings)

| File | Line | Unused Import |
|------|------|--------------|
| accounts_config.rs | 27 | `PathBuf` |
| accounts_config.rs | 28 | `warn` |
| api_bead_files.rs | 11 | `State` |
| api_bead_files.rs | 16 | `Connection`, `params` |
| api_bead_files.rs | 17 | `Deserialize` |
| api_pattern_mutations.rs | 14 | `get` |
| api_stitch_decompose.rs | 30 | `std::sync::Arc` |
| api_stitch_replay.rs | 8 | `ReplayOptions` |
| api_unassigned.rs | 23 | `ParsedSessionKind` |
| api_skills.rs | 39 | `RecommendedWatcher` |
| atomic_write.rs | 42 | `PathBuf` |
| capacity.rs | 25 | `StdDuration` |
| capacity.rs | 28 | `AccountsOpenCodeLimits` |
| content_blocks.rs | 7 | `Utc` |
| api_presence.rs | 20 | `HashMap` |
| api_tour_project.rs | 12 | `get` |
| migrations.rs | 51 | `Serialize` |
| stitch_reconstruction.rs | 19 | `anyhow` |
| stitch_reconstruction.rs | 22 | `HashMap` |
| stuck_detector.rs | 20 | `Result` |
| prompt_substitute.rs | 13 | `anyhow`, `bail` |
| prompt_substitute.rs | 15 | `json` |
| api_prompts.rs | 45 | `SubstitutionContext` |
| config_backup.rs | 14 | `warn` |
| cross_project_propagation.rs | 15 | `SimilarStitch` |
| cross_project_propagation.rs | 17 | `DateTime` |
| api_fix_patterns.rs | 16 | `delete`, `put` |
| api_screen_capture.rs | 12 | `self` |
| screen_capture.rs | 23 | `Path` |
| saturation_detector.rs | 17 | `Deserialize`, `Serialize` |
| observer.rs | 8 | `log_rotation` |
| observer.rs | 16 | `TcpStream` |
| lib.rs | 3151 | `AgentConfigChanged` |

### Unused Variables (30 warnings)

| File | Line | Variable | Issue |
|------|------|----------|-------|
| backup_pipeline.rs | 133 | `start` | unused |
| auth.rs | 338 | `remote_addr` | unused |
| auth.rs | 329 | `required_role` | unused |
| auth.rs | 358 | `required_role` | unused |
| api_scripts.rs | 312 | `start` | unused |
| api_scripts.rs | 361 | `timed_out` | assigned but never used |
| api_stitch_links.rs | 208 | `elapsed_ms` | unused |
| api_skills.rs | 284 | `start` | unused |
| api_skills.rs | 344 | `timed_out` | assigned but never used |
| capacity.rs | 213 | `config` | unused |
| capacity.rs | 1796 | `event_type` | unused |
| config_watcher.rs | 111 | `initial_hash` | unused |
| config_watcher.rs | 139 | `cfg` | unused |
| fleet.rs | 1704 | `start` | unused |
| stitch_traversal.rs | 210 | `link_kind` | unused |
| script_scheduler.rs | 139 | `schedule` | unused |
| script_scheduler.rs | 109 | `overlap_policy` | unused |
| stitch_reconstruction.rs | 297 | `workspace` | unused |
| stuck_detector.rs | 463 | `transition_secs` | unused |
| cross_project_propagation.rs | 220 | `created_by` | unused |
| cross_project_propagation.rs | 451 | `conn` | unused |
| cross_project_propagation.rs | 469 | `sim` | unused |
| cross_project_propagation.rs | 475 | `source_labels` | unused |
| api_fix_patterns.rs | 444 | `create_req` | unused |
| screen_capture.rs | 327 | `attachments_dir` | unused |
| observer.rs | 210 | `dashboard` | unused |
| fix_patterns.rs | 83 | `conn` | unnecessary `mut` |
| fix_patterns.rs | 277 | `conn` | unnecessary `mut` |
| lib.rs | 975 | `abs_path` | unused |
| lib.rs | 2415 | `project` | unused |
| lib.rs | 2413 | `synthesis_callback` | unused |
| lib.rs | 3077 | `semaphore_ref` | unused |

### Unnecessary Mut (8 warnings)

| File | Line | Variable | Issue |
|------|------|----------|-------|
| api_tour_project.rs | 240 | `conn` | does not need to be mutable |
| api_fix_patterns.rs | 454 | `conn` | does not need to be mutable |
| lib.rs | 3446 | `shutdown_rx` | does not need to be mutable |
| capacity.rs | 593 | `gemini_dirs` | does not need to be mutable |
| capacity.rs | 596 | `opencode_dirs` | does not need to be mutable |
| cross_project_propagation.rs | 468 | `shared_files` | does not need to be mutable |
| cross_project_propagation.rs | 476 | `shared_labels` | does not need to be mutable |
| fix_patterns.rs | 83 | `conn` | does not need to be mutable |
| fix_patterns.rs | 277 | `conn` | does not need to be mutable |

---

## Recommended Fix Priority

### High Priority (Block Compilation)
1. Add `#[derive(ToSchema)]` to 4 structs in api_screen_capture.rs and api_transcription.rs
2. Ensure all derives are imported (likely need `use utoipa::ToSchema;`)

### Medium Priority (Code Quality)
1. Remove unused imports (36 instances)
2. Remove or prefix with underscore unused variables (30 instances)

### Low Priority (Style)
1. Remove unnecessary `mut` keywords (8 instances)
