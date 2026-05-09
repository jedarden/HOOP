# Adapter Failover Test Summary (hoop-ttb.6.2.2)

## Task
Integration test simulates Anthropic 5xx. Operator-initiated switch to ZAI via `/reload`. Agent session survives (or starts fresh cleanly). Old transcript archived as a Stitch.

## Why
§7 says "Anthropic outage or model deprecation is operator-recoverable, not an incident." Claim needs a test.

## Acceptance Criteria ✅

### 1. Simulated Anthropic 500 doesn't crash daemon ✅
- **Test**: `daemon_survives_simulated_anthropic_5xx`
- **Location**: `hoop-daemon/tests/adapter_failover_test.rs:148-178`
- **Coverage**: Verifies daemon health after simulated 5xx error condition

### 2. Operator switches adapter via config.yml edit → hot-reload triggers new session ✅
- **Test**: `config_yml_hot_reload_triggers_adapter_switch` (PRIMARY TEST)
- **Location**: `hoop-daemon/tests/adapter_failover_test.rs:584-724`
- **Coverage**:
  - Edits `config.yml` to switch from claude → zai adapter
  - Waits for hot-reload to detect change (2-second debounce)
  - Verifies new agent status reflects ZAI adapter
  - Verifies old session is archived with `status=switched`
  - Verifies old session has `stitch_id` linking to preserved Stitch
  - Verifies daemon remains healthy after hot-reload

### 3. Old session's final transcript preserved as closed Stitch (kind=operator, archived) ✅
- **Test**: `old_session_transcript_preserved_as_stitch`
- **Location**: `hoop-daemon/tests/adapter_failover_test.rs:259-335`
- **Coverage**:
  - Verifies archived session has `stitch_id`
  - Queries `fleet.db` to verify Stitch exists
  - Validates Stitch `kind=operator`
  - Validates Stitch `project=hoop-agent`
  - Validates Stitch `created_by=hoop:agent`

### 4. Reflection Ledger continuity preserved ✅
- **Test**: `reflection_ledger_continuity_preserved_on_switch`
- **Location**: `hoop-daemon/tests/adapter_failover_test.rs:338-393`
- **Coverage**:
  - Inserts reflection ledger entry before switch
  - Performs adapter switch
  - Verifies reflection entry still exists after switch
  - Validates entry content unchanged

## Additional Test Coverage

### Comprehensive Integration Tests
1. **`adapter_switch_creates_new_session_and_archives_old`** - Verifies new session creation and old session archival
2. **`multiple_adapter_switches_create_multiple_stitches`** - Tests multiple sequential switches
3. **`adapter_switch_with_active_turn_preserves_continuity`** - Tests continuity during active turn
4. **`concurrent_switch_requests_are_handled_gracefully`** - Tests concurrent switch handling

### HTTP Mock Server Tests (Child Beads)
1. **`MockAnthropicServer`** - Mock server returning 503 Service Unavailable
2. **`anthropic_5xx_mock_server_daemon_survives`** - 30-second survival test with 503 responses
3. **`anthropic_5xx_mock_then_adapter_switch_recovery`** - Full failover scenario test

### Unit Tests (adapter_failover_integration.rs)
1. **`test_anthropic_5xx_doesnt_crash_daemon`** - Unit test for 5xx handling
2. **`test_adapter_switch_archives_session_as_stitch`** - Unit test for Stitch archival
3. **`test_new_session_created_after_adapter_switch`** - Unit test for session creation
4. **`test_adapter_switch_preserves_usage_stats`** - Unit test for usage stats preservation
5. **`test_multiple_adapter_switches_maintain_history`** - Unit test for multiple switches
6. **`test_reflection_ledger_continuity_across_switch`** - Unit test for reflection ledger
7. **`test_session_continuity_after_daemon_restart`** - Unit test for daemon restart
8. **`test_archived_session_preserves_timestamp`** - Unit test for timestamp preservation

## Test Files

| File | Description | Test Count |
|------|-------------|------------|
| `hoop-daemon/tests/adapter_failover_test.rs` | Main integration tests with mock HTTP server | 10 |
| `hoop-daemon/tests/adapter_failover_integration.rs` | Unit tests with serial_test | 8 |
| `hoop-daemon/tests/adapter_failover.rs` | Legacy unit tests | 15 |

## Key Implementation Details

### Integration Harness
- `integration_harness::spawn_test_daemon_with_config` - Spawns daemon with custom config
- `DaemonHandle` - Exposes `temp_dir` for config file editing tests
- `FailoverClient` - Test client for adapter failover operations

### Fleet DB Helpers
- `fleet::archive_agent_session(session_id, reason)` - Archives session
- `fleet::archive_session_as_stitch(session_row, history)` - Creates Stitch from session
- `fleet::load_stitch_by_id(stitch_id)` - Loads Stitch for verification
- `fleet::list_approved_reflection_entries(scope)` - Lists reflection entries

### API Endpoints Tested
- `GET /api/agent/status` - Agent session status
- `POST /api/agent/spawn` - Spawn new session
- `POST /api/agent/switch` - Switch adapter
- `GET /api/agent/sessions` - List recent sessions
- `GET /healthz` - Health check
- `GET /readyz` - Readiness check

## Plan Reference
- §6 Phase 5 deliverable 7
- §7 LLM-agnostic

## Status
✅ **COMPLETE** - All acceptance criteria met with comprehensive test coverage.
