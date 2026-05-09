# Adapter Failover Test Final Verification (hoop-ttb.6.2.2)

## Task
Integration test simulates Anthropic 5xx. Operator-initiated switch to ZAI via `/reload`. Agent session survives (or starts fresh cleanly). Old transcript archived as a Stitch.

## Final Verification Status: COMPLETE ✅

### Test Coverage Summary

All acceptance criteria are covered by comprehensive tests across multiple test files:

#### 1. Simulated Anthropic 500 doesn't crash daemon ✅
**Tests**:
- `daemon_survives_simulated_anthropic_5xx` (adapter_failover_test.rs:148)
- `test_anthropic_5xx_doesnt_crash_daemon` (adapter_failover_integration.rs:51)
- `adapter_error_doesnt_crash_daemon` (agent_session.rs:1985)

**Coverage**: Verifies daemon remains healthy after adapter errors, healthz endpoints continue responding

#### 2. Operator switches adapter via config edit → new session created ✅
**Tests**:
- `adapter_switch_creates_new_session_and_archives_old` (adapter_failover_test.rs:182)
- `hot_reload_config_change_triggers_adapter_switch` (agent_session.rs:2070)

**Coverage**: Verifies POST /api/agent/switch creates new session, old session archived with `status='switched'`

#### 3. Old session transcript preserved as closed Stitch (kind=operator) ✅
**Tests**:
- `old_session_transcript_preserved_as_stitch` (adapter_failover_test.rs:260)
- `adapter_failover_archives_session_preserves_reflection_ledger` (agent_session.rs:1852)

**Coverage**: Verifies Stitch created with `kind='operator'`, `project='hoop-agent'`, linked via `stitch_id`

#### 4. Reflection Ledger continuity preserved ✅
**Tests**:
- `reflection_ledger_continuity_preserved_on_switch` (adapter_failover_test.rs:339)
- `adapter_failover_archives_session_preserves_reflection_ledger` (agent_session.rs:1852)

**Coverage**: Verifies approved Reflection Ledger entries persist and are included in new session's system prompt

### Test Files Inventory

| File | Location | Type |
|------|----------|------|
| `adapter_failover_test.rs` | hoop-daemon/tests/ | Integration (HTTP client) |
| `adapter_failover.rs` | hoop-daemon/tests/ | Unit (DB-backed) |
| `adapter_failover_integration.rs` | hoop-daemon/tests/ | Integration (DB-backed) |
| `integration_harness.rs` | hoop-daemon/tests/ | Test harness |
| Unit tests | hoop-daemon/src/agent_session.rs | Unit (lines 1843-2166) |

### Implementation Files

| Component | File | Key Functions |
|-----------|------|---------------|
| Session Manager | agent_session.rs | `switch_adapter()` (647), `build_handoff_context()` (798) |
| Fleet DB | fleet.rs | `archive_session_as_stitch()` (4590), `list_approved_reflection_entries()` |
| Adapter | agent_adapter.rs | `build_adapter()`, `AdapterKind` enum |

### Key Verification Points

1. **Session archival**: Old session marked `status='switched'` with `archived_reason='adapter_switch'`
2. **Stitch creation**: Stitch with `kind='operator'`, `project='hoop-agent'`, `created_by='hoop:agent'`
3. **Session linkage**: `agent_sessions.stitch_id` references the created Stitch
4. **Clean transition**: Exactly one active session after switch
5. **Reflection Ledger**: Approved rules persist and are carried into new session's system prompt

## Plan Reference
§6 Phase 5 deliverable 7, §7 LLM-agnostic

## Date
2026-05-09
