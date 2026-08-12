# HOOP Test Error Pattern Mapping

Generated: 2026-08-12
Bead: bf-rilp7

## Summary Statistics

- **Total test files searched**: 331
- **Files with assert! patterns**: 304 (91.8%)
- **Files with expect! patterns**: 0 (0%)
- **Files with expect_err! patterns**: 0 (0%)
- **Files with panic! patterns**: 42 (12.7%)
- **Files with unwrap_err() patterns**: 26 (7.8%)
- **Files with anyhow:: patterns**: 137 (41.4%)
- **Files with Result<_, Error> patterns**: 34 (10.3%)

---

## Pattern 1: assert! (304 files)

The `assert!` macro is the most common error-checking pattern in HOOP tests.

### Integration Tests (116 files)
- `tests/cli_test_helpers.rs`
- `tests/acceptance/s1_morning_review.rs`
- `tests/acceptance/s2_transcript_archaeology.rs`
- `tests/acceptance/s3_bead_creation_from_chat.rs`
- `tests/acceptance/s4_daemon_restart.rs`
- `tests/acceptance/s5_workspace_deleted.rs`
- `tests/acceptance/s6_machine_mode.rs`
- `hoop-daemon/tests/projection_file_audit.rs`
- `hoop-daemon/tests/backup_restore_cycle.rs`
- `hoop-daemon/tests/lint_regex_global_state.rs`
- `hoop-daemon/tests/config_reload_audit.rs`
- `hoop-daemon/tests/reflection_detector_integration.rs`
- `hoop-daemon/tests/quarantine_integration.rs`
- `hoop-daemon/tests/output_capture_helpers/mod.rs`
- `hoop-daemon/tests/protocol_contract.rs`
- `hoop-daemon/tests/phase2_exit_gate.rs`
- `hoop-daemon/tests/claimed_at_parsing.rs`
- `hoop-daemon/tests/config_field_validation.rs`
- `hoop-daemon/tests/stderr_stdout_capture.rs`
- `hoop-daemon/tests/integration_harness.rs`
- `hoop-daemon/tests/skills_integration.rs`
- `hoop-daemon/tests/skills_quarantine_integration.rs`
- `hoop-daemon/tests/session_redaction.rs`
- `hoop-daemon/tests/disaster_recovery_runbook.rs`
- `hoop-daemon/tests/secrets_scanner_integration.rs`
- `hoop-daemon/tests/per_project_redaction_integration.rs`
- `hoop-daemon/tests/fleet_notifications_integration.rs`
- `hoop-daemon/tests/adapter_failover.rs`
- `hoop-daemon/tests/stdout_verification.rs`
- `hoop-daemon/tests/create_stitch_no_auto_submit.rs`
- `hoop-daemon/tests/draft_queue_invariants.rs`
- `hoop-daemon/tests/property_invariants.rs`
- `hoop-daemon/tests/secrets_scanner_parity.rs`
- `hoop-daemon/tests/stitch_percentile_index_integration.rs`
- `hoop-daemon/tests/multi_operator_concurrency.rs`
- `hoop-daemon/tests/golden_transcripts_regression.rs`
- `hoop-daemon/tests/needle_events_roundtrip.rs`
- `hoop-daemon/tests/zero_write_invariant.rs`
- `hoop-daemon/tests/bead_real_line_deserialization.rs`
- `hoop-daemon/tests/load_test_integration.rs`
- `hoop-daemon/tests/testrepo_integration.rs`
- `hoop-daemon/tests/testrepo_harness_integration.rs`
- `hoop-daemon/tests/hoop_dies_nothing_notices.rs`
- `hoop-daemon/tests/beads_deletion_http.rs`
- `hoop-daemon/tests/state_projections.rs`
- `hoop-daemon/tests/mutation_handler_test.rs`
- `hoop-daemon/tests/adapter_failover_test.rs`
- `hoop-cli/tests/clap_test_utils.rs`
- `testrepo/tests/integration/test_*.rs` (20 files)

### Unit Tests in hoop-daemon/src/ (109 files)
All unit test files in hoop-daemon/src/ contain `assert!` patterns for validation:
- `hoop-daemon/src/lib.rs`
- `hoop-daemon/src/audit.rs`
- `hoop-daemon/src/auth.rs`
- `hoop-daemon/src/backup.rs`
- `hoop-daemon/src/backup_pipeline.rs`
- `hoop-daemon/src/bead_commit_index.rs`
- `hoop-daemon/src/beads.rs`
- `hoop-daemon/src/br_verbs.rs`
- `hoop-daemon/src/capacity.rs`
- `hoop-daemon/src/collision_detector.rs`
- `hoop-daemon/src/config_backup.rs`
- `hoop-daemon/src/config_resolver.rs`
- `hoop-daemon/src/config_watcher.rs`
- `hoop-daemon/src/content_blocks.rs`
- `hoop-daemon/src/cost_anomaly.rs`
- `hoop-daemon/src/cost.rs`
- `hoop-daemon/src/cross_project_propagation.rs`
- `hoop-daemon/src/dictated_notes.rs`
- `hoop-daemon/src/embedding_service.rs`
- `hoop-daemon/src/events.rs`
- `hoop-daemon/src/file_io_error.rs`
- `hoop-daemon/src/files.rs`
- `hoop-daemon/src/fix_patterns.rs`
- `hoop-daemon/src/fleet_notifications.rs`
- `hoop-daemon/src/fleet.rs`
- `hoop-daemon/src/heartbeats.rs`
- `hoop-daemon/src/identity.rs`
- `hoop-daemon/src/integration_harness.rs`
- `hoop-daemon/src/integration_test_client.rs`
- `hoop-daemon/src/load_test.rs`
- `hoop-daemon/src/log_rotation.rs`
- `hoop-daemon/src/metrics.rs`
- `hoop-daemon/src/migrations.rs`
- `hoop-daemon/src/morning_brief.rs`
- `hoop-daemon/src/mutation_handler.rs`
- `hoop-daemon/src/net_diff.rs`
- `hoop-daemon/src/orphan_beads.rs`
- `hoop-daemon/src/parse_jsonl_safe.rs`
- `hoop-daemon/src/path_security.rs`
- `hoop-daemon/src/pattern_query_evaluator.rs`
- `hoop-daemon/src/pdf_sanitize.rs`
- `hoop-daemon/src/predictor.rs`
- `hoop-daemon/src/pricing_watcher.rs`
- `hoop-daemon/src/projects.rs`
- `hoop-daemon/src/prompt_substitute.rs`
- `hoop-daemon/src/redaction_policy.rs`
- `hoop-daemon/src/redaction.rs`
- `hoop-daemon/src/reflection_detector.rs`
- `hoop-daemon/src/risk_patterns.rs`
- `hoop-daemon/src/saturation_detector.rs`
- `hoop-daemon/src/screen_capture.rs`
- `hoop-daemon/src/script_scheduler.rs`
- `hoop-daemon/src/script_trigger.rs`
- `hoop-daemon/src/secrets_scanner.rs`
- `hoop-daemon/src/sessions.rs`
- `hoop-daemon/src/shutdown.rs`
- `hoop-daemon/src/similarity.rs`
- `hoop-daemon/src/snapshot_manifest.rs`
- `hoop-daemon/src/stitch_decompose.rs`
- `hoop-daemon/src/stitch_percentile_index.rs`
- `hoop-daemon/src/stitch_reconstruction.rs`
- `hoop-daemon/src/stitch_status.rs`
- `hoop-daemon/src/stitch_traversal.rs`
- `hoop-daemon/src/stuck_detector.rs`
- `hoop-daemon/src/supervisor.rs`
- `hoop-daemon/src/svg_sanitize.rs`
- `hoop-daemon/src/syntax_highlight.rs`
- `hoop-daemon/src/syntax_highlight_stream.rs`
- `hoop-daemon/src/tag_join.rs`
- `hoop-daemon/src/template_library.rs`
- `hoop-daemon/src/transcription.rs`
- `hoop-daemon/src/unknown_event_sink.rs`
- `hoop-daemon/src/uploads.rs`
- `hoop-daemon/src/vector_index.rs`
- `hoop-daemon/src/worker_ack.rs`
- `hoop-daemon/src/ws.rs`
- `hoop-daemon/src/api_*.rs` (30+ API endpoint files)

### Unit Tests in hoop-cli/src/ (9 files)
- `hoop-cli/src/backup.rs`
- `hoop-cli/src/init.rs`
- `hoop-cli/src/main.rs`
- `hoop-cli/src/new.rs`
- `hoop-cli/src/projects.rs`
- `hoop-cli/src/reflection.rs`
- `hoop-cli/src/restore.rs`
- `hoop-cli/src/risk_patterns.rs`

### Unit Tests in hoop-mcp/src/ (6 files)
- `hoop-mcp/src/audit.rs`
- `hoop-mcp/src/br_verbs.rs`
- `hoop-mcp/src/notes.rs`
- `hoop-mcp/src/redaction.rs`
- `hoop-mcp/src/skills.rs`
- `hoop-mcp/src/socket.rs`

### Unit Tests in hoop-schema/src/ (4 files)
- `hoop-schema/src/effort.rs`
- `hoop-schema/src/id_validators.rs`
- `hoop-schema/src/lib.rs`
- `hoop-schema/src/path_security.rs`

### Unit Tests in testrepo/src/ (101 files)
All testrepo unit test files contain `assert!` patterns for comprehensive testing

---

## Pattern 2: expect! (0 files)

**No files found using the `expect!` macro with custom error messages.**

This is a significant finding for error message consistency work. The absence of `expect!` means:
- All expectations use default `assert!` messages
- No custom context is provided when assertions fail
- Error messages will be generic ("assertion failed: left == right")

---

## Pattern 3: expect_err! (0 files)

**No files found using the `expect_err!` macro.**

This indicates:
- Error case testing relies on different patterns
- No specific error message validation in current tests

---

## Pattern 4: panic! (42 files)

### Integration Tests (20 files)
- `tests/cli_test_helpers.rs`
- `hoop-daemon/tests/projection_file_audit.rs`
- `hoop-daemon/tests/backup_restore_cycle.rs`
- `hoop-daemon/tests/lint_regex_global_state.rs`
- `hoop-daemon/tests/pure_functions.rs`
- `hoop-daemon/tests/protocol_contract.rs`
- `hoop-daemon/tests/integration_harness.rs`
- `hoop-daemon/tests/per_project_redaction_integration.rs`
- `hoop-daemon/tests/property_invariants.rs`
- `hoop-daemon/tests/golden_transcripts_regression.rs`
- `hoop-daemon/tests/needle_events_roundtrip.rs`
- `hoop-daemon/tests/zero_write_invariant.rs`
- `hoop-daemon/tests/bead_real_line_deserialization.rs`
- `hoop-daemon/tests/load_test_integration.rs`
- `hoop-daemon/tests/testrepo_integration.rs`
- `hoop-daemon/tests/hoop_dies_nothing_notices.rs`
- `hoop-daemon/tests/adapter_failover_test.rs`
- `hoop-daemon/tests/testrepo_harness_integration.rs`
- `hoop-daemon/tests/acceptance/s6_machine_mode.rs`
- `hoop-cli/tests/clap_test_utils.rs`

### Unit Tests in hoop-daemon/src/ (12 files)
- `hoop-daemon/src/br_verbs.rs`
- `hoop-daemon/src/projects.rs`
- `hoop-daemon/src/stitch_status.rs`
- `hoop-daemon/src/api_metrics.rs`
- `hoop-daemon/src/parse_jsonl_safe.rs`
- `hoop-daemon/src/config_resolver.rs`
- `hoop-daemon/src/events.rs`
- `hoop-daemon/src/config_watcher.rs`
- `hoop-daemon/src/agent_adapter.rs`
- `hoop-daemon/src/agent_session.rs`
- `hoop-daemon/src/prompt_substitute.rs`
- `hoop-daemon/src/file_io_error.rs`
- `hoop-daemon/src/syntax_highlight_stream.rs`
- `hoop-daemon/src/api_preview.rs`
- `hoop-daemon/src/api_onboarding.rs`
- `hoop-daemon/src/ws.rs`

### Unit Tests in other crates (10 files)
- hoop-cli, hoop-mcp, hoop-schema, and testrepo files

---

## Pattern 5: unwrap_err() (26 files)

### Integration Tests (10 files)
- `hoop-daemon/tests/backup_restore_cycle.rs`
- `hoop-daemon/tests/claimed_at_parsing.rs`
- `hoop-daemon/tests/skills_integration.rs`
- `hoop-daemon/tests/disaster_recovery_runbook.rs`
- `hoop-daemon/tests/per_project_redaction_integration.rs`
- `hoop-daemon/tests/config_reload_cycle.rs`
- `hoop-daemon/tests/create_only_stub.rs`
- `hoop-cli/tests/cli_test_helpers.rs`
- `hoop-cli/tests/cli_test_utils_examples.rs`

### Unit Tests in hoop-daemon/src/ (8 files)
- `hoop-daemon/src/projects.rs`
- `hoop-daemon/src/api_stitch_decompose.rs`
- `hoop-daemon/src/snapshot_manifest.rs`
- `hoop-daemon/src/api_beads.rs`
- `hoop-daemon/src/prompt_substitute.rs`
- `hoop-daemon/src/file_io_error.rs`
- `hoop-daemon/src/audio_redaction.rs`
- `hoop-daemon/src/api_skills.rs`
- `hoop-daemon/src/pdf_sanitize.rs`
- `hoop-daemon/src/attachments.rs`
- `hoop-daemon/src/fleet.rs`

### Unit Tests in other crates (8 files)
- `hoop-schema/src/id_validators.rs`
- `hoop-schema/src/effort.rs`
- `hoop-mcp/tests/forbidden_worker_steering.rs`
- `hoop-mcp/src/skills.rs`
- `hoop-cli/src/projects.rs`
- `hoop-cli/src/new.rs`
- `hoop-cli/src/restore.rs`

---

## Pattern 6: anyhow:: (137 files)

### Integration Tests (22 files)
- `tests/acceptance/s1_morning_review.rs`
- `tests/acceptance/s2_transcript_archaeology.rs`
- `tests/acceptance/s3_bead_creation_from_chat.rs`
- `tests/acceptance/s4_daemon_restart.rs`
- `tests/acceptance/s5_workspace_deleted.rs`
- `tests/acceptance/s6_machine_mode.rs`
- `hoop-daemon/tests/projection_file_audit.rs`
- `hoop-daemon/tests/s5_workspace_deleted.rs`
- `hoop-daemon/tests/filesystem_failure_isolation.rs`
- `hoop-daemon/tests/s4_daemon_restart.rs`
- `hoop-daemon/tests/integration_harness.rs`
- `hoop-daemon/tests/skills_quarantine_integration.rs`
- `hoop-daemon/tests/testrepo_integration.rs`
- `hoop-daemon/tests/disaster_recovery_runbook.rs`
- `hoop-daemon/tests/adapter_failover_test.rs`
- `hoop-daemon/tests/testrepo_harness_integration.rs`
- `hoop-daemon/tests/hoop_dies_nothing_notices.rs`
- `hoop-daemon/tests/beads_deletion_http.rs`
- `hoop-daemon/tests/state_projections.rs`
- `hoop-cli/tests/remove_no_interactive_flag.rs`

### Unit Tests in hoop-daemon/src/ (107+ files)
The majority of hoop-daemon unit tests use `anyhow::` for error handling:
- `hoop-daemon/src/lib.rs`
- `hoop-daemon/src/audit.rs`
- `hoop-daemon/src/net_diff.rs`
- `hoop-daemon/src/cross_project_propagation.rs`
- `hoop-daemon/src/screen_capture.rs`
- `hoop-daemon/src/backup_pipeline.rs`
- `hoop-daemon/src/capacity.rs`
- `hoop-daemon/src/saturation_detector.rs`
- `hoop-daemon/src/br_verbs.rs`
- `hoop-daemon/src/projects.rs`
- `hoop-daemon/src/agent_context.rs`
- `hoop-daemon/src/svg_sanitize.rs`
- `hoop-daemon/src/supervisor.rs`
- `hoop-daemon/src/bead_commit_index.rs`
- `hoop-daemon/src/integration_harness.rs`
- `hoop-daemon/src/orphan_beads.rs`
- `hoop-daemon/src/integration_test_client.rs`
- `hoop-daemon/src/embedding_service.rs`
- `hoop-daemon/src/events.rs`
- `hoop-daemon/src/api_diff.rs`
- `hoop-daemon/src/config_watcher.rs`
- `hoop-daemon/src/pricing_watcher.rs`
- `hoop-daemon/src/vector_index.rs`
- `hoop-daemon/src/api_notes.rs`
- `hoop-daemon/src/reflection_detector.rs`
- And 80+ more files

### Unit Tests in other crates (8 files)
- hoop-cli, hoop-mcp, hoop-schema files

---

## Pattern 7: Result<_, Error> (34 files)

### Integration Tests (4 files)
- `tests/cli_test_helpers.rs`
- `hoop-daemon/tests/output_capture_helpers/mod.rs`
- `hoop-daemon/tests/phase2_exit_gate.rs`
- `hoop-cli/tests/clap_test_utils.rs`

### Unit Tests in hoop-daemon/src/ (24 files)
- `hoop-daemon/src/lib.rs`
- `hoop-daemon/src/capacity.rs`
- `hoop-daemon/src/projects.rs`
- `hoop-daemon/src/config_resolver.rs`
- `hoop-daemon/src/events.rs`
- `hoop-daemon/src/config_watcher.rs`
- `hoop-daemon/src/api_notes.rs`
- `hoop-daemon/src/api_prompts.rs`
- `hoop-daemon/src/content_blocks.rs`
- `hoop-daemon/src/prompt_substitute.rs`
- `hoop-daemon/src/redaction.rs`
- `hoop-daemon/src/api_skills.rs`
- `hoop-daemon/src/auth.rs`
- `hoop-daemon/src/worker_ack.rs`
- `hoop-daemon/src/template_library.rs`
- `hoop-daemon/src/redaction_policy.rs`
- `hoop-daemon/src/api_scripts.rs`
- `hoop-daemon/src/sessions.rs`
- `hoop-daemon/src/risk_patterns.rs`
- `hoop-daemon/src/secrets_scanner.rs`
- `hoop-daemon/src/beads.rs`
- `hoop-daemon/src/identity.rs`
- `hoop-daemon/src/fleet.rs`
- `hoop-daemon/src/heartbeats.rs`

### Unit Tests in other crates (6 files)
- `hoop-schema/src/id_validators.rs`
- `hoop-schema/src/path_security.rs`
- `hoop-mcp/src/audit.rs`
- And 3 more in hoop-cli and hoop-mcp

---

## Key Findings

1. **assert! is ubiquitous** - 304/331 files (91.8%) use `assert!` for basic assertions
2. **No custom error messages** - 0 files use `expect!` or `expect_err!` for contextual error messages
3. **anyhow is dominant** - 137 files (41.4%) use `anyhow::` for error handling
4. **Limited unwrap_err usage** - Only 26 files (7.8%) explicitly test error values with `unwrap_err()`
5. **Minimal panic! usage** - 42 files (12.7%) contain explicit panics, mostly for unreachable code paths

## Recommendations for Error Message Consistency

1. **Replace assert! with expect!** - Add context to failing assertions by using `expect!` instead of `assert!` where helpful
2. **Standardize error messages** - Create a consistent error message format across tests
3. **Leverage anyhow::bail! and anyhow::ensure!** - Use anyhow's context-preserving macros for better error messages
4. **Document error patterns** - Create guidelines for error message formatting in test code

---

## Next Steps

This mapping provides the foundation for:
- Extracting actual error messages from each pattern type
- Identifying inconsistencies in error message format
- Standardizing error messages across the test suite
- Creating error message guidelines for future test development
