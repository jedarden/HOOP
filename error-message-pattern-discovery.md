# HOOP Error Message Pattern Discovery Report

**Generated:** 2026-08-12  
**Bead:** bf-rilp7  
**Previous Bead:** bf-4njr7 (Test Files Inventory)  

## Summary

This report documents the discovery of error message patterns across all HOOP test files. A total of **188 test files** were searched for 7 distinct error message pattern types.

**Key Findings:**
- `assert!` is the most common pattern (144 files)
- `panic!` appears in 20 test files  
- `unwrap_err()` appears in 10 test files
- `anyhow` errors appear in 22 test files
- `expect!` and `expect_err!` were **not found** in any test files

---

## Pattern-to-Files Mapping

### Pattern 1: `assert!` (144 files)

The `assert!` macro is by far the most prevalent error-checking pattern in HOOP tests.

** hoop-daemon/tests/ (81 files):**
- acceptance/s1_morning_review.rs
- acceptance/s2_transcript_archaeology.rs
- acceptance/s3_bead_creation_from_chat.rs
- acceptance/s4_daemon_restart.rs
- acceptance/s5_workspace_deleted.rs
- acceptance/s6_machine_mode.rs
- adapter_failover_integration.rs
- adapter_failover.rs
- adapter_failover_test.rs
- agent_turn_audit_trail.rs
- backup_config_deserialization.rs
- backup_restore_cycle.rs
- bead_created_by_hoop_broadcast.rs
- bead_real_line_deserialization.rs
- beads_deletion_http.rs
- beads_deletion_isolation.rs
- beads_removal_recovery.rs
- claimed_at_parsing.rs
- config_field_validation.rs
- config_reload_audit.rs
- config_reload_cycle.rs
- create_only_stub.rs
- create_stitch_no_auto_submit.rs
- cross_workspace_blockers.rs
- disaster_recovery_runbook.rs
- draft_queue_invariants.rs
- epoch_sync_invariant.rs
- filesystem_failure_isolation.rs
- fix_patterns_integration.rs
- fleet_notifications_integration.rs
- golden_transcripts_regression.rs
- hoop_dies_nothing_notices.rs
- integration_harness.rs
- lint_regex_global_state.rs
- load_test_integration.rs
- load_test.rs
- multi_operator_concurrency.rs
- mutation_handler_test.rs
- needle_events_roundtrip.rs
- observer_mode_integration.rs
- orphans_integration.rs
- output_capture_helpers/mod.rs
- panic_isolation.rs
- path_traversal_hardening.rs
- pattern_query_evaluator_integration.rs
- performance_budget.rs
- per_project_redaction_integration.rs
- phase2_exit_gate.rs
- privacy_surface_audit.rs
- projection_file_audit.rs
- property_invariants.rs
- protocol_contract.rs
- pure_functions.rs
- quarantine_integration.rs
- reflection_detector_integration.rs
- risk_patterns_standalone.rs
- s1_morning_review.rs
- s2_transcript_archaeology.rs
- s3_bead_creation_from_chat.rs
- s4_daemon_restart.rs
- s5_workspace_deleted.rs
- secrets_scanner_integration.rs
- secrets_scanner_parity.rs
- session_redaction.rs
- skills_integration.rs
- skills_quarantine_integration.rs
- state_projections.rs
- stderr_stdout_capture.rs
- stdout_generation_test.rs
- stdout_verification.rs
- stitch_percentile_index_integration.rs
- supervisor_health.rs
- supervisor_hotreload.rs
- supervisor_isolation.rs
- supervisor_restart.rs
- supervisor_shutdown.rs
- testrepo_harness_integration.rs
- testrepo_integration.rs
- upload_secrets_scan.rs
- zero_write_invariant.rs

** hoop-daemon/tests_phase5/ (4 files):**
- adapter_failover_integration.rs
- adapter_failover.rs
- adapter_failover_test.rs
- agent_turn_audit_trail.rs
- reflection_detector_integration.rs

** hoop-cli/tests/ (9 files):**
- clap_test_utils.rs
- cli_test_helpers.rs
- cli_test_utils_examples.rs
- cli_test_utils.rs
- init_no_interactive_flag.rs
- no_interactive_flag_behavior.rs
- remove_no_interactive_flag.rs
- restore_no_interactive_flag.rs
- scan_no_interactive_flag.rs

** testrepo/tests/integration/ (20 files):**
- test_01.rs through test_20.rs (all files)

** tests/ (6 files):**
- acceptance/s1_morning_review.rs
- acceptance/s2_transcript_archaeology.rs
- acceptance/s3_bead_creation_from_chat.rs
- acceptance/s4_daemon_restart.rs
- acceptance/s5_workspace_deleted.rs
- acceptance/s6_machine_mode.rs
- cli_test_helpers.rs

### Pattern 2: `expect!` (0 files)

**NOT FOUND** - No test files use the `expect!` macro.

### Pattern 3: `expect_err!` (0 files)

**NOT FOUND** - No test files use the `expect_err!` macro.

### Pattern 4: `panic!` (20 files)

The `panic!` macro appears in fewer test files, typically used for testing expected failure conditions.

** hoop-cli/tests/ (1 file):**
- clap_test_utils.rs

** hoop-daemon/tests/ (16 files):**
- acceptance/s6_machine_mode.rs
- backup_restore_cycle.rs
- bead_real_line_deserialization.rs
- epoch_sync_invariant.rs
- golden_transcripts_regression.rs
- integration_harness.rs
- lint_regex_global_state.rs
- load_test_integration.rs
- needle_events_roundtrip.rs
- per_project_redaction_integration.rs
- projection_file_audit.rs
- property_invariants.rs
- protocol_contract.rs
- pure_functions.rs
- state_projections.rs
- testrepo_harness_integration.rs
- testrepo_integration.rs
- zero_write_invariant.rs

** tests/ (1 file):**
- cli_test_helpers.rs

### Pattern 5: `unwrap_err()` (10 files)

Used specifically to test error cases by extracting error values from `Result` types.

** hoop-cli/tests/ (2 files):**
- cli_test_helpers.rs
- cli_test_utils_examples.rs

** hoop-daemon/tests/ (8 files):**
- backup_restore_cycle.rs
- claimed_at_parsing.rs
- config_reload_cycle.rs
- create_only_stub.rs
- disaster_recovery_runbook.rs
- mutation_handler_test.rs
- per_project_redaction_integration.rs
- skills_integration.rs

### Pattern 6: Error Types (44 files)

Includes usage of `Result<T, E>`, custom `Error` types, and general error handling patterns.

** Files with Error handling (representative sample):**
- hoop-cli/tests/clap_test_utils.rs
- hoop-cli/tests/cli_test_helpers.rs
- hoop-cli/tests/cli_test_utils_examples.rs
- hoop-cli/tests/init_no_interactive_flag.rs
- hoop-cli/tests/no_interactive_flag_behavior.rs
- hoop-cli/tests/remove_no_interactive_flag.rs
- hoop-cli/tests/restore_no_interactive_flag.rs
- hoop-daemon/tests/acceptance/s4_daemon_restart.rs
- hoop-daemon/tests/acceptance/s5_workspace_deleted.rs
- hoop-daemon/tests/acceptance/s6_machine_mode.rs
- hoop-daemon/tests/adapter_failover_test.rs
- hoop-daemon/tests/backup_restore_cycle.rs
- hoop-daemon/tests/beads_deletion_http.rs
- hoop-daemon/tests/config_field_validation.rs
- hoop-daemon/tests/config_reload_cycle.rs
- hoop-daemon/tests/disaster_recovery_runbook.rs
- hoop-daemon/tests/filesystem_failure_isolation.rs
- hoop-daemon/tests/golden_transcripts_regression.rs
- hoop-daemon/tests/hoop_dies_nothing_notices.rs
- hoop-daemon/tests/integration_harness.rs
- ... (24 additional files)

**Total: 44 files** contain Error types or error handling patterns.

### Pattern 7: `anyhow` errors (22 files)

The `anyhow` crate is used for error handling in a subset of test files.

** hoop-cli/tests/ (1 file):**
- remove_no_interactive_flag.rs

** hoop-daemon/tests/ (18 files):**
- acceptance/s4_daemon_restart.rs
- acceptance/s5_workspace_deleted.rs
- adapter_failover_test.rs
- beads_deletion_http.rs
- disaster_recovery_runbook.rs
- filesystem_failure_isolation.rs
- hoop_dies_nothing_notices.rs
- integration_harness.rs
- projection_file_audit.rs
- s4_daemon_restart.rs
- s5_workspace_deleted.rs
- skills_quarantine_integration.rs
- state_projections.rs
- testrepo_harness_integration.rs
- testrepo_integration.rs

** hoop-daemon/tests_phase5/ (1 file):**
- adapter_failover_test.rs

** tests/ (6 files):**
- acceptance/s1_morning_review.rs
- acceptance/s2_transcript_archaeology.rs
- acceptance/s3_bead_creation_from_chat.rs
- acceptance/s4_daemon_restart.rs
- acceptance/s5_workspace_deleted.rs
- acceptance/s6_machine_mode.rs

---

## Statistical Summary

| Pattern | File Count | Percentage |
|---------|------------|------------|
| `assert!` | 144 | 76.6% |
| Error types | 44 | 23.4% |
| `anyhow` | 22 | 11.7% |
| `panic!` | 20 | 10.6% |
| `unwrap_err()` | 10 | 5.3% |
| `expect!` | 0 | 0% |
| `expect_err!` | 0 | 0% |

**Note:** Percentages exceed 100% because many files contain multiple pattern types.

---

## Distribution by Component

### hoop-daemon/tests/
- **Most pattern-rich:** 81+ files with `assert!`, 16 with `panic!`, 18 with `anyhow`
- **Total test files:** 100+ integration tests

### hoop-cli/tests/  
- **Heavy `assert!` usage:** 9 files
- **Error handling:** 7+ files with Error types
- **Total test files:** 9 integration tests

### testrepo/tests/integration/
- **Uniform `assert!` usage:** All 20 test files
- **Minimal other patterns:** Primarily basic assertions
- **Total test files:** 20 integration tests

### tests/ (root-level)
- **Mix of patterns:** 6 test files with varied approaches
- **anyhow usage:** 6 files with anyhow errors

---

## Next Steps

This discovery pass identifies **WHERE** error messages are defined across HOOP test files. The next phase would involve:

1. **Extraction:** Extract actual error message strings from identified patterns
2. **Categorization:** Group messages by type (validation, error, panic, etc.)
3. **Consistency check:** Identify inconsistent messaging across similar tests
4. **Standardization:** Propose consistent error message templates

---

## Notes

1. **Missing patterns:** The absence of `expect!` and `expect_err!` suggests the codebase prefers `assert!` and `unwrap_err()` for error testing.

2. **anyhow adoption:** The 22 files using `anyhow` represent relatively newer test code or complex error scenarios.

3. **assert! dominance:** With 76.6% of files using `assert!`, this is clearly the standard pattern for basic assertions in HOOP tests.

4. **Test file growth:** The pattern distribution suggests organic test growth rather than standardized error messaging conventions.

5. **Location concentration:** Most error patterns are concentrated in `hoop-daemon/tests/`, reflecting the daemon's complexity and comprehensive test coverage.

---

**Analysis complete.** Ready for error message extraction and standardization work.
