# Adapter Failover Test Verification (hoop-ttb.6.2.2)

## Implementation Verified

The adapter failover test implementation at `hoop-daemon/tests/adapter_failover_test.rs` is **complete and comprehensive**.

## Acceptance Criteria Covered

All acceptance criteria from hoop-ttb.6.2.2 are covered:

### 1. Simulated Anthropic 500 doesn't crash daemon
- **Test**: `daemon_survives_simulated_anthropic_5xx`
- Verifies daemon health before and after simulated 5xx error

### 2. Operator switches adapter via config.yml edit → hot-reload triggers new session
- **Test**: `config_yml_hot_reload_triggers_adapter_switch`
- Edits config.yml to switch from claude to zai adapter
- Waits for hot-reload (2-second debounce + processing time)
- Verifies new agent status reflects ZAI adapter

### 3. Old session's final transcript preserved as closed Stitch (kind=operator, archived)
- **Test**: `old_session_transcript_preserved_as_stitch`
- Verifies archived session has stitch_id
- Verifies Stitch exists with:
  - kind="operator"
  - project="hoop-agent"
  - created_by="hoop:agent"

### 4. Reflection Ledger continuity preserved
- **Test**: `reflection_ledger_continuity_preserved_on_switch`
- Inserts reflection entry before switch
- Verifies entry persists after adapter switch

## Additional Tests

The implementation includes additional tests for robustness:

5. `adapter_switch_creates_new_session_and_archives_old` - API-based switching
6. `multiple_adapter_switches_create_multiple_stitches` - Multiple switches
7. `adapter_switch_with_active_turn_preserves_continuity` - Turn preservation
8. `concurrent_switch_requests_are_handled_gracefully` - Concurrency

## Implementation Details

- Uses `spawn_test_daemon_with_config` from integration_harness
- `FailoverClient` struct provides methods for agent operations
- Helper functions for session counting and stitch_id extraction
- Direct `fleet` module calls for Stitch verification

## Test File Location

`hoop-daemon/tests/adapter_failover_test.rs` (729 lines, 8 tests)

## Supporting Implementation Verified

**config_watcher.rs**:
- `AgentConfigChanged` event type for agent config changes
- `detect_agent_config_changes()` function to detect adapter/model/API key changes
- `agent_config_changed_tx` broadcast channel for triggering adapter switches
- Hot-reload with 2-second debounce

**agent_session.rs**:
- `switch_adapter()` method that:
  - Archives old session as a Stitch via `fleet::archive_session_as_stitch()`
  - Builds new adapter with fresh session
  - Carries forward Reflection Ledger + recent activity context
- `build_handoff_context()` for continuity

**fleet.rs**:
- `archive_session_as_stitch()` - Creates Stitch from session transcript
- `load_stitch_by_id()` - For test verification
- `StitchRow` type with required fields (id, project, kind, title, created_by, etc.)

**lib.rs**:
- Agent config change listener task that subscribes to events and calls `switch_adapter()`

**integration_harness.rs**:
- `spawn_test_daemon_with_config()` for isolated test daemon instances
- Temporary directories with isolated `.hoop/` config

## Verification Date

2026-05-09

## Status

**COMPLETE** - All acceptance criteria verified and implemented.
