# Adapter Failover Test - Final Verification (hoop-ttb.6.2.2)

## Task Summary
Integration test simulates Anthropic 5xx → ZAI/GLM switch; session continuity surfaced.

## Acceptance Criteria Status: ALL MET ✓

### 1. Simulated Anthropic 500 doesn't crash daemon ✓
**Implementation:**
- Test: `daemon_survives_simulated_anthropic_5xx` in `adapter_failover_test.rs`
- Test: `anthropic_5xx_mock_server_daemon_survives` with mock server returning 503
- Verification: Daemon remains healthy after 5xx errors, health endpoint returns "ok"

**Key Implementation:**
- AgentSessionManager handles adapter errors gracefully
- Errors are logged but don't crash the daemon
- Agent can be recovered by spawning new session or switching adapter

### 2. Operator switches adapter via config.yml edit → hot-reload triggers new session ✓
**Implementation:**
- Test: `config_yml_hot_reload_triggers_adapter_switch` in `adapter_failover_test.rs`
- Test: `hot_reload_config_change_triggers_adapter_switch` in `agent_session.rs`
- Verification: ConfigWatcher detects changes, triggers switch_adapter()

**Key Implementation:**
- ConfigWatcher (`config_watcher.rs`) watches `~/.hoop/config.yml`
- Debounces changes (2 seconds) to avoid multiple reloads
- AgentConfigChanged events trigger AgentSessionManager::switch_adapter()
- Clean transition with only one active session

### 3. Old session's final transcript preserved as closed Stitch (kind=operator, archived) ✓
**Implementation:**
- Test: `old_session_transcript_preserved_as_stitch` in `adapter_failover_test.rs`
- Test: `test_adapter_switch_archives_session_as_stitch` in `adapter_failover_integration.rs`
- Verification: Stitch created with correct metadata

**Key Implementation:**
- `fleet::archive_session_as_stitch()` creates Stitch in hoop-agent project
- Stitch kind is "operator", created_by is "hoop:agent"
- Stitch title references the adapter (e.g., "Agent session anthropic (archived)")
- Agent session row linked via stitch_id

### 4. Reflection Ledger continuity preserved ✓
**Implementation:**
- Test: `reflection_ledger_continuity_preserved_on_switch` in `adapter_failover_test.rs`
- Test: `test_reflection_ledger_continuity_across_switch` in `adapter_failover_integration.rs`
- Verification: Approved entries persist after adapter switch

**Key Implementation:**
- `build_handoff_context()` includes Reflection Ledger entries
- Approved rules carried forward in new session's system prompt
- Both global and project-scoped rules preserved

## Test Files

1. **`hoop-daemon/src/agent_session.rs`** - Unit tests with in-memory DB
   - `adapter_failover_archives_session_preserves_reflection_ledger`
   - `adapter_error_doesnt_crash_daemon`
   - `hot_reload_config_change_triggers_adapter_switch`

2. **`hoop-daemon/tests/adapter_failover_integration.rs`** - Integration tests with serial_test
   - 10+ tests covering all acceptance criteria
   - Database-backed verification

3. **`hoop-daemon/tests/adapter_failover.rs`** - Full integration tests with daemon spawn
   - API endpoint tests (/api/agent/switch, /api/agent/status, /api/agent/sessions)
   - Config.yml hot-reload with file watcher
   - Concurrent switch requests

4. **`hoop-daemon/tests/adapter_failover_test.rs`** - HTTP mock server for 5xx simulation
   - MockAnthropicServer returns 503 on all requests
   - Tests daemon survives 30s of continuous 503 responses
   - Tests recovery by switching adapter after 5xx error

## Plan Reference
- §6 Phase 5 deliverable 7: Agent-off switch and adapter failover
- §7 LLM-agnostic: "HOOP is LLM-agnostic — the agent is an adapter-configured resource"

## Status: COMPLETE
All acceptance criteria met with comprehensive test coverage.
