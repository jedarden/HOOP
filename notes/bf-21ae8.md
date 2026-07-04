# bead bf-21ae8: Risk Patterns Test Suite Run

## Task
Run the full risk_patterns test suite to verify no regressions after the fix.

## Execution
Attempted to run: `cargo test -p hoop-daemon risk_patterns::tests`

## Result
**BLOCKED: Compilation errors prevent test execution**

The test suite cannot run due to compilation errors in `hoop-daemon/tests/fleet_notifications_integration.rs`:

### Compilation Errors

1. **Private function access** (6 occurrences):
   ```
   error[E0624]: associated function `new` is private
   --> hoop-daemon/tests/fleet_notifications_integration.rs:18:73
   |
 18 |     let ring = hoop_daemon::fleet_notifications::FleetNotificationRing::new();
    |                                                                         ^^^ private associated function
   ```
   - Occurs on lines: 18, 80, 108, 190, 221
   - `FleetNotificationRing::new()` is defined as `fn new() -> Self` (private) in `hoop-daemon/src/fleet_notifications.rs:127`
   - Tests are trying to call it directly, but it's not accessible

2. **Missing test helper**:
   ```
   error[E0599]: no associated function or constant named `build_for_test` found for struct `ContextIndex`
   --> hoop-daemon/tests/fleet_notifications_integration.rs:178:59
   |
 178 |     let index = hoop_daemon::agent_context::ContextIndex::build_for_test(&yaml);
    |                                                           ^^^^^^^^^^^^^^ associated function not found
   ```
   - `ContextIndex::build_for_test()` does not exist
   - Only `ContextIndex::build()` is available (returns `Result<ContextIndex, anyhow::Error>`)

## Analysis
The risk_patterns tests themselves were not reached because the compilation fails first. The compilation errors appear to be in the `fleet_notifications_integration` test suite, which suggests:

1. `FleetNotificationRing::new()` was likely meant to be a test helper but was never made public
2. `ContextIndex::build_for_test()` either:
   - Was never implemented
   - Was removed during refactoring
   - Should be using `ContextIndex::build()` instead with proper error handling

## Required Actions
Before the risk_patterns test suite can run, these compilation errors must be resolved:

1. Make `FleetNotificationRing::new()` public or provide a test constructor
2. Implement `ContextIndex::build_for_test()` or update test to use `ContextIndex::build()` with proper Result handling

## Impact
- **risk_patterns tests cannot be executed** until compilation succeeds
- No regression verification possible in current state
- Test infrastructure needs repair before `bf-21ae8` can be completed

## Recommendation
Create bug bead(s) to fix the fleet_notifications_integration test infrastructure:
- Fix `FleetNotificationRing::new()` visibility
- Implement or replace `ContextIndex::build_for_test()`
