# Adapter Failover Test Implementation Summary

## Task
Integration test simulates Anthropic 5xx. Operator-initiated switch to ZAI via `/reload`. Agent session survives (or starts fresh cleanly). Old transcript archived as a Stitch.

## Implementation Status

The adapter failover integration test is **fully implemented** in `hoop-daemon/src/agent_session.rs` at lines 2257-2460.

### Test: `adapter_failover_integration_full_flow`

This test simulates the complete failover flow:

1. **Start with Anthropic adapter** - Creates an active Anthropic session with usage history
2. **Simulate 5xx error** - Archives session with error reason (doesn't crash daemon)
3. **Operator switches adapter** - Simulates config.yml edit changing adapter to ZAI
4. **Hot-reload triggers new session** - New ZAI session is created
5. **Old session archived as Stitch** - Creates Stitch with kind=operator, archived
6. **Reflection Ledger continuity** - Verifies approved rules are preserved

### Acceptance Criteria Coverage

| Criterion | Status | Implementation |
|-----------|--------|----------------|
| Simulated Anthropic 500 doesn't crash daemon | ✅ | Test archives session cleanly with status='switched' |
| Operator switches adapter via config.yml edit → hot-reload triggers new session | ✅ | Test simulates adapter change and creates new ZAI session |
| Old session's final transcript preserved as closed Stitch (kind=operator, archived) | ✅ | Test creates Stitch with kind=operator and copies session messages |
| Reflection Ledger continuity preserved | ✅ | Test verifies approved rules are present after switch |

### Supporting Tests

Additional tests in `agent_session.rs` provide comprehensive coverage:

- `hot_reload_config_change_triggers_adapter_switch` (lines 2077-2172)
- `adapter_error_doesnt_crash_daemon` (lines 1992-2070)
- `adapter_error_event_propagates_cleanly` (lines 2468-2488)
- `switch_adapter_preserves_reflection_ledger_in_handoff` (lines 2496-2562)

### Production Code Integration

The test exercises the following production code paths:

1. `AgentSessionManager::switch_adapter` (agent_session.rs:647-744)
2. `fleet::archive_session_as_stitch` (fleet.rs:4595-4640)
3. `build_handoff_context` (agent_session.rs:798-845)
4. ConfigWatcher integration (config_watcher.rs:483-510)

## Plan Reference

- §6 Phase 5 deliverable 7
- §7 LLM-agnostic
- hoop-ttb.6.2.2
