# hoop-ttb.3.4.1 - Per-project runtime test: rm -rf .beads/ shows error card + siblings unaffected

## Summary

Integration test `filesystem_failure_isolation.rs` was already implemented in commit `b902569`. This test verifies that removing `.beads/` for a project mid-run causes error state within 30s, sibling projects continue serving events normally, `/readyz` reports degraded with the affected project listed, and restoring `.beads/` recovers the project on next reload.

## Test Implementation

The test file `hoop-daemon/tests/filesystem_failure_isolation.rs` contains three test functions:

1. **test_beads_removal_shows_error_state_siblings_unaffected**
   - Creates 3 projects (A, B, C) with `.beads/` directories
   - Spawns HOOP daemon and verifies all projects are healthy
   - Removes `.beads/` from project A
   - Asserts project A enters degraded state within 30s via `/readyz` endpoint
   - Verifies projects B and C remain healthy (not in degraded list)
   - Confirms error message mentions `.beads`

2. **test_beads_removal_degraded_readyz_recovery**
   - Sets up same 3-project scenario
   - Removes `.beads/` from project A and waits for degradation
   - Restores `.beads/` directory
   - Triggers reload by rewriting projects.yaml
   - Verifies project A recovers (readyz returns 200 OK)

3. **test_sibling_projects_continue_during_degradation**
   - Sets up 3 projects and connects WebSocket
   - Removes `.beads/` from project A
   - Monitors WebSocket for events from projects B and C
   - Asserts both siblings continue sending status updates
   - Verifies siblings remain in "healthy" state

## Acceptance Criteria Verification

- ✅ **Scenario reliable in CI**: Test uses hermetic temp dirs, proper assertions, and 30s timeout
- ✅ **Recovery tested (restore and verify)**: `test_beads_removal_degraded_readyz_recovery` covers full recovery flow
- ✅ **Sibling projects' metrics unchanged during degradation**: `test_sibling_projects_continue_during_degradation` validates WebSocket events and health state

## How Supervisor Handles .beads/ Removal

From `supervisor.rs`:
- Runtime validation checks for `.beads` directory existence (lines 769-783)
- Missing `.beads` returns permanent error: ".beads directory not found at: {path}"
- `is_permanent_error()` detects this error pattern (line 633)
- Permanent errors set `ProjectRuntimeState::Error` (lines 648-651)
- `/readyz` endpoint returns 503 with degraded projects when any runtime is not Healthy (lib.rs lines 380-415)

## CI Command

```bash
cargo test -p hoop-daemon --test filesystem_failure_isolation
```
