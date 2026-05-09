# Adapter Failover Test Implementation (hoop-ttb.6.2.2)

## Summary

Implementation complete. Integration test simulates Anthropic 5xx error and verifies operator-initiated switch to ZAI/GLM via config.yml hot-reload, with session continuity and transcript archival.

## Acceptance Criteria Met

✅ **Simulated Anthropic 500 doesn't crash daemon**
- Test: `daemon_survives_simulated_anthropic_5xx`
- Mock server: `MockAnthropicServer` returns 503 Service Unavailable
- Daemon remains healthy after 5xx errors
- `/readyz` continues responding

✅ **Operator switches adapter via config.yml edit → hot-reload triggers new session**
- Test: `config_yml_hot_reload_triggers_adapter_switch`
- ConfigWatcher detects file changes with 2-second debounce
- `AgentConfigChanged` event triggers `AgentSessionManager::switch_adapter`
- New session starts with new adapter configuration

✅ **Old session's final transcript preserved as closed Stitch (kind=operator, archived)**
- Test: `old_session_transcript_preserved_as_stitch`
- `fleet::archive_session_as_stitch` creates Stitch with kind=operator
- Session row linked to stitch_id
- Stitch created in hoop-agent project with creator hoop:agent

✅ **Reflection Ledger continuity preserved**
- Test: `reflection_ledger_continuity_preserved_on_switch`
- `build_handoff_context` carries forward approved Reflection Ledger entries
- New session's system prompt includes operator preferences from previous session

## Test Coverage

### Primary Tests (adapter_failover_test.rs)

1. `daemon_survives_simulated_anthropic_5xx` - Basic health check during error
2. `adapter_switch_creates_new_session_and_archives_old` - API-based switch
3. `old_session_transcript_preserved_as_stitch` - Stitch archival verification
4. `reflection_ledger_continuity_preserved_on_switch` - Reflection ledger preservation
5. `config_yml_hot_reload_triggers_adapter_switch` - Config file hot-reload
6. `multiple_adapter_switches_create_multiple_stitches` - Multiple switches
7. `adapter_switch_with_active_turn_preserves_continuity` - Mid-turn switch
8. `concurrent_switch_requests_are_handled_gracefully` - Concurrent requests
9. `anthropic_5xx_mock_server_daemon_survives` - Mock server 30-second survival
10. `anthropic_5xx_mock_then_adapter_switch_recovery` - Full failover scenario

### Unit Tests (agent_session.rs)

1. `adapter_failover_archives_session_preserves_reflection_ledger` - DB-level verification
2. `adapter_error_doesnt_crash_daemon` - Error handling
3. `hot_reload_config_change_triggers_adapter_switch` - Config change flow

## Implementation Components

### Config Hot-Reload (config_watcher.rs)
- `AgentConfigChanged` event emitted when agent config changes
- `detect_agent_config_changes` function detects adapter/model/API key changes
- `subscribe_agent_config_changed` returns receiver for events
- 2-second debounce prevents rapid reloads

### Agent Session Management (agent_session.rs)
- `switch_adapter` method archives old session and spawns new one
- `build_handoff_context` carries forward Reflection Ledger + recent activity
- `archive_session_as_stitch` creates Stitch from session transcript

### REST API (api_agent.rs)
- `POST /api/agent/switch` - Manual adapter switch
- `GET /api/agent/sessions` - List recent sessions
- `GET /api/agent/status` - Current session status

### Fleet DB (fleet.rs)
- `archive_session_as_stitch` - Create Stitch from session
- `list_agent_sessions` - List all sessions
- `load_stitch_by_id` - Retrieve Stitch by ID
- `insert_reflection_entry` - Add Reflection Ledger entry
- `list_approved_reflection_entries` - List approved rules

## Mock Server

`MockAnthropicServer`:
- Listens on random port (127.0.0.1)
- Returns 503 Service Unavailable for all `/v1/messages` requests
- Simulates Anthropic outage for testing

## Plan Reference

- §6 Phase 5 deliverable 7
- §7 LLM-agnostic

## Files Modified

- `hoop-daemon/tests/adapter_failover_test.rs` (970 lines)
- `hoop-daemon/src/agent_session.rs` (unit tests)
- `hoop-daemon/src/config_watcher.rs` (AgentConfigChanged event)
- `hoop-daemon/src/lib.rs` (agent config change subscriber task)
- `hoop-daemon/src/api_agent.rs` (switch endpoint)
- `hoop-daemon/src/fleet.rs` (archive_session_as_stitch, list_agent_sessions)
