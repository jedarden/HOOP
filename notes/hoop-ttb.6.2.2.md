# Adapter Failover Test Implementation (hoop-ttb.6.2.2)

## Task Summary

Integration test simulates Anthropic 5xx → ZAI/GLM switch; session continuity surfaced.

## Acceptance Criteria Status

### ✅ 1. Simulated Anthropic 500 doesn't crash daemon

**Test Coverage:**
- `adapter_failover.rs::test_anthropic_5xx_doesnt_crash_daemon` - Unit test verifying adapter build succeeds after error
- `adapter_failover_test.rs::daemon_survives_simulated_anthropic_5xx` - Integration test with health check
- `adapter_failover_test.rs::simulated_anthropic_5xx_error_allows_recovery_via_adapter_switch` - Mock HTTP server returning 500

### ✅ 2. Operator switches adapter via config.yml edit → hot-reload triggers new session

**Test Coverage:**
- `adapter_failover_test.rs::config_yml_hot_reload_triggers_adapter_switch` - Full end-to-end test that:
  - Edits config.yml to change adapter from claude to zai
  - Waits for hot-reload (4 second debounce + processing)
  - Verifies new adapter is active
  - Verifies old session is archived

### ✅ 3. Old session's final transcript preserved as closed Stitch (kind=operator, archived)

**Test Coverage:**
- `adapter_failover.rs::test_adapter_switch_archives_session_as_stitch` - Unit test verifying Stitch creation
- `adapter_failover.rs::test_archived_stitch_metadata` - Verifies Stitch metadata correctness
- `adapter_failover.rs::test_session_history_round_trip` - Verifies conversation history preservation
- `adapter_failover_test.rs::old_session_transcript_preserved_as_stitch` - Integration test with API verification

### ✅ 4. Reflection Ledger continuity preserved

**Test Coverage:**
- `adapter_failover.rs::test_reflection_ledger_preserved_across_switch` - Verifies approved rules persist
- `adapter_failover.rs::test_handoff_context_includes_reflection_ledger` - Verifies handoff context includes ledger
- `adapter_failover_test.rs::reflection_ledger_continuity_preserved_on_switch` - Integration test with live verification

## Test Files

1. **hoop-daemon/tests/adapter_failover.rs** (27KB)
   - Unit tests with temporary fleet.db
   - 11 test functions covering core functionality

2. **hoop-daemon/tests/adapter_failover_test.rs** (25KB)
   - Integration tests with HTTP client and daemon
   - 8 tokio::test functions
   - Includes `config_yml_hot_reload_triggers_adapter_switch` - primary test for this bead

3. **hoop-daemon/tests/adapter_failover_integration.rs** (27KB)
   - Additional integration tests
   - 9 test functions (async and sync)
   - Uses serial_test for parallel execution safety

## Implementation Details

### Config Hot-Reload Mechanism
The `config_yml_hot_reload_triggers_adapter_switch` test verifies:
1. Initial agent session spawned with Claude adapter
2. Config file edited to switch to ZAI adapter
3. 4-second wait for hot-reload debounce (2s debounce + processing time)
4. New session verified active with ZAI adapter
5. Old session verified archived with linked Stitch

### Stitch Archival
When an adapter switch occurs:
1. Old session's `AgentSessionRow` is loaded from fleet.db
2. In-memory conversation history is extracted
3. `fleet::archive_session_as_stitch()` creates Stitch with:
   - `project = "hoop-agent"`
   - `kind = "operator"`
   - `created_by = "hoop:agent"`
   - Title includes adapter name and timestamp
4. Stitch messages preserve full conversation history
5. Agent session row updated with `stitch_id` reference

### Reflection Ledger Continuity
The `build_handoff_context()` function in `agent_session.rs`:
1. Queries approved Reflection Ledger entries
2. Builds system prompt with:
   - "## Operator Preferences (Reflection Ledger)" section
   - Each rule with scope prefix
   - "## Recent Activity" section for context
3. New session starts with this handoff context

## Verification

All tests are already committed in git:
- Commit d2baba5: "tests: add adapter failover integration test (hoop-ttb.6.2.2)"
- Commit f7fd74e: "On main: Unrelated changes for later"
- Commit a7c35c9: "tests: fix adapter_failover.rs unit tests"

## Plan Reference

- §6 Phase 5 deliverable 7
- §7 LLM-agnostic

## Status: COMPLETE

All acceptance criteria met. Tests are comprehensive, well-structured, and already committed to the repository.
