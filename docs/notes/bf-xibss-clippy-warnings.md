# Clippy Warnings for hoop-daemon

**Date:** 2026-06-28
**Command:** `cargo clippy -p hoop-daemon`

## Summary

- **Total Warnings:** 74
- **Total Errors:** 54 (all compilation errors, not clippy warnings)
- **Exit Status:** Failed to compile due to utoipa::ToSchema trait errors

---

## Compilation Errors (54 total) — All utoipa::ToSchema Related

All errors are `E0277: the trait bound X: utoipa::ToSchema is not satisfied`. These types are used in OpenAPI documentation but lack the `#[derive(utoipa::ToSchema)]` attribute.

### Types Missing utoipa::ToSchema:

1. **api_agent.rs** (3 types):
   - `SwitchRequest` (line 127)
   - `TurnRequest` (line 194)
   - `TurnAttachment` (line 186)

2. **api_reflection_ledger.rs** (2 types):
   - `ApproveProposalRequest` (line 42)
   - `RejectProposalRequest` (line 59)

3. **api_scripts.rs** (1 type):
   - `ScriptRunRequest` (line 162)

4. **api_tour_project.rs** (1 type):
   - `EnableTourRequest` (line 34)

5. **api_transcription.rs** (1 type):
   - `ListJobsQuery` (line 19)

6. **cross_project_propagation.rs** (1 type):
   - `SiblingProject` (line 23)

7. **api_screen_capture.rs** (3 types):
   - `CreateScreenCaptureRequest` (line 34)
   - `StartStreamingUploadRequest` (line 352)
   - `CompleteStreamingUploadRequest` (line 469)

### Error Pattern:

Each error appears in two contexts:
1. When used in `#[utoipa::request_body = ...]` attribute
2. When listed in `openapi.rs` schemas list

Example:
```
error[E0277]: the trait bound `api_agent::SwitchRequest: utoipa::ToSchema` is not satisfied
   --> hoop-daemon/src/api_agent.rs:143:20
    |
143 |     request_body = SwitchRequest,
    |                    ^^^^^^^^^^^^^ unsatisfied trait bound
```

---

## Clippy Warnings (74 total)

### Unused Imports (40 warnings)

| File | Unused Imports |
|------|----------------|
| accounts_config.rs | `PathBuf`, `warn` |
| api_bead_files.rs | `State`, `Connection`, `params`, `Deserialize` |
| api_pattern_mutations.rs | `get` |
| api_stitch_decompose.rs | `std::sync::Arc` |
| api_stitch_replay.rs | `ReplayOptions` |
| api_unassigned.rs | `ParsedSessionKind` |
| api_skills.rs | `RecommendedWatcher` |
| atomic_write.rs | `PathBuf` |
| capacity.rs | `std::time::Duration as StdDuration`, `OpenCodeLimits as AccountsOpenCodeLimits` |
| content_blocks.rs | `chrono::Utc` |
| api_presence.rs | `std::collections::HashMap` |
| api_tour_project.rs | `get` |
| migrations.rs | `serde::Serialize` |
| stitch_reconstruction.rs | `anyhow`, `std::collections::HashMap` |
| stuck_detector.rs | `anyhow::Result` |
| prompt_substitute.rs | `anyhow`, `bail`, `json` |
| api_prompts.rs | `SubstitutionContext` |
| config_backup.rs | `warn` |
| cross_project_propagation.rs | `SimilarStitch`, `DateTime` |
| api_fix_patterns.rs | `delete`, `put` |
| api_screen_capture.rs | `self` |
| screen_capture.rs | `Path` |
| saturation_detector.rs | `Deserialize`, `Serialize` |
| observer.rs | `crate::log_rotation`, `TcpStream` |
| lib.rs | `config_watcher::AgentConfigChanged` |

### Unused Variables (28 warnings)

| File | Variable | Context |
|------|----------|---------|
| backup_pipeline.rs | `start` | Timing measurement |
| auth.rs | `remote_addr` | Connection info |
| auth.rs | `required_role` (2 instances) | Role checking |
| api_scripts.rs | `start` | Timing measurement |
| api_scripts.rs | `timed_out` | Timeout tracking |
| api_stitch_links.rs | `elapsed_ms` | Performance metric |
| api_skills.rs | `start` | Timing measurement |
| api_skills.rs | `timed_out` | Timeout tracking |
| capacity.rs | `config` | GCP quota config |
| capacity.rs | `event_type` | Event log parsing |
| config_watcher.rs | `initial_hash` | Config hash |
| config_watcher.rs | `cfg` | Config reference |
| fleet.rs | `start` | Timing measurement |
| stitch_traversal.rs | `link_kind` | Link relationship |
| script_scheduler.rs | `schedule` | Cron schedule |
| script_scheduler.rs | `overlap_policy` | Execution policy |
| stitch_reconstruction.rs | `workspace` | File path |
| stuck_detector.rs | `transition_secs` | Time calculation |
| cross_project_propagation.rs | `created_by` | User attribution |
| cross_project_propagation.rs | `conn` | Database connection |
| cross_project_propagation.rs | `sim` | Similarity score |
| cross_project_propagation.rs | `source_labels` | Label set |
| api_fix_patterns.rs | `create_req` | Pattern creation |
| screen_capture.rs | `attachments_dir` | Directory path |
| observer.rs | `dashboard` | JSON response |
| lib.rs | `abs_path` | File path |
| lib.rs | `project` | Project name |
| lib.rs | `synthesis_callback` | Callback closure |
| lib.rs | `semaphore_ref` | Semaphore reference |

### Unnecessary Mutability (6 warnings)

| File | Variable | Line |
|------|----------|------|
| api_tour_project.rs | `conn` | 239 |
| api_fix_patterns.rs | `conn` | 454 |
| lib.rs | `shutdown_rx` | 3446 |
| capacity.rs | `gemini_dirs` | 593 |
| capacity.rs | `opencode_dirs` | 596 |
| cross_project_propagation.rs | `shared_files` | 472 |
| cross_project_propagation.rs | `shared_labels` | 480 |
| fix_patterns.rs | `conn` (2 instances) | 83, 277 |

---

## Recommendation

**Priority 1:** Fix the 54 compilation errors by adding `#[derive(utoipa::ToSchema)]` to the 11 request/response types listed above. This is blocking compilation.

**Priority 2:** Clean up 74 clippy warnings:
- Remove 40 unused imports
- Prefix 28 unused variables with `_` or remove them
- Remove `mut` keyword from 6 unnecessarily mutable variables

---

## Context

These warnings were documented as part of bead **bf-xibss**: "Check clippy warnings on hoop-daemon". The goal was to identify and document all utoipa-related warnings and the total warning count before cleanup.
