# Adapter Failover Test Verification (hoop-ttb.6.2.2)

## Summary

This document verifies that the adapter failover tests meet all acceptance criteria from hoop-ttb.6.2.2:
- Simulated Anthropic 500 doesn't crash daemon
- Operator switches adapter via config.yml edit → hot-reload triggers new session
- Old session's final transcript preserved as closed Stitch (kind=operator, archived)
- Reflection Ledger continuity preserved

## Test Files

### 1. `hoop-daemon/tests/adapter_failover.rs`
**Comprehensive integration tests with full daemon boot:**

- `daemon_survives_simulated_anthropic_5xx` - Verifies daemon health during and after simulated 5xx
- `adapter_switch_creates_new_session_and_archives_old` - Tests `/api/agent/switch` endpoint
- `old_session_transcript_preserved_as_stitch` - Verifies Stitch creation with correct metadata
- `reflection_ledger_continuity_preserved_on_switch` - Validates Reflection Ledger preservation
- `multiple_adapter_switches_create_multiple_stitches` - Tests multiple consecutive switches
- `adapter_switch_with_active_turn_preserves_continuity` - Tests mid-turn switch
- `concurrent_switch_requests_are_handled_gracefully` - Tests concurrent switch safety
- `config_yml_hot_reload_triggers_adapter_switch` - **Primary test for config file edit → hot-reload flow**
- `anthropic_5xx_mock_server_daemon_survives` - Mock server returning 503 for 30s
- `anthropic_5xx_mock_then_adapter_switch_recovery` - Full 5xx → switch → recovery flow

### 2. `hoop-daemon/tests/adapter_failover_integration.rs`
**Unit-level integration tests with serial execution:**

- `test_anthropic_5xx_doesnt_crash_daemon` - Basic adapter build verification
- `test_adapter_switch_archives_session_as_stitch` - Direct DB verification
- `test_new_session_created_after_adapter_switch` - Session lifecycle verification
- `test_adapter_switch_preserves_usage_stats` - Cost/token tracking preservation
- `test_multiple_adapter_switches_maintain_history` - Multi-switch history tracking
- `test_reflection_ledger_continuity_across_switch` - Reflection Ledger preservation
- `test_session_continuity_after_daemon_restart` - Restart persistence verification

### 3. `hoop-daemon/tests/adapter_failover_test.rs`
**Unit tests for core failover functionality:**

- `test_anthropic_5xx_doesnt_crash_daemon` - Adapter factory resilience
- `test_adapter_switch_archives_session_as_stitch` - Stitch archival verification
- `test_adapter_switch_archives_session_row` - Session row archival verification
- `test_multiple_adapter_switches_single_active` - Single active session invariant
- `test_reflection_ledger_preserved_across_switch` - Reflection Ledger preservation
- `test_session_status_shows_new_adapter_after_switch` - Status correctness
- `test_archived_stitch_metadata` - Stitch metadata verification
- `test_session_history_round_trip` - Conversation history preservation
- `test_handoff_context_includes_reflection_ledger` - Handoff context verification

## Implementation Verification

### Core Components

1. **Agent Session Management** (`hoop-daemon/src/agent_session.rs`)
   - `AgentSessionManager::switch_adapter()` (line 647) - Handles adapter switching
   - Archives old session as Stitch with full history
   - Builds handoff context from Reflection Ledger
   - Spawns new session with carried-forward context

2. **Adapter Abstraction** (`hoop-daemon/src/agent_adapter.rs`)
   - `AdapterKind` enum with Claude/Anthropic/Zai variants
   - `AgentAdapter` trait for unified interface
   - `build_adapter()` factory for config-driven selection

3. **Fleet Database** (`hoop-daemon/src/fleet.rs`)
   - `archive_session_as_stitch()` (line 4595) - Creates Stitch from session history
   - `archive_agent_session()` (line 3643) - Marks session as switched/archived
   - `load_active_agent_session()` - Session reattachment on restart
   - `list_approved_reflection_entries()` - Reflection Ledger continuity

## Acceptance Criteria Coverage

| Criteria | Test Coverage | Implementation |
|----------|--------------|----------------|
| Simulated Anthropic 500 doesn't crash daemon | `anthropic_5xx_mock_server_daemon_survives` | Adapter error handling in `send_turn()` |
| config.yml edit → hot-reload triggers new session | `config_yml_hot_reload_triggers_adapter_switch` | Config watcher → `switch_adapter()` |
| Old transcript preserved as closed Stitch | `old_session_transcript_preserved_as_stitch` | `archive_session_as_stitch()` |
| Reflection Ledger continuity preserved | `reflection_ledger_continuity_preserved_on_switch` | `build_handoff_context()` |
| Stitch kind=operator, archived | Verified via DB queries | Stitch creation with `kind="operator"` |

## Key Flows Verified

1. **Normal Adapter Switch** (Claude → ZAI)
   - Old session marked "switched" with archived_at timestamp
   - Old transcript saved as Stitch in hoop-agent project
   - New session created with ZAI adapter
   - Only one active session at a time

2. **Error Recovery** (5xx → Switch)
   - Mock server returns 503 Service Unavailable
   - Daemon remains healthy (doesn't crash)
   - Operator switches adapter via config or API
   - Service restored with new adapter

3. **Session Continuity**
   - Reflection Ledger rules carried to new session
   - Recent Stitches included in handoff context
   - Old session history preserved in Stitch
   - New session starts fresh with context

## Test Execution

```bash
# Run all adapter failover tests
cargo test --test adapter_failover --test adapter_failover_test --test adapter_failover_integration

# Run specific test
cargo test config_yml_hot_reload_triggers_adapter_switch
```

## Verification Status

All acceptance criteria are met by the existing implementation and test suite:

✅ Simulated Anthropic 500 doesn't crash daemon
✅ Operator switches adapter via config.yml edit → hot-reload triggers new session
✅ Old session's final transcript preserved as closed Stitch (kind=operator, archived)
✅ Reflection Ledger continuity preserved

The tests verify the claim from §7 that "Anthropic outage or model deprecation is operator-recoverable, not an incident."
