# Adapter Failover Test Verification (hoop-ttb.6.2.2)

## Summary

This document verifies that the adapter failover test implementation in `tests/adapter_failover_test.rs` meets all acceptance criteria for bead hoop-ttb.6.2.2.

## Acceptance Criteria Coverage

### 1. Simulated Anthropic 500 doesn't crash daemon

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

### 2. Operator switches adapter via config.yml edit → hot-reload triggers new session

**Test:** `config_yml_hot_reload_triggers_adapter_switch` (lines 584-724)
- Spawns initial agent session with Claude adapter
- Edits config.yml to switch to ZAI adapter
- Waits for hot-reload to detect change (2-second debounce + processing time)
- Verifies new agent status reflects ZAI adapter
- Confirms exactly 1 active session and 1 switched session
- Verifies daemon remains healthy after hot-reload

**Additional Test:** `adapter_switch_creates_new_session_and_archives_old` (lines 181-256)
- Tests adapter switching via API endpoint
- Verifies new session ID differs from initial
- Counts active vs archived sessions

### 3. Old session's final transcript preserved as closed Stitch (kind=operator, archived)

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

**Implementation:** `fleet::archive_session_as_stitch` (fleet.rs:4595-4629)
- Creates Stitch with ID, project, kind, title, created_by, timestamps
- Stores in-memory history as stitch_messages
- Returns stitch_id for linking to session

### 4. Reflection Ledger continuity preserved

**Test:** `reflection_ledger_continuity_preserved_on_switch` (lines 338-393)
- Inserts a reflection ledger entry before switching
- Performs adapter switch
- Verifies reflection entry still exists after switch
- Confirms entry content is unchanged (rule, scope, status)

**Additional Test:** `adapter_switch_with_active_turn_preserves_continuity` (lines 474-530)
- Adds reflection entries that should be carried forward
- Switches adapter
- Verifies new session is active
- Confirms reflection entries are still accessible

**Implementation:** `agent_session::build_handoff_context` (agent_session.rs:798-817)
- Loads approved reflection ledger entries (global + all scopes)
- Injects them into system prompt for new session
- Ensures continuity across adapter switches

## Child Beads Coverage

### hoop-ttb.6.2.2.1: HTTP mock for Anthropic 5xx
**Implementation:** `MockAnthropicServer` (lines 730-793)
- Axum-based mock server on random port
- Returns 503 Service Unavailable for /v1/messages
- Can be toggled on/off per test
- Graceful shutdown support

### hoop-ttb.6.2.2.2: Daemon survives 5xx
**Test:** `anthropic_5xx_mock_server_daemon_survives` (lines 796-900)
- 30-second survival test with health checks every 5 seconds
- Verifies /healthz and /readyz remain responsive
- Confirms error logging without panic

### hoop-ttb.6.2.2.3: Config.yml hot-reload triggers new session
**Test:** `config_yml_hot_reload_triggers_adapter_switch` (lines 584-724)
- Direct config.yml file editing
- 4-second wait for hot-reload debounce
- Verifies adapter switch completes within timeout

### hoop-ttb.6.2.2.4: Old session archived as Stitch
**Test:** `old_session_transcript_preserved_as_stitch` (lines 259-335)
- Direct fleet.db query for Stitch verification
- Validates all required fields

### hoop-ttb.6.2.2.5: Reflection Ledger continuity
**Test:** `reflection_ledger_continuity_preserved_on_switch` (lines 338-393)
- Pre-populated reflection entries
- Post-switch verification

## Additional Test Coverage

### Multiple switches
**Test:** `multiple_adapter_switches_create_multiple_stitches` (lines 396-471)
- Verifies each switch creates a separate archived session
- Confirms distinct stitch_ids for each archived session

### Concurrent requests
**Test:** `concurrent_switch_requests_are_handled_gracefully` (lines 533-581)
- Issues concurrent switch requests
- Verifies at least one succeeds
- Confirms daemon remains healthy

### Full failover scenario
**Test:** `anthropic_5xx_mock_then_adapter_switch_recovery` (lines 903-965)
- Starts with Anthropic adapter pointing to mock server
- Experiences 5xx errors
- Operator recovers by switching to ZAI adapter
- Verifies service restoration

## Implementation Quality

1. **Hermetic testing:** Uses `spawn_test_daemon_with_config` for isolated test environments
2. **Comprehensive assertions:** Each test verifies multiple aspects of behavior
3. **Realistic scenarios:** Tests cover both API and config-file based adapter switching
4. **Error handling:** Verifies graceful degradation under error conditions
5. **State verification:** Direct database queries ensure correct persistence

## Conclusion

All acceptance criteria for hoop-ttb.6.2.2 are met by the implementation in `tests/adapter_failover_test.rs`. The test suite provides comprehensive coverage of:

- Daemon survival during Anthropic 5xx errors
- Hot-reload triggered adapter switching
- Session transcript archival as Stitches
- Reflection Ledger continuity across switches

The implementation supports §7's claim that "Anthropic outage or model deprecation is operator-recoverable, not an incident."
