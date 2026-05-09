# Adapter Failover Test Implementation Summary (hoop-ttb.6.2.2)

## Task Completion Status: COMPLETE

The adapter failover test for HOOP has been fully implemented and verified. All acceptance criteria from bead hoop-ttb.6.2.2 are met.

## Acceptance Criteria Verification

### 1. Simulated Anthropic 500 doesn't crash daemon ✓

**Implementation:**
- `test_anthropic_5xx_doesnt_crash_daemon` in adapter_failover_integration.rs
- `daemon_survives_simulated_anthropic_5xx` in adapter_failover_test.rs
- `anthropic_5xx_mock_server_daemon_survives` in adapter_failover_test.rs (with MockAnthropicServer)
- `adapter_error_doesnt_crash_daemon` in agent_session.rs

**Verification:** Tests verify that:
- Adapter can be built and initialized successfully
- Errors are handled gracefully without panic
- Daemon remains responsive after 5xx errors
- `/healthz` and `/readyz` endpoints continue responding

### 2. Operator switches adapter via config.yml edit → hot-reload triggers new session ✓

**Implementation:**
- `config_yml_hot_reload_triggers_adapter_switch` in adapter_failover_test.rs
- `hot_reload_config_change_triggers_adapter_switch` in agent_session.rs
- `test_adapter_switch_archives_session_as_stitch` in adapter_failover_integration.rs

**Verification:** Tests verify that:
- Config file changes are detected by the watcher (2-second debounce)
- Hot-reload triggers `switch_adapter` in AgentSessionManager
- New session is created with the new adapter
- Config validation prevents invalid changes

### 3. Old session's final transcript preserved as closed Stitch (kind=operator, archived) ✓

**Implementation:**
- `old_session_transcript_preserved_as_stitch` in adapter_failover_test.rs
- `test_adapter_switch_archives_session_as_stitch` in adapter_failover_integration.rs
- `adapter_failover_archives_session_preserves_reflection_ledger` in agent_session.rs

**Verification:** Tests verify that:
- Old session is archived with status="switched"
- Stitch is created with kind="operator"
- Stitch belongs to "hoop-agent" project
- Stitch is created by "hoop:agent"
- stitch_id is linked in agent_sessions table
- Conversation history is preserved in stitch_messages

### 4. Reflection Ledger continuity preserved ✓

**Implementation:**
- `reflection_ledger_continuity_preserved_on_switch` in adapter_failover_test.rs
- `test_reflection_ledger_continuity_across_switch` in adapter_failover_integration.rs
- `test_handoff_context_includes_reflection_ledger` in agent_session.rs

**Verification:** Tests verify that:
- Approved reflection entries persist after adapter switch
- `list_approved_reflection_entries()` returns all entries
- Rules are carried forward via `build_handoff_context()`
- New sessions have access to previous reflection rules

## Test Files Summary

| File | Tests | Description |
|------|-------|-------------|
| `hoop-daemon/tests/adapter_failover_test.rs` | 10 | Full integration tests with daemon spawning, HTTP client, mock server |
| `hoop-daemon/tests/adapter_failover_integration.rs` | 9 | Unit tests using fleet.db directly |
| `hoop-daemon/tests/adapter_failover.rs` | 9 | Additional adapter failover scenarios |
| `hoop-daemon/src/agent_session.rs` | 3+ | Unit tests for session lifecycle and adapter switching |

## Key Implementation Details

### `AgentSessionManager::switch_adapter()` (agent_session.rs:647)
- Archives old session as Stitch via `fleet::archive_session_as_stitch()`
- Builds new adapter via `agent_adapter::build_adapter()`
- Spawns fresh session with handoff context from `build_handoff_context()`

### `fleet::archive_session_as_stitch()` (fleet.rs)
- Creates Stitch row with kind="operator"
- Stores conversation history in stitch_messages
- Links stitch_id to agent_sessions row

### `build_handoff_context()` (agent_session.rs:798)
- Carries forward Reflection Ledger entries
- Includes recent Stitches for context
- Injected into new session's system prompt

### ConfigWatcher (config_watcher.rs)
- Detects config.yml changes
- Emits `AgentConfigChanged` events
- Triggers adapter switch via hot-reload

## Running the Tests

```bash
# Run all adapter failover tests
nix-shell --run 'cargo test -p hoop-daemon --test adapter_failover'

# Run specific test file
nix-shell --run 'cargo test -p hoop-daemon --test adapter_failover_integration'

# Run unit tests in agent_session.rs
nix-shell --run 'cargo test -p hoop-daemon --lib agent_session::tests::adapter_failover'
```

## Plan Reference

- §6 Phase 5 deliverable 7
- §7 LLM-agnostic (HOOP is adapter-agnostic; Anthropic outage is operator-recoverable, not an incident)

## Conclusion

The adapter failover test implementation is complete and comprehensive. All acceptance criteria for bead hoop-ttb.6.2.2 are met, providing confidence that:
- HOOP daemon survives Anthropic 5xx errors without crashing
- Operators can recover via config.yml edit and hot-reload
- Session transcripts are preserved as archived Stitches
- Reflection Ledger continuity is maintained across adapter switches
