# Test Failures Analysis - beads_deletion_http Tests

**Source:** bf-7vowz verification output  
**Analysis Date:** 2026-08-02  
**Status:** Tests BLOCKED - Compilation Failed

## Critical Finding

**The `beads_deletion_http` tests did NOT execute.** They were blocked by compilation failures in unrelated test files (`property_invariants.rs` and `draft_queue_invariants.rs`).

## Intended Test Names (from beads_deletion_http.rs)

1. `test_beads_deletion_readyz_degraded` - Verify /readyz reports degraded when .beads is deleted
2. `test_beads_deletion_sibling_events_continue` - Verify sibling projects continue serving events during degradation  
3. `test_readyz_response_format` - Verify /readyz response format is correct

**Execution Status:** ❌ BLOCKED - Never reached execution phase

## Compilation Errors (Blocking Test Execution)

### Total: 21 compilation errors across 2 test files

#### File: `hoop-daemon/tests/property_invariants.rs` (19 errors)

**Error Category 1: Proptest Strategy Issues (6 errors - E0434)**
```
error[E0434]: can't capture dynamic environment in a fn item
   --> hoop-daemon/tests/property_invariants.rs:657:49
    |
657 |                 events in prop::collection::vec(event_strategy, 0..20)
    |                                                 ^^^^^^^^^^^^^^
    |
    = help: use the `|| { ... }` closure form instead
```

**Locations:**
- Line 657: `events in prop::collection::vec(event_strategy, 0..20)`
- Line 732: `split_pos in 0..valid_event.len()`
- Line 735: `let chunk1 = &valid_event[..split_pos]`
- Line 736: `let chunk2 = &valid_event[split_pos..]`
- Line 765: `split_pos == 0 || split_pos == valid_event.len()`
- Line 821: `events in prop::collection::vec(event_strategy, 0..10)`

**Error Category 2: Missing File Import (9 errors - E0433)**
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

**Locations:**
- Line 670: `File::create(&events_path)`
- Line 678: `File::open(&events_path)`
- Line 828: `File::create(&events_path)`
- Line 838: `File::open(path)`
- Line 867: `File::create(&events_path)`
- Line 874: `File::open(&events_path)`
- Line 881: `File::create(&events_path)`
- Line 887: `File::open(&events_path)`
- Line 906: `File::create(&events_path)`
- Line 916: `File::open(&events_path)`

**Error Category 3: Missing BufRead Trait (1 error - E0599)**
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

**Error Category 4: Missing Trait Implementation (1 error - E0277)**
```
error[E0277]: the trait bound `StitchStatus: Hash` is not satisfied
   --> hoop-daemon/tests/property_invariants.rs:576:21
    |
576 |             results.insert(ctx.derive_status());
    |                     ^^^^^^ the trait `Hash` is not implemented for `StitchStatus`
    |
note: required by a bound in `std::collections::HashSet::<T, S, A>::insert`
```

**Error Category 5: Use After Move (1 error - E0382)**
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
    |
help: consider cloning the value if the performance cost is acceptable
    |
379 +             prop_assert_eq!(status1, status2.clone(), "First and second calls differ");
```

#### File: `hoop-daemon/tests/draft_queue_invariants.rs` (2 errors)

**Error Category: Missing Struct Fields (2 errors - E0063)**
```
error[E0063]: missing fields `abandoned_at`, `last_autosave_at`, `opened_at` and 2 other fields in initializer of `DraftRow`
   --> hoop-daemon/tests/draft_queue_invariants.rs:363:17
    |
363 |     let draft = hoop_daemon::fleet::DraftRow {
    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing `abandoned_at`, `last_autosave_at`, `opened_at` and 2 other fields
```

**Locations:**
- Line 363: DraftRow initializer
- Line 506: DraftRow initializer

## Root Cause Analysis

**Primary Issue:** Stale test fixtures

Production structs gained new fields that test initializers were never updated for:
- `DraftRow` now requires 5 additional fields: `abandoned_at`, `last_autosave_at`, `opened_at`, plus 2 others
- Missing imports in `property_invariants.rs`: `std::fs::File` and `std::io::BufRead`

## Impact on Test Execution

Since `cargo test` compiles all tests in the `hoop-daemon` test target before running any, compilation errors in **unrelated test files** blocked execution of the target `beads_deletion_http` tests.

**Consequence:** No test execution results available. The following cannot be determined:
- Pass/fail status for `test_beads_deletion_readyz_degraded`
- Pass/fail status for `test_beads_deletion_sibling_events_continue`  
- Pass/fail status for `test_readyz_response_format`
- Any runtime panics or timeouts
- Actual functional correctness of the beads deletion behavior

## Non-Blocking Warnings (Not Blocking Execution)

### hoop-daemon lib (12 warnings)
- Type privacy issue with `PatternCategory`
- Dead code: `openapi_router`, `load_hoop_config`, `check_and_emit_capacity_alert`
- Unused fields: `session_id`, `session_subpath`, `rpm_limit`, `subpath`
- Unused struct: `QuotaLimit`
- Unused constants: `MAX_UNASSIGNED_SESSIONS`, `STITCH_CLOSED_THRESHOLD_SECONDS`

### hoop-mcp tests (2 warnings)
- Unused mut variables in `forbidden_worker_steering.rs`

### hoop-cli tests (9 warnings)
- Unused variables, non-snake-case field name, unused functions

## Recommendations

1. **Immediate:** Fix compilation errors in `property_invariants.rs` and `draft_queue_invariants.rs`
2. **Then:** Re-run `beads_deletion_http` tests to get actual execution results
3. **Follow-up:** Address non-blocking warnings (dead code, unused imports)

## Conclusion

**No failing tests to report** - tests never reached execution phase. The bead bf-7vowz acceptance criteria (capturing pass/fail status, error messages, panics) cannot be met because compilation failed before any tests ran.