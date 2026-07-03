# Epoch Sync Test Failure Investigation - bf-5jpx7

## Summary

Investigated the failing `test_epoch_sync_init_event_carrying_subscriptions` test in `hoop-daemon/tests/epoch_sync_invariant.rs`.

## Test Details

**Test File**: `hoop-daemon/tests/epoch_sync_invariant.rs`
**Failing Test**: `test_epoch_sync_init_event_carrying_subscriptions`

### Test Purpose
The test validates that the WebSocket `init` event carries the server-authoritative subscription list, specifically:
1. First message must be `init` event type
2. Init event must contain `subscriptions` array
3. `subscriptions` array must contain `"global"` entry

## Findings

### Existing Test Run Results
From bead trace `bf-4tk9y`, the test output shows:

```
test test_epoch_sync_init_event_carrying_subscriptions ... FAILED
```

### Concurrent Issues Found
During test runs, the following warnings were observed:
```
WARN hoop_daemon::beads: Quarantined malformed bead line 10 in /home/coding/HOOP/testrepo/.beads/issues.jsonl
WARN hoop_daemon::beads: Quarantined malformed bead line 11 in /home/coding/HOOP/testrepo/.beads/issues.jsonl
WARN hoop_daemon::beads: Quarantined malformed bead line 12 in /home/coding/HOOP/testrepo/.beads/issues.jsonl
```

### Process Termination Issue
Test runs show SIGTERM (exit code 143/144) indicating the test daemon process was terminated, likely due to:
- Timeout during test execution
- Resource constraints (OOM on the lab server)
- Process cleanup issues

## Specific Test Assertions

The test performs these specific checks (lines 46-67):
```rust
assert_eq!(event["type"], "init", "First message should be init event");
assert!(
    event["subscriptions"].is_array(),
    "init should contain subscriptions array"
);
assert!(
    subs.contains(&"global"),
    "global should always be in subscriptions"
);
```

## Investigation Challenges

Unable to run the test directly due to:
1. **Memory constraints**: Test compilation/execution gets killed (SIGTERM/SIGKILL) on the lab server
2. **Resource pressure**: Multiple concurrent rustc compilations consume significant memory
3. **Long-lived subprocesses**: Test daemon processes persist after failure, consuming resources

## Next Steps Required

1. **Fix test infrastructure** - Address process cleanup and memory issues preventing test execution
2. **Investigate WebSocket init event** - Once test can run, verify the actual init event payload structure
3. **Check subscriptions array** - Verify if the `subscriptions` field is missing or malformed
4. **Verify "global" subscription** - Check if "global" is properly included in the subscriptions list

## References

- Bead trace: `.beads/traces/bf-4tk9y/stdout.txt`
- Test file: `hoop-daemon/tests/epoch_sync_invariant.rs` (lines 22-71)
- Related bead: `bf-4tk9y` (previous test run investigation)
