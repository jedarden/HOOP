# Adapter Failover Test - Final Verification (hoop-ttb.6.2.2)

## Date: 2026-05-09

## Task Status: COMPLETE ✓

## Summary

Verified that the adapter failover test implementation in `tests/adapter_failover_test.rs` (969 lines) meets all acceptance criteria for bead hoop-ttb.6.2.2.

## Acceptance Criteria Coverage

### 1. Simulated Anthropic 500 doesn't crash daemon ✓
**Test:** `daemon_survives_simulated_anthropic_5xx` (lines 148-178)
- Spawns agent session with Anthropic adapter
- Simulates 5xx error condition
- Verifies daemon remains healthy after error
- Verifies agent status remains accessible

**Additional Test:** `anthropic_5xx_mock_server_daemon_survives` (lines 796-900)
- Implements `MockAnthropicServer` that returns 503 Service Unavailable
- Verifies daemon survives for 30 seconds of 503 responses
- Checks healthz and readyz endpoints throughout
- Verifies no crash or panic occurs

### 2. Operator switches adapter via config.yml edit → hot-reload triggers new session ✓
**Test:** `config_yml_hot_reload_triggers_adapter_switch` (lines 584-724)
- Spawns initial agent session with Claude adapter
- Edits config.yml to switch to ZAI adapter
- Waits for hot-reload to detect change (2-second debounce + processing time)
- Verifies new agent status reflects ZAI adapter
- Confirms exactly 1 active session and 1 switched session
- Verifies daemon remains healthy after hot-reload

### 3. Old session's final transcript preserved as closed Stitch (kind=operator, archived) ✓
**Test:** `old_session_transcript_preserved_as_stitch` (lines 259-335)
- Switches adapter to trigger archival
- Finds archived session in sessions list
- Verifies session status is "switched"
- Verifies stitch_id is present
- Queries fleet.db to verify Stitch exists with:
  - kind = "operator"
  - title contains "Agent session"
  - project = "hoop-agent"
  - created_by = "hoop:agent"

### 4. Reflection Ledger continuity preserved ✓
**Test:** `reflection_ledger_continuity_preserved_on_switch` (lines 338-393)
- Inserts a reflection ledger entry before switching
- Performs adapter switch
- Verifies reflection entry still exists after switch
- Confirms entry content is unchanged (rule, scope, status)

## Implementation Details

### Mock Server
- `MockAnthropicServer` struct (lines 734-793)
- Axum-based mock server on random port
- Returns 503 Service Unavailable for /v1/messages
- Graceful shutdown support

### Test Client
- `FailoverClient` struct (lines 24-125)
- HTTP client for testing adapter failover operations
- Methods: `get_agent_status`, `spawn_agent`, `switch_adapter`, `list_sessions`, `healthz`

### Helper Functions
- `count_sessions_by_status` - Count sessions by status in sessions list
- `get_session_stitch_id` - Get stitch_id from session if present

## All Tests

1. `daemon_survives_simulated_anthropic_5xx` - Basic daemon survival
2. `adapter_switch_creates_new_session_and_archives_old` - API-based adapter switch
3. `old_session_transcript_preserved_as_stitch` - Stitch archival verification
4. `reflection_ledger_continuity_preserved_on_switch` - Reflection Ledger continuity
5. `multiple_adapter_switches_create_multiple_stitches` - Multiple switches
6. `adapter_switch_with_active_turn_preserves_continuity` - Turn continuity
7. `concurrent_switch_requests_are_handled_gracefully` - Concurrency handling
8. `config_yml_hot_reload_triggers_adapter_switch` - Config file hot-reload
9. `anthropic_5xx_mock_server_daemon_survives` - Mock server survival test
10. `anthropic_5xx_mock_then_adapter_switch_recovery` - Full failover scenario

## Plan Reference
- §6 Phase 5 deliverable 7: Agent-off switch and adapter failover
- §7 LLM-agnostic: "Anthropic outage or model deprecation is operator-recoverable, not an incident."

## Conclusion

The adapter failover test implementation is complete and correct. All acceptance criteria are verified by comprehensive tests that simulate the full failover flow from Anthropic 5xx error through ZAI adapter switch with session continuity and Reflection Ledger preservation.
