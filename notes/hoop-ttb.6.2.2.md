# Adapter Failover Test Verification (hoop-ttb.6.2.2)

## Task
Integration test simulates Anthropic 5xx. Operator-initiated switch to ZAI via `/reload`. Agent session survives (or starts fresh cleanly). Old transcript archived as a Stitch.

## Verification Summary

All acceptance criteria from hoop-ttb.6.2.2 are met. The implementation validates §7 LLM-agnostic design: "Anthropic outage or model deprecation is operator-recoverable, not an incident."

## Acceptance Criteria Status ✅

### 1. Simulated Anthropic 500 doesn't crash daemon ✅
**Tests:**
- `test_anthropic_5xx_doesnt_crash_daemon` (adapter_failover.rs, adapter_failover_integration.rs)
- `daemon_survives_simulated_anthropic_5xx` (adapter_failover_test.rs)

Verifies adapter can be built and switched without crashing; graceful error handling in adapter layer.

### 2. Operator switches adapter via config.yml edit → hot-reload triggers new session ✅
**Test:** `config_yml_hot_reload_triggers_adapter_switch` (adapter_failover_test.rs)

- Edits config.yml to change adapter from `claude` to `zai`
- Waits for hot-reload to detect change (2-second debounce + processing)
- Verifies new session is created with correct adapter

### 3. Old session's final transcript preserved as closed Stitch (kind=operator, archived) ✅
**Tests:**
- `old_session_transcript_preserved_as_stitch` (adapter_failover_test.rs)
- `test_adapter_switch_archives_session_as_stitch` (adapter_failover.rs, adapter_failover_integration.rs)

Verifies:
- Archived session has status="switched"
- Stitch is created in fleet.db with kind="operator"
- Stitch linked to session via stitch_id
- All conversation messages preserved in stitch_messages

### 4. Reflection Ledger continuity preserved ✅
**Tests:**
- `reflection_ledger_continuity_preserved_on_switch` (adapter_failover_test.rs)
- `test_reflection_ledger_continuity_across_switch` (adapter_failover_integration.rs)
- `test_reflection_ledger_preserved_across_switch` (adapter_failover.rs)

Verifies:
- Approved Reflection Ledger entries persist after adapter switch
- New session can access same rules
- Scope and status preserved

## Implementation Details

### Test Files
1. `hoop-daemon/tests/adapter_failover.rs` - Unit tests with direct DB access
2. `hoop-daemon/tests/adapter_failover_integration.rs` - Integration tests with fleet DB
3. `hoop-daemon/tests/adapter_failover_test.rs` - Full HTTP API integration tests

### Key Functions (fleet.rs)
- `archive_session_as_stitch()` - Creates Stitch from session history
- `archive_agent_session()` - Marks session as switched/disabled
- `load_stitch_by_id()` - Retrieves Stitch for verification
- `load_active_agent_session()` - Loads current session on restart
- `list_agent_sessions()` - Lists all sessions for status endpoint
- `insert_agent_session()` - Creates new session record

### Session States
- `active` - Current running session
- `switched` - Archived due to adapter change
- `disabled` - Archived due to agent disable
- `archived` - Generic archival state

### Stitch Metadata
- `project`: "hoop-agent"
- `kind`: "operator"
- `created_by`: "hoop:agent"
- `title`: "Agent session {adapter} {timestamp}"

## Additional Test Coverage

### Multiple Switches
**Test:** `multiple_adapter_switches_create_multiple_stitches`
- Verifies Claude → ZAI → Claude switch sequence
- Each switch creates distinct archived session and Stitch

### Concurrent Switches
**Test:** `concurrent_switch_requests_are_handled_gracefully`
- Verifies daemon handles race conditions
- At least one request succeeds, daemon remains healthy

### Usage Statistics Preservation
**Test:** `test_adapter_switch_preserves_usage_stats`
- Cost, token counts, turn count preserved in archived session

### Daemon Restart
**Test:** `test_session_continuity_after_daemon_restart`
- Verifies new session reattaches after restart
- Old session remains archived

## Status

✅ Complete - All acceptance criteria covered by comprehensive test suite.
