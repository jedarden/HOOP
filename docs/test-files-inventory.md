# HOOP Test Files Inventory

Complete list of all test files in the HOOP project as of 2026-08-12.

## Summary
- **Total test files:** 241
- **Crates with tests:** hoop-cli, hoop-daemon, hoop-mcp, hoop-schema, testrepo
- **Root-level tests:** Yes

---

## hoop-cli (11 files)

### CLI test utilities
- `hoop-cli/tests/clap_test_utils.rs`
- `hoop-cli/tests/cli_test_helpers.rs`
- `hoop-cli/tests/cli_test_utils_examples.rs`
- `hoop-cli/tests/cli_test_utils.rs`

### CLI command tests
- `hoop-cli/tests/init_no_interactive_flag.rs`
- `hoop-cli/tests/no_interactive_flag_behavior.rs`
- `hoop-cli/tests/remove_no_interactive_flag.rs`
- `hoop-cli/tests/restore_no_interactive_flag.rs`
- `hoop-cli/tests/scan_no_interactive_flag.rs`

### Root level
- `hoop-cli/test_all_subcommands.rs`

---

## hoop-daemon (103 files)

### Acceptance tests (suite)
- `hoop-daemon/tests/acceptance/mod.rs`
- `hoop-daemon/tests/acceptance/s1_morning_review.rs`
- `hoop-daemon/tests/acceptance/s2_transcript_archaeology.rs`
- `hoop-daemon/tests/acceptance/s3_bead_creation_from_chat.rs`
- `hoop-daemon/tests/acceptance/s4_daemon_restart.rs`
- `hoop-daemon/tests/acceptance/s5_workspace_deleted.rs`
- `hoop-daemon/tests/acceptance/s6_machine_mode.rs`

### Integration tests
- `hoop-daemon/tests/adapter_failover_integration.rs`
- `hoop-daemon/tests/adapter_failover.rs`
- `hoop-daemon/tests/adapter_failover_test.rs`
- `hoop-daemon/tests/agent_turn_audit_trail.rs`
- `hoop-daemon/tests/backup_config_deserialization.rs`
- `hoop-daemon/tests/backup_restore_cycle.rs`
- `hoop-daemon/tests/bead_created_by_hoop_broadcast.rs`
- `hoop-daemon/tests/bead_real_line_deserialization.rs`
- `hoop-daemon/tests/beads_deletion_http.rs`
- `hoop-daemon/tests/beads_deletion_isolation.rs`
- `hoop-daemon/tests/beads_removal_recovery.rs`
- `hoop-daemon/tests/bead_status_deserialization.rs`
- `hoop-daemon/tests/claimed_at_parsing.rs`
- `hoop-daemon/tests/compile_fail_create_only.rs`
- `hoop-daemon/tests/config_field_validation.rs`
- `hoop-daemon/tests/config_reload_audit.rs`
- `hoop-daemon/tests/config_reload_cycle.rs`
- `hoop-daemon/tests/create_only_stub.rs`
- `hoop-daemon/tests/create_stitch_no_auto_submit.rs`
- `hoop-daemon/tests/cross_workspace_blockers.rs`
- `hoop-daemon/tests/disaster_recovery_runbook.rs`
- `hoop-daemon/tests/draft_queue_invariants.rs`
- `hoop-daemon/tests/epoch_sync_invariant.rs`
- `hoop-daemon/tests/filesystem_failure_isolation.rs`
- `hoop-daemon/tests/fix_patterns_integration.rs`
- `hoop-daemon/tests/fleet_notifications_integration.rs`
- `hoop-daemon/tests/golden_transcripts_regression.rs`
- `hoop-daemon/tests/hoop_dies_nothing_notices.rs`
- `hoop-daemon/tests/integration_harness.rs`
- `hoop-daemon/tests/lint_regex_global_state.rs`
- `hoop-daemon/tests/load_test_integration.rs`
- `hoop-daemon/tests/load_test.rs`
- `hoop-daemon/tests/mod.rs`
- `hoop-daemon/tests/multi_operator_concurrency.rs`
- `hoop-daemon/tests/mutation_handler_test.rs`
- `hoop-daemon/tests/needle_events_roundtrip.rs`
- `hoop-daemon/tests/observer_mode_integration.rs`
- `hoop-daemon/tests/orphans_integration.rs`
- `hoop-daemon/tests/output_capture_helpers/mod.rs`
- `hoop-daemon/tests/panic_isolation.rs`
- `hoop-daemon/tests/path_traversal_hardening.rs`
- `hoop-daemon/tests/pattern_query_evaluator_integration.rs`
- `hoop-daemon/tests/performance_budget.rs`
- `hoop-daemon/tests/per_project_redaction_integration.rs`
- `hoop-daemon/tests/phase2_exit_gate.rs`
- `hoop-daemon/tests/privacy_surface_audit.rs`
- `hoop-daemon/tests/projection_file_audit.rs`
- `hoop-daemon/tests/property_invariants.rs`
- `hoop-daemon/tests/protocol_contract.rs`
- `hoop-daemon/tests/pure_functions.rs`
- `hoop-daemon/tests/quarantine_integration.rs`
- `hoop-daemon/tests/reflection_detector_integration.rs`
- `hoop-daemon/tests/risk_patterns_standalone.rs`
- `hoop-daemon/tests/s1_morning_review.rs`
- `hoop-daemon/tests/s2_transcript_archaeology.rs`
- `hoop-daemon/tests/s3_bead_creation_from_chat.rs`
- `hoop-daemon/tests/s4_daemon_restart.rs`
- `hoop-daemon/tests/s5_workspace_deleted.rs`
- `hoop-daemon/tests/secrets_scanner_integration.rs`
- `hoop-daemon/tests/secrets_scanner_parity.rs`
- `hoop-daemon/tests/session_redaction.rs`
- `hoop-daemon/tests/skills_integration.rs`
- `hoop-daemon/tests/skills_quarantine_integration.rs`
- `hoop-daemon/tests/state_projections.rs`
- `hoop-daemon/tests/stderr_stdout_capture.rs`
- `hoop-daemon/tests/stdout_generation_test.rs`
- `hoop-daemon/tests/stdout_verification.rs`
- `hoop-daemon/tests/stitch_percentile_index_integration.rs`
- `hoop-daemon/tests/supervisor_health.rs`
- `hoop-daemon/tests/supervisor_hotreload.rs`
- `hoop-daemon/tests/supervisor_isolation.rs`
- `hoop-daemon/tests/supervisor_restart.rs`
- `hoop-daemon/tests/supervisor_shutdown.rs`
- `hoop-daemon/tests/testrepo_harness_integration.rs`
- `hoop-daemon/tests/testrepo_integration.rs`

### UI tests (invoke_br_* forbidden)
- `hoop-daemon/tests/ui/invoke_br_claim_forbidden.rs`
- `hoop-daemon/tests/ui/invoke_br_close_raw_forbidden.rs`
- `hoop-daemon/tests/ui/invoke_br_depend_forbidden.rs`
- `hoop-daemon/tests/ui/invoke_br_release_forbidden.rs`
- `hoop-daemon/tests/ui/invoke_br_update_forbidden.rs`
- `hoop-daemon/tests/ui/invoke_br_write_forbidden.rs`

### Security & validation
- `hoop-daemon/tests/upload_secrets_scan.rs`
- `hoop-daemon/tests/zero_write_invariant.rs`

### Example/manual test files
- `hoop-daemon/examples/test_bead_parse_manual.rs`
- `hoop-daemon/examples/test_bead_parse.rs`
- `hoop-daemon/examples/test_config_validation.rs`
- `hoop-daemon/examples/test_oauth.rs`
- `hoop-daemon/examples/test_sim.rs`
- `hoop-daemon/examples/test_string_coercion.rs`
- `hoop-daemon/examples/test_tokens.rs`
- `hoop-daemon/examples/test_type_check.rs`
- `hoop-daemon/examples/test_yaml_debug.rs`
- `hoop-daemon/examples/test_yaml_err.rs`
- `hoop-daemon/examples/test_yaml_invalid.rs`
- `hoop-daemon/examples/test_yaml_parsing.rs`

### Load test
- `hoop-daemon/src/load_test.rs`

### Root level
- `hoop-daemon/test_invoke_br_write_availability.rs`

### Phase 5 tests
- `hoop-daemon/tests_phase5/adapter_failover_test.rs`

---

## hoop-mcp (11 files)

### Integration & security tests
- `hoop-mcp/tests/compile_fail_create_only.rs`
- `hoop-mcp/tests/create_only_stub.rs`
- `hoop-mcp/tests/forbidden_worker_steering.rs`
- `hoop-mcp/tests/protocol_contract.rs`
- `hoop-mcp/tests/socket_permissions.rs`

### UI tests (invoke_br_* forbidden)
- `hoop-mcp/tests/ui/invoke_br_claim_forbidden.rs`
- `hoop-mcp/tests/ui/invoke_br_close_raw_forbidden.rs`
- `hoop-mcp/tests/ui/invoke_br_depend_forbidden.rs`
- `hoop-mcp/tests/ui/invoke_br_release_forbidden.rs`
- `hoop-mcp/tests/ui/invoke_br_update_forbidden.rs`
- `hoop-mcp/tests/ui/invoke_br_write_forbidden_under_create_only.rs`

---

## hoop-schema (1 file)

- `hoop-schema/tests/schema_drift.rs`

---

## testrepo (108 files)

### Integration test suite (numbered series)
- `testrepo/tests/example_1.rs` through `testrepo/tests/example_30.rs` (30 files)
- `testrepo/tests/integration_1.rs` through `testrepo/tests/integration_50.rs` (50 files)
- `testrepo/tests/integration/test_01.rs` through `testrepo/tests/integration/test_20.rs` (20 files)

### Additional numbered tests
- `testrepo/tests/integration_21.rs` through `testrepo/tests/integration_49.rs` (29 files)

---

## Root-level tests (7 files)

### Acceptance test suite
- `tests/acceptance/mod.rs`
- `tests/acceptance/s1_morning_review.rs`
- `tests/acceptance/s2_transcript_archaeology.rs`
- `tests/acceptance/s3_bead_creation_from_chat.rs`
- `tests/acceptance/s4_daemon_restart.rs`
- `tests/acceptance/s5_workspace_deleted.rs`
- `tests/acceptance/s6_machine_mode.rs`

### Test utilities
- `tests/cli_test_helpers.rs`
- `tests/mod.rs`

---

## Test File Patterns Used

The inventory was generated by searching for:
- Files ending in `_test.rs`
- Files starting with `test_`
- Files named `tests.rs`
- Files in `tests/` directories
- Files in `examples/` directories with `test_` prefix

---

## Notes

- `testrepo/` contains the largest number of test files (108 files), representing integration test suites
- `hoop-daemon/` has the most diverse test coverage (103 files) including acceptance, integration, security, and UI tests
- Some tests appear in multiple locations (e.g., acceptance tests are mirrored between root-level and `hoop-daemon/tests/acceptance/`)
- The `examples/` directories contain manual test files used for development and debugging
- This inventory serves as the foundation for error message extraction work

---

*Generated: 2026-08-12*
*Bead: bf-25p4c*
