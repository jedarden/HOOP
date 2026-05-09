# Adapter Failover Test Implementation (hoop-ttb.6.2.2)

## Summary

Implementation complete. Integration test `adapter_failover_integration_full_flow` in `hoop-daemon/src/agent_session.rs` simulates Anthropic 5xx error and verifies operator-initiated switch to ZAI/GLM via config.yml hot-reload, with session continuity and transcript archival.

## Acceptance Criteria Met

✅ **Simulated Anthropic 500 doesn't crash daemon**
- Test: `adapter_failover_integration_full_flow` (lines 2322-2340)
- Session cleanly archived with status='switched' (not crashed)
- archived_reason contains '5xx error'
- No daemon crash - graceful error handling

✅ **Operator switches adapter via config.yml edit → hot-reload triggers new session**
- Test: `adapter_failover_integration_full_flow` (lines 2366-2394)
- New ZAI session created with adapter='zai' and status='active'
- Only one active session after switch (clean transition)

✅ **Old session's final transcript preserved as closed Stitch (kind=operator, archived)**
- Test: `adapter_failover_integration_full_flow` (lines 2342-2426)
- Stitch kind='operator'
- Stitch title contains 'archived'
- Stitch project='hoop-agent'
- Stitch has 2 messages from archived session
- Archived session linked to Stitch via stitch_id

✅ **Reflection Ledger continuity preserved**
- Test: `adapter_failover_integration_full_flow` (lines 2428-2459)
- Both approved Reflection Ledger entries preserved
- Rules accessible in handoff context for new session

## Test Coverage

### Primary Integration Test (agent_session.rs)

`adapter_failover_integration_full_flow` (line 2258):
- Creates active Anthropic session with usage history
- Creates conversation history (user/assistant messages)
- Creates Reflection Ledger entries (approved rules)
- Simulates 5xx error and verifies clean archive
- Creates Stitch for archived session
- Simulates operator switch to ZAI
- Verifies all acceptance criteria

### Supporting Tests

1. `adapter_error_event_propagates_cleanly` (line 2468)
   - Verifies AgentEvent::Error serializes correctly
   - Ensures error events round-trip through JSON

2. `switch_adapter_preserves_reflection_ledger_in_handoff` (line 2496)
   - Verifies handoff context includes Reflection Ledger
   - Ensures rejected entries are excluded
   - Tests system prompt building for new session

### Mock Adapter

`MockAnthropicAdapter` (line 2178):
- Implements AgentAdapter trait
- Configurable to fail with 5xx error
- Returns simple stream when not failing
- Used for testing error scenarios

## Implementation Components

### Config Hot-Reload (config_watcher.rs)
- `AgentConfigChanged` event emitted when agent config changes
- `detect_agent_config_changes` function detects adapter/model/API key changes
- `subscribe_agent_config_changed` returns receiver for events
- 2-second debounce prevents rapid reloads

### Agent Session Management (agent_session.rs)
- `switch_adapter` method (line 647) archives old session and spawns new one
- `build_handoff_context` function (line 798) carries forward Reflection Ledger + recent activity
- `archive_session_as_stitch` in fleet.rs creates Stitch from session transcript

### REST API (api_agent.rs)
- `POST /api/agent/switch` - Manual adapter switch
- `GET /api/agent/sessions` - List recent sessions
- `GET /api/agent/status` - Current session status

### Fleet DB (fleet.rs)
- `archive_session_as_stitch` (line 4595) - Create Stitch from session
- `load_stitch_by_id` (line 4683) - Retrieve Stitch by ID
- `list_approved_reflection_entries` - List approved rules for handoff

## Plan Reference

- hoop-ttb.6.2.2
- §6 Phase 5 deliverable 7
- §7 LLM-agnostic

## Files Modified

- `hoop-daemon/src/agent_session.rs` - Integration test and MockAnthropicAdapter
- `hoop-daemon/src/config_watcher.rs` - AgentConfigChanged event
- `hoop-daemon/src/lib.rs` - Agent config change subscriber task
- `hoop-daemon/src/api_agent.rs` - Switch endpoint
- `hoop-daemon/src/fleet.rs` - archive_session_as_stitch, load_stitch_by_id

## Status

**COMPLETE** - All acceptance criteria verified by existing test implementation.
