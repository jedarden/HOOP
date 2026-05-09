# Adapter Failover Test Verification (hoop-ttb.6.2.2)

## Task Summary
Integration test simulates Anthropic 5xx. Operator-initiated switch to ZAI via `/reload`. Agent session survives (or starts fresh cleanly). Old transcript archived as a Stitch.

## Acceptance Criteria Verification

### 1. Simulated Anthropic 500 doesn't crash daemon ✓
- **Test**: `daemon_survives_simulated_anthropic_5xx` in `adapter_failover.rs`
- **Verification**: Test verifies daemon health before and after simulated error conditions
- **Implementation**: Error handling in `agent_adapter.rs` and `agent_session.rs` ensures daemon survives adapter errors

### 2. Operator switches adapter via config.yml edit → hot-reload triggers new session ✓
- **Test**: `config_yml_hot_reload_triggers_adapter_switch` in `adapter_failover.rs`
- **Verification**: Full integration test that:
  - Spawns daemon with initial config
  - Edits config.yml to switch from claude to zai adapter
  - Waits for hot-reload detection (2-second debounce)
  - Verifies new session is created with new adapter
  - Verifies old session is archived

### 3. Old session's final transcript preserved as closed Stitch (kind=operator, archived) ✓
- **Test**: `old_session_transcript_preserved_as_stitch` in `adapter_failover.rs`
- **Verification**: Test verifies:
  - Archived session has `status=switched`
  - Stitch is created in `hoop-agent` project
  - Stitch has `kind=operator`
  - Stitch has correct title referencing adapter
  - Conversation history is stored in stitch_messages

### 4. Reflection Ledger continuity preserved ✓
- **Test**: `reflection_ledger_continuity_preserved_on_switch` in `adapter_failover.rs`
- **Verification**: Test verifies:
  - Reflection Ledger entries persist after adapter switch
  - Approved rules are carried forward to new session
  - Entry content (rule, scope, status) remains unchanged

## Implementation Details

### Core Components
1. **AgentSessionManager::switch_adapter()** (`agent_session.rs:643`)
   - Archives old session as Stitch
   - Creates new adapter instance
   - Builds handoff context with Reflection Ledger
   - Spawns fresh session

2. **fleet::archive_session_as_stitch()** (`fleet.rs:4590`)
   - Creates Stitch row in `hoop-agent` project
   - Stores conversation history as stitch_messages
   - Links stitch_id to agent_sessions row

3. **fleet::archive_agent_session()** (`fleet.rs:3643`)
   - Updates session status to `switched`
   - Sets archived_at and archived_reason

4. **ConfigWatcher** (`config_watcher.rs`)
   - Detects config.yml changes
   - Sends `AgentConfigChanged` events
   - Triggers adapter switch via dedicated task

5. **API Endpoint** (`api_agent.rs:151`)
   - `POST /api/agent/switch` for manual adapter switching

## Test Files
1. **`adapter_failover.rs`** (803 lines)
   - Integration tests with full daemon spawning
   - Tests API-based and config-based switching
   - Verifies Stitch archival and Reflection Ledger continuity

2. **`adapter_failover_test.rs`** (729 lines)
   - Unit tests for fleet DB functions
   - Tests session archival and Stitch creation
   - Verifies history round-trip

3. **`adapter_failover_integration.rs`** (736 lines)
   - Integration tests with serial execution
   - Tests adapter switch, session archival, and continuity
   - Verifies handoff context includes Reflection Ledger

## Coverage Summary
All acceptance criteria are met with comprehensive test coverage:
- Daemon resilience to adapter errors ✓
- Hot-reload adapter switching ✓
- Session transcript archival ✓
- Reflection Ledger continuity ✓
- Multiple adapter switches ✓
- Concurrent switch handling ✓

## Plan Reference
- §6 Phase 5 deliverable 7: Agent-off switch and adapter failover
- §7 LLM-agnostic: "HOOP is LLM-agnostic — the agent is an adapter-configured resource"
