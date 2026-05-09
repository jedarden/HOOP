# Adapter Failover Test - Session 11 Verification (hoop-ttb.6.2.2)

## Date: 2026-05-09

## Task
Adapter failover test: Anthropic 5xx → ZAI/GLM switch; session continuity surfaced.

## Implementation Status: COMPLETE ✅

The adapter failover test implementation was completed in commit `039bfe6` and verified in this session.

## Acceptance Criteria Verification

### 1. Simulated Anthropic 500 doesn't crash daemon ✅
**Test:** `daemon_survives_simulated_anthropic_5xx` (adapter_failover_test.rs:148)
**Test:** `anthropic_5xx_mock_server_daemon_survives` (adapter_failover_test.rs:796)
- Mock server returns 503 Service Unavailable
- Daemon survives for 30+ seconds with continuous health checks
- `/healthz` and `/readyz` endpoints remain responsive

### 2. Operator switches adapter via config.yml edit → hot-reload triggers new session ✅
**Test:** `config_yml_hot_reload_triggers_adapter_switch` (adapter_failover_test.rs:584)
- Config file watcher detects changes (2-second debounce)
- Hot-reload triggers `AgentSessionManager::switch_adapter()`
- New session created with new adapter configuration
- Old session archived with status="switched"

### 3. Old session's final transcript preserved as closed Stitch (kind=operator, archived) ✅
**Test:** `old_session_transcript_preserved_as_stitch` (adapter_failover_test.rs:259)
**Implementation:** `fleet::archive_session_as_stitch()` (fleet.rs:4595)
- Stitch created with kind="operator"
- Stitch belongs to "hoop-agent" project
- Stitch created by "hoop:agent"
- Conversation history preserved in stitch_messages table
- stitch_id linked in agent_sessions row

### 4. Reflection Ledger continuity preserved ✅
**Test:** `reflection_ledger_continuity_preserved_on_switch` (adapter_failover_test.rs:338)
**Implementation:** `build_handoff_context()` (agent_session.rs:798)
- Approved reflection entries persist across adapter switches
- Rules carried forward via system prompt injection
- New sessions have access to previous reflection rules

## Additional Tests

1. **`adapter_switch_creates_new_session_and_archives_old`** - Session isolation
2. **`multiple_adapter_switches_create_multiple_stitches`** - Sequential switches
3. **`adapter_switch_with_active_turn_preserves_continuity`** - Active turn handling
4. **`concurrent_switch_requests_are_handled_gracefully`** - Concurrent requests
5. **`anthropic_5xx_mock_then_adapter_switch_recovery`** - Full failover scenario

## Key Implementation Files

| File | Description |
|------|-------------|
| `hoop-daemon/tests/adapter_failover_test.rs` | 10 integration tests with daemon spawning |
| `hoop-daemon/tests/adapter_failover_integration.rs` | 9 unit tests using fleet.db |
| `hoop-daemon/src/agent_session.rs:647` | `switch_adapter()` function |
| `hoop-daemon/src/fleet.rs:4595` | `archive_session_as_stitch()` function |
| `hoop-daemon/src/api_agent.rs:139` | `POST /api/agent/switch` endpoint |

## Plan Reference
- §6 Phase 5 deliverable 7: Agent adapter abstraction
- §7 LLM-agnostic: Anthropic outage is operator-recoverable, not an incident

## Retrospective

### What worked
- Comprehensive test coverage with both integration and unit tests
- Mock server for realistic 5xx error simulation
- Clean separation of concerns (session management, fleet DB, API)

### What didn't
- N/A - Implementation completed successfully

### Surprise
- The test implementation includes a sophisticated MockAnthropicServer that returns 503
- Config hot-reload integration was already in place from prior work

### Reusable pattern
- Test harness pattern: `spawn_test_daemon_with_config()` for isolated test environments
- Mock server pattern for testing error scenarios
- Fleet DB helpers for direct database verification in tests
