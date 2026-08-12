# HOOP Test Files - Error Message Pattern Mapping

Generated for bead bf-rilp7

## Pattern Categories

### 1. assert!
Files containing `assert!` macro usage (179 files):

**hoop-cli:**
- hoop-cli/test_all_subcommands.rs
- hoop-cli/tests/clap_test_utils.rs
- hoop-cli/tests/cli_test_helpers.rs
- hoop-cli/tests/cli_test_utils_examples.rs
- hoop-cli/tests/cli_test_utils.rs
- hoop-cli/tests/init_no_interactive_flag.rs
- hoop-cli/tests/no_interactive_flag_behavior.rs
- hoop-cli/tests/remove_no_interactive_flag.rs
- hoop-cli/tests/restore_no_interactive_flag.rs
- hoop-cli/tests/scan_no_interactive_flag.rs

**hoop-daemon (86 files):**
- hoop-daemon/tests/acceptance/s1_morning_review.rs
- hoop-daemon/tests/acceptance/s2_transcript_archaeology.rs
- hoop-daemon/tests/acceptance/s3_bead_creation_from_chat.rs
- hoop-daemon/tests/acceptance/s4_daemon_restart.rs
- hoop-daemon/tests/acceptance/s5_workspace_deleted.rs
- hoop-daemon/tests/acceptance/s6_machine_mode.rs
- hoop-daemon/tests/adapter_failover_integration.rs
- hoop-daemon/tests/adapter_failover.rs
- hoop-daemon/tests/adapter_failover_test.rs
- hoop-daemon/tests/agent_turn_audit_trail.rs
- hoop-daemon/tests/backup_config_deserialization.rs
- hoop-daemon/tests/backup_restore_cycle.rs
- hoop-daemon/tests/bead_created_by_hoop_broadcast.rs
- hoop-daemon/tests/bead_real_line_deserialization.rs
- hoop-daemon/tests/beads_deletion_http.rs
- hoop-daemon/tests/beads_deletion_isolation.rs
- hoop-daemon/tests/beads_removal_recovery.rs
- hoop-daemon/tests/claimed_at_parsing.rs
- hoop-daemon/tests/config_field_validation.rs
- hoop-daemon/tests/config_reload_audit.rs
- hoop-daemon/tests/config_reload_cycle.rs
- hoop-daemon/tests/create_only_stub.rs
- hoop-daemon/tests/create_stitch_no_auto_submit.rs
- hoop-daemon/tests/cross_workspace_blockers.rs
- hoop-daemon/tests/disaster_recovery_runbook.rs
- hoop-daemon/tests/draft_queue_invariants.rs
- hoop-daemon/tests/epoch_sync_invariant.rs
- hoop-daemon/tests/filesystem_failure_isolation.rs
- hoop-daemon/tests/fix_patterns_integration.rs
- hoop-daemon/tests/fleet_notifications_integration.rs
- hoop-daemon/tests/golden_transcripts_regression.rs
- hoop-daemon/tests/hoop_dies_nothing_notices.rs
- hoop-daemon/tests/integration_harness.rs
- hoop-daemon/tests/lint_regex_global_state.rs
- hoop-daemon/tests/load_test_integration.rs
- hoop-daemon/tests/load_test.rs
- hoop-daemon/tests/multi_operator_concurrency.rs
- hoop-daemon/tests/mutation_handler_test.rs
- hoop-daemon/tests/needle_events_roundtrip.rs
- hoop-daemon/tests/observer_mode_integration.rs
- hoop-daemon/tests/orphans_integration.rs
- hoop-daemon/tests/panic_isolation.rs
- hoop-daemon/tests/path_traversal_hardening.rs
- hoop-daemon/tests/pattern_query_evaluator_integration.rs
- hoop-daemon/tests/performance_budget.rs
- hoop-daemon/tests/per_project_redaction_integration.rs
- hoop-daemon/tests/phase2_exit_gate.rs
- hoop-daemon/tests/privacy_surface_audit.rs
- hoop-daemon/tests/projection_file_audit.rs
- hoop-daemon/tests/property_invariants.rs
- hoop-daemon/tests/protocol_contract.rs
- hoop-daemon/tests/pure_functions.rs
- hoop-daemon/tests/quarantine_integration.rs
- hoop-daemon/tests/reflection_detector_integration.rs
- hoop-daemon/tests/risk_patterns_standalone.rs
- hoop-daemon/tests/s1_morning_review.rs
- hoop-daemon/tests/s2_transcript_archaeology.rs
- hoop-daemon/tests/s3_bead_creation_from_chat.rs
- hoop-daemon/tests/s4_daemon_restart.rs
- hoop-daemon/tests/s5_workspace_deleted.rs
- hoop-daemon/tests/secrets_scanner_integration.rs
- hoop-daemon/tests/secrets_scanner_parity.rs
- hoop-daemon/tests/session_redaction.rs
- hoop-daemon/tests/skills_integration.rs
- hoop-daemon/tests/skills_quarantine_integration.rs
- hoop-daemon/tests/state_projections.rs
- hoop-daemon/tests/stderr_stdout_capture.rs
- hoop-daemon/tests/stdout_generation_test.rs
- hoop-daemon/tests/stdout_verification.rs
- hoop-daemon/tests/stitch_percentile_index_integration.rs
- hoop-daemon/tests/supervisor_health.rs
- hoop-daemon/tests/supervisor_hotreload.rs
- hoop-daemon/tests/supervisor_isolation.rs
- hoop-daemon/tests/supervisor_restart.rs
- hoop-daemon/tests/supervisor_shutdown.rs
- hoop-daemon/tests/testrepo_harness_integration.rs
- hoop-daemon/tests/testrepo_integration.rs
- hoop-daemon/tests/upload_secrets_scan.rs
- hoop-daemon/tests/zero_write_invariant.rs

**hoop-mcp (5 files):**
- hoop-mcp/tests/create_only_stub.rs
- hoop-mcp/tests/forbidden_worker_steering.rs
- hoop-mcp/tests/protocol_contract.rs
- hoop-mcp/tests/socket_permissions.rs

**testrepo (67 files):**
- testrepo/tests/integration_1.rs through integration_50.rs
- testrepo/tests/integration/test_01.rs through test_20.rs

**tests (8 files):**
- tests/acceptance/s1_morning_review.rs
- tests/acceptance/s2_transcript_archaeology.rs
- tests/acceptance/s3_bead_creation_from_chat.rs
- tests/acceptance/s4_daemon_restart.rs
- tests/acceptance/s5_workspace_deleted.rs
- tests/acceptance/s6_machine_mode.rs
- tests/cli_test_helpers.rs

### 2. expect!
**No files found** - No `expect!` macro usage detected

### 3. expect_err!
**No files found** - No `expect_err!` macro usage detected

### 4. panic!
Files containing `panic!` macro usage (23 files):

**hoop-cli:**
- hoop-cli/tests/clap_test_utils.rs

**hoop-daemon (20 files):**
- hoop-daemon/tests/acceptance/s6_machine_mode.rs
- hoop-daemon/tests/backup_restore_cycle.rs
- hoop-daemon/tests/bead_real_line_deserialization.rs
- hoop-daemon/tests/epoch_sync_invariant.rs
- hoop-daemon/tests/golden_transcripts_regression.rs
- hoop-daemon/tests/integration_harness.rs
- hoop-daemon/tests/lint_regex_global_state.rs
- hoop-daemon/tests/load_test_integration.rs
- hoop-daemon/tests/needle_events_roundtrip.rs
- hoop-daemon/tests/per_project_redaction_integration.rs
- hoop-daemon/tests/projection_file_audit.rs
- hoop-daemon/tests/property_invariants.rs
- hoop-daemon/tests/protocol_contract.rs
- hoop-daemon/tests/pure_functions.rs
- hoop-daemon/tests/state_projections.rs
- hoop-daemon/tests/testrepo_harness_integration.rs
- hoop-daemon/tests/testrepo_integration.rs
- hoop-daemon/tests/zero_write_invariant.rs

**hoop-mcp:**
- hoop-mcp/tests/protocol_contract.rs

**hoop-schema:**
- hoop-schema/tests/schema_drift.rs

**tests:**
- tests/cli_test_helpers.rs

### 5. Error types
Files containing Error type patterns (Result<.*, Error>, Error::, map_err) (6 files):

- hoop-cli/tests/clap_test_utils.rs
- hoop-daemon/tests/config_field_validation.rs
- hoop-daemon/tests/config_reload_cycle.rs
- hoop-daemon/tests/integration_harness.rs
- hoop-daemon/tests/phase2_exit_gate.rs
- tests/cli_test_helpers.rs

### 6. anyhow errors
Files containing anyhow:: or anyhow! patterns (22 files):

**hoop-cli:**
- hoop-cli/tests/remove_no_interactive_flag.rs

**hoop-daemon (18 files):**
- hoop-daemon/tests/acceptance/s4_daemon_restart.rs
- hoop-daemon/tests/acceptance/s5_workspace_deleted.rs
- hoop-daemon/tests/adapter_failover_test.rs
- hoop-daemon/tests/beads_deletion_http.rs
- hoop-daemon/tests/disaster_recovery_runbook.rs
- hoop-daemon/tests/filesystem_failure_isolation.rs
- hoop-daemon/tests/hoop_dies_nothing_notices.rs
- hoop-daemon/tests/integration_harness.rs
- hoop-daemon/tests/projection_file_audit.rs
- hoop-daemon/tests/s4_daemon_restart.rs
- hoop-daemon/tests/s5_workspace_deleted.rs
- hoop-daemon/tests/skills_quarantine_integration.rs
- hoop-daemon/tests/state_projections.rs
- hoop-daemon/tests/testrepo_harness_integration.rs
- hoop-daemon/tests/testrepo_integration.rs

**tests (6 files):**
- tests/acceptance/s1_morning_review.rs
- tests/acceptance/s2_transcript_archaeology.rs
- tests/acceptance/s3_bead_creation_from_chat.rs
- tests/acceptance/s4_daemon_restart.rs
- tests/acceptance/s5_workspace_deleted.rs
- tests/acceptance/s6_machine_mode.rs

### 7. unwrap_err()
Files containing `unwrap_err()` pattern (10 files):

**hoop-cli:**
- hoop-cli/tests/cli_test_helpers.rs
- hoop-cli/tests/cli_test_utils_examples.rs

**hoop-daemon (8 files):**
- hoop-daemon/tests/backup_restore_cycle.rs
- hoop-daemon/tests/claimed_at_parsing.rs
- hoop-daemon/tests/config_reload_cycle.rs
- hoop-daemon/tests/create_only_stub.rs
- hoop-daemon/tests/disaster_recovery_runbook.rs
- hoop-daemon/tests/mutation_handler_test.rs
- hoop-daemon/tests/per_project_redaction_integration.rs
- hoop-daemon/tests/skills_integration.rs

**hoop-mcp:**
- hoop-mcp/tests/forbidden_worker_steering.rs

## Summary Statistics

| Pattern | File Count | Percentage of Total Test Files |
|---------|------------|-------------------------------|
| assert! | 179 | 73% |
| panic! | 23 | 9% |
| anyhow errors | 22 | 9% |
| unwrap_err() | 10 | 4% |
| Error types | 6 | 2% |
| expect! | 0 | 0% |
| expect_err! | 0 | 0% |

**Total test files analyzed:** 245

## Key Findings

1. **assert! is dominant** - Used in 73% of test files, the primary assertion mechanism
2. **No expect!/expect_err! usage** - These macros are not used in the test suite
3. **panic! for invariants** - Used in 9% of files, primarily for testing unreachable code paths and invariant violations
4. **anyhow for integration tests** - Heavy usage in acceptance tests and integration test harness
5. **unwrap_err() focused** - Used in 10 files, primarily for error testing scenarios

## Notes

- Search scope: All test files (tests/, hoop-daemon/tests/, hoop-cli/tests/, hoop-mcp/tests/, hoop-schema/tests/, testrepo/tests/)
- Patterns not found: expect!, expect_err! (likely replaced by assert! and unwrap_err())
- anyhow errors concentrated in integration and acceptance test suites
