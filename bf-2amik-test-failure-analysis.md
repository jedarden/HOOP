# Test Failure Pattern Analysis - Bead bf-2amik

## Executive Summary

The `beads_deletion_http` integration tests **did not execute** - they were blocked by pre-existing compilation errors in unrelated test files within the same test target (`hoop-daemon` lib tests).

**Key Finding**: All three `beads_deletion_http` tests (`test_beads_deletion_readyz_degraded`, `test_beads_deletion_sibling_events_continue`, `test_readyz_response_format`) were blocked from running due to 21 compilation errors across two other test files: `property_invariants.rs` (19 errors) and `draft_queue_invariants.rs` (2 errors).

---

## Test-Specific Failure Analysis

### Test 1: `test_beads_deletion_readyz_degraded`

**File**: `hoop-daemon/tests/beads_deletion_http.rs:81-277`

**Functionality Being Tested**:
- Verifies §6 Phase 2 success criterion: "Killing one project's runtime (delete `.beads/`) shows an error card; other projects unaffected. /readyz reports degraded (A-listed)."
- Tests that deleting a project's `.beads/` directory during runtime:
  1. Causes the project to show error state within 30 seconds
  2. Reports degraded status via `/readyz` endpoint (HTTP 503)
  3. Lists the degraded project in the response
  4. Does NOT affect sibling projects (they remain healthy)
  5. Automatically recovers when `.beads/` is restored

**Failure Point**: **COMPILATION PHASE** - Test never reached execution

**Error Classification**: **DETERMINISTIC** - Compilation errors always fail the build

**Blocking Root Cause**: Compilation errors in `property_invariants.rs` and `draft_queue_invariants.rs` prevented the entire test target from compiling

**Detailed Test Steps** (what would have happened if not blocked):
1. Spawn daemon with 3 projects (project-a, project-b, project-c)
2. Wait for initial health check (all healthy)
3. Delete project-a's `.beads/` directory
4. Poll `/readyz` for up to 30 seconds expecting HTTP 503 with degraded status
5. Verify project-a is in degraded list
6. Verify projects b and c are NOT in degraded list
7. Restore `.beads/` and verify automatic recovery

**Expected Assertions** (never executed):
- Daemon becomes healthy initially within 10 seconds
- `/readyz` reports project-a as degraded within 30 seconds
- project-a state is not Healthy
- project-b and project-c are NOT in degraded list
- `/api/projects` reflects same state as `/readyz`
- Daemon recovers to healthy state after `.beads/` restoration

---

### Test 2: `test_beads_deletion_sibling_events_continue`

**File**: `hoop-daemon/tests/beads_deletion_http.rs:280-403`

**Functionality Being Tested**:
- Verifies that when one project's `.beads/` is deleted during runtime:
  1. Sibling projects continue serving events normally
  2. API endpoints remain accessible
  3. Metrics collection continues
  4. Beads can still be queried
  5. Sibling projects remain in Healthy/Starting state

**Failure Point**: **COMPILATION PHASE** - Test never reached execution

**Error Classification**: **DETERMINISTIC** - Compilation errors always fail the build

**Blocking Root Cause**: Compilation errors in `property_invariants.rs` and `draft_queue_invariants.rs` prevented the entire test target from compiling

**Detailed Test Steps** (what would have happened if not blocked):
1. Spawn daemon with 3 projects
2. Record baseline metrics
3. Delete project-a's `.beads/` directory
4. Wait for degraded status detection
5. Verify API still accessible (HTTP 200)
6. Verify metrics are still collected
7. Verify sibling projects are operational
8. Verify beads API still functional

**Expected Assertions** (never executed):
- project-a becomes degraded within 30 seconds
- `/api/projects` returns HTTP 200 during degradation
- Metrics are still collected (non-empty response)
- project-b and project-c remain Healthy/Starting
- `/api/beads` remains accessible (HTTP 200)

---

### Test 3: `test_readyz_response_format`

**File**: `hoop-daemon/tests/beads_deletion_http.rs:406-421`

**Functionality Being Tested**:
- Simple smoke test verifying `/readyz` endpoint returns valid response format
- Checks that status is "ok" and degraded list is empty when healthy

**Failure Point**: **COMPILATION PHASE** - Test never reached execution

**Error Classification**: **DETERMINISTIC** - Compilation errors always fail the build

**Blocking Root Cause**: Compilation errors in `property_invariants.rs` and `draft_queue_invariants.rs` prevented the entire test target from compiling

**Detailed Test Steps** (what would have happened if not blocked):
1. Spawn test daemon
2. GET `/readyz`
3. Parse JSON response
4. Verify status is "ok" and degraded list is empty

**Expected Assertions** (never executed):
- HTTP response is successful (2xx)
- Response body deserializes to `ReadinessResponse`
- `body.status == "ok"`
- `body.degraded.is_empty()`

---

## Blocking Compilation Errors

### File: `hoop-daemon/tests/property_invariants.rs` (19 errors)

#### Error Group 1: Proptest Strategy Closure Issues (E0434 - 6 occurrences)

**Lines**: 657, 732, 735, 736, 765, 821

**Error Pattern**:
```
error[E0434]: can't capture dynamic environment in a fn item
   --> hoop-daemon/tests/property_invariants.rs:657:49
    |
657 |                 events in prop::collection::vec(event_strategy, 0..20)
    |                                                 ^^^^^^^^^^^^^^
    |
    = help: use the `|| { ... }` closure form instead
```

**Root Cause**: Proptest strategies defined outside the `proptest! { }` macro cannot capture dynamic environment variables

**Affected Test Functions**:
- `proptest_replay_equals_live_inner` (lines 654-704)
- `proptest_replay_handles_partial_lines_inner` (lines 729-783)
- `proptest_replay_is_idempotent_inner` (lines 818-856)

**Fix Required**: Add `||` closure wrapper around strategy generation:
```rust
// Before (incorrect):
events in prop::collection::vec(event_strategy, 0..20)

// After (correct):
events in prop::collection::vec(|| event_strategy(), 0..20)
```

---

#### Error Group 2: Missing File Import (E0433 - 9 occurrences)

**Lines**: 670, 678, 828, 838, 867, 874, 881, 887, 906, 916

**Error Pattern**:
```
error[E0433]: cannot find type `File` in this scope
   --> hoop-daemon/tests/property_invariants.rs:670:36
    |
670 |                     let mut file = File::create(&events_path).unwrap();
    |                                    ^^^^ use of undeclared type `File`
    |
help: consider importing this struct
    |
588 +     use std::fs::File;
```

**Root Cause**: `std::fs::File` type used but not imported in test module

**Affected Code**: Multiple test functions using `File::create()` and `File::open()`

**Fix Required**: Add import at top of test module:
```rust
use std::fs::File;
```

---

#### Error Group 3: Missing BufRead Trait (E0599 - 1 occurrence)

**Line**: 258

**Error Pattern**:
```
error[E0599]: no method named `lines` found for struct `std::io::BufReader<R>` in the current scope
   --> hoop-daemon/tests/property_invariants.rs:258:28
    |
258 |         for line in reader.lines() {
    |                            ^^^^^
    |
help: trait `BufRead` which provides `lines` is implemented but not in scope
    |
 85 +     use std::io::BufRead;
```

**Root Cause**: `BufRead` trait required for `.lines()` method on `BufReader` but not imported

**Fix Required**: Add import:
```rust
use std::io::BufRead;
```

---

#### Error Group 4: StitchStatus Missing Hash Trait (E0277 - 1 occurrence)

**Line**: 576

**Error Pattern**:
```
error[E0277]: the trait bound `StitchStatus: Hash` is not satisfied
   --> hoop-daemon/tests/property_invariants.rs:576:21
    |
576 |             results.insert(ctx.derive_status());
    |                     ^^^^^^ the trait `Hash` is not implemented for `StitchStatus`
```

**Root Cause**: Test attempts to insert `StitchStatus` into `HashSet` but `StitchStatus` doesn't implement `Hash`

**Affected Code**: Property test for `derive_status()` purity

**Fix Required**: Either derive `Hash` on `StitchStatus` or use `Vec` to collect results instead of `HashSet`

---

#### Error Group 5: Use of Moved Value (E0382 - 1 occurrence)

**Line**: 380

**Error Pattern**:
```
error[E0382]: use of moved value: `status2`
   --> hoop-daemon/tests/property_invariants.rs:380:29
    |
375 |             let status2 = ctx.derive_status();
    |                 ------- move occurs because `status2` has type `StitchStatus`, which does not implement the `Copy` trait
...
379 |             prop_assert_eq!(status1, status2, "First and second calls differ");
    |                                      ------- value moved here
380 |             prop_assert_eq!(status2, status3, "Second and third calls differ");
    |                             ^^^^^^^ value used here after move
```

**Root Cause**: `StitchStatus` doesn't implement `Copy`, so first assertion consumes `status2`

**Fix Required**: Clone the value before first assertion:
```rust
prop_assert_eq!(status1, status2.clone(), "First and second calls differ");
```

---

### File: `hoop-daemon/tests/draft_queue_invariants.rs` (2 errors)

#### Error Group 6: DraftRow Missing Fields (E0063 - 2 occurrences)

**Lines**: 363, 506

**Error Pattern**:
```
error[E0063]: missing fields `abandoned_at`, `last_autosave_at`, `opened_at` and 2 other fields in initializer of `DraftRow`
   --> hoop-daemon/tests/draft_queue_invariants.rs:363:17
    |
363 |     let draft = hoop_daemon::fleet::DraftRow {
    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing `abandoned_at`, `last_autosave_at`, `opened_at` and 2 other fields
```

**Root Cause**: Production struct `DraftRow` gained new fields after test fixtures were written

**Affected Test Functions**:
- Two test functions attempting to construct `DraftRow` for testing

**Missing Fields** (at least 5):
- `abandoned_at`
- `last_autosave_at`
- `opened_at`
- 2 other fields not listed in error message

**Fix Required**: Update test fixture to include all required fields with appropriate test values (likely `None`, empty strings, or default timestamps)

---

## Compilation Warnings (Non-Blocking)

### hoop-daemon lib (12 warnings)
- Private interface visibility warning (`PatternCategory`)
- Dead code warnings (unused functions/structs/fields)
- Not blocking tests but should be cleaned up

### hoop-mcp tests (2 warnings)
- Unused `mut` variables in test setup

### hoop-cli tests (9 warnings)
- Unused variables and imports
- Non-snake-case field naming

---

## Impact Assessment

### Test Execution Status: **BLOCKED - No Tests Ran**

All three `beads_deletion_http` tests failed to reach the execution phase. The compilation target `hoop-daemon` lib tests includes:
- `property_invariants.rs` (19 errors)
- `draft_queue_invariants.rs` (2 errors)
- `beads_deletion_http.rs` (target tests - 0 compilation errors but blocked)

Because Rust compiles all tests in a target before running any, errors in unrelated test files blocked execution of the target tests.

### Deterministic vs Flaky Classification

**All failures are DETERMINISTIC**:
- Compilation errors always fail the build
- No test execution occurred, so no flaky runtime behavior possible
- These errors will block tests 100% of the time until fixed

---

## Required Fixes Before Tests Can Run

### Priority 1: Unblock Test Execution (21 errors)

1. **property_invariants.rs** (19 errors):
   - Add `use std::fs::File;` import (1 line)
   - Add `use std::io::BufRead;` import (1 line)
   - Wrap proptest strategies in closures: `|| event_strategy()` (6 locations)
   - Derive `Hash` on `StitchStatus` or use `Vec` instead of `HashSet` (design decision)
   - Add `.clone()` to moved value assertion (1 location)

2. **draft_queue_invariants.rs** (2 errors):
   - Update `DraftRow` test fixtures to include 5 missing fields (2 locations)

### Priority 2: Clean Up Warnings (23 warnings)
- Remove dead code or add `#[allow(dead_code)]`
- Prefix unused variables with underscore
- Fix non-snake-case naming

---

## Test Execution Timeline (Expected After Fixes)

Once compilation errors are fixed:

1. **Compilation**: ~30-60 seconds for `hoop-daemon` test target
2. **Test Execution**:
   - `test_readyz_response_format`: ~5 seconds (smoke test)
   - `test_beads_deletion_readyz_degraded`: ~60-90 seconds (full degradation cycle with recovery)
   - `test_beads_deletion_sibling_events_continue`: ~45 seconds (degradation without recovery)
3. **Total Runtime**: ~2-3 minutes for all three tests

---

## Recommendations

### Immediate Action Required

1. **Fix compilation errors first** - Tests cannot run until all 21 errors are resolved
2. **Run tests incrementally** - After fixing errors, run each test individually to verify they work
3. **Add test dependency isolation** - Consider splitting test files into separate compilation targets to prevent unrelated errors from blocking

### Long-term Improvements

1. **CI pre-check** - Add `cargo check --workspace` to CI before test steps to catch compilation errors early
2. **Test fixture maintenance** - Review all test fixtures quarterly to ensure they track production struct changes
3. **Compilation test grouping** - Group property tests separately to prevent them from blocking integration tests

---

## Detailed Error Code Reference

### Error Code Summary

| Error Code | Count | Description | Fix Complexity |
|------------|-------|-------------|----------------|
| E0434 | 6 | Proptest closure capture | Low - add `||` wrapper |
| E0433 | 9 | Missing `File` type import | Low - add 1 import line |
| E0599 | 1 | Missing `BufRead` trait | Low - add 1 import line |
| E0277 | 1 | Missing `Hash` trait on `StitchStatus` | Medium - requires production code change |
| E0382 | 1 | Use of moved value | Low - add `.clone()` |
| E0063 | 2 | Missing struct fields | Low - update test fixtures |

### Compiler Error Explanations

**E0434** (Proptest closure capture): Rust's `fn` items cannot capture variables from their environment. Proptest strategies need closure form (`|| { ... }`) to capture dynamic environment variables.

**E0433** (Missing type): The `File` type lives in `std::fs` but isn't imported. Rust doesn't pre-import common types like some other languages.

**E0599** (Missing trait method): The `.lines()` method comes from the `BufRead` trait, which must be in scope. Even though `BufReader` implements `BufRead`, the trait must be imported to use its methods.

**E0277** (Trait bound not satisfied): The `HashSet<T>` type requires `T: Hash`. The test tries to insert `StitchStatus` into a `HashSet`, but `StitchStatus` doesn't derive `Hash`.

**E0382** (Use of moved value): Rust's ownership system means values are moved (not copied) by default. The first assertion consumes `status2`, making it unavailable for the second assertion.

**E0063** (Missing struct fields): Rust requires all struct fields to be initialized. When `DraftRow` gained new fields in production, the test fixtures became incomplete.

### Test File Line Number Mapping

**property_invariants.rs errors:**
- Lines 258: Missing `BufRead` trait
- Lines 363, 380: `StitchStatus` trait/move issues
- Lines 657, 670, 678: First proptest test closure and File issues
- Lines 732, 735, 736: Second proptest test closure issues
- Lines 821, 828, 838, 867, 874, 881, 887, 906, 916: Third proptest test closure and File issues

**draft_queue_invariants.rs errors:**
- Line 363: First `DraftRow` initialization with missing fields
- Line 506: Second `DraftRow` initialization with missing fields

### Test Function Names Mapping

From the error locations:
- **Lines 654-704**: `proptest_replay_equals_live_inner` - Event replay correctness test
- **Lines 729-783**: `proptest_replay_handles_partial_lines_inner` - Partial line handling test
- **Lines 818-856**: `proptest_replay_is_idempotent_inner` - Replay idempotence test
- **Line 363 area**: Draft insertion test (verifies no beads created)
- **Line 506 area**: Draft state test (verifies draft persistence)

These property tests verify critical HOOP invariants for event stream processing and Stitch derivation, but they all fail at **compilation phase**, never reaching execution.

---

## Conclusion

The `beads_deletion_http` tests are **well-written integration tests** covering critical Phase 2 success criteria, but they are **completely blocked** by pre-existing compilation errors in unrelated test files.

The tests themselves have 0 compilation errors - the blocking is entirely from `property_invariants.rs` (19 errors) and `draft_queue_invariants.rs` (2 errors). Once the 21 compilation errors are fixed, these tests should run successfully and provide valuable coverage of the beads deletion degradation scenario.

**Test Quality Assessment**: The target tests are properly structured with:
- Clear test names describing functionality
- Comprehensive assertion coverage
- Appropriate timeouts (30s for degradation detection)
- Cleanup verification (recovery after restoration)
- Sibling project isolation testing

**Status**: Tests cannot be evaluated for pass/fail until compilation blockers are resolved. All 21 errors are deterministic and will 100% block test execution until fixed.
