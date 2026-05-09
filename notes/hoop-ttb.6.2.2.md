# Adapter Failover Test Verification (hoop-ttb.6.2.2)

## Summary

Verified that the adapter failover test implementation is complete and meets all acceptance criteria.

## Acceptance Criteria Verification

### 1. Simulated Anthropic 500 doesn't crash daemon ✅
**Test**: `daemon_survives_simulated_anthropic_5xx()` in `hoop-daemon/tests/adapter_failover.rs`

Verifies that the daemon remains healthy during adapter errors and can recover.

### 2. Operator switches adapter via config.yml edit → hot-reload triggers new session ✅
**Test**: `config_yml_hot_reload_triggers_adapter_switch()` in `hoop-daemon/tests/adapter_failover.rs`

Tests the complete flow:
- Operator edits `~/.hoop/config.yml` to change `agent.adapter`
- ConfigWatcher detects the change (2-second debounce)
- `AgentConfigChanged` event is sent
- `AgentSessionManager::switch_adapter()` is called
- Old session is archived as Stitch
- New session is spawned with new adapter

### 3. Old session's final transcript preserved as closed Stitch (kind=operator, archived) ✅
**Test**: `old_session_transcript_preserved_as_stitch()` in `hoop-daemon/tests/adapter_failover.rs`

Verifies:
- Session is archived with status="switched"
- Stitch is created with kind="operator"
- Stitch is in "hoop-agent" project
- Stitch has correct title with adapter name and timestamp

### 4. Reflection Ledger continuity preserved ✅
**Test**: `reflection_ledger_continuity_preserved_on_switch()` in `hoop-daemon/tests/adapter_failover.rs`

Verifies that Reflection Ledger entries are preserved across adapter switches.

## Implementation Components

### Core Implementation Files

1. **`hoop-daemon/src/agent_adapter.rs`**
   - Defines `AdapterKind` enum (Claude, Anthropic, Zai, etc.)
   - `build_adapter()` factory function
   - Individual adapter implementations

2. **`hoop-daemon/src/agent_session.rs`**
   - `AgentSessionManager::switch_adapter()` (lines 643-740)
   - Archives old session as Stitch
   - Builds new adapter
   - Spawns fresh session with Reflection Ledger carried forward

3. **`hoop-daemon/src/config_watcher.rs`**
   - `detect_agent_config_changes()` function (lines 482-509)
   - Detects adapter, model, API key, ZAI base URL changes
   - Sends `AgentConfigChanged` event

4. **`hoop-daemon/src/fleet.rs`**
   - `archive_session_as_stitch()` - Creates Stitch from session
   - `archive_agent_session()` - Archives session with reason
   - `list_approved_reflection_entries()` - Lists Reflection Ledger
   - `load_stitch_by_id()` - Loads Stitch by ID

5. **`hoop-daemon/src/lib.rs`**
   - Agent config change listener (lines 2932-2987)
   - Wires `AgentConfigChanged` events to `switch_adapter()`

## Status

**COMPLETE** - All acceptance criteria met with comprehensive tests.
