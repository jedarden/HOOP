# Test Failure Analysis - beads_deletion_http Tests

**Bead ID:** bf-wss0q
**Analysis Date:** 2026-08-02
**Source Bead:** bf-4l8jp (raw test output extraction)

## Executive Summary

The `beads_deletion_http` integration tests **did not execute**. All three tests were **BLOCKED** by compilation failures in unrelated test files within the same `hoop-daemon` test target.

**Root Cause:** Stale test fixtures - production structs gained new fields that test initializers were never updated for, plus missing imports in `property_invariants.rs`.

**Tests Blocked:**
1. `test_beads_deletion_readyz_degraded` - ❌ BLOCKED
2. `test_beads_deletion_sibling_events_continue` - ❌ BLOCKED  
3. `test_readyz_response_format` - ❌ BLOCKED

**Compilation Blockers:**
- `property_invariants.rs`: 19 compilation errors
- `draft_queue_invariants.rs`: 2 compilation errors

---

## Test 1: `test_beads_deletion_readyz_degraded`

### Test Purpose

Verifies **§6 Phase 2 success criterion**: "Killing one project's runtime (delete `.beads/`) shows an error card; other projects unaffected. /readyz reports degraded (A-listed)."

**Test scenario:**
1. Spawn daemon with 3 projects (A, B, C)
2. Delete project A's `.beads/` directory during runtime
3. Assert:
   - Project A's card shows error state within 30s
   - Projects B/C continue serving events normally
   - `/readyz` reports degraded (A-listed)
4. Restore `.beads/` and verify recovery

### Functionality Tested

| Component | What's Tested |
|-----------|---------------|
| **Graceful Degradation** | HOOP continues serving sibling projects when one project's `.beads/` is deleted |
| **Readiness Endpoint** | `/readyz` returns 503 + degraded status when a project is unhealthy |
| **Degraded Response Format** | Degraded list includes affected project name and non-healthy state |
| **Isolation** | Deleting one project's `.beads/` doesn't affect sibling projects |
| **Recovery** | Restoring `.beads/` triggers automatic recovery to healthy state |
| **API Consistency** | `/api/projects` reflects same state as `/readyz` |

### Failure Point

**Phase:** **COMPILATION BLOCKED** - Test never reached execution phase

**Blocking Location:** Test compilation failed before test binary could be linked

**Why Blocked:** The `cargo test` command compiles all tests in the `hoop-daemon` test target together. Compilation errors in `property_invariants.rs` and `draft_queue_invariants.rs` caused the entire test target to fail compilation, preventing `beads_deletion_http.rs` tests from being linked or executed.

### Error Messages

**Direct Test Errors:** None (test never executed)

**Blocking Compilation Errors:**

See complete compilation errors in the [Compilation Errors](#compilation-errors-section) section below.

The blocking errors are:

1. **property_invariants.rs (19 errors)**
   - E0434: Proptest environment capture issues (6 occurrences)
   - E0433: Missing `File` type import (9 occurrences)
   - E0599: Missing `BufRead` trait (1 occurrence)
   - E0277: Missing `Hash` trait on `StitchStatus` (1 occurrence)
   - E0382: Use after move error (1 occurrence)

2. **draft_queue_invariants.rs (2 errors)**
   - E0063: Missing fields in `DraftRow` initializer (2 occurrences)

### Stack Traces

**Runtime Stack Trace:** None (test never executed)

**Compilation Error Traces:**

```
error: could not compile `hoop-daemon` (test "property_invariants") due to 19 previous errors; 17 warnings emitted
error: could not compile `hoop-daemon` (test "draft_queue_invariants") due to 2 previous errors
```

Full compilation error traces are provided in the [Compilation Errors](#compilation-errors-section) section.

---

## Test 2: `test_beads_deletion_sibling_events_continue`

### Test Purpose

Verifies that **sibling projects continue serving events** while one project is degraded.

**Test scenario:**
1. Spawn daemon with 3 projects (A, B, C)
2. Delete project A's `.beads/` directory
3. Assert:
   - Sibling projects (B, C) remain operational
   - Metrics are still collected during degradation
   - API endpoints remain accessible
   - Beads can still be queried via API

### Functionality Tested

| Component | What's Tested |
|-----------|---------------|
| **Sibling Isolation** | Deleting project A's `.beads/` doesn't stop projects B or C |
| **API Availability** | All HTTP endpoints remain accessible during partial degradation |
| **Metrics Collection** | Metrics continue to be collected for healthy projects |
| **Query Functionality** | Beads API remains functional during degraded state |
| **No Cascading Failures** | One project's failure doesn't crash the daemon or block other projects |

### Failure Point

**Phase:** **COMPILATION BLOCKED** - Test never reached execution phase

**Blocking Location:** Same blocking compilation errors as Test 1

**Why Blocked:** Same root cause - `cargo test` compiles all tests in the target together, and the compilation failures in `property_invariants.rs` and `draft_queue_invariants.rs` prevented the entire test suite from building.

### Error Messages

**Direct Test Errors:** None (test never executed)

**Blocking Compilation Errors:** Same as Test 1

### Stack Traces

**Runtime Stack Trace:** None (test never executed)

**Compilation Error Traces:** Same as Test 1

---

## Test 3: `test_readyz_response_format`

### Test Purpose

Verifies that the **`/readyz` response format is correct** and matches the expected schema.

**Test scenario:**
1. Spawn daemon with default configuration
2. Assert:
   - `/readyz` returns 200 OK when healthy
   - Response body has correct `ReadinessResponse` schema
   - `status` field is `"ok"`
   - `degraded` list is empty when healthy

### Functionality Tested

| Component | What's Tested |
|-----------|---------------|
| **Response Schema** | `/readyz` returns valid `ReadinessResponse` JSON structure |
| **Success Status** | Healthy daemon returns 200 OK |
| **Status Field** | `status` field correctly set to `"ok"` when healthy |
| **Degraded List** | `degraded` array is empty when no projects are degraded |
| **JSON Serialization** | Response properly serializes to JSON |

### Failure Point

**Phase:** **COMPILATION BLOCKED** - Test never reached execution phase

**Blocking Location:** Same blocking compilation errors as Tests 1 and 2

**Why Blocked:** Same root cause - compilation failures in unrelated test files prevented the entire test target from building.

### Error Messages

**Direct Test Errors:** None (test never executed)

**Blocking Compilation Errors:** Same as Test 1

### Stack Traces

**Runtime Stack Trace:** None (test never executed)

**Compilation Error Traces:** Same as Test 1

---

## Compilation Errors Section <a name="compilation-errors-section"></a>

This section details all compilation errors that blocked the `beads_deletion_http` tests from executing.

### File 1: `hoop-daemon/tests/property_invariants.rs` (19 errors)

#### Error Category: Proptest Environment Capture (E0434) - 6 occurrences

**Error 1 - Line 657:**
```
error[E0434]: can't capture dynamic environment in a fn item
   --> hoop-daemon/tests/property_invariants.rs:657:49
    |
657 |                 events in prop::collection::vec(event_strategy, 0..20)
    |                                                 ^^^^^^^^^^^^^^
    |
    = help: use the `|| { ... }` closure form instead
```

**Error 2 - Line 732:**
```
error[E0434]: can't capture dynamic environment in a fn item
   --> hoop-daemon/tests/property_invariants.rs:732:33
    |
732 |                 split_pos in 0..valid_event.len()
    |                                 ^^^^^^^^^^^
    |
    = help: use the `|| { ... }` closure form instead
```

**Error 3 - Line 735:**
```
error[E0434]: can't capture dynamic environment in a fn item
   --> hoop-daemon/tests/property_invariants.rs:735:31
    |
735 |                 let chunk1 = &valid_event[..split_pos];
    |                               ^^^^^^^^^^^
    |
    = help: use the `|| { ... }` closure form instead
```

**Error 4 - Line 736:**
```
error[E0434]: can't capture dynamic environment in a fn item
   --> hoop-daemon/tests/property_invariants.rs:736:31
    |
736 |                 let chunk2 = &valid_event[split_pos..];
    |                               ^^^^^^^^^^^
    |
    = help: use the `|| { ... }` closure form instead
```

**Error 5 - Line 765:**
```
error[E0434]: can't capture dynamic environment in a fn item
   --> hoop-daemon/tests/property_invariants.rs:765:72
    |
765 |                 let split_at_boundary = split_pos == 0 || split_pos == valid_event.len();
    |                                                                        ^^^^^^^^^^^
    |
    = help: use the `|| { ... }` closure form instead
```

**Error 6 - Line 821:**
```
error[E0434]: can't capture dynamic environment in a fn item
   --> hoop-daemon/tests/property_invariants.rs:821:49
    |
821 |                 events in prop::collection::vec(event_strategy, 0..10)
    |                                                 ^^^^^^^^^^^^^^
    |
    = help: use the `|| { ... }` closure form instead
```

#### Error Category: Missing File Import (E0433) - 9 occurrences

**Error 7 - Line 670:**
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
    |
```

**Error 8 - Line 678:**
```
error[E0433]: cannot find type `File` in this scope
   --> hoop-daemon/tests/property_invariants.rs:678:28
    |
678 |                 let file = File::open(&events_path).unwrap();
    |                            ^^^^ use of undeclared type `File`
    |
help: consider importing this struct
    |
588 +     use std::fs::File;
    |
```

**Error 9 - Line 828:**
```
error[E0433]: cannot find type `File` in this scope
   --> hoop-daemon/tests/property_invariants.rs:828:36
    |
828 |                     let mut file = File::create(&events_path).unwrap();
    |                                    ^^^^ use of undeclared type `File`
    |
help: consider importing this struct
    |
588 +     use std::fs::File;
    |
```

**Error 10 - Line 838:**
```
error[E0433]: cannot find type `File` in this scope
   --> hoop-daemon/tests/property_invariants.rs:838:32
    |
838 |                 let file = File::open(path).unwrap();
    |                            ^^^^ use of undeclared type `File`
    |
help: consider importing this struct
    |
588 +     use std::fs::File;
    |
```

**Error 11 - Line 867:**
```
error[E0433]: cannot find type `File` in this scope
   --> hoop-daemon/tests/property_invariants.rs:867:28
    |
867 |             let mut file = File::create(&events_path).unwrap();
    |                                    ^^^^ use of undeclared type `File`
    |
help: consider importing this struct
    |
588 +     use std::fs::File;
    |
```

**Error 12 - Line 874:**
```
error[E0433]: cannot find type `File` in this scope
   --> hoop-daemon/tests/property_invariants.rs:874:24
    |
874 |             let file = File::open(&events_path).unwrap();
    |                            ^^^^ use of undeclared type `File`
    |
help: consider importing this struct
    |
588 +     use std::fs::File;
    |
```

**Error 13 - Line 881:**
```
error[E0433]: cannot find type `File` in this scope
   --> hoop-daemon/tests/property_invariants.rs:881:28
    |
881 |             let mut file = File::create(&events_path).unwrap();
    |                                    ^^^^ use of undeclared type `File`
    |
help: consider importing this struct
    |
588 +     use std::fs::File;
    |
```

**Error 14 - Line 887:**
```
error[E0433]: cannot find type `File` in this scope
   --> hoop-daemon/tests/property_invariants.rs:887:24
    |
887 |             let file = File::open(&events_path).unwrap();
    |                            ^^^^ use of undeclared type `File`
    |
help: consider importing this struct
    |
588 +     use std::fs::File;
    |
```

**Error 15 - Line 906:**
```
error[E0433]: cannot find type `File` in this scope
   --> hoop-daemon/tests/property_invariants.rs:906:28
    |
906 |             let mut file = File::create(&events_path).unwrap();
    |                                    ^^^^ use of undeclared type `File`
    |
help: consider importing this struct
    |
588 +     use std::fs::File;
    |
```

**Error 16 - Line 916:**
```
error[E0433]: cannot find type `File` in this scope
   --> hoop-daemon/tests/property_invariants.rs:916:20
    |
916 |         let file = File::open(&events_path).unwrap();
    |                    ^^^^ use of undeclared type `File`
    |
help: consider importing this struct
    |
588 +     use std::fs::File;
    |
```

#### Error Category: Missing BufRead Trait (E0599) - 1 occurrence

**Error 17 - Line 258:**
```
error[E0599]: no method named `lines` found for struct `std::io::BufReader<R>` in the current scope
   --> hoop-daemon/tests/property_invariants.rs:258:28
    |
258 |         for line in reader.lines() {
    |                            ^^^^^
    |
   --> /rustc/59807616e1fa2540724bfbac14d7976d7e4a3860/library/std/src/io/mod.rs:2702:7
    |
    = note: the method is available for `std::io::BufReader<std::fs::File>` here
    |
    = help: items from traits can only be used if the trait is in scope
help: trait `BufRead` which provides `lines` is implemented but not in scope; perhaps you want to import it
    |
 85 +     use std::io::BufRead;
    |
help: there is a method `byte_lines` with a similar name
    |
258 |         for line in reader.byte_lines() {
    |                            +++++
```

#### Error Category: Missing Trait Implementation (E0277) - 1 occurrence

**Error 18 - Line 576:**
```
error[E0277]: the trait bound `StitchStatus: Hash` is not satisfied
   --> hoop-daemon/tests/property_invariants.rs:576:21
    |
576 |             results.insert(ctx.derive_status());
    |                     ^^^^^^ the trait `Hash` is not implemented for `StitchStatus`
    |
note: required by a bound in `std::collections::HashSet::<T, S, A>::insert`
   --> /rustc/59807616e1fa2540724bfbac14d7976d7e4a3860/library/std/src/collections/hash/set.rs:995:4
```

#### Error Category: Use After Move (E0382) - 1 occurrence

**Error 19 - Line 380:**
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
    |                                             ++++++++
```

### File 2: `hoop-daemon/tests/draft_queue_invariants.rs` (2 errors)

#### Error Category: Missing Struct Fields (E0063) - 2 occurrences

**Error 20 - Line 363:**
```
error[E0063]: missing fields `abandoned_at`, `last_autosave_at`, `opened_at` and 2 other fields in initializer of `DraftRow`
   --> hoop-daemon/tests/draft_queue_invariants.rs:363:17
    |
363 |     let draft = hoop_daemon::fleet::DraftRow {
    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing `abandoned_at`, `last_autosave_at`, `opened_at` and 2 other fields
```

**Error 21 - Line 506:**
```
error[E0063]: missing fields `abandoned_at`, `last_autosave_at`, `opened_at` and 2 other fields in initializer of `DraftRow`
   --> hoop-daemon/tests/draft_queue_invariants.rs:506:17
    |
506 |     let draft = hoop_daemon::fleet::DraftRow {
    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing `abandoned_at`, `last_autosave_at`, `opened_at` and 2 other fields
```

### Final Build Status

```
error: could not compile `hoop-daemon` (test "property_invariants") due to 19 previous errors; 17 warnings emitted
error: could not compile `hoop-daemon` (test "draft_queue_invariants") due to 2 previous errors

For more information about this error, try `rustc --explain E0063`.
For more information about this error, try `rustc --explain E0277`.
For more information about this error, try `rustc --explain E0382`.
For more information about this error, try `rustc --explain E0433`.
For more information about this error, try `rustc --explain E0434`.
For more information about this error, try `rustc --explain E0599`.
```

---

## Summary

### Test Execution Status: BLOCKED - Compilation Failed

| Test Name | Purpose | Failure Phase | Error Count |
|----------|---------|---------------|-------------|
| `test_beads_deletion_readyz_degraded` | Graceful degradation + /readyz reporting | **COMPILATION BLOCKED** | N/A (never executed) |
| `test_beads_deletion_sibling_events_continue` | Sibling project isolation during degradation | **COMPILATION BLOCKED** | N/A (never executed) |
| `test_readyz_response_format` | /readyz response schema validation | **COMPILATION BLOCKED** | N/A (never executed) |

### Root Cause Analysis

**Primary Issue:** Stale test fixtures - production structs like `DraftRow` gained new fields that test initializers were never updated for.

**Secondary Issue:** Missing imports in `property_invariants.rs`:
- Missing `use std::fs::File;` import (9 uses)
- Missing `use std::io::BufRead;` import (1 use)
- Proptest strategy usage in `fn` items instead of closures (6 uses)
- `StitchStatus` missing `Hash` trait implementation (1 use)
- Use-after-move in property test assertions (1 use)

### Impact on Phase 1 Exit Gate

These compilation failures are part of the **Phase 1 CI gate (bead `bf-5mpcl`)** which requires:
- ✅ `cargo build --workspace` - **PASSES**
- ❌ `cargo test --workspace` - **FAILS** (compilation errors)
- ❌ `cargo clippy --workspace -- -D warnings` - **FAILS** (90 errors)
- ✅ `hoop status --json | jq .` - **PASSES**

### Next Steps

To unblock these tests, the following fixes are needed:

1. **property_invariants.rs:**
   - Add `use std::fs::File;` import
   - Add `use std::io::BufRead;` import
   - Convert proptest strategies to use closure form `|| { ... }`
   - Add `#[derive(Hash)]` to `StitchStatus` or use `.clone()` in assertions
   - Fix use-after-move with `.clone()`

2. **draft_queue_invariants.rs:**
   - Update `DraftRow` initializers to include new fields: `abandoned_at`, `last_autosave_at`, `opened_at`, and 2 others

Once these compilation errors are fixed, the `beads_deletion_http` tests will be able to execute and provide actual runtime test results.

### References

- **Source bead:** bf-4l8jp (raw test output extraction)
- **Original test attempt:** bf-7vowz
- **Plan reference:** §6 Phase 2 success criterion, §3.9
- **Related documentation:** `bf-4l8jp-raw-test-output-extracted.md`
