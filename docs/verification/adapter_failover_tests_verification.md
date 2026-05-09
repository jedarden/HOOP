# Adapter Failover Test Verification (hoop-ttb.6.2.2)

## Summary

The adapter failover integration tests are fully implemented and comprehensive. All acceptance criteria from bead hoop-ttb.6.2.2 are met.

## Test Files

1. **`hoop-daemon/tests/adapter_failover_test.rs`** (10 tests)
   - Full integration tests with daemon spawning
   - HTTP client for REST API testing
   - Mock Anthropic server for 5xx simulation

2. **`hoop-daemon/tests/adapter_failover_integration.rs`** (9 tests)
   - Unit tests using fleet.db directly
   - Tests for session archival and Stitch creation

3. **`hoop-daemon/tests/adapter_failover.rs`** (9 tests)
   - Additional adapter failover test scenarios

4. **`hoop-daemon/src/agent_session.rs`** (tests module)
   - Unit tests for session lifecycle and adapter switching

## Acceptance Criteria Coverage

### 1. Simulated Anthropic 500 doesn't crash daemon ✓

**Tests:**
- `test_anthropic_5xx_doesnt_crash_daemon` (adapter_failover_integration.rs)
- `daemon_survives_simulated_anthropic_5xx` (adapter_failover_test.rs)
- `anthropic_5xx_mock_server_daemon_survives` (adapter_failover_test.rs)
- `adapter_error_doesnt_crash_daemon` (agent_session.rs)

**Verification:** Tests verify that:
- Adapter can be built and initialized successfully
- Errors are handled gracefully
- Daemon remains responsive after 5xx errors
- `/healthz` and `/readyz` endpoints continue responding

### 2. Operator switches adapter via config.yml edit → hot-reload triggers new session ✓

**Tests:**
- `config_yml_hot_reload_triggers_adapter_switch` (adapter_failover_test.rs)
- `hot_reload_config_change_triggers_adapter_switch` (agent_session.rs)
- `test_adapter_switch_archives_session_as_stitch` (adapter_failover_integration.rs)

**Verification:** Tests verify that:
- Config file changes are detected by the watcher
- Hot-reload triggers `switch_adapter` in AgentSessionManager
- New session is created with the new adapter
- Config validation prevents invalid changes

### 3. Old session's final transcript preserved as closed Stitch (kind=operator, archived) ✓

**Tests:**
- `old_session_transcript_preserved_as_stitch` (adapter_failover_test.rs)
- `test_adapter_switch_archives_session_as_stitch` (adapter_failover_integration.rs)
- `adapter_failover_archives_session_preserves_reflection_ledger` (agent_session.rs)

**Verification:** Tests verify that:
- Old session is archived with status="switched"
- Stitch is created with kind="operator"
- Stitch belongs to "hoop-agent" project
- Stitch is created by "hoop:agent"
- stitch_id is linked in agent_sessions table
- Conversation history is preserved in stitch_messages

### 4. Reflection Ledger continuity preserved ✓

**Tests:**
- `reflection_ledger_continuity_preserved_on_switch` (adapter_failover_test.rs)
- `test_reflection_ledger_continuity_across_switch` (adapter_failover_integration.rs)
- `test_handoff_context_includes_reflection_ledger` (agent_session.rs)

**Verification:** Tests verify that:
- Approved reflection entries persist after adapter switch
- `list_approved_reflection_entries()` returns all entries
- Rules are carried forward via `build_handoff_context()`
- New sessions have access to previous reflection rules

## Additional Test Coverage

### Multiple Adapter Switches
- `multiple_adapter_switches_create_multiple_stitches` (adapter_failover_test.rs)
- `test_multiple_adapter_switches_maintain_history` (adapter_failover_integration.rs)

### Usage Statistics Preservation
- `test_adapter_switch_preserves_usage_stats` (adapter_failover_integration.rs)

### Session Continuity After Restart
- `test_session_continuity_after_daemon_restart` (adapter_failover_integration.rs)

### Mock Server Testing
- `MockAnthropicServer` implementation for 503 simulation
- `anthropic_5xx_mock_then_adapter_switch_recovery` (adapter_failover_test.rs)

### Concurrent Request Handling
- `concurrent_switch_requests_are_handled_gracefully` (adapter_failover_test.rs)

## Test Execution

Run all adapter failover tests:
```bash
cargo test --package hoop-daemon --test adapter_failover
cargo test --package hoop-daemon --test adapter_failover_integration
cargo test --package hoop-daemon --test adapter_failover_test
cargo test --package hoop-daemon --lib agent_session::tests::adapter_failover
```

## Implementation Details

### Key Functions Tested

1. **`AgentSessionManager::switch_adapter()`** (agent_session.rs:647)
   - Archives old session as Stitch
   - Builds new adapter
   - Spawns fresh session with handoff context

2. **`fleet::archive_session_as_stitch()`** (fleet.rs:4590)
   - Creates Stitch row with kind="operator"
   - Stores conversation history in stitch_messages
   - Links stitch_id to agent_sessions row

3. **`build_handoff_context()`** (agent_session.rs:798)
   - Carries forward Reflection Ledger entries
   - Includes recent Stitches for context
   - Injected into new session's system prompt

4. **ConfigWatcher** (config_watcher.rs)
   - Detects config.yml changes
   - Emits `AgentConfigChanged` events
   - Triggers adapter switch via hot-reload

## Conclusion

All acceptance criteria for bead hoop-ttb.6.2.2 are met. The implementation provides:
- ✅ Daemon survivability during Anthropic 5xx errors
- ✅ Operator-initiated adapter switch via config.yml hot-reload
- ✅ Old session transcript preservation as archived Stitch
- ✅ Reflection Ledger continuity across adapter switches

The tests validate both the happy path and edge cases, including multiple switches, concurrent requests, and mock server failures.
