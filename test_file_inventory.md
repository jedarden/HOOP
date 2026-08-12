# HOOP Test File Inventory

**Generated:** 2026-08-12  
**Purpose:** Complete enumeration of all test files in the HOOP project

## Summary Statistics

- **Total standalone test files:** 184+
- **Total source files with embedded unit tests:** 164+
- **Total test locations:** 348+

---

## 1. hoop-daemon Tests (Primary Daemon)

### Integration Tests (`hoop-daemon/tests/`)

#### Acceptance Tests (`tests/acceptance/`)
- `acceptance/mod.rs` - Acceptance test module
- `acceptance/s1_morning_review.rs` - Morning review acceptance test
- `acceptance/s2_transcript_archaeology.rs` - Transcript archaeology acceptance test
- `acceptance/s3_bead_creation_from_chat.rs` - Bead creation from chat acceptance test
- `acceptance/s4_daemon_restart.rs` - Daemon restart acceptance test
- `acceptance/s5_workspace_deleted.rs` - Workspace deleted acceptance test
- `acceptance/s6_machine_mode.rs` - Machine mode acceptance test

#### Adapter & Agent Tests
- `adapter_failover_integration.rs` - Adapter failover integration test
- `adapter_failover.rs` - Adapter failover test
- `adapter_failover_test.rs` - Adapter failover unit test
- `agent_turn_audit_trail.rs` - Agent turn audit trail test

#### Backup & Config Tests
- `backup_config_deserialization.rs` - Backup config deserialization test
- `backup_restore_cycle.rs` - Backup restore cycle test
- `config_field_validation.rs` - Config field validation test
- `config_reload_audit.rs` - Config reload audit test
- `config_reload_cycle.rs` - Config reload cycle test

#### Bead Tests
- `bead_created_by_hoop_broadcast.rs` - Bead creation broadcast test
- `bead_real_line_deserialization.rs` - Bead real line deserialization test
- `bead_status_deserialization.rs` - Bead status deserialization test
- `beads_deletion_http.rs` - Beads deletion via HTTP test
- `beads_deletion_isolation.rs` - Beads deletion isolation test
- `beads_removal_recovery.rs` - Beads removal recovery test
- `claimed_at_parsing.rs` - Claimed-at timestamp parsing test

#### Compile-Fail Tests
- `compile_fail_create_only.rs` - Compile fail test (create-only enforcement)
- `create_only_stub.rs` - Create-only stub test

#### Config & State Tests
- `create_stitch_no_auto_submit.rs` - Create stitch without auto-submit test
- `cross_workspace_blockers.rs` - Cross-workspace blockers test
- `draft_queue_invariants.rs` - Draft queue invariants test

#### Disaster Recovery & Invariants
- `disaster_recovery_runbook.rs` - Disaster recovery runbook test
- `epoch_sync_invariant.rs` - Epoch sync invariant test
- `filesystem_failure_isolation.rs` - Filesystem failure isolation test
- `property_invariants.rs` - Property invariants test

#### Fix Patterns & Detection
- `fix_patterns_integration.rs` - Fix patterns integration test
- `reflection_detector_integration.rs` - Reflection detector integration test
- `risk_patterns_standalone.rs` - Risk patterns standalone test

#### Fleet & Worker Tests
- `fleet_notifications_integration.rs` - Fleet notifications integration test
- `hoop_dies_nothing_notices.rs` - HOOP daemon death isolation test

#### Golden Master & Regression
- `golden_transcripts_regression.rs` - Golden transcripts regression test

#### Harness & Helpers
- `integration_harness.rs` - Integration test harness
- `mod.rs` - Integration tests module

#### Lint & Regex Tests
- `lint_regex_global_state.rs` - Lint regex global state test

#### Load Testing
- `load_test_integration.rs` - Load test integration test
- `load_test.rs` - Load test (duplicate)
- `src/load_test.rs` - Load test source

#### Concurrency & Isolation
- `multi_operator_concurrency.rs` - Multi-operator concurrency test
- `panic_isolation.rs` - Panic isolation test

#### Mutation & Protocol Tests
- `mutation_handler_test.rs` - Mutation handler test
- `protocol_contract.rs` - Protocol contract test

#### Needle Integration
- `needle_events_roundtrip.rs` - Needle events roundtrip test

#### Observer Mode
- `observer_mode_integration.rs` - Observer mode integration test

#### Orphans & Recovery
- `orphans_integration.rs` - Orphans integration test

#### Output Capture
- `output_capture_helpers/mod.rs` - Output capture helpers module
- `stderr_stdout_capture.rs` - Stderr/stdout capture test
- `stdout_generation_test.rs` - Stdout generation test
- `stdout_verification.rs` - Stdout verification test

#### Path Security
- `path_traversal_hardening.rs` - Path traversal hardening test

#### Performance
- `performance_budget.rs` - Performance budget test

#### Phase Gates
- `phase2_exit_gate.rs` - Phase 2 exit gate test

#### Privacy & Redaction
- `per_project_redaction_integration.rs` - Per-project redaction integration test
- `privacy_surface_audit.rs` - Privacy surface audit test
- `session_redaction.rs` - Session redaction test

#### Projections & State
- `projection_file_audit.rs` - Projection file audit test
- `state_projections.rs` - State projections test

#### Pure Functions
- `pure_functions.rs` - Pure functions test

#### Quarantine
- `quarantine_integration.rs` - Quarantine integration test

#### Secrets Scanner
- `secrets_scanner_integration.rs` - Secrets scanner integration test
- `secrets_scanner_parity.rs` - Secrets scanner parity test
- `upload_secrets_scan.rs` - Upload secrets scan test

#### Skills
- `skills_integration.rs` - Skills integration test
- `skills_quarantine_integration.rs` - Skills quarantine integration test

#### Stitch & Index Tests
- `stitch_percentile_index_integration.rs` - Stitch percentile index integration test

#### Supervisor Tests
- `supervisor_health.rs` - Supervisor health test
- `supervisor_hotreload.rs` - Supervisor hot-reload test
- `supervisor_isolation.rs` - Supervisor isolation test
- `supervisor_restart.rs` - Supervisor restart test
- `supervisor_shutdown.rs` - Supervisor shutdown test

#### Testrepo Integration
- `testrepo_harness_integration.rs` - Testrepo harness integration test
- `testrepo_integration.rs` - Testrepo integration test

#### UI Forbidden Action Tests (`tests/ui/`)
- `ui/invoke_br_claim_forbidden.rs` - BR claim forbidden test
- `ui/invoke_br_close_raw_forbidden.rs` - BR close_raw forbidden test
- `ui/invoke_br_depend_forbidden.rs` - BR depend forbidden test
- `ui/invoke_br_release_forbidden.rs` - BR release forbidden test
- `ui/invoke_br_update_forbidden.rs` - BR update forbidden test
- `ui/invoke_br_write_forbidden.rs` - BR write forbidden test

#### Write Invariants
- `zero_write_invariant.rs` - Zero write invariant test

---

## 2. hoop-cli Tests (CLI Client)

### Integration Tests (`hoop-cli/tests/`)

#### Test Utilities
- `clap_test_utils.rs` - CLAP test utilities
- `cli_test_helpers.rs` - CLI test helpers
- `cli_test_utils_examples.rs` - CLI test utilities examples
- `cli_test_utils.rs` - CLI test utilities

#### No-Interactive Flag Tests
- `init_no_interactive_flag.rs` - Init no-interactive flag test
- `no_interactive_flag_behavior.rs` - No-interactive flag behavior test
- `remove_no_interactive_flag.rs` - Remove no-interactive flag test
- `restore_no_interactive_flag.rs` - Restore no-interactive flag test
- `scan_no_interactive_flag.rs` - Scan no-interactive flag test

---

## 3. hoop-mcp Tests (MCP Server)

### Integration Tests (`hoop-mcp/tests/`)

#### Compile-Fail & Enforcement
- `compile_fail_create_only.rs` - Compile fail test (create-only enforcement)
- `create_only_stub.rs` - Create-only stub test

#### Forbidden Actions
- `forbidden_worker_steering.rs` - Worker steering forbidden test

#### Protocol & Contract
- `protocol_contract.rs` - Protocol contract test

#### Socket Permissions
- `socket_permissions.rs` - Socket permissions test

#### UI Forbidden Action Tests (`tests/ui/`)
- `ui/invoke_br_claim_forbidden.rs` - BR claim forbidden test
- `ui/invoke_br_close_raw_forbidden.rs` - BR close_raw forbidden test
- `ui/invoke_br_depend_forbidden.rs` - BR depend forbidden test
- `ui/invoke_br_release_forbidden.rs` - BR release forbidden test
- `ui/invoke_br_update_forbidden.rs` - BR update forbidden test
- `ui/invoke_br_write_forbidden_under_create_only.rs` - BR write forbidden under create-only test

---

## 4. hoop-schema Tests (Schema Validation)

### Integration Tests (`hoop-schema/tests/`)
- `schema_drift.rs` - Schema drift detection test

---

## 5. testrepo Tests (Test Repository Fixtures)

### Example Tests (`testrepo/tests/`)
- `example_1.rs` through `example_30.rs` - Example test fixtures (30 files)

### Integration Tests (`testrepo/tests/integration/`)
- `integration_1.rs` through `integration_50.rs` - Integration test fixtures (50 files)
- `test_01.rs` through `test_20.rs` - Integration test fixtures (20 files)

---

## 6. Root-Level Tests

### Acceptance Tests (`tests/acceptance/`)
- `acceptance/mod.rs` - Acceptance test module
- `acceptance/s1_morning_review.rs` - Morning review acceptance test
- `acceptance/s2_transcript_archaeology.rs` - Transcript archaeology acceptance test
- `acceptance/s3_bead_creation_from_chat.rs` - Bead creation from chat acceptance test
- `acceptance/s4_daemon_restart.rs` - Daemon restart acceptance test
- `acceptance/s5_workspace_deleted.rs` - Workspace deleted acceptance test
- `acceptance/s6_machine_mode.rs` - Machine mode acceptance test

### Test Utilities
- `cli_test_helpers.rs` - CLI test helpers
- `mod.rs` - Tests module

---

## 7. Source Files with Embedded Unit Tests

### hoop-daemon Source Files (`hoop-daemon/src/`)

**164 source files** contain embedded `#[cfg(test)]` modules, including:
- All API handlers (`api_*.rs`)
- All core business logic (agent, beads, events, patterns, etc.)
- All utility modules (redaction, crypto, parsing, etc.)
- Integration support modules

### hoop-cli Source Files (`hoop-cli/src/`)

**9 source files** with embedded tests:
- `backup.rs`, `init.rs`, `main.rs`, `new.rs`, `projects.rs`, `reflection.rs`, `restore.rs`, `risk_patterns.rs`

### hoop-mcp Source Files (`hoop-mcp/src/`)

**7 source files** with embedded tests:
- `audit.rs`, `br_verbs.rs`, `notes.rs`, `redaction.rs`, `skills.rs`, `socket.rs`

### hoop-schema Source Files (`hoop-schema/src/`)

**4 source files** with embedded tests:
- `effort.rs`, `id_validators.rs`, `lib.rs`, `path_security.rs`

### testrepo Source Files (`testrepo/src/`)

**80+ source files** with embedded tests covering:
- API handlers (GraphQL, REST, SSE, WebSocket)
- CLI commands
- Core services
- Models
- Storage backends
- Utilities

---

## Test Categories by Purpose

### Acceptance Tests (16)
- End-to-end workflows testing complete scenarios
- Located in `tests/acceptance/` and `hoop-daemon/tests/acceptance/`

### Integration Tests (100+)
- Component interaction tests
- API contract tests
- State management tests

### Unit Tests (164+ embedded)
- Module-level tests in source files
- Function-level behavior verification

### Property-Based / Invariant Tests (15+)
- `property_invariants.rs`
- `zero_write_invariant.rs`
- `epoch_sync_invariant.rs`
- `draft_queue_invariants.rs`

### Regression Tests (10+)
- `golden_transcripts_regression.rs`
- `schema_drift.rs`
- Various `*_regression.rs` tests

### Performance Tests (5+)
- `load_test.rs`, `load_test_integration.rs`
- `performance_budget.rs`

### Security & Hardening Tests (12+)
- `path_traversal_hardening.rs`
- `forbidden_worker_steering.rs`
- `upload_secrets_scan.rs`
- `socket_permissions.rs`
- All `ui/invoke_*_forbidden.rs` tests

### Compile-Fail Tests (4)
- Verify compilation enforces create-only invariant
- `compile_fail_create_only.rs` (daemon & MCP)

### Disaster Recovery Tests (5+)
- `disaster_recovery_runbook.rs`
- `backup_restore_cycle.rs`
- `supervisor_restart.rs`

---

## Notes

1. **Duplicate test files:** Some tests appear in multiple locations (e.g., `load_test.rs` appears in both `hoop-daemon/tests/` and `hoop-daemon/src/`)

2. **Fixture repository:** The `testrepo/` directory contains 100+ test fixtures used for integration testing

3. **UI tests:** Forbidden action tests verify that HOOP cannot perform operations outside its design (e.g., steering NEEDLE workers)

4. **Acceptance test suite:** The `s1_` through `s6_` prefixed tests represent the core acceptance scenarios from the implementation plan

---

**End of Inventory**
