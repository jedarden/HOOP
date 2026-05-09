# Adapter Failover Tests - Verification Summary

## Task (hoop-ttb.6.2.2)
Integration test simulates Anthropic 5xx → ZAI/GLM switch; session continuity surfaced.

## Acceptance Criteria Met

### 1. Simulated Anthropic 500 doesn't crash daemon
**Test**: `daemon_survives_simulated_anthropic_5xx` in `adapter_failover.rs`
- Verifies daemon remains healthy during and after simulated 5xx error
- Confirms agent session continues to function
- Health check endpoint returns `status: "ok"`

### 2. Operator switches adapter via config.yml edit → hot-reload triggers new session
**Test**: `config_yml_hot_reload_triggers_adapter_switch` in `adapter_failover.rs`
- Edits `~/.hoop/config.yml` to change adapter from `claude` to `zai`
- Waits for hot-reload to detect the change (2-second debounce + processing time)
- Verifies new agent status reflects ZAI adapter
- Confirms old session is archived with reason "switched"
- Verifies archived session has a linked `stitch_id`

### 3. Old session's final transcript preserved as closed Stitch (kind=operator, archived)
**Tests**:
- `old_session_transcript_preserved_as_stitch` in `adapter_failover.rs`
- `test_adapter_switch_archives_session_as_stitch` in `adapter_failover_integration.rs`
- `test_archived_stitch_metadata` in `adapter_failover_test.rs`

**Verification**:
- Stitch is created in `hoop-agent` project
- Stitch kind is `operator`
- Stitch title references the old adapter (e.g., "Agent session anthropic (archived)")
- Stitch `created_by` is `hoop:agent`
- All conversation messages are preserved in `stitch_messages` table
- Agent session row has `stitch_id` linking to the archived Stitch

### 4. Reflection Ledger continuity preserved
**Tests**:
- `reflection_ledger_continuity_preserved_on_switch` in `adapter_failover.rs`
- `test_reflection_ledger_continuity_across_switch` in `adapter_failover_integration.rs`
- `test_handoff_context_includes_reflection_ledger` in `adapter_failover_test.rs`

**Verification**:
- Approved Reflection Ledger entries persist after adapter switch
- Entries are accessible via `fleet::list_approved_reflection_entries()`
- Rules are carried forward in `build_handoff_context()` for new session's system prompt
- Both global and project-scoped rules are preserved

## Additional Test Coverage

### Multiple Adapter Switches
**Test**: `multiple_adapter_switches_create_multiple_stitches` in `adapter_failover.rs`
- Verifies multiple switches create separate Stitches
- Each archived session has a distinct `stitch_id`
- Only one session is active at a time

### Session Continuity After Restart
**Test**: `test_session_continuity_after_daemon_restart` in `adapter_failover_integration.rs`
- Simulates daemon restart after adapter switch
- Verifies new session is properly reattached
- Old session remains archived

### Usage Statistics Preservation
**Test**: `test_adapter_switch_preserves_usage_stats` in `adapter_failover_integration.rs`
- Verifies cost, token counts, and turn count are preserved in archived session

### Concurrent Switch Handling
**Test**: `concurrent_switch_requests_are_handled_gracefully` in `adapter_failover.rs`
- Verifies daemon handles concurrent switch requests gracefully
- At least one switch should succeed
- Daemon remains healthy

## Test Files

1. **`hoop-daemon/tests/adapter_failover.rs`** - Full integration tests with daemon spawn
2. **`hoop-daemon/tests/adapter_failover_test.rs`** - Unit tests with direct fleet.db access
3. **`hoop-daemon/tests/adapter_failover_integration.rs`** - Integration tests with serial_test

## Key Implementation Details

### API Endpoints Used
- `GET /api/agent/status` - Get current agent session status
- `POST /api/agent/spawn` - Spawn a new agent session
- `POST /api/agent/switch` - Switch adapter mid-stream
- `GET /api/agent/sessions` - List recent sessions

### Fleet Functions Used
- `fleet::archive_session_as_stitch()` - Archives session transcript as Stitch
- `fleet::load_stitch_by_id()` - Loads a Stitch by ID for verification
- `fleet::insert_agent_session()` - Inserts a new agent session row
- `fleet::archive_agent_session()` - Archives an agent session with reason
- `fleet::list_approved_reflection_entries()` - Lists approved Reflection Ledger entries
- `fleet::insert_reflection_entry()` - Inserts a Reflection Ledger entry (for testing)

## Plan Reference
- §6 Phase 5 deliverable 7
- §7 LLM-agnostic

## Verification Status
All acceptance criteria met. Tests are comprehensive and cover:
- ✅ Daemon survives simulated Anthropic 5xx
- ✅ Config.yml hot-reload triggers adapter switch
- ✅ Old session transcript preserved as closed Stitch
- ✅ Reflection Ledger continuity preserved
