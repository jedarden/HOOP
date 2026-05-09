# Adapter Failover Test Implementation (hoop-ttb.6.2.2)

## Summary

Integration test for adapter failover: Anthropic 5xx → ZAI/GLM switch; session continuity surfaced.

## Acceptance Criteria - ALL MET ✅

### 1. Simulated Anthropic 500 doesn't crash daemon
- **Test**: `daemon_survives_simulated_anthropic_5xx` in `adapter_failover_test.rs`
- **Implementation**: Adapter error handling in `agent_adapter.rs` and `agent_session.rs`
- Daemon continues running when adapter returns errors; errors are logged but don't crash the process

### 2. Operator switches adapter via config.yml edit → hot-reload triggers new session
- **Test**: `config_yml_hot_reload_triggers_adapter_switch` in `adapter_failover.rs`
- **Implementation**:
  - `config_watcher.rs` detects config changes and emits `AgentConfigChanged` events
  - `lib.rs` background task subscribes to events and calls `AgentSessionManager::switch_adapter()`
  - `api_agent.rs` provides `/api/agent/switch` endpoint for manual switching

### 3. Old session's final transcript preserved as closed Stitch (kind=operator, archived)
- **Test**: `old_session_transcript_preserved_as_stitch` in `adapter_failover.rs`
- **Implementation**:
  - `fleet::archive_session_as_stitch()` creates Stitch in "hoop-agent" project
  - Stitch kind is "operator", created_by is "hoop:agent"
  - Session history stored in `stitch_messages` table
  - Agent session row linked via `stitch_id` column

### 4. Reflection Ledger continuity preserved
- **Test**: `reflection_ledger_continuity_preserved_on_switch` in `adapter_failover.rs`
- **Implementation**:
  - `agent_session::build_handoff_context()` includes approved Reflection Ledger entries
  - Rules carried forward in new session's system prompt
  - Both global and project-scoped rules preserved

## Test Files

1. **`hoop-daemon/tests/adapter_failover.rs`** - Integration tests with in-memory DB
2. **`hoop-daemon/tests/adapter_failover_integration.rs`** - Integration tests with serial_test
3. **`hoop-daemon/tests/adapter_failover_test.rs`** - Full daemon spawn tests with HTTP API
4. **`hoop-daemon/src/agent_session.rs`** - Inline unit tests (lines 1842-2166)

## Key Implementation Files

- `hoop-daemon/src/agent_session.rs` - `switch_adapter()` method (lines 647-743)
- `hoop-daemon/src/fleet.rs` - `archive_session_as_stitch()` (lines 4590-4633)
- `hoop-daemon/src/config_watcher.rs` - Agent config change detection (lines 360-413)
- `hoop-daemon/src/lib.rs` - Background task for adapter switch (lines 2957-3014)
- `hoop-daemon/src/api_agent.rs` - `/api/agent/switch` endpoint (lines 151-197)

## Plan Reference

- §6 Phase 5 deliverable 7
- §7 LLM-agnostic

## Verification Status

**COMPLETE** - All acceptance criteria verified with comprehensive test coverage.
