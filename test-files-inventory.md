# HOOP Test Files Inventory

Generated: 2026-08-12
Bead: bf-4njr7

## Summary Statistics
- **Total integration test files**: 39
- **Total source files with unit tests**: 143
- **Total test utilities/helpers**: 6
- **Grand total test files**: 188

---

## I. Integration Test Files (standalone `tests/` directories)

### hoop-daemon/tests/ (11 files)
```
hoop-daemon/tests/adapter_failover_test.rs
hoop-daemon/tests/load_test_integration.rs
hoop-daemon/tests/load_test.rs
hoop-daemon/tests/mutation_handler_test.rs
hoop-daemon/tests/stdout_generation_test.rs
hoop-daemon/tests/testrepo_harness_integration.rs
hoop-daemon/tests/testrepo_integration.rs
```

### hoop-daemon/tests_phase5/ (1 file)
```
hoop-daemon/tests_phase5/adapter_failover_test.rs
```

### testrepo/tests/integration/ (20 files)
```
testrepo/tests/integration/test_01.rs
testrepo/tests/integration/test_02.rs
testrepo/tests/integration/test_03.rs
testrepo/tests/integration/test_04.rs
testrepo/tests/integration/test_05.rs
testrepo/tests/integration/test_06.rs
testrepo/tests/integration/test_07.rs
testrepo/tests/integration/test_08.rs
testrepo/tests/integration/test_09.rs
testrepo/tests/integration/test_10.rs
testrepo/tests/integration/test_11.rs
testrepo/tests/integration/test_12.rs
testrepo/tests/integration/test_13.rs
testrepo/tests/integration/test_14.rs
testrepo/tests/integration/test_15.rs
testrepo/tests/integration/test_16.rs
testrepo/tests/integration/test_17.rs
testrepo/tests/integration/test_18.rs
testrepo/tests/integration/test_19.rs
testrepo/tests/integration/test_20.rs
```

### hoop-cli/tests/ (3 files)
```
hoop-cli/tests/clap_test_utils.rs
hoop-cli/tests/cli_test_helpers.rs
hoop-cli/tests/cli_test_utils_examples.rs
hoop-cli/tests/cli_test_utils.rs
```

### root-level tests/ (1 file)
```
tests/cli_test_helpers.rs
```

### hoop-daemon/test invoke files (1 file)
```
hoop-daemon/test_invoke_br_write_availability.rs
```

---

## II. Unit Test Files (embedded in `src/` with `#[cfg(test)]`)

### hoop-daemon/src/ (109 files with unit tests)
```
hoop-daemon/src/accounts_config.rs
hoop-daemon/src/agent_adapter.rs
hoop-daemon/src/agent_context.rs
hoop-daemon/src/agent_session.rs
hoop-daemon/src/ansi_strip.rs
hoop-daemon/src/api_bead_files.rs
hoop-daemon/src/api_beads.rs
hoop-daemon/src/api_blame.rs
hoop-daemon/src/api_bulk_create.rs
hoop-daemon/src/api_config.rs
hoop-daemon/src/api_cost_per_stitch.rs
hoop-daemon/src/api_diff.rs
hoop-daemon/src/api_embedding.rs
hoop-daemon/src/api_files.rs
hoop-daemon/src/api_metrics.rs
hoop-daemon/src/api_notes.rs
hoop-daemon/src/api_onboarding.rs
hoop-daemon/src/api_orphans.rs
hoop-daemon/src/api_preview.rs
hoop-daemon/src/api_prompts.rs
hoop-daemon/src/api_risk_patterns.rs
hoop-daemon/src/api_scripts.rs
hoop-daemon/src/api_skills.rs
hoop-daemon/src/api_stitch_decompose.rs
hoop-daemon/src/api_stitch_read.rs
hoop-daemon/src/api_stitch_replay.rs
hoop-daemon/src/api_tour_project.rs
hoop-daemon/src/api_ui_state.rs
hoop-daemon/src/atomic_write.rs
hoop-daemon/src/attachments.rs
hoop-daemon/src/attachment_sync.rs
hoop-daemon/src/audio_redaction.rs
hoop-daemon/src/audit.rs
hoop-daemon/src/auth.rs
hoop-daemon/src/backup_pipeline.rs
hoop-daemon/src/backup.rs
hoop-daemon/src/bead_commit_index.rs
hoop-daemon/src/beads.rs
hoop-daemon/src/br_verbs.rs
hoop-daemon/src/capacity.rs
hoop-daemon/src/collision_detector.rs
hoop-daemon/src/config_backup.rs
hoop-daemon/src/config_resolver.rs
hoop-daemon/src/config_watcher.rs
hoop-daemon/src/content_blocks.rs
hoop-daemon/src/cost_anomaly.rs
hoop-daemon/src/cost.rs
hoop-daemon/src/cross_project_propagation.rs
hoop-daemon/src/dictated_notes.rs
hoop-daemon/src/embedding_service.rs
hoop-daemon/src/events.rs
hoop-daemon/src/file_io_error.rs
hoop-daemon/src/files.rs
hoop-daemon/src/fix_patterns.rs
hoop-daemon/src/fleet_notifications.rs
hoop-daemon/src/fleet.rs
hoop-daemon/src/heartbeats.rs
hoop-daemon/src/identity.rs
hoop-daemon/src/integration_harness.rs
hoop-daemon/src/integration_test_client.rs
hoop-daemon/src/lib.rs
hoop-daemon/src/load_test.rs
hoop-daemon/src/log_rotation.rs
hoop-daemon/src/metrics.rs
hoop-daemon/src/migrations.rs
hoop-daemon/src/morning_brief.rs
hoop-daemon/src/mutation_handler.rs
hoop-daemon/src/net_diff.rs
hoop-daemon/src/orphan_beads.rs
hoop-daemon/src/parse_jsonl_safe.rs
hoop-daemon/src/path_security.rs
hoop-daemon/src/pattern_query_evaluator.rs
hoop-daemon/src/pdf_sanitize.rs
hoop-daemon/src/predictor.rs
hoop-daemon/src/pricing_watcher.rs
hoop-daemon/src/projects.rs
hoop-daemon/src/prompt_substitute.rs
hoop-daemon/src/redaction_policy.rs
hoop-daemon/src/redaction.rs
hoop-daemon/src/reflection_detector.rs
hoop-daemon/src/risk_patterns.rs
hoop-daemon/src/saturation_detector.rs
hoop-daemon/src/screen_capture.rs
hoop-daemon/src/script_scheduler.rs
hoop-daemon/src/script_trigger.rs
hoop-daemon/src/secrets_scanner.rs
hoop-daemon/src/sessions.rs
hoop-daemon/src/shutdown.rs
hoop-daemon/src/similarity.rs
hoop-daemon/src/snapshot_manifest.rs
hoop-daemon/src/stitch_decompose.rs
hoop-daemon/src/stitch_percentile_index.rs
hoop-daemon/src/stitch_reconstruction.rs
hoop-daemon/src/stitch_status.rs
hoop-daemon/src/stitch_traversal.rs
hoop-daemon/src/stuck_detector.rs
hoop-daemon/src/supervisor.rs
hoop-daemon/src/svg_sanitize.rs
hoop-daemon/src/syntax_highlight.rs
hoop-daemon/src/syntax_highlight_stream.rs
hoop-daemon/src/tag_join.rs
hoop-daemon/src/template_library.rs
hoop-daemon/src/transcription.rs
hoop-daemon/src/unknown_event_sink.rs
hoop-daemon/src/uploads.rs
hoop-daemon/src/vector_index.rs
hoop-daemon/src/worker_ack.rs
hoop-daemon/src/ws.rs
```

### hoop-cli/src/ (9 files with unit tests)
```
hoop-cli/src/backup.rs
hoop-cli/src/init.rs
hoop-cli/src/main.rs
hoop-cli/src/new.rs
hoop-cli/src/projects.rs
hoop-cli/src/reflection.rs
hoop-cli/src/restore.rs
hoop-cli/src/risk_patterns.rs
```

### testrepo/src/ (101 files with unit tests)
```
testrepo/src/api/graphql.rs
testrepo/src/api/handlers.rs
testrepo/src/api/middleware.rs
testrepo/src/api/rest.rs
testrepo/src/api/routes.rs
testrepo/src/api/sse.rs
testrepo/src/api/websocket.rs
testrepo/src/async/runtime.rs
testrepo/src/async/task.rs
testrepo/src/cli/commands/build.rs
testrepo/src/cli/commands/config.rs
testrepo/src/cli/commands/deploy.rs
testrepo/src/cli/commands/export.rs
testrepo/src/cli/commands/import.rs
testrepo/src/cli/commands/init.rs
testrepo/src/cli/commands/logs.rs
testrepo/src/cli/commands/monitor.rs
testrepo/src/cli/commands/run.rs
testrepo/src/cli/commands/status.rs
testrepo/src/core/auth.rs
testrepo/src/core/cache.rs
testrepo/src/core/config.rs
testrepo/src/core/crypto.rs
testrepo/src/core/db.rs
testrepo/src/core/error.rs
testrepo/src/core/metrics.rs
testrepo/src/core/tracing.rs
testrepo/src/crypto/aes.rs
testrepo/src/crypto/hash.rs
testrepo/src/models/attachment.rs
testrepo/src/models/audit.rs
testrepo/src/models/config.rs
testrepo/src/models/event.rs
testrepo/src/models/log.rs
testrepo/src/models/metric.rs
testrepo/src/models/project.rs
testrepo/src/models/session.rs
testrepo/src/models/task.rs
testrepo/src/models/user.rs
testrepo/src/network/http.rs
testrepo/src/network/tcp.rs
testrepo/src/parsing/csv.rs
testrepo/src/parsing/json.rs
testrepo/src/services/analytics.rs
testrepo/src/services/auth.rs
testrepo/src/services/exporter.rs
testrepo/src/services/indexer.rs
testrepo/src/services/notification.rs
testrepo/src/services/project.rs
testrepo/src/services/scheduler.rs
testrepo/src/services/storage.rs
testrepo/src/services/user.rs
testrepo/src/storage/memory.rs
testrepo/src/storage/sql.rs
testrepo/src/utils/crypto.rs
testrepo/src/utils/formatting.rs
testrepo/src/utils/http.rs
testrepo/src/utils/id.rs
testrepo/src/utils/json.rs
testrepo/src/utils/logging.rs
testrepo/src/utils/retry.rs
testrepo/src/utils/time.rs
testrepo/src/utils/validation.rs
```

### hoop-mcp/src/ (6 files with unit tests)
```
hoop-mcp/src/audit.rs
hoop-mcp/src/br_verbs.rs
hoop-mcp/src/notes.rs
hoop-mcp/src/redaction.rs
hoop-mcp/src/skills.rs
hoop-mcp/src/socket.rs
```

### hoop-schema/src/ (4 files with unit tests)
```
hoop-schema/src/effort.rs
hoop-schema/src/id_validators.rs
hoop-schema/src/lib.rs
hoop-schema/src/path_security.rs
```

---

## III. Test Utilities and Test Harness Files

### Test helpers
```
hoop-cli/tests/cli_test_helpers.rs
hoop-cli/tests/cli_test_utils.rs
hoop-cli/tests/cli_test_utils_examples.rs
tests/cli_test_helpers.rs
```

### Test harness infrastructure
```
hoop-daemon/src/integration_test_client.rs
hoop-daemon/src/integration_harness.rs
hoop-daemon/src/load_test.rs
hoop-cli/test_all_subcommands.rs
```

---

## IV. Example/Test Binary Files (hoop-daemon/examples/)

```
hoop-daemon/examples/load-test-runner.rs
hoop-daemon/examples/populate-testrepo.rs
hoop-daemon/examples/test_bead_parse_manual.rs
hoop-daemon/examples/test_bead_parse.rs
hoop-daemon/examples/test_config_validation.rs
hoop-daemon/examples/test_oauth.rs
hoop-daemon/examples/test_sim.rs
hoop-daemon/examples/test_string_coercion.rs
hoop-daemon/examples/test_tokens.rs
hoop-daemon/examples/test_type_check.rs
hoop-daemon/examples/test_yaml_debug.rs
hoop-daemon/examples/test_yaml_err.rs
hoop-daemon/examples/test_yaml_invalid.rs
hoop-daemon/examples/test_yaml_parsing.rs
```

---

## V. Categorization by Module/Component

### Core Daemon (hoop-daemon)
- Integration tests: 11 files
- Unit tests: 109 files with embedded tests
- Test harness: 2 files
- Examples: 13 test binaries

### CLI (hoop-cli)
- Integration tests: 4 files
- Unit tests: 9 files with embedded tests
- Test helpers: 3 files

### MCP Server (hoop-mcp)
- Unit tests: 6 files with embedded tests

### Schema (hoop-schema)
- Unit tests: 4 files with embedded tests

### Test Repository (testrepo)
- Integration tests: 20 files
- Unit tests: 101 files with embedded tests

---

## VI. Test Type Classification

### Integration Tests
- Full daemon integration: `hoop-daemon/tests/` (7 files)
- Phase 5 specific: `hoop-daemon/tests_phase5/` (1 file)
- CLI integration: `hoop-cli/tests/` (4 files)
- Test repository: `testrepo/tests/integration/` (20 files)

### Unit Tests
- Business logic: 109 files in hoop-daemon
- CLI commands: 9 files in hoop-cli
- MCP handlers: 6 files in hoop-mcp
- Schema validation: 4 files in hoop-schema
- Test repo logic: 101 files in testrepo

### Load Tests
- Load test runner: `hoop-daemon/examples/load-test-runner.rs`
- Load test suite: `hoop-daemon/tests/load_test*.rs`

### Test Utilities
- CLI test helpers: Shared test utilities for CLI testing
- Integration harness: Reusable test infrastructure
- Mock/test data: Example binaries for manual testing

---

## Notes

1. **file count**: Total of 188 distinct test files across the project
2. **unit test pattern**: Most business logic has embedded `#[cfg(test)]` modules
3. **integration test pattern**: Standalone test files in `tests/` directories
4. **testrepo**: Contains the most comprehensive test suite (20 integration + 101 unit test files)
5. **load tests**: Dedicated infrastructure for performance testing
6. **examples**: Several example binaries that serve as manual test tools

This inventory serves as the foundation for error message extraction work (bead bf-4njr7).
