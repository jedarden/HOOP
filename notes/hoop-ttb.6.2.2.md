# Adapter Failover Test Verification (hoop-ttb.6.2.2)

## Summary

Integration test for adapter failover is **already implemented** in `hoop-daemon/src/agent_session.rs`. The implementation covers all acceptance criteria specified in the task.

## Acceptance Criteria Coverage

### 1. Simulated Anthropic 500 doesn't crash daemon
**Test:** `adapter_error_doesnt_crash_daemon` (lines 1982-2060)
- Creates an active session
- Simulates adapter error with "500 Internal Server Error" reason
- Verifies session is cleanly archived (not active)
- Verifies operator can recover by spawning new session on ZAI adapter
- Confirms only one active session after recovery

### 2. Operator switches adapter via config.yml edit → hot-reload triggers new session
**Test:** `hot_reload_config_change_triggers_adapter_switch` (lines 2067-2162)
- Simulates config watcher detecting adapter change from "claude" to "zai"
- Verifies old session is archived with "adapter_switch" reason
- Verifies new session is created on ZAI adapter
- Verifies Stitch is created for old session transcript
- Confirms clean transition (only one active session)

**Implementation:** The hot-reload mechanism is in `config_watcher.rs`:
- `detect_agent_config_changes()` detects adapter, model, API key changes
- `AgentConfigChanged` event is sent via broadcast channel
- AgentSessionManager subscribes to these events (lib.rs lines 2929-2967)
- `switch_adapter()` archives old session and spawns new one

### 3. Old session's final transcript preserved as closed Stitch (kind=operator, archived)
**Test:** `adapter_failover_archives_session_preserves_reflection_ledger` (lines 1849-1975)
- Creates active Anthropic session with usage history
- Archives session with "adapter_switch" reason
- Creates Stitch with kind="operator" in "hoop-agent" project
- Links Stitch to agent_sessions row via stitch_id
- Verifies Stitch metadata (kind, project, title)

**Implementation:** `fleet::archive_session_as_stitch()` (lines 4590-4633)
- Creates Stitch row with kind="operator"
- Stores in-memory history as stitch_messages
- Links Stitch to agent_sessions via stitch_id column

### 4. Reflection Ledger continuity preserved
**Test:** `adapter_failover_archives_session_preserves_reflection_ledger`
- Creates approved Reflection Ledger entries before switch
- Verifies entries are preserved after adapter switch
- Verifies `build_handoff_context()` includes Reflection Ledger rules
- Confirms new session has access to previous operator preferences

**Implementation:** `build_handoff_context()` in agent_session.rs (lines 794-841)
- Loads approved reflection ledger entries
- Includes them in system prompt for new session
- Carries forward recent activity context

## Key Files

| File | Purpose |
|------|---------|
| `hoop-daemon/src/agent_session.rs` | Session manager with failover tests |
| `hoop-daemon/src/agent_adapter.rs` | LLM-agnostic adapter abstraction |
| `hoop-daemon/src/config_watcher.rs` | Hot-reload with agent config change detection |
| `hoop-daemon/src/fleet.rs` | Stitch archival via `archive_session_as_stitch()` |
| `hoop-daemon/src/lib.rs` | Agent config change listener wiring (lines 2929-2967) |

## Hot-Reload Flow

```
config.yml edited
    ↓
ConfigWatcher detects change
    ↓
detect_agent_config_changes() compares old vs new config
    ↓
AgentConfigChanged event sent via broadcast channel
    ↓
AgentSessionManager receives event (lib.rs:2929)
    ↓
switch_adapter() called with new config
    ↓
1. Archive old session as Stitch
2. Build handoff context (Reflection Ledger + recent activity)
3. Spawn new session with new adapter
4. Emit SessionArchived and SessionSpawned events
```

## Verification

All tests pass the following assertions:
- Old session status changes to "switched" or "archived"
- Old session has archived_reason set correctly
- Stitch is created with kind="operator" and project="hoop-agent"
- New session is active on the new adapter
- Only one session is active at any time
- Reflection Ledger entries are preserved across the switch

## Conclusion

The adapter failover integration test is **complete and verified**. The implementation demonstrates:
- Graceful error handling (daemon doesn't crash on 5xx)
- Operator-initiated recovery via config hot-reload
- Session continuity through Stitch archival
- Reflection Ledger preservation across adapter switches
