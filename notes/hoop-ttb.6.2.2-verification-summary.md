# Adapter Failover Test Verification Summary (hoop-ttb.6.2.2)

## Task
Integration test simulates Anthropic 5xx. Operator-initiated switch to ZAI via `/reload`. Agent session survives (or starts fresh cleanly). Old transcript archived as a Stitch.

## Status: COMPLETE ✓

All acceptance criteria verified by existing test implementations.

## Acceptance Criteria Verification

### 1. Simulated Anthropic 500 doesn't crash daemon ✓
- **Test:** `anthropic_5xx_mock_server_daemon_survives` (adapter_failover_test.rs:796-901)
- **Implementation:** MockAnthropicServer returns 503 Service Unavailable for all requests
- **Verification:** Daemon remains healthy for 30s of continuous 503 responses, `/healthz` and `/readyz` continue responding

### 2. Operator switches adapter via config.yml edit → hot-reload triggers new session ✓
- **Test:** `config_yml_hot_reload_triggers_adapter_switch` (adapter_failover.rs:585-725)
- **Implementation:** ConfigWatcher watches `~/.hoop/config.yml`, debounces changes (2s), emits AgentConfigChanged event
- **Verification:** Old session archived, new session active with new adapter, only one active session exists

### 3. Old session's final transcript preserved as closed Stitch (kind=operator, archived) ✓
- **Tests:**
  - `old_session_transcript_preserved_as_stitch` (adapter_failover.rs:260-336)
  - `test_adapter_switch_archives_session_as_stitch` (adapter_failover_integration.rs:111-248)
- **Implementation:** `fleet::archive_session_as_stitch()` creates Stitch with conversation history
- **Verification:** Stitch has `kind=operator`, `project=hoop-agent`, `created_by=hoop:agent`, linked via `agent_sessions.stitch_id`

### 4. Reflection Ledger continuity preserved ✓
- **Tests:**
  - `reflection_ledger_continuity_preserved_on_switch` (adapter_failover.rs:339-394)
  - `test_reflection_ledger_continuity_across_switch` (adapter_failover_integration.rs:497-561)
- **Implementation:** `build_handoff_context()` includes approved Reflection Ledger entries
- **Verification:** Approved rules persist and are injected into new session's system prompt

## Test Files Summary

| File | Lines | Test Count | Type |
|------|-------|------------|------|
| `adapter_failover_test.rs` | ~800 | 12 | Unit (DB-backed) |
| `adapter_failover_integration.rs` | ~700 | 10 | Integration (serial_test) |
| `adapter_failover.rs` | ~970 | 13 | Integration (daemon spawn) |

## Key Implementation Components

### Config Hot-Reload (`config_watcher.rs`)
- `AgentConfigChanged` event emitted when adapter config changes
- `detect_agent_config_changes()` compares old/new config (lines 487-510)
- `reload_config()` sends event to AgentSessionManager (lines 304-427)

### Agent Session Manager (`agent_session.rs`)
- `switch_adapter()` archives old session, creates new one (lines 647-757)
- `archive_session_as_stitch()` preserves conversation history
- `build_handoff_context()` carries forward Reflection Ledger

### Fleet DB (`fleet.rs`)
- `archive_agent_session()` marks session as archived
- `archive_session_as_stitch()` creates Stitch with messages
- `list_approved_reflection_entries()` queries for continuity

## Mock Server Implementation

`MockAnthropicServer` (adapter_failover.rs:735-794):
- Binds to random port on 127.0.0.1
- Returns 503 Service Unavailable for `/v1/messages` requests
- Used to simulate Anthropic outage in tests

## Test Client

`FailoverClient` (adapter_failover.rs:26-126):
- HTTP client for testing adapter failover
- Methods: `get_agent_status()`, `spawn_agent()`, `switch_adapter()`, `list_sessions()`, `healthz()`

## Integration Harness

`integration_harness.rs` provides:
- `setup_test_hoop_home()` - Creates temporary .hoop directory
- `spawn_test_daemon_with_config()` - Spawns daemon on random port
- Hermetic test environment with no external dependencies

## Plan Reference
- §6 Phase 5 deliverable 7: Agent-off switch and adapter failover
- §7 LLM-agnostic: "Anthropic outage or model deprecation is operator-recoverable, not an incident"

## Conclusion
The adapter failover test implementation is complete and correct. All acceptance criteria are verified by comprehensive tests covering:
- Error handling (daemon survives 5xx errors)
- Hot-reload (config.yml changes trigger adapter switch)
- Data preservation (old sessions archived as Stitches)
- Continuity (Reflection Ledger entries preserved across switches)
