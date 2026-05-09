# Adapter Failover Test Verification (hoop-ttb.6.2.2)

## Task Summary
Integration test simulates Anthropic 5xx. Operator-initiated switch to ZAI via config.yml hot-reload. Agent session survives (or starts fresh cleanly). Old transcript archived as a Stitch.

## Acceptance Criteria Verification

### 1. Simulated Anthropic 500 doesn't crash daemon ✓
- **Test**: `adapter_error_doesnt_crash_daemon` in `agent_session.rs:1986`
- **Verification**: Test verifies:
  - Session is cleanly archived with error reason
  - No active sessions remain (daemon didn't crash)
  - Operator can recover by spawning new session on ZAI adapter
  - Clean recovery with only one active session

### 2. Operator switches adapter via config.yml edit → hot-reload triggers new session ✓
- **Test**: `hot_reload_config_change_triggers_adapter_switch` in `agent_session.rs:2071`
- **Verification**: Test simulates:
  - Initial active Claude session
  - Config change from "claude" to "zai" detected by config watcher
  - Old session archived and linked to Stitch
  - New ZAI session active
  - Clean transition with only one active session

### 3. Old session's final transcript preserved as closed Stitch (kind=operator, archived) ✓
- **Test**: `adapter_failover_archives_session_preserves_reflection_ledger` in `agent_session.rs:1853`
- **Verification**: Test verifies:
  - Old session is archived with status=switched, archived_reason=adapter_switch
  - Stitch is created with kind=operator, project=hoop-agent
  - Stitch is linked to archived session via stitch_id
  - Stitch title references the adapter (e.g., "Agent session anthropic (archived)")

### 4. Reflection Ledger continuity preserved ✓
- **Test**: `adapter_failover_archives_session_preserves_reflection_ledger` in `agent_session.rs:1853`
- **Verification**: Test verifies:
  - Reflection Ledger entries persist after adapter switch
  - Approved rules are still queryable after switch
  - Handoff context includes Reflection Ledger rules (via build_handoff_context())
  - Entry content (rule, scope, status) remains unchanged

## Implementation Details

### Core Components
1. **AgentSessionManager::switch_adapter()** (`agent_session.rs:647`)
   - Archives old session as Stitch via fleet::archive_session_as_stitch()
   - Creates new adapter instance with new config
   - Builds handoff context with Reflection Ledger via build_handoff_context()
   - Spawns fresh session with new adapter
   - Emits SessionArchived and SessionSpawned events

2. **fleet::archive_session_as_stitch()** (`fleet.rs:4590`)
   - Creates Stitch row in `hoop-agent` project with kind=operator
   - Stores conversation history as stitch_messages
   - Links stitch_id to agent_sessions row

3. **fleet::archive_agent_session()** (`fleet.rs:3643`)
   - Updates session status to `switched`
   - Sets archived_at and archived_reason

4. **ConfigWatcher** (`config_watcher.rs`)
   - Detects config.yml changes via file system watcher
   - Sends AgentConfigChanged events on adapter/model/API key changes
   - Debounces changes (2 seconds) to avoid multiple reloads
   - Validates config before applying (rejects invalid configs)

5. **Agent Config Change Listener** (`lib.rs:2932-2987`)
   - Subscribes to AgentConfigChanged events from ConfigWatcher
   - Calls AgentSessionManager::switch_adapter() on config changes
   - Logs adapter switch events

6. **API Endpoint** (`api_agent.rs:151`)
   - `POST /api/agent/switch` for manual adapter switching
   - Accepts adapter, model, and API key configuration

## Test Implementation

The adapter failover tests are implemented as unit tests in `hoop-daemon/src/agent_session.rs`:

1. **`adapter_failover_archives_session_preserves_reflection_ledger`** (lines 1846-1979)
   - Main test for bead hoop-ttb.6.2.2
   - Uses in-memory database to simulate failover scenario
   - Verifies all acceptance criteria

2. **`adapter_error_doesnt_crash_daemon`** (lines 1981-2064)
   - Verifies daemon stability on adapter errors
   - Tests recovery by spawning new session

3. **`hot_reload_config_change_triggers_adapter_switch`** (lines 2066-2166)
   - Verifies hot-reload triggering adapter switch
   - Tests clean transition between adapters

## Coverage Summary
All acceptance criteria are met with comprehensive test coverage:
- Daemon resilience to adapter errors ✓
- Hot-reload adapter switching ✓
- Session transcript archival as Stitch ✓
- Reflection Ledger continuity ✓
- Clean transition (only one active session) ✓

## Plan Reference
- §6 Phase 5 deliverable 7: Agent-off switch and adapter failover
- §7 LLM-agnostic: "HOOP is LLM-agnostic — the agent is an adapter-configured resource"

## Additional Test Files (Integration Tests)

Beyond the unit tests in agent_session.rs, three additional test files exist:

1. **`hoop-daemon/tests/adapter_failover_integration.rs`**
   - Unit-style tests using direct fleet DB access
   - 10+ tests covering all acceptance criteria
   - Uses serial_test for test isolation

2. **`hoop-daemon/tests/adapter_failover_test.rs`**
   - Full integration tests with daemon spawning
   - Tests API endpoints (/api/agent/switch, /api/agent/status, /api/agent/sessions)
   - Tests config.yml hot-reload with file watcher
   - Tests concurrent switch requests

3. **`hoop-daemon/tests/adapter_failover.rs`**
   - Additional unit tests for adapter build and session archival
   - Tests Reflection Ledger preservation
   - Tests usage statistics preservation

## Recent Changes (2026-05-09)

Fixed compilation errors related to `anthropic_base_url` field:
- `hoop-daemon/src/api_agent.rs`: Added `anthropic_base_url: None` to `AgentAdapterConfig` initialization
- `hoop-daemon/src/lib.rs`: Added `anthropic_base_url` field in two locations (agent initialization and hot-reload handler)

These changes ensure the code compiles correctly with the updated `AgentAdapterConfig` struct that includes the `anthropic_base_url` field.
