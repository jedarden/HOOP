# Clippy Warnings for hoop-daemon (bf-xibss)

**Date:** 2025-01-16
**Command:** `cargo clippy -p hoop-daemon`

## Summary

- **Total Warnings:** 74
- **Total Errors:** 54 (all compilation errors, not clippy-specific warnings)

## utoipa::ToSchema Errors (54 compilation errors)

All errors are related to missing `utoipa::ToSchema` derives on structs used in OpenAPI handlers. The following types need the derive:

### API Agent (3 types)
- `api_agent::SwitchRequest` (api_agent.rs:127)
- `api_agent::TurnRequest` (api_agent.rs:194)
- `api_agent::TurnAttachment` (api_agent.rs:186)

### Cross Project Propagation (1 type)
- `cross_project_propagation::SiblingProject` (cross_project_propagation.rs:23)

### API Reflection Ledger (2 types)
- `api_reflection_ledger::ApproveProposalRequest` (api_reflection_ledger.rs:42)
- `api_reflection_ledger::RejectProposalRequest` (api_reflection_ledger.rs:59)

### API Scripts (1 type)
- `api_scripts::ScriptRunRequest` (api_scripts.rs:162)

### API Tour Project (1 type)
- `api_tour_project::EnableTourRequest` (api_tour_project.rs:34)

### API Transcription (1 type)
- `api_transcription::ListJobsQuery` (api_transcription.rs:19)

### API Screen Capture (3 types)
- `api_screen_capture::CreateScreenCaptureRequest` (api_screen_capture.rs:34)
- `api_screen_capture::StartStreamingUploadRequest` (api_screen_capture.rs:352)
- `api_screen_capture::CompleteStreamingUploadRequest` (api_screen_capture.rs:469)

## Other Warnings (74 total)

### Unused Imports (37 warnings)

- `PathBuf` - accounts_config.rs:27, atomic_write.rs:42
- `warn` - accounts_config.rs:28, config_backup.rs:14
- `State` - api_bead_files.rs:11
- `Connection`, `params` - api_bead_files.rs:16
- `Deserialize` - api_bead_files.rs:17
- `get` - api_pattern_mutations.rs:14, api_tour_project.rs:12
- `std::sync::Arc` - api_stitch_decompose.rs:30
- `ReplayOptions` - api_stitch_replay.rs:8
- `ParsedSessionKind` - api_unassigned.rs:23
- `RecommendedWatcher` - api_skills.rs:39
- `std::time::Duration as StdDuration` - capacity.rs:25
- `OpenCodeLimits as AccountsOpenCodeLimits` - capacity.rs:28
- `chrono::Utc` - content_blocks.rs:7
- `std::collections::HashMap` - api_presence.rs:20, stitch_reconstruction.rs:22
- `serde::Serialize` - migrations.rs:51
- `anyhow` - stitch_reconstruction.rs:19
- `anyhow::Result` - stuck_detector.rs:20
- `anyhow`, `bail` - prompt_substitute.rs:13
- `json` - prompt_substitute.rs:15
- `SubstitutionContext` - api_prompts.rs:45
- `SimilarStitch` - cross_project_propagation.rs:15
- `DateTime` - cross_project_propagation.rs:17
- `delete`, `put` - api_fix_patterns.rs:16
- `self` - api_screen_capture.rs:12
- `Path` - screen_capture.rs:23
- `Deserialize`, `Serialize` - saturation_detector.rs:17
- `crate::log_rotation` - observer.rs:8
- `TcpStream` - observer.rs:16
- `config_watcher::AgentConfigChanged` - lib.rs:3151

### Unused Variables (30 warnings)

Timing variables (typically used for debugging but not currently used):
- `start` - backup_pipeline.rs:133, api_scripts.rs:311, api_skills.rs:284, fleet.rs:1704

Auth/role variables:
- `remote_addr` - auth.rs:338
- `required_role` - auth.rs:329, auth.rs:358

Other unused variables:
- `elapsed_ms` - api_stitch_links.rs:208
- `config` - capacity.rs:213
- `event_type` - capacity.rs:1796
- `initial_hash` - config_watcher.rs:111
- `cfg` - config_watcher.rs:139
- `link_kind` - stitch_traversal.rs:210
- `schedule` - script_scheduler.rs:139
- `overlap_policy` - script_scheduler.rs:109
- `workspace` - stitch_reconstruction.rs:297
- `transition_secs` - stuck_detector.rs:463
- `created_by` - cross_project_propagation.rs:224
- `conn` - cross_project_propagation.rs:455
- `sim` - cross_project_propagation.rs:473
- `source_labels` - cross_project_propagation.rs:479
- `create_req` - api_fix_patterns.rs:444
- `attachments_dir` - screen_capture.rs:327
- `dashboard` - observer.rs:210
- `abs_path` - lib.rs:975
- `project` - lib.rs:2415
- `synthesis_callback` - lib.rs:2413
- `semaphore_ref` - lib.rs:3077

### Unused `mut` Keywords (6 warnings)

Variables declared as `mut` but never mutated:
- `conn` - api_tour_project.rs:239, api_fix_patterns.rs:454, fix_patterns.rs:83, fix_patterns.rs:277
- `shutdown_rx` - lib.rs:3446
- `gemini_dirs` - capacity.rs:593
- `opencode_dirs` - capacity.rs:596
- `shared_files` - cross_project_propagation.rs:472
- `shared_labels` - cross_project_propagation.rs:480

### Unused Assignments (1 warning)

- `timed_out` - api_scripts.rs:360, api_skills.rs:344 (assigned but never read)

## Notes

- No actual clippy-specific warnings (like `clippy::all` or `clippy::pedantic` warnings) were found
- All errors are compilation errors due to missing utoipa derives
- The warnings are mostly about unused code, which could be cleaned up for better maintainability
